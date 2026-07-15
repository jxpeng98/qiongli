use qiongli_config::{
    ConfigError, ConfigState, EmailAddress, ProviderReadiness, RedactedConfigStatus,
    RedactedProviderStatus,
};
use qiongli_content::{EmbeddedContent, ProfileId};
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
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use qiongli_ui::{
    ActivationPolicy, ArchitectureView, CapabilityView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, IntegrationTarget, IntegrationView,
    McpView, OperatingSystemView, OperationApproval, OperationPreview, OperationToken, ProductView,
    ProfileKind, ProfileView, ProviderKind, ProviderReadinessView, ProviderView, RemediationCode,
    StatusCode, SymbolicLocation,
};

use std::fmt::{self, Debug, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::command::{CommandEnvironment, config_store};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLaunchError;

const ACTIVATION_PLAN_TTL_SECONDS: u64 = 600;
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
    qiongli_ui::run_native(Box::new(service)).map_err(|_| DesktopLaunchError)
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
    qiongli_ui::run_native(Box::new(service)).map_err(|_| DesktopLaunchError)
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
            title: match self.target {
                IntegrationTarget::Codex => "Codex activation preview",
                IntegrationTarget::ClaudeCode => "Claude Code activation preview",
            },
            summary: "Register the verified local Qiongli source. Client-owned enablement remains a host action.",
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
            title: match self.target {
                IntegrationTarget::Codex => "Codex candidate installation preview",
                IntegrationTarget::ClaudeCode => "Claude Code candidate installation preview",
            },
            summary: "Install the verified native payload and fixed local Qiongli source, then register it. Client-owned enablement remains a host action.",
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

struct NativeDesktopService {
    environment: CommandEnvironment,
    content: EmbeddedContent,
    active_operation: Option<PendingDesktopOperation>,
    activation_sessions: Vec<DesktopActivationSession>,
    candidate_sessions: Vec<DesktopCandidateSession>,
}

#[derive(Clone, Copy)]
enum PendingDesktopOperation {
    Blocked(OperationToken),
    Activation {
        token: OperationToken,
        target: IntegrationTarget,
    },
    Candidate {
        token: OperationToken,
        target: IntegrationTarget,
    },
}

impl PendingDesktopOperation {
    const fn token(self) -> OperationToken {
        match self {
            Self::Blocked(token)
            | Self::Activation { token, .. }
            | Self::Candidate { token, .. } => token,
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
            activation_sessions: Vec::new(),
            candidate_sessions,
        }
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
            title,
            summary,
            plan_digest_sha256: None,
            approvals_required: Vec::new(),
            can_confirm: false,
            blocked_reason: Some(blocked_reason),
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
                title: match target {
                    IntegrationTarget::Codex => "Codex installation preview",
                    IntegrationTarget::ClaudeCode => "Claude Code installation preview",
                },
                summary: "The local target was inspected. No host state was changed.",
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
        if let Some(PendingDesktopOperation::Activation { target, .. }) = self.active_operation
            && let Some(session) = self
                .activation_sessions
                .iter_mut()
                .find(|session| session.target == target)
        {
            session.cancel();
        }
        if let Some(PendingDesktopOperation::Candidate { target, .. }) = self.active_operation
            && let Some(session) = self
                .candidate_sessions
                .iter_mut()
                .find(|session| session.target == target)
        {
            session.cancel();
        }
        self.active_operation = None;
    }
}

impl DesktopService for NativeDesktopService {
    fn snapshot(&mut self) -> DesktopSnapshotV1 {
        let mut snapshot = build_snapshot(&self.environment, &self.content);
        snapshot.capabilities.apply =
            !self.activation_sessions.is_empty() || !self.candidate_sessions.is_empty();
        snapshot
    }

    fn execute(&mut self, intent: DesktopIntent) -> DesktopEvent {
        match intent {
            DesktopIntent::Refresh => DesktopEvent::SnapshotReplaced(self.snapshot()),
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
                let Some(operation) = self.active_operation else {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                };
                if operation.token() != token {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                self.active_operation = None;
                match operation {
                    PendingDesktopOperation::Blocked(_) => DesktopEvent::Failed {
                        code: "desktop-apply-unavailable",
                    },
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
                if self.active_operation.map(PendingDesktopOperation::token) != Some(token) {
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
            provider_preview: true,
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
    let status = match config_store(environment) {
        Ok(store) => store.status(),
        Err(error) => return unavailable_config(error),
    };
    let view_status = config_status(status.state);
    let providers = status
        .providers
        .as_ref()
        .map_or_else(unavailable_providers, |providers| {
            [
                provider_view(ProviderKind::OpenAlex, &providers.openalex),
                provider_view(ProviderKind::SemanticScholar, &providers.semantic_scholar),
                provider_view(ProviderKind::Crossref, &providers.crossref),
                provider_view(ProviderKind::PubMed, &providers.pubmed),
                provider_view(ProviderKind::Arxiv, &providers.arxiv),
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

fn provider_view(provider: ProviderKind, status: &RedactedProviderStatus) -> ProviderView {
    ProviderView {
        provider,
        enabled: status.enabled,
        readiness: match status.readiness {
            ProviderReadiness::Disabled => ProviderReadinessView::Disabled,
            ProviderReadiness::Ready => ProviderReadinessView::Ready,
            ProviderReadiness::NeedsSecret => ProviderReadinessView::NeedsSecret,
            ProviderReadiness::NeedsPublicSetting => ProviderReadinessView::NeedsPublicSetting,
        },
        secret_reference_present: status.secret_ref_present,
    }
}

fn unavailable_providers() -> [ProviderView; 5] {
    ProviderKind::ALL.map(|provider| ProviderView {
        provider,
        enabled: false,
        readiness: ProviderReadinessView::Unavailable,
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
                    overall: integration_overall(source, marketplace, None, registration),
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
    match discover_claude_user_with_config(home, config_root) {
        Ok(target) => {
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
                    overall: integration_overall(
                        source,
                        marketplace,
                        Some(direct_package),
                        registration,
                    ),
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
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

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

    fn isolated_root(name: &str) -> PathBuf {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "qiongli-r3f-{name}-{}-{sequence}",
            std::process::id()
        ))
    }
}
