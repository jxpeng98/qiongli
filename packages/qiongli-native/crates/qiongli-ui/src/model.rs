use std::fmt::{self, Debug, Formatter};

use zeroize::Zeroizing;

pub const DESKTOP_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const MAX_DISPLAY_TEXT_BYTES: usize = 128;
const MAX_CONTENT_ENTRIES: usize = 100_000;
const MAX_PUBLIC_TOOLS: usize = 256;
const MAX_RESOURCE_KINDS: usize = 32;
const MAX_PROVIDERS: usize = 5;
const MAX_INTEGRATIONS: usize = 2;
pub const MAX_INTEGRATION_PATHS: usize = 10;
pub const MAX_DIAGNOSTIC_PATHS: usize = 64;
const MAX_UPDATE_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_EXACT_PATH_BYTES: usize = 4096;
const MAX_DIAGNOSTIC_DETAILS_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopSection {
    Overview,
    Skills,
    Mcp,
    Providers,
    Integrations,
    Settings,
    About,
    Diagnostics,
}

impl DesktopSection {
    pub const ALL: [Self; 8] = [
        Self::Overview,
        Self::Skills,
        Self::Mcp,
        Self::Providers,
        Self::Integrations,
        Self::Settings,
        Self::About,
        Self::Diagnostics,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Skills => "Skills",
            Self::Mcp => "MCP",
            Self::Providers => "Literature Providers",
            Self::Integrations => "Integrations",
            Self::Settings => "Global Settings",
            Self::About => "About",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusCode {
    Ready,
    Attention,
    Missing,
    Unavailable,
    Disabled,
    Blocked,
    RecoveryRequired,
    Conflict,
    Drifted,
    Invalid,
    FutureSchema,
    Insecure,
    Busy,
    WriteUnsupported,
}

impl StatusCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Attention => "attention",
            Self::Missing => "missing",
            Self::Unavailable => "unavailable",
            Self::Disabled => "disabled",
            Self::Blocked => "blocked",
            Self::RecoveryRequired => "recovery-required",
            Self::Conflict => "conflict",
            Self::Drifted => "drifted",
            Self::Invalid => "invalid",
            Self::FutureSchema => "future-schema",
            Self::Insecure => "insecure",
            Self::Busy => "busy",
            Self::WriteUnsupported => "write-unsupported",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Attention => "Attention",
            Self::Missing => "Missing",
            Self::Unavailable => "Unavailable",
            Self::Disabled => "Disabled",
            Self::Blocked => "Blocked",
            Self::RecoveryRequired => "Recovery required",
            Self::Conflict => "Conflict",
            Self::Drifted => "Drifted",
            Self::Invalid => "Invalid",
            Self::FutureSchema => "Newer schema",
            Self::Insecure => "Insecure",
            Self::Busy => "Busy",
            Self::WriteUnsupported => "Write unsupported",
        }
    }

    #[must_use]
    pub const fn requires_attention(self) -> bool {
        !matches!(self, Self::Ready | Self::Missing | Self::Disabled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatingSystemView {
    Linux,
    MacOs,
    Windows,
    Unsupported,
}

impl OperatingSystemView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::MacOs => "macOS",
            Self::Windows => "Windows",
            Self::Unsupported => "Unsupported target",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchitectureView {
    Aarch64,
    X86_64,
    Unsupported,
}

impl ArchitectureView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Aarch64 => "AArch64",
            Self::X86_64 => "x86-64",
            Self::Unsupported => "Unsupported architecture",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateStreamView {
    Stable,
    Beta,
}

impl UpdateStreamView {
    pub const ALL: [Self; 2] = [Self::Stable, Self::Beta];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Beta => "Beta",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdatePhaseView {
    Unavailable,
    Idle,
    Checking,
    Current,
    Available,
    Downloading,
    Verifying,
    Staging,
    ReadyToInstall,
    Installing,
    AwaitingRestart,
    Cancelling,
    Cancelled,
    RecoveryRequired,
    Failed,
}

impl UpdatePhaseView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "Unavailable",
            Self::Idle => "Ready to check",
            Self::Checking => "Checking",
            Self::Current => "Up to date",
            Self::Available => "Update available",
            Self::Downloading => "Downloading",
            Self::Verifying => "Verifying",
            Self::Staging => "Preparing",
            Self::ReadyToInstall => "Ready to install",
            Self::Installing => "Installing",
            Self::AwaitingRestart => "Restarting",
            Self::Cancelling => "Cancelling",
            Self::Cancelled => "Cancelled",
            Self::RecoveryRequired => "Recovery required",
            Self::Failed => "Update failed",
        }
    }

    #[must_use]
    pub const fn is_busy(self) -> bool {
        matches!(
            self,
            Self::Checking
                | Self::Downloading
                | Self::Verifying
                | Self::Staging
                | Self::Installing
                | Self::AwaitingRestart
                | Self::Cancelling
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateRemediation {
    None,
    RetryCheck,
    RetryPreparation,
    CancelAndRetry,
    RestartApplication,
    MoveToApplications,
    ReinstallApplication,
    InstallTrustedRelease,
    UseSupportedPlatform,
}

impl UpdateRemediation {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::RetryCheck => "retry-update-check",
            Self::RetryPreparation => "retry-update-preparation",
            Self::CancelAndRetry => "cancel-update-and-retry",
            Self::RestartApplication => "restart-qiongli",
            Self::MoveToApplications => "move-qiongli-to-applications",
            Self::ReinstallApplication => "reinstall-qiongli",
            Self::InstallTrustedRelease => "install-trusted-qiongli-release",
            Self::UseSupportedPlatform => "use-supported-update-platform",
        }
    }

    #[must_use]
    pub const fn guidance(self) -> &'static str {
        match self {
            Self::None => "No action is required.",
            Self::RetryCheck => "Check the network connection, then retry the update check.",
            Self::RetryPreparation => "Retry preparation. Existing installed bytes are unchanged.",
            Self::CancelAndRetry => "Cancel the staged transaction, then check again.",
            Self::RestartApplication => "Close and reopen Qiongli to continue recovery.",
            Self::MoveToApplications => {
                "Move Qiongli to a private writable Applications folder, then retry."
            }
            Self::ReinstallApplication => {
                "Install a fresh trusted Qiongli 2 application without deleting research data."
            }
            Self::InstallTrustedRelease => {
                "Install a signed Qiongli 2 release that contains update authority."
            }
            Self::UseSupportedPlatform => {
                "Automatic update in Alpha.1 requires macOS on Apple silicon."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateProgressView {
    pub completed_steps: u8,
    pub total_steps: u8,
    pub label: &'static str,
    pub indeterminate: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateView {
    pub status: StatusCode,
    pub selected_stream: UpdateStreamView,
    pub phase: UpdatePhaseView,
    pub available_version: Option<String>,
    pub archive_size_bytes: Option<u64>,
    pub progress: Option<UpdateProgressView>,
    pub reason_code: &'static str,
    pub remediation: UpdateRemediation,
    pub can_select_stream: bool,
    pub can_check: bool,
    pub can_prepare: bool,
    pub can_install: bool,
    pub can_cancel: bool,
}

impl UpdateView {
    #[must_use]
    pub fn validate(&self) -> bool {
        let version_valid = self
            .available_version
            .as_deref()
            .is_none_or(|version| validate_version_text(version, "update-version-invalid").is_ok());
        let archive_valid = self
            .archive_size_bytes
            .is_none_or(|size| (1..=MAX_UPDATE_ARCHIVE_BYTES).contains(&size));
        let progress_valid = self.progress.is_none_or(|progress| {
            progress.total_steps > 0
                && progress.completed_steps <= progress.total_steps
                && !progress.label.is_empty()
                && progress.label.len() <= MAX_DISPLAY_TEXT_BYTES
                && !progress.label.chars().any(char::is_control)
        });
        let reason_valid = !self.reason_code.is_empty()
            && self.reason_code.len() <= MAX_DISPLAY_TEXT_BYTES
            && self
                .reason_code
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        let busy_actions_valid = !self.phase.is_busy()
            || (!self.can_select_stream
                && !self.can_check
                && !self.can_prepare
                && !self.can_install);
        let install_valid = !self.can_install || self.phase == UpdatePhaseView::ReadyToInstall;
        let prepare_valid = !self.can_prepare
            || matches!(
                self.phase,
                UpdatePhaseView::Available | UpdatePhaseView::Failed | UpdatePhaseView::Cancelled
            );
        version_valid
            && archive_valid
            && progress_valid
            && reason_valid
            && busy_actions_valid
            && install_valid
            && prepare_valid
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileKind {
    SkillOnly,
    MarketplaceLite,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillsDestinationPreset {
    QiongliManaged,
    DetectedCodex,
    DetectedClaudeCode,
    CurrentProject,
    CustomFolder,
}

impl SkillsDestinationPreset {
    pub const ALL: [Self; 5] = [
        Self::QiongliManaged,
        Self::DetectedCodex,
        Self::DetectedClaudeCode,
        Self::CurrentProject,
        Self::CustomFolder,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::QiongliManaged => "Qiongli Managed",
            Self::DetectedCodex => "Detected Codex",
            Self::DetectedClaudeCode => "Detected Claude Code",
            Self::CurrentProject => "Current project",
            Self::CustomFolder => "Custom Folder",
        }
    }

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::QiongliManaged => "qiongli-managed",
            Self::DetectedCodex => "detected-codex",
            Self::DetectedClaudeCode => "detected-claude-code",
            Self::CurrentProject => "current-project",
            Self::CustomFolder => "custom-folder",
        }
    }

    #[must_use]
    pub const fn symbolic_path(self) -> &'static str {
        match self {
            Self::QiongliManaged => "<user-home>/.qiongli-skills",
            Self::DetectedCodex => "<qiongli-home>/clients/codex/plugins/qiongli-next",
            Self::DetectedClaudeCode => "<qiongli-home>/clients/claude-code/plugins/qiongli-next",
            Self::CurrentProject => "<project>/.qiongli-skills",
            Self::CustomFolder => "<custom-folder>",
        }
    }

    #[must_use]
    pub const fn install_method(self) -> SkillsInstallMethodView {
        SkillsInstallMethodView::ReceiptOwnedCopy
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillsInstallMethodView {
    ManagedSymlink,
    ReceiptOwnedCopy,
}

impl SkillsInstallMethodView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ManagedSymlink => "Managed symlink",
            Self::ReceiptOwnedCopy => "Receipt-owned copy",
        }
    }
}

impl ProfileKind {
    pub const ALL: [Self; 3] = [Self::SkillOnly, Self::MarketplaceLite, Self::Full];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::SkillOnly => "skill-only",
            Self::MarketplaceLite => "marketplace-lite",
            Self::Full => "full",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::SkillOnly => "Academic skills and their supporting resources",
            Self::MarketplaceLite => "Skills plus the dependency-free Lite MCP contract",
            Self::Full => "Complete embedded research workflow content",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderKind {
    OpenAlex,
    SemanticScholar,
    Crossref,
    PubMed,
    Arxiv,
}

impl ProviderKind {
    pub const ALL: [Self; 5] = [
        Self::OpenAlex,
        Self::SemanticScholar,
        Self::Crossref,
        Self::PubMed,
        Self::Arxiv,
    ];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::OpenAlex => "openalex",
            Self::SemanticScholar => "semantic-scholar",
            Self::Crossref => "crossref",
            Self::PubMed => "pubmed",
            Self::Arxiv => "arxiv",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpenAlex => "OpenAlex",
            Self::SemanticScholar => "Semantic Scholar",
            Self::Crossref => "Crossref",
            Self::PubMed => "PubMed",
            Self::Arxiv => "arXiv",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderReadinessView {
    Disabled,
    Ready,
    NeedsSecret,
    NeedsPublicSetting,
    Unavailable,
}

impl ProviderReadinessView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Disabled => "Disabled",
            Self::Ready => "Ready",
            Self::NeedsSecret => "Needs a secret",
            Self::NeedsPublicSetting => "Needs a public setting",
            Self::Unavailable => "Unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationTarget {
    Codex,
    ClaudeCode,
}

impl IntegrationTarget {
    pub const ALL: [Self; 2] = [Self::Codex, Self::ClaudeCode];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::ClaudeCode => "Claude Code",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolicLocation {
    CodexMarketplace,
    ClaudeMarketplace,
}

impl SymbolicLocation {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::CodexMarketplace => "Codex personal marketplace",
            Self::ClaudeMarketplace => "Claude Code marketplace",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationPolicy {
    ClientActionRequired,
    ReloadOrClientActionRequired,
}

impl ActivationPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ClientActionRequired => "Client action required",
            Self::ReloadOrClientActionRequired => "Reload or client action required",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCheckId {
    EmbeddedContent,
    GlobalConfig,
    SecureStore,
    ManagedContent,
    CodexLocal,
    ClaudeCodeLocal,
    LiteMcp,
    LiteratureProviders,
    UpdateRecovery,
    FullRuntime,
}

impl DiagnosticCheckId {
    pub const ALL: [Self; 10] = [
        Self::EmbeddedContent,
        Self::GlobalConfig,
        Self::SecureStore,
        Self::ManagedContent,
        Self::CodexLocal,
        Self::ClaudeCodeLocal,
        Self::LiteMcp,
        Self::LiteratureProviders,
        Self::UpdateRecovery,
        Self::FullRuntime,
    ];

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmbeddedContent => "embedded-content",
            Self::GlobalConfig => "global-config",
            Self::SecureStore => "secure-store",
            Self::ManagedContent => "managed-content",
            Self::CodexLocal => "codex-local",
            Self::ClaudeCodeLocal => "claude-code-local",
            Self::LiteMcp => "lite-mcp",
            Self::LiteratureProviders => "literature-providers",
            Self::UpdateRecovery => "update-recovery",
            Self::FullRuntime => "full-runtime",
        }
    }

    #[must_use]
    pub const fn section(self) -> DesktopSection {
        match self {
            Self::EmbeddedContent | Self::UpdateRecovery => DesktopSection::About,
            Self::GlobalConfig => DesktopSection::Settings,
            Self::SecureStore | Self::LiteratureProviders => DesktopSection::Providers,
            Self::ManagedContent => DesktopSection::Skills,
            Self::CodexLocal | Self::ClaudeCodeLocal => DesktopSection::Integrations,
            Self::LiteMcp => DesktopSection::Mcp,
            Self::FullRuntime => DesktopSection::Diagnostics,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmbeddedContent => "Embedded content",
            Self::GlobalConfig => "Global configuration",
            Self::SecureStore => "Secure store",
            Self::ManagedContent => "Managed content",
            Self::CodexLocal => "Codex local integration",
            Self::ClaudeCodeLocal => "Claude Code local integration",
            Self::LiteMcp => "Lite MCP",
            Self::LiteratureProviders => "Literature providers",
            Self::UpdateRecovery => "Update and recovery",
            Self::FullRuntime => "Full runtime",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemediationCode {
    None,
    InspectGlobalConfig,
    UpgradeQiongli,
    RepairGlobalConfigPermissions,
    RetryGlobalConfig,
    RecoverGlobalConfig,
    UseSupportedPlatform,
    SecureStoreNotImplemented,
    HomeUnavailable,
    InspectManagedContent,
    InspectCodexLocal,
    InspectClaudeCodeLocal,
    RetryLiteMcpSelfTest,
    ConfigureLiteratureProviders,
    InspectUpdateState,
    InstallSupportedClient,
    InstallClientIntegration,
    ResolveClientConflict,
    RepairClientIntegration,
    ReinstallQiongli,
    UseSupportedSecureStore,
    UpgradeToFullRuntime,
}

impl RemediationCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::InspectGlobalConfig => "inspect-global-config",
            Self::UpgradeQiongli => "upgrade-qiongli",
            Self::RepairGlobalConfigPermissions => "repair-global-config-permissions",
            Self::RetryGlobalConfig => "retry-global-config",
            Self::RecoverGlobalConfig => "recover-global-config",
            Self::UseSupportedPlatform => "use-supported-platform",
            Self::SecureStoreNotImplemented => "secure-store-not-implemented",
            Self::HomeUnavailable => "home-unavailable",
            Self::InspectManagedContent => "inspect-managed-content",
            Self::InspectCodexLocal => "inspect-codex-local",
            Self::InspectClaudeCodeLocal => "inspect-claude-code-local",
            Self::RetryLiteMcpSelfTest => "retry-mcp-self-test",
            Self::ConfigureLiteratureProviders => "configure-literature-providers",
            Self::InspectUpdateState => "inspect-update-state",
            Self::InstallSupportedClient => "install-supported-client",
            Self::InstallClientIntegration => "install-client-integration",
            Self::ResolveClientConflict => "resolve-client-conflict",
            Self::RepairClientIntegration => "repair-client-integration",
            Self::ReinstallQiongli => "reinstall-qiongli",
            Self::UseSupportedSecureStore => "use-supported-secure-store",
            Self::UpgradeToFullRuntime => "upgrade-to-r4-full-runtime",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductView {
    pub version: String,
    pub build: String,
    pub operating_system: OperatingSystemView,
    pub architecture: ArchitectureView,
    pub trust: ProductTrustView,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductTrustView {
    SourceBuild,
    PackagedProductControl,
}

impl ProductTrustView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceBuild => "Source build",
            Self::PackagedProductControl => "Verified packaged product control",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileView {
    pub profile: ProfileKind,
    pub included_resource_kinds: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentView {
    pub status: StatusCode,
    pub pack_id: String,
    pub content_version: String,
    pub entry_count: usize,
    pub profiles: [ProfileView; 3],
    pub managed_skills_status: StatusCode,
    pub managed_skills: Vec<ManagedSkillsView>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedSkillsStateView {
    Missing,
    Current,
    UpdateAvailable,
    Drifted,
    Unmanaged,
}

impl ManagedSkillsStateView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Current => "current",
            Self::UpdateAvailable => "update-available",
            Self::Drifted => "drifted",
            Self::Unmanaged => "unmanaged",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedSkillsView {
    pub target_id: String,
    pub preset: SkillsDestinationPreset,
    pub state: ManagedSkillsStateView,
    pub status: StatusCode,
    pub profile: Option<ProfileKind>,
    pub product_version: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpView {
    pub status: StatusCode,
    pub profile: ProfileKind,
    pub public_tool_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliInstallStateView {
    Missing,
    InstalledCurrent,
    UpdateAvailable,
    Unavailable,
    Conflict,
}

impl CliInstallStateView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::InstalledCurrent => "installed-current",
            Self::UpdateAvailable => "update-available",
            Self::Unavailable => "unavailable",
            Self::Conflict => "conflict",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliPathStateView {
    Active,
    Configured,
    NotConfigured,
    Shadowed,
    VersionMismatch,
    NotObservable,
}

impl CliPathStateView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Configured => "configured",
            Self::NotConfigured => "not-configured",
            Self::Shadowed => "shadowed",
            Self::VersionMismatch => "version-mismatch",
            Self::NotObservable => "not-observable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliView {
    pub status: StatusCode,
    pub state: CliInstallStateView,
    pub installed_version: Option<String>,
    pub available_version: String,
    pub symbolic_target: &'static str,
    pub path_status: StatusCode,
    pub path_state: CliPathStateView,
    pub reason_code: &'static str,
    pub can_install: bool,
    pub can_test: bool,
}

pub const ZOTERO_FALLBACK_FORMATS: [&str; 4] = [
    "references.json",
    "references.ris",
    "bibliography.bib",
    "zotero-import-report.md",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoteroIntegrationStateView {
    NotObserved,
    ZoteroNotDetected,
    ZoteroIncompatible,
    ZoteroNotRunning,
    CompanionMissing,
    CompanionIncompatible,
    CompanionUpdateAvailable,
    RestartRequired,
    Ready,
    Disabled,
    NotObservable,
}

impl ZoteroIntegrationStateView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotObserved => "not-observed",
            Self::ZoteroNotDetected => "zotero-not-detected",
            Self::ZoteroIncompatible => "zotero-incompatible",
            Self::ZoteroNotRunning => "zotero-not-running",
            Self::CompanionMissing => "companion-missing",
            Self::CompanionIncompatible => "companion-incompatible",
            Self::CompanionUpdateAvailable => "companion-update-available",
            Self::RestartRequired => "restart-required",
            Self::Ready => "ready",
            Self::Disabled => "disabled",
            Self::NotObservable => "not-observable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoteroObservationView {
    NotObserved,
    Observed,
    NotObservable,
}

impl ZoteroObservationView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotObserved => "not-observed",
            Self::Observed => "observed",
            Self::NotObservable => "not-observable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoteroIntegrationView {
    pub status: StatusCode,
    pub state: ZoteroIntegrationStateView,
    pub observation: ZoteroObservationView,
    pub zotero_version: Option<String>,
    pub connector_available: bool,
    pub companion_available: bool,
    pub companion_version: Option<String>,
    pub available_companion_version: Option<String>,
    pub available_companion_sha256: Option<String>,
    pub available_companion_size_bytes: Option<u64>,
    pub endpoint_version: Option<String>,
    pub supported_endpoint_version: &'static str,
    pub supported_zotero_min_version: &'static str,
    pub supported_zotero_max_version: &'static str,
    pub installation_prepared: bool,
    pub fallback_import_available: bool,
    pub fallback_formats: [&'static str; 4],
    pub reason_code: &'static str,
    pub can_prepare_install: bool,
    pub can_reveal: bool,
    pub can_open_zotero: bool,
    pub can_verify: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpSelfTestState {
    Running,
    Passed,
    Failed,
    Cancelled,
    TimedOut,
}

impl McpSelfTestState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::TimedOut => "Timed out",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpSelfTestCheckId {
    EmbeddedContract,
    Initialize,
    ToolRegistry,
    OfflineDispatch,
    ProviderReadiness,
    ClientRegistration,
}

impl McpSelfTestCheckId {
    pub const ALL: [Self; 6] = [
        Self::EmbeddedContract,
        Self::Initialize,
        Self::ToolRegistry,
        Self::OfflineDispatch,
        Self::ProviderReadiness,
        Self::ClientRegistration,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmbeddedContract => "Embedded contract",
            Self::Initialize => "MCP initialize",
            Self::ToolRegistry => "Exact tools registry",
            Self::OfflineDispatch => "Offline dispatch",
            Self::ProviderReadiness => "Provider readiness",
            Self::ClientRegistration => "Client registration",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpSelfTestCheckView {
    pub check: McpSelfTestCheckId,
    pub status: StatusCode,
    pub code: &'static str,
    pub remediation: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpSelfTestView {
    pub state: McpSelfTestState,
    pub checks: [McpSelfTestCheckView; 6],
    pub public_tool_count: usize,
    pub enabled_provider_count: usize,
    pub ready_provider_count: usize,
    pub discovered_client_count: usize,
    pub registered_client_count: usize,
}

impl McpSelfTestView {
    #[must_use]
    pub fn validate(&self) -> bool {
        let state_valid = match self.state {
            McpSelfTestState::Running | McpSelfTestState::Cancelled => self
                .checks
                .iter()
                .all(|check| check.status == StatusCode::Missing),
            McpSelfTestState::Passed => {
                self.checks[..4]
                    .iter()
                    .all(|check| check.status == StatusCode::Ready)
                    && self.checks[4..].iter().all(|check| {
                        matches!(check.status, StatusCode::Ready | StatusCode::Attention)
                    })
            }
            McpSelfTestState::Failed => self.checks[..4]
                .iter()
                .any(|check| check.status != StatusCode::Ready),
            McpSelfTestState::TimedOut => self
                .checks
                .iter()
                .all(|check| check.status == StatusCode::Blocked),
        };
        state_valid
            && self.checks.map(|check| check.check) == McpSelfTestCheckId::ALL
            && self.public_tool_count <= MAX_PUBLIC_TOOLS
            && self.enabled_provider_count <= MAX_PROVIDERS
            && self.ready_provider_count <= self.enabled_provider_count
            && self.discovered_client_count <= MAX_INTEGRATIONS
            && self.registered_client_count <= self.discovered_client_count
            && self.checks.iter().all(|check| {
                [check.code, check.remediation].iter().all(|value| {
                    !value.is_empty()
                        && value.len() <= MAX_DISPLAY_TEXT_BYTES
                        && value.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                        })
                })
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderView {
    pub provider: ProviderKind,
    pub enabled: bool,
    pub readiness: ProviderReadinessView,
    pub public_setting_present: bool,
    pub secret_reference_present: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentBackendReadinessView {
    Disabled,
    NeedsSecretReference,
    SecretStoreUnavailable,
    CredentialMissing,
    CredentialInvalid,
    Ready,
}

impl AgentBackendReadinessView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::NeedsSecretReference => "needs-secret-reference",
            Self::SecretStoreUnavailable => "secret-store-unavailable",
            Self::CredentialMissing => "credential-missing",
            Self::CredentialInvalid => "credential-invalid",
            Self::Ready => "ready",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentBackendView {
    pub enabled: bool,
    pub readiness: AgentBackendReadinessView,
    pub secret_reference_present: bool,
    pub test_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigView {
    pub status: StatusCode,
    pub revision: Option<u64>,
    pub default_profile: Option<ProfileKind>,
    pub secret_store: StatusCode,
    pub providers: [ProviderView; 5],
    pub openai_backend: AgentBackendView,
    pub cleanup_required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationDiscoveryState {
    NotDiscovered,
    DiscoveredUnmanaged,
    Managed,
    Drifted,
    Conflict,
    RecoveryRequired,
    Unavailable,
}

impl IntegrationDiscoveryState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotDiscovered => "Client not discovered",
            Self::DiscoveredUnmanaged => "Discovered but unmanaged",
            Self::Managed => "Managed",
            Self::Drifted => "Managed installation drifted",
            Self::Conflict => "Conflicting installation",
            Self::RecoveryRequired => "Recovery required",
            Self::Unavailable => "Discovery unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationOwnershipView {
    NotInstalled,
    QiongliManaged,
    Unmanaged,
    Mixed,
    Unknown,
}

impl IntegrationOwnershipView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotInstalled => "not-installed",
            Self::QiongliManaged => "qiongli-managed",
            Self::Unmanaged => "unmanaged",
            Self::Mixed => "mixed",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotInstalled => "Not installed",
            Self::QiongliManaged => "Qiongli managed",
            Self::Unmanaged => "Unmanaged",
            Self::Mixed => "Mixed ownership",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationActionView {
    InspectOnly,
    InstallReady,
    Current,
    RepairReady,
    UpgradeClient,
    ResolveConflict,
    Unavailable,
}

impl IntegrationActionView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InspectOnly => "inspect-only",
            Self::InstallReady => "install-ready",
            Self::Current => "current",
            Self::RepairReady => "repair-ready",
            Self::UpgradeClient => "upgrade-client",
            Self::ResolveConflict => "resolve-conflict",
            Self::Unavailable => "unavailable",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::InspectOnly => "Inspect only",
            Self::InstallReady => "Install available",
            Self::Current => "No action required",
            Self::RepairReady => "Repair available",
            Self::UpgradeClient => "Upgrade client",
            Self::ResolveConflict => "Resolve conflict",
            Self::Unavailable => "Action unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationPathSurfaceView {
    ClientConfig,
    SkillsRoot,
    SkillsPackage,
    PluginMarketplace,
    PluginSource,
    StandaloneMcp,
}

impl IntegrationPathSurfaceView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ClientConfig => "Client config",
            Self::SkillsRoot => "Skills root",
            Self::SkillsPackage => "Skills package",
            Self::PluginMarketplace => "Marketplace",
            Self::PluginSource => "Plugin source",
            Self::StandaloneMcp => "Standalone MCP",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationPathScopeView {
    User,
    Project,
    Managed,
    Custom,
    Legacy,
}

impl IntegrationPathScopeView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Project => "Project",
            Self::Managed => "Qiongli managed",
            Self::Custom => "Custom",
            Self::Legacy => "Legacy",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationPathSourceView {
    EnvironmentOverride,
    OfficialDefault,
    ProjectContext,
    QiongliManaged,
    ExplicitCustom,
    LegacyObserved,
}

impl IntegrationPathSourceView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EnvironmentOverride => "Environment override",
            Self::OfficialDefault => "Official default",
            Self::ProjectContext => "Current project",
            Self::QiongliManaged => "Qiongli managed",
            Self::ExplicitCustom => "Explicit custom",
            Self::LegacyObserved => "Legacy observed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationPathManagementView {
    Supported,
    InspectOnly,
    LegacyOnly,
    Unsafe,
    Unavailable,
}

impl IntegrationPathManagementView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Supported => "Supported",
            Self::InspectOnly => "Inspect only",
            Self::LegacyOnly => "Legacy only",
            Self::Unsafe => "Unsafe",
            Self::Unavailable => "Unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegrationPathView {
    pub surface: IntegrationPathSurfaceView,
    pub scope: IntegrationPathScopeView,
    pub source: IntegrationPathSourceView,
    pub state: StatusCode,
    pub management: IntegrationPathManagementView,
    pub selected: bool,
    pub symbolic_path: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientVersionView {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl ClientVersionView {
    #[must_use]
    pub fn label(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductVersionChannelView {
    Alpha,
    Beta,
    Stable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductVersionView {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub channel: ProductVersionChannelView,
    pub prerelease_number: Option<u64>,
}

impl ProductVersionView {
    #[must_use]
    pub fn label(self) -> String {
        let base = format!("{}.{}.{}", self.major, self.minor, self.patch);
        match (self.channel, self.prerelease_number) {
            (ProductVersionChannelView::Alpha, Some(number)) => {
                format!("{base}-alpha.{number}")
            }
            (ProductVersionChannelView::Beta, Some(number)) => {
                format!("{base}-beta.{number}")
            }
            (ProductVersionChannelView::Stable, None) => base,
            _ => format!("{base}-invalid"),
        }
    }

    #[must_use]
    pub const fn validate(self) -> bool {
        matches!(
            (self.channel, self.prerelease_number),
            (
                ProductVersionChannelView::Alpha | ProductVersionChannelView::Beta,
                Some(_)
            ) | (ProductVersionChannelView::Stable, None)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientCompatibilityView {
    Supported,
    Unsupported,
    NotEvaluated,
}

impl ClientCompatibilityView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::NotEvaluated => "not-evaluated",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationObservationView {
    Observed,
    ClientActionRequired,
    NotObservable,
    ProbeUnavailable,
    ProbeFailed,
    Missing,
    InspectionBlocked,
}

impl IntegrationObservationView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::ClientActionRequired => "client-action-required",
            Self::NotObservable => "not-observable",
            Self::ProbeUnavailable => "probe-unavailable",
            Self::ProbeFailed => "probe-failed",
            Self::Missing => "missing",
            Self::InspectionBlocked => "inspection-blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationMigrationStateView {
    NotDetected,
    Available,
    ReviewRequired,
    Unavailable,
}

impl IntegrationMigrationStateView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotDetected => "not-detected",
            Self::Available => "available",
            Self::ReviewRequired => "review-required",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegrationMigrationView {
    pub state: IntegrationMigrationStateView,
    pub detected_items: usize,
    pub eligible_items: usize,
    pub review_items: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationStateView {
    NotDetected,
    Available,
    PreviewReady,
    Staged,
    AwaitingClientActivation,
    VerificationRequired,
    CleanupReady,
    Complete,
    RecoveryRequired,
    ReviewRequired,
    Unavailable,
}

impl LegacyMigrationStateView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotDetected => "not-detected",
            Self::Available => "available",
            Self::PreviewReady => "preview-ready",
            Self::Staged => "staged",
            Self::AwaitingClientActivation => "awaiting-client-activation",
            Self::VerificationRequired => "verification-required",
            Self::CleanupReady => "cleanup-ready",
            Self::Complete => "complete",
            Self::RecoveryRequired => "recovery-required",
            Self::ReviewRequired => "review-required",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationActionView {
    None,
    Start,
    Apply,
    ConfirmHostActivation,
    Cleanup,
    Finalize,
    Recover,
    Review,
}

impl LegacyMigrationActionView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Start => "start",
            Self::Apply => "apply",
            Self::ConfirmHostActivation => "confirm-host-activation",
            Self::Cleanup => "cleanup",
            Self::Finalize => "finalize",
            Self::Recover => "recover",
            Self::Review => "review",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyMigrationView {
    pub state: LegacyMigrationStateView,
    pub next_action: LegacyMigrationActionView,
    pub migration_id: Option<String>,
    pub detected_items: usize,
    pub eligible_items: usize,
    pub review_items: usize,
    pub reason_code: &'static str,
    pub provider_conflicts: Vec<LegacyProviderConflictView>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LegacyProviderView {
    OpenAlex,
    SemanticScholar,
    Crossref,
    Pubmed,
    Arxiv,
}

impl LegacyProviderView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::OpenAlex => "openalex",
            Self::SemanticScholar => "semantic-scholar",
            Self::Crossref => "crossref",
            Self::Pubmed => "pubmed",
            Self::Arxiv => "arxiv",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyProviderResolutionStrategyView {
    KeepV2,
    UseLegacy,
    MergeCompatible,
}

impl LegacyProviderResolutionStrategyView {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::KeepV2 => "keep-v2",
            Self::UseLegacy => "use-legacy",
            Self::MergeCompatible => "merge-compatible",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyProviderResolutionView {
    pub provider: LegacyProviderView,
    pub strategy: LegacyProviderResolutionStrategyView,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyProviderConflictView {
    pub provider: LegacyProviderView,
    pub differing_fields: Vec<String>,
    pub legacy_secret_present: bool,
    pub current_secret_reference_present: bool,
}

pub const EMPTY_INTEGRATION_PATHS: [Option<IntegrationPathView>; MAX_INTEGRATION_PATHS] =
    [None; MAX_INTEGRATION_PATHS];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegrationView {
    pub target: IntegrationTarget,
    pub client_version: Option<ClientVersionView>,
    pub compatibility: ClientCompatibilityView,
    pub installed_plugin_version: Option<ProductVersionView>,
    pub available_plugin_version: ProductVersionView,
    pub discovery: IntegrationDiscoveryState,
    pub candidate_required: bool,
    pub migration: IntegrationMigrationView,
    pub client: StatusCode,
    pub overall: StatusCode,
    pub source: StatusCode,
    pub skills: StatusCode,
    pub marketplace: StatusCode,
    pub direct_package: Option<StatusCode>,
    pub registration: StatusCode,
    pub activation_status: StatusCode,
    pub activation_observation: IntegrationObservationView,
    pub mcp_attachment: StatusCode,
    pub mcp_attachment_observation: IntegrationObservationView,
    pub symbolic_location: SymbolicLocation,
    pub activation: ActivationPolicy,
    pub ownership: IntegrationOwnershipView,
    pub next_action: IntegrationActionView,
    pub evidence_code: &'static str,
    pub path_count: usize,
    pub paths: [Option<IntegrationPathView>; MAX_INTEGRATION_PATHS],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegrationSelection {
    pub codex: bool,
    pub claude_code: bool,
}

impl IntegrationSelection {
    pub const ALL: Self = Self {
        codex: true,
        claude_code: true,
    };

    #[must_use]
    pub const fn is_empty(self) -> bool {
        !self.codex && !self.claude_code
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticCheckView {
    pub check: DiagnosticCheckId,
    pub status: StatusCode,
    pub blocking: bool,
    pub remediation: RemediationCode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticPathView {
    pub id: String,
    pub label: String,
    pub symbolic_path: String,
    pub exact_path: PrivateDisplayText,
    pub reveal_path: PrivateDisplayText,
    pub details: String,
    pub selected: bool,
    pub status: StatusCode,
    pub resolved_target: Option<PrivateDisplayText>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityView {
    pub refresh: bool,
    pub config_edit: bool,
    pub skills_materialize: bool,
    pub provider_preview: bool,
    pub mcp_self_test: bool,
    pub integration_discovery: bool,
    pub integration_preview: bool,
    pub apply: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopSnapshotV1 {
    pub schema_version: u32,
    pub product: ProductView,
    pub content: ContentView,
    pub mcp: McpView,
    pub cli: CliView,
    pub zotero: ZoteroIntegrationView,
    pub config: ConfigView,
    pub update: UpdateView,
    pub legacy_migration: LegacyMigrationView,
    pub integrations: [IntegrationView; 2],
    pub diagnostics: [DiagnosticCheckView; 10],
    pub diagnostic_paths: Vec<DiagnosticPathView>,
    pub capabilities: CapabilityView,
}

impl DesktopSnapshotV1 {
    pub fn validate(&self) -> Result<(), SnapshotValidationError> {
        if self.schema_version != DESKTOP_SNAPSHOT_SCHEMA_VERSION {
            return Err(SnapshotValidationError::new(
                "desktop-snapshot-schema-invalid",
            ));
        }
        validate_version_text(&self.product.version, "product-version-invalid")?;
        validate_display_text(&self.product.build, "product-build-invalid")?;
        validate_pack_id(&self.content.pack_id)?;
        validate_version_text(&self.content.content_version, "content-version-invalid")?;
        if !(1..=MAX_CONTENT_ENTRIES).contains(&self.content.entry_count) {
            return Err(SnapshotValidationError::new("content-entry-count-invalid"));
        }
        if !(1..=MAX_PUBLIC_TOOLS).contains(&self.mcp.public_tool_count) {
            return Err(SnapshotValidationError::new("mcp-tool-count-invalid"));
        }
        validate_version_text(&self.cli.available_version, "cli-version-invalid")?;
        if let Some(installed_version) = self.cli.installed_version.as_deref() {
            validate_version_text(installed_version, "cli-installed-version-invalid")?;
        }
        if self.cli.symbolic_target.is_empty()
            || self.cli.symbolic_target.len() > 256
            || self.cli.reason_code.is_empty()
            || self.cli.reason_code.len() > 128
        {
            return Err(SnapshotValidationError::new("cli-view-invalid"));
        }
        validate_zotero_integration(&self.zotero)?;
        if self.content.profiles.map(|profile| profile.profile) != ProfileKind::ALL {
            return Err(SnapshotValidationError::new("profile-order-invalid"));
        }
        if self
            .content
            .profiles
            .iter()
            .any(|profile| !(1..=MAX_RESOURCE_KINDS).contains(&profile.included_resource_kinds))
        {
            return Err(SnapshotValidationError::new(
                "profile-resource-kind-count-invalid",
            ));
        }
        if self.content.managed_skills.len() > 130
            || self
                .content
                .managed_skills
                .windows(2)
                .any(|pair| pair[0].target_id >= pair[1].target_id)
            || self.content.managed_skills.iter().any(|managed| {
                let target_id = managed
                    .target_id
                    .strip_prefix("skills-target-")
                    .unwrap_or_default();
                target_id.len() != 64
                    || !target_id
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    || match managed.state {
                        ManagedSkillsStateView::Missing => {
                            managed.status != StatusCode::Missing
                                || managed.profile.is_some()
                                || managed.product_version.is_some()
                        }
                        ManagedSkillsStateView::Current => {
                            managed.status != StatusCode::Ready
                                || !valid_managed_skills_install(managed)
                        }
                        ManagedSkillsStateView::UpdateAvailable => {
                            managed.status != StatusCode::Attention
                                || !valid_managed_skills_install(managed)
                        }
                        ManagedSkillsStateView::Drifted => {
                            managed.status != StatusCode::Drifted
                                || !valid_managed_skills_install(managed)
                        }
                        ManagedSkillsStateView::Unmanaged => {
                            managed.status != StatusCode::Conflict
                                || managed.profile.is_some()
                                || managed.product_version.is_some()
                        }
                    }
            })
        {
            return Err(SnapshotValidationError::new("managed-skills-view-invalid"));
        }
        if self.config.providers.map(|provider| provider.provider) != ProviderKind::ALL {
            return Err(SnapshotValidationError::new("provider-order-invalid"));
        }
        if !self.update.validate() {
            return Err(SnapshotValidationError::new("update-view-invalid"));
        }
        if self.legacy_migration.detected_items > 8
            || self.legacy_migration.eligible_items > self.legacy_migration.detected_items
            || self.legacy_migration.review_items > self.legacy_migration.detected_items
            || self.legacy_migration.eligible_items + self.legacy_migration.review_items
                != self.legacy_migration.detected_items
            || self
                .legacy_migration
                .migration_id
                .as_deref()
                .is_some_and(|value| {
                    value.is_empty()
                        || value.len() > 128
                        || !value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                })
            || !matches!(
                (
                    self.legacy_migration.state,
                    self.legacy_migration.next_action,
                    self.legacy_migration.migration_id.is_some(),
                ),
                (
                    LegacyMigrationStateView::NotDetected,
                    LegacyMigrationActionView::None,
                    false,
                ) | (
                    LegacyMigrationStateView::Available,
                    LegacyMigrationActionView::Start,
                    false,
                ) | (
                    LegacyMigrationStateView::PreviewReady,
                    LegacyMigrationActionView::Apply,
                    true,
                ) | (
                    LegacyMigrationStateView::Staged
                        | LegacyMigrationStateView::AwaitingClientActivation
                        | LegacyMigrationStateView::VerificationRequired,
                    LegacyMigrationActionView::ConfirmHostActivation,
                    true,
                ) | (
                    LegacyMigrationStateView::CleanupReady,
                    LegacyMigrationActionView::Cleanup,
                    true,
                ) | (
                    LegacyMigrationStateView::Complete,
                    LegacyMigrationActionView::None | LegacyMigrationActionView::Finalize,
                    true,
                ) | (
                    LegacyMigrationStateView::RecoveryRequired,
                    LegacyMigrationActionView::Recover,
                    true,
                ) | (
                    LegacyMigrationStateView::RecoveryRequired
                        | LegacyMigrationStateView::ReviewRequired,
                    LegacyMigrationActionView::Review,
                    _,
                ) | (
                    LegacyMigrationStateView::Unavailable,
                    LegacyMigrationActionView::None,
                    false,
                )
            )
            || self.legacy_migration.provider_conflicts.len() > 5
            || !self
                .legacy_migration
                .provider_conflicts
                .windows(2)
                .all(|pair| pair[0].provider < pair[1].provider)
            || self
                .legacy_migration
                .provider_conflicts
                .iter()
                .any(|conflict| {
                    conflict.differing_fields.is_empty()
                        || conflict.differing_fields.len() > 3
                        || conflict.differing_fields.iter().any(|field| {
                            field.is_empty()
                                || field.len() > 64
                                || field.chars().any(char::is_control)
                        })
                })
        {
            return Err(SnapshotValidationError::new(
                "legacy-migration-view-invalid",
            ));
        }
        validate_reason_code(
            self.legacy_migration.reason_code,
            "legacy-migration-reason-code-invalid",
        )?;
        if self.integrations.map(|integration| integration.target) != IntegrationTarget::ALL {
            return Err(SnapshotValidationError::new("integration-order-invalid"));
        }
        for integration in self.integrations {
            validate_integration_state(&integration)?;
            if !integration.available_plugin_version.validate()
                || integration
                    .installed_plugin_version
                    .is_some_and(|version| !version.validate())
            {
                return Err(SnapshotValidationError::new(
                    "integration-plugin-version-invalid",
                ));
            }
            if integration.path_count > MAX_INTEGRATION_PATHS
                || integration
                    .paths
                    .iter()
                    .enumerate()
                    .any(|(index, path)| (index < integration.path_count) != path.is_some())
            {
                return Err(SnapshotValidationError::new(
                    "integration-path-order-invalid",
                ));
            }
            if integration.migration.detected_items > 4
                || integration.migration.eligible_items > integration.migration.detected_items
                || integration.migration.review_items > integration.migration.detected_items
                || integration.migration.eligible_items + integration.migration.review_items
                    != integration.migration.detected_items
            {
                return Err(SnapshotValidationError::new(
                    "integration-migration-count-invalid",
                ));
            }
            validate_reason_code(
                integration.evidence_code,
                "integration-evidence-code-invalid",
            )?;
            for path in integration.paths.into_iter().flatten() {
                validate_symbolic_path(path.symbolic_path)?;
            }
        }
        if self.diagnostics.map(|diagnostic| diagnostic.check) != DiagnosticCheckId::ALL {
            return Err(SnapshotValidationError::new("diagnostic-order-invalid"));
        }
        if self.diagnostic_paths.len() > MAX_DIAGNOSTIC_PATHS {
            return Err(SnapshotValidationError::new(
                "diagnostic-path-count-invalid",
            ));
        }
        for path in &self.diagnostic_paths {
            validate_reason_code(&path.id, "diagnostic-path-id-invalid")?;
            validate_display_text(&path.label, "diagnostic-path-label-invalid")?;
            validate_symbolic_path(&path.symbolic_path)?;
            validate_private_path(path.exact_path.expose())?;
            validate_private_path(path.reveal_path.expose())?;
            if path.details.is_empty()
                || path.details.len() > MAX_DIAGNOSTIC_DETAILS_BYTES
                || path.details.chars().any(char::is_control)
            {
                return Err(SnapshotValidationError::new(
                    "diagnostic-path-details-invalid",
                ));
            }
            if let Some(target) = path.resolved_target.as_ref() {
                validate_private_path(target.expose())?;
            }
        }
        Ok(())
    }
}

fn validate_integration_state(
    integration: &IntegrationView,
) -> Result<(), SnapshotValidationError> {
    let unsupported = integration.compatibility == ClientCompatibilityView::Unsupported;
    if unsupported {
        if integration.next_action != IntegrationActionView::UpgradeClient
            || integration.overall != StatusCode::Blocked
            || integration.client != StatusCode::Ready
            || integration.discovery == IntegrationDiscoveryState::NotDiscovered
        {
            return Err(SnapshotValidationError::new("integration-state-invalid"));
        }
    } else if integration.next_action == IntegrationActionView::UpgradeClient {
        return Err(SnapshotValidationError::new("integration-state-invalid"));
    }

    let action_matches = |action| unsupported || integration.next_action == action;
    let valid = match integration.discovery {
        IntegrationDiscoveryState::NotDiscovered => {
            integration.compatibility == ClientCompatibilityView::NotEvaluated
                && integration.client == StatusCode::Missing
                && integration.registration == StatusCode::Missing
                && integration.ownership == IntegrationOwnershipView::NotInstalled
                && integration.next_action == IntegrationActionView::InspectOnly
        }
        IntegrationDiscoveryState::DiscoveredUnmanaged => {
            integration.client == StatusCode::Ready
                && integration.registration == StatusCode::Missing
                && matches!(
                    integration.ownership,
                    IntegrationOwnershipView::NotInstalled | IntegrationOwnershipView::Unmanaged
                )
                && action_matches(IntegrationActionView::InstallReady)
        }
        IntegrationDiscoveryState::Managed => {
            integration.client == StatusCode::Ready
                && integration.registration == StatusCode::Ready
                && integration.ownership == IntegrationOwnershipView::QiongliManaged
                && action_matches(IntegrationActionView::Current)
        }
        IntegrationDiscoveryState::Drifted => {
            integration.client == StatusCode::Ready
                && integration.registration == StatusCode::Drifted
                && integration.ownership == IntegrationOwnershipView::QiongliManaged
                && action_matches(IntegrationActionView::RepairReady)
        }
        IntegrationDiscoveryState::Conflict => {
            integration.client == StatusCode::Ready
                && integration.registration == StatusCode::Conflict
                && matches!(
                    integration.ownership,
                    IntegrationOwnershipView::Unmanaged | IntegrationOwnershipView::Mixed
                )
                && action_matches(IntegrationActionView::ResolveConflict)
        }
        IntegrationDiscoveryState::RecoveryRequired => {
            integration.client == StatusCode::Ready
                && integration.registration == StatusCode::RecoveryRequired
                && integration.ownership == IntegrationOwnershipView::QiongliManaged
                && action_matches(IntegrationActionView::RepairReady)
        }
        IntegrationDiscoveryState::Unavailable => {
            if unsupported {
                integration.client == StatusCode::Ready
                    && integration.next_action == IntegrationActionView::UpgradeClient
            } else {
                integration.next_action == IntegrationActionView::Unavailable
                    && matches!(
                        integration.client,
                        StatusCode::Ready | StatusCode::Unavailable
                    )
            }
        }
    };
    if valid {
        Ok(())
    } else {
        Err(SnapshotValidationError::new("integration-state-invalid"))
    }
}

fn validate_private_path(value: &str) -> Result<(), SnapshotValidationError> {
    if value.is_empty() || value.len() > MAX_EXACT_PATH_BYTES || value.chars().any(char::is_control)
    {
        return Err(SnapshotValidationError::new(
            "diagnostic-exact-path-invalid",
        ));
    }
    Ok(())
}

fn validate_display_text(value: &str, code: &'static str) -> Result<(), SnapshotValidationError> {
    if value.is_empty()
        || value.len() > MAX_DISPLAY_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(SnapshotValidationError::new(code));
    }
    Ok(())
}

fn validate_reason_code(value: &str, code: &'static str) -> Result<(), SnapshotValidationError> {
    validate_display_text(value, code)?;
    if value.starts_with('-')
        || value.ends_with('-')
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(SnapshotValidationError::new(code));
    }
    Ok(())
}

fn validate_symbolic_path(value: &str) -> Result<(), SnapshotValidationError> {
    validate_display_text(value, "integration-symbolic-path-invalid")?;
    if !value.is_ascii() || !value.starts_with('<') || !value.contains('>') {
        return Err(SnapshotValidationError::new(
            "integration-symbolic-path-invalid",
        ));
    }
    Ok(())
}

fn validate_version_text(value: &str, code: &'static str) -> Result<(), SnapshotValidationError> {
    validate_display_text(value, code)?;
    if !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'*'))
    {
        return Err(SnapshotValidationError::new(code));
    }
    Ok(())
}

fn valid_managed_skills_install(managed: &ManagedSkillsView) -> bool {
    managed.profile.is_some()
        && managed.product_version.as_deref().is_some_and(|version| {
            validate_version_text(version, "managed-skills-version-invalid").is_ok()
        })
}

fn validate_zotero_integration(
    view: &ZoteroIntegrationView,
) -> Result<(), SnapshotValidationError> {
    for version in [
        view.zotero_version.as_deref(),
        view.companion_version.as_deref(),
        view.available_companion_version.as_deref(),
        view.endpoint_version.as_deref(),
        Some(view.supported_endpoint_version),
        Some(view.supported_zotero_min_version),
        Some(view.supported_zotero_max_version),
    ]
    .into_iter()
    .flatten()
    {
        validate_version_text(version, "zotero-version-invalid")?;
    }
    validate_reason_code(view.reason_code, "zotero-reason-code-invalid")?;
    if view
        .available_companion_sha256
        .as_deref()
        .is_some_and(|value| !valid_lower_sha256(value))
    {
        return Err(SnapshotValidationError::new(
            "zotero-companion-digest-invalid",
        ));
    }
    if !view.fallback_import_available
        || view.fallback_formats != ZOTERO_FALLBACK_FORMATS
        || view.available_companion_version.is_some() != view.available_companion_sha256.is_some()
        || view.available_companion_version.is_some()
            != view.available_companion_size_bytes.is_some()
        || view
            .available_companion_size_bytes
            .is_some_and(|size| size == 0 || size > 2 * 1024 * 1024)
        || view.can_prepare_install && view.available_companion_version.is_none()
        || view.can_reveal != view.installation_prepared
        || view.companion_available && !view.connector_available
        || view.companion_version.is_some() && !view.companion_available
        || view.endpoint_version.is_some() && !view.companion_available
        || view.state == ZoteroIntegrationStateView::Ready
            && (view.observation != ZoteroObservationView::Observed
                || !view.connector_available
                || !view.companion_available
                || view.endpoint_version.as_deref() != Some(view.supported_endpoint_version))
        || view.state == ZoteroIntegrationStateView::ZoteroIncompatible
            && (view.observation != ZoteroObservationView::Observed
                || view.zotero_version.is_none()
                || view.can_prepare_install)
        || view.state == ZoteroIntegrationStateView::CompanionMissing
            && (view.observation != ZoteroObservationView::Observed
                || !view.connector_available
                || view.companion_available)
        || view.state == ZoteroIntegrationStateView::CompanionIncompatible
            && (view.observation != ZoteroObservationView::Observed
                || !view.connector_available
                || !view.companion_available
                || view.endpoint_version.as_deref() == Some(view.supported_endpoint_version))
        || view.state == ZoteroIntegrationStateView::CompanionUpdateAvailable
            && (view.observation != ZoteroObservationView::Observed
                || !view.connector_available
                || !view.companion_available
                || view.companion_version.is_none()
                || view.available_companion_version.is_none())
        || view.state == ZoteroIntegrationStateView::RestartRequired && !view.installation_prepared
        || view.state == ZoteroIntegrationStateView::NotObserved
            && (view.observation != ZoteroObservationView::NotObserved
                || view.connector_available
                || view.companion_available
                || view.zotero_version.is_some()
                || view.companion_version.is_some()
                || view.endpoint_version.is_some())
    {
        return Err(SnapshotValidationError::new(
            "zotero-integration-view-invalid",
        ));
    }
    Ok(())
}

fn validate_pack_id(value: &str) -> Result<(), SnapshotValidationError> {
    validate_display_text(value, "content-pack-id-invalid")?;
    if value.starts_with('-')
        || value.ends_with('-')
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(SnapshotValidationError::new("content-pack-id-invalid"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotValidationError {
    code: &'static str,
}

impl SnapshotValidationError {
    const fn new(code: &'static str) -> Self {
        Self { code }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }
}

pub struct PrivateText(Zeroizing<String>);

impl PrivateText {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PrivateDisplayText(Zeroizing<String>);

impl PrivateDisplayText {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl Debug for PrivateDisplayText {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("<private-display-text>")
    }
}

pub enum PublicSettingChange {
    Keep,
    Clear,
    Replace(PrivateText),
}

pub struct GlobalSettingsPatch {
    pub expected_revision: u64,
    pub default_profile: ProfileKind,
}

pub struct ProviderSettingsPatch {
    pub expected_revision: u64,
    pub providers_enabled: [bool; 5],
    pub openalex_email: PublicSettingChange,
    pub crossref_email: PublicSettingChange,
}

pub struct AgentBackendSettingsPatch {
    pub expected_revision: u64,
    pub openai_enabled: bool,
}

pub struct AgentRunDraft {
    pub project_id: String,
    pub expected_project_revision: u64,
    pub prompt: PrivateText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRunResultView {
    pub schema_version: u32,
    pub run_id: String,
    pub backend_id: String,
    pub model: String,
    pub finish_reason: &'static str,
    pub content: PrivateDisplayText,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: u64,
    pub model_turns: u32,
    pub tool_calls: u32,
    pub network_requests: u32,
    pub audited_tool_calls: usize,
}

pub enum ProviderSecretChange {
    Replace(PrivateText),
    Remove,
}

pub enum AgentBackendSecretChange {
    Replace(PrivateText),
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationToken(u128);

impl OperationToken {
    #[must_use]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationApproval {
    FilesystemWrite,
    ClientConfigChange,
    HostTrust,
    SecretStoreWrite,
    NetworkRequest,
}

impl OperationApproval {
    pub const ACTIVATION: [Self; 3] = [
        Self::FilesystemWrite,
        Self::ClientConfigChange,
        Self::HostTrust,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FilesystemWrite => "Filesystem write",
            Self::ClientConfigChange => "Client configuration change",
            Self::HostTrust => "Host trust",
            Self::SecretStoreWrite => "Secure credential write",
            Self::NetworkRequest => "Send prompt and redacted project data to OpenAI",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationKind {
    Activation,
    GlobalSettings,
    ProviderSettings,
    ProviderSecret,
    AgentBackendSettings,
    AgentBackendSecret,
    AgentRun,
    SkillsMaterialization,
    SkillsRemoval,
    SkillsDetach,
    CliInstall,
    ZoteroCompanionStage,
    UpdateInstall,
    LegacyMigrationStage,
    LegacyMigrationHostActivation,
    LegacyMigrationCleanup,
    LegacyMigrationFinalize,
    LegacyMigrationRecovery,
}

impl OperationKind {
    #[must_use]
    const fn approvals(self) -> &'static [OperationApproval] {
        match self {
            Self::Activation => &OperationApproval::ACTIVATION,
            Self::GlobalSettings => &[OperationApproval::ClientConfigChange],
            Self::ProviderSettings => &[OperationApproval::ClientConfigChange],
            Self::ProviderSecret => &[
                OperationApproval::SecretStoreWrite,
                OperationApproval::ClientConfigChange,
            ],
            Self::AgentBackendSettings => &[OperationApproval::ClientConfigChange],
            Self::AgentBackendSecret => &[
                OperationApproval::SecretStoreWrite,
                OperationApproval::ClientConfigChange,
            ],
            Self::AgentRun => &[OperationApproval::NetworkRequest],
            Self::SkillsMaterialization
            | Self::SkillsRemoval
            | Self::SkillsDetach
            | Self::CliInstall
            | Self::ZoteroCompanionStage
            | Self::UpdateInstall => &[OperationApproval::FilesystemWrite],
            Self::LegacyMigrationStage | Self::LegacyMigrationCleanup => &[
                OperationApproval::FilesystemWrite,
                OperationApproval::ClientConfigChange,
                OperationApproval::SecretStoreWrite,
            ],
            Self::LegacyMigrationHostActivation => &[OperationApproval::HostTrust],
            Self::LegacyMigrationFinalize => &[OperationApproval::FilesystemWrite],
            Self::LegacyMigrationRecovery => &[
                OperationApproval::FilesystemWrite,
                OperationApproval::ClientConfigChange,
            ],
        }
    }
}

pub enum DesktopIntent {
    Refresh,
    RunLiteMcpSelfTest,
    PollLiteMcpSelfTest,
    CancelLiteMcpSelfTest,
    RefreshIntegrationDiscovery,
    RefreshZoteroIntegration,
    PreviewZoteroCompanionStage,
    RevealZoteroCompanion,
    OpenZotero,
    VerifyZoteroIntegration,
    PrepareLegacyMigration {
        provider_resolutions: Vec<LegacyProviderResolutionView>,
    },
    PreviewLegacyMigrationNext,
    SelectUpdateStream {
        stream: UpdateStreamView,
    },
    CheckForUpdates,
    PrepareUpdate,
    PollUpdate,
    CancelUpdate,
    PreviewUpdateInstall,
    PreviewCliInstall,
    TestCliCommand,
    PreviewGlobalSettingsPatch(GlobalSettingsPatch),
    PreviewProviderSettingsPatch(ProviderSettingsPatch),
    PreviewProviderSecretChange {
        provider: ProviderKind,
        change: ProviderSecretChange,
    },
    PreviewAgentBackendSettingsPatch(AgentBackendSettingsPatch),
    PreviewAgentBackendSecretChange {
        change: AgentBackendSecretChange,
    },
    PreviewAgentRun(AgentRunDraft),
    TestOpenAiBackend,
    TestLiteratureProvider {
        provider: ProviderKind,
    },
    SelectSkillsDestination,
    PreviewSkillsMaterialization {
        profile: ProfileKind,
    },
    VerifySkillsMaterialization,
    PreviewSkillsRemoval,
    PreviewSkillsPresetMaterialization {
        profile: ProfileKind,
        preset: SkillsDestinationPreset,
    },
    VerifySkillsPreset {
        preset: SkillsDestinationPreset,
    },
    PreviewSkillsPresetRemoval {
        preset: SkillsDestinationPreset,
    },
    VerifyManagedSkillsTarget {
        target_id: String,
    },
    PreviewManagedSkillsTargetUpdate {
        target_id: String,
    },
    PreviewManagedSkillsTargetRemoval {
        target_id: String,
    },
    PreviewManagedSkillsTargetDetach {
        target_id: String,
    },
    PreviewProviderPublicSetting {
        provider: ProviderKind,
        public_email: PrivateText,
    },
    PreviewIntegration {
        target: IntegrationTarget,
    },
    PreviewInstallRecommended,
    PreviewInstallSelected {
        selection: IntegrationSelection,
    },
    VerifyIntegrations {
        selection: IntegrationSelection,
    },
    PreviewReconcileIntegrations {
        selection: IntegrationSelection,
    },
    PreviewRemoveIntegrations {
        selection: IntegrationSelection,
    },
    ConfirmOperation {
        token: OperationToken,
    },
    CancelOperation {
        token: OperationToken,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationPreview {
    pub token: OperationToken,
    pub kind: OperationKind,
    pub title: &'static str,
    pub summary: &'static str,
    pub display_target: Option<PrivateDisplayText>,
    pub plan_digest_sha256: Option<String>,
    pub approvals_required: Vec<OperationApproval>,
    pub can_confirm: bool,
    pub blocked_reason: Option<&'static str>,
}

impl OperationPreview {
    #[must_use]
    pub fn validate(&self) -> bool {
        if self.can_confirm {
            let approvals_valid = match self.kind {
                OperationKind::LegacyMigrationStage
                | OperationKind::LegacyMigrationCleanup
                | OperationKind::LegacyMigrationRecovery => {
                    self.approvals_required == [OperationApproval::FilesystemWrite]
                        || self.approvals_required
                            == [
                                OperationApproval::FilesystemWrite,
                                OperationApproval::ClientConfigChange,
                            ]
                        || self.approvals_required
                            == [
                                OperationApproval::FilesystemWrite,
                                OperationApproval::ClientConfigChange,
                                OperationApproval::SecretStoreWrite,
                            ]
                }
                _ => self.approvals_required == self.kind.approvals(),
            };
            let display_target_valid = match self.kind {
                OperationKind::SkillsMaterialization
                | OperationKind::SkillsRemoval
                | OperationKind::SkillsDetach
                | OperationKind::CliInstall
                | OperationKind::ZoteroCompanionStage
                | OperationKind::Activation => self.display_target.is_some(),
                OperationKind::GlobalSettings
                | OperationKind::ProviderSettings
                | OperationKind::ProviderSecret
                | OperationKind::AgentBackendSettings
                | OperationKind::AgentBackendSecret
                | OperationKind::AgentRun
                | OperationKind::UpdateInstall
                | OperationKind::LegacyMigrationStage
                | OperationKind::LegacyMigrationHostActivation
                | OperationKind::LegacyMigrationCleanup
                | OperationKind::LegacyMigrationFinalize
                | OperationKind::LegacyMigrationRecovery => self.display_target.is_none(),
            };
            self.blocked_reason.is_none()
                && approvals_valid
                && display_target_valid
                && self
                    .plan_digest_sha256
                    .as_deref()
                    .is_some_and(valid_lower_sha256)
        } else {
            self.blocked_reason.is_some()
                && self.approvals_required.is_empty()
                && self.plan_digest_sha256.is_none()
        }
    }
}

fn valid_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesktopEvent {
    SnapshotReplaced(Box<DesktopSnapshotV1>),
    McpSelfTestUpdated(McpSelfTestView),
    UpdateChanged {
        update: UpdateView,
        close_requested: bool,
    },
    SkillsDestinationSelected {
        display_path: PrivateDisplayText,
        target_id: String,
    },
    ValidationFailed {
        code: &'static str,
    },
    PreviewReady(OperationPreview),
    AgentRunCompleted(AgentRunResultView),
    Completed {
        code: &'static str,
    },
    Cancelled {
        code: &'static str,
    },
    Failed {
        code: &'static str,
    },
}

pub trait DesktopService {
    fn snapshot(&mut self) -> DesktopSnapshotV1;

    fn execute(&mut self, intent: DesktopIntent) -> DesktopEvent;
}

#[cfg(test)]
pub(crate) fn sample_snapshot() -> DesktopSnapshotV1 {
    DesktopSnapshotV1 {
        schema_version: DESKTOP_SNAPSHOT_SCHEMA_VERSION,
        product: ProductView {
            version: "2.0.0-alpha.2".to_owned(),
            build: "source-build".to_owned(),
            operating_system: OperatingSystemView::Linux,
            architecture: ArchitectureView::X86_64,
            trust: ProductTrustView::SourceBuild,
        },
        content: ContentView {
            status: StatusCode::Ready,
            pack_id: "qiongli-core".to_owned(),
            content_version: "1.19.0-beta.1".to_owned(),
            entry_count: 42,
            profiles: [
                ProfileView {
                    profile: ProfileKind::SkillOnly,
                    included_resource_kinds: 4,
                },
                ProfileView {
                    profile: ProfileKind::MarketplaceLite,
                    included_resource_kinds: 7,
                },
                ProfileView {
                    profile: ProfileKind::Full,
                    included_resource_kinds: 11,
                },
            ],
            managed_skills_status: StatusCode::Ready,
            managed_skills: vec![ManagedSkillsView {
                target_id: format!("skills-target-{}", "1".repeat(64)),
                preset: SkillsDestinationPreset::QiongliManaged,
                state: ManagedSkillsStateView::Missing,
                status: StatusCode::Missing,
                profile: None,
                product_version: None,
            }],
        },
        mcp: McpView {
            status: StatusCode::Ready,
            profile: ProfileKind::MarketplaceLite,
            public_tool_count: 12,
        },
        cli: CliView {
            status: StatusCode::Missing,
            state: CliInstallStateView::Missing,
            installed_version: None,
            available_version: "2.0.0-alpha.2".to_owned(),
            symbolic_target: "<user-home>/.local/bin/qiongli",
            path_status: StatusCode::Attention,
            path_state: CliPathStateView::NotConfigured,
            reason_code: "qiongli-cli-not-installed",
            can_install: false,
            can_test: false,
        },
        zotero: ZoteroIntegrationView {
            status: StatusCode::Disabled,
            state: ZoteroIntegrationStateView::NotObserved,
            observation: ZoteroObservationView::NotObserved,
            zotero_version: None,
            connector_available: false,
            companion_available: false,
            companion_version: None,
            available_companion_version: None,
            available_companion_sha256: None,
            available_companion_size_bytes: None,
            endpoint_version: None,
            supported_endpoint_version: "2",
            supported_zotero_min_version: "8.0",
            supported_zotero_max_version: "9.0.*",
            installation_prepared: false,
            fallback_import_available: true,
            fallback_formats: ZOTERO_FALLBACK_FORMATS,
            reason_code: "zotero-integration-not-observed",
            can_prepare_install: false,
            can_reveal: false,
            can_open_zotero: false,
            can_verify: true,
        },
        config: ConfigView {
            status: StatusCode::Missing,
            revision: Some(0),
            default_profile: Some(ProfileKind::MarketplaceLite),
            secret_store: StatusCode::Unavailable,
            providers: ProviderKind::ALL.map(|provider| ProviderView {
                provider,
                enabled: false,
                readiness: ProviderReadinessView::Disabled,
                public_setting_present: false,
                secret_reference_present: false,
            }),
            openai_backend: AgentBackendView {
                enabled: false,
                readiness: AgentBackendReadinessView::Disabled,
                secret_reference_present: false,
                test_available: false,
            },
            cleanup_required: false,
        },
        update: UpdateView {
            status: StatusCode::Ready,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::Idle,
            available_version: None,
            archive_size_bytes: None,
            progress: None,
            reason_code: "update-ready",
            remediation: UpdateRemediation::None,
            can_select_stream: true,
            can_check: true,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
        legacy_migration: LegacyMigrationView {
            state: LegacyMigrationStateView::NotDetected,
            next_action: LegacyMigrationActionView::None,
            migration_id: None,
            detected_items: 0,
            eligible_items: 0,
            review_items: 0,
            reason_code: "legacy-migration-not-detected",
            provider_conflicts: Vec::new(),
        },
        integrations: [
            IntegrationView {
                target: IntegrationTarget::Codex,
                client_version: None,
                compatibility: ClientCompatibilityView::NotEvaluated,
                installed_plugin_version: None,
                available_plugin_version: ProductVersionView {
                    major: 2,
                    minor: 0,
                    patch: 0,
                    channel: ProductVersionChannelView::Alpha,
                    prerelease_number: Some(2),
                },
                discovery: IntegrationDiscoveryState::NotDiscovered,
                candidate_required: false,
                migration: IntegrationMigrationView {
                    state: IntegrationMigrationStateView::NotDetected,
                    detected_items: 0,
                    eligible_items: 0,
                    review_items: 0,
                },
                client: StatusCode::Missing,
                overall: StatusCode::Missing,
                source: StatusCode::Missing,
                skills: StatusCode::Missing,
                marketplace: StatusCode::Missing,
                direct_package: None,
                registration: StatusCode::Missing,
                activation_status: StatusCode::Missing,
                activation_observation: IntegrationObservationView::Missing,
                mcp_attachment: StatusCode::Missing,
                mcp_attachment_observation: IntegrationObservationView::Missing,
                symbolic_location: SymbolicLocation::CodexMarketplace,
                activation: ActivationPolicy::ClientActionRequired,
                ownership: IntegrationOwnershipView::NotInstalled,
                next_action: IntegrationActionView::InspectOnly,
                evidence_code: "client-not-detected",
                path_count: 0,
                paths: EMPTY_INTEGRATION_PATHS,
            },
            IntegrationView {
                target: IntegrationTarget::ClaudeCode,
                client_version: None,
                compatibility: ClientCompatibilityView::NotEvaluated,
                installed_plugin_version: None,
                available_plugin_version: ProductVersionView {
                    major: 2,
                    minor: 0,
                    patch: 0,
                    channel: ProductVersionChannelView::Alpha,
                    prerelease_number: Some(2),
                },
                discovery: IntegrationDiscoveryState::NotDiscovered,
                candidate_required: false,
                migration: IntegrationMigrationView {
                    state: IntegrationMigrationStateView::NotDetected,
                    detected_items: 0,
                    eligible_items: 0,
                    review_items: 0,
                },
                client: StatusCode::Missing,
                overall: StatusCode::Missing,
                source: StatusCode::Missing,
                skills: StatusCode::Missing,
                marketplace: StatusCode::Missing,
                direct_package: Some(StatusCode::Missing),
                registration: StatusCode::Missing,
                activation_status: StatusCode::Missing,
                activation_observation: IntegrationObservationView::Missing,
                mcp_attachment: StatusCode::Missing,
                mcp_attachment_observation: IntegrationObservationView::Missing,
                symbolic_location: SymbolicLocation::ClaudeMarketplace,
                activation: ActivationPolicy::ReloadOrClientActionRequired,
                ownership: IntegrationOwnershipView::NotInstalled,
                next_action: IntegrationActionView::InspectOnly,
                evidence_code: "client-not-detected",
                path_count: 0,
                paths: EMPTY_INTEGRATION_PATHS,
            },
        ],
        diagnostics: [
            DiagnosticCheckView {
                check: DiagnosticCheckId::EmbeddedContent,
                status: StatusCode::Ready,
                blocking: false,
                remediation: RemediationCode::None,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::GlobalConfig,
                status: StatusCode::Missing,
                blocking: false,
                remediation: RemediationCode::None,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::SecureStore,
                status: StatusCode::Unavailable,
                blocking: false,
                remediation: RemediationCode::SecureStoreNotImplemented,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::ManagedContent,
                status: StatusCode::Ready,
                blocking: false,
                remediation: RemediationCode::None,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::CodexLocal,
                status: StatusCode::Missing,
                blocking: false,
                remediation: RemediationCode::InspectCodexLocal,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::ClaudeCodeLocal,
                status: StatusCode::Missing,
                blocking: false,
                remediation: RemediationCode::InspectClaudeCodeLocal,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::LiteMcp,
                status: StatusCode::Ready,
                blocking: false,
                remediation: RemediationCode::None,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::LiteratureProviders,
                status: StatusCode::Ready,
                blocking: false,
                remediation: RemediationCode::None,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::UpdateRecovery,
                status: StatusCode::Ready,
                blocking: false,
                remediation: RemediationCode::None,
            },
            DiagnosticCheckView {
                check: DiagnosticCheckId::FullRuntime,
                status: StatusCode::Disabled,
                blocking: false,
                remediation: RemediationCode::UpgradeToFullRuntime,
            },
        ],
        diagnostic_paths: vec![DiagnosticPathView {
            id: "config-root".to_owned(),
            label: "Qiongli 2 configuration root".to_owned(),
            symbolic_path: "<user-home>/.config/qiongli/v2".to_owned(),
            exact_path: PrivateDisplayText::new("/Users/example/.config/qiongli/v2".to_owned()),
            reveal_path: PrivateDisplayText::new("/Users/example/.config/qiongli/v2".to_owned()),
            details: "Group: Configuration · Scope: Managed · Source: OfficialDefault · Type: Directory · Owner: CurrentUser · Writability: Writable · Safety: Supported".to_owned(),
            selected: true,
            status: StatusCode::Ready,
            resolved_target: None,
        }],
        capabilities: CapabilityView {
            refresh: true,
            config_edit: true,
            skills_materialize: true,
            provider_preview: true,
            mcp_self_test: true,
            integration_discovery: true,
            integration_preview: true,
            apply: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmable_preview() -> OperationPreview {
        OperationPreview {
            token: OperationToken::new(1),
            kind: OperationKind::Activation,
            title: "Activation preview",
            summary: "A bounded activation preview.",
            display_target: Some(PrivateDisplayText::new(
                "Selected managed client locations".to_owned(),
            )),
            plan_digest_sha256: Some("a".repeat(64)),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        }
    }

    fn valid_mcp_self_test() -> McpSelfTestView {
        McpSelfTestView {
            state: McpSelfTestState::Passed,
            checks: McpSelfTestCheckId::ALL.map(|check| McpSelfTestCheckView {
                check,
                status: StatusCode::Ready,
                code: "check-ready",
                remediation: "none",
            }),
            public_tool_count: 12,
            enabled_provider_count: 2,
            ready_provider_count: 2,
            discovered_client_count: 1,
            registered_client_count: 1,
        }
    }

    #[test]
    fn snapshot_rejects_noncanonical_order_and_unbounded_text() {
        let mut snapshot = sample_snapshot();
        assert_eq!(snapshot.validate(), Ok(()));

        snapshot.config.providers.swap(0, 1);
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("provider-order-invalid")
        );

        snapshot = sample_snapshot();
        snapshot.integrations.swap(0, 1);
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("integration-order-invalid")
        );

        snapshot = sample_snapshot();
        snapshot.product.version = "x".repeat(MAX_DISPLAY_TEXT_BYTES + 1);
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("product-version-invalid")
        );

        snapshot = sample_snapshot();
        snapshot.product.version = "/private/path".to_owned();
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("product-version-invalid")
        );

        snapshot = sample_snapshot();
        snapshot.integrations[0].path_count = 1;
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("integration-path-order-invalid")
        );

        snapshot = sample_snapshot();
        snapshot.integrations[0].path_count = 1;
        snapshot.integrations[0].paths[0] = Some(IntegrationPathView {
            surface: IntegrationPathSurfaceView::SkillsRoot,
            scope: IntegrationPathScopeView::User,
            source: IntegrationPathSourceView::OfficialDefault,
            state: StatusCode::Ready,
            management: IntegrationPathManagementView::Supported,
            selected: true,
            symbolic_path: "/private/path",
        });
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("integration-symbolic-path-invalid")
        );
    }

    #[test]
    fn snapshot_requires_causal_integration_actions_and_upgrade_state() {
        let mut snapshot = sample_snapshot();
        snapshot.integrations[0].next_action = IntegrationActionView::Current;
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("integration-state-invalid")
        );

        let integration = &mut snapshot.integrations[0];
        integration.compatibility = ClientCompatibilityView::Unsupported;
        integration.discovery = IntegrationDiscoveryState::DiscoveredUnmanaged;
        integration.client = StatusCode::Ready;
        integration.overall = StatusCode::Blocked;
        integration.registration = StatusCode::Missing;
        integration.ownership = IntegrationOwnershipView::NotInstalled;
        integration.next_action = IntegrationActionView::UpgradeClient;
        integration.evidence_code = "client-version-below-supported-minimum";
        assert_eq!(snapshot.validate(), Ok(()));

        snapshot.integrations[0].next_action = IntegrationActionView::InstallReady;
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("integration-state-invalid")
        );
    }

    #[test]
    fn snapshot_requires_truthful_zotero_compatibility_evidence() {
        let mut snapshot = sample_snapshot();
        snapshot.zotero = ZoteroIntegrationView {
            status: StatusCode::Ready,
            state: ZoteroIntegrationStateView::Ready,
            observation: ZoteroObservationView::Observed,
            zotero_version: Some("8.0.0".to_owned()),
            connector_available: true,
            companion_available: true,
            companion_version: Some("0.3.0".to_owned()),
            available_companion_version: Some("0.3.0".to_owned()),
            available_companion_sha256: Some("a".repeat(64)),
            available_companion_size_bytes: Some(32_768),
            endpoint_version: Some("2".to_owned()),
            supported_endpoint_version: "2",
            supported_zotero_min_version: "8.0",
            supported_zotero_max_version: "9.0.*",
            installation_prepared: false,
            fallback_import_available: true,
            fallback_formats: ZOTERO_FALLBACK_FORMATS,
            reason_code: "zotero-companion-ready",
            can_prepare_install: false,
            can_reveal: false,
            can_open_zotero: true,
            can_verify: true,
        };
        assert_eq!(snapshot.validate(), Ok(()));

        snapshot.zotero.endpoint_version = Some("1".to_owned());
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("zotero-integration-view-invalid")
        );

        snapshot.zotero.state = ZoteroIntegrationStateView::CompanionIncompatible;
        snapshot.zotero.status = StatusCode::Attention;
        snapshot.zotero.reason_code = "zotero-companion-endpoint-incompatible";
        assert_eq!(snapshot.validate(), Ok(()));

        snapshot.zotero.state = ZoteroIntegrationStateView::ZoteroIncompatible;
        snapshot.zotero.zotero_version = Some("7.0.15".to_owned());
        snapshot.zotero.can_prepare_install = false;
        snapshot.zotero.reason_code = "zotero-version-incompatible";
        assert_eq!(snapshot.validate(), Ok(()));

        snapshot.zotero.fallback_import_available = false;
        assert_eq!(
            snapshot.validate().map_err(SnapshotValidationError::code),
            Err("zotero-integration-view-invalid")
        );
    }

    #[test]
    fn mcp_self_test_rejects_reordered_or_unbounded_results() {
        let mut view = valid_mcp_self_test();
        assert!(view.validate());

        view.checks.swap(0, 1);
        assert!(!view.validate());

        let mut view = valid_mcp_self_test();
        view.registered_client_count = 2;
        assert!(!view.validate());

        let mut view = valid_mcp_self_test();
        view.checks[0].code = "NOT_CANONICAL";
        assert!(!view.validate());
    }

    #[test]
    fn operation_preview_requires_exact_digest_and_activation_approvals() {
        let mut preview = confirmable_preview();
        assert!(preview.validate());

        preview.plan_digest_sha256 = Some("A".repeat(64));
        assert!(!preview.validate());

        preview = confirmable_preview();
        preview.approvals_required.swap(0, 1);
        assert!(!preview.validate());

        preview = confirmable_preview();
        preview.blocked_reason = Some("unexpected-block");
        assert!(!preview.validate());

        preview = confirmable_preview();
        preview.can_confirm = false;
        preview.plan_digest_sha256 = None;
        preview.approvals_required.clear();
        preview.blocked_reason = Some("activation-unavailable");
        assert!(preview.validate());

        preview = confirmable_preview();
        preview.kind = OperationKind::GlobalSettings;
        assert!(!preview.validate());
        preview.approvals_required = vec![OperationApproval::ClientConfigChange];
        preview.display_target = None;
        assert!(preview.validate());

        preview = confirmable_preview();
        preview.kind = OperationKind::AgentRun;
        preview.approvals_required = vec![OperationApproval::NetworkRequest];
        preview.display_target = None;
        assert!(preview.validate());

        preview = confirmable_preview();
        preview.kind = OperationKind::SkillsMaterialization;
        preview.approvals_required = vec![OperationApproval::FilesystemWrite];
        preview.display_target = None;
        assert!(!preview.validate());
        preview.display_target = Some(PrivateDisplayText::new("/selected-folder".to_owned()));
        assert!(preview.validate());
        assert!(!format!("{preview:?}").contains("selected-folder"));
    }

    #[test]
    fn update_view_rejects_unsafe_versions_progress_and_actions() {
        let mut update = sample_snapshot().update;
        assert!(update.validate());

        update.available_version = Some("/private/update".to_owned());
        assert!(!update.validate());

        update = sample_snapshot().update;
        update.archive_size_bytes = Some(MAX_UPDATE_ARCHIVE_BYTES + 1);
        assert!(!update.validate());

        update = sample_snapshot().update;
        update.progress = Some(UpdateProgressView {
            completed_steps: 5,
            total_steps: 4,
            label: "Invalid progress",
            indeterminate: false,
        });
        assert!(!update.validate());

        update = sample_snapshot().update;
        update.phase = UpdatePhaseView::Checking;
        update.can_check = true;
        assert!(!update.validate());

        update = sample_snapshot().update;
        update.phase = UpdatePhaseView::ReadyToInstall;
        update.can_check = false;
        update.can_select_stream = false;
        update.can_install = true;
        assert!(update.validate());
    }
}
