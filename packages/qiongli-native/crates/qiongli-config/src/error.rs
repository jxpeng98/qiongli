use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceStage {
    Inspect,
    CreateStore,
    AcquireLock,
    ReadCurrent,
    WriteStaging,
    SyncStaging,
    CreateRecovery,
    Activate,
    SyncDirectory,
    Rollback,
    Cleanup,
}

impl PersistenceStage {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::CreateStore => "create-store",
            Self::AcquireLock => "acquire-lock",
            Self::ReadCurrent => "read-current",
            Self::WriteStaging => "write-staging",
            Self::SyncStaging => "sync-staging",
            Self::CreateRecovery => "create-recovery",
            Self::Activate => "activate",
            Self::SyncDirectory => "sync-directory",
            Self::Rollback => "rollback",
            Self::Cleanup => "cleanup",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    InvalidConfigHome,
    HomeUnavailable,
    InvalidDocumentKind,
    UnsupportedSchema {
        observed: Option<u64>,
    },
    InvalidDocument,
    DocumentTooLarge,
    UnsafeManagedPath,
    InsecurePermissions,
    LockBusy,
    RevisionConflict {
        observed: u64,
    },
    RevisionExhausted,
    PersistenceFailed {
        stage: PersistenceStage,
        kind: io::ErrorKind,
    },
    RecoveryRequired,
    UnsupportedPlatformSecurity,
}

impl ConfigError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidConfigHome => "invalid-config-home",
            Self::HomeUnavailable => "home-unavailable",
            Self::InvalidDocumentKind => "invalid-document-kind",
            Self::UnsupportedSchema { .. } => "unsupported-schema",
            Self::InvalidDocument => "invalid-document",
            Self::DocumentTooLarge => "document-too-large",
            Self::UnsafeManagedPath => "unsafe-managed-path",
            Self::InsecurePermissions => "insecure-permissions",
            Self::LockBusy => "lock-busy",
            Self::RevisionConflict { .. } => "revision-conflict",
            Self::RevisionExhausted => "revision-exhausted",
            Self::PersistenceFailed { .. } => "persistence-failed",
            Self::RecoveryRequired => "recovery-required",
            Self::UnsupportedPlatformSecurity => "unsupported-platform-security",
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())?;
        match self {
            Self::UnsupportedSchema {
                observed: Some(observed),
            } => write!(formatter, " (observed {observed})"),
            Self::RevisionConflict { observed } => {
                write!(formatter, " (observed revision {observed})")
            }
            Self::PersistenceFailed { stage, kind } => {
                write!(formatter, " ({}: {kind:?})", stage.code())
            }
            _ => Ok(()),
        }
    }
}

impl Error for ConfigError {}
