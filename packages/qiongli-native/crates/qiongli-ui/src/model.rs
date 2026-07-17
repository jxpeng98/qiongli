use std::fmt::{self, Debug, Formatter};

use zeroize::Zeroizing;

pub const DESKTOP_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const MAX_DISPLAY_TEXT_BYTES: usize = 128;
const MAX_CONTENT_ENTRIES: usize = 100_000;
const MAX_PUBLIC_TOOLS: usize = 256;
const MAX_RESOURCE_KINDS: usize = 32;
const MAX_PROVIDERS: usize = 5;
const MAX_INTEGRATIONS: usize = 2;
const MAX_UPDATE_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopSection {
    Overview,
    Skills,
    Mcp,
    Providers,
    Integrations,
    Diagnostics,
}

impl DesktopSection {
    pub const ALL: [Self; 6] = [
        Self::Overview,
        Self::Skills,
        Self::Mcp,
        Self::Providers,
        Self::Integrations,
        Self::Diagnostics,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Skills => "Skills",
            Self::Mcp => "MCP",
            Self::Providers => "Providers",
            Self::Integrations => "Integrations",
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
    CodexLocal,
    ClaudeCodeLocal,
}

impl DiagnosticCheckId {
    pub const ALL: [Self; 5] = [
        Self::EmbeddedContent,
        Self::GlobalConfig,
        Self::SecureStore,
        Self::CodexLocal,
        Self::ClaudeCodeLocal,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmbeddedContent => "Embedded content",
            Self::GlobalConfig => "Global configuration",
            Self::SecureStore => "Secure store",
            Self::CodexLocal => "Codex local integration",
            Self::ClaudeCodeLocal => "Claude Code local integration",
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
    InspectCodexLocal,
    InspectClaudeCodeLocal,
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
            Self::InspectCodexLocal => "inspect-codex-local",
            Self::InspectClaudeCodeLocal => "inspect-claude-code-local",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductView {
    pub version: String,
    pub operating_system: OperatingSystemView,
    pub architecture: ArchitectureView,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpView {
    pub status: StatusCode,
    pub profile: ProfileKind,
    pub public_tool_count: usize,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigView {
    pub status: StatusCode,
    pub revision: Option<u64>,
    pub default_profile: Option<ProfileKind>,
    pub secret_store: StatusCode,
    pub providers: [ProviderView; 5],
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
pub struct IntegrationView {
    pub target: IntegrationTarget,
    pub discovery: IntegrationDiscoveryState,
    pub candidate_required: bool,
    pub overall: StatusCode,
    pub source: StatusCode,
    pub marketplace: StatusCode,
    pub direct_package: Option<StatusCode>,
    pub registration: StatusCode,
    pub symbolic_location: SymbolicLocation,
    pub activation: ActivationPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticCheckView {
    pub check: DiagnosticCheckId,
    pub status: StatusCode,
    pub blocking: bool,
    pub remediation: RemediationCode,
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
    pub config: ConfigView,
    pub update: UpdateView,
    pub integrations: [IntegrationView; 2],
    pub diagnostics: [DiagnosticCheckView; 5],
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
        validate_pack_id(&self.content.pack_id)?;
        validate_version_text(&self.content.content_version, "content-version-invalid")?;
        if !(1..=MAX_CONTENT_ENTRIES).contains(&self.content.entry_count) {
            return Err(SnapshotValidationError::new("content-entry-count-invalid"));
        }
        if !(1..=MAX_PUBLIC_TOOLS).contains(&self.mcp.public_tool_count) {
            return Err(SnapshotValidationError::new("mcp-tool-count-invalid"));
        }
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
        if self.config.providers.map(|provider| provider.provider) != ProviderKind::ALL {
            return Err(SnapshotValidationError::new("provider-order-invalid"));
        }
        if !self.update.validate() {
            return Err(SnapshotValidationError::new("update-view-invalid"));
        }
        if self.integrations.map(|integration| integration.target) != IntegrationTarget::ALL {
            return Err(SnapshotValidationError::new("integration-order-invalid"));
        }
        if self.diagnostics.map(|diagnostic| diagnostic.check) != DiagnosticCheckId::ALL {
            return Err(SnapshotValidationError::new("diagnostic-order-invalid"));
        }
        Ok(())
    }
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

fn validate_version_text(value: &str, code: &'static str) -> Result<(), SnapshotValidationError> {
    validate_display_text(value, code)?;
    if !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
    {
        return Err(SnapshotValidationError::new(code));
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
    pub providers_enabled: [bool; 5],
    pub openalex_email: PublicSettingChange,
    pub crossref_email: PublicSettingChange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationToken(u128);

impl OperationToken {
    #[must_use]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationApproval {
    FilesystemWrite,
    ClientConfigChange,
    HostTrust,
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
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationKind {
    Activation,
    GlobalSettings,
    SkillsMaterialization,
    SkillsRemoval,
    UpdateInstall,
}

impl OperationKind {
    #[must_use]
    const fn approvals(self) -> &'static [OperationApproval] {
        match self {
            Self::Activation => &OperationApproval::ACTIVATION,
            Self::GlobalSettings => &[OperationApproval::ClientConfigChange],
            Self::SkillsMaterialization | Self::SkillsRemoval | Self::UpdateInstall => {
                &[OperationApproval::FilesystemWrite]
            }
        }
    }
}

pub enum DesktopIntent {
    Refresh,
    RunLiteMcpSelfTest,
    PollLiteMcpSelfTest,
    CancelLiteMcpSelfTest,
    RefreshIntegrationDiscovery,
    SelectUpdateStream {
        stream: UpdateStreamView,
    },
    CheckForUpdates,
    PrepareUpdate,
    PollUpdate,
    CancelUpdate,
    PreviewUpdateInstall,
    PreviewGlobalSettingsPatch(GlobalSettingsPatch),
    SelectSkillsDestination,
    PreviewSkillsMaterialization {
        profile: ProfileKind,
    },
    VerifySkillsMaterialization,
    PreviewSkillsRemoval,
    PreviewProviderPublicSetting {
        provider: ProviderKind,
        public_email: PrivateText,
    },
    PreviewIntegration {
        target: IntegrationTarget,
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
    pub(crate) fn validate(&self) -> bool {
        if self.can_confirm {
            let display_target_valid = match self.kind {
                OperationKind::SkillsMaterialization | OperationKind::SkillsRemoval => {
                    self.display_target.is_some()
                }
                OperationKind::Activation
                | OperationKind::GlobalSettings
                | OperationKind::UpdateInstall => self.display_target.is_none(),
            };
            self.blocked_reason.is_none()
                && self.approvals_required == self.kind.approvals()
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
    SnapshotReplaced(DesktopSnapshotV1),
    McpSelfTestUpdated(McpSelfTestView),
    UpdateChanged {
        update: UpdateView,
        close_requested: bool,
    },
    SkillsDestinationSelected {
        display_path: PrivateDisplayText,
    },
    ValidationFailed {
        code: &'static str,
    },
    PreviewReady(OperationPreview),
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
            version: "2.0.0-alpha.1".to_owned(),
            operating_system: OperatingSystemView::Linux,
            architecture: ArchitectureView::X86_64,
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
        },
        mcp: McpView {
            status: StatusCode::Ready,
            profile: ProfileKind::MarketplaceLite,
            public_tool_count: 12,
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
        integrations: [
            IntegrationView {
                target: IntegrationTarget::Codex,
                discovery: IntegrationDiscoveryState::NotDiscovered,
                candidate_required: false,
                overall: StatusCode::Missing,
                source: StatusCode::Missing,
                marketplace: StatusCode::Missing,
                direct_package: None,
                registration: StatusCode::Missing,
                symbolic_location: SymbolicLocation::CodexMarketplace,
                activation: ActivationPolicy::ClientActionRequired,
            },
            IntegrationView {
                target: IntegrationTarget::ClaudeCode,
                discovery: IntegrationDiscoveryState::NotDiscovered,
                candidate_required: false,
                overall: StatusCode::Missing,
                source: StatusCode::Missing,
                marketplace: StatusCode::Missing,
                direct_package: Some(StatusCode::Missing),
                registration: StatusCode::Missing,
                symbolic_location: SymbolicLocation::ClaudeMarketplace,
                activation: ActivationPolicy::ReloadOrClientActionRequired,
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
        ],
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
            display_target: None,
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
        assert!(preview.validate());

        preview = confirmable_preview();
        preview.kind = OperationKind::SkillsMaterialization;
        preview.approvals_required = vec![OperationApproval::FilesystemWrite];
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
