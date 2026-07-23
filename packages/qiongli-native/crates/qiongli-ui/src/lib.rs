#[cfg(feature = "egui-presentation")]
mod app;
mod model;

#[cfg(feature = "egui-presentation")]
pub use app::{
    DesktopApplicationMetadata, QiongliDesktopApp, native_application_icon, run_native_application,
};
pub use model::{
    ActivationPolicy, AgentBackendReadinessView, AgentBackendSecretChange,
    AgentBackendSettingsPatch, AgentBackendView, ArchitectureView, CapabilityView,
    ClientCompatibilityView, ClientVersionView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopSection, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, DiagnosticPathView,
    EMPTY_INTEGRATION_PATHS, GlobalSettingsPatch, IntegrationActionView, IntegrationDiscoveryState,
    IntegrationObservationView, IntegrationOwnershipView, IntegrationPathManagementView,
    IntegrationPathScopeView, IntegrationPathSourceView, IntegrationPathSurfaceView,
    IntegrationPathView, IntegrationSelection, IntegrationTarget, IntegrationView,
    MAX_DIAGNOSTIC_PATHS, MAX_INTEGRATION_PATHS, McpSelfTestCheckId, McpSelfTestCheckView,
    McpSelfTestState, McpSelfTestView, McpView, OperatingSystemView, OperationApproval,
    OperationKind, OperationPreview, OperationToken, PrivateDisplayText, PrivateText,
    ProductTrustView, ProductVersionChannelView, ProductVersionView, ProductView, ProfileKind,
    ProfileView, ProviderKind, ProviderReadinessView, ProviderSecretChange, ProviderSettingsPatch,
    ProviderView, PublicSettingChange, RemediationCode, SkillsDestinationPreset,
    SkillsInstallMethodView, SnapshotValidationError, StatusCode, SymbolicLocation,
    UpdatePhaseView, UpdateProgressView, UpdateRemediation, UpdateStreamView, UpdateView,
};
