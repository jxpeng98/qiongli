#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(any(unix, windows))]
use std::fs::TryLockError;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use qiongli_config::ConfigRoot;
use same_file::Handle;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::ProjectError;
use crate::json::parse_unique_json;
use crate::model::{
    ArticleProjectManifestV1, MissingContinuityArtifact, ProjectOverviewV1,
    ResearchLibraryDocumentV1, valid_overview_text,
};

pub(crate) const PROJECT_MANIFEST_RELATIVE_PATH: [&str; 2] = ["context", "project_manifest.json"];
const RESEARCH_LIBRARY_DIR: &str = "research-library";
const RESEARCH_LIBRARY_FILE: &str = "library.json";
const LIBRARY_LOCK_FILE: &str = ".library.lock";
const MAX_LIBRARY_BYTES: usize = 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_ARTIFACT_BYTES: usize = 4 * 1024 * 1024;
const MAX_SEMANTIC_BYTES: usize = 16 * 1024 * 1024;
const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const LOCK_RETRY: Duration = Duration::from_millis(10);

const SEMANTIC_ARTIFACTS: [&str; 8] = [
    "context/research_state.md",
    "context/decision_log.md",
    "context/stage_handoff.md",
    "context/boundary_review.md",
    "context/idea_funnel.md",
    "literature/literature_map.md",
    "evidence/claim-evidence-ledger.csv",
    "manuscript/claims_evidence_map.md",
];

#[derive(Clone)]
pub(crate) struct LibraryStore {
    config_root: ConfigRoot,
}

pub(crate) struct LibraryMutation {
    store: LibraryStore,
    pub(crate) document: ResearchLibraryDocumentV1,
    _lock: File,
}

impl LibraryStore {
    pub(crate) const fn new(config_root: ConfigRoot) -> Self {
        Self { config_root }
    }

    fn root(&self) -> PathBuf {
        self.config_root.state_root().join(RESEARCH_LIBRARY_DIR)
    }

    pub(crate) fn load(&self) -> Result<ResearchLibraryDocumentV1, ProjectError> {
        let root = self.root();
        let Some(metadata) = metadata_if_exists(&root)? else {
            return Ok(ResearchLibraryDocumentV1::empty());
        };
        validate_private_directory(&root, &metadata)?;
        let path = root.join(RESEARCH_LIBRARY_FILE);
        let Some(metadata) = metadata_if_exists(&path)? else {
            return Ok(ResearchLibraryDocumentV1::empty());
        };
        let bytes = read_bounded_file(&path, &metadata, MAX_LIBRARY_BYTES, true)?;
        decode_document(&bytes, true)
    }

    pub(crate) fn begin(&self, expected_revision: u64) -> Result<LibraryMutation, ProjectError> {
        self.prepare()?;
        let root = self.root();
        let lock = acquire_lock(&root.join(LIBRARY_LOCK_FILE))?;
        let document = self.load()?;
        if document.revision != expected_revision {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(LibraryMutation {
            store: self.clone(),
            document,
            _lock: lock,
        })
    }

    fn prepare(&self) -> Result<(), ProjectError> {
        ensure_directory_tree(self.config_root.compatibility_root())?;
        ensure_private_directory(self.config_root.state_root())?;
        ensure_private_directory(&self.root())
    }
}

impl LibraryMutation {
    pub(crate) fn commit(mut self) -> Result<u64, ProjectError> {
        self.document.revision = self
            .document
            .revision
            .checked_add(1)
            .ok_or(ProjectError::RevisionConflict)?;
        self.document
            .projects
            .sort_by(|left, right| left.project_id.cmp(&right.project_id));
        self.document.validate()?;
        let bytes = encode_document(&self.document, true)?;
        atomic_write(&self.store.root(), RESEARCH_LIBRARY_FILE, &bytes, true)?;
        Ok(self.document.revision)
    }
}

pub(crate) fn validate_existing_project_root(root: &Path) -> Result<(), ProjectError> {
    validate_project_path_shape(root)?;
    let metadata = metadata_if_exists(root)?.ok_or(ProjectError::ProjectRootMissing)?;
    validate_project_directory(root, &metadata)?;
    let canonical = dunce::canonicalize(root).map_err(map_io)?;
    if canonical != root {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    Ok(())
}

pub(crate) fn validate_create_project_root(root: &Path) -> Result<(), ProjectError> {
    validate_project_path_shape(root)?;
    if metadata_if_exists(root)?.is_some() {
        return Err(ProjectError::ProjectRootConflict);
    }
    let parent = root.parent().ok_or(ProjectError::InvalidProjectRoot)?;
    let metadata = metadata_if_exists(parent)?.ok_or(ProjectError::ProjectRootMissing)?;
    validate_project_directory(parent, &metadata)?;
    let canonical = dunce::canonicalize(parent).map_err(map_io)?;
    if canonical != parent {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    Ok(())
}

pub(crate) fn create_project_root(root: &Path) -> Result<(), ProjectError> {
    validate_create_project_root(root)?;
    create_private_directory(root)?;
    let metadata = metadata_if_exists(root)?.ok_or(ProjectError::RecoveryRequired)?;
    validate_project_directory(root, &metadata)
}

pub(crate) fn project_root_string(root: &Path) -> Result<String, ProjectError> {
    validate_project_path_shape(root)?;
    root.to_str()
        .filter(|value| value.len() <= 4096 && !value.chars().any(char::is_control))
        .map(str::to_owned)
        .ok_or(ProjectError::InvalidProjectRoot)
}

pub(crate) fn project_root_from_string(value: &str) -> Result<PathBuf, ProjectError> {
    if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        return Err(ProjectError::InvalidLibraryDocument);
    }
    let root = PathBuf::from(value);
    validate_project_path_shape(&root).map_err(|_| ProjectError::InvalidLibraryDocument)?;
    Ok(root)
}

pub(crate) fn project_root_label(root: &Path) -> String {
    root.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| {
            !value.is_empty() && value.len() <= 160 && !value.chars().any(char::is_control)
        })
        .unwrap_or("Article project")
        .to_string()
}

pub(crate) fn read_manifest(
    root: &Path,
) -> Result<Option<(ArticleProjectManifestV1, String)>, ProjectError> {
    validate_existing_project_root(root)?;
    let path = manifest_path(root);
    let Some(metadata) = metadata_if_exists(&path)? else {
        return Ok(None);
    };
    let bytes = read_bounded_file(&path, &metadata, MAX_MANIFEST_BYTES, false)?;
    let manifest: ArticleProjectManifestV1 = decode_document(&bytes, false)?;
    manifest.validate()?;
    Ok(Some((manifest, sha256(&bytes))))
}

pub(crate) fn write_manifest(
    root: &Path,
    manifest: &ArticleProjectManifestV1,
    expected_document_digest: Option<&str>,
) -> Result<(), ProjectError> {
    validate_existing_project_root(root)?;
    let context = root.join("context");
    ensure_project_directory(&context)?;
    let path = manifest_path(root);
    match (metadata_if_exists(&path)?, expected_document_digest) {
        (None, None) => {}
        (Some(metadata), Some(expected)) => {
            let bytes = read_bounded_file(&path, &metadata, MAX_MANIFEST_BYTES, false)?;
            if sha256(&bytes) != expected {
                return Err(ProjectError::RevisionConflict);
            }
        }
        _ => return Err(ProjectError::RevisionConflict),
    }
    manifest.validate()?;
    let bytes = encode_document(manifest, false)?;
    atomic_write(&context, "project_manifest.json", &bytes, false)
}

pub(crate) fn semantic_digest(root: &Path) -> Result<String, ProjectError> {
    validate_existing_project_root(root)?;
    semantic_digest_from_root(Some(root))
}

pub(crate) fn empty_semantic_digest() -> String {
    semantic_digest_from_root(None).expect("the fixed empty semantic digest cannot fail")
}

fn semantic_digest_from_root(root: Option<&Path>) -> Result<String, ProjectError> {
    let mut digest = Sha256::new();
    let mut total = 0usize;
    for relative in SEMANTIC_ARTIFACTS {
        digest.update(relative.as_bytes());
        digest.update([0]);
        let metadata = root
            .map(|root| metadata_if_exists(&root.join(relative)))
            .transpose()?
            .flatten();
        match metadata {
            None => digest.update(b"missing"),
            Some(metadata) => {
                let path = root
                    .expect("artifact metadata exists only when a root was supplied")
                    .join(relative);
                let bytes = read_bounded_file(&path, &metadata, MAX_ARTIFACT_BYTES, false)?;
                total = total
                    .checked_add(bytes.len())
                    .filter(|value| *value <= MAX_SEMANTIC_BYTES)
                    .ok_or(ProjectError::DocumentTooLarge)?;
                digest.update((bytes.len() as u64).to_be_bytes());
                digest.update(&bytes);
            }
        }
        digest.update([0xff]);
    }
    Ok(lower_hex(&digest.finalize()))
}

pub(crate) fn missing_continuity(
    root: &Path,
) -> Result<Vec<MissingContinuityArtifact>, ProjectError> {
    validate_existing_project_root(root)?;
    let candidates = [
        (
            "context/research_state.md",
            MissingContinuityArtifact::ResearchState,
        ),
        (
            "context/decision_log.md",
            MissingContinuityArtifact::DecisionLog,
        ),
        (
            "context/stage_handoff.md",
            MissingContinuityArtifact::StageHandoff,
        ),
        (
            "literature/literature_map.md",
            MissingContinuityArtifact::LiteratureMap,
        ),
        (
            "evidence/claim-evidence-ledger.csv",
            MissingContinuityArtifact::ClaimEvidenceLedger,
        ),
        (
            "manuscript/claims_evidence_map.md",
            MissingContinuityArtifact::ManuscriptClaimMap,
        ),
    ];
    let mut missing = Vec::new();
    for (relative, kind) in candidates {
        if metadata_if_exists(&root.join(relative))?.is_none() {
            missing.push(kind);
        }
    }
    Ok(missing)
}

pub(crate) fn read_overview(root: &Path) -> Result<ProjectOverviewV1, ProjectError> {
    validate_existing_project_root(root)?;
    let mut overview = ProjectOverviewV1::empty();
    if let Some(bytes) = read_optional_artifact(&root.join("context/research_state.md"))?
        && let Ok(text) = std::str::from_utf8(&bytes)
    {
        overview.focal_question = first_prefixed(text, &["RQ:", "Research question:"]);
        overview.thesis = first_prefixed(text, &["Thesis:", "Contribution:"]);
        overview.evidence_position = first_prefixed(text, &["Evidence position:"]);
        overview.next_priorities = text
            .lines()
            .filter_map(|line| prefixed_value(line, &["Next:", "Priority:"]))
            .take(8)
            .collect();
        overview.unresolved_risk_count = text
            .lines()
            .filter(|line| {
                let line = line.to_ascii_lowercase();
                (line.contains("risk") || line.contains("contradiction"))
                    && (line.contains("open") || line.contains("unresolved"))
            })
            .count()
            .try_into()
            .unwrap_or(u32::MAX);
    }
    if let Some(bytes) = read_optional_artifact(&root.join("evidence/claim-evidence-ledger.csv"))?
        && let Ok(text) = std::str::from_utf8(&bytes)
    {
        let rows = text
            .lines()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        if !rows.is_empty() {
            let supported = rows
                .iter()
                .filter(|line| {
                    let value = line.to_ascii_lowercase();
                    value.contains("support") || value.contains("verified")
                })
                .count();
            overview.claim_evidence_coverage_percent =
                Some(((supported.saturating_mul(100)) / rows.len()).min(100) as u8);
        }
    }
    overview.validate()?;
    Ok(overview)
}

fn read_optional_artifact(path: &Path) -> Result<Option<Vec<u8>>, ProjectError> {
    let Some(metadata) = metadata_if_exists(path)? else {
        return Ok(None);
    };
    read_bounded_file(path, &metadata, MAX_ARTIFACT_BYTES, false).map(Some)
}

fn first_prefixed(text: &str, prefixes: &[&str]) -> Option<String> {
    text.lines().find_map(|line| prefixed_value(line, prefixes))
}

fn prefixed_value(line: &str, prefixes: &[&str]) -> Option<String> {
    let line = line.trim().trim_start_matches(['-', '*']).trim();
    let value = prefixes
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))?
        .trim();
    valid_overview_text(value).then(|| value.to_string())
}

fn manifest_path(root: &Path) -> PathBuf {
    PROJECT_MANIFEST_RELATIVE_PATH
        .iter()
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

fn validate_project_path_shape(path: &Path) -> Result<(), ProjectError> {
    if !path.is_absolute()
        || path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(ProjectError::InvalidProjectRoot);
    }
    Ok(())
}

fn ensure_directory_tree(path: &Path) -> Result<(), ProjectError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if matches!(component, Component::Prefix(_) | Component::RootDir) {
            continue;
        }
        match metadata_if_exists(&current)? {
            Some(metadata) => validate_directory_component(&current, &metadata)?,
            None => {
                create_private_directory(&current)?;
                let metadata =
                    metadata_if_exists(&current)?.ok_or(ProjectError::RecoveryRequired)?;
                validate_directory_component(&current, &metadata)?;
            }
        }
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), ProjectError> {
    match metadata_if_exists(path)? {
        Some(metadata) => validate_private_directory(path, &metadata),
        None => {
            create_private_directory(path)?;
            let metadata = metadata_if_exists(path)?.ok_or(ProjectError::RecoveryRequired)?;
            validate_private_directory(path, &metadata)
        }
    }
}

fn ensure_project_directory(path: &Path) -> Result<(), ProjectError> {
    match metadata_if_exists(path)? {
        Some(metadata) => validate_project_directory(path, &metadata),
        None => {
            create_private_directory(path)?;
            let metadata = metadata_if_exists(path)?.ok_or(ProjectError::RecoveryRequired)?;
            validate_project_directory(path, &metadata)
        }
    }
}

fn validate_directory_component(path: &Path, metadata: &Metadata) -> Result<(), ProjectError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_dir() {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    #[cfg(windows)]
    qiongli_windows_security::open_directory_no_reparse(path)
        .map(|_| ())
        .map_err(|_| ProjectError::UnsafeProjectRoot)?;
    #[cfg(not(windows))]
    let _ = path;
    Ok(())
}

fn validate_project_directory(path: &Path, metadata: &Metadata) -> Result<(), ProjectError> {
    validate_directory_component(path, metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o022 != 0
        {
            return Err(ProjectError::UnsafeProjectRoot);
        }
    }
    #[cfg(windows)]
    qiongli_windows_security::open_directory_no_reparse(path)
        .map(|_| ())
        .map_err(|_| ProjectError::UnsafeProjectRoot)?;
    Ok(())
}

fn validate_private_directory(path: &Path, metadata: &Metadata) -> Result<(), ProjectError> {
    validate_directory_component(path, metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err(ProjectError::UnsafeProjectRoot);
        }
    }
    #[cfg(windows)]
    qiongli_windows_security::open_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| ProjectError::UnsafeProjectRoot)?;
    Ok(())
}

fn acquire_lock(path: &Path) -> Result<File, ProjectError> {
    if let Some(metadata) = metadata_if_exists(path)? {
        validate_file(path, &metadata, true)?;
    }
    let file = open_or_create_private_lock(path)?;
    validate_opened_file(path, &file, true)?;
    let started = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) if started.elapsed() < LOCK_TIMEOUT => {
                std::thread::sleep(LOCK_RETRY);
            }
            Err(TryLockError::WouldBlock) => return Err(ProjectError::LockBusy),
            Err(TryLockError::Error(error)) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(TryLockError::Error(error)) => {
                return Err(ProjectError::PersistenceFailed(error.kind()));
            }
        }
    }
}

fn atomic_write(
    directory: &Path,
    file_name: &str,
    bytes: &[u8],
    private_existing: bool,
) -> Result<(), ProjectError> {
    let destination = directory.join(file_name);
    if let Some(metadata) = metadata_if_exists(&destination)? {
        validate_file(&destination, &metadata, private_existing)?;
    }
    let mut token = [0u8; 12];
    getrandom::fill(&mut token).map_err(|_| ProjectError::RandomUnavailable)?;
    let staging = directory.join(format!(".{file_name}.qiongli-stage-{}", lower_hex(&token)));
    let mut file = create_private_new_file(&staging)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&staging);
        return Err(ProjectError::PersistenceFailed(error.kind()));
    }
    drop(file);
    if let Err(error) = replace_file(&staging, &destination, true) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    sync_directory(directory)?;
    let metadata = metadata_if_exists(&destination)?.ok_or(ProjectError::RecoveryRequired)?;
    let committed = read_bounded_file(&destination, &metadata, bytes.len(), true)?;
    if committed != bytes {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(())
}

fn encode_document<T: Serialize>(value: &T, library: bool) -> Result<Vec<u8>, ProjectError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| {
        if library {
            ProjectError::InvalidLibraryDocument
        } else {
            ProjectError::InvalidProjectDocument
        }
    })
}

fn decode_document<T: DeserializeOwned>(bytes: &[u8], library: bool) -> Result<T, ProjectError> {
    let value = parse_unique_json(bytes).map_err(|_| {
        if library {
            ProjectError::InvalidLibraryDocument
        } else {
            ProjectError::InvalidProjectDocument
        }
    })?;
    serde_json::from_value(value).map_err(|_| {
        if library {
            ProjectError::InvalidLibraryDocument
        } else {
            ProjectError::InvalidProjectDocument
        }
    })
}

fn metadata_if_exists(path: &Path) -> Result<Option<Metadata>, ProjectError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ProjectError::PersistenceFailed(error.kind())),
    }
}

fn validate_file(path: &Path, metadata: &Metadata, private: bool) -> Result<(), ProjectError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_file() {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let forbidden = if private { 0o077 } else { 0o022 };
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & forbidden != 0
            || metadata.nlink() != 1
        {
            return Err(ProjectError::UnsafeProjectRoot);
        }
    }
    #[cfg(windows)]
    if private {
        qiongli_windows_security::open_owner_only_file(path)
            .map(|_| ())
            .map_err(|_| ProjectError::UnsafeProjectRoot)?;
    }
    #[cfg(not(windows))]
    let _ = path;
    Ok(())
}

fn read_bounded_file(
    path: &Path,
    expected: &Metadata,
    max: usize,
    private: bool,
) -> Result<Vec<u8>, ProjectError> {
    validate_file(path, expected, private)?;
    if expected.len() > max as u64 {
        return Err(ProjectError::DocumentTooLarge);
    }
    let file = File::open(path).map_err(map_io)?;
    validate_opened_file(path, &file, private)?;
    let opened = file.metadata().map_err(map_io)?;
    if opened.len() != expected.len() {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    let mut bytes = Vec::new();
    file.take((max + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(map_io)?;
    if bytes.len() > max {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok(bytes)
}

fn validate_opened_file(path: &Path, file: &File, private: bool) -> Result<(), ProjectError> {
    let metadata = file.metadata().map_err(map_io)?;
    validate_file(path, &metadata, private)?;
    let before = Handle::from_path(path).map_err(|_| ProjectError::UnsafeProjectRoot)?;
    let cloned = file.try_clone().map_err(map_io)?;
    let after = Handle::from_file(cloned).map_err(|_| ProjectError::UnsafeProjectRoot)?;
    if before != after {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    Ok(())
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
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
}

#[cfg(unix)]
fn create_private_new_file(path: &Path) -> Result<File, ProjectError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(map_io)
}

#[cfg(windows)]
fn create_private_new_file(path: &Path) -> Result<File, ProjectError> {
    qiongli_windows_security::create_owner_only_new_file(path).map_err(|error| {
        ProjectError::PersistenceFailed(error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied))
    })
}

#[cfg(not(any(unix, windows)))]
fn create_private_new_file(_path: &Path) -> Result<File, ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
}

#[cfg(unix)]
fn open_or_create_private_lock(path: &Path) -> Result<File, ProjectError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(map_io)
}

#[cfg(windows)]
fn open_or_create_private_lock(path: &Path) -> Result<File, ProjectError> {
    qiongli_windows_security::open_or_create_owner_only_lock(path).map_err(|error| {
        ProjectError::PersistenceFailed(error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied))
    })
}

#[cfg(not(any(unix, windows)))]
fn open_or_create_private_lock(_path: &Path) -> Result<File, ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
}

#[cfg(unix)]
fn replace_file(source: &Path, destination: &Path, _replace: bool) -> Result<(), ProjectError> {
    fs::rename(source, destination).map_err(map_io)
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path, replace: bool) -> Result<(), ProjectError> {
    qiongli_windows_security::move_file_write_through(source, destination, replace).map_err(
        |error| {
            ProjectError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        },
    )
}

#[cfg(not(any(unix, windows)))]
fn replace_file(_source: &Path, _destination: &Path, _replace: bool) -> Result<(), ProjectError> {
    Err(ProjectError::UnsupportedPlatformSecurity)
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

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

fn map_io(error: io::Error) -> ProjectError {
    ProjectError::PersistenceFailed(error.kind())
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
