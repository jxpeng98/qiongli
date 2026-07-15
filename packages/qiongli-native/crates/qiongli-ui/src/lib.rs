mod app;
mod model;

pub use app::{QiongliDesktopApp, run_native};
pub use model::{
    ActivationPolicy, ArchitectureView, CapabilityView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopSection, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, GlobalSettingsPatch,
    IntegrationTarget, IntegrationView, McpView, OperatingSystemView, OperationApproval,
    OperationKind, OperationPreview, OperationToken, PrivateDisplayText, PrivateText, ProductView,
    ProfileKind, ProfileView, ProviderKind, ProviderReadinessView, ProviderView,
    PublicSettingChange, RemediationCode, SnapshotValidationError, StatusCode, SymbolicLocation,
};
