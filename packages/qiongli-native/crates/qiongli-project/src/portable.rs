use std::fmt::{self, Debug, Formatter};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File, Metadata};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use same_file::Handle;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ProjectError;
use crate::model::{
    ArticleProjectManifestV1, ProjectId, ProjectKind, ProjectLifecycle, ProjectStage,
    valid_lower_hex,
};
use crate::storage::{
    project_root_label, read_manifest, semantic_digest, validate_create_project_root,
    validate_existing_project_root,
};

pub const PORTABLE_PROJECT_SCHEMA_VERSION: u32 = 1;
pub const PORTABLE_PROJECT_DOCUMENT_KIND: &str = "qiongli-portable-project";
const PORTABLE_PROJECT_FILE: &str = "qiongli-portable-project.json";
const PORTABLE_CONTENT_DIR: &str = "project";
const MAX_PORTABLE_FILES: usize = 1_024;
const MAX_PORTABLE_FILE_BYTES: usize = 32 * 1024 * 1024;
const MAX_PORTABLE_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PORTABLE_METADATA_BYTES: usize = 2 * 1024 * 1024;
const MAX_PORTABLE_PATH_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortableProjectOperation {
    Export,
    Import,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableProjectPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub operation: PortableProjectOperation,
    pub project_id: ProjectId,
    pub display_name: String,
    pub source_label: String,
    pub destination_label: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub excluded_entry_count: usize,
    pub expected_library_revision: u64,
    pub approvals_required: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedPortableProjectOperation {
    preview: PortableProjectPreviewV1,
    source: PathBuf,
    destination: PathBuf,
    source_reference_digest: String,
    destination_reference_digest: String,
    package: PortableProjectPackageV1,
}

impl VerifiedPortableProjectOperation {
    #[must_use]
    pub const fn preview(&self) -> &PortableProjectPreviewV1 {
        &self.preview
    }

    pub(crate) const fn package(&self) -> &PortableProjectPackageV1 {
        &self.package
    }

    pub(crate) fn source(&self) -> &Path {
        &self.source
    }

    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }
}

impl Debug for VerifiedPortableProjectOperation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedPortableProjectOperation")
            .field("preview", &self.preview)
            .field("source", &"<portable-project-source>")
            .field("destination", &"<portable-project-destination>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableProjectCommitV1 {
    pub schema_version: u32,
    pub operation: PortableProjectOperation,
    pub project_id: ProjectId,
    pub library_revision: Option<u64>,
    pub files_copied: usize,
    pub total_bytes: u64,
    pub destination_label: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortableProjectPackageV1 {
    schema_version: u32,
    document_kind: String,
    pub(crate) project_id: ProjectId,
    display_name: String,
    project_kind: ProjectKind,
    stage: ProjectStage,
    lifecycle: ProjectLifecycle,
    semantic_revision: u64,
    semantic_digest: String,
    inventory: Vec<PortableProjectEntryV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortableProjectEntryV1 {
    pub(crate) relative_path: String,
    pub(crate) size_bytes: u64,
    pub(crate) sha256: String,
}

#[derive(Serialize)]
struct PortablePlanSemantics<'a> {
    schema_version: u32,
    operation: PortableProjectOperation,
    project_id: &'a ProjectId,
    package_digest: String,
    source_reference_digest: String,
    destination_reference_digest: String,
    expected_library_revision: u64,
}

pub(crate) fn preview_export(
    source: &Path,
    destination: &Path,
    expected_library_revision: u64,
) -> Result<VerifiedPortableProjectOperation, ProjectError> {
    validate_existing_project_root(source)?;
    validate_create_project_root(destination)?;
    reject_nested_destination(source, destination)?;
    let (manifest, _) = read_manifest(source)?.ok_or(ProjectError::ProjectManifestMissing)?;
    if semantic_digest(source)? != manifest.semantic_digest {
        return Err(ProjectError::RevisionConflict);
    }
    let (inventory, excluded_entry_count) = inventory(source)?;
    let package = package_from_manifest(&manifest, inventory);
    build_plan(
        PortableProjectOperation::Export,
        source,
        destination,
        package,
        excluded_entry_count,
        expected_library_revision,
    )
}

pub(crate) fn preview_import(
    source: &Path,
    destination: &Path,
    expected_library_revision: u64,
) -> Result<VerifiedPortableProjectOperation, ProjectError> {
    validate_existing_project_root(source)?;
    validate_create_project_root(destination)?;
    reject_nested_destination(source, destination)?;
    let package = read_package(source)?;
    validate_package(&package)?;
    let content_root = source.join(PORTABLE_CONTENT_DIR);
    validate_existing_project_root(&content_root)?;
    let (inventory, excluded) = inventory(&content_root)?;
    if excluded != 0 || inventory != package.inventory {
        return Err(ProjectError::PortablePackageInvalid);
    }
    let (manifest, _) =
        read_manifest(&content_root)?.ok_or(ProjectError::PortablePackageInvalid)?;
    if !package_matches_manifest(&package, &manifest)
        || semantic_digest(&content_root)? != manifest.semantic_digest
    {
        return Err(ProjectError::PortablePackageInvalid);
    }
    build_plan(
        PortableProjectOperation::Import,
        source,
        destination,
        package,
        0,
        expected_library_revision,
    )
}

pub(crate) fn apply_files(plan: &VerifiedPortableProjectOperation) -> Result<(), ProjectError> {
    validate_plan_paths(plan)?;
    validate_create_project_root(&plan.destination)?;
    let current = match plan.preview.operation {
        PortableProjectOperation::Export => {
            validate_existing_project_root(&plan.source)?;
            let (manifest, _) =
                read_manifest(&plan.source)?.ok_or(ProjectError::ProjectManifestMissing)?;
            let (inventory, excluded) = inventory(&plan.source)?;
            let package = package_from_manifest(&manifest, inventory);
            if excluded != plan.preview.excluded_entry_count {
                return Err(ProjectError::RevisionConflict);
            }
            package
        }
        PortableProjectOperation::Import => read_package(&plan.source)?,
    };
    if current != plan.package {
        return Err(ProjectError::RevisionConflict);
    }
    let content_source = match plan.preview.operation {
        PortableProjectOperation::Export => plan.source.as_path(),
        PortableProjectOperation::Import => {
            let root = plan.source.join(PORTABLE_CONTENT_DIR);
            validate_existing_project_root(&root)?;
            let (inventory, excluded) = inventory(&root)?;
            if excluded != 0 || inventory != plan.package.inventory {
                return Err(ProjectError::RevisionConflict);
            }
            return copy_import_content(plan, &root);
        }
    };
    copy_export_package(plan, content_source)
}

fn copy_export_package(
    plan: &VerifiedPortableProjectOperation,
    content_source: &Path,
) -> Result<(), ProjectError> {
    let staging = create_staging_directory(&plan.destination)?;
    let result = (|| {
        let content_destination = staging.join(PORTABLE_CONTENT_DIR);
        create_private_directory(&content_destination)?;
        copy_inventory(
            content_source,
            &content_destination,
            &plan.package.inventory,
        )?;
        write_package(&staging, &plan.package)?;
        commit_staging(&staging, &plan.destination)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn copy_import_content(
    plan: &VerifiedPortableProjectOperation,
    content_source: &Path,
) -> Result<(), ProjectError> {
    let staging = create_staging_directory(&plan.destination)?;
    let result = (|| {
        copy_inventory(content_source, &staging, &plan.package.inventory)?;
        commit_staging(&staging, &plan.destination)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn build_plan(
    operation: PortableProjectOperation,
    source: &Path,
    destination: &Path,
    package: PortableProjectPackageV1,
    excluded_entry_count: usize,
    expected_library_revision: u64,
) -> Result<VerifiedPortableProjectOperation, ProjectError> {
    validate_package(&package)?;
    let source_reference_digest = path_digest(source)?;
    let destination_reference_digest = path_digest(destination)?;
    let package_digest = canonical_digest(&package)?;
    let semantics = PortablePlanSemantics {
        schema_version: PORTABLE_PROJECT_SCHEMA_VERSION,
        operation,
        project_id: &package.project_id,
        package_digest,
        source_reference_digest: source_reference_digest.clone(),
        destination_reference_digest: destination_reference_digest.clone(),
        expected_library_revision,
    };
    let total_bytes = package.inventory.iter().map(|entry| entry.size_bytes).sum();
    let preview = PortableProjectPreviewV1 {
        schema_version: PORTABLE_PROJECT_SCHEMA_VERSION,
        plan_digest: canonical_digest(&semantics)?,
        operation,
        project_id: package.project_id.clone(),
        display_name: package.display_name.clone(),
        source_label: project_root_label(source),
        destination_label: project_root_label(destination),
        file_count: package.inventory.len(),
        total_bytes,
        excluded_entry_count,
        expected_library_revision,
        approvals_required: vec!["filesystem-write".to_string()],
    };
    Ok(VerifiedPortableProjectOperation {
        preview,
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        source_reference_digest,
        destination_reference_digest,
        package,
    })
}

fn inventory(root: &Path) -> Result<(Vec<PortableProjectEntryV1>, usize), ProjectError> {
    let (entries, excluded) = inventory_entries(root)?;
    if !entries
        .iter()
        .any(|entry| entry.relative_path == "context/project_manifest.json")
    {
        return Err(ProjectError::PortablePackageInvalid);
    }
    Ok((entries, excluded))
}

pub(crate) fn migration_inventory(
    root: &Path,
) -> Result<(Vec<PortableProjectEntryV1>, usize), ProjectError> {
    validate_existing_project_root(root)?;
    inventory_entries(root)
}

fn inventory_entries(root: &Path) -> Result<(Vec<PortableProjectEntryV1>, usize), ProjectError> {
    let mut entries = Vec::new();
    let mut excluded = 0usize;
    collect_inventory(root, root, &mut entries, &mut excluded)?;
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    if entries.len() > MAX_PORTABLE_FILES {
        return Err(ProjectError::PortablePackageInvalid);
    }
    let total = entries
        .iter()
        .try_fold(0_u64, |total, entry| total.checked_add(entry.size_bytes));
    if total.is_none_or(|total| total > MAX_PORTABLE_TOTAL_BYTES) {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok((entries, excluded))
}

fn collect_inventory(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<PortableProjectEntryV1>,
    excluded: &mut usize,
) -> Result<(), ProjectError> {
    let mut children = fs::read_dir(directory)
        .map_err(map_io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_io)?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children {
        let path = child.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| ProjectError::PortablePackageInvalid)?;
        if excluded_relative(relative)? {
            *excluded = excluded.saturating_add(1);
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(map_io)?;
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return Err(ProjectError::UnsafeProjectRoot);
        }
        if metadata.is_dir() {
            validate_owned_directory(&metadata)?;
            collect_inventory(root, &path, entries, excluded)?;
        } else if metadata.is_file() {
            let relative_path = portable_path(relative)?;
            let bytes = read_portable_file(&path, &metadata)?;
            entries.push(PortableProjectEntryV1 {
                relative_path,
                size_bytes: bytes.len() as u64,
                sha256: sha256(&bytes),
            });
            if entries.len() > MAX_PORTABLE_FILES {
                return Err(ProjectError::DocumentTooLarge);
            }
        } else {
            return Err(ProjectError::UnsafeProjectRoot);
        }
    }
    Ok(())
}

fn excluded_relative(relative: &Path) -> Result<bool, ProjectError> {
    let mut parts = Vec::new();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(ProjectError::PortablePackageInvalid);
        };
        let value = value
            .to_str()
            .ok_or(ProjectError::PortablePackageInvalid)?
            .to_ascii_lowercase();
        parts.push(value);
    }
    let Some(file_name) = parts.last() else {
        return Err(ProjectError::PortablePackageInvalid);
    };
    let excluded_component = parts.iter().any(|part| {
        matches!(
            part.as_str(),
            ".git"
                | ".qiongli"
                | ".codex"
                | ".claude"
                | "node_modules"
                | "target"
                | "dist"
                | "build"
                | "cache"
                | "caches"
                | "sessions"
                | "conversations"
                | "chats"
                | "transcripts"
        )
    });
    let excluded_file = file_name == PORTABLE_PROJECT_FILE
        || file_name == ".env"
        || file_name.starts_with(".env.")
        || [".pem", ".key", ".p12", ".pfx"]
            .iter()
            .any(|suffix| file_name.ends_with(suffix))
        || [
            "credential",
            "credentials",
            "secret",
            "secrets",
            "token",
            "tokens",
        ]
        .iter()
        .any(|marker| file_name.contains(marker));
    Ok(excluded_component || excluded_file)
}

fn validate_package(package: &PortableProjectPackageV1) -> Result<(), ProjectError> {
    if package.schema_version != PORTABLE_PROJECT_SCHEMA_VERSION
        || package.document_kind != PORTABLE_PROJECT_DOCUMENT_KIND
        || package.inventory.is_empty()
        || package.inventory.len() > MAX_PORTABLE_FILES
        || !valid_lower_hex(&package.semantic_digest, 64)
    {
        return Err(ProjectError::PortablePackageInvalid);
    }
    package.project_id.validate()?;
    let mut previous: Option<&str> = None;
    let mut total = 0_u64;
    for entry in &package.inventory {
        validate_portable_path(&entry.relative_path)?;
        if previous.is_some_and(|value| value >= entry.relative_path.as_str())
            || entry.size_bytes > MAX_PORTABLE_FILE_BYTES as u64
            || !valid_lower_hex(&entry.sha256, 64)
        {
            return Err(ProjectError::PortablePackageInvalid);
        }
        total = total
            .checked_add(entry.size_bytes)
            .filter(|total| *total <= MAX_PORTABLE_TOTAL_BYTES)
            .ok_or(ProjectError::DocumentTooLarge)?;
        previous = Some(&entry.relative_path);
    }
    Ok(())
}

fn package_from_manifest(
    manifest: &ArticleProjectManifestV1,
    inventory: Vec<PortableProjectEntryV1>,
) -> PortableProjectPackageV1 {
    PortableProjectPackageV1 {
        schema_version: PORTABLE_PROJECT_SCHEMA_VERSION,
        document_kind: PORTABLE_PROJECT_DOCUMENT_KIND.to_string(),
        project_id: manifest.project_id.clone(),
        display_name: manifest.display_name.clone(),
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        lifecycle: manifest.lifecycle,
        semantic_revision: manifest.semantic_revision,
        semantic_digest: manifest.semantic_digest.clone(),
        inventory,
    }
}

fn package_matches_manifest(
    package: &PortableProjectPackageV1,
    manifest: &ArticleProjectManifestV1,
) -> bool {
    package.project_id == manifest.project_id
        && package.display_name == manifest.display_name
        && package.project_kind == manifest.project_kind
        && package.stage == manifest.stage
        && package.lifecycle == manifest.lifecycle
        && package.semantic_revision == manifest.semantic_revision
        && package.semantic_digest == manifest.semantic_digest
}

fn read_package(root: &Path) -> Result<PortableProjectPackageV1, ProjectError> {
    let path = root.join(PORTABLE_PROJECT_FILE);
    let metadata = fs::symlink_metadata(&path).map_err(map_io)?;
    let bytes = read_portable_file(&path, &metadata)?;
    if bytes.len() > MAX_PORTABLE_METADATA_BYTES {
        return Err(ProjectError::DocumentTooLarge);
    }
    let value =
        crate::json::parse_unique_json(&bytes).map_err(|_| ProjectError::PortablePackageInvalid)?;
    serde_json::from_value(value).map_err(|_| ProjectError::PortablePackageInvalid)
}

fn write_package(root: &Path, package: &PortableProjectPackageV1) -> Result<(), ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(package)
        .map_err(|_| ProjectError::PortablePackageInvalid)?;
    if bytes.len() > MAX_PORTABLE_METADATA_BYTES {
        return Err(ProjectError::DocumentTooLarge);
    }
    write_private_file(&root.join(PORTABLE_PROJECT_FILE), &bytes)
}

pub(crate) fn copy_inventory(
    source: &Path,
    destination: &Path,
    inventory: &[PortableProjectEntryV1],
) -> Result<(), ProjectError> {
    for entry in inventory {
        validate_portable_path(&entry.relative_path)?;
        let source_path = source.join(&entry.relative_path);
        let metadata = fs::symlink_metadata(&source_path).map_err(map_io)?;
        let bytes = read_portable_file(&source_path, &metadata)?;
        if bytes.len() as u64 != entry.size_bytes || sha256(&bytes) != entry.sha256 {
            return Err(ProjectError::RevisionConflict);
        }
        let destination_path = destination.join(&entry.relative_path);
        if let Some(parent) = destination_path.parent() {
            ensure_private_subdirectories(destination, parent)?;
        }
        write_private_file(&destination_path, &bytes)?;
    }
    Ok(())
}

pub(crate) fn ensure_private_subdirectories(
    root: &Path,
    destination: &Path,
) -> Result<(), ProjectError> {
    let relative = destination
        .strip_prefix(root)
        .map_err(|_| ProjectError::PortablePackageInvalid)?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return Err(ProjectError::PortablePackageInvalid);
        };
        current.push(value);
        if !current.exists() {
            create_private_directory(&current)?;
        }
    }
    Ok(())
}

fn validate_plan_paths(plan: &VerifiedPortableProjectOperation) -> Result<(), ProjectError> {
    if path_digest(&plan.source)? != plan.source_reference_digest
        || path_digest(&plan.destination)? != plan.destination_reference_digest
    {
        return Err(ProjectError::PlanMismatch);
    }
    Ok(())
}

fn reject_nested_destination(source: &Path, destination: &Path) -> Result<(), ProjectError> {
    if destination.starts_with(source) || source.starts_with(destination) {
        return Err(ProjectError::InvalidProjectRoot);
    }
    Ok(())
}

fn portable_path(path: &Path) -> Result<String, ProjectError> {
    let mut value = String::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(ProjectError::PortablePackageInvalid);
        };
        let component = component
            .to_str()
            .ok_or(ProjectError::PortablePackageInvalid)?;
        if component.is_empty() || component.chars().any(char::is_control) {
            return Err(ProjectError::PortablePackageInvalid);
        }
        if !value.is_empty() {
            value.push('/');
        }
        value.push_str(component);
    }
    validate_portable_path(&value)?;
    Ok(value)
}

fn validate_portable_path(value: &str) -> Result<(), ProjectError> {
    if value.is_empty()
        || value.len() > MAX_PORTABLE_PATH_BYTES
        || value.starts_with('/')
        || value.contains('\\')
        || value.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.chars().any(char::is_control)
        })
    {
        return Err(ProjectError::PortablePackageInvalid);
    }
    Ok(())
}

fn read_portable_file(path: &Path, expected: &Metadata) -> Result<Vec<u8>, ProjectError> {
    validate_owned_file(expected)?;
    if expected.len() > MAX_PORTABLE_FILE_BYTES as u64 {
        return Err(ProjectError::DocumentTooLarge);
    }
    let file = File::open(path).map_err(map_io)?;
    let opened = file.metadata().map_err(map_io)?;
    validate_owned_file(&opened)?;
    let before = Handle::from_path(path).map_err(|_| ProjectError::UnsafeProjectRoot)?;
    let after = Handle::from_file(file.try_clone().map_err(map_io)?)
        .map_err(|_| ProjectError::UnsafeProjectRoot)?;
    if before != after || opened.len() != expected.len() {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    let mut bytes = Vec::new();
    file.take((MAX_PORTABLE_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(map_io)?;
    if bytes.len() > MAX_PORTABLE_FILE_BYTES {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok(bytes)
}

fn validate_owned_directory(metadata: &Metadata) -> Result<(), ProjectError> {
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(metadata) {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o022 != 0
        {
            return Err(ProjectError::UnsafeProjectRoot);
        }
    }
    Ok(())
}

fn validate_owned_file(metadata: &Metadata) -> Result<(), ProjectError> {
    if !metadata.is_file() || metadata.file_type().is_symlink() || is_reparse_point(metadata) {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o022 != 0
            || metadata.nlink() != 1
        {
            return Err(ProjectError::UnsafeProjectRoot);
        }
    }
    Ok(())
}

pub(crate) fn create_staging_directory(destination: &Path) -> Result<PathBuf, ProjectError> {
    let parent = destination
        .parent()
        .ok_or(ProjectError::InvalidProjectRoot)?;
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ProjectError::InvalidProjectRoot)?;
    let mut token = [0_u8; 12];
    getrandom::fill(&mut token).map_err(|_| ProjectError::RandomUnavailable)?;
    let staging = parent.join(format!(".{name}.qiongli-stage-{}", lower_hex(&token)));
    create_private_directory(&staging)?;
    Ok(staging)
}

pub(crate) fn commit_staging(staging: &Path, destination: &Path) -> Result<(), ProjectError> {
    validate_create_project_root(destination)?;
    fs::rename(staging, destination).map_err(map_io)?;
    sync_directory(
        destination
            .parent()
            .ok_or(ProjectError::InvalidProjectRoot)?,
    )
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ProjectError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(map_io)
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), ProjectError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn sync_directory(_path: &Path) -> Result<(), ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), ProjectError> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path).map_err(map_io)
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), ProjectError> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|error| {
            ProjectError::PersistenceFailed(
                error
                    .io_kind()
                    .unwrap_or(std::io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
}

#[cfg(unix)]
pub(crate) fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), ProjectError> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(map_io)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(map_io)
}

#[cfg(windows)]
pub(crate) fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), ProjectError> {
    let mut file = qiongli_windows_security::create_owner_only_new_file(path).map_err(|error| {
        ProjectError::PersistenceFailed(
            error
                .io_kind()
                .unwrap_or(std::io::ErrorKind::PermissionDenied),
        )
    })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(map_io)
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn write_private_file(_path: &Path, _bytes: &[u8]) -> Result<(), ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

pub(crate) fn path_digest(path: &Path) -> Result<String, ProjectError> {
    let value = path
        .to_str()
        .filter(|value| !value.is_empty() && !value.chars().any(char::is_control))
        .ok_or(ProjectError::InvalidProjectRoot)?;
    Ok(sha256(value.as_bytes()))
}

pub(crate) fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    serde_json_canonicalizer::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|_| ProjectError::PortablePackageInvalid)
}

fn sha256(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn map_io(error: std::io::Error) -> ProjectError {
    ProjectError::PersistenceFailed(error.kind())
}
