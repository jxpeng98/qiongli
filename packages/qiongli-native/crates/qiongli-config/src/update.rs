#[cfg(unix)]
use std::fs::{self, File, Metadata, OpenOptions};
#[cfg(unix)]
use std::io::{self, Read, Write};
use std::path::PathBuf;
#[cfg(unix)]
use std::path::{Component, Path};
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(unix)]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(any(unix, test))]
use semver::Version;
use serde::{Deserialize, Serialize};

#[cfg(unix)]
use crate::PersistenceStage;
#[cfg(any(unix, test))]
use crate::document::parse_unique_json;
use crate::{CommitOutcome, ConfigError, ConfigRoot};

pub const UPDATE_STATE_FILE: &str = "update-state.json";
pub const UPDATE_STATE_DOCUMENT_KIND: &str = "qiongli-update-state";
pub const UPDATE_STATE_SCHEMA_VERSION: u64 = 1;
pub const MAX_UPDATE_STATE_BYTES: usize = 64 * 1024;

#[cfg(any(unix, test))]
const MAX_REVISION: u64 = 9_007_199_254_740_991;
#[cfg(unix)]
const LOCK_FILE: &str = ".update-state.lock";
#[cfg(unix)]
const STAGING_PREFIX: &str = ".update-state.json.qiongli-stage-";
#[cfg(unix)]
const RECOVERY_PREFIX: &str = ".update-state.json.qiongli-recovery-";
#[cfg(unix)]
const ABSENT_RECOVERY_MARKER: &[u8] = b"qiongli-update-state-recovery-v1:absent\n";
#[cfg(unix)]
const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(unix)]
const LOCK_RETRY: Duration = Duration::from_millis(10);
#[cfg(unix)]
static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateStreamPreference {
    Stable,
    Beta,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateReleaseChannel {
    Alpha,
    Beta,
    Stable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateTransactionPhase {
    Downloading,
    Downloaded,
    Cancelling,
    Staged,
    ReconciliationPrepared,
    AwaitingExit,
    Activating,
    HealthWindow,
    RecoveryRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateLastKnownGood {
    pub version: String,
    pub channel: UpdateReleaseChannel,
    pub generation: u64,
    pub archive_sha256: String,
    pub resource_pack_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateActiveTransaction {
    pub transaction_id: String,
    pub target_version: String,
    pub phase: UpdateTransactionPhase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateState {
    pub selected_stream: UpdateStreamPreference,
    pub last_accepted_generation: u64,
    pub last_known_good: Option<UpdateLastKnownGood>,
    pub active_transaction: Option<UpdateActiveTransaction>,
}

impl UpdateState {
    #[must_use]
    pub const fn initial(selected_stream: UpdateStreamPreference) -> Self {
        Self {
            selected_stream,
            last_accepted_generation: 0,
            last_known_good: None,
            active_transaction: None,
        }
    }

    #[cfg(any(unix, test))]
    fn validate(&self) -> Result<(), ConfigError> {
        if self.last_accepted_generation > MAX_REVISION {
            return Err(ConfigError::InvalidDocument);
        }
        if let Some(last_known_good) = &self.last_known_good
            && (last_known_good.generation == 0
                || last_known_good.generation > self.last_accepted_generation
                || !valid_product_version(&last_known_good.version)
                || !channel_matches_version(last_known_good.channel, &last_known_good.version)
                || !valid_sha256(&last_known_good.archive_sha256)
                || !valid_sha256(&last_known_good.resource_pack_sha256))
        {
            return Err(ConfigError::InvalidDocument);
        }
        if let Some(transaction) = &self.active_transaction
            && (!valid_transaction_id(&transaction.transaction_id)
                || !valid_product_version(&transaction.target_version))
        {
            return Err(ConfigError::InvalidDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedUpdateState {
    pub revision: u64,
    pub state: UpdateState,
}

#[derive(Clone, Debug)]
pub struct UpdateStateStore {
    root: ConfigRoot,
    #[cfg(unix)]
    default_stream: UpdateStreamPreference,
}

impl UpdateStateStore {
    #[must_use]
    pub fn new(root: ConfigRoot, default_stream: UpdateStreamPreference) -> Self {
        #[cfg(not(unix))]
        let _ = default_stream;
        Self {
            root,
            #[cfg(unix)]
            default_stream,
        }
    }

    pub fn load(&self) -> Result<LoadedUpdateState, ConfigError> {
        #[cfg(unix)]
        {
            self.load_supported().map(|loaded| loaded.value)
        }
        #[cfg(not(unix))]
        {
            Err(ConfigError::UnsupportedPlatformSecurity)
        }
    }

    pub fn replace(
        &self,
        expected_revision: u64,
        replacement: UpdateState,
    ) -> Result<CommitOutcome, ConfigError> {
        #[cfg(unix)]
        {
            self.replace_supported(expected_revision, replacement)
        }
        #[cfg(not(unix))]
        {
            let _ = (expected_revision, replacement);
            Err(ConfigError::UnsupportedPlatformSecurity)
        }
    }

    #[must_use]
    pub fn symbolic_staging_root(&self) -> &'static str {
        match self.root.source() {
            crate::ConfigRootSource::Default => "<user-home>/.config/qiongli/v2/updates/staging",
            crate::ConfigRootSource::Override => "<configured-root>/v2/updates/staging",
        }
    }

    pub fn staging_root(&self) -> PathBuf {
        self.root.state_root().join("updates").join("staging")
    }

    #[cfg(unix)]
    fn load_supported(&self) -> Result<UpdateStoreLoad, ConfigError> {
        if !validate_existing_directory_chain(self.root.compatibility_root())? {
            return Ok(UpdateStoreLoad::missing(self.default_stream));
        }
        let Some(root_metadata) = metadata_if_exists(self.root.state_root())? else {
            return Ok(UpdateStoreLoad::missing(self.default_stream));
        };
        validate_private_directory(self.root.state_root(), &root_metadata)?;
        if self.has_transaction_artifact() {
            return Err(ConfigError::RecoveryRequired);
        }
        let path = self.root.state_root().join(UPDATE_STATE_FILE);
        let Some(metadata) = metadata_if_exists(&path)? else {
            return Ok(UpdateStoreLoad::missing(self.default_stream));
        };
        validate_private_file(&metadata)?;
        let bytes = read_bounded_private_file(&path, &metadata)?;
        let value = decode_update_state(&bytes)?;
        Ok(UpdateStoreLoad {
            value,
            bytes: Some(bytes),
        })
    }

    #[cfg(unix)]
    fn replace_supported(
        &self,
        expected_revision: u64,
        replacement: UpdateState,
    ) -> Result<CommitOutcome, ConfigError> {
        replacement.validate()?;
        self.prepare_root()?;
        let _lock = self.acquire_lock()?;
        let current = self.load_supported()?;
        if current.value.revision != expected_revision {
            return Err(ConfigError::RevisionConflict {
                observed: current.value.revision,
            });
        }
        let revision = current
            .value
            .revision
            .checked_add(1)
            .filter(|revision| *revision <= MAX_REVISION)
            .ok_or(ConfigError::RevisionExhausted)?;
        let next_bytes = encode_update_state(&replacement, revision)?;
        let staging = write_private_transaction_file(
            self.root.state_root(),
            STAGING_PREFIX,
            &next_bytes,
            PersistenceStage::WriteStaging,
        )?;
        let recovery_bytes = current.bytes.as_deref().unwrap_or(ABSENT_RECOVERY_MARKER);
        let recovery = match write_private_transaction_file(
            self.root.state_root(),
            RECOVERY_PREFIX,
            recovery_bytes,
            PersistenceStage::CreateRecovery,
        ) {
            Ok(path) => path,
            Err(error) => {
                remove_if_present(&staging);
                return Err(error);
            }
        };
        if let Err(error) = sync_directory(self.root.state_root()) {
            remove_if_present(&staging);
            remove_if_present(&recovery);
            return Err(error);
        }
        let live = self.root.state_root().join(UPDATE_STATE_FILE);
        if let Err(error) =
            fs::rename(&staging, &live).map_err(|error| ConfigError::PersistenceFailed {
                stage: PersistenceStage::Activate,
                kind: error.kind(),
            })
        {
            remove_if_present(&staging);
            remove_if_present(&recovery);
            return Err(error);
        }
        let commit = sync_directory(self.root.state_root())
            .and_then(|()| verify_live_document(&live, &next_bytes, revision));
        if let Err(error) = commit {
            return self.rollback(&live, &recovery, current.bytes.as_deref(), error);
        }
        if fs::remove_file(&recovery).is_err() || sync_directory(self.root.state_root()).is_err() {
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
    fn prepare_root(&self) -> Result<(), ConfigError> {
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
            Some(metadata) => validate_private_directory(self.root.state_root(), &metadata),
            None => {
                use std::os::unix::fs::DirBuilderExt;
                let mut builder = fs::DirBuilder::new();
                builder.mode(0o700);
                match builder.create(self.root.state_root()) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        return Err(ConfigError::PersistenceFailed {
                            stage: PersistenceStage::CreateStore,
                            kind: error.kind(),
                        });
                    }
                }
                let metadata = fs::symlink_metadata(self.root.state_root()).map_err(|error| {
                    ConfigError::PersistenceFailed {
                        stage: PersistenceStage::Inspect,
                        kind: error.kind(),
                    }
                })?;
                validate_private_directory(self.root.state_root(), &metadata)?;
                sync_directory(self.root.compatibility_root())
            }
        }
    }

    #[cfg(unix)]
    fn acquire_lock(&self) -> Result<File, ConfigError> {
        use std::fs::TryLockError;
        use std::os::unix::fs::OpenOptionsExt;

        let path = self.root.state_root().join(LOCK_FILE);
        if let Some(metadata) = metadata_if_exists(&path)? {
            validate_private_file(&metadata)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(&path)
            .map_err(|error| ConfigError::PersistenceFailed {
                stage: PersistenceStage::AcquireLock,
                kind: error.kind(),
            })?;
        let opened = file
            .metadata()
            .map_err(|error| ConfigError::PersistenceFailed {
                stage: PersistenceStage::AcquireLock,
                kind: error.kind(),
            })?;
        validate_private_file(&opened)?;
        let linked =
            fs::symlink_metadata(&path).map_err(|error| ConfigError::PersistenceFailed {
                stage: PersistenceStage::AcquireLock,
                kind: error.kind(),
            })?;
        validate_private_file(&linked)?;
        if !same_file_identity(&opened, &linked) {
            return Err(ConfigError::UnsafeManagedPath);
        }
        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(TryLockError::WouldBlock) if started.elapsed() >= LOCK_TIMEOUT => {
                    return Err(ConfigError::LockBusy);
                }
                Err(TryLockError::WouldBlock) => std::thread::sleep(LOCK_RETRY),
                Err(TryLockError::Error(error)) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(TryLockError::Error(error)) => {
                    return Err(ConfigError::PersistenceFailed {
                        stage: PersistenceStage::AcquireLock,
                        kind: error.kind(),
                    });
                }
            }
        }
    }

    #[cfg(unix)]
    fn rollback(
        &self,
        live: &Path,
        recovery: &Path,
        previous: Option<&[u8]>,
        original_error: ConfigError,
    ) -> Result<CommitOutcome, ConfigError> {
        let restored = match previous {
            Some(bytes) => fs::rename(recovery, live)
                .map_err(|_| ConfigError::RecoveryRequired)
                .and_then(|()| sync_directory(self.root.state_root()))
                .and_then(|()| verify_restored_document(live, bytes)),
            None => fs::remove_file(live)
                .map_err(|_| ConfigError::RecoveryRequired)
                .and_then(|()| sync_directory(self.root.state_root()))
                .and_then(|()| {
                    (!live.exists())
                        .then_some(())
                        .ok_or(ConfigError::RecoveryRequired)
                })
                .and_then(|()| fs::remove_file(recovery).map_err(|_| ConfigError::RecoveryRequired))
                .and_then(|()| sync_directory(self.root.state_root())),
        };
        restored.map_or(Err(ConfigError::RecoveryRequired), |()| Err(original_error))
    }

    #[cfg(unix)]
    fn has_transaction_artifact(&self) -> bool {
        fs::read_dir(self.root.state_root())
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .any(|entry| {
                entry.file_name().to_str().is_some_and(|name| {
                    name.starts_with(STAGING_PREFIX) || name.starts_with(RECOVERY_PREFIX)
                })
            })
    }
}

#[cfg(any(unix, test))]
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateStateDocumentV1 {
    document_kind: String,
    schema_version: u64,
    revision: u64,
    selected_stream: UpdateStreamPreference,
    last_accepted_generation: u64,
    last_known_good: Option<UpdateLastKnownGood>,
    active_transaction: Option<UpdateActiveTransaction>,
}

#[cfg(any(unix, test))]
impl UpdateStateDocumentV1 {
    fn from_state(state: &UpdateState, revision: u64) -> Self {
        Self {
            document_kind: UPDATE_STATE_DOCUMENT_KIND.to_string(),
            schema_version: UPDATE_STATE_SCHEMA_VERSION,
            revision,
            selected_stream: state.selected_stream,
            last_accepted_generation: state.last_accepted_generation,
            last_known_good: state.last_known_good.clone(),
            active_transaction: state.active_transaction.clone(),
        }
    }

    fn into_loaded(self) -> Result<LoadedUpdateState, ConfigError> {
        if self.document_kind != UPDATE_STATE_DOCUMENT_KIND
            || self.schema_version != UPDATE_STATE_SCHEMA_VERSION
            || self.revision == 0
            || self.revision > MAX_REVISION
        {
            return Err(ConfigError::InvalidDocument);
        }
        let state = UpdateState {
            selected_stream: self.selected_stream,
            last_accepted_generation: self.last_accepted_generation,
            last_known_good: self.last_known_good,
            active_transaction: self.active_transaction,
        };
        state.validate()?;
        Ok(LoadedUpdateState {
            revision: self.revision,
            state,
        })
    }
}

#[cfg(unix)]
struct UpdateStoreLoad {
    value: LoadedUpdateState,
    bytes: Option<Vec<u8>>,
}

#[cfg(unix)]
impl UpdateStoreLoad {
    fn missing(default_stream: UpdateStreamPreference) -> Self {
        Self {
            value: LoadedUpdateState {
                revision: 0,
                state: UpdateState::initial(default_stream),
            },
            bytes: None,
        }
    }
}

#[cfg(any(unix, test))]
fn encode_update_state(state: &UpdateState, revision: u64) -> Result<Vec<u8>, ConfigError> {
    if revision == 0 || revision > MAX_REVISION {
        return Err(ConfigError::InvalidDocument);
    }
    state.validate()?;
    let mut bytes = serde_json::to_vec_pretty(&UpdateStateDocumentV1::from_state(state, revision))
        .map_err(|_| ConfigError::InvalidDocument)?;
    bytes.push(b'\n');
    if decode_update_state(&bytes)?
        != (LoadedUpdateState {
            revision,
            state: state.clone(),
        })
    {
        return Err(ConfigError::InvalidDocument);
    }
    Ok(bytes)
}

#[cfg(any(unix, test))]
fn decode_update_state(bytes: &[u8]) -> Result<LoadedUpdateState, ConfigError> {
    if bytes.len() > MAX_UPDATE_STATE_BYTES {
        return Err(ConfigError::DocumentTooLarge);
    }
    let value = parse_unique_json(bytes)?;
    let object = value.as_object().ok_or(ConfigError::InvalidDocument)?;
    if object
        .get("document_kind")
        .and_then(serde_json::Value::as_str)
        != Some(UPDATE_STATE_DOCUMENT_KIND)
    {
        return Err(ConfigError::InvalidDocumentKind);
    }
    match object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(UPDATE_STATE_SCHEMA_VERSION) => {}
        observed => return Err(ConfigError::UnsupportedSchema { observed }),
    }
    serde_json::from_value::<UpdateStateDocumentV1>(value)
        .map_err(|_| ConfigError::InvalidDocument)?
        .into_loaded()
}

#[cfg(unix)]
fn write_private_transaction_file(
    root: &Path,
    prefix: &str,
    bytes: &[u8],
    stage: PersistenceStage,
) -> Result<PathBuf, ConfigError> {
    use std::os::unix::fs::OpenOptionsExt;

    for _ in 0..128 {
        let path = root.join(format!("{prefix}{}", transaction_token()));
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
                    stage,
                    kind: error.kind(),
                });
            }
        };
        let result = file
            .write_all(bytes)
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .map_err(|error| ConfigError::PersistenceFailed {
                stage,
                kind: error.kind(),
            });
        drop(file);
        match result {
            Ok(()) => return Ok(path),
            Err(error) => {
                remove_if_present(&path);
                return Err(error);
            }
        }
    }
    Err(ConfigError::PersistenceFailed {
        stage,
        kind: io::ErrorKind::AlreadyExists,
    })
}

#[cfg(unix)]
fn read_bounded_private_file(path: &Path, expected: &Metadata) -> Result<Vec<u8>, ConfigError> {
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
    validate_private_file(&opened)?;
    if !same_file_identity(expected, &opened) {
        return Err(ConfigError::UnsafeManagedPath);
    }
    let mut bytes = Vec::new();
    file.take((MAX_UPDATE_STATE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::ReadCurrent,
            kind: error.kind(),
        })?;
    if bytes.len() > MAX_UPDATE_STATE_BYTES {
        return Err(ConfigError::DocumentTooLarge);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn verify_live_document(path: &Path, bytes: &[u8], revision: u64) -> Result<(), ConfigError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ConfigError::RecoveryRequired)?;
    validate_private_file(&metadata)?;
    let observed = read_bounded_private_file(path, &metadata)?;
    let loaded = decode_update_state(&observed)?;
    if observed != bytes || loaded.revision != revision {
        return Err(ConfigError::RecoveryRequired);
    }
    Ok(())
}

#[cfg(unix)]
fn verify_restored_document(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ConfigError::RecoveryRequired)?;
    validate_private_file(&metadata)?;
    let observed = read_bounded_private_file(path, &metadata)?;
    if observed != bytes || decode_update_state(&observed).is_err() {
        return Err(ConfigError::RecoveryRequired);
    }
    Ok(())
}

#[cfg(unix)]
fn validate_existing_directory_chain(path: &Path) -> Result<bool, ConfigError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => current.push(component.as_os_str()),
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
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
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

#[cfg(unix)]
fn validate_private_directory(path: &Path, metadata: &Metadata) -> Result<(), ConfigError> {
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ConfigError::UnsafeManagedPath);
    }
    validate_owner_only(metadata)?;
    let _ = path;
    Ok(())
}

#[cfg(unix)]
fn validate_private_file(metadata: &Metadata) -> Result<(), ConfigError> {
    use std::os::unix::fs::MetadataExt;

    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.nlink() != 1 {
        return Err(ConfigError::UnsafeManagedPath);
    }
    validate_owner_only(metadata)
}

#[cfg(unix)]
fn validate_owner_only(metadata: &Metadata) -> Result<(), ConfigError> {
    use std::os::unix::fs::MetadataExt;

    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
        return Err(ConfigError::InsecurePermissions);
    }
    Ok(())
}

#[cfg(unix)]
fn same_file_identity(first: &Metadata, second: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    first.dev() == second.dev() && first.ino() == second.ino()
}

#[cfg(unix)]
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

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ConfigError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| ConfigError::PersistenceFailed {
            stage: PersistenceStage::SyncDirectory,
            kind: error.kind(),
        })
}

#[cfg(unix)]
fn transaction_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = NEXT_TRANSACTION.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{counter}", std::process::id())
}

#[cfg(unix)]
fn remove_if_present(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => {}
    }
}

#[cfg(any(unix, test))]
fn valid_product_version(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 || !value.is_ascii() {
        return false;
    }
    Version::parse(value).is_ok_and(|version| {
        version.major >= 2
            && version.build.is_empty()
            && (version.pre.is_empty()
                || valid_numbered_prerelease(version.pre.as_str(), "alpha")
                || valid_numbered_prerelease(version.pre.as_str(), "beta"))
    })
}

#[cfg(any(unix, test))]
fn channel_matches_version(channel: UpdateReleaseChannel, value: &str) -> bool {
    let Ok(version) = Version::parse(value) else {
        return false;
    };
    let pre = version.pre.as_str();
    match channel {
        UpdateReleaseChannel::Alpha => valid_numbered_prerelease(pre, "alpha"),
        UpdateReleaseChannel::Beta => valid_numbered_prerelease(pre, "beta"),
        UpdateReleaseChannel::Stable => pre.is_empty(),
    }
}

#[cfg(any(unix, test))]
fn valid_numbered_prerelease(value: &str, label: &str) -> bool {
    let Some(sequence) = value
        .strip_prefix(label)
        .and_then(|value| value.strip_prefix('.'))
    else {
        return false;
    };
    !sequence.is_empty()
        && !sequence.starts_with('0')
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.parse::<u64>().is_ok_and(|number| number > 0)
}

#[cfg(any(unix, test))]
fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(any(unix, test))]
fn valid_transaction_id(value: &str) -> bool {
    (16..=96).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_document_round_trips_and_rejects_legacy_or_unknown_state() {
        let mut state = UpdateState::initial(UpdateStreamPreference::Beta);
        state.last_accepted_generation = 8;
        state.last_known_good = Some(UpdateLastKnownGood {
            version: "2.0.0-alpha.1".to_string(),
            channel: UpdateReleaseChannel::Alpha,
            generation: 8,
            archive_sha256: "1".repeat(64),
            resource_pack_sha256: "2".repeat(64),
        });
        let bytes = encode_update_state(&state, 3).unwrap();
        assert_eq!(
            decode_update_state(&bytes).unwrap(),
            LoadedUpdateState { revision: 3, state }
        );

        let legacy = String::from_utf8(bytes.clone())
            .unwrap()
            .replace("2.0.0-alpha.1", "1.19.0-beta.1");
        assert_eq!(
            decode_update_state(legacy.as_bytes()),
            Err(ConfigError::InvalidDocument)
        );
        let unknown = String::from_utf8(bytes)
            .unwrap()
            .replace("\"revision\": 3,", "\"revision\": 3,\n  \"unknown\": true,");
        assert_eq!(
            decode_update_state(unknown.as_bytes()),
            Err(ConfigError::InvalidDocument)
        );
        let duplicate = String::from_utf8(
            encode_update_state(&UpdateState::initial(UpdateStreamPreference::Beta), 1).unwrap(),
        )
        .unwrap()
        .replace(
            "\"selected_stream\": \"beta\",",
            "\"selected_stream\": \"beta\",\n  \"selected_stream\": \"stable\",",
        );
        assert_eq!(
            decode_update_state(duplicate.as_bytes()),
            Err(ConfigError::InvalidDocument)
        );
    }

    #[cfg(unix)]
    #[test]
    fn update_store_is_revision_safe_private_and_independent_from_global_settings() {
        use std::os::unix::fs::MetadataExt;

        let root = test_root("store");
        let compatibility = root.compatibility_root().to_path_buf();
        let store = UpdateStateStore::new(root, UpdateStreamPreference::Beta);
        assert_eq!(
            store.load().unwrap(),
            LoadedUpdateState {
                revision: 0,
                state: UpdateState::initial(UpdateStreamPreference::Beta),
            }
        );
        assert!(!compatibility.exists());

        let next = UpdateState::initial(UpdateStreamPreference::Stable);
        let committed = store.replace(0, next.clone()).unwrap();
        assert_eq!(committed.revision, 1);
        assert!(!committed.cleanup_required);
        assert_eq!(
            store.load().unwrap(),
            LoadedUpdateState {
                revision: 1,
                state: next,
            }
        );
        let state_root = compatibility.join("v2");
        let metadata = fs::metadata(state_root.join(UPDATE_STATE_FILE)).unwrap();
        assert_eq!(metadata.mode() & 0o777, 0o600);
        assert!(!state_root.join(crate::GLOBAL_SETTINGS_FILE).exists());
        assert_eq!(
            store.replace(0, UpdateState::initial(UpdateStreamPreference::Beta)),
            Err(ConfigError::RevisionConflict { observed: 1 })
        );
        let _ = fs::remove_dir_all(compatibility);
    }

    #[cfg(unix)]
    #[test]
    fn concurrent_first_writes_share_the_lock_and_one_revision_wins() {
        use std::sync::{Arc, Barrier};

        let root = test_root("concurrent-first-write");
        let compatibility = root.compatibility_root().to_path_buf();
        let store = UpdateStateStore::new(root, UpdateStreamPreference::Beta);
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let store = store.clone();
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                store.replace(0, UpdateState::initial(UpdateStreamPreference::Stable))
            }));
        }

        let results: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(
                    result,
                    Err(ConfigError::RevisionConflict { observed: 1 })
                ))
                .count(),
            1
        );
        assert_eq!(store.load().unwrap().revision, 1);
        let _ = fs::remove_dir_all(compatibility);
    }

    #[cfg(unix)]
    fn test_root(name: &str) -> ConfigRoot {
        let compatibility = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-update-state-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&compatibility);
        crate::resolve_config_root(Some(compatibility.as_os_str()), Path::new("/tmp")).unwrap()
    }
}
