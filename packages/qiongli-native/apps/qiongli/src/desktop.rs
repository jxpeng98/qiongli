use qiongli_config::{
    ConfigError, ConfigState, EmailAddress, GlobalSettings, ProviderReadiness,
    RedactedConfigStatus, RedactedProviderStatus, UnavailableSecretStore,
};
use qiongli_content::{
    EmbeddedContent, MaterializationReceiptV1, MaterializationTarget, ProfileId,
    approve_materialization_target, remove_materialization, verify_materialization,
};
use qiongli_platform::{
    ApprovalRequirement, Architecture, ClaudeAdapterError, ClaudeMarketplaceState,
    ClaudeRegistrationState, ClaudeSkillsPluginState, ClaudeSourceState,
    ClientActivationCoordinator, ClientActivationDisposition, ClientActivationHandle,
    ClientActivationPreview, ClientActivationTarget, CodexAdapterError, CodexMarketplaceState,
    CodexRegistrationState, CodexSourceState, InstallPlanMetadataV1, OperatingSystem,
    TrustedPublicKey, VerifiedLaunchGrant, VerifiedNativeReleaseCandidate,
    apply_native_release_candidate_local, approve_install_plan, discover_claude_user_with_config,
    discover_codex_user, preview_client_activation,
};
use qiongli_runtime::mcp::{LiteMcpServer, MCP_PROTOCOL_VERSION};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{LITE_PUBLIC_TOOL_NAMES, LiteToolRegistry};
use qiongli_ui::{
    ActivationPolicy, ArchitectureView, CapabilityView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, GlobalSettingsPatch,
    IntegrationDiscoveryState, IntegrationTarget, IntegrationView, McpSelfTestCheckId,
    McpSelfTestCheckView, McpSelfTestState, McpSelfTestView, McpView, OperatingSystemView,
    OperationApproval, OperationKind, OperationPreview, OperationToken, PrivateDisplayText,
    ProductView, ProfileKind, ProfileView, ProviderKind, ProviderReadinessView, ProviderView,
    PublicSettingChange, RemediationCode, StatusCode, SymbolicLocation,
};
use serde_json::json;
use sha2::{Digest, Sha256};

use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::command::{CommandEnvironment, config_root, config_store};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLaunchError;

const ACTIVATION_PLAN_TTL_SECONDS: u64 = 600;
const MCP_SELF_TEST_TIMEOUT: Duration = Duration::from_secs(5);
const ACTIVATION_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];

pub fn run_desktop(
    environment: CommandEnvironment,
    content: EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    run_desktop_with_activation_sessions(environment, content, Vec::new())
}

pub fn run_desktop_with_activation_sessions(
    environment: CommandEnvironment,
    content: EmbeddedContent,
    sessions: Vec<DesktopActivationSession>,
) -> Result<(), DesktopLaunchError> {
    if sessions.len() > 2
        || sessions.iter().enumerate().any(|(index, session)| {
            sessions[..index]
                .iter()
                .any(|prior| prior.target == session.target)
        })
    {
        return Err(DesktopLaunchError);
    }
    let service = NativeDesktopService::new(environment, content, sessions);
    qiongli_ui::run_native_application(crate::desktop_application_metadata(), Box::new(service))
        .map_err(|_| DesktopLaunchError)
}

pub fn run_desktop_with_candidate_sessions(
    environment: CommandEnvironment,
    content: EmbeddedContent,
    sessions: Vec<DesktopCandidateSession>,
) -> Result<(), DesktopLaunchError> {
    if sessions.len() > 2
        || sessions.iter().enumerate().any(|(index, session)| {
            sessions[..index]
                .iter()
                .any(|prior| prior.target == session.target)
        })
    {
        return Err(DesktopLaunchError);
    }
    let service = NativeDesktopService::new_with_candidate_sessions(environment, content, sessions);
    qiongli_ui::run_native_application(crate::desktop_application_metadata(), Box::new(service))
        .map_err(|_| DesktopLaunchError)
}

pub(crate) fn validate_desktop_startup(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    let _window_entrypoint: fn(
        CommandEnvironment,
        EmbeddedContent,
    ) -> Result<(), DesktopLaunchError> = run_desktop;
    let owned_content = crate::embedded_content().map_err(|_| DesktopLaunchError)?;
    if owned_content.pack().pack_sha256() != content.pack().pack_sha256() {
        return Err(DesktopLaunchError);
    }
    let mut service = NativeDesktopService::new(environment.clone(), owned_content, Vec::new());
    service
        .snapshot()
        .validate()
        .map_err(|_| DesktopLaunchError)?;
    let _app = qiongli_ui::QiongliDesktopApp::new(Box::new(service));
    Ok(())
}

pub struct DesktopActivationSession {
    target: IntegrationTarget,
    handle: ClientActivationHandle,
    grant: VerifiedLaunchGrant,
    trusted_keys: Vec<TrustedPublicKey>,
    minimum_generation: u64,
    pending: Option<ClientActivationPreview>,
}

impl DesktopActivationSession {
    #[must_use]
    pub fn new(
        handle: ClientActivationHandle,
        grant: VerifiedLaunchGrant,
        trusted_keys: Vec<TrustedPublicKey>,
        minimum_generation: u64,
    ) -> Self {
        let target = integration_target(handle.target());
        Self {
            target,
            handle,
            grant,
            trusted_keys,
            minimum_generation,
            pending: None,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        now_unix: u64,
    ) -> Result<OperationPreview, &'static str> {
        let expires_at_unix = now_unix
            .saturating_add(ACTIVATION_PLAN_TTL_SECONDS)
            .min(self.grant.grant().expires_at_unix);
        let plan_id = match self.target {
            IntegrationTarget::Codex => "desktop-activate-codex",
            IntegrationTarget::ClaudeCode => "desktop-activate-claude-code",
        };
        let preview = preview_client_activation(
            &self.handle,
            InstallPlanMetadataV1 {
                plan_id: plan_id.to_owned(),
                created_at_unix: now_unix,
                expires_at_unix,
            },
            &self.grant,
            &self.trusted_keys,
            self.minimum_generation,
            now_unix,
        )
        .map_err(|error| error.reason_code())?;
        let digest = preview.plan().plan().semantic_digest_sha256.clone();
        self.pending = Some(preview);
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match self.target {
                IntegrationTarget::Codex => "Codex activation preview",
                IntegrationTarget::ClaudeCode => "Claude Code activation preview",
            },
            summary: "Register the verified local Qiongli source. Client-owned enablement remains a host action.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(&mut self, now_unix: u64) -> Result<&'static str, &'static str> {
        let preview = self
            .pending
            .take()
            .ok_or("desktop-activation-preview-missing")?;
        let approval = approve_install_plan(preview.plan(), &ACTIVATION_APPROVALS, now_unix)
            .map_err(|error| error.reason_code())?;
        let commit = ClientActivationCoordinator::new(self.handle.clone())
            .apply(&preview, &approval, now_unix)
            .map_err(|error| error.reason_code())?;
        Ok(match commit.disposition {
            ClientActivationDisposition::Activated => "client-activation-applied",
            ClientActivationDisposition::AlreadyActive => "client-activation-already-active",
            ClientActivationDisposition::Repaired => "client-activation-repaired",
            ClientActivationDisposition::AlreadyHealthy => "client-activation-already-healthy",
        })
    }

    fn cancel(&mut self) {
        self.pending = None;
    }
}

impl Debug for DesktopActivationSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopActivationSession")
            .field("target", &self.target)
            .field("pending", &self.pending.is_some())
            .finish()
    }
}

pub struct DesktopCandidateSession {
    target: IntegrationTarget,
    candidate: VerifiedNativeReleaseCandidate,
    pending: bool,
}

impl DesktopCandidateSession {
    #[must_use]
    pub fn new(candidate: VerifiedNativeReleaseCandidate) -> Self {
        Self {
            target: integration_target(candidate.target()),
            candidate,
            pending: false,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        now_unix: u64,
    ) -> Result<OperationPreview, &'static str> {
        if now_unix < self.candidate.candidate().not_before_unix {
            return Err("native-release-candidate-not-yet-valid");
        }
        if now_unix >= self.candidate.candidate().expires_at_unix {
            return Err("native-release-candidate-expired");
        }
        self.pending = true;
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match self.target {
                IntegrationTarget::Codex => "Codex candidate installation preview",
                IntegrationTarget::ClaudeCode => "Claude Code candidate installation preview",
            },
            summary: "Install the verified native payload and fixed local Qiongli source, then register it. Client-owned enablement remains a host action.",
            display_target: None,
            plan_digest_sha256: Some(crate::candidate_cli::candidate_approval_digest(
                self.candidate.signed_payload_sha256(),
                self.candidate.target(),
            )),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(
        &mut self,
        content: &EmbeddedContent,
        home: &std::path::Path,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        if !std::mem::take(&mut self.pending) {
            return Err("desktop-candidate-preview-missing");
        }
        let commit =
            apply_native_release_candidate_local(content.pack(), &self.candidate, home, now_unix)
                .map_err(|error| error.reason_code())?;
        Ok(match commit.payload.disposition {
            qiongli_platform::InstallDisposition::Applied => "native-candidate-install-applied",
            qiongli_platform::InstallDisposition::AlreadyApplied => {
                "native-candidate-install-already-applied"
            }
            qiongli_platform::InstallDisposition::Repaired => "native-candidate-install-repaired",
            qiongli_platform::InstallDisposition::AlreadyHealthy => {
                "native-candidate-install-already-healthy"
            }
        })
    }

    fn cancel(&mut self) {
        self.pending = false;
    }
}

impl Debug for DesktopCandidateSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopCandidateSession")
            .field("target", &self.target)
            .field("candidate_digest", &"<verified-candidate>")
            .field("pending", &self.pending)
            .finish()
    }
}

#[derive(Clone, Copy)]
struct McpSelfTestCounts {
    enabled_providers: usize,
    ready_providers: usize,
    discovered_clients: usize,
    registered_clients: usize,
}

struct McpSelfTestInput {
    server: LiteMcpServer,
    counts: McpSelfTestCounts,
}

trait McpSelfTestExecutor: Send + Sync {
    fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView;
}

struct NativeMcpSelfTestExecutor;

impl McpSelfTestExecutor for NativeMcpSelfTestExecutor {
    fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView {
        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }

        let initialize = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }));
        let initialize_ready = initialize.as_ref().is_some_and(|response| {
            response
                .pointer("/result/protocolVersion")
                .and_then(|value| value.as_str())
                == Some(MCP_PROTOCOL_VERSION)
                && response.pointer("/result/capabilities/tools").is_some()
        });

        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }
        let tools = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }));
        let tools_ready = tools.as_ref().is_some_and(|response| {
            response
                .pointer("/result/tools")
                .and_then(|value| value.as_array())
                .is_some_and(|tools| {
                    tools.len() == LITE_PUBLIC_TOOL_NAMES.len()
                        && tools
                            .iter()
                            .zip(LITE_PUBLIC_TOOL_NAMES)
                            .all(|(tool, expected)| {
                                tool.get("name").and_then(|value| value.as_str()) == Some(expected)
                            })
                })
        });

        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }
        let offline_dispatch = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "qiongli_task_plan",
                "arguments": {
                    "task_id": "desktop-self-test",
                    "paper_type": "review",
                    "topic": "offline self-test"
                }
            }
        }));
        let offline_ready = offline_dispatch.as_ref().is_some_and(|response| {
            response.get("result").is_some() && response.get("error").is_none()
        });

        let provider_check = if input.counts.enabled_providers == 0 {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Attention,
                "no-provider-enabled",
            )
        } else if input.counts.ready_providers == input.counts.enabled_providers {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Ready,
                "enabled-providers-ready",
            )
        } else {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Attention,
                "provider-configuration-attention",
            )
        };
        let client_check = if input.counts.discovered_clients == 0 {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Attention,
                "no-client-discovered",
            )
        } else if input.counts.registered_clients == input.counts.discovered_clients {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Ready,
                "discovered-clients-registered",
            )
        } else {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Attention,
                "client-registration-attention",
            )
        };
        let checks = [
            mcp_check(
                McpSelfTestCheckId::EmbeddedContract,
                StatusCode::Ready,
                "embedded-contract-ready",
            ),
            mcp_check(
                McpSelfTestCheckId::Initialize,
                if initialize_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if initialize_ready {
                    "mcp-initialize-ready"
                } else {
                    "mcp-initialize-invalid"
                },
            ),
            mcp_check(
                McpSelfTestCheckId::ToolRegistry,
                if tools_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if tools_ready {
                    "exact-tool-registry-ready"
                } else {
                    "tool-registry-drifted"
                },
            ),
            mcp_check(
                McpSelfTestCheckId::OfflineDispatch,
                if offline_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if offline_ready {
                    "offline-dispatch-ready"
                } else {
                    "offline-dispatch-failed"
                },
            ),
            provider_check,
            client_check,
        ];
        let state = if initialize_ready && tools_ready && offline_ready {
            McpSelfTestState::Passed
        } else {
            McpSelfTestState::Failed
        };
        mcp_self_test_view(state, checks, input.counts)
    }
}

struct ActiveMcpSelfTest {
    receiver: mpsc::Receiver<McpSelfTestView>,
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
    running: McpSelfTestView,
}

fn mcp_check(
    check: McpSelfTestCheckId,
    status: StatusCode,
    code: &'static str,
) -> McpSelfTestCheckView {
    McpSelfTestCheckView {
        check,
        status,
        code,
        remediation: mcp_check_remediation(check, status),
    }
}

const fn mcp_check_remediation(check: McpSelfTestCheckId, status: StatusCode) -> &'static str {
    if matches!(status, StatusCode::Ready | StatusCode::Missing) {
        return "none";
    }
    match check {
        McpSelfTestCheckId::EmbeddedContract | McpSelfTestCheckId::ToolRegistry => {
            "reinstall-qiongli"
        }
        McpSelfTestCheckId::Initialize => "upgrade-qiongli",
        McpSelfTestCheckId::OfflineDispatch => "retry-mcp-self-test",
        McpSelfTestCheckId::ProviderReadiness => "configure-enabled-providers",
        McpSelfTestCheckId::ClientRegistration => "refresh-integration-discovery",
    }
}

fn mcp_self_test_view(
    state: McpSelfTestState,
    checks: [McpSelfTestCheckView; 6],
    counts: McpSelfTestCounts,
) -> McpSelfTestView {
    McpSelfTestView {
        state,
        checks,
        public_tool_count: LITE_PUBLIC_TOOL_NAMES.len(),
        enabled_provider_count: counts.enabled_providers,
        ready_provider_count: counts.ready_providers,
        discovered_client_count: counts.discovered_clients,
        registered_client_count: counts.registered_clients,
    }
}

fn pending_mcp_self_test(counts: McpSelfTestCounts) -> McpSelfTestView {
    mcp_self_test_view(
        McpSelfTestState::Running,
        McpSelfTestCheckId::ALL
            .map(|check| mcp_check(check, StatusCode::Missing, "self-test-check-pending")),
        counts,
    )
}

fn terminal_mcp_self_test(state: McpSelfTestState, counts: McpSelfTestCounts) -> McpSelfTestView {
    let (status, code) = match state {
        McpSelfTestState::Cancelled => (StatusCode::Missing, "self-test-cancelled"),
        McpSelfTestState::TimedOut => (StatusCode::Blocked, "self-test-timed-out"),
        McpSelfTestState::Failed => (StatusCode::Invalid, "self-test-failed"),
        McpSelfTestState::Running | McpSelfTestState::Passed => {
            (StatusCode::Invalid, "self-test-state-invalid")
        }
    };
    let mut view = mcp_self_test_view(
        state,
        McpSelfTestCheckId::ALL.map(|check| mcp_check(check, status, code)),
        counts,
    );
    for check in &mut view.checks {
        check.remediation = if state == McpSelfTestState::Cancelled {
            "none"
        } else {
            "retry-mcp-self-test"
        };
    }
    view
}

fn contract_failure_mcp_self_test(counts: McpSelfTestCounts) -> McpSelfTestView {
    let mut view = terminal_mcp_self_test(McpSelfTestState::Failed, counts);
    view.public_tool_count = 0;
    view.checks[0] = mcp_check(
        McpSelfTestCheckId::EmbeddedContract,
        StatusCode::Invalid,
        "embedded-contract-invalid",
    );
    view
}

trait FolderPicker {
    fn pick_folder(&mut self) -> Option<PathBuf>;
}

struct NativeFolderPicker;

impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a Qiongli Skills destination")
            .pick_folder()
    }
}

struct NativeDesktopService {
    environment: CommandEnvironment,
    content: EmbeddedContent,
    active_operation: Option<PendingDesktopOperation>,
    folder_picker: Box<dyn FolderPicker>,
    selected_skills_target: Option<MaterializationTarget>,
    mcp_self_test: Option<ActiveMcpSelfTest>,
    mcp_self_test_executor: Arc<dyn McpSelfTestExecutor>,
    mcp_self_test_timeout: Duration,
    activation_sessions: Vec<DesktopActivationSession>,
    candidate_sessions: Vec<DesktopCandidateSession>,
}

enum PendingDesktopOperation {
    Blocked(OperationToken),
    GlobalSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    SkillsMaterialization {
        token: OperationToken,
        profile: ProfileKind,
        target: MaterializationTarget,
    },
    SkillsRemoval {
        token: OperationToken,
        target: MaterializationTarget,
        expected_receipt: MaterializationReceiptV1,
    },
    Activation {
        token: OperationToken,
        target: IntegrationTarget,
    },
    Candidate {
        token: OperationToken,
        target: IntegrationTarget,
    },
}

impl Drop for NativeDesktopService {
    fn drop(&mut self) {
        if let Some(active) = &self.mcp_self_test {
            active.cancelled.store(true, Ordering::Release);
        }
    }
}

impl PendingDesktopOperation {
    const fn token(&self) -> OperationToken {
        match self {
            Self::Blocked(token)
            | Self::GlobalSettings { token, .. }
            | Self::SkillsMaterialization { token, .. }
            | Self::SkillsRemoval { token, .. }
            | Self::Activation { token, .. }
            | Self::Candidate { token, .. } => *token,
        }
    }
}

impl NativeDesktopService {
    fn new(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        activation_sessions: Vec<DesktopActivationSession>,
    ) -> Self {
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker: Box::new(NativeFolderPicker),
            selected_skills_target: None,
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            activation_sessions,
            candidate_sessions: Vec::new(),
        }
    }

    fn new_with_candidate_sessions(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        candidate_sessions: Vec<DesktopCandidateSession>,
    ) -> Self {
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker: Box::new(NativeFolderPicker),
            selected_skills_target: None,
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            activation_sessions: Vec::new(),
            candidate_sessions,
        }
    }

    #[cfg(test)]
    fn new_with_folder_picker(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        folder_picker: Box<dyn FolderPicker>,
    ) -> Self {
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker,
            selected_skills_target: None,
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            activation_sessions: Vec::new(),
            candidate_sessions: Vec::new(),
        }
    }

    fn start_mcp_self_test(&mut self) -> DesktopEvent {
        if let Some(active) = &self.mcp_self_test {
            return DesktopEvent::McpSelfTestUpdated(active.running.clone());
        }
        let snapshot = build_snapshot(&self.environment, &self.content);
        let counts = mcp_self_test_counts(&snapshot);
        let registry = match LiteToolRegistry::from_embedded_content(&self.content) {
            Ok(registry) => registry,
            Err(_) => {
                return DesktopEvent::McpSelfTestUpdated(contract_failure_mcp_self_test(counts));
            }
        };
        let server = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => {
                let access =
                    ProviderAccess::from_global_settings(&loaded.settings, &UnavailableSecretStore);
                LiteMcpServer::production("qiongli", env!("CARGO_PKG_VERSION"), registry, access)
            }
            Err(_) => {
                LiteMcpServer::config_unavailable("qiongli", env!("CARGO_PKG_VERSION"), registry)
            }
        };
        let running = pending_mcp_self_test(counts);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let executor = Arc::clone(&self.mcp_self_test_executor);
        let (sender, receiver) = mpsc::sync_channel(1);
        let spawn = thread::Builder::new()
            .name("qiongli-mcp-self-test".to_owned())
            .spawn(move || {
                let result = executor.run(McpSelfTestInput { server, counts }, worker_cancelled);
                let _ = sender.send(result);
            });
        if spawn.is_err() {
            return DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                McpSelfTestState::Failed,
                counts,
            ));
        }
        let now = Instant::now();
        self.mcp_self_test = Some(ActiveMcpSelfTest {
            receiver,
            cancelled,
            deadline: now.checked_add(self.mcp_self_test_timeout).unwrap_or(now),
            running: running.clone(),
        });
        DesktopEvent::McpSelfTestUpdated(running)
    }

    fn poll_mcp_self_test(&mut self) -> DesktopEvent {
        let Some(active) = self.mcp_self_test.as_ref() else {
            return DesktopEvent::Failed {
                code: "mcp-self-test-not-running",
            };
        };
        if Instant::now() >= active.deadline {
            let active = self
                .mcp_self_test
                .take()
                .expect("validated MCP self-test remains active");
            active.cancelled.store(true, Ordering::Release);
            return DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                McpSelfTestState::TimedOut,
                mcp_counts_from_view(&active.running),
            ));
        }
        match active.receiver.try_recv() {
            Ok(result) => {
                self.mcp_self_test = None;
                DesktopEvent::McpSelfTestUpdated(result)
            }
            Err(mpsc::TryRecvError::Empty) => {
                DesktopEvent::McpSelfTestUpdated(active.running.clone())
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                let counts = mcp_counts_from_view(&active.running);
                self.mcp_self_test = None;
                DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                    McpSelfTestState::Failed,
                    counts,
                ))
            }
        }
    }

    fn cancel_mcp_self_test(&mut self) -> DesktopEvent {
        let Some(active) = self.mcp_self_test.take() else {
            return DesktopEvent::Failed {
                code: "mcp-self-test-not-running",
            };
        };
        active.cancelled.store(true, Ordering::Release);
        DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
            McpSelfTestState::Cancelled,
            mcp_counts_from_view(&active.running),
        ))
    }

    fn next_operation_token() -> Result<OperationToken, &'static str> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| "operation-token-unavailable")?;
        Ok(OperationToken::new(u128::from_le_bytes(bytes)))
    }

    fn issue_preview(
        &mut self,
        title: &'static str,
        summary: &'static str,
        blocked_reason: &'static str,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        self.active_operation = Some(PendingDesktopOperation::Blocked(token));
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::GlobalSettings,
            title,
            summary,
            display_target: None,
            plan_digest_sha256: None,
            approvals_required: Vec::new(),
            can_confirm: false,
            blocked_reason: Some(blocked_reason),
        })
    }

    fn preview_global_settings(&mut self, patch: GlobalSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }

        let mut replacement = loaded.settings.clone();
        replacement.default_profile = profile_to_content(patch.default_profile);
        replacement.providers.openalex.enabled = patch.providers_enabled[0];
        replacement.providers.semantic_scholar.enabled = patch.providers_enabled[1];
        replacement.providers.crossref.enabled = patch.providers_enabled[2];
        replacement.providers.pubmed.enabled = patch.providers_enabled[3];
        replacement.providers.arxiv.enabled = patch.providers_enabled[4];
        if let Err(code) = apply_public_email_change(
            &mut replacement.providers.openalex.email,
            patch.openalex_email,
        ) {
            return DesktopEvent::ValidationFailed { code };
        }
        if let Err(code) = apply_public_email_change(
            &mut replacement.providers.crossref.email,
            patch.crossref_email,
        ) {
            return DesktopEvent::ValidationFailed { code };
        }
        if replacement == loaded.settings {
            return DesktopEvent::ValidationFailed {
                code: "global-settings-unchanged",
            };
        }

        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = global_settings_patch_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::GlobalSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::GlobalSettings,
            title: "Global settings preview",
            summary: "Atomically update the default profile, provider enablement, and supported public contact settings.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn select_skills_destination(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(path) = self.folder_picker.pick_folder() else {
            return DesktopEvent::Cancelled {
                code: "skills-destination-selection-cancelled",
            };
        };
        let target = match approve_materialization_target(&path) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let display_path = PrivateDisplayText::new(display_path(&path));
        self.selected_skills_target = Some(target);
        DesktopEvent::SkillsDestinationSelected { display_path }
    }

    fn preview_skills_materialization(&mut self, profile: ProfileKind) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target = PrivateDisplayText::new(display_path(target.path()));
        let digest = skills_materialization_digest(
            self.content.pack().pack_sha256(),
            profile,
            target.path(),
        );
        self.active_operation = Some(PendingDesktopOperation::SkillsMaterialization {
            token,
            profile,
            target,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsMaterialization,
            title: "Skills materialization preview",
            summary: "Write the selected embedded profile to the explicitly selected folder and verify its managed receipt.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn verify_skills_materialization(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        match verify_materialization(&target) {
            Ok(_) => DesktopEvent::Completed {
                code: "skills-materialization-verified",
            },
            Err(error) => DesktopEvent::Failed {
                code: error.reason_code(),
            },
        }
    }

    fn preview_skills_removal(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let receipt = match verify_materialization(&target) {
            Ok(receipt) => receipt,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target = PrivateDisplayText::new(display_path(target.path()));
        let digest = skills_removal_digest(&receipt, target.path());
        self.active_operation = Some(PendingDesktopOperation::SkillsRemoval {
            token,
            target,
            expected_receipt: receipt,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsRemoval,
            title: "Skills removal preview",
            summary: "Remove only the selected Qiongli-managed materialization after re-verifying its complete receipt.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_activation(&mut self, target: IntegrationTarget) -> DesktopEvent {
        self.cancel_active_operation();
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let now_unix = match now_unix() {
            Ok(now_unix) => now_unix,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if let Some(session) = self
            .candidate_sessions
            .iter_mut()
            .find(|session| session.target == target)
        {
            return match session.preview(token, now_unix) {
                Ok(preview) => {
                    self.active_operation =
                        Some(PendingDesktopOperation::Candidate { token, target });
                    DesktopEvent::PreviewReady(preview)
                }
                Err(code) => DesktopEvent::Failed { code },
            };
        }
        let Some(session) = self
            .activation_sessions
            .iter_mut()
            .find(|session| session.target == target)
        else {
            self.active_operation = Some(PendingDesktopOperation::Blocked(token));
            return DesktopEvent::PreviewReady(OperationPreview {
                token,
                kind: OperationKind::Activation,
                title: match target {
                    IntegrationTarget::Codex => "Codex installation preview",
                    IntegrationTarget::ClaudeCode => "Claude Code installation preview",
                },
                summary: "The local target was inspected. No host state was changed.",
                display_target: None,
                plan_digest_sha256: None,
                approvals_required: Vec::new(),
                can_confirm: false,
                blocked_reason: Some("production-activation-session-unavailable"),
            });
        };
        match session.preview(token, now_unix) {
            Ok(preview) => {
                self.active_operation = Some(PendingDesktopOperation::Activation { token, target });
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn cancel_active_operation(&mut self) {
        if let Some(PendingDesktopOperation::Activation { target, .. }) = &self.active_operation
            && let Some(session) = self
                .activation_sessions
                .iter_mut()
                .find(|session| session.target == *target)
        {
            session.cancel();
        }
        if let Some(PendingDesktopOperation::Candidate { target, .. }) = &self.active_operation
            && let Some(session) = self
                .candidate_sessions
                .iter_mut()
                .find(|session| session.target == *target)
        {
            session.cancel();
        }
        self.active_operation = None;
    }
}

fn apply_public_email_change(
    current: &mut Option<EmailAddress>,
    change: PublicSettingChange,
) -> Result<(), &'static str> {
    match change {
        PublicSettingChange::Keep => Ok(()),
        PublicSettingChange::Clear => {
            *current = None;
            Ok(())
        }
        PublicSettingChange::Replace(value) => {
            *current = Some(
                EmailAddress::parse(value.expose())
                    .map_err(|_| "provider-public-setting-invalid")?,
            );
            Ok(())
        }
    }
}

fn global_settings_patch_digest(expected_revision: u64, settings: &GlobalSettings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-GLOBAL-SETTINGS-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hash_component(
        &mut hasher,
        profile_from_content(settings.default_profile)
            .id()
            .as_bytes(),
    );
    hasher.update([
        u8::from(settings.providers.openalex.enabled),
        u8::from(settings.providers.semantic_scholar.enabled),
        u8::from(settings.providers.crossref.enabled),
        u8::from(settings.providers.pubmed.enabled),
        u8::from(settings.providers.arxiv.enabled),
        u8::from(settings.providers.openalex.api_key_ref.is_some()),
        u8::from(settings.providers.semantic_scholar.api_key_ref.is_some()),
        u8::from(settings.providers.pubmed.api_key_ref.is_some()),
    ]);
    hash_optional_email(&mut hasher, settings.providers.openalex.email.as_ref());
    hash_optional_email(&mut hasher, settings.providers.crossref.email.as_ref());
    lower_hex(&hasher.finalize())
}

fn skills_materialization_digest(pack_sha256: &str, profile: ProfileKind, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-MATERIALIZATION-V1\0");
    hash_component(&mut hasher, pack_sha256.as_bytes());
    hash_component(&mut hasher, profile.id().as_bytes());
    hash_path(&mut hasher, path);
    lower_hex(&hasher.finalize())
}

fn skills_removal_digest(receipt: &MaterializationReceiptV1, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-REMOVAL-V1\0");
    hash_component(&mut hasher, receipt.pack_sha256.as_bytes());
    hash_component(&mut hasher, receipt.content_root_sha256.as_bytes());
    hash_component(
        &mut hasher,
        profile_from_content(receipt.profile).id().as_bytes(),
    );
    hasher.update(
        u64::try_from(receipt.entries.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for entry in &receipt.entries {
        hash_component(&mut hasher, entry.path.as_bytes());
        hash_component(&mut hasher, entry.sha256.as_bytes());
    }
    hash_path(&mut hasher, path);
    lower_hex(&hasher.finalize())
}

fn hash_optional_email(hasher: &mut Sha256, value: Option<&EmailAddress>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hash_component(hasher, value.as_str().as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn hash_component(hasher: &mut Sha256, value: &[u8]) {
    hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(value);
}

#[cfg(unix)]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    use std::os::unix::ffi::OsStrExt;

    hash_component(hasher, path.as_os_str().as_bytes());
}

#[cfg(windows)]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    use std::os::windows::ffi::OsStrExt;

    let encoded = path
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    hash_component(hasher, &encoded);
}

#[cfg(not(any(unix, windows)))]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    hash_component(hasher, path.to_string_lossy().as_bytes());
}

fn display_path(path: &Path) -> String {
    let mut display = path
        .to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_control() {
                '�'
            } else {
                character
            }
        })
        .take(1_024)
        .collect::<String>();
    if display.is_empty() {
        display.push_str("<selected-folder>");
    }
    display
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}

fn mcp_self_test_counts(snapshot: &DesktopSnapshotV1) -> McpSelfTestCounts {
    McpSelfTestCounts {
        enabled_providers: snapshot
            .config
            .providers
            .iter()
            .filter(|provider| provider.enabled)
            .count(),
        ready_providers: snapshot
            .config
            .providers
            .iter()
            .filter(|provider| {
                provider.enabled && provider.readiness == ProviderReadinessView::Ready
            })
            .count(),
        discovered_clients: snapshot
            .integrations
            .iter()
            .filter(|integration| {
                !matches!(
                    integration.discovery,
                    IntegrationDiscoveryState::NotDiscovered
                        | IntegrationDiscoveryState::Unavailable
                )
            })
            .count(),
        registered_clients: snapshot
            .integrations
            .iter()
            .filter(|integration| {
                integration.registration == StatusCode::Ready
                    && !matches!(
                        integration.discovery,
                        IntegrationDiscoveryState::NotDiscovered
                            | IntegrationDiscoveryState::Unavailable
                    )
            })
            .count(),
    }
}

const fn mcp_counts_from_view(view: &McpSelfTestView) -> McpSelfTestCounts {
    McpSelfTestCounts {
        enabled_providers: view.enabled_provider_count,
        ready_providers: view.ready_provider_count,
        discovered_clients: view.discovered_client_count,
        registered_clients: view.registered_client_count,
    }
}

impl DesktopService for NativeDesktopService {
    fn snapshot(&mut self) -> DesktopSnapshotV1 {
        let mut snapshot = build_snapshot(&self.environment, &self.content);
        snapshot.capabilities.apply =
            !self.activation_sessions.is_empty() || !self.candidate_sessions.is_empty();
        for integration in &mut snapshot.integrations {
            let authority_available = self
                .activation_sessions
                .iter()
                .any(|session| session.target == integration.target)
                || self
                    .candidate_sessions
                    .iter()
                    .any(|session| session.target == integration.target);
            integration.candidate_required = integration.discovery
                == IntegrationDiscoveryState::DiscoveredUnmanaged
                && !authority_available;
        }
        snapshot
    }

    fn execute(&mut self, intent: DesktopIntent) -> DesktopEvent {
        match intent {
            DesktopIntent::Refresh => DesktopEvent::SnapshotReplaced(self.snapshot()),
            DesktopIntent::RunLiteMcpSelfTest => self.start_mcp_self_test(),
            DesktopIntent::PollLiteMcpSelfTest => self.poll_mcp_self_test(),
            DesktopIntent::CancelLiteMcpSelfTest => self.cancel_mcp_self_test(),
            DesktopIntent::RefreshIntegrationDiscovery => {
                DesktopEvent::SnapshotReplaced(self.snapshot())
            }
            DesktopIntent::PreviewGlobalSettingsPatch(patch) => self.preview_global_settings(patch),
            DesktopIntent::SelectSkillsDestination => self.select_skills_destination(),
            DesktopIntent::PreviewSkillsMaterialization { profile } => {
                self.preview_skills_materialization(profile)
            }
            DesktopIntent::VerifySkillsMaterialization => self.verify_skills_materialization(),
            DesktopIntent::PreviewSkillsRemoval => self.preview_skills_removal(),
            DesktopIntent::PreviewProviderPublicSetting {
                provider,
                public_email,
            } => {
                if !matches!(provider, ProviderKind::Crossref | ProviderKind::OpenAlex)
                    || EmailAddress::parse(public_email.expose()).is_err()
                {
                    return DesktopEvent::ValidationFailed {
                        code: "provider-public-setting-invalid",
                    };
                }
                self.issue_preview(
                    "Provider setting preview",
                    "The public contact email is valid. No value was stored.",
                    "config-write-unavailable",
                )
            }
            DesktopIntent::PreviewIntegration { target } => self.preview_activation(target),
            DesktopIntent::ConfirmOperation { token } => {
                let Some(operation) = self.active_operation.as_ref() else {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                };
                if operation.token() != token {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                let operation = self
                    .active_operation
                    .take()
                    .expect("validated active operation remains available");
                match operation {
                    PendingDesktopOperation::Blocked(_) => DesktopEvent::Failed {
                        code: "desktop-apply-unavailable",
                    },
                    PendingDesktopOperation::GlobalSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "global-settings-updated-cleanup-required"
                                } else {
                                    "global-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::SkillsMaterialization {
                        profile, target, ..
                    } => {
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let previous = verify_materialization(&target).ok();
                        match self.content.materialize_profile(profile.id(), &target) {
                        Ok(receipt) => {
                            match crate::managed_content::register_managed_materialization(
                                root.state_root(),
                                &target,
                                &receipt,
                            ) {
                                Ok(()) => DesktopEvent::Completed {
                                    code: "skills-materialization-completed",
                                },
                                Err(code) => {
                                    match crate::managed_content::compensate_unregistered_materialization(
                                        &self.content,
                                        &target,
                                        &receipt,
                                        previous.as_ref(),
                                    ) {
                                        Ok(()) => DesktopEvent::Failed { code },
                                        Err(recovery) => DesktopEvent::Failed { code: recovery },
                                    }
                                }
                            }
                        }
                        Err(error) => DesktopEvent::Failed {
                            code: error.reason_code(),
                        },
                    }
                    }
                    PendingDesktopOperation::SkillsRemoval {
                        target,
                        expected_receipt,
                        ..
                    } => {
                        let observed = match verify_materialization(&target) {
                            Ok(observed) => observed,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        if observed != expected_receipt {
                            return DesktopEvent::Failed {
                                code: "materialization-target-changed",
                            };
                        }
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match remove_materialization(&target) {
                            Ok(removed) if removed == expected_receipt => {
                                match crate::managed_content::unregister_managed_materialization(
                                    root.state_root(),
                                    &target,
                                    &expected_receipt,
                                ) {
                                    Ok(()) => DesktopEvent::Completed {
                                        code: "skills-materialization-removed",
                                    },
                                    Err(code) => {
                                        match crate::managed_content::restore_managed_materialization(
                                            &self.content,
                                            &target,
                                            &expected_receipt,
                                        ) {
                                            Ok(()) => DesktopEvent::Failed { code },
                                            Err(recovery) => {
                                                DesktopEvent::Failed { code: recovery }
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(_) => DesktopEvent::Failed {
                                code: "materialization-target-changed",
                            },
                            Err(error) => {
                                DesktopEvent::Failed {
                                    code: error.reason_code(),
                                }
                            }
                        }
                    }
                    PendingDesktopOperation::Activation { target, .. } => {
                        let Some(session) = self
                            .activation_sessions
                            .iter_mut()
                            .find(|session| session.target == target)
                        else {
                            return DesktopEvent::Failed {
                                code: "desktop-activation-session-missing",
                            };
                        };
                        match now_unix().and_then(|now_unix| session.confirm(now_unix)) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::Candidate { target, .. } => {
                        let Some(home) = self.environment.platform_home() else {
                            return DesktopEvent::Failed {
                                code: "native-candidate-home-unavailable",
                            };
                        };
                        let Some(session) = self
                            .candidate_sessions
                            .iter_mut()
                            .find(|session| session.target == target)
                        else {
                            return DesktopEvent::Failed {
                                code: "desktop-candidate-session-missing",
                            };
                        };
                        match now_unix()
                            .and_then(|now_unix| session.confirm(&self.content, home, now_unix))
                        {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                }
            }
            DesktopIntent::CancelOperation { token } => {
                if self
                    .active_operation
                    .as_ref()
                    .map(PendingDesktopOperation::token)
                    != Some(token)
                {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                self.cancel_active_operation();
                DesktopEvent::Cancelled {
                    code: "operation-preview-cancelled",
                }
            }
        }
    }
}

fn build_snapshot(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> DesktopSnapshotV1 {
    let manifest = content.pack().manifest();
    let profiles = ProfileKind::ALL.map(|profile| ProfileView {
        profile,
        included_resource_kinds: content
            .profiles()
            .iter()
            .find(|candidate| profile_from_content(candidate.id) == profile)
            .map_or(0, |candidate| candidate.included_resource_kinds.len()),
    });
    let (config, config_diagnostic) = config_snapshot(environment);
    let (codex, codex_diagnostic) = codex_snapshot(environment);
    let (claude, claude_diagnostic) = claude_snapshot(environment);
    let config_edit = matches!(config.status, StatusCode::Missing | StatusCode::Ready)
        && config.revision.is_some();
    DesktopSnapshotV1 {
        schema_version: DESKTOP_SNAPSHOT_SCHEMA_VERSION,
        product: ProductView {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            operating_system: operating_system_view(OperatingSystem::current()),
            architecture: architecture_view(Architecture::current()),
        },
        content: ContentView {
            status: StatusCode::Ready,
            pack_id: manifest.pack_id.clone(),
            content_version: manifest.content_version.clone(),
            entry_count: manifest.entries.len(),
            profiles,
        },
        mcp: McpView {
            status: StatusCode::Ready,
            profile: ProfileKind::MarketplaceLite,
            public_tool_count: LITE_PUBLIC_TOOL_NAMES.len(),
        },
        config,
        integrations: [codex, claude],
        diagnostics: [
            DiagnosticCheckView {
                check: DiagnosticCheckId::EmbeddedContent,
                status: StatusCode::Ready,
                blocking: false,
                remediation: RemediationCode::None,
            },
            config_diagnostic,
            DiagnosticCheckView {
                check: DiagnosticCheckId::SecureStore,
                status: StatusCode::Unavailable,
                blocking: false,
                remediation: RemediationCode::SecureStoreNotImplemented,
            },
            codex_diagnostic,
            claude_diagnostic,
        ],
        capabilities: CapabilityView {
            refresh: true,
            config_edit,
            skills_materialize: OperatingSystem::current().is_some(),
            provider_preview: true,
            mcp_self_test: true,
            integration_discovery: true,
            integration_preview: true,
            apply: false,
        },
    }
}

const fn integration_target(target: ClientActivationTarget) -> IntegrationTarget {
    match target {
        ClientActivationTarget::Codex => IntegrationTarget::Codex,
        ClientActivationTarget::ClaudeCode => IntegrationTarget::ClaudeCode,
    }
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system-clock-unavailable")
}

fn config_snapshot(environment: &CommandEnvironment) -> (ConfigView, DiagnosticCheckView) {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return unavailable_config(error),
    };
    let loaded = store.load().ok();
    let status = store.status();
    let view_status = config_status(status.state);
    let providers = status
        .providers
        .as_ref()
        .map_or_else(unavailable_providers, |providers| {
            [
                provider_view(
                    ProviderKind::OpenAlex,
                    &providers.openalex,
                    loaded
                        .as_ref()
                        .is_some_and(|loaded| loaded.settings.providers.openalex.email.is_some()),
                ),
                provider_view(
                    ProviderKind::SemanticScholar,
                    &providers.semantic_scholar,
                    false,
                ),
                provider_view(
                    ProviderKind::Crossref,
                    &providers.crossref,
                    loaded
                        .as_ref()
                        .is_some_and(|loaded| loaded.settings.providers.crossref.email.is_some()),
                ),
                provider_view(ProviderKind::PubMed, &providers.pubmed, false),
                provider_view(ProviderKind::Arxiv, &providers.arxiv, false),
            ]
        });
    let diagnostic = DiagnosticCheckView {
        check: DiagnosticCheckId::GlobalConfig,
        status: view_status,
        blocking: config_is_blocking(status.state),
        remediation: config_remediation(status.state),
    };
    (
        ConfigView {
            status: view_status,
            revision: status.revision,
            default_profile: status.default_profile.map(profile_from_content),
            secret_store: secret_store_status(&status),
            providers,
            cleanup_required: status.cleanup_required,
        },
        diagnostic,
    )
}

fn unavailable_config(error: ConfigError) -> (ConfigView, DiagnosticCheckView) {
    let (status, remediation) = if error == ConfigError::HomeUnavailable {
        (StatusCode::Unavailable, RemediationCode::HomeUnavailable)
    } else {
        (StatusCode::Invalid, RemediationCode::InspectGlobalConfig)
    };
    (
        ConfigView {
            status,
            revision: None,
            default_profile: None,
            secret_store: StatusCode::Unavailable,
            providers: unavailable_providers(),
            cleanup_required: false,
        },
        DiagnosticCheckView {
            check: DiagnosticCheckId::GlobalConfig,
            status,
            blocking: false,
            remediation,
        },
    )
}

fn provider_view(
    provider: ProviderKind,
    status: &RedactedProviderStatus,
    public_setting_present: bool,
) -> ProviderView {
    ProviderView {
        provider,
        enabled: status.enabled,
        readiness: match status.readiness {
            ProviderReadiness::Disabled => ProviderReadinessView::Disabled,
            ProviderReadiness::Ready => ProviderReadinessView::Ready,
            ProviderReadiness::NeedsSecret => ProviderReadinessView::NeedsSecret,
            ProviderReadiness::NeedsPublicSetting => ProviderReadinessView::NeedsPublicSetting,
        },
        public_setting_present,
        secret_reference_present: status.secret_ref_present,
    }
}

fn unavailable_providers() -> [ProviderView; 5] {
    ProviderKind::ALL.map(|provider| ProviderView {
        provider,
        enabled: false,
        readiness: ProviderReadinessView::Unavailable,
        public_setting_present: false,
        secret_reference_present: false,
    })
}

fn codex_snapshot(environment: &CommandEnvironment) -> (IntegrationView, DiagnosticCheckView) {
    let Some(home) = environment.platform_home() else {
        return unavailable_integration(
            IntegrationTarget::Codex,
            StatusCode::Unavailable,
            RemediationCode::HomeUnavailable,
        );
    };
    match discover_codex_user(home) {
        Ok(target) => {
            let client_discovered = match discover_client_config_root(&home.join(".codex")) {
                Ok(discovered) => discovered,
                Err(()) => {
                    return unavailable_integration(
                        IntegrationTarget::Codex,
                        StatusCode::Unavailable,
                        RemediationCode::InspectCodexLocal,
                    );
                }
            };
            let summary = target.summary();
            let source = match summary.source {
                CodexSourceState::Missing => StatusCode::Missing,
                CodexSourceState::Ready => StatusCode::Ready,
            };
            let marketplace = match summary.marketplace {
                CodexMarketplaceState::Missing => StatusCode::Missing,
                CodexMarketplaceState::Ready => StatusCode::Ready,
            };
            let registration = registration_status_codex(summary.registration);
            integration_result(
                IntegrationView {
                    target: IntegrationTarget::Codex,
                    discovery: integration_discovery(client_discovered, registration),
                    candidate_required: false,
                    overall: if client_discovered {
                        integration_overall(source, marketplace, None, registration)
                    } else {
                        StatusCode::Missing
                    },
                    source,
                    marketplace,
                    direct_package: None,
                    registration,
                    symbolic_location: SymbolicLocation::CodexMarketplace,
                    activation: ActivationPolicy::ClientActionRequired,
                },
                DiagnosticCheckId::CodexLocal,
                RemediationCode::InspectCodexLocal,
            )
        }
        Err(error) => unavailable_integration(
            IntegrationTarget::Codex,
            codex_error_status(error),
            codex_error_remediation(error),
        ),
    }
}

fn claude_snapshot(environment: &CommandEnvironment) -> (IntegrationView, DiagnosticCheckView) {
    let Some(home) = environment.platform_home() else {
        return unavailable_integration(
            IntegrationTarget::ClaudeCode,
            StatusCode::Unavailable,
            RemediationCode::HomeUnavailable,
        );
    };
    let config_root = environment
        .claude_config_root()
        .map_or_else(|| home.join(".claude"), ToOwned::to_owned);
    match discover_claude_user_with_config(home, &config_root) {
        Ok(target) => {
            let client_discovered = match discover_client_config_root(&config_root) {
                Ok(discovered) => discovered,
                Err(()) => {
                    return unavailable_integration(
                        IntegrationTarget::ClaudeCode,
                        StatusCode::Unavailable,
                        RemediationCode::InspectClaudeCodeLocal,
                    );
                }
            };
            let summary = target.summary();
            let source = match summary.source {
                ClaudeSourceState::Missing => StatusCode::Missing,
                ClaudeSourceState::Ready => StatusCode::Ready,
            };
            let marketplace = match summary.marketplace {
                ClaudeMarketplaceState::Missing => StatusCode::Missing,
                ClaudeMarketplaceState::Ready => StatusCode::Ready,
            };
            let direct_package = match summary.skills_plugin {
                ClaudeSkillsPluginState::Missing => StatusCode::Missing,
                ClaudeSkillsPluginState::Ready => StatusCode::Ready,
                ClaudeSkillsPluginState::Conflict => StatusCode::Conflict,
            };
            let registration = registration_status_claude(summary.registration);
            integration_result(
                IntegrationView {
                    target: IntegrationTarget::ClaudeCode,
                    discovery: integration_discovery(client_discovered, registration),
                    candidate_required: false,
                    overall: if client_discovered {
                        integration_overall(source, marketplace, Some(direct_package), registration)
                    } else {
                        StatusCode::Missing
                    },
                    source,
                    marketplace,
                    direct_package: Some(direct_package),
                    registration,
                    symbolic_location: SymbolicLocation::ClaudeMarketplace,
                    activation: ActivationPolicy::ReloadOrClientActionRequired,
                },
                DiagnosticCheckId::ClaudeCodeLocal,
                RemediationCode::InspectClaudeCodeLocal,
            )
        }
        Err(error) => unavailable_integration(
            IntegrationTarget::ClaudeCode,
            claude_error_status(error),
            claude_error_remediation(error),
        ),
    }
}

fn integration_result(
    view: IntegrationView,
    check: DiagnosticCheckId,
    remediation: RemediationCode,
) -> (IntegrationView, DiagnosticCheckView) {
    (
        view,
        DiagnosticCheckView {
            check,
            status: view.overall,
            blocking: integration_is_blocking(view.overall),
            remediation: if view.overall == StatusCode::Ready {
                RemediationCode::None
            } else {
                remediation
            },
        },
    )
}

fn discover_client_config_root(path: &Path) -> Result<bool, ()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(()),
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(()),
    }
}

const fn integration_discovery(
    client_discovered: bool,
    registration: StatusCode,
) -> IntegrationDiscoveryState {
    if !client_discovered {
        return IntegrationDiscoveryState::NotDiscovered;
    }
    match registration {
        StatusCode::Ready => IntegrationDiscoveryState::Managed,
        StatusCode::Drifted => IntegrationDiscoveryState::Drifted,
        StatusCode::Conflict => IntegrationDiscoveryState::Conflict,
        StatusCode::RecoveryRequired => IntegrationDiscoveryState::RecoveryRequired,
        StatusCode::Missing => IntegrationDiscoveryState::DiscoveredUnmanaged,
        StatusCode::Attention
        | StatusCode::Unavailable
        | StatusCode::Disabled
        | StatusCode::Blocked
        | StatusCode::Invalid
        | StatusCode::FutureSchema
        | StatusCode::Insecure
        | StatusCode::Busy
        | StatusCode::WriteUnsupported => IntegrationDiscoveryState::Unavailable,
    }
}

fn unavailable_integration(
    target: IntegrationTarget,
    status: StatusCode,
    remediation: RemediationCode,
) -> (IntegrationView, DiagnosticCheckView) {
    let (check, symbolic_location, activation, direct_package) = match target {
        IntegrationTarget::Codex => (
            DiagnosticCheckId::CodexLocal,
            SymbolicLocation::CodexMarketplace,
            ActivationPolicy::ClientActionRequired,
            None,
        ),
        IntegrationTarget::ClaudeCode => (
            DiagnosticCheckId::ClaudeCodeLocal,
            SymbolicLocation::ClaudeMarketplace,
            ActivationPolicy::ReloadOrClientActionRequired,
            Some(status),
        ),
    };
    let view = IntegrationView {
        target,
        discovery: IntegrationDiscoveryState::Unavailable,
        candidate_required: false,
        overall: status,
        source: status,
        marketplace: status,
        direct_package,
        registration: status,
        symbolic_location,
        activation,
    };
    (
        view,
        DiagnosticCheckView {
            check,
            status,
            blocking: false,
            remediation,
        },
    )
}

const fn profile_from_content(profile: ProfileId) -> ProfileKind {
    match profile {
        ProfileId::SkillOnly => ProfileKind::SkillOnly,
        ProfileId::MarketplaceLite => ProfileKind::MarketplaceLite,
        ProfileId::Full => ProfileKind::Full,
    }
}

const fn profile_to_content(profile: ProfileKind) -> ProfileId {
    match profile {
        ProfileKind::SkillOnly => ProfileId::SkillOnly,
        ProfileKind::MarketplaceLite => ProfileId::MarketplaceLite,
        ProfileKind::Full => ProfileId::Full,
    }
}

const fn operating_system_view(os: Option<OperatingSystem>) -> OperatingSystemView {
    match os {
        Some(OperatingSystem::Linux) => OperatingSystemView::Linux,
        Some(OperatingSystem::Macos) => OperatingSystemView::MacOs,
        Some(OperatingSystem::Windows) => OperatingSystemView::Windows,
        None => OperatingSystemView::Unsupported,
    }
}

const fn architecture_view(architecture: Option<Architecture>) -> ArchitectureView {
    match architecture {
        Some(Architecture::Aarch64) => ArchitectureView::Aarch64,
        Some(Architecture::X86_64) => ArchitectureView::X86_64,
        None => ArchitectureView::Unsupported,
    }
}

const fn config_status(state: ConfigState) -> StatusCode {
    match state {
        ConfigState::Missing => StatusCode::Missing,
        ConfigState::Ready => StatusCode::Ready,
        ConfigState::Invalid => StatusCode::Invalid,
        ConfigState::FutureSchema => StatusCode::FutureSchema,
        ConfigState::Insecure => StatusCode::Insecure,
        ConfigState::Busy => StatusCode::Busy,
        ConfigState::RecoveryRequired => StatusCode::RecoveryRequired,
        ConfigState::WriteUnsupported => StatusCode::WriteUnsupported,
    }
}

const fn config_is_blocking(state: ConfigState) -> bool {
    !matches!(state, ConfigState::Missing | ConfigState::Ready)
}

const fn config_remediation(state: ConfigState) -> RemediationCode {
    match state {
        ConfigState::Missing | ConfigState::Ready => RemediationCode::None,
        ConfigState::Invalid => RemediationCode::InspectGlobalConfig,
        ConfigState::FutureSchema => RemediationCode::UpgradeQiongli,
        ConfigState::Insecure => RemediationCode::RepairGlobalConfigPermissions,
        ConfigState::Busy => RemediationCode::RetryGlobalConfig,
        ConfigState::RecoveryRequired => RemediationCode::RecoverGlobalConfig,
        ConfigState::WriteUnsupported => RemediationCode::UseSupportedPlatform,
    }
}

fn secret_store_status(status: &RedactedConfigStatus) -> StatusCode {
    if status.secret_store == "ready" {
        StatusCode::Ready
    } else {
        StatusCode::Unavailable
    }
}

const fn registration_status_codex(state: CodexRegistrationState) -> StatusCode {
    match state {
        CodexRegistrationState::Absent => StatusCode::Missing,
        CodexRegistrationState::Registered => StatusCode::Ready,
        CodexRegistrationState::Conflict => StatusCode::Conflict,
        CodexRegistrationState::Drifted => StatusCode::Drifted,
        CodexRegistrationState::RecoveryRequired => StatusCode::RecoveryRequired,
    }
}

const fn registration_status_claude(state: ClaudeRegistrationState) -> StatusCode {
    match state {
        ClaudeRegistrationState::Absent => StatusCode::Missing,
        ClaudeRegistrationState::Registered => StatusCode::Ready,
        ClaudeRegistrationState::Conflict => StatusCode::Conflict,
        ClaudeRegistrationState::Drifted => StatusCode::Drifted,
        ClaudeRegistrationState::RecoveryRequired => StatusCode::RecoveryRequired,
    }
}

fn integration_overall(
    source: StatusCode,
    marketplace: StatusCode,
    direct_package: Option<StatusCode>,
    registration: StatusCode,
) -> StatusCode {
    let states = [
        source,
        marketplace,
        direct_package.unwrap_or(StatusCode::Ready),
        registration,
    ];
    for priority in [
        StatusCode::RecoveryRequired,
        StatusCode::Conflict,
        StatusCode::Drifted,
        StatusCode::Invalid,
        StatusCode::Busy,
        StatusCode::Unavailable,
    ] {
        if states.contains(&priority) {
            return priority;
        }
    }
    if states.iter().all(|state| *state == StatusCode::Ready) {
        StatusCode::Ready
    } else {
        StatusCode::Missing
    }
}

const fn integration_is_blocking(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::RecoveryRequired
            | StatusCode::Conflict
            | StatusCode::Drifted
            | StatusCode::Invalid
            | StatusCode::Insecure
    )
}

const fn codex_error_status(error: CodexAdapterError) -> StatusCode {
    match error {
        CodexAdapterError::RecoveryRequired => StatusCode::RecoveryRequired,
        CodexAdapterError::RegistrationConflict => StatusCode::Conflict,
        CodexAdapterError::RegistrationDrift | CodexAdapterError::ObservedStateMismatch => {
            StatusCode::Drifted
        }
        CodexAdapterError::LockBusy => StatusCode::Busy,
        CodexAdapterError::UnsupportedPlatform | CodexAdapterError::HomeUnavailable => {
            StatusCode::Unavailable
        }
        _ => StatusCode::Invalid,
    }
}

const fn claude_error_status(error: ClaudeAdapterError) -> StatusCode {
    match error {
        ClaudeAdapterError::RecoveryRequired => StatusCode::RecoveryRequired,
        ClaudeAdapterError::RegistrationConflict => StatusCode::Conflict,
        ClaudeAdapterError::RegistrationDrift | ClaudeAdapterError::ObservedStateMismatch => {
            StatusCode::Drifted
        }
        ClaudeAdapterError::LockBusy => StatusCode::Busy,
        ClaudeAdapterError::UnsupportedPlatform | ClaudeAdapterError::HomeUnavailable => {
            StatusCode::Unavailable
        }
        _ => StatusCode::Invalid,
    }
}

const fn codex_error_remediation(error: CodexAdapterError) -> RemediationCode {
    match error {
        CodexAdapterError::UnsupportedPlatform => RemediationCode::UseSupportedPlatform,
        CodexAdapterError::HomeUnavailable => RemediationCode::HomeUnavailable,
        _ => RemediationCode::InspectCodexLocal,
    }
}

const fn claude_error_remediation(error: ClaudeAdapterError) -> RemediationCode {
    match error {
        ClaudeAdapterError::UnsupportedPlatform => RemediationCode::UseSupportedPlatform,
        ClaudeAdapterError::HomeUnavailable => RemediationCode::HomeUnavailable,
        _ => RemediationCode::InspectClaudeCodeLocal,
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::SecretRef;

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct FakeFolderPicker {
        path: Option<PathBuf>,
    }

    struct CancelAwareExecutor;

    impl McpSelfTestExecutor for CancelAwareExecutor {
        fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView {
            while !cancelled.load(Ordering::Acquire) {
                thread::yield_now();
            }
            terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts)
        }
    }

    impl FolderPicker for FakeFolderPicker {
        fn pick_folder(&mut self) -> Option<PathBuf> {
            self.path.take()
        }
    }

    #[test]
    fn startup_validation_is_repeatable_without_a_config_home() {
        let environment = CommandEnvironment::with_paths(None, None, None);
        let content = crate::embedded_content().expect("embedded content must load");

        assert_eq!(validate_desktop_startup(&environment, &content), Ok(()));
        assert_eq!(validate_desktop_startup(&environment, &content), Ok(()));
    }

    #[test]
    fn read_only_snapshot_is_valid_and_creates_no_state() {
        let root = isolated_root("snapshot");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(
            Some(OsString::from(&config)),
            Some(home.clone()),
            Some(root.join("claude-config")),
        );
        let content = crate::embedded_content().unwrap();

        let snapshot = build_snapshot(&environment, &content);

        assert_eq!(snapshot.validate(), Ok(()));
        assert_eq!(snapshot.mcp.public_tool_count, LITE_PUBLIC_TOOL_NAMES.len());
        assert!(!config.exists());
        assert!(!home.join(".qiongli").exists());
        assert!(!home.join(".agents").exists());
        assert!(!home.join(".claude").exists());
        assert!(!root.join("claude-config").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn production_previews_never_enable_apply_or_echo_private_input() {
        let environment = CommandEnvironment::default();
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        assert!(!service.snapshot().capabilities.apply);

        let event = service.execute(DesktopIntent::PreviewProviderPublicSetting {
            provider: ProviderKind::Crossref,
            public_email: qiongli_ui::PrivateText::new("private@example.org".to_owned()),
        });

        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid public setting should produce a preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("config-write-unavailable"));
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        assert!(!format!("{preview:?}").contains("private@example.org"));

        let wrong_token = if preview.token == OperationToken::new(99) {
            OperationToken::new(100)
        } else {
            OperationToken::new(99)
        };
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation { token: wrong_token }),
            DesktopEvent::Failed {
                code: "operation-token-invalid",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::CancelOperation {
                token: preview.token,
            }),
            DesktopEvent::Cancelled {
                code: "operation-preview-cancelled",
            }
        );
    }

    #[test]
    fn source_session_discovers_clients_without_install_authority() {
        assert_eq!(
            integration_discovery(true, StatusCode::Ready),
            IntegrationDiscoveryState::Managed
        );
        assert_eq!(
            integration_discovery(true, StatusCode::Drifted),
            IntegrationDiscoveryState::Drifted
        );
        assert_eq!(
            integration_discovery(true, StatusCode::Conflict),
            IntegrationDiscoveryState::Conflict
        );
        assert_eq!(
            integration_discovery(true, StatusCode::RecoveryRequired),
            IntegrationDiscoveryState::RecoveryRequired
        );
        let root = isolated_root("source-discovery");
        let home = root.join("home");
        let claude_config = home.join(".claude");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(None, Some(home.clone()), Some(claude_config.clone()));
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment.clone(), content, Vec::new());

        let missing = service.snapshot();
        assert!(missing.integrations.iter().all(|integration| {
            integration.discovery == IntegrationDiscoveryState::NotDiscovered
                && !integration.candidate_required
        }));

        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(&claude_config).unwrap();
        let discovered = service.snapshot();
        assert!(discovered.integrations.iter().all(|integration| {
            integration.discovery == IntegrationDiscoveryState::DiscoveredUnmanaged
                && integration.candidate_required
                && integration.registration == StatusCode::Missing
        }));
        assert!(!discovered.capabilities.apply);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lite_mcp_self_test_uses_exact_registry_and_offline_dispatch() {
        let root = isolated_root("mcp-self-test");
        let home = root.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        let DesktopEvent::McpSelfTestUpdated(started) =
            service.execute(DesktopIntent::RunLiteMcpSelfTest)
        else {
            panic!("self-test must return bounded progress");
        };
        assert_eq!(started.state, McpSelfTestState::Running);
        let completed = (0..1_000)
            .find_map(|_| {
                let DesktopEvent::McpSelfTestUpdated(view) =
                    service.execute(DesktopIntent::PollLiteMcpSelfTest)
                else {
                    panic!("self-test poll must return typed progress");
                };
                if view.state == McpSelfTestState::Running {
                    thread::sleep(Duration::from_millis(1));
                    None
                } else {
                    Some(view)
                }
            })
            .expect("bounded self-test must complete");
        assert_eq!(completed.state, McpSelfTestState::Passed);
        assert!(completed.validate());
        assert_eq!(completed.public_tool_count, LITE_PUBLIC_TOOL_NAMES.len());
        assert!(
            completed.checks[..4]
                .iter()
                .all(|check| check.status == StatusCode::Ready)
        );
        assert_eq!(completed.discovered_client_count, 2);
        assert_eq!(completed.registered_client_count, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lite_mcp_self_test_supports_cancel_and_fixed_timeout() {
        let root = isolated_root("mcp-self-test-timeout");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        service.mcp_self_test_executor = Arc::new(CancelAwareExecutor);

        let _ = service.execute(DesktopIntent::RunLiteMcpSelfTest);
        let DesktopEvent::McpSelfTestUpdated(cancelled) =
            service.execute(DesktopIntent::CancelLiteMcpSelfTest)
        else {
            panic!("cancellation must return a typed result");
        };
        assert_eq!(cancelled.state, McpSelfTestState::Cancelled);

        service.mcp_self_test_timeout = Duration::ZERO;
        let _ = service.execute(DesktopIntent::RunLiteMcpSelfTest);
        let DesktopEvent::McpSelfTestUpdated(timed_out) =
            service.execute(DesktopIntent::PollLiteMcpSelfTest)
        else {
            panic!("timeout must return a typed result");
        };
        assert_eq!(timed_out.state, McpSelfTestState::TimedOut);
        assert!(timed_out.validate());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn global_settings_preview_and_confirm_preserve_secret_references() {
        let root = isolated_root("global-settings");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let mut initial = GlobalSettings::default();
        initial.providers.openalex.api_key_ref =
            Some(SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap());
        let initial_outcome = store.replace(0, initial.clone()).unwrap();
        assert_eq!(initial_outcome.revision, 1);

        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        let event = service.execute(DesktopIntent::PreviewGlobalSettingsPatch(
            GlobalSettingsPatch {
                expected_revision: 1,
                default_profile: ProfileKind::Full,
                providers_enabled: [true, false, true, false, true],
                openalex_email: PublicSettingChange::Replace(qiongli_ui::PrivateText::new(
                    "openalex-private-canary@example.org".to_owned(),
                )),
                crossref_email: PublicSettingChange::Replace(qiongli_ui::PrivateText::new(
                    "crossref-private-canary@example.org".to_owned(),
                )),
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid settings must produce a preview");
        };
        assert_eq!(preview.kind, OperationKind::GlobalSettings);
        assert_eq!(
            preview.approvals_required,
            vec![OperationApproval::ClientConfigChange]
        );
        assert!(!format!("{preview:?}").contains("private-canary"));

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "global-settings-updated",
            }
        );
        let committed = store.load().unwrap();
        assert_eq!(committed.revision, 2);
        assert_eq!(committed.settings.default_profile, ProfileId::Full);
        assert_eq!(
            committed.settings.providers.openalex.api_key_ref,
            initial.providers.openalex.api_key_ref
        );
        assert_eq!(
            committed
                .settings
                .providers
                .openalex
                .email
                .as_ref()
                .map(EmailAddress::as_str),
            Some("openalex-private-canary@example.org")
        );
        assert_eq!(
            committed
                .settings
                .providers
                .crossref
                .email
                .as_ref()
                .map(EmailAddress::as_str),
            Some("crossref-private-canary@example.org")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn global_settings_confirmation_rejects_a_stale_revision() {
        let root = isolated_root("global-settings-stale");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );

        let event = service.execute(DesktopIntent::PreviewGlobalSettingsPatch(
            GlobalSettingsPatch {
                expected_revision: 0,
                default_profile: ProfileKind::Full,
                providers_enabled: [false, false, false, false, true],
                openalex_email: PublicSettingChange::Keep,
                crossref_email: PublicSettingChange::Keep,
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("missing settings must preview from revision zero");
        };
        let external = GlobalSettings {
            default_profile: ProfileId::SkillOnly,
            ..GlobalSettings::default()
        };
        store.replace(0, external.clone()).unwrap();

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Failed {
                code: "revision-conflict",
            }
        );
        assert_eq!(store.load().unwrap().settings, external);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selected_skills_destination_materializes_and_verifies_without_debug_path() {
        let root = isolated_root("skills-destination");
        let home = root.join("home");
        let target = root.join("selected-private-canary");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&target).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker {
                path: Some(target.clone()),
            }),
        );

        let selected = service.execute(DesktopIntent::SelectSkillsDestination);
        assert!(matches!(
            selected,
            DesktopEvent::SkillsDestinationSelected { .. }
        ));
        assert!(!format!("{selected:?}").contains("selected-private-canary"));

        let event = service.execute(DesktopIntent::PreviewSkillsMaterialization {
            profile: ProfileKind::SkillOnly,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("selected Skills destination must preview");
        };
        assert_eq!(preview.kind, OperationKind::SkillsMaterialization);
        assert_eq!(
            preview.approvals_required,
            vec![OperationApproval::FilesystemWrite]
        );
        assert!(!format!("{preview:?}").contains("selected-private-canary"));

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::VerifySkillsMaterialization),
            DesktopEvent::Completed {
                code: "skills-materialization-verified",
            }
        );
        let approved = approve_materialization_target(&target).unwrap();
        let receipt = verify_materialization(&approved).unwrap();
        assert_eq!(receipt.profile, ProfileId::SkillOnly);

        let removal = service.execute(DesktopIntent::PreviewSkillsRemoval);
        let DesktopEvent::PreviewReady(removal_preview) = removal else {
            panic!("verified Skills destination must preview removal");
        };
        assert_eq!(removal_preview.kind, OperationKind::SkillsRemoval);
        assert!(!format!("{removal_preview:?}").contains("selected-private-canary"));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: removal_preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-removed",
            }
        );
        assert!(!target.exists());
        fs::remove_dir_all(root).unwrap();
    }

    fn isolated_root(name: &str) -> PathBuf {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let fixture_base = native_root.join("target/qiongli-desktop-service-tests");
        fs::create_dir_all(&fixture_base).expect("desktop fixture base must be created");
        let root = fixture_base.join(format!(
            "qiongli-r3f-{name}-{}-{sequence}",
            std::process::id()
        ));
        create_private_directory(&root);
        root
    }

    #[cfg(unix)]
    fn create_private_directory(path: &Path) {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .expect("private directory must be created");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("private directory mode must be retained");
    }

    #[cfg(windows)]
    fn create_private_directory(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path)
            .expect("owner-only Windows directory must be created");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_private_directory(path: &Path) {
        fs::create_dir(path).expect("private directory must be created");
    }
}
