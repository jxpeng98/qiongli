use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectError {
    HomeUnavailable,
    InvalidProjectRoot,
    UnsafeProjectRoot,
    ProjectRootMissing,
    ProjectRootConflict,
    ProjectManifestMissing,
    ProjectManifestConflict,
    ProjectAlreadyRegistered,
    ProjectNotRegistered,
    ProjectIdentityConflict,
    InvalidProjectDocument,
    InvalidLibraryDocument,
    InvalidCaptureDocument,
    CaptureAlreadyApplied,
    CaptureIdentityConflict,
    PortablePackageInvalid,
    MigrationSourceInvalid,
    DocumentTooLarge,
    LibraryFull,
    RevisionConflict,
    PlanMismatch,
    ApprovalRequired,
    LockBusy,
    RecoveryRequired,
    UnsupportedPlatformSecurity,
    RandomUnavailable,
    PersistenceFailed(io::ErrorKind),
}

impl ProjectError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::HomeUnavailable => "project-home-unavailable",
            Self::InvalidProjectRoot => "project-root-invalid",
            Self::UnsafeProjectRoot => "project-root-unsafe",
            Self::ProjectRootMissing => "project-root-missing",
            Self::ProjectRootConflict => "project-root-conflict",
            Self::ProjectManifestMissing => "project-manifest-missing",
            Self::ProjectManifestConflict => "project-manifest-conflict",
            Self::ProjectAlreadyRegistered => "project-already-registered",
            Self::ProjectNotRegistered => "project-not-registered",
            Self::ProjectIdentityConflict => "project-identity-conflict",
            Self::InvalidProjectDocument => "project-document-invalid",
            Self::InvalidLibraryDocument => "research-library-document-invalid",
            Self::InvalidCaptureDocument => "research-capture-document-invalid",
            Self::CaptureAlreadyApplied => "research-capture-already-applied",
            Self::CaptureIdentityConflict => "research-capture-identity-conflict",
            Self::PortablePackageInvalid => "portable-project-package-invalid",
            Self::MigrationSourceInvalid => "legacy-project-migration-source-invalid",
            Self::DocumentTooLarge => "project-document-too-large",
            Self::LibraryFull => "research-library-full",
            Self::RevisionConflict => "project-revision-conflict",
            Self::PlanMismatch => "project-plan-mismatch",
            Self::ApprovalRequired => "project-filesystem-approval-required",
            Self::LockBusy => "project-library-lock-busy",
            Self::RecoveryRequired => "project-recovery-required",
            Self::UnsupportedPlatformSecurity => "unsupported-platform-security",
            Self::RandomUnavailable => "project-random-unavailable",
            Self::PersistenceFailed(_) => "project-persistence-failed",
        }
    }
}

impl Display for ProjectError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for ProjectError {}
