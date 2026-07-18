mod app;
mod model;

pub use app::{
    DesktopApplicationMetadata, QiongliDesktopApp, native_application_icon, run_native_application,
};
pub use model::{
    ActivationPolicy, ArchitectureView, CapabilityView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopSection, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, EMPTY_INTEGRATION_PATHS,
    GlobalSettingsPatch, IntegrationActionView, IntegrationDiscoveryState,
    IntegrationOwnershipView, IntegrationPathManagementView, IntegrationPathScopeView,
    IntegrationPathSourceView, IntegrationPathSurfaceView, IntegrationPathView,
    IntegrationSelection, IntegrationTarget, IntegrationView, MAX_INTEGRATION_PATHS,
    McpSelfTestCheckId, McpSelfTestCheckView, McpSelfTestState, McpSelfTestView, McpView,
    OperatingSystemView, OperationApproval, OperationKind, OperationPreview, OperationToken,
    PrivateDisplayText, PrivateText, ProductView, ProfileKind, ProfileView, ProviderKind,
    ProviderReadinessView, ProviderView, PublicSettingChange, RemediationCode,
    SkillsDestinationPreset, SkillsInstallMethodView, SnapshotValidationError, StatusCode,
    SymbolicLocation, UpdatePhaseView, UpdateProgressView, UpdateRemediation, UpdateStreamView,
    UpdateView,
};
