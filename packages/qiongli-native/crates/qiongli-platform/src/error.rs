use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PlatformError {
    #[error("artifact identity is invalid")]
    InvalidArtifactIdentity,
    #[error("artifact profile is unavailable")]
    UnsupportedArtifactProfile,
    #[error("launch grant input is too large")]
    LaunchGrantTooLarge,
    #[error("launch grant JSON is invalid")]
    InvalidLaunchGrantJson,
    #[error("launch grant schema is unsupported")]
    InvalidLaunchGrantSchema,
    #[error("launch grant is invalid")]
    InvalidLaunchGrant,
    #[error("launch grant key is not trusted")]
    LaunchGrantKeyUntrusted,
    #[error("launch grant signature is invalid")]
    LaunchGrantSignatureInvalid,
    #[error("launch grant is not yet valid")]
    LaunchGrantNotYetValid,
    #[error("launch grant has expired")]
    LaunchGrantExpired,
    #[error("launch grant generation is stale")]
    LaunchGrantReplayed,
    #[error("launch grant artifact does not match")]
    LaunchGrantArtifactMismatch,
    #[error("launch grant binary does not match")]
    LaunchGrantBinaryMismatch,
    #[error("launch grant content does not match")]
    LaunchGrantContentMismatch,
    #[error("launch grant mode is unavailable")]
    LaunchGrantModeUnavailable,
    #[error("launch grant integration scope is unavailable")]
    LaunchGrantScopeUnavailable,
    #[error("install plan input is too large")]
    InstallPlanTooLarge,
    #[error("install plan JSON is invalid")]
    InvalidInstallPlanJson,
    #[error("install plan schema is unsupported")]
    InvalidInstallPlanSchema,
    #[error("install plan is invalid")]
    InvalidInstallPlan,
    #[error("install plan semantic digest does not match")]
    InstallPlanDigestMismatch,
    #[error("install plan is not yet valid")]
    InstallPlanNotYetValid,
    #[error("install plan has expired")]
    InstallPlanExpired,
    #[error("install plan target does not match")]
    InstallPlanTargetMismatch,
    #[error("canonical serialization failed")]
    CanonicalSerializationFailed,
}

impl PlatformError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidArtifactIdentity => "invalid-artifact-identity",
            Self::UnsupportedArtifactProfile => "unsupported-artifact-profile",
            Self::LaunchGrantTooLarge => "launch-grant-too-large",
            Self::InvalidLaunchGrantJson => "invalid-launch-grant-json",
            Self::InvalidLaunchGrantSchema => "invalid-launch-grant-schema",
            Self::InvalidLaunchGrant => "invalid-launch-grant",
            Self::LaunchGrantKeyUntrusted => "launch-grant-key-untrusted",
            Self::LaunchGrantSignatureInvalid => "launch-grant-signature-invalid",
            Self::LaunchGrantNotYetValid => "launch-grant-not-yet-valid",
            Self::LaunchGrantExpired => "launch-grant-expired",
            Self::LaunchGrantReplayed => "launch-grant-replayed",
            Self::LaunchGrantArtifactMismatch => "launch-grant-artifact-mismatch",
            Self::LaunchGrantBinaryMismatch => "launch-grant-binary-mismatch",
            Self::LaunchGrantContentMismatch => "launch-grant-content-mismatch",
            Self::LaunchGrantModeUnavailable => "launch-grant-mode-unavailable",
            Self::LaunchGrantScopeUnavailable => "launch-grant-scope-unavailable",
            Self::InstallPlanTooLarge => "install-plan-too-large",
            Self::InvalidInstallPlanJson => "invalid-install-plan-json",
            Self::InvalidInstallPlanSchema => "invalid-install-plan-schema",
            Self::InvalidInstallPlan => "invalid-install-plan",
            Self::InstallPlanDigestMismatch => "install-plan-digest-mismatch",
            Self::InstallPlanNotYetValid => "install-plan-not-yet-valid",
            Self::InstallPlanExpired => "install-plan-expired",
            Self::InstallPlanTargetMismatch => "install-plan-target-mismatch",
            Self::CanonicalSerializationFailed => "canonical-serialization-failed",
        }
    }
}
