mod app;
mod model;

pub use app::{QiongliDesktopApp, run_native};
pub use model::{
    ActivationPolicy, ArchitectureView, CapabilityView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopSection, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, IntegrationTarget, IntegrationView,
    McpView, OperatingSystemView, OperationPreview, OperationToken, PrivateText, ProductView,
    ProfileKind, ProfileView, ProviderKind, ProviderReadinessView, ProviderView, RemediationCode,
    SnapshotValidationError, StatusCode, SymbolicLocation,
};
