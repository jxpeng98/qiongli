mod codex;
mod error;
mod grant;
mod identity;
mod plan;
mod transaction;

pub use codex::{
    CODEX_ADAPTER_SCHEMA_VERSION, CODEX_MARKETPLACE_SYMBOLIC_PATH,
    CODEX_PLUGIN_SOURCE_MARKETPLACE_PATH, CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
    CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION, CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
    CodexAdapterError, CodexDiscoverySummaryV1, CodexMarketplaceState, CodexRegistrationCommit,
    CodexRegistrationDisposition, CodexRegistrationEffect, CodexRegistrationExecutor,
    CodexRegistrationLifecycleCommit, CodexRegistrationLifecycleDisposition,
    CodexRegistrationLifecycleKind, CodexRegistrationLifecycleReceiptV1, CodexRegistrationPreview,
    CodexRegistrationReceiptV1, CodexRegistrationState, CodexRegistrationStateV1,
    CodexRegistrationVerification, CodexSourceState, CodexUserTarget, discover_codex_user,
    preview_codex_registration,
};
pub use error::PlatformError;
pub use grant::{
    GrantMode, GrantSignatureV1, GrantVerificationContext, IntegrationScope, LaunchGrantV1,
    SignatureAlgorithm, SignedLaunchGrantV1, TrustedPublicKey, VerifiedLaunchGrant,
    launch_grant_signing_bytes,
};
pub use identity::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, OperatingSystem, ProductId,
    ReleaseChannel,
};
pub use plan::{
    AllowedRootV1, ApprovalRequirement, HostAction, InstallActionV1, InstallOperationV1,
    InstallPlanDraftV1, InstallPlanMetadataV1, InstallPlanV1, InstallScope, LocalSurface,
    LocalTargetFamily, OwnershipMarkerV1, PlanStateV1, SymbolicRoot, TargetDescriptorV1,
    VerifiedInstallPlan, observed_plan_state_sha256,
};
pub use transaction::{
    ApprovedInstallPlan, ApprovedManagedRoot, INSTALL_JOURNAL_SCHEMA_VERSION,
    INSTALL_RECEIPT_SCHEMA_VERSION, InstallCommit, InstallDisposition, InstallLifecycleKind,
    InstallLifecycleReceiptV1, InstallReceiptV1, InstallVerification, LifecycleCommit,
    LifecycleDisposition, MANAGED_INSTALL_STATE_SCHEMA_VERSION, ManagedInstallStateV1,
    ManagedOperationReceiptV1, ManagedResourceExecutor, TransactionError, approve_install_plan,
    approve_managed_root,
};

pub const ARTIFACT_IDENTITY_SCHEMA_VERSION: u32 = 1;
pub const LAUNCH_GRANT_SCHEMA_VERSION: u32 = 1;
pub const INSTALL_PLAN_SCHEMA_VERSION: u32 = 1;
