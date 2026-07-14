#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::fs::TryLockError;
use std::fs::{self, File, Metadata};
#[cfg(unix)]
use std::io::Write;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(unix)]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::document::decode_global_settings;
#[cfg(unix)]
use crate::document::encode_global_settings;
use crate::{
    ConfigError, ConfigRoot, ConfigState, GlobalSettings, LoadedGlobalSettings,
    MAX_GLOBAL_SETTINGS_BYTES, PersistenceStage, RedactedConfigStatus,
};

pub const GLOBAL_SETTINGS_FILE: &str = "settings.json";
#[cfg(unix)]
const LOCK_FILE: &str = ".settings.lock";
const STAGING_FILE_PREFIX: &str = ".settings.json.qiongli-stage-";
const RECOVERY_FILE_PREFIX: &str = ".settings.json.qiongli-recovery-";
#[cfg(unix)]
const ABSENT_RECOVERY_MARKER: &[u8] = b"qiongli-settings-recovery-v1:absent\n";
#[cfg(unix)]
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(unix)]
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(unix)]
static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct GlobalSettingsStore {
    root: ConfigRoot,
    #[cfg(unix)]
    lock_timeout: Duration,
}

impl GlobalSettingsStore {
    #[must_use]
    pub const fn new(root: ConfigRoot) -> Self {
        Self {
            root,
            #[cfg(unix)]
            lock_timeout: DEFAULT_LOCK_TIMEOUT,
        }
    }

    pub fn load(&self) -> Result<LoadedGlobalSettings, ConfigError> {
        self.load_state().map(|state| state.loaded)
    }

    #[must_use]
    pub fn status(&self) -> RedactedConfigStatus {
        match self.load_state() {
            Ok(state) => {
                let cleanup_required = state.present && self.has_cleanup_artifact();
                RedactedConfigStatus::loaded(
                    &self.root,
                    readable_state(state.present),
                    &state.loaded,
                    cleanup_required,
                )
            }
            Err(error) => RedactedConfigStatus::failed(&self.root, &error),
        }
    }

    pub fn replace(
        &self,
        expected_revision: u64,
        replacement: GlobalSettings,
    ) -> Result<CommitOutcome, ConfigError> {
        #[cfg(unix)]
        {
            self.replace_unix_with(expected_revision, replacement, &NoFaults)
        }
        #[cfg(not(unix))]
        {
            let _ = (expected_revision, replacement);
            Err(ConfigError::UnsupportedPlatformSecurity)
        }
    }

    #[cfg(unix)]
    fn replace_unix_with<F: PersistenceFaults>(
        &self,
        expected_revision: u64,
        replacement: GlobalSettings,
        faults: &F,
    ) -> Result<CommitOutcome, ConfigError> {
        self.prepare_unix_store()?;
        let _lock = self.acquire_lock()?;
        faults.check(FaultPoint::AfterLock)?;

        let current = self.load_state()?;
        faults.check(FaultPoint::AfterCurrentRead)?;
        if current.loaded.revision != expected_revision {
            return Err(ConfigError::RevisionConflict {
                observed: current.loaded.revision,
            });
        }
        let revision = current
            .loaded
            .revision
            .checked_add(1)
            .filter(|revision| *revision <= crate::MAX_GLOBAL_SETTINGS_REVISION)
            .ok_or(ConfigError::RevisionExhausted)?;
        let next_bytes = encode_global_settings(&replacement, revision)?;

        let staging = write_unique_transaction_file(
            self.root.state_root(),
            STAGING_FILE_PREFIX,
            &next_bytes,
            PersistenceStage::WriteStaging,
            PersistenceStage::SyncStaging,
        )?;
        if let Err(error) = faults.check(FaultPoint::AfterStagingSync) {
            remove_transaction_file(&staging);
            return Err(error);
        }

        let recovery_bytes = current.bytes.as_deref().unwrap_or(ABSENT_RECOVERY_MARKER);
        let recovery = match write_unique_transaction_file(
            self.root.state_root(),
            RECOVERY_FILE_PREFIX,
            recovery_bytes,
            PersistenceStage::CreateRecovery,
            PersistenceStage::CreateRecovery,
        ) {
            Ok(path) => path,
            Err(error) => {
                remove_transaction_file(&staging);
                return Err(error);
            }
        };
        if let Err(error) = faults.check(FaultPoint::AfterRecoverySync) {
            remove_transaction_file(&staging);
            remove_transaction_file(&recovery);
            return Err(error);
        }
        if let Err(error) = sync_directory(self.root.state_root()) {
            remove_transaction_file(&staging);
            remove_transaction_file(&recovery);
            return Err(error);
        }

        let settings_path = self.root.state_root().join(GLOBAL_SETTINGS_FILE);
        if let Err(error) = fs::rename(&staging, &settings_path) {
            remove_transaction_file(&staging);
            remove_transaction_file(&recovery);
            return Err(ConfigError::PersistenceFailed {
                stage: PersistenceStage::Activate,
                kind: error.kind(),
            });
        }

        if let Err(error) = faults.check(FaultPoint::AfterActivation) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }
        if let Err(error) = faults.check(FaultPoint::BeforeCommitDirectorySync) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }
        if let Err(error) = sync_directory(self.root.state_root()) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }
        if let Err(error) = verify_committed_document(&settings_path, &next_bytes, revision) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }

        if faults.check(FaultPoint::DuringCleanup).is_err()
            || fs::remove_file(&recovery).is_err()
            || sync_directory(self.root.state_root()).is_err()
        {
            return Ok(CommitOutcome {
                revision,
                cleanup_required: true,
            });
        }
        Ok(CommitOutcome {
            revision,
            cleanup_required: false,
        })
    }

    #[cfg(unix)]
    fn prepare_unix_store(&self) -> Result<(), ConfigError> {
        if !validate_existing_directory_chain(self.root.compatibility_root())? {
            fs::create_dir_all(self.root.compatibility_root()).map_err(|error| {
                ConfigError::PersistenceFailed {
                    stage: PersistenceStage::CreateStore,
                    kind: error.kind(),
                }
            })?;
            if !validate_existing_directory_chain(self.root.compatibility_root())? {
                return Err(ConfigError::UnsafeManagedPath);
            }
        }
        match metadata_if_exists(self.root.state_root())? {
            Some(metadata) => validate_managed_directory(&metadata)?,
            None => {
                create_private_directory(self.root.state_root())?;
                let metadata = metadata_if_exists(self.root.state_root())?
                    .ok_or(ConfigError::UnsafeManagedPath)?;
                validate_managed_directory(&metadata)?;
                sync_directory(self.root.compatibility_root())?;
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    fn acquire_lock(&self) -> Result<File, ConfigError> {
        let lock_path = self.root.state_root().join(LOCK_FILE);
        let expected = metadata_if_exists(&lock_path)?;
        if let Some(metadata) = expected.as_ref() {
            validate_managed_file(metadata)?;
        }
        let file = open_lock_file(&lock_path)?;
        let opened = file
            .metadata()
            .map_err(|error| ConfigError::PersistenceFailed {
                stage: PersistenceStage::AcquireLock,
                kind: error.kind(),
            })?;
        validate_managed_file(&opened)?;
        let linked = metadata_if_exists(&lock_path)?.ok_or(ConfigError::UnsafeManagedPath)?;
        validate_managed_file(&linked)?;
        if !same_file_identity(&linked, &opened)
            || expected
                .as_ref()
                .is_some_and(|metadata| !same_file_identity(metadata, &opened))
        {
            return Err(ConfigError::UnsafeManagedPath);
        }

        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => break,
                Err(TryLockError::WouldBlock) => {
                    if started.elapsed() >= self.lock_timeout {
                        return Err(ConfigError::LockBusy);
                    }
                    let remaining = self.lock_timeout.saturating_sub(started.elapsed());
                    std::thread::sleep(LOCK_RETRY_INTERVAL.min(remaining));
                }
                Err(TryLockError::Error(error)) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(TryLockError::Error(error)) => {
                    return Err(ConfigError::PersistenceFailed {
                        stage: PersistenceStage::AcquireLock,
                        kind: error.kind(),
                    });
                }
            }
        }
        let linked_after = metadata_if_exists(&lock_path)?.ok_or(ConfigError::UnsafeManagedPath)?;
        if !same_file_identity(&linked_after, &opened) {
            return Err(ConfigError::UnsafeManagedPath);
        }
        Ok(file)
    }

    #[cfg(unix)]
    fn rollback_after_activation<F: PersistenceFaults>(
        &self,
        faults: &F,
        settings_path: &Path,
        recovery_path: &Path,
        previous_bytes: Option<&[u8]>,
        original_error: ConfigError,
    ) -> Result<CommitOutcome, ConfigError> {
        if faults.check(FaultPoint::DuringRollback).is_err() {
            return Err(ConfigError::RecoveryRequired);
        }
        let rollback = match previous_bytes {
            Some(bytes) => fs::rename(recovery_path, settings_path)
                .map_err(|_| ConfigError::RecoveryRequired)
                .and_then(|()| sync_directory(self.root.state_root()))
                .and_then(|()| verify_restored_document(settings_path, bytes)),
            None => fs::remove_file(settings_path)
                .map_err(|_| ConfigError::RecoveryRequired)
                .and_then(|()| sync_directory(self.root.state_root()))
                .and_then(|()| verify_restored_absence(settings_path))
                .and_then(|()| {
                    fs::remove_file(recovery_path).map_err(|_| ConfigError::RecoveryRequired)
                })
                .and_then(|()| sync_directory(self.root.state_root())),
        };
        match rollback {
            Ok(()) => Err(original_error),
            Err(_) => Err(ConfigError::RecoveryRequired),
        }
    }

    fn load_state(&self) -> Result<StoreLoad, ConfigError> {
        if !validate_existing_directory_chain(self.root.compatibility_root())? {
            return Ok(StoreLoad::missing());
        }
        let Some(state_metadata) = metadata_if_exists(self.root.state_root())? else {
            return Ok(StoreLoad::missing());
        };
        validate_managed_directory(&state_metadata)?;

        let settings_path = self.root.state_root().join(GLOBAL_SETTINGS_FILE);
        let Some(settings_metadata) = metadata_if_exists(&settings_path)? else {
            return Ok(StoreLoad::missing());
        };
        validate_managed_file(&settings_metadata)?;
        let bytes = read_bounded_file(&settings_path, &settings_metadata)?;
        let loaded = decode_global_settings(&bytes)?;
        Ok(StoreLoad {
            loaded,
            present: true,
            #[cfg(unix)]
            bytes: Some(bytes),
        })
    }

    fn has_cleanup_artifact(&self) -> bool {
        fs::read_dir(self.root.state_root())
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .any(|entry| {
                entry.file_name().to_str().is_some_and(|name| {
                    name.starts_with(RECOVERY_FILE_PREFIX) || name.starts_with(STAGING_FILE_PREFIX)
                })
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitOutcome {
    pub revision: u64,
    pub cleanup_required: bool,
}

struct StoreLoad {
    loaded: LoadedGlobalSettings,
    present: bool,
    #[cfg(unix)]
    bytes: Option<Vec<u8>>,
}

impl StoreLoad {
    fn missing() -> Self {
        Self {
            loaded: LoadedGlobalSettings {
                revision: 0,
                settings: GlobalSettings::default(),
            },
            present: false,
            #[cfg(unix)]
            bytes: None,
        }
    }
}

#[cfg(windows)]
const fn readable_state(_present: bool) -> ConfigState {
    ConfigState::WriteUnsupported
}

#[cfg(not(windows))]
const fn readable_state(present: bool) -> ConfigState {
    if present {
        ConfigState::Ready
    } else {
        ConfigState::Missing
    }
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum FaultPoint {
    AfterLock,
    AfterCurrentRead,
    AfterStagingSync,
    AfterRecoverySync,
    AfterActivation,
    BeforeCommitDirectorySync,
    DuringCleanup,
    DuringRollback,
}

#[cfg(unix)]
trait PersistenceFaults {
    fn check(&self, point: FaultPoint) -> Result<(), ConfigError>;
}

#[cfg(unix)]
struct NoFaults;

#[cfg(unix)]
impl PersistenceFaults for NoFaults {
    fn check(&self, _point: FaultPoint) -> Result<(), ConfigError> {
        Ok(())
    }
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    match builder.create(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(ConfigError::PersistenceFailed {
            stage: PersistenceStage::CreateStore,
            kind: error.kind(),
        }),
    }
}

#[cfg(unix)]
fn open_lock_file(path: &Path) -> Result<File, ConfigError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::AcquireLock,
            kind: error.kind(),
        })
}

#[cfg(unix)]
fn write_unique_transaction_file(
    state_root: &Path,
    prefix: &str,
    bytes: &[u8],
    write_stage: PersistenceStage,
    sync_stage: PersistenceStage,
) -> Result<PathBuf, ConfigError> {
    use std::os::unix::fs::OpenOptionsExt;

    for _ in 0..128 {
        let path = state_root.join(format!("{prefix}{}", transaction_token()));
        let mut file = match OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(ConfigError::PersistenceFailed {
                    stage: write_stage,
                    kind: error.kind(),
                });
            }
        };
        let write_result = file
            .write_all(bytes)
            .map_err(|error| ConfigError::PersistenceFailed {
                stage: write_stage,
                kind: error.kind(),
            })
            .and_then(|()| {
                file.flush()
                    .map_err(|error| ConfigError::PersistenceFailed {
                        stage: sync_stage,
                        kind: error.kind(),
                    })
            })
            .and_then(|()| {
                file.sync_all()
                    .map_err(|error| ConfigError::PersistenceFailed {
                        stage: sync_stage,
                        kind: error.kind(),
                    })
            })
            .and_then(|()| {
                file.metadata()
                    .map_err(|error| ConfigError::PersistenceFailed {
                        stage: sync_stage,
                        kind: error.kind(),
                    })
            })
            .and_then(|metadata| validate_managed_file(&metadata));
        drop(file);
        match write_result {
            Ok(()) => return Ok(path),
            Err(error) => {
                remove_transaction_file(&path);
                return Err(error);
            }
        }
    }
    Err(ConfigError::PersistenceFailed {
        stage: write_stage,
        kind: io::ErrorKind::AlreadyExists,
    })
}

#[cfg(unix)]
fn transaction_token() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = NEXT_TRANSACTION.fetch_add(1, Ordering::Relaxed);
    format!("{}-{timestamp}-{counter}", std::process::id())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ConfigError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::SyncDirectory,
            kind: error.kind(),
        })
}

#[cfg(unix)]
fn verify_committed_document(
    path: &Path,
    expected_bytes: &[u8],
    expected_revision: u64,
) -> Result<(), ConfigError> {
    let metadata = metadata_if_exists(path)?.ok_or(ConfigError::RecoveryRequired)?;
    validate_managed_file(&metadata)?;
    let bytes = read_bounded_file(path, &metadata)?;
    let loaded = decode_global_settings(&bytes)?;
    if bytes != expected_bytes || loaded.revision != expected_revision {
        return Err(ConfigError::RecoveryRequired);
    }
    Ok(())
}

#[cfg(unix)]
fn verify_restored_document(path: &Path, expected_bytes: &[u8]) -> Result<(), ConfigError> {
    let metadata = metadata_if_exists(path)?.ok_or(ConfigError::RecoveryRequired)?;
    validate_managed_file(&metadata)?;
    let bytes = read_bounded_file(path, &metadata)?;
    if bytes != expected_bytes {
        return Err(ConfigError::RecoveryRequired);
    }
    decode_global_settings(&bytes)?;
    Ok(())
}

#[cfg(unix)]
fn verify_restored_absence(path: &Path) -> Result<(), ConfigError> {
    if metadata_if_exists(path)?.is_some() {
        Err(ConfigError::RecoveryRequired)
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn remove_transaction_file(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => {}
    }
}

fn validate_existing_directory_chain(path: &Path) -> Result<bool, ConfigError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(_) => {
                current.push(component.as_os_str());
                let metadata = match fs::symlink_metadata(&current) {
                    Ok(metadata) => metadata,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
                    Err(error) => {
                        return Err(ConfigError::PersistenceFailed {
                            stage: PersistenceStage::Inspect,
                            kind: error.kind(),
                        });
                    }
                };
                if metadata.file_type().is_symlink()
                    || is_reparse_point(&metadata)
                    || !metadata.is_dir()
                {
                    return Err(ConfigError::UnsafeManagedPath);
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(ConfigError::UnsafeManagedPath);
            }
        }
    }
    Ok(true)
}

fn metadata_if_exists(path: &Path) -> Result<Option<Metadata>, ConfigError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ConfigError::PersistenceFailed {
            stage: PersistenceStage::Inspect,
            kind: error.kind(),
        }),
    }
}

fn validate_managed_directory(metadata: &Metadata) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_dir() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    validate_owner_only(metadata)
}

fn validate_managed_file(metadata: &Metadata) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_file() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    validate_owner_only(metadata)?;
    if has_multiple_hard_links(metadata) {
        return Err(ConfigError::UnsafeManagedPath);
    }
    Ok(())
}

fn read_bounded_file(path: &Path, expected: &Metadata) -> Result<Vec<u8>, ConfigError> {
    let file = File::open(path).map_err(|error| ConfigError::PersistenceFailed {
        stage: PersistenceStage::ReadCurrent,
        kind: error.kind(),
    })?;
    let opened = file
        .metadata()
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::ReadCurrent,
            kind: error.kind(),
        })?;
    validate_managed_file(&opened)?;
    if !same_file_identity(expected, &opened) {
        return Err(ConfigError::UnsafeManagedPath);
    }
    let mut bytes = Vec::new();
    file.take((MAX_GLOBAL_SETTINGS_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::ReadCurrent,
            kind: error.kind(),
        })?;
    if bytes.len() > MAX_GLOBAL_SETTINGS_BYTES {
        return Err(ConfigError::DocumentTooLarge);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn validate_owner_only(metadata: &Metadata) -> Result<(), ConfigError> {
    use std::os::unix::fs::MetadataExt;

    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
        return Err(ConfigError::InsecurePermissions);
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_owner_only(_metadata: &Metadata) -> Result<(), ConfigError> {
    Ok(())
}

#[cfg(unix)]
fn same_file_identity(expected: &Metadata, opened: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    expected.dev() == opened.dev() && expected.ino() == opened.ino()
}

#[cfg(not(unix))]
fn same_file_identity(_expected: &Metadata, _opened: &Metadata) -> bool {
    true
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
