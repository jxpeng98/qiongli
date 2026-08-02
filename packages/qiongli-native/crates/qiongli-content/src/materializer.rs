use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use same_file::Handle;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::collector::expected_resource_kind;
use crate::loader::{LoadedResource, LoadedResourcePack, ResourcePackLoaderError};
use crate::manifest::{LogicalMode, ProfileId, ResourceKind};

pub const MATERIALIZATION_RECEIPT_VERSION: u32 = 1;
pub const MATERIALIZATION_RECEIPT_FILE: &str = ".qiongli-materialization.json";

const MAX_RECEIPT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_MANAGED_ENTRIES: usize = 4_096;
const MAX_MANAGED_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_MANAGED_TOTAL_BYTES: u64 = 128 * 1024 * 1024;
const MAX_MANAGED_PATH_DEPTH: usize = 32;
const MAX_TARGET_LEAF_BYTES: usize = 128;
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterializationAuthorization {
    Temporary,
    ExplicitlyApproved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializationTarget {
    path: PathBuf,
    authorization: MaterializationAuthorization,
}

impl MaterializationTarget {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn authorization(&self) -> MaterializationAuthorization {
        self.authorization
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializedEntry {
    pub path: String,
    pub mode: LogicalMode,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializationReceiptV1 {
    pub receipt_version: u32,
    pub pack_id: String,
    pub content_version: String,
    pub source_commit: String,
    pub profile: ProfileId,
    pub authorization: MaterializationAuthorization,
    pub pack_sha256: String,
    pub content_root_sha256: String,
    pub entries: Vec<MaterializedEntry>,
}

#[derive(Debug)]
pub enum MaterializationError {
    InvalidTarget {
        path: PathBuf,
        reason: &'static str,
    },
    MissingTargetParent {
        path: PathBuf,
    },
    TargetParentNotDirectory {
        path: PathBuf,
    },
    InsecureTargetParent {
        path: PathBuf,
    },
    LinkNotAllowed {
        path: PathBuf,
    },
    TargetNotDirectory {
        path: PathBuf,
    },
    MissingManagedTarget {
        path: PathBuf,
    },
    TargetBusy {
        path: PathBuf,
    },
    TargetChanged {
        path: PathBuf,
    },
    UnmanagedTarget {
        path: PathBuf,
    },
    InvalidManagedReceipt {
        path: PathBuf,
        reason: String,
    },
    ManagedTargetDrift {
        path: PathBuf,
        reason: String,
    },
    Profile(ResourcePackLoaderError),
    CanonicalJson(serde_json::Error),
    CommitFailed {
        path: PathBuf,
        kind: io::ErrorKind,
    },
    RollbackFailed {
        path: PathBuf,
        backup_path: PathBuf,
        commit_kind: io::ErrorKind,
        rollback_kind: io::ErrorKind,
    },
    CommittedWithCleanupFailure {
        path: PathBuf,
        backup_path: Option<PathBuf>,
        detail: String,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        kind: io::ErrorKind,
    },
}

impl MaterializationError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidTarget { .. } => "invalid-materialization-target",
            Self::MissingTargetParent { .. } => "missing-materialization-parent",
            Self::TargetParentNotDirectory { .. } => "invalid-materialization-parent",
            Self::InsecureTargetParent { .. } => "insecure-materialization-parent",
            Self::LinkNotAllowed { .. } => "linked-materialization-path",
            Self::TargetNotDirectory { .. } => "invalid-materialization-target-kind",
            Self::MissingManagedTarget { .. } => "missing-managed-materialization",
            Self::TargetBusy { .. } => "materialization-target-busy",
            Self::TargetChanged { .. } => "materialization-target-changed",
            Self::UnmanagedTarget { .. } => "unmanaged-materialization-target",
            Self::InvalidManagedReceipt { .. } => "invalid-materialization-receipt",
            Self::ManagedTargetDrift { .. } => "materialization-target-drift",
            Self::Profile(_) => "invalid-content-profile",
            Self::CanonicalJson(_) => "materialization-receipt-failed",
            Self::CommitFailed { .. } => "materialization-commit-failed",
            Self::RollbackFailed { .. } => "materialization-rollback-failed",
            Self::CommittedWithCleanupFailure { .. } => {
                "materialization-committed-cleanup-required"
            }
            Self::Io { .. } => "materialization-io-failed",
        }
    }

    fn io(operation: &'static str, path: &Path, error: &io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound && operation == "inspect target parent" {
            Self::MissingTargetParent {
                path: path.to_path_buf(),
            }
        } else {
            Self::Io {
                operation,
                path: path.to_path_buf(),
                kind: error.kind(),
            }
        }
    }

    fn drift(path: &Path, reason: impl Into<String>) -> Self {
        Self::ManagedTargetDrift {
            path: path.to_path_buf(),
            reason: reason.into(),
        }
    }

    fn invalid_receipt(path: &Path, reason: impl Into<String>) -> Self {
        Self::InvalidManagedReceipt {
            path: path.to_path_buf(),
            reason: reason.into(),
        }
    }
}

impl Display for MaterializationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget { path, reason } => write!(
                formatter,
                "materialization target {} is invalid: {reason}",
                path.display()
            ),
            Self::MissingTargetParent { path } => write!(
                formatter,
                "materialization target parent is missing: {}",
                path.display()
            ),
            Self::TargetParentNotDirectory { path } => write!(
                formatter,
                "materialization target parent is not a directory: {}",
                path.display()
            ),
            Self::InsecureTargetParent { path } => write!(
                formatter,
                "materialization target parent is writable by an untrusted Unix group or user: {}",
                path.display()
            ),
            Self::LinkNotAllowed { path } => write!(
                formatter,
                "materialization target contains a link or reparse point: {}",
                path.display()
            ),
            Self::TargetNotDirectory { path } => write!(
                formatter,
                "materialization target is not a directory: {}",
                path.display()
            ),
            Self::MissingManagedTarget { path } => write!(
                formatter,
                "managed materialization target is missing: {}",
                path.display()
            ),
            Self::TargetBusy { path } => write!(
                formatter,
                "another materialization transaction owns target {}",
                path.display()
            ),
            Self::TargetChanged { path } => write!(
                formatter,
                "materialization target changed during preflight: {}",
                path.display()
            ),
            Self::UnmanagedTarget { path } => write!(
                formatter,
                "refusing to replace unmanaged materialization target {}",
                path.display()
            ),
            Self::InvalidManagedReceipt { path, reason } => write!(
                formatter,
                "managed receipt at {} is invalid: {reason}",
                path.display()
            ),
            Self::ManagedTargetDrift { path, reason } => write!(
                formatter,
                "managed materialization target {} drifted: {reason}",
                path.display()
            ),
            Self::Profile(error) => write!(formatter, "profile cannot materialize: {error}"),
            Self::CanonicalJson(error) => {
                write!(
                    formatter,
                    "materialization receipt canonicalization failed: {error}"
                )
            }
            Self::CommitFailed { path, kind } => write!(
                formatter,
                "could not promote staged materialization to {}: {kind:?}",
                path.display()
            ),
            Self::RollbackFailed {
                path,
                backup_path,
                commit_kind,
                rollback_kind,
            } => write!(
                formatter,
                "materialization commit to {} failed with {commit_kind:?}, and restoring {} failed with {rollback_kind:?}",
                path.display(),
                backup_path.display()
            ),
            Self::CommittedWithCleanupFailure {
                path,
                backup_path,
                detail,
            } => {
                write!(
                    formatter,
                    "materialization committed to {}, but post-commit cleanup failed: {detail}",
                    path.display()
                )?;
                if let Some(backup_path) = backup_path {
                    write!(formatter, " (cleanup path: {})", backup_path.display())?;
                }
                Ok(())
            }
            Self::Io {
                operation,
                path,
                kind,
            } => write!(
                formatter,
                "could not {operation} materialization path {}: {kind:?}",
                path.display()
            ),
        }
    }
}

impl Error for MaterializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::CanonicalJson(error) => Some(error),
            _ => None,
        }
    }
}

/// Creates an unused target inside a private directory below the canonical operating-system
/// temporary directory.
///
/// The private parent is created with owner-only permissions on Unix. The returned target path
/// itself is not created until [`materialize_profile`] commits it.
pub fn temporary_materialization_target() -> Result<MaterializationTarget, MaterializationError> {
    let requested_temp = std::env::temp_dir();
    let canonical_temp = fs::canonicalize(&requested_temp).map_err(|error| {
        MaterializationError::io("canonicalize temporary", &requested_temp, &error)
    })?;
    validate_existing_directory_chain(&canonical_temp)?;

    for _ in 0..128 {
        let container = canonical_temp.join(format!(
            ".qiongli-materialization-{}-{}",
            std::process::id(),
            transaction_id()
        ));
        match create_directory_with_mode(&container, 0o700) {
            Ok(()) => {
                let path = container.join("materialized");
                validate_target_parent_policy(&container, MaterializationAuthorization::Temporary)?;
                return Ok(MaterializationTarget {
                    path,
                    authorization: MaterializationAuthorization::Temporary,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(MaterializationError::io(
                    "create private temporary container",
                    &container,
                    &error,
                ));
            }
        }
    }

    Err(MaterializationError::InvalidTarget {
        path: canonical_temp,
        reason: "could not allocate a unique temporary target",
    })
}

/// Approves a caller-selected target at a trusted CLI, UI, or installer boundary.
///
/// This function is a security boundary. Callers must not pass a path sourced from an
/// untrusted MCP request or model-generated tool argument.
pub fn approve_materialization_target(
    path: impl AsRef<Path>,
) -> Result<MaterializationTarget, MaterializationError> {
    let path = path.as_ref();
    validate_target_path(path)?;
    validate_target_filesystem(path)?;
    validate_target_parent_policy(
        path.parent()
            .expect("validated materialization targets always have a parent"),
        MaterializationAuthorization::ExplicitlyApproved,
    )?;
    Ok(MaterializationTarget {
        path: path.to_path_buf(),
        authorization: MaterializationAuthorization::ExplicitlyApproved,
    })
}

pub fn materialize_profile(
    pack: &LoadedResourcePack<'_>,
    profile: &str,
    target: &MaterializationTarget,
) -> Result<MaterializationReceiptV1, MaterializationError> {
    let profile_id = pack.manifest().resolve_profile(profile).map_err(|error| {
        MaterializationError::Profile(ResourcePackLoaderError::InvalidProfile(error))
    })?;
    let resources = pack
        .resources_for_profile(profile)
        .map_err(MaterializationError::Profile)?;
    let receipt = build_receipt(pack, profile_id, target.authorization, &resources);
    let receipt_bytes =
        serde_json_canonicalizer::to_vec(&receipt).map_err(MaterializationError::CanonicalJson)?;

    validate_target_path(&target.path)?;
    validate_target_filesystem(&target.path)?;
    validate_target_parent_policy(
        target
            .path
            .parent()
            .expect("validated materialization targets always have a parent"),
        target.authorization,
    )?;
    let _lock = TargetLock::acquire(target)?;
    let original = inspect_existing_target(&target.path)?;
    let parent = target
        .path
        .parent()
        .expect("validated materialization targets always have a parent");
    let leaf = target_leaf(&target.path)?;
    let staging = create_unique_sibling_directory(parent, leaf, "stage")?;
    let staging_cleanup = DirectoryCleanup::new(staging.clone());

    write_staging_tree(&staging, &resources, &receipt_bytes)?;
    let staged_receipt = verify_managed_tree(&staging)?;
    if staged_receipt != receipt {
        return Err(MaterializationError::drift(
            &staging,
            "staged receipt does not match the selected profile",
        ));
    }

    validate_target_filesystem(&target.path)?;
    validate_target_parent_policy(parent, target.authorization)?;
    let current = inspect_existing_target(&target.path)?;
    if current != original {
        return Err(MaterializationError::TargetChanged {
            path: target.path.clone(),
        });
    }

    let backup = original
        .is_present()
        .then(|| unique_sibling_path(parent, leaf, "backup"))
        .transpose()?;
    promote_staging_with(&staging, &target.path, backup.as_deref(), |from, to| {
        fs::rename(from, to)
    })?;
    finish_committed_transaction(&target.path, backup.as_deref(), parent)?;
    staging_cleanup.disarm();

    Ok(receipt)
}

/// Verifies an existing managed materialization without modifying it.
///
/// The target must have been approved by a trusted CLI, UI, or installer
/// boundary. The complete tree, canonical receipt, modes, paths, file kinds,
/// sizes, and digests are checked before the receipt is returned.
pub fn verify_materialization(
    target: &MaterializationTarget,
) -> Result<MaterializationReceiptV1, MaterializationError> {
    validate_target_path(&target.path)?;
    validate_target_filesystem(&target.path)?;
    validate_target_parent_policy(
        target
            .path
            .parent()
            .expect("validated materialization targets always have a parent"),
        target.authorization,
    )?;
    match inspect_existing_target(&target.path)? {
        ExistingTarget::Managed(receipt) => Ok(receipt),
        ExistingTarget::Absent => Err(MaterializationError::MissingManagedTarget {
            path: target.path.clone(),
        }),
        ExistingTarget::Empty => Err(MaterializationError::UnmanagedTarget {
            path: target.path.clone(),
        }),
    }
}

/// Removes a verified Qiongli-managed materialization and no other tree.
///
/// The target is pinned and verified before it is moved to a private sibling
/// quarantine. The quarantine is verified again before recursive removal. If
/// the post-rename state becomes ambiguous, the quarantine is retained and a
/// cleanup-required error is returned rather than deleting unverified bytes.
pub fn remove_materialization(
    target: &MaterializationTarget,
) -> Result<MaterializationReceiptV1, MaterializationError> {
    validate_target_path(&target.path)?;
    validate_target_filesystem(&target.path)?;
    let parent = target
        .path
        .parent()
        .expect("validated materialization targets always have a parent");
    validate_target_parent_policy(parent, target.authorization)?;
    let _lock = TargetLock::acquire(target)?;

    let before = Handle::from_path(&target.path)
        .map_err(|error| MaterializationError::io("pin managed target", &target.path, &error))?;
    let receipt = verify_materialization(target)?;
    let after = Handle::from_path(&target.path)
        .map_err(|error| MaterializationError::io("repin managed target", &target.path, &error))?;
    if before != after {
        return Err(MaterializationError::TargetChanged {
            path: target.path.clone(),
        });
    }

    let quarantine = unique_sibling_path(parent, target_leaf(&target.path)?, "remove")?;
    fs::rename(&target.path, &quarantine).map_err(|error| {
        MaterializationError::io("move target to quarantine", &target.path, &error)
    })?;
    if let Err(error) = sync_directory(parent) {
        return Err(removal_cleanup_error(&target.path, &quarantine, &error));
    }

    let quarantined = MaterializationTarget {
        path: quarantine.clone(),
        authorization: target.authorization,
    };
    let quarantine_before = Handle::from_path(&quarantine)
        .map_err(|error| MaterializationError::io("pin removal quarantine", &quarantine, &error))?;
    let observed = verify_materialization(&quarantined)
        .map_err(|error| removal_cleanup_error(&target.path, &quarantine, &error))?;
    let quarantine_after = Handle::from_path(&quarantine).map_err(|error| {
        MaterializationError::io("repin removal quarantine", &quarantine, &error)
    })?;
    if observed != receipt || quarantine_before != quarantine_after {
        return Err(MaterializationError::CommittedWithCleanupFailure {
            path: target.path.clone(),
            backup_path: Some(quarantine),
            detail: "removal quarantine identity or receipt changed".to_owned(),
        });
    }

    fs::remove_dir_all(&quarantine)
        .map_err(|error| removal_cleanup_error(&target.path, &quarantine, &error))?;
    if let Err(error) = sync_directory(parent) {
        return Err(MaterializationError::CommittedWithCleanupFailure {
            path: target.path.clone(),
            backup_path: None,
            detail: error.to_string(),
        });
    }
    Ok(receipt)
}

fn removal_cleanup_error(
    target: &Path,
    quarantine: &Path,
    error: &impl Display,
) -> MaterializationError {
    MaterializationError::CommittedWithCleanupFailure {
        path: target.to_path_buf(),
        backup_path: Some(quarantine.to_path_buf()),
        detail: error.to_string(),
    }
}

fn build_receipt(
    pack: &LoadedResourcePack<'_>,
    profile: ProfileId,
    authorization: MaterializationAuthorization,
    resources: &[LoadedResource<'_, '_>],
) -> MaterializationReceiptV1 {
    MaterializationReceiptV1 {
        receipt_version: MATERIALIZATION_RECEIPT_VERSION,
        pack_id: pack.manifest().pack_id.clone(),
        content_version: pack.manifest().content_version.clone(),
        source_commit: pack.manifest().source_commit.clone(),
        profile,
        authorization,
        pack_sha256: pack.pack_sha256().to_string(),
        content_root_sha256: pack.manifest().content_root_sha256.clone(),
        entries: resources
            .iter()
            .map(|resource| MaterializedEntry {
                path: resource.entry().path.clone(),
                mode: resource.entry().mode,
                size_bytes: resource.entry().size_bytes,
                sha256: resource.entry().sha256.clone(),
            })
            .collect(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ExistingTarget {
    Absent,
    Empty,
    Managed(MaterializationReceiptV1),
}

impl ExistingTarget {
    fn is_present(&self) -> bool {
        !matches!(self, Self::Absent)
    }
}

fn inspect_existing_target(path: &Path) -> Result<ExistingTarget, MaterializationError> {
    let Some(metadata) = path_state(path)? else {
        return Ok(ExistingTarget::Absent);
    };
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(MaterializationError::LinkNotAllowed {
            path: path.to_path_buf(),
        });
    }
    if !metadata.is_dir() {
        return Err(MaterializationError::TargetNotDirectory {
            path: path.to_path_buf(),
        });
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| MaterializationError::io("list target", path, &error))?;
    if entries.next().is_none() {
        return Ok(ExistingTarget::Empty);
    }
    if path_state(&path.join(MATERIALIZATION_RECEIPT_FILE))?.is_none() {
        return Err(MaterializationError::UnmanagedTarget {
            path: path.to_path_buf(),
        });
    }

    verify_managed_tree(path).map(ExistingTarget::Managed)
}

fn validate_target_path(path: &Path) -> Result<(), MaterializationError> {
    if !path.is_absolute() {
        return Err(MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target must be absolute",
        });
    }
    if has_lexical_traversal_component(path) {
        return Err(MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target must not contain traversal components",
        });
    }
    let parent = path
        .parent()
        .filter(|parent| parent != &path)
        .ok_or_else(|| MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "filesystem roots cannot be materialization targets",
        })?;
    if parent.as_os_str().is_empty() {
        return Err(MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target must have an absolute parent",
        });
    }
    let leaf = target_leaf(path)?;
    validate_target_leaf(leaf, path)
}

#[cfg(unix)]
fn has_lexical_traversal_component(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str()
        .as_bytes()
        .split(|byte| *byte == b'/')
        .any(|component| component == b"." || component == b"..")
}

#[cfg(windows)]
fn has_lexical_traversal_component(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let path = path.as_os_str().encode_wide().collect::<Vec<_>>();
    path.split(|unit| matches!(*unit, 47 | 92))
        .any(|component| component == [46] || component == [46, 46])
}

#[cfg(not(any(unix, windows)))]
fn has_lexical_traversal_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn target_leaf(path: &Path) -> Result<&str, MaterializationError> {
    path.file_name()
        .and_then(|leaf| leaf.to_str())
        .ok_or_else(|| MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target leaf must be UTF-8",
        })
}

fn validate_target_leaf(leaf: &str, path: &Path) -> Result<(), MaterializationError> {
    if leaf.is_empty() || leaf == "." || leaf == ".." {
        return Err(MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target leaf is empty or reserved",
        });
    }
    if leaf.len() > MAX_TARGET_LEAF_BYTES {
        return Err(MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target leaf exceeds the portable length limit",
        });
    }
    if leaf.nfc().collect::<String>() != leaf
        || leaf.ends_with('.')
        || leaf.ends_with(' ')
        || leaf.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
                )
        })
        || is_windows_device_name(leaf)
    {
        return Err(MaterializationError::InvalidTarget {
            path: path.to_path_buf(),
            reason: "target leaf is not portable",
        });
    }
    Ok(())
}

fn validate_target_filesystem(path: &Path) -> Result<(), MaterializationError> {
    let parent = path
        .parent()
        .expect("validated materialization targets always have a parent");
    validate_existing_directory_chain(parent)?;
    if let Some(metadata) = path_state(path)? {
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return Err(MaterializationError::LinkNotAllowed {
                path: path.to_path_buf(),
            });
        }
        if !metadata.is_dir() {
            return Err(MaterializationError::TargetNotDirectory {
                path: path.to_path_buf(),
            });
        }
    }
    Ok(())
}

#[cfg(unix)]
fn validate_target_parent_policy(
    path: &Path,
    authorization: MaterializationAuthorization,
) -> Result<(), MaterializationError> {
    use std::os::unix::fs::PermissionsExt;

    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(_) => {
                current.push(component.as_os_str());
                let mode = fs::symlink_metadata(&current)
                    .map_err(|error| {
                        MaterializationError::io("inspect target ancestor policy", &current, &error)
                    })?
                    .permissions()
                    .mode();
                let writable_by_others = mode & 0o022 != 0;
                let temporary_sticky_boundary =
                    authorization == MaterializationAuthorization::Temporary && mode & 0o1000 != 0;
                if writable_by_others && !temporary_sticky_boundary {
                    return Err(MaterializationError::InsecureTargetParent { path: current });
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(MaterializationError::InvalidTarget {
                    path: path.to_path_buf(),
                    reason: "target ancestor policy received traversal components",
                });
            }
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_target_parent_policy(
    _path: &Path,
    _authorization: MaterializationAuthorization,
) -> Result<(), MaterializationError> {
    Ok(())
}

fn validate_existing_directory_chain(path: &Path) -> Result<(), MaterializationError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(_) => {
                current.push(component.as_os_str());
                let metadata = fs::symlink_metadata(&current).map_err(|error| {
                    if error.kind() == io::ErrorKind::NotFound {
                        MaterializationError::MissingTargetParent {
                            path: current.clone(),
                        }
                    } else {
                        MaterializationError::io("inspect target parent", &current, &error)
                    }
                })?;
                if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                    return Err(MaterializationError::LinkNotAllowed { path: current });
                }
                if !metadata.is_dir() {
                    return Err(MaterializationError::TargetParentNotDirectory { path: current });
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(MaterializationError::InvalidTarget {
                    path: path.to_path_buf(),
                    reason: "target parent must not contain traversal components",
                });
            }
        }
    }
    Ok(())
}

fn write_staging_tree(
    staging: &Path,
    resources: &[LoadedResource<'_, '_>],
    receipt_bytes: &[u8],
) -> Result<(), MaterializationError> {
    let mut directories = vec![staging.to_path_buf()];
    for resource in resources {
        let destination = staging.join(Path::new(&resource.entry().path));
        let parent = destination
            .parent()
            .expect("validated resource-pack entries always have a parent");
        ensure_staging_directories(staging, parent, &mut directories)?;
        write_new_file(
            &destination,
            resource.bytes(),
            logical_mode_bits(resource.entry().mode),
        )?;
    }

    write_new_file(
        &staging.join(MATERIALIZATION_RECEIPT_FILE),
        receipt_bytes,
        0o644,
    )?;
    for directory in directories.iter().rev() {
        set_directory_mode(directory, 0o755)?;
        sync_directory(directory)?;
    }
    Ok(())
}

fn ensure_staging_directories(
    staging: &Path,
    destination_parent: &Path,
    directories: &mut Vec<PathBuf>,
) -> Result<(), MaterializationError> {
    let relative = destination_parent.strip_prefix(staging).map_err(|_| {
        MaterializationError::InvalidTarget {
            path: destination_parent.to_path_buf(),
            reason: "resource path escaped the staging root",
        }
    })?;
    let mut current = staging.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(MaterializationError::InvalidTarget {
                path: destination_parent.to_path_buf(),
                reason: "resource path contains traversal components",
            });
        };
        current.push(component);
        match fs::create_dir(&current) {
            Ok(()) => {
                directories.push(current.clone());
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let metadata = fs::symlink_metadata(&current).map_err(|inspect| {
                    MaterializationError::io("inspect staging", &current, &inspect)
                })?;
                if metadata.file_type().is_symlink()
                    || is_reparse_point(&metadata)
                    || !metadata.is_dir()
                {
                    return Err(MaterializationError::LinkNotAllowed { path: current });
                }
            }
            Err(error) => {
                return Err(MaterializationError::io(
                    "create staging directory",
                    &current,
                    &error,
                ));
            }
        }
    }
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8], mode: u32) -> Result<(), MaterializationError> {
    let mut file = open_new_private_file(path)
        .map_err(|error| MaterializationError::io("create staged file", path, &error))?;
    file.write_all(bytes)
        .map_err(|error| MaterializationError::io("write staged file", path, &error))?;
    file.sync_all()
        .map_err(|error| MaterializationError::io("sync staged file", path, &error))?;
    drop(file);
    set_file_mode(path, mode)?;
    sync_file_mode(path)?;
    Ok(())
}

#[cfg(unix)]
fn open_new_private_file(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
}

#[cfg(unix)]
fn sync_file_mode(path: &Path) -> Result<(), MaterializationError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| MaterializationError::io("sync staged file mode", path, &error))
}

#[cfg(not(unix))]
fn sync_file_mode(_path: &Path) -> Result<(), MaterializationError> {
    Ok(())
}

#[cfg(not(unix))]
fn open_new_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().create_new(true).write(true).open(path)
}

fn verify_managed_or_empty_tree(path: &Path) -> Result<(), MaterializationError> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| MaterializationError::io("list backup", path, &error))?;
    if entries.next().is_none() {
        Ok(())
    } else {
        verify_managed_tree(path).map(|_| ())
    }
}

fn finish_committed_transaction(
    target: &Path,
    backup: Option<&Path>,
    parent: &Path,
) -> Result<(), MaterializationError> {
    finish_committed_transaction_with(
        target,
        backup,
        parent,
        verify_managed_or_empty_tree,
        |path| {
            fs::remove_dir_all(path)
                .map_err(|error| MaterializationError::io("remove backup", path, &error))
        },
        sync_directory,
    )
}

fn finish_committed_transaction_with<V, R, S>(
    target: &Path,
    backup: Option<&Path>,
    parent: &Path,
    mut verify_backup: V,
    mut remove_backup: R,
    mut sync_parent: S,
) -> Result<(), MaterializationError>
where
    V: FnMut(&Path) -> Result<(), MaterializationError>,
    R: FnMut(&Path) -> Result<(), MaterializationError>,
    S: FnMut(&Path) -> Result<(), MaterializationError>,
{
    if let Some(backup) = backup {
        if let Err(error) = verify_backup(backup) {
            return Err(MaterializationError::CommittedWithCleanupFailure {
                path: target.to_path_buf(),
                backup_path: Some(backup.to_path_buf()),
                detail: error.to_string(),
            });
        }
        if let Err(error) = remove_backup(backup) {
            return Err(MaterializationError::CommittedWithCleanupFailure {
                path: target.to_path_buf(),
                backup_path: Some(backup.to_path_buf()),
                detail: error.to_string(),
            });
        }
    }
    if let Err(error) = sync_parent(parent) {
        return Err(MaterializationError::CommittedWithCleanupFailure {
            path: target.to_path_buf(),
            backup_path: None,
            detail: error.to_string(),
        });
    }
    Ok(())
}

fn verify_managed_tree(path: &Path) -> Result<MaterializationReceiptV1, MaterializationError> {
    let receipt_path = path.join(MATERIALIZATION_RECEIPT_FILE);
    let receipt_metadata = checked_regular_metadata(&receipt_path, path)?;
    if receipt_metadata.len() > MAX_RECEIPT_BYTES {
        return Err(MaterializationError::invalid_receipt(
            &receipt_path,
            "receipt exceeds the bounded size limit",
        ));
    }
    verify_file_mode(&receipt_path, path, LogicalMode::Regular)?;
    let receipt_bytes = fs::read(&receipt_path)
        .map_err(|error| MaterializationError::io("read managed receipt", &receipt_path, &error))?;
    let receipt: MaterializationReceiptV1 = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| MaterializationError::invalid_receipt(&receipt_path, error.to_string()))?;
    let canonical =
        serde_json_canonicalizer::to_vec(&receipt).map_err(MaterializationError::CanonicalJson)?;
    if canonical != receipt_bytes {
        return Err(MaterializationError::invalid_receipt(
            &receipt_path,
            "receipt is not canonical RFC 8785 JSON",
        ));
    }
    validate_receipt(&receipt, &receipt_path)?;

    let expected_files = receipt
        .entries
        .iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    if expected_files.len() != receipt.entries.len() {
        return Err(MaterializationError::invalid_receipt(
            &receipt_path,
            "receipt contains duplicate paths",
        ));
    }
    let expected_directories = expected_directory_paths(&receipt.entries);
    let mut seen_files = BTreeSet::new();
    let mut seen_directories = BTreeSet::new();
    verify_directory(
        path,
        path,
        &expected_files,
        &expected_directories,
        &mut seen_files,
        &mut seen_directories,
    )?;
    if seen_files.len() != expected_files.len()
        || seen_directories.len() != expected_directories.len()
    {
        return Err(MaterializationError::drift(
            path,
            "managed entries or directories are missing",
        ));
    }
    Ok(receipt)
}

fn validate_receipt(
    receipt: &MaterializationReceiptV1,
    receipt_path: &Path,
) -> Result<(), MaterializationError> {
    if receipt.receipt_version != MATERIALIZATION_RECEIPT_VERSION {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "unsupported receipt version",
        ));
    }
    if receipt.pack_id.is_empty()
        || !receipt
            .pack_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "pack_id is invalid",
        ));
    }
    if Version::parse(&receipt.content_version).is_err() {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "content_version is not SemVer",
        ));
    }
    if !is_lower_hex(&receipt.source_commit, 40)
        || !is_lower_hex(&receipt.pack_sha256, 64)
        || !is_lower_hex(&receipt.content_root_sha256, 64)
    {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "receipt digest fields are invalid",
        ));
    }
    if receipt.entries.len() > MAX_MANAGED_ENTRIES {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "receipt exceeds the managed entry limit",
        ));
    }
    let mut total_bytes = 0_u64;
    let mut previous: Option<&str> = None;
    for entry in &receipt.entries {
        validate_materialized_path(&entry.path, receipt_path)?;
        let resource_kind = expected_resource_kind(&entry.path).ok_or_else(|| {
            MaterializationError::invalid_receipt(
                receipt_path,
                "receipt entry is outside the canonical source allowlist",
            )
        })?;
        if entry.path == MATERIALIZATION_RECEIPT_FILE
            || previous.is_some_and(|candidate| candidate >= entry.path.as_str())
            || !is_lower_hex(&entry.sha256, 64)
            || (receipt.profile == ProfileId::SkillOnly
                && matches!(
                    resource_kind,
                    ResourceKind::TargetMetadata | ResourceKind::McpContract | ResourceKind::Schema
                ))
        {
            return Err(MaterializationError::invalid_receipt(
                receipt_path,
                "receipt entries are not unique canonical pack entries",
            ));
        }
        if entry.size_bytes > MAX_MANAGED_ENTRY_BYTES {
            return Err(MaterializationError::invalid_receipt(
                receipt_path,
                "receipt entry exceeds the managed size limit",
            ));
        }
        total_bytes = total_bytes.checked_add(entry.size_bytes).ok_or_else(|| {
            MaterializationError::invalid_receipt(receipt_path, "receipt size overflow")
        })?;
        if total_bytes > MAX_MANAGED_TOTAL_BYTES {
            return Err(MaterializationError::invalid_receipt(
                receipt_path,
                "receipt exceeds the managed total-size limit",
            ));
        }
        previous = Some(&entry.path);
    }
    Ok(())
}

fn validate_materialized_path(path: &str, receipt_path: &Path) -> Result<(), MaterializationError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "receipt entry path is not portable",
        ));
    }
    let components = path.split('/').collect::<Vec<_>>();
    if components.len() > MAX_MANAGED_PATH_DEPTH {
        return Err(MaterializationError::invalid_receipt(
            receipt_path,
            "receipt entry exceeds the managed path-depth limit",
        ));
    }
    for component in components {
        if component.nfc().collect::<String>() != component
            || component.ends_with('.')
            || component.ends_with(' ')
            || component.chars().any(|character| {
                character.is_control()
                    || matches!(character, ':' | '*' | '?' | '"' | '<' | '>' | '|')
            })
            || is_windows_device_name(component)
        {
            return Err(MaterializationError::invalid_receipt(
                receipt_path,
                "receipt entry path is not portable",
            ));
        }
    }
    Ok(())
}

fn expected_directory_paths(entries: &[MaterializedEntry]) -> BTreeSet<String> {
    let mut directories = BTreeSet::new();
    for entry in entries {
        let mut current = String::new();
        let mut components = entry.path.split('/').collect::<Vec<_>>();
        components.pop();
        for component in components {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(component);
            directories.insert(current.clone());
        }
    }
    directories
}

fn verify_directory(
    root: &Path,
    directory: &Path,
    expected_files: &BTreeMap<String, &MaterializedEntry>,
    expected_directories: &BTreeSet<String>,
    seen_files: &mut BTreeSet<String>,
    seen_directories: &mut BTreeSet<String>,
) -> Result<(), MaterializationError> {
    verify_directory_mode(directory, root)?;
    let entries = fs::read_dir(directory)
        .map_err(|error| MaterializationError::io("list managed target", directory, &error))?;
    for entry in entries {
        let entry = entry
            .map_err(|error| MaterializationError::io("list managed target", directory, &error))?;
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(|_| {
            MaterializationError::drift(root, "managed path escaped the target root")
        })?;
        let relative = relative
            .to_str()
            .ok_or_else(|| MaterializationError::drift(root, "managed path is not UTF-8"))?;
        let relative = relative.replace(std::path::MAIN_SEPARATOR, "/");
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| MaterializationError::io("inspect managed path", &path, &error))?;
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return Err(MaterializationError::LinkNotAllowed { path });
        }
        if metadata.is_dir() {
            if !expected_directories.contains(&relative) {
                return Err(MaterializationError::drift(
                    root,
                    format!("unexpected directory {relative:?}"),
                ));
            }
            seen_directories.insert(relative);
            verify_directory(
                root,
                &path,
                expected_files,
                expected_directories,
                seen_files,
                seen_directories,
            )?;
        } else if metadata.is_file() {
            if relative == MATERIALIZATION_RECEIPT_FILE {
                continue;
            }
            let expected = expected_files.get(&relative).ok_or_else(|| {
                MaterializationError::drift(root, format!("unexpected file {relative:?}"))
            })?;
            verify_materialized_file(&path, root, expected)?;
            seen_files.insert(relative);
        } else {
            return Err(MaterializationError::drift(
                root,
                format!("unsupported file type at {relative:?}"),
            ));
        }
    }
    Ok(())
}

fn verify_materialized_file(
    path: &Path,
    root: &Path,
    expected: &MaterializedEntry,
) -> Result<(), MaterializationError> {
    let metadata = checked_regular_metadata(path, root)?;
    if metadata.len() != expected.size_bytes {
        return Err(MaterializationError::drift(
            root,
            format!("size changed for {:?}", expected.path),
        ));
    }
    verify_file_mode(path, root, expected.mode)?;
    let actual_sha256 = hash_file(path)?;
    if actual_sha256 != expected.sha256 {
        return Err(MaterializationError::drift(
            root,
            format!("digest changed for {:?}", expected.path),
        ));
    }
    Ok(())
}

fn checked_regular_metadata(path: &Path, root: &Path) -> Result<Metadata, MaterializationError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| MaterializationError::io("inspect managed file", path, &error))?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(MaterializationError::LinkNotAllowed {
            path: path.to_path_buf(),
        });
    }
    if !metadata.is_file() {
        return Err(MaterializationError::drift(
            root,
            format!("managed path is not a regular file: {}", path.display()),
        ));
    }
    if has_multiple_hard_links(&metadata) {
        return Err(MaterializationError::drift(
            root,
            format!("managed path is hard linked: {}", path.display()),
        ));
    }
    Ok(metadata)
}

fn hash_file(path: &Path) -> Result<String, MaterializationError> {
    let mut file = File::open(path)
        .map_err(|error| MaterializationError::io("open managed file", path, &error))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| MaterializationError::io("read managed file", path, &error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn create_unique_sibling_directory(
    parent: &Path,
    leaf: &str,
    kind: &str,
) -> Result<PathBuf, MaterializationError> {
    for _ in 0..128 {
        let path = sibling_path(parent, leaf, kind);
        match create_directory_with_mode(&path, 0o700) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(MaterializationError::io(
                    "create transaction directory",
                    &path,
                    &error,
                ));
            }
        }
    }
    Err(MaterializationError::InvalidTarget {
        path: parent.to_path_buf(),
        reason: "could not allocate a unique transaction directory",
    })
}

#[cfg(unix)]
fn create_directory_with_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(mode);
    builder.create(path)
}

#[cfg(not(unix))]
fn create_directory_with_mode(path: &Path, _mode: u32) -> io::Result<()> {
    fs::create_dir(path)
}

fn unique_sibling_path(
    parent: &Path,
    leaf: &str,
    kind: &str,
) -> Result<PathBuf, MaterializationError> {
    for _ in 0..128 {
        let path = sibling_path(parent, leaf, kind);
        if path_state(&path)?.is_none() {
            return Ok(path);
        }
    }
    Err(MaterializationError::InvalidTarget {
        path: parent.to_path_buf(),
        reason: "could not allocate a unique transaction path",
    })
}

fn sibling_path(parent: &Path, leaf: &str, kind: &str) -> PathBuf {
    parent.join(format!(
        ".{leaf}.qiongli-{kind}-{}-{}",
        std::process::id(),
        transaction_id()
    ))
}

fn promote_staging_with<F>(
    staging: &Path,
    target: &Path,
    backup: Option<&Path>,
    mut rename: F,
) -> Result<(), MaterializationError>
where
    F: FnMut(&Path, &Path) -> io::Result<()>,
{
    if let Some(backup) = backup {
        rename(target, backup).map_err(|error| {
            MaterializationError::io("move prior target to backup", target, &error)
        })?;
    }

    if let Err(commit_error) = rename(staging, target) {
        if let Some(backup) = backup
            && let Err(rollback_error) = rename(backup, target)
        {
            return Err(MaterializationError::RollbackFailed {
                path: target.to_path_buf(),
                backup_path: backup.to_path_buf(),
                commit_kind: commit_error.kind(),
                rollback_kind: rollback_error.kind(),
            });
        }
        return Err(MaterializationError::CommitFailed {
            path: target.to_path_buf(),
            kind: commit_error.kind(),
        });
    }
    Ok(())
}

struct TargetLock {
    path: PathBuf,
    identity: Option<Handle>,
}

impl TargetLock {
    fn acquire(target: &MaterializationTarget) -> Result<Self, MaterializationError> {
        let parent = target
            .path
            .parent()
            .expect("validated materialization targets always have a parent");
        let leaf = target_leaf(&target.path)?;
        let path = parent.join(format!(".{leaf}.qiongli-materialize.lock"));
        let mut file = match OpenOptions::new().create_new(true).write(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Err(MaterializationError::TargetBusy {
                    path: target.path.clone(),
                });
            }
            Err(error) => {
                return Err(MaterializationError::io(
                    "acquire transaction lock",
                    &path,
                    &error,
                ));
            }
        };
        let setup = (|| {
            set_file_mode(&path, 0o600)?;
            writeln!(file, "{}", std::process::id()).map_err(|error| {
                MaterializationError::io("write transaction lock", &path, &error)
            })?;
            file.sync_all().map_err(|error| {
                MaterializationError::io("sync transaction lock", &path, &error)
            })?;
            Ok(())
        })();
        drop(file);
        if let Err(error) = setup {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        let identity = match Handle::from_path(&path) {
            Ok(identity) => identity,
            Err(error) => {
                let _ = fs::remove_file(&path);
                return Err(MaterializationError::io(
                    "pin transaction lock identity",
                    &path,
                    &error,
                ));
            }
        };
        Ok(Self {
            path,
            identity: Some(identity),
        })
    }
}

impl Drop for TargetLock {
    fn drop(&mut self) {
        let still_owned = self.identity.as_ref().is_some_and(|expected| {
            Handle::from_path(&self.path).is_ok_and(|current| &current == expected)
        });
        self.identity.take();
        if still_owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct DirectoryCleanup {
    path: PathBuf,
    armed: bool,
}

impl DirectoryCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for DirectoryCleanup {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn path_state(path: &Path) -> Result<Option<Metadata>, MaterializationError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(MaterializationError::io("inspect", path, &error)),
    }
}

fn transaction_id() -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
    format!("{nonce}-{sequence}")
}

fn logical_mode_bits(mode: LogicalMode) -> u32 {
    match mode {
        LogicalMode::Regular => 0o644,
        LogicalMode::Executable => 0o755,
    }
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_windows_device_name(component: &str) -> bool {
    let base = component.split('.').next().unwrap_or(component);
    let upper = base.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && matches!(&upper[..3], "COM" | "LPT")
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}

#[cfg(unix)]
fn set_file_mode(path: &Path, mode: u32) -> Result<(), MaterializationError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| MaterializationError::io("set file permissions", path, &error))
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path, _mode: u32) -> Result<(), MaterializationError> {
    Ok(())
}

#[cfg(unix)]
fn set_directory_mode(path: &Path, mode: u32) -> Result<(), MaterializationError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| MaterializationError::io("set directory permissions", path, &error))
}

#[cfg(not(unix))]
fn set_directory_mode(_path: &Path, _mode: u32) -> Result<(), MaterializationError> {
    Ok(())
}

#[cfg(unix)]
fn verify_file_mode(
    path: &Path,
    root: &Path,
    mode: LogicalMode,
) -> Result<(), MaterializationError> {
    use std::os::unix::fs::PermissionsExt;

    let actual = fs::metadata(path)
        .map_err(|error| MaterializationError::io("inspect file permissions", path, &error))?
        .permissions()
        .mode()
        & 0o777;
    let expected = logical_mode_bits(mode);
    if actual != expected {
        return Err(MaterializationError::drift(
            root,
            format!("permissions changed for {}", path.display()),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn verify_file_mode(
    _path: &Path,
    _root: &Path,
    _mode: LogicalMode,
) -> Result<(), MaterializationError> {
    Ok(())
}

#[cfg(unix)]
fn verify_directory_mode(path: &Path, root: &Path) -> Result<(), MaterializationError> {
    use std::os::unix::fs::PermissionsExt;

    let actual = fs::metadata(path)
        .map_err(|error| MaterializationError::io("inspect directory permissions", path, &error))?
        .permissions()
        .mode()
        & 0o777;
    if actual != 0o755 {
        return Err(MaterializationError::drift(
            root,
            format!("directory permissions changed for {}", path.display()),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn verify_directory_mode(_path: &Path, _root: &Path) -> Result<(), MaterializationError> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), MaterializationError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| MaterializationError::io("sync directory", path, &error))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), MaterializationError> {
    Ok(())
}

#[cfg(unix)]
fn has_multiple_hard_links(metadata: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    metadata.nlink() > 1
}

#[cfg(not(unix))]
fn has_multiple_hard_links(_metadata: &Metadata) -> bool {
    false
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root(name: &str) -> PathBuf {
        let test_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qiongli-content-unit-tests");
        fs::create_dir_all(&test_base).expect("unit-test base must be created");
        let requested = test_base.join(format!(
            "qiongli-materializer-unit-{name}-{}-{}",
            std::process::id(),
            transaction_id()
        ));
        fs::create_dir(&requested).expect("unit-test root must be created");
        fs::canonicalize(requested).expect("unit-test root must canonicalize")
    }

    #[test]
    fn target_lock_is_exclusive_and_removed_on_drop() {
        let root = test_root("lock");
        let target = approve_materialization_target(root.join("install"))
            .expect("unit-test target must approve");
        let first = TargetLock::acquire(&target).expect("first lock must succeed");
        assert!(matches!(
            TargetLock::acquire(&target),
            Err(MaterializationError::TargetBusy { .. })
        ));
        drop(first);
        TargetLock::acquire(&target).expect("released lock must be reusable");
        fs::remove_dir_all(root).expect("unit-test root must clean");
    }

    #[test]
    fn replaced_lock_path_is_not_removed_by_the_prior_owner() {
        let root = test_root("lock-identity");
        let target = approve_materialization_target(root.join("install"))
            .expect("unit-test target must approve");
        let lock = TargetLock::acquire(&target).expect("lock must succeed");
        let lock_path = lock.path.clone();
        if let Err(error) = fs::remove_file(&lock_path) {
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            ) {
                drop(lock);
                fs::remove_dir_all(root).expect("unit-test root must clean");
                return;
            }
            panic!("lock replacement fixture could not remove the first path: {error}");
        }
        fs::write(&lock_path, b"replacement").expect("replacement lock must be written");

        drop(lock);

        assert_eq!(fs::read(&lock_path).unwrap(), b"replacement");
        fs::remove_file(lock_path).expect("replacement lock must clean");
        fs::remove_dir_all(root).expect("unit-test root must clean");
    }

    #[test]
    fn failed_promotion_restores_the_prior_target() {
        let root = test_root("rollback");
        let target = root.join("install");
        let staging = root.join("stage");
        let backup = root.join("backup");
        fs::create_dir(&target).expect("old target must be created");
        fs::write(target.join("value"), b"old").expect("old value must be written");
        fs::create_dir(&staging).expect("staging must be created");
        fs::write(staging.join("value"), b"new").expect("new value must be written");
        let mut calls = 0_u8;

        let result = promote_staging_with(&staging, &target, Some(&backup), |from, to| {
            calls += 1;
            if calls == 2 {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected promotion failure",
                ))
            } else {
                fs::rename(from, to)
            }
        });

        assert!(matches!(
            result,
            Err(MaterializationError::CommitFailed {
                kind: io::ErrorKind::PermissionDenied,
                ..
            })
        ));
        assert_eq!(fs::read(target.join("value")).unwrap(), b"old");
        assert_eq!(fs::read(staging.join("value")).unwrap(), b"new");
        assert!(!backup.exists());
        fs::remove_dir_all(root).expect("unit-test root must clean");
    }

    #[test]
    fn post_commit_cleanup_failure_is_reported_as_committed_state() {
        let target = Path::new("/target");
        let backup = Path::new("/backup");
        let parent = Path::new("/");
        let result = finish_committed_transaction_with(
            target,
            Some(backup),
            parent,
            |_| Ok(()),
            |path| {
                Err(MaterializationError::Io {
                    operation: "injected backup cleanup",
                    path: path.to_path_buf(),
                    kind: io::ErrorKind::PermissionDenied,
                })
            },
            |_| Ok(()),
        );

        assert!(matches!(
            result,
            Err(MaterializationError::CommittedWithCleanupFailure {
                path,
                backup_path: Some(backup_path),
                ..
            }) if path == target && backup_path == backup
        ));
    }
}
