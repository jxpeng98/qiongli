mod error;
mod grant;
mod identity;
mod plan;

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
    VerifiedInstallPlan,
};

pub const ARTIFACT_IDENTITY_SCHEMA_VERSION: u32 = 1;
pub const LAUNCH_GRANT_SCHEMA_VERSION: u32 = 1;
pub const INSTALL_PLAN_SCHEMA_VERSION: u32 = 1;
