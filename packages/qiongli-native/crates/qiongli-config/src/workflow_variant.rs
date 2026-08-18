use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use qiongli_content::{
    JCS_MAX_SAFE_INTEGER, LoadedResourcePack, WorkflowOverrideError, WorkflowOverrides,
    workflow_resource_is_editable,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::store::{
    create_private_directory, create_private_new_file, metadata_if_exists, read_bounded_file,
    sync_directory, transaction_token, validate_existing_directory_chain,
    validate_managed_directory, validate_managed_file,
};
use crate::{ConfigError, ConfigRoot, GlobalSettingsStore, PersistenceStage};

pub const WORKFLOW_VARIANT_DIRECTORY: &str = "workflow-variant";
pub const WORKFLOW_VARIANT_RECEIPT_FILE: &str = ".qiongli-workflow-variant.json";
const WORKFLOW_VARIANT_DOCUMENT_KIND: &str = "qiongli-workflow-variant";
const WORKFLOW_VARIANT_SCHEMA_VERSION: u32 = 1;
const STAGING_PREFIX: &str = ".workflow-variant-stage-";
const BACKUP_PREFIX: &str = ".workflow-variant-backup-";

#[derive(Debug)]
pub enum WorkflowVariantError {
    Config(ConfigError),
    Override(WorkflowOverrideError),
    InvalidReceipt,
    ParentChanged,
    RevisionConflict { observed: u64 },
    VariantConflict,
    ResourceConflict,
    ResourceNotOverridden,
    Unchanged,
    RecoveryRequired,
}

impl WorkflowVariantError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::Config(error) => error.reason_code(),
            Self::Override(_) => "workflow-variant-content-invalid",
            Self::InvalidReceipt => "workflow-variant-receipt-invalid",
            Self::ParentChanged => "workflow-variant-parent-changed",
            Self::RevisionConflict { .. } => "workflow-variant-revision-conflict",
            Self::VariantConflict => "workflow-variant-digest-conflict",
            Self::ResourceConflict => "workflow-variant-resource-conflict",
            Self::ResourceNotOverridden => "workflow-variant-resource-not-overridden",
            Self::Unchanged => "workflow-variant-unchanged",
            Self::RecoveryRequired => "workflow-variant-recovery-required",
        }
    }
}

impl Display for WorkflowVariantError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())?;
        if let Self::RevisionConflict { observed } = self {
            write!(formatter, " (observed revision {observed})")?;
        }
        Ok(())
    }
}

impl Error for WorkflowVariantError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            Self::Override(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ConfigError> for WorkflowVariantError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<WorkflowOverrideError> for WorkflowVariantError {
    fn from(error: WorkflowOverrideError) -> Self {
        Self::Override(error)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkflowVariantReceiptEntryV1 {
    path: String,
    base_sha256: String,
    current_sha256: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkflowVariantReceiptV1 {
    document_kind: String,
    schema_version: u32,
    pack_id: String,
    content_version: String,
    source_commit: String,
    pack_sha256: String,
    content_root_sha256: String,
    revision: u64,
    variant_sha256: Option<String>,
    total_size_bytes: u64,
    entries: Vec<WorkflowVariantReceiptEntryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedWorkflowVariant {
    revision: u64,
    overrides: Option<WorkflowOverrides>,
    cleanup_required: bool,
}

impl LoadedWorkflowVariant {
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn overrides(&self) -> Option<&WorkflowOverrides> {
        self.overrides.as_ref()
    }

    #[must_use]
    pub fn variant_sha256(&self) -> Option<&str> {
        self.overrides
            .as_ref()
            .map(WorkflowOverrides::variant_sha256)
    }

    #[must_use]
    pub const fn cleanup_required(&self) -> bool {
        self.cleanup_required
    }

    fn missing() -> Self {
        Self {
            revision: 0,
            overrides: None,
            cleanup_required: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowVariantCommit {
    pub revision: u64,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowVariantPreview {
    next_revision: u64,
    next_variant_sha256: Option<String>,
    next_resource_sha256: String,
    plan_digest_sha256: String,
}

impl WorkflowVariantPreview {
    #[must_use]
    pub const fn next_revision(&self) -> u64 {
        self.next_revision
    }

    #[must_use]
    pub fn next_variant_sha256(&self) -> Option<&str> {
        self.next_variant_sha256.as_deref()
    }

    #[must_use]
    pub fn next_resource_sha256(&self) -> &str {
        &self.next_resource_sha256
    }

    #[must_use]
    pub fn plan_digest_sha256(&self) -> &str {
        &self.plan_digest_sha256
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowVariantStore {
    root: ConfigRoot,
}

impl WorkflowVariantStore {
    #[must_use]
    pub const fn new(root: ConfigRoot) -> Self {
        Self { root }
    }

    pub fn load(
        &self,
        pack: &LoadedResourcePack<'_>,
    ) -> Result<LoadedWorkflowVariant, WorkflowVariantError> {
        if !validate_existing_directory_chain(self.root.compatibility_root())? {
            return Ok(LoadedWorkflowVariant::missing());
        }
        let Some(metadata) = metadata_if_exists(self.root.state_root())? else {
            return Ok(LoadedWorkflowVariant::missing());
        };
        validate_managed_directory(self.root.state_root(), &metadata)?;
        let live = self.live_path();
        let cleanup_required = self.has_transaction_artifact();
        let Some(metadata) = metadata_if_exists(&live)? else {
            return if cleanup_required {
                Err(WorkflowVariantError::RecoveryRequired)
            } else {
                Ok(LoadedWorkflowVariant::missing())
            };
        };
        validate_managed_directory(&live, &metadata)?;
        let mut loaded = verify_variant_tree(&live, pack)?;
        loaded.cleanup_required = cleanup_required;
        Ok(loaded)
    }

    pub fn replace_resource(
        &self,
        pack: &LoadedResourcePack<'_>,
        expected_revision: u64,
        expected_variant_sha256: Option<&str>,
        expected_current_sha256: &str,
        path: &str,
        content: Vec<u8>,
    ) -> Result<WorkflowVariantCommit, WorkflowVariantError> {
        self.commit_resource_change(
            pack,
            expected_revision,
            expected_variant_sha256,
            expected_current_sha256,
            path,
            Some(content),
        )
    }

    pub fn preview_replace_resource(
        &self,
        pack: &LoadedResourcePack<'_>,
        expected_revision: u64,
        expected_variant_sha256: Option<&str>,
        expected_current_sha256: &str,
        path: &str,
        content: &[u8],
    ) -> Result<WorkflowVariantPreview, WorkflowVariantError> {
        self.prepare_resource_change(
            pack,
            expected_revision,
            expected_variant_sha256,
            expected_current_sha256,
            path,
            Some(content),
        )
        .map(|(_, _, preview)| preview)
    }

    pub fn reset_resource(
        &self,
        pack: &LoadedResourcePack<'_>,
        expected_revision: u64,
        expected_variant_sha256: Option<&str>,
        expected_current_sha256: &str,
        path: &str,
    ) -> Result<WorkflowVariantCommit, WorkflowVariantError> {
        self.commit_resource_change(
            pack,
            expected_revision,
            expected_variant_sha256,
            expected_current_sha256,
            path,
            None,
        )
    }

    pub fn preview_reset_resource(
        &self,
        pack: &LoadedResourcePack<'_>,
        expected_revision: u64,
        expected_variant_sha256: Option<&str>,
        expected_current_sha256: &str,
        path: &str,
    ) -> Result<WorkflowVariantPreview, WorkflowVariantError> {
        self.prepare_resource_change(
            pack,
            expected_revision,
            expected_variant_sha256,
            expected_current_sha256,
            path,
            None,
        )
        .map(|(_, _, preview)| preview)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the compare-and-swap boundary binds every observed identity explicitly"
    )]
    fn commit_resource_change(
        &self,
        pack: &LoadedResourcePack<'_>,
        expected_revision: u64,
        expected_variant_sha256: Option<&str>,
        expected_current_sha256: &str,
        path: &str,
        replacement: Option<Vec<u8>>,
    ) -> Result<WorkflowVariantCommit, WorkflowVariantError> {
        GlobalSettingsStore::new(self.root.clone()).prepare_store()?;
        let settings_store = GlobalSettingsStore::new(self.root.clone());
        let _lock = settings_store.acquire_lock()?;
        if self.has_transaction_artifact() {
            return Err(WorkflowVariantError::RecoveryRequired);
        }
        let (current, overrides, _) = self.prepare_resource_change(
            pack,
            expected_revision,
            expected_variant_sha256,
            expected_current_sha256,
            path,
            replacement.as_deref(),
        )?;
        let revision = current
            .revision
            .checked_add(1)
            .filter(|revision| *revision <= JCS_MAX_SAFE_INTEGER)
            .ok_or(ConfigError::RevisionExhausted)?;
        self.commit(pack, &current, revision, overrides)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the compare-and-swap preview binds every observed identity explicitly"
    )]
    fn prepare_resource_change(
        &self,
        pack: &LoadedResourcePack<'_>,
        expected_revision: u64,
        expected_variant_sha256: Option<&str>,
        expected_current_sha256: &str,
        path: &str,
        replacement: Option<&[u8]>,
    ) -> Result<
        (
            LoadedWorkflowVariant,
            Option<WorkflowOverrides>,
            WorkflowVariantPreview,
        ),
        WorkflowVariantError,
    > {
        let current = self.load(pack)?;
        if current.revision != expected_revision {
            return Err(WorkflowVariantError::RevisionConflict {
                observed: current.revision,
            });
        }
        if current.variant_sha256() != expected_variant_sha256 {
            return Err(WorkflowVariantError::VariantConflict);
        }
        let canonical = pack
            .resource_for_profile("full", path)
            .map_err(WorkflowOverrideError::Profile)?
            .filter(|resource| workflow_resource_is_editable(resource.entry().resource_kind, path))
            .ok_or_else(|| {
                WorkflowVariantError::Override(WorkflowOverrideError::PathNotAllowed(
                    path.to_owned(),
                ))
            })?;
        let current_sha256 = current
            .overrides()
            .and_then(|overrides| overrides.entry(path))
            .map_or(canonical.entry().sha256.as_str(), |entry| {
                entry.current_sha256()
            });
        if current_sha256 != expected_current_sha256 {
            return Err(WorkflowVariantError::ResourceConflict);
        }

        let mut contents = current
            .overrides()
            .into_iter()
            .flat_map(WorkflowOverrides::entries)
            .map(|entry| (entry.path().to_owned(), entry.bytes().to_vec()))
            .collect::<BTreeMap<_, _>>();
        match replacement {
            Some(content) => {
                contents.insert(path.to_owned(), content.to_vec());
            }
            None if contents.remove(path).is_none() => {
                return Err(WorkflowVariantError::ResourceNotOverridden);
            }
            None => {}
        }
        let overrides = WorkflowOverrides::new(pack, contents)?;
        if overrides.as_ref() == current.overrides() {
            return Err(WorkflowVariantError::Unchanged);
        }
        let next_revision = current
            .revision
            .checked_add(1)
            .filter(|revision| *revision <= JCS_MAX_SAFE_INTEGER)
            .ok_or(ConfigError::RevisionExhausted)?;
        let next_resource_sha256 = overrides
            .as_ref()
            .and_then(|value| value.entry(path))
            .map_or_else(
                || canonical.entry().sha256.clone(),
                |entry| entry.current_sha256().to_owned(),
            );
        let next_variant_sha256 = overrides
            .as_ref()
            .map(|value| value.variant_sha256().to_owned());
        let plan_digest_sha256 = workflow_variant_plan_digest(
            pack.pack_sha256(),
            expected_revision,
            next_revision,
            expected_variant_sha256,
            next_variant_sha256.as_deref(),
            path,
            expected_current_sha256,
            &next_resource_sha256,
            replacement.is_none(),
        );
        Ok((
            current,
            overrides,
            WorkflowVariantPreview {
                next_revision,
                next_variant_sha256,
                next_resource_sha256,
                plan_digest_sha256,
            },
        ))
    }

    fn commit(
        &self,
        pack: &LoadedResourcePack<'_>,
        current: &LoadedWorkflowVariant,
        revision: u64,
        overrides: Option<WorkflowOverrides>,
    ) -> Result<WorkflowVariantCommit, WorkflowVariantError> {
        let receipt = build_receipt(pack, revision, overrides.as_ref());
        let staging = self.unique_transaction_path(STAGING_PREFIX)?;
        create_private_directory(&staging)?;
        if let Err(error) = write_variant_tree(&staging, &receipt, overrides.as_ref())
            .and_then(|()| verify_variant_tree(&staging, pack).map(|_| ()))
        {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }

        let live = self.live_path();
        let backup = if live.exists() {
            Some(self.unique_transaction_path(BACKUP_PREFIX)?)
        } else {
            None
        };
        if let Some(backup) = backup.as_ref() {
            fs::rename(&live, backup)
                .map_err(|error| persistence(PersistenceStage::Activate, error))?;
        }
        if let Err(error) = fs::rename(&staging, &live) {
            if let Some(backup) = backup.as_ref() {
                let _ = fs::rename(backup, &live);
            }
            return Err(persistence(PersistenceStage::Activate, error));
        }
        if sync_directory(self.root.state_root()).is_err()
            || verify_variant_tree(&live, pack).is_err()
        {
            let recovery = self.unique_transaction_path(STAGING_PREFIX)?;
            let _ = fs::rename(&live, &recovery);
            if let Some(backup) = backup.as_ref() {
                let _ = fs::rename(backup, &live);
            }
            let _ = sync_directory(self.root.state_root());
            return Err(WorkflowVariantError::RecoveryRequired);
        }

        let cleanup_required = if let Some(backup) = backup.as_ref() {
            verify_variant_tree(backup, pack)
                .ok()
                .filter(|loaded| loaded == current)
                .is_none()
                || fs::remove_dir_all(backup).is_err()
                || sync_directory(self.root.state_root()).is_err()
        } else {
            false
        };
        Ok(WorkflowVariantCommit {
            revision,
            cleanup_required,
        })
    }

    fn live_path(&self) -> PathBuf {
        self.root.state_root().join(WORKFLOW_VARIANT_DIRECTORY)
    }

    fn has_transaction_artifact(&self) -> bool {
        fs::read_dir(self.root.state_root())
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .any(|entry| {
                entry.file_name().to_str().is_some_and(|name| {
                    name.starts_with(STAGING_PREFIX) || name.starts_with(BACKUP_PREFIX)
                })
            })
    }

    fn unique_transaction_path(&self, prefix: &str) -> Result<PathBuf, WorkflowVariantError> {
        for _ in 0..128 {
            let path = self
                .root
                .state_root()
                .join(format!("{prefix}{}", transaction_token()));
            if metadata_if_exists(&path)?.is_none() {
                return Ok(path);
            }
        }
        Err(WorkflowVariantError::RecoveryRequired)
    }
}

fn build_receipt(
    pack: &LoadedResourcePack<'_>,
    revision: u64,
    overrides: Option<&WorkflowOverrides>,
) -> WorkflowVariantReceiptV1 {
    let entries = overrides
        .into_iter()
        .flat_map(WorkflowOverrides::entries)
        .map(|entry| WorkflowVariantReceiptEntryV1 {
            path: entry.path().to_owned(),
            base_sha256: entry.base_sha256().to_owned(),
            current_sha256: entry.current_sha256().to_owned(),
            size_bytes: entry.size_bytes(),
        })
        .collect::<Vec<_>>();
    WorkflowVariantReceiptV1 {
        document_kind: WORKFLOW_VARIANT_DOCUMENT_KIND.to_owned(),
        schema_version: WORKFLOW_VARIANT_SCHEMA_VERSION,
        pack_id: pack.manifest().pack_id.clone(),
        content_version: pack.manifest().content_version.clone(),
        source_commit: pack.manifest().source_commit.clone(),
        pack_sha256: pack.pack_sha256().to_owned(),
        content_root_sha256: pack.manifest().content_root_sha256.clone(),
        revision,
        variant_sha256: overrides.map(|value| value.variant_sha256().to_owned()),
        total_size_bytes: entries.iter().map(|entry| entry.size_bytes).sum(),
        entries,
    }
}

fn write_variant_tree(
    root: &Path,
    receipt: &WorkflowVariantReceiptV1,
    overrides: Option<&WorkflowOverrides>,
) -> Result<(), WorkflowVariantError> {
    let mut directories = vec![root.to_path_buf()];
    for entry in overrides.into_iter().flat_map(WorkflowOverrides::entries) {
        let destination = root.join(entry.path());
        let mut current = root.to_path_buf();
        for component in Path::new(entry.path()).components() {
            let Component::Normal(component) = component else {
                return Err(WorkflowVariantError::InvalidReceipt);
            };
            current.push(component);
            if current == destination {
                break;
            }
            if metadata_if_exists(&current)?.is_none() {
                create_private_directory(&current)?;
                directories.push(current.clone());
            }
        }
        write_private_file(&destination, entry.bytes())?;
    }
    let receipt_bytes = serde_json_canonicalizer::to_vec(receipt)
        .map_err(|_| WorkflowVariantError::InvalidReceipt)?;
    write_private_file(&root.join(WORKFLOW_VARIANT_RECEIPT_FILE), &receipt_bytes)?;
    for directory in directories.iter().rev() {
        sync_directory(directory)?;
    }
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), WorkflowVariantError> {
    let mut file = create_private_new_file(path, PersistenceStage::WriteStaging)?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(|error| persistence(PersistenceStage::SyncStaging, error))
}

fn verify_variant_tree(
    root: &Path,
    pack: &LoadedResourcePack<'_>,
) -> Result<LoadedWorkflowVariant, WorkflowVariantError> {
    let root_metadata = metadata_if_exists(root)?.ok_or(WorkflowVariantError::InvalidReceipt)?;
    validate_managed_directory(root, &root_metadata)?;
    let receipt_path = root.join(WORKFLOW_VARIANT_RECEIPT_FILE);
    let receipt_metadata =
        metadata_if_exists(&receipt_path)?.ok_or(WorkflowVariantError::InvalidReceipt)?;
    validate_managed_file(&receipt_path, &receipt_metadata)?;
    let receipt_bytes = read_bounded_file(&receipt_path, &receipt_metadata)?;
    let receipt: WorkflowVariantReceiptV1 =
        serde_json::from_slice(&receipt_bytes).map_err(|_| WorkflowVariantError::InvalidReceipt)?;
    if serde_json_canonicalizer::to_vec(&receipt)
        .map_err(|_| WorkflowVariantError::InvalidReceipt)?
        != receipt_bytes
    {
        return Err(WorkflowVariantError::InvalidReceipt);
    }
    validate_receipt(&receipt, pack)?;

    let expected_files = receipt
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .chain(std::iter::once(WORKFLOW_VARIANT_RECEIPT_FILE))
        .collect::<BTreeSet<_>>();
    let expected_directories = expected_directories(&receipt.entries);
    let mut seen_files = BTreeSet::new();
    let mut seen_directories = BTreeSet::new();
    collect_tree(
        root,
        root,
        &expected_files,
        &expected_directories,
        &mut seen_files,
        &mut seen_directories,
    )?;
    if seen_files != expected_files || seen_directories != expected_directories {
        return Err(WorkflowVariantError::InvalidReceipt);
    }

    let mut contents = BTreeMap::new();
    for entry in &receipt.entries {
        let path = root.join(&entry.path);
        let metadata = metadata_if_exists(&path)?.ok_or(WorkflowVariantError::InvalidReceipt)?;
        validate_managed_file(&path, &metadata)?;
        let bytes = read_bounded_file(&path, &metadata)?;
        contents.insert(entry.path.clone(), bytes);
    }
    let overrides = WorkflowOverrides::new(pack, contents)?;
    let actual_entries = build_receipt(pack, receipt.revision, overrides.as_ref()).entries;
    if actual_entries != receipt.entries
        || overrides.as_ref().map(WorkflowOverrides::variant_sha256)
            != receipt.variant_sha256.as_deref()
    {
        return Err(WorkflowVariantError::InvalidReceipt);
    }
    Ok(LoadedWorkflowVariant {
        revision: receipt.revision,
        overrides,
        cleanup_required: false,
    })
}

fn validate_receipt(
    receipt: &WorkflowVariantReceiptV1,
    pack: &LoadedResourcePack<'_>,
) -> Result<(), WorkflowVariantError> {
    if receipt.document_kind != WORKFLOW_VARIANT_DOCUMENT_KIND
        || receipt.schema_version != WORKFLOW_VARIANT_SCHEMA_VERSION
        || receipt.revision == 0
        || receipt.revision > JCS_MAX_SAFE_INTEGER
    {
        return Err(WorkflowVariantError::InvalidReceipt);
    }
    if receipt.pack_id != pack.manifest().pack_id
        || receipt.content_version != pack.manifest().content_version
        || receipt.source_commit != pack.manifest().source_commit
        || receipt.pack_sha256 != pack.pack_sha256()
        || receipt.content_root_sha256 != pack.manifest().content_root_sha256
    {
        return Err(WorkflowVariantError::ParentChanged);
    }
    if receipt.entries.is_empty() != receipt.variant_sha256.is_none()
        || receipt
            .variant_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || receipt.total_size_bytes
            != receipt
                .entries
                .iter()
                .map(|entry| entry.size_bytes)
                .sum::<u64>()
    {
        return Err(WorkflowVariantError::InvalidReceipt);
    }
    let mut previous: Option<&str> = None;
    for entry in &receipt.entries {
        if previous.is_some_and(|path| path >= entry.path.as_str())
            || !valid_sha256(&entry.base_sha256)
            || !valid_sha256(&entry.current_sha256)
        {
            return Err(WorkflowVariantError::InvalidReceipt);
        }
        previous = Some(&entry.path);
    }
    Ok(())
}

fn collect_tree<'a>(
    root: &Path,
    directory: &Path,
    expected_files: &BTreeSet<&'a str>,
    expected_directories: &BTreeSet<String>,
    seen_files: &mut BTreeSet<&'a str>,
    seen_directories: &mut BTreeSet<String>,
) -> Result<(), WorkflowVariantError> {
    for item in
        fs::read_dir(directory).map_err(|error| persistence(PersistenceStage::Inspect, error))?
    {
        let item = item.map_err(|error| persistence(PersistenceStage::Inspect, error))?;
        let path = item.path();
        let relative = path
            .strip_prefix(root)
            .ok()
            .and_then(Path::to_str)
            .map(|value| value.replace(std::path::MAIN_SEPARATOR, "/"))
            .ok_or(WorkflowVariantError::InvalidReceipt)?;
        let metadata = metadata_if_exists(&path)?.ok_or(WorkflowVariantError::InvalidReceipt)?;
        if metadata.is_dir() {
            validate_managed_directory(&path, &metadata)?;
            if !expected_directories.contains(&relative) {
                return Err(WorkflowVariantError::InvalidReceipt);
            }
            seen_directories.insert(relative);
            collect_tree(
                root,
                &path,
                expected_files,
                expected_directories,
                seen_files,
                seen_directories,
            )?;
        } else {
            validate_managed_file(&path, &metadata)?;
            let expected = expected_files
                .get(relative.as_str())
                .copied()
                .ok_or(WorkflowVariantError::InvalidReceipt)?;
            seen_files.insert(expected);
        }
    }
    Ok(())
}

fn expected_directories(entries: &[WorkflowVariantReceiptEntryV1]) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();
    for entry in entries {
        let mut current = String::new();
        let mut parts = entry.path.split('/').collect::<Vec<_>>();
        parts.pop();
        for part in parts {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(part);
            expected.insert(current.clone());
        }
    }
    expected
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[allow(
    clippy::too_many_arguments,
    reason = "the digest explicitly binds the complete compare-and-swap preview"
)]
fn workflow_variant_plan_digest(
    pack_sha256: &str,
    current_revision: u64,
    next_revision: u64,
    current_variant_sha256: Option<&str>,
    next_variant_sha256: Option<&str>,
    path: &str,
    current_resource_sha256: &str,
    next_resource_sha256: &str,
    reset: bool,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qiongli-workflow-variant-plan-v1\0");
    for value in [
        pack_sha256,
        current_variant_sha256.unwrap_or("canonical"),
        next_variant_sha256.unwrap_or("canonical"),
        path,
        current_resource_sha256,
        next_resource_sha256,
        if reset { "reset" } else { "replace" },
    ] {
        hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(current_revision.to_be_bytes());
    hasher.update(next_revision.to_be_bytes());
    format!("{:x}", hasher.finalize())
}

fn persistence(stage: PersistenceStage, error: std::io::Error) -> WorkflowVariantError {
    WorkflowVariantError::Config(ConfigError::PersistenceFailed {
        stage,
        kind: error.kind(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_content::{
        CompatibleProduct, ResourcePackBuildMetadata, build_resource_pack,
        collect_canonical_sources, load_resource_pack,
    };

    use super::*;

    const DIRECTORY_ROOTS: [&str; 12] = [
        ".claude-plugin",
        ".codex-plugin",
        "distribution",
        "mcp-contracts",
        "roles",
        "schemas",
        "skills",
        "standards",
        "subjects",
        "templates",
        "venue-profiles",
        "workflow",
    ];
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        built: qiongli_content::BuiltResourcePack,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let requested = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-config-tests")
                .join(format!(
                    "workflow-variant-{}-{nonce}-{id}",
                    std::process::id()
                ));
            fs::create_dir_all(&requested).unwrap();
            let root = fs::canonicalize(&requested).unwrap();
            let source = root.join("source");
            fs::create_dir(&source).unwrap();
            for directory in DIRECTORY_ROOTS {
                fs::create_dir_all(source.join(directory)).unwrap();
            }
            for (path, content) in [
                ("workflow/SKILL.md", "# Canonical workflow\n"),
                ("skills/method.md", "# Canonical method\n"),
                ("skills-core.md", "core\n"),
                ("skills-summary.md", "summary\n"),
                ("schemas/example.json", "{}\n"),
                ("distribution/plugins.yaml", "plugins: []\n"),
                ("mcp-contracts/tools.json", "{}\n"),
            ] {
                fs::write(source.join(path), content).unwrap();
            }
            let resources = collect_canonical_sources(&source).unwrap();
            let built = build_resource_pack(
                &ResourcePackBuildMetadata {
                    pack_id: "qiongli-core".to_owned(),
                    content_version: "2.0.0-alpha.3".to_owned(),
                    source_commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
                    compatible_product: CompatibleProduct {
                        minimum: "2.0.0-alpha.1".to_owned(),
                        maximum_exclusive: "3.0.0".to_owned(),
                    },
                },
                &resources,
            )
            .unwrap();
            Self { root, built }
        }

        fn pack(&self) -> qiongli_content::LoadedResourcePack<'_> {
            load_resource_pack(self.built.core_bytes(), self.built.pack_sha256()).unwrap()
        }

        fn store(&self, name: &str) -> WorkflowVariantStore {
            let compatibility = self.root.join(name);
            let root =
                crate::resolve_config_root(Some(compatibility.as_os_str()), &self.root).unwrap();
            WorkflowVariantStore::new(root)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn replace_and_reset_are_revision_safe_and_preserve_unrelated_state() {
        let fixture = Fixture::new();
        let pack = fixture.pack();
        let first = fixture.store("first-config");
        let canonical = pack
            .resource_for_profile("full", "workflow/SKILL.md")
            .unwrap()
            .unwrap();
        let preview = first
            .preview_replace_resource(
                &pack,
                0,
                None,
                &canonical.entry().sha256,
                "workflow/SKILL.md",
                b"# Customized workflow\n",
            )
            .unwrap();
        assert_eq!(preview.next_revision(), 1);
        assert!(preview.next_variant_sha256().is_some());
        assert_ne!(preview.next_resource_sha256(), canonical.entry().sha256);
        assert!(valid_sha256(preview.plan_digest_sha256()));

        let committed = first
            .replace_resource(
                &pack,
                0,
                None,
                &canonical.entry().sha256,
                "workflow/SKILL.md",
                b"# Customized workflow\n".to_vec(),
            )
            .unwrap();
        assert_eq!(committed.revision, 1);
        let loaded = first.load(&pack).unwrap();
        let variant = loaded.variant_sha256().unwrap().to_owned();
        let current = loaded
            .overrides()
            .unwrap()
            .entry("workflow/SKILL.md")
            .unwrap();
        let reset_preview = first
            .preview_reset_resource(
                &pack,
                1,
                Some(&variant),
                current.current_sha256(),
                "workflow/SKILL.md",
            )
            .unwrap();
        assert_eq!(reset_preview.next_revision(), 2);
        assert_eq!(reset_preview.next_variant_sha256(), None);
        assert_eq!(
            reset_preview.next_resource_sha256(),
            canonical.entry().sha256
        );

        let second = fixture.store("second-config");
        second
            .replace_resource(
                &pack,
                0,
                None,
                &canonical.entry().sha256,
                "workflow/SKILL.md",
                b"# Customized workflow\n".to_vec(),
            )
            .unwrap();
        assert_eq!(
            second.load(&pack).unwrap().variant_sha256(),
            Some(variant.as_str())
        );

        assert!(matches!(
            first.replace_resource(
                &pack,
                0,
                None,
                &canonical.entry().sha256,
                "workflow/SKILL.md",
                b"stale\n".to_vec(),
            ),
            Err(WorkflowVariantError::RevisionConflict { observed: 1 })
        ));
        let unrelated = first.root.state_root().join("keep.txt");
        fs::write(&unrelated, b"keep").unwrap();
        first
            .reset_resource(
                &pack,
                1,
                Some(&variant),
                current.current_sha256(),
                "workflow/SKILL.md",
            )
            .unwrap();
        let reset = first.load(&pack).unwrap();
        assert_eq!(reset.revision(), 2);
        assert!(reset.overrides().is_none());
        assert_eq!(fs::read(unrelated).unwrap(), b"keep");
    }

    #[test]
    fn invalid_or_drifted_overrides_fail_closed() {
        let fixture = Fixture::new();
        let pack = fixture.pack();
        let store = fixture.store("config");
        let canonical = pack
            .resource_for_profile("full", "workflow/SKILL.md")
            .unwrap()
            .unwrap();

        assert!(matches!(
            store.replace_resource(
                &pack,
                0,
                None,
                &canonical.entry().sha256,
                "distribution/plugins.yaml",
                b"plugins: changed\n".to_vec(),
            ),
            Err(WorkflowVariantError::Override(
                WorkflowOverrideError::PathNotAllowed(_)
            ))
        ));
        assert!(
            store
                .replace_resource(
                    &pack,
                    0,
                    None,
                    &canonical.entry().sha256,
                    "workflow/SKILL.md",
                    vec![b'x'; qiongli_content::MAX_WORKFLOW_OVERRIDE_BYTES + 1],
                )
                .is_err()
        );
        assert!(
            store
                .replace_resource(
                    &pack,
                    0,
                    None,
                    &canonical.entry().sha256,
                    "workflow/SKILL.md",
                    b"invalid\r\n".to_vec(),
                )
                .is_err()
        );

        store
            .replace_resource(
                &pack,
                0,
                None,
                &canonical.entry().sha256,
                "workflow/SKILL.md",
                b"# Customized workflow\n".to_vec(),
            )
            .unwrap();
        fs::write(store.live_path().join("workflow/SKILL.md"), b"tampered\n").unwrap();
        assert!(store.load(&pack).is_err());
    }
}
