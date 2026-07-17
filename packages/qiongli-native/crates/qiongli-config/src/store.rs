#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(any(unix, windows))]
use std::fs::TryLockError;
use std::fs::{self, File, Metadata};
#[cfg(any(unix, windows))]
use std::io::Write;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
#[cfg(any(unix, windows))]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(any(unix, windows))]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::document::decode_global_settings;
#[cfg(any(unix, windows))]
use crate::document::encode_global_settings;
use crate::{
    ConfigError, ConfigRoot, ConfigState, GlobalSettings, LoadedGlobalSettings,
    MAX_GLOBAL_SETTINGS_BYTES, PersistenceStage, RedactedConfigStatus,
};

pub const GLOBAL_SETTINGS_FILE: &str = "settings.json";
#[cfg(any(unix, windows))]
const LOCK_FILE: &str = ".settings.lock";
const STAGING_FILE_PREFIX: &str = ".settings.json.qiongli-stage-";
const RECOVERY_FILE_PREFIX: &str = ".settings.json.qiongli-recovery-";
#[cfg(any(unix, windows))]
const ABSENT_RECOVERY_MARKER: &[u8] = b"qiongli-settings-recovery-v1:absent\n";
#[cfg(any(unix, windows))]
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(any(unix, windows))]
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(any(unix, windows))]
static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct GlobalSettingsStore {
    root: ConfigRoot,
    #[cfg(any(unix, windows))]
    lock_timeout: Duration,
}

impl GlobalSettingsStore {
    #[must_use]
    pub const fn new(root: ConfigRoot) -> Self {
        Self {
            root,
            #[cfg(any(unix, windows))]
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
                let cleanup_required = self.has_cleanup_artifact();
                let config_state = if cleanup_required {
                    ConfigState::RecoveryRequired
                } else {
                    readable_state(state.present)
                };
                RedactedConfigStatus::loaded(
                    &self.root,
                    config_state,
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
        #[cfg(any(unix, windows))]
        {
            self.replace_supported_with(expected_revision, replacement, &NoFaults)
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (expected_revision, replacement);
            Err(ConfigError::UnsupportedPlatformSecurity)
        }
    }

    #[cfg(any(unix, windows))]
    fn replace_supported_with<F: PersistenceFaults>(
        &self,
        expected_revision: u64,
        replacement: GlobalSettings,
        faults: &F,
    ) -> Result<CommitOutcome, ConfigError> {
        self.prepare_store()?;
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
        if let Err(error) = replace_file(&staging, &settings_path, true, PersistenceStage::Activate)
        {
            remove_transaction_file(&staging);
            remove_transaction_file(&recovery);
            return Err(error);
        }

        if let Err(error) = faults.check(FaultPoint::AfterActivation) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &staging,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }
        if let Err(error) = faults.check(FaultPoint::BeforeCommitDirectorySync) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &staging,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }
        if let Err(error) = sync_directory(self.root.state_root()) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &staging,
                &recovery,
                current.bytes.as_deref(),
                error,
            );
        }
        if let Err(error) = verify_committed_document(&settings_path, &next_bytes, revision) {
            return self.rollback_after_activation(
                faults,
                &settings_path,
                &staging,
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

    #[cfg(any(unix, windows))]
    fn prepare_store(&self) -> Result<(), ConfigError> {
        if !validate_existing_directory_chain(self.root.compatibility_root())? {
            create_compatibility_directory_chain(self.root.compatibility_root())?;
            if !validate_existing_directory_chain(self.root.compatibility_root())? {
                return Err(ConfigError::UnsafeManagedPath);
            }
        }
        match metadata_if_exists(self.root.state_root())? {
            Some(metadata) => validate_managed_directory(self.root.state_root(), &metadata)?,
            None => {
                create_private_directory(self.root.state_root())?;
                let metadata = metadata_if_exists(self.root.state_root())?
                    .ok_or(ConfigError::UnsafeManagedPath)?;
                validate_managed_directory(self.root.state_root(), &metadata)?;
                sync_directory(self.root.compatibility_root())?;
            }
        }
        Ok(())
    }

    #[cfg(any(unix, windows))]
    fn acquire_lock(&self) -> Result<File, ConfigError> {
        let lock_path = self.root.state_root().join(LOCK_FILE);
        let expected = metadata_if_exists(&lock_path)?;
        if let Some(metadata) = expected.as_ref() {
            validate_managed_file(&lock_path, metadata)?;
        }
        let file = open_lock_file(&lock_path)?;
        let opened = file
            .metadata()
            .map_err(|error| ConfigError::PersistenceFailed {
                stage: PersistenceStage::AcquireLock,
                kind: error.kind(),
            })?;
        validate_opened_managed_file(&file, &opened, PersistenceStage::AcquireLock)?;
        let linked = metadata_if_exists(&lock_path)?.ok_or(ConfigError::UnsafeManagedPath)?;
        validate_managed_file(&lock_path, &linked)?;
        validate_lock_identity(&lock_path, &file, &linked, expected.as_ref())?;

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
        validate_managed_file(&lock_path, &linked_after)?;
        validate_lock_identity(&lock_path, &file, &linked_after, Some(&opened))?;
        Ok(file)
    }

    #[cfg(any(unix, windows))]
    fn rollback_after_activation<F: PersistenceFaults>(
        &self,
        faults: &F,
        settings_path: &Path,
        staging_path: &Path,
        recovery_path: &Path,
        previous_bytes: Option<&[u8]>,
        original_error: ConfigError,
    ) -> Result<CommitOutcome, ConfigError> {
        if faults.check(FaultPoint::DuringRollback).is_err() {
            return Err(ConfigError::RecoveryRequired);
        }
        let rollback = match previous_bytes {
            Some(bytes) => replace_file(
                recovery_path,
                settings_path,
                true,
                PersistenceStage::Rollback,
            )
            .map_err(|_| ConfigError::RecoveryRequired)
            .and_then(|()| sync_directory(self.root.state_root()))
            .and_then(|()| verify_restored_document(settings_path, bytes)),
            None => rollback_to_absence(
                settings_path,
                staging_path,
                recovery_path,
                self.root.state_root(),
            ),
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
        validate_managed_directory(self.root.state_root(), &state_metadata)?;

        let settings_path = self.root.state_root().join(GLOBAL_SETTINGS_FILE);
        let Some(settings_metadata) = metadata_if_exists(&settings_path)? else {
            return Ok(StoreLoad::missing());
        };
        validate_managed_file(&settings_path, &settings_metadata)?;
        let bytes = read_bounded_file(&settings_path, &settings_metadata)?;
        let loaded = decode_global_settings(&bytes)?;
        Ok(StoreLoad {
            loaded,
            present: true,
            #[cfg(any(unix, windows))]
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
    #[cfg(any(unix, windows))]
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
            #[cfg(any(unix, windows))]
            bytes: None,
        }
    }
}

#[cfg(any(unix, windows))]
const fn readable_state(present: bool) -> ConfigState {
    if present {
        ConfigState::Ready
    } else {
        ConfigState::Missing
    }
}

#[cfg(not(any(unix, windows)))]
const fn readable_state(_present: bool) -> ConfigState {
    ConfigState::WriteUnsupported
}

#[cfg(any(unix, windows))]
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

#[cfg(any(unix, windows))]
trait PersistenceFaults {
    fn check(&self, point: FaultPoint) -> Result<(), ConfigError>;
}

#[cfg(any(unix, windows))]
struct NoFaults;

#[cfg(any(unix, windows))]
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

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), ConfigError> {
    match qiongli_windows_security::create_owner_only_directory(path) {
        Ok(directory) => qiongli_windows_security::verify_owner_only_directory_handle(&directory)
            .map_err(|error| map_windows_security_error(error, PersistenceStage::CreateStore)),
        Err(qiongli_windows_security::SecurityError::Io(io::ErrorKind::AlreadyExists)) => Ok(()),
        Err(error) => Err(map_windows_security_error(
            error,
            PersistenceStage::CreateStore,
        )),
    }
}

#[cfg(unix)]
fn create_compatibility_directory_chain(path: &Path) -> Result<(), ConfigError> {
    fs::create_dir_all(path).map_err(|error| ConfigError::PersistenceFailed {
        stage: PersistenceStage::CreateStore,
        kind: error.kind(),
    })
}

#[cfg(windows)]
fn create_compatibility_directory_chain(path: &Path) -> Result<(), ConfigError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(_) => {
                current.push(component.as_os_str());
                match fs::symlink_metadata(&current) {
                    Ok(metadata) => validate_directory_component(&current, &metadata)?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        match fs::create_dir(&current) {
                            Ok(()) => {}
                            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                            Err(error) => {
                                return Err(ConfigError::PersistenceFailed {
                                    stage: PersistenceStage::CreateStore,
                                    kind: error.kind(),
                                });
                            }
                        }
                        let metadata = fs::symlink_metadata(&current).map_err(|error| {
                            ConfigError::PersistenceFailed {
                                stage: PersistenceStage::Inspect,
                                kind: error.kind(),
                            }
                        })?;
                        validate_directory_component(&current, &metadata)?;
                    }
                    Err(error) => {
                        return Err(ConfigError::PersistenceFailed {
                            stage: PersistenceStage::Inspect,
                            kind: error.kind(),
                        });
                    }
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(ConfigError::UnsafeManagedPath);
            }
        }
    }
    Ok(())
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

#[cfg(windows)]
fn open_lock_file(path: &Path) -> Result<File, ConfigError> {
    qiongli_windows_security::open_or_create_owner_only_lock(path)
        .map_err(|error| map_windows_security_error(error, PersistenceStage::AcquireLock))
}

#[cfg(any(unix, windows))]
fn write_unique_transaction_file(
    state_root: &Path,
    prefix: &str,
    bytes: &[u8],
    write_stage: PersistenceStage,
    sync_stage: PersistenceStage,
) -> Result<PathBuf, ConfigError> {
    for _ in 0..128 {
        let path = state_root.join(format!("{prefix}{}", transaction_token()));
        let mut file = match create_private_new_file(&path, write_stage) {
            Ok(file) => file,
            Err(ConfigError::PersistenceFailed {
                kind: io::ErrorKind::AlreadyExists,
                ..
            }) => continue,
            Err(error) => {
                return Err(error);
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
            .and_then(|metadata| validate_opened_managed_file(&file, &metadata, sync_stage));
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
fn create_private_new_file(path: &Path, stage: PersistenceStage) -> Result<File, ConfigError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| ConfigError::PersistenceFailed {
            stage,
            kind: error.kind(),
        })
}

#[cfg(windows)]
fn create_private_new_file(path: &Path, stage: PersistenceStage) -> Result<File, ConfigError> {
    qiongli_windows_security::create_owner_only_new_file(path)
        .map_err(|error| map_windows_security_error(error, stage))
}

#[cfg(any(unix, windows))]
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

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), ConfigError> {
    // MoveFileExW with MOVEFILE_WRITE_THROUGH is the documented Windows activation boundary.
    Ok(())
}

#[cfg(unix)]
fn replace_file(
    source: &Path,
    destination: &Path,
    _replace_existing: bool,
    stage: PersistenceStage,
) -> Result<(), ConfigError> {
    fs::rename(source, destination).map_err(|error| ConfigError::PersistenceFailed {
        stage,
        kind: error.kind(),
    })
}

#[cfg(windows)]
fn replace_file(
    source: &Path,
    destination: &Path,
    replace_existing: bool,
    stage: PersistenceStage,
) -> Result<(), ConfigError> {
    qiongli_windows_security::move_file_write_through(source, destination, replace_existing)
        .map_err(|error| map_windows_security_error(error, stage))
}

#[cfg(unix)]
fn rollback_to_absence(
    settings_path: &Path,
    _staging_path: &Path,
    recovery_path: &Path,
    state_root: &Path,
) -> Result<(), ConfigError> {
    fs::remove_file(settings_path)
        .map_err(|_| ConfigError::RecoveryRequired)
        .and_then(|()| sync_directory(state_root))
        .and_then(|()| verify_restored_absence(settings_path))
        .and_then(|()| fs::remove_file(recovery_path).map_err(|_| ConfigError::RecoveryRequired))
        .and_then(|()| sync_directory(state_root))
}

#[cfg(windows)]
fn rollback_to_absence(
    settings_path: &Path,
    staging_path: &Path,
    recovery_path: &Path,
    state_root: &Path,
) -> Result<(), ConfigError> {
    replace_file(
        settings_path,
        staging_path,
        false,
        PersistenceStage::Rollback,
    )
    .map_err(|_| ConfigError::RecoveryRequired)
    .and_then(|()| verify_restored_absence(settings_path))
    .and_then(|()| {
        let metadata = metadata_if_exists(staging_path)?.ok_or(ConfigError::RecoveryRequired)?;
        validate_managed_file(staging_path, &metadata).map_err(|_| ConfigError::RecoveryRequired)
    })
    .and_then(|()| fs::remove_file(staging_path).map_err(|_| ConfigError::RecoveryRequired))
    .and_then(|()| fs::remove_file(recovery_path).map_err(|_| ConfigError::RecoveryRequired))
    .and_then(|()| sync_directory(state_root))
}

#[cfg(any(unix, windows))]
fn verify_committed_document(
    path: &Path,
    expected_bytes: &[u8],
    expected_revision: u64,
) -> Result<(), ConfigError> {
    let metadata = metadata_if_exists(path)?.ok_or(ConfigError::RecoveryRequired)?;
    validate_managed_file(path, &metadata)?;
    let bytes = read_bounded_file(path, &metadata)?;
    let loaded = decode_global_settings(&bytes)?;
    if bytes != expected_bytes || loaded.revision != expected_revision {
        return Err(ConfigError::RecoveryRequired);
    }
    Ok(())
}

#[cfg(any(unix, windows))]
fn verify_restored_document(path: &Path, expected_bytes: &[u8]) -> Result<(), ConfigError> {
    let metadata = metadata_if_exists(path)?.ok_or(ConfigError::RecoveryRequired)?;
    validate_managed_file(path, &metadata)?;
    let bytes = read_bounded_file(path, &metadata)?;
    if bytes != expected_bytes {
        return Err(ConfigError::RecoveryRequired);
    }
    decode_global_settings(&bytes)?;
    Ok(())
}

#[cfg(any(unix, windows))]
fn verify_restored_absence(path: &Path) -> Result<(), ConfigError> {
    if metadata_if_exists(path)?.is_some() {
        Err(ConfigError::RecoveryRequired)
    } else {
        Ok(())
    }
}

#[cfg(any(unix, windows))]
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
                validate_directory_component(&current, &metadata)?;
            }
            Component::CurDir | Component::ParentDir => {
                return Err(ConfigError::UnsafeManagedPath);
            }
        }
    }
    Ok(true)
}

fn validate_directory_component(path: &Path, metadata: &Metadata) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_dir() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    #[cfg(windows)]
    {
        qiongli_windows_security::open_directory_no_reparse(path)
            .map(|_| ())
            .map_err(|error| map_windows_security_error(error, PersistenceStage::Inspect))?;
    }
    #[cfg(not(windows))]
    let _ = path;
    Ok(())
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

fn validate_managed_directory(path: &Path, metadata: &Metadata) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_dir() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    validate_owner_only_directory(path, metadata)
}

fn validate_managed_file(path: &Path, metadata: &Metadata) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_file() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    validate_owner_only_file(path, metadata)
}

fn read_bounded_file(path: &Path, expected: &Metadata) -> Result<Vec<u8>, ConfigError> {
    let file = open_managed_file_for_read(path)?;
    let opened = file
        .metadata()
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::ReadCurrent,
            kind: error.kind(),
        })?;
    validate_opened_managed_file(&file, &opened, PersistenceStage::ReadCurrent)?;
    validate_read_identity(path, expected, &file, &opened)?;
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
fn validate_owner_only_metadata(metadata: &Metadata) -> Result<(), ConfigError> {
    use std::os::unix::fs::MetadataExt;

    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
        return Err(ConfigError::InsecurePermissions);
    }
    Ok(())
}

#[cfg(unix)]
fn validate_owner_only_directory(_path: &Path, metadata: &Metadata) -> Result<(), ConfigError> {
    validate_owner_only_metadata(metadata)
}

#[cfg(windows)]
fn validate_owner_only_directory(path: &Path, _metadata: &Metadata) -> Result<(), ConfigError> {
    qiongli_windows_security::open_owner_only_directory(path)
        .map(|_| ())
        .map_err(|error| map_windows_security_error(error, PersistenceStage::Inspect))
}

#[cfg(not(any(unix, windows)))]
fn validate_owner_only_directory(_path: &Path, _metadata: &Metadata) -> Result<(), ConfigError> {
    Ok(())
}

#[cfg(unix)]
fn validate_owner_only_file(_path: &Path, metadata: &Metadata) -> Result<(), ConfigError> {
    validate_owner_only_metadata(metadata)?;
    if has_multiple_hard_links(metadata) {
        return Err(ConfigError::UnsafeManagedPath);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_owner_only_file(path: &Path, _metadata: &Metadata) -> Result<(), ConfigError> {
    qiongli_windows_security::open_owner_only_file(path)
        .map(|_| ())
        .map_err(|error| map_windows_security_error(error, PersistenceStage::Inspect))
}

#[cfg(not(any(unix, windows)))]
fn validate_owner_only_file(_path: &Path, _metadata: &Metadata) -> Result<(), ConfigError> {
    Ok(())
}

fn validate_opened_managed_file(
    file: &File,
    metadata: &Metadata,
    stage: PersistenceStage,
) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_file() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    #[cfg(unix)]
    {
        validate_owner_only_metadata(metadata)?;
        if has_multiple_hard_links(metadata) {
            return Err(ConfigError::UnsafeManagedPath);
        }
    }
    #[cfg(windows)]
    qiongli_windows_security::verify_owner_only_file_handle(file)
        .map_err(|error| map_windows_security_error(error, stage))?;
    #[cfg(not(windows))]
    let _ = (file, stage);
    Ok(())
}

#[cfg(unix)]
fn open_managed_file_for_read(path: &Path) -> Result<File, ConfigError> {
    File::open(path).map_err(|error| ConfigError::PersistenceFailed {
        stage: PersistenceStage::ReadCurrent,
        kind: error.kind(),
    })
}

#[cfg(windows)]
fn open_managed_file_for_read(path: &Path) -> Result<File, ConfigError> {
    qiongli_windows_security::open_owner_only_file(path)
        .map_err(|error| map_windows_security_error(error, PersistenceStage::ReadCurrent))
}

#[cfg(not(any(unix, windows)))]
fn open_managed_file_for_read(path: &Path) -> Result<File, ConfigError> {
    File::open(path).map_err(|error| ConfigError::PersistenceFailed {
        stage: PersistenceStage::ReadCurrent,
        kind: error.kind(),
    })
}

#[cfg(unix)]
fn validate_read_identity(
    _path: &Path,
    expected: &Metadata,
    _file: &File,
    opened: &Metadata,
) -> Result<(), ConfigError> {
    if same_file_identity(expected, opened) {
        Ok(())
    } else {
        Err(ConfigError::UnsafeManagedPath)
    }
}

#[cfg(windows)]
fn validate_read_identity(
    path: &Path,
    _expected: &Metadata,
    file: &File,
    _opened: &Metadata,
) -> Result<(), ConfigError> {
    let linked = qiongli_windows_security::open_owner_only_file(path)
        .map_err(|error| map_windows_security_error(error, PersistenceStage::ReadCurrent))?;
    if same_windows_file_identity(file, &linked, PersistenceStage::ReadCurrent)? {
        Ok(())
    } else {
        Err(ConfigError::UnsafeManagedPath)
    }
}

#[cfg(not(any(unix, windows)))]
fn validate_read_identity(
    _path: &Path,
    _expected: &Metadata,
    _file: &File,
    _opened: &Metadata,
) -> Result<(), ConfigError> {
    Ok(())
}

#[cfg(unix)]
fn validate_lock_identity(
    _path: &Path,
    file: &File,
    linked: &Metadata,
    expected: Option<&Metadata>,
) -> Result<(), ConfigError> {
    let opened = file
        .metadata()
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::AcquireLock,
            kind: error.kind(),
        })?;
    if !same_file_identity(linked, &opened)
        || expected.is_some_and(|metadata| !same_file_identity(metadata, &opened))
    {
        return Err(ConfigError::UnsafeManagedPath);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_lock_identity(
    path: &Path,
    file: &File,
    _linked: &Metadata,
    _expected: Option<&Metadata>,
) -> Result<(), ConfigError> {
    qiongli_windows_security::verify_owner_only_file_handle(file)
        .map_err(|error| map_windows_security_error(error, PersistenceStage::AcquireLock))?;
    let linked = qiongli_windows_security::open_owner_only_file(path)
        .map_err(|error| map_windows_security_error(error, PersistenceStage::AcquireLock))?;
    if same_windows_file_identity(file, &linked, PersistenceStage::AcquireLock)? {
        Ok(())
    } else {
        Err(ConfigError::UnsafeManagedPath)
    }
}

#[cfg(unix)]
fn same_file_identity(expected: &Metadata, opened: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    expected.dev() == opened.dev() && expected.ino() == opened.ino()
}

#[cfg(windows)]
fn same_windows_file_identity(
    first: &File,
    second: &File,
    stage: PersistenceStage,
) -> Result<bool, ConfigError> {
    let first = qiongli_windows_security::handle_facts(first)
        .map_err(|error| map_windows_security_error(error, stage))?;
    let second = qiongli_windows_security::handle_facts(second)
        .map_err(|error| map_windows_security_error(error, stage))?;
    Ok(first.identity == second.identity)
}

#[cfg(unix)]
fn has_multiple_hard_links(metadata: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    metadata.nlink() > 1
}

#[cfg(windows)]
fn map_windows_security_error(
    error: qiongli_windows_security::SecurityError,
    stage: PersistenceStage,
) -> ConfigError {
    match error {
        qiongli_windows_security::SecurityError::Io(kind) => {
            ConfigError::PersistenceFailed { stage, kind }
        }
        qiongli_windows_security::SecurityError::InsecurePermissions => {
            ConfigError::InsecurePermissions
        }
        qiongli_windows_security::SecurityError::UnsafeObject => ConfigError::UnsafeManagedPath,
    }
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

#[cfg(all(test, any(unix, windows)))]
mod tests {
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::fs::OpenOptions;
    use std::path::{Path, PathBuf};

    use qiongli_content::ProfileId;

    use super::*;

    static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

    struct TestFixture {
        compatibility_root: PathBuf,
    }

    impl TestFixture {
        fn new(name: &str) -> Self {
            let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("config crate must live below the native workspace");
            let compatibility_root =
                native_root
                    .join("target/qiongli-config-unit-tests")
                    .join(format!(
                        "{name}-{}-{}",
                        std::process::id(),
                        NEXT_TEST.fetch_add(1, Ordering::Relaxed)
                    ));
            let _ = fs::remove_dir_all(&compatibility_root);
            Self { compatibility_root }
        }

        fn store(&self) -> GlobalSettingsStore {
            let root =
                crate::resolve_config_root(Some(OsStr::new(&self.compatibility_root)), test_home())
                    .unwrap();
            GlobalSettingsStore::new(root)
        }

        fn settings_path(&self) -> PathBuf {
            self.compatibility_root
                .join("v2")
                .join(GLOBAL_SETTINGS_FILE)
        }

        fn state_root(&self) -> PathBuf {
            self.compatibility_root.join("v2")
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.compatibility_root);
        }
    }

    struct TestFaults {
        points: BTreeSet<FaultPoint>,
    }

    impl TestFaults {
        fn one(point: FaultPoint) -> Self {
            Self {
                points: BTreeSet::from([point]),
            }
        }

        fn paired(first: FaultPoint, second: FaultPoint) -> Self {
            Self {
                points: BTreeSet::from([first, second]),
            }
        }
    }

    impl PersistenceFaults for TestFaults {
        fn check(&self, point: FaultPoint) -> Result<(), ConfigError> {
            if self.points.contains(&point) {
                Err(ConfigError::PersistenceFailed {
                    stage: fault_stage(point),
                    kind: io::ErrorKind::Other,
                })
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn fault_before_activation_preserves_prior_bytes_and_revision() {
        for point in [
            FaultPoint::AfterLock,
            FaultPoint::AfterCurrentRead,
            FaultPoint::AfterStagingSync,
            FaultPoint::AfterRecoverySync,
        ] {
            let fixture = TestFixture::new("pre-activation");
            let store = fixture.store();
            store.replace(0, GlobalSettings::default()).unwrap();
            let before = fs::read(fixture.settings_path()).unwrap();
            let result = store.replace_supported_with(1, next_settings(), &TestFaults::one(point));
            assert!(matches!(result, Err(ConfigError::PersistenceFailed { .. })));
            assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
            assert_eq!(store.load().unwrap().revision, 1);
            assert!(transaction_files(&fixture.state_root()).is_empty());
        }
    }

    #[test]
    fn fault_after_activation_rolls_back_prior_bytes_and_revision() {
        for point in [
            FaultPoint::AfterActivation,
            FaultPoint::BeforeCommitDirectorySync,
        ] {
            let fixture = TestFixture::new("post-activation");
            let store = fixture.store();
            store.replace(0, GlobalSettings::default()).unwrap();
            let before = fs::read(fixture.settings_path()).unwrap();
            let result = store.replace_supported_with(1, next_settings(), &TestFaults::one(point));
            assert!(matches!(result, Err(ConfigError::PersistenceFailed { .. })));
            assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
            assert_eq!(store.load().unwrap().revision, 1);
            assert!(transaction_files(&fixture.state_root()).is_empty());
        }
    }

    #[test]
    fn failed_first_activation_restores_absence() {
        for point in [
            FaultPoint::AfterActivation,
            FaultPoint::BeforeCommitDirectorySync,
        ] {
            let fixture = TestFixture::new("first-write-rollback");
            let store = fixture.store();
            let result = store.replace_supported_with(0, next_settings(), &TestFaults::one(point));
            assert!(matches!(result, Err(ConfigError::PersistenceFailed { .. })));
            assert!(!fixture.settings_path().exists());
            assert_eq!(store.load().unwrap().revision, 0);
            assert!(transaction_files(&fixture.state_root()).is_empty());
        }
    }

    #[test]
    fn cleanup_fault_reports_committed_revision_and_recovery_marker() {
        let fixture = TestFixture::new("cleanup");
        let store = fixture.store();
        store.replace(0, GlobalSettings::default()).unwrap();
        let outcome = store
            .replace_supported_with(
                1,
                next_settings(),
                &TestFaults::one(FaultPoint::DuringCleanup),
            )
            .unwrap();
        assert_eq!(outcome.revision, 2);
        assert!(outcome.cleanup_required);
        assert_eq!(store.load().unwrap().revision, 2);
        let artifacts = transaction_files(&fixture.state_root());
        assert_eq!(artifacts.len(), 1);
        #[cfg(windows)]
        qiongli_windows_security::open_owner_only_file(&fixture.state_root().join(&artifacts[0]))
            .unwrap();
        let status = store.status();
        assert_eq!(status.state, ConfigState::RecoveryRequired);
        assert!(status.cleanup_required);
    }

    #[test]
    fn rollback_fault_returns_recovery_required_without_false_success() {
        let fixture = TestFixture::new("rollback-failure");
        let store = fixture.store();
        store.replace(0, GlobalSettings::default()).unwrap();
        let result = store.replace_supported_with(
            1,
            next_settings(),
            &TestFaults::paired(FaultPoint::AfterActivation, FaultPoint::DuringRollback),
        );
        assert_eq!(result, Err(ConfigError::RecoveryRequired));
        assert_eq!(store.load().unwrap().revision, 2);
        assert_eq!(transaction_files(&fixture.state_root()).len(), 1);
        assert_eq!(store.status().state, ConfigState::RecoveryRequired);
    }

    #[test]
    fn held_lock_times_out_without_changing_live_bytes() {
        let fixture = TestFixture::new("lock-timeout");
        let store = fixture.store();
        store.replace(0, GlobalSettings::default()).unwrap();
        let before = fs::read(fixture.settings_path()).unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(fixture.state_root().join(LOCK_FILE))
            .unwrap();
        lock.lock().unwrap();
        let mut contender = store.clone();
        contender.lock_timeout = Duration::from_millis(20);
        assert_eq!(
            contender.replace(1, next_settings()),
            Err(ConfigError::LockBusy)
        );
        assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
    }

    fn next_settings() -> GlobalSettings {
        GlobalSettings {
            default_profile: ProfileId::Full,
            ..GlobalSettings::default()
        }
    }

    fn transaction_files(state_root: &Path) -> Vec<String> {
        let mut files = fs::read_dir(state_root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .filter(|name| {
                name.starts_with(STAGING_FILE_PREFIX) || name.starts_with(RECOVERY_FILE_PREFIX)
            })
            .collect::<Vec<_>>();
        files.sort();
        files
    }

    #[cfg(unix)]
    fn test_home() -> &'static Path {
        Path::new("/home/qiongli-test")
    }

    #[cfg(windows)]
    fn test_home() -> &'static Path {
        Path::new(r"C:\Users\qiongli-test")
    }

    const fn fault_stage(point: FaultPoint) -> PersistenceStage {
        match point {
            FaultPoint::AfterLock => PersistenceStage::AcquireLock,
            FaultPoint::AfterCurrentRead => PersistenceStage::ReadCurrent,
            FaultPoint::AfterStagingSync => PersistenceStage::SyncStaging,
            FaultPoint::AfterRecoverySync => PersistenceStage::CreateRecovery,
            FaultPoint::AfterActivation => PersistenceStage::Activate,
            FaultPoint::BeforeCommitDirectorySync => PersistenceStage::SyncDirectory,
            FaultPoint::DuringCleanup => PersistenceStage::Cleanup,
            FaultPoint::DuringRollback => PersistenceStage::Rollback,
        }
    }
}
