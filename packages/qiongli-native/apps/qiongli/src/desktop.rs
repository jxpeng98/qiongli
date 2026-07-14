use qiongli_config::{
    ConfigError, ConfigState, EmailAddress, ProviderReadiness, RedactedConfigStatus,
    RedactedProviderStatus,
};
use qiongli_content::{EmbeddedContent, ProfileId};
use qiongli_platform::{
    Architecture, ClaudeAdapterError, ClaudeMarketplaceState, ClaudeRegistrationState,
    ClaudeSkillsPluginState, ClaudeSourceState, CodexAdapterError, CodexMarketplaceState,
    CodexRegistrationState, CodexSourceState, OperatingSystem, discover_claude_user_with_config,
    discover_codex_user,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use qiongli_ui::{
    ActivationPolicy, ArchitectureView, CapabilityView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, IntegrationTarget, IntegrationView,
    McpView, OperatingSystemView, OperationPreview, OperationToken, ProductView, ProfileKind,
    ProfileView, ProviderKind, ProviderReadinessView, ProviderView, RemediationCode, StatusCode,
    SymbolicLocation,
};

use crate::command::{CommandEnvironment, config_store};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLaunchError;

pub fn run_desktop(
    environment: CommandEnvironment,
    content: EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    let service = NativeDesktopService::new(environment, content);
    qiongli_ui::run_native(Box::new(service)).map_err(|_| DesktopLaunchError)
}

struct NativeDesktopService {
    environment: CommandEnvironment,
    content: EmbeddedContent,
    next_token: u64,
    active_token: Option<OperationToken>,
}

impl NativeDesktopService {
    const fn new(environment: CommandEnvironment, content: EmbeddedContent) -> Self {
        Self {
            environment,
            content,
            next_token: 1,
            active_token: None,
        }
    }

    fn issue_preview(
        &mut self,
        title: &'static str,
        summary: &'static str,
        blocked_reason: &'static str,
    ) -> DesktopEvent {
        let token = OperationToken::new(self.next_token);
        self.next_token = self.next_token.checked_add(1).unwrap_or(1);
        self.active_token = Some(token);
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            title,
            summary,
            can_confirm: false,
            blocked_reason: Some(blocked_reason),
        })
    }
}

impl DesktopService for NativeDesktopService {
    fn snapshot(&mut self) -> DesktopSnapshotV1 {
        build_snapshot(&self.environment, &self.content)
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
            DesktopIntent::PreviewIntegration { target } => {
                let title = match target {
                    IntegrationTarget::Codex => "Codex installation preview",
                    IntegrationTarget::ClaudeCode => "Claude Code installation preview",
                };
                self.issue_preview(
                    title,
                    "The local target was inspected. No host state was changed.",
                    "production-launch-grant-unavailable",
                )
            }
            DesktopIntent::ConfirmOperation { token } => {
                if self.active_token != Some(token) {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                self.active_token = None;
                DesktopEvent::Failed {
                    code: "desktop-apply-unavailable",
                }
            }
            DesktopIntent::CancelOperation { token } => {
                if self.active_token != Some(token) {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                self.active_token = None;
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
        let mut service = NativeDesktopService::new(environment, content);

        let event = service.execute(DesktopIntent::PreviewProviderPublicSetting {
            provider: ProviderKind::Crossref,
            public_email: qiongli_ui::PrivateText::new("private@example.org".to_owned()),
        });

        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid public setting should produce a preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("config-write-unavailable"));
        assert!(!format!("{preview:?}").contains("private@example.org"));
    }

    fn isolated_root(name: &str) -> PathBuf {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "qiongli-r3f-{name}-{}-{sequence}",
            std::process::id()
        ))
    }
}
