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

use crate::json::parse_unique_json;
use crate::model::{
    ArticleProjectManifestV1, MissingContinuityArtifact, ProjectId, ProjectOverviewV1,
    ResearchLibraryDocumentV1, valid_overview_text,
};
use crate::{CaptureId, ProjectError, ResearchCaptureV1};

pub(crate) const PROJECT_MANIFEST_RELATIVE_PATH: [&str; 2] = ["context", "project_manifest.json"];
const RESEARCH_LIBRARY_DIR: &str = "research-library";
const RESEARCH_LIBRARY_FILE: &str = "library.json";
const LIBRARY_LOCK_FILE: &str = ".library.lock";
const PROJECT_RUNTIME_DIR: &str = ".qiongli";
const CAPTURE_HISTORY_LOCK_FILE: &str = ".capture-history.lock";
const CONSOLIDATION_LOCK_FILE: &str = ".consolidation.lock";
const REGISTRATION_JOURNAL_LOCK_FILE: &str = ".registration-journal.lock";
const CONSOLIDATION_TRANSACTION_DIR: &str = "consolidation-transaction";
const CAPTURE_HISTORY_DIR: [&str; 2] = ["context", "captures"];
const REPOSITORY_CAPTURE_INBOX_DIR: [&str; 2] = ["context", "capture-inbox"];
const MAX_LIBRARY_BYTES: usize = 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_ARTIFACT_BYTES: usize = 4 * 1024 * 1024;
const MAX_SEMANTIC_BYTES: usize = 16 * 1024 * 1024;
const MAX_GRAPH_SEMANTIC_LINKS_BYTES: usize = 1024 * 1024;
const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const LOCK_RETRY: Duration = Duration::from_millis(10);

pub(crate) const SEMANTIC_ARTIFACTS: [&str; 8] = [
    "context/research_state.md",
    "context/decision_log.md",
    "context/stage_handoff.md",
    "context/boundary_review.md",
    "context/idea_funnel.md",
    "literature/literature_map.md",
    "evidence/claim-evidence-ledger.csv",
    "manuscript/claims_evidence_map.md",
];

pub(crate) const GRAPH_SEMANTIC_LINKS_RELATIVE_PATH: &str = "graph/semantic_links.jsonl";

#[derive(Clone)]
pub(crate) struct LibraryStore {
    config_root: ConfigRoot,
}

pub(crate) struct LibraryMutation {
    store: LibraryStore,
    pub(crate) document: ResearchLibraryDocumentV1,
    _lock: File,
}

pub(crate) struct LibraryGuard {
    pub(crate) document: ResearchLibraryDocumentV1,
    _lock: File,
}

pub(crate) struct CaptureHistoryLock {
    _lock: File,
}

pub(crate) struct ProjectRegistrationJournalLock {
    root: PathBuf,
    _lock: File,
}

#[derive(Clone)]
pub(crate) struct ProjectFileUpdate {
    pub(crate) relative_path: String,
    pub(crate) expected_digest: Option<String>,
    pub(crate) next_bytes: Vec<u8>,
}

struct ProjectFileBackup {
    relative_path: String,
    previous_bytes: Option<Vec<u8>>,
    next_digest: String,
}

pub(crate) struct ProjectFileTransaction {
    root: PathBuf,
    transaction_dir: PathBuf,
    backups: Vec<ProjectFileBackup>,
    finalized: bool,
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

    pub(crate) fn lock(&self, expected_revision: u64) -> Result<LibraryGuard, ProjectError> {
        self.prepare()?;
        let root = self.root();
        let lock = acquire_lock(&root.join(LIBRARY_LOCK_FILE))?;
        let document = self.load()?;
        if document.revision != expected_revision {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(LibraryGuard {
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
        self.document
            .registration_tombstones
            .sort_by(|left, right| {
                left.identity_kind
                    .cmp(&right.identity_kind)
                    .then_with(|| left.identity_value.cmp(&right.identity_value))
            });
        self.document.validate()?;
        let bytes = encode_document(&self.document, true)?;
        if bytes.len() > MAX_LIBRARY_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        atomic_write(&self.store.root(), RESEARCH_LIBRARY_FILE, &bytes, true)?;
        Ok(self.document.revision)
    }
}

pub(crate) fn validate_existing_project_root(root: &Path) -> Result<(), ProjectError> {
    validate_project_path_shape(root)?;
    let metadata = metadata_if_exists(root)?.ok_or(ProjectError::ProjectRootMissing)?;
    validate_project_directory(root, &metadata)?;
    let canonical = dunce::canonicalize(root).map_err(map_io)?;
    if canonical != dunce::simplified(root) {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    let runtime = root.join(PROJECT_RUNTIME_DIR);
    if let Some(metadata) = project_metadata_if_exists(root, &runtime)? {
        validate_directory_component(&runtime, &metadata)?;
        if project_metadata_if_exists(root, &consolidation_transaction_directory(root))?.is_some() {
            return Err(ProjectError::RecoveryRequired);
        }
    }
    Ok(())
}

pub(crate) fn validate_create_project_root(root: &Path) -> Result<(), ProjectError> {
    validate_project_path_shape(root)?;
    if metadata_if_exists(root)?.is_some() {
        return Err(ProjectError::ProjectRootConflict);
    }
    let parent = root.parent().ok_or(ProjectError::InvalidProjectRoot)?;
    let existing_parent = if metadata_if_exists(parent)?.is_some() {
        parent
    } else if parent
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        parent.parent().ok_or(ProjectError::InvalidProjectRoot)?
    } else {
        return Err(ProjectError::ProjectRootMissing);
    };
    let metadata = metadata_if_exists(existing_parent)?.ok_or(ProjectError::ProjectRootMissing)?;
    validate_project_directory(existing_parent, &metadata)?;
    let canonical = dunce::canonicalize(existing_parent).map_err(map_io)?;
    if canonical != dunce::simplified(existing_parent) {
        return Err(ProjectError::UnsafeProjectRoot);
    }
    Ok(())
}

pub(crate) fn create_project_root(root: &Path) -> Result<(), ProjectError> {
    validate_create_project_root(root)?;
    let parent = root.parent().ok_or(ProjectError::InvalidProjectRoot)?;
    if metadata_if_exists(parent)?.is_none() {
        create_private_directory(parent)?;
    }
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
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    let bytes = read_bounded_project_file(root, &path, &metadata, MAX_MANIFEST_BYTES, false)?;
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
    ensure_project_directory_beneath(root, &context)?;
    let path = manifest_path(root);
    match (
        project_metadata_if_exists(root, &path)?,
        expected_document_digest,
    ) {
        (None, None) => {}
        (Some(metadata), Some(expected)) => {
            let bytes =
                read_bounded_project_file(root, &path, &metadata, MAX_MANIFEST_BYTES, false)?;
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

pub(crate) fn lock_capture_history(root: &Path) -> Result<CaptureHistoryLock, ProjectError> {
    validate_existing_project_root(root)?;
    let runtime = root.join(PROJECT_RUNTIME_DIR);
    ensure_private_directory_beneath(root, &runtime)?;
    Ok(CaptureHistoryLock {
        _lock: acquire_lock(&runtime.join(CAPTURE_HISTORY_LOCK_FILE))?,
    })
}

pub(crate) fn lock_project_registration_journal(
    root: &Path,
) -> Result<ProjectRegistrationJournalLock, ProjectError> {
    validate_existing_project_root(root)?;
    let runtime = root.join(PROJECT_RUNTIME_DIR);
    ensure_private_directory_beneath(root, &runtime)?;
    Ok(ProjectRegistrationJournalLock {
        root: root.to_path_buf(),
        _lock: acquire_lock(&runtime.join(REGISTRATION_JOURNAL_LOCK_FILE))?,
    })
}

pub(crate) fn capture_history_relative_path(capture_id: &CaptureId) -> String {
    format!("context/captures/{}.json", capture_id.as_str())
}

pub(crate) fn read_capture_document(
    root: &Path,
    capture_id: &CaptureId,
) -> Result<Option<(ResearchCaptureV1, String)>, ProjectError> {
    read_capture_document_from(root, &capture_history_directory(root), capture_id)
}

pub(crate) fn repository_capture_inbox_relative_path(capture_id: &CaptureId) -> String {
    format!("context/capture-inbox/{}.json", capture_id.as_str())
}

pub(crate) fn read_repository_capture_document(
    root: &Path,
    capture_id: &CaptureId,
) -> Result<Option<(ResearchCaptureV1, String)>, ProjectError> {
    read_capture_document_from(root, &repository_capture_inbox_directory(root), capture_id)
}

fn read_capture_document_from(
    root: &Path,
    directory: &Path,
    capture_id: &CaptureId,
) -> Result<Option<(ResearchCaptureV1, String)>, ProjectError> {
    validate_existing_project_root(root)?;
    let Some(directory_metadata) = project_metadata_if_exists(root, directory)? else {
        return Ok(None);
    };
    validate_project_directory(directory, &directory_metadata)?;
    let path = directory.join(format!("{}.json", capture_id.as_str()));
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    let bytes = read_bounded_project_file(
        root,
        &path,
        &metadata,
        crate::capture::MAX_CAPTURE_BYTES,
        false,
    )?;
    let value = parse_unique_json(&bytes).map_err(|_| ProjectError::InvalidCaptureDocument)?;
    let capture: ResearchCaptureV1 =
        serde_json::from_value(value).map_err(|_| ProjectError::InvalidCaptureDocument)?;
    capture.validate()?;
    if &capture.capture_id != capture_id {
        return Err(ProjectError::CaptureIdentityConflict);
    }
    Ok(Some((capture, sha256(&bytes))))
}

pub(crate) fn list_capture_documents(
    root: &Path,
) -> Result<Vec<(ResearchCaptureV1, String)>, ProjectError> {
    list_capture_documents_from(root, &capture_history_directory(root))
}

pub(crate) fn list_repository_capture_documents(
    root: &Path,
) -> Result<Vec<(ResearchCaptureV1, String)>, ProjectError> {
    list_capture_documents_from(root, &repository_capture_inbox_directory(root))
}

fn list_capture_documents_from(
    root: &Path,
    directory: &Path,
) -> Result<Vec<(ResearchCaptureV1, String)>, ProjectError> {
    const MAX_CAPTURE_DOCUMENTS: usize = 1_024;

    validate_existing_project_root(root)?;
    let Some(directory_metadata) = project_metadata_if_exists(root, directory)? else {
        return Ok(Vec::new());
    };
    validate_project_directory(directory, &directory_metadata)?;

    let mut capture_ids = Vec::new();
    for entry in fs::read_dir(directory).map_err(map_io)? {
        let entry = entry.map_err(map_io)?;
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| ProjectError::InvalidCaptureDocument)?;
        let capture_id = file_name
            .strip_suffix(".json")
            .ok_or(ProjectError::InvalidCaptureDocument)
            .and_then(|value| CaptureId::parse(value.to_string()))?;
        capture_ids.push(capture_id);
        if capture_ids.len() > MAX_CAPTURE_DOCUMENTS {
            return Err(ProjectError::DocumentTooLarge);
        }
    }
    capture_ids.sort();

    capture_ids
        .into_iter()
        .map(|capture_id| {
            read_capture_document_from(root, directory, &capture_id)?
                .ok_or(ProjectError::InvalidCaptureDocument)
        })
        .collect()
}

pub(crate) fn read_portable_capture_document(
    path: &Path,
) -> Result<ResearchCaptureV1, ProjectError> {
    if !path.is_absolute()
        || path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(ProjectError::InvalidCaptureDocument);
    }
    let metadata = metadata_if_exists(path)?.ok_or(ProjectError::InvalidCaptureDocument)?;
    let bytes = read_bounded_file(path, &metadata, crate::capture::MAX_CAPTURE_BYTES, false)
        .map_err(|error| match error {
            ProjectError::UnsafeProjectRoot => ProjectError::InvalidCaptureDocument,
            other => other,
        })?;
    ResearchCaptureV1::from_json_slice(&bytes)
}

pub(crate) fn write_capture_document(
    root: &Path,
    capture: &ResearchCaptureV1,
    _lock: &CaptureHistoryLock,
) -> Result<String, ProjectError> {
    capture.validate()?;
    if read_capture_document(root, &capture.capture_id)?.is_some() {
        return Err(ProjectError::CaptureAlreadyApplied);
    }
    let context = root.join("context");
    ensure_project_directory_beneath(root, &context)?;
    let directory = capture_history_directory(root);
    ensure_project_directory_beneath(root, &directory)?;
    let file_name = format!("{}.json", capture.capture_id.as_str());
    let bytes = serde_json_canonicalizer::to_vec(capture)
        .map_err(|_| ProjectError::InvalidCaptureDocument)?;
    if bytes.len() > crate::capture::MAX_CAPTURE_BYTES {
        return Err(ProjectError::InvalidCaptureDocument);
    }
    atomic_write(&directory, &file_name, &bytes, false)?;
    let Some((committed, digest)) = read_capture_document(root, &capture.capture_id)? else {
        return Err(ProjectError::RecoveryRequired);
    };
    if &committed != capture || sha256(&bytes) != digest {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(digest)
}

pub(crate) fn semantic_digest(root: &Path) -> Result<String, ProjectError> {
    validate_existing_project_root(root)?;
    let graph_project_id = read_manifest(root)?.map(|(manifest, _)| manifest.project_id);
    semantic_digest_from_root(Some(root), &[], graph_project_id.as_ref())
}

pub(crate) fn semantic_digest_for_project(
    root: &Path,
    project_id: &ProjectId,
) -> Result<String, ProjectError> {
    validate_existing_project_root(root)?;
    semantic_digest_from_root(Some(root), &[], Some(project_id))
}

pub(crate) fn empty_semantic_digest() -> String {
    semantic_digest_from_root(None, &[], None).expect("the fixed empty semantic digest cannot fail")
}

pub(crate) fn semantic_digest_with_overrides(
    root: &Path,
    overrides: &[ProjectFileUpdate],
) -> Result<String, ProjectError> {
    validate_existing_project_root(root)?;
    let mut seen = Vec::new();
    for update in overrides {
        if !SEMANTIC_ARTIFACTS.contains(&update.relative_path.as_str())
            || seen.contains(&update.relative_path.as_str())
            || update.next_bytes.len() > MAX_ARTIFACT_BYTES
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        seen.push(update.relative_path.as_str());
    }
    let graph_project_id = read_manifest(root)?.map(|(manifest, _)| manifest.project_id);
    semantic_digest_from_root(Some(root), overrides, graph_project_id.as_ref())
}

fn semantic_digest_from_root(
    root: Option<&Path>,
    overrides: &[ProjectFileUpdate],
    graph_project_id: Option<&ProjectId>,
) -> Result<String, ProjectError> {
    let mut digest = Sha256::new();
    let mut total = 0usize;
    for relative in SEMANTIC_ARTIFACTS {
        digest.update(relative.as_bytes());
        digest.update([0]);
        let override_bytes = overrides
            .iter()
            .find(|update| update.relative_path == relative)
            .map(|update| update.next_bytes.as_slice());
        let bytes = match override_bytes {
            Some(bytes) => Some(bytes.to_vec()),
            None => match root {
                None => None,
                Some(root) => {
                    let path = root.join(relative);
                    project_metadata_if_exists(root, &path)?
                        .map(|metadata| {
                            read_bounded_project_file(
                                root,
                                &path,
                                &metadata,
                                MAX_ARTIFACT_BYTES,
                                false,
                            )
                        })
                        .transpose()?
                }
            },
        };
        match bytes {
            None => digest.update(b"missing"),
            Some(bytes) => {
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
    if let Some(root) = root
        && let Some(bytes) = read_graph_semantic_links(root)?
    {
        let canonical_bytes =
            crate::academic_graph::canonical_semantic_links_bytes(&bytes, graph_project_id)?;
        total
            .checked_add(canonical_bytes.len())
            .filter(|value| *value <= MAX_SEMANTIC_BYTES)
            .ok_or(ProjectError::DocumentTooLarge)?;
        digest.update(GRAPH_SEMANTIC_LINKS_RELATIVE_PATH.as_bytes());
        digest.update([0]);
        digest.update((canonical_bytes.len() as u64).to_be_bytes());
        digest.update(&canonical_bytes);
        digest.update([0xff]);
    }
    Ok(lower_hex(&digest.finalize()))
}

pub(crate) fn read_semantic_artifact(
    root: &Path,
    relative_path: &str,
) -> Result<Option<(Vec<u8>, String)>, ProjectError> {
    validate_existing_project_root(root)?;
    if !SEMANTIC_ARTIFACTS.contains(&relative_path) {
        return Err(ProjectError::InvalidProjectDocument);
    }
    let path = root.join(relative_path);
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    let bytes = read_bounded_project_file(root, &path, &metadata, MAX_ARTIFACT_BYTES, false)?;
    let digest = sha256(&bytes);
    Ok(Some((bytes, digest)))
}

pub(crate) fn read_graph_semantic_links(root: &Path) -> Result<Option<Vec<u8>>, ProjectError> {
    validate_existing_project_root(root)?;
    let path = root.join(GRAPH_SEMANTIC_LINKS_RELATIVE_PATH);
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    read_bounded_project_file(
        root,
        &path,
        &metadata,
        MAX_GRAPH_SEMANTIC_LINKS_BYTES,
        false,
    )
    .map(Some)
}

pub(crate) fn consolidation_relative_path(capture_id: &CaptureId) -> String {
    format!("context/consolidations/{}.json", capture_id.as_str())
}

pub(crate) fn read_consolidation_document(
    root: &Path,
    capture_id: &CaptureId,
) -> Result<Option<Vec<u8>>, ProjectError> {
    validate_existing_project_root(root)?;
    let path = root.join(consolidation_relative_path(capture_id));
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    read_bounded_project_file(root, &path, &metadata, MAX_MANIFEST_BYTES, false).map(Some)
}

pub(crate) fn encode_project_document<T: Serialize>(value: &T) -> Result<Vec<u8>, ProjectError> {
    encode_document(value, false)
}

pub(crate) fn read_private_project_metadata(
    root: &Path,
    relative_path: &str,
) -> Result<Option<Vec<u8>>, ProjectError> {
    validate_existing_project_root(root)?;
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ProjectError::InvalidProjectDocument);
    }
    let path = root.join(relative);
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    read_bounded_project_file(root, &path, &metadata, MAX_MANIFEST_BYTES, true).map(Some)
}

pub(crate) fn write_private_project_metadata_once_locked(
    lock: &ProjectRegistrationJournalLock,
    root: &Path,
    relative_path: &str,
    bytes: &[u8],
) -> Result<(), ProjectError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(ProjectError::DocumentTooLarge);
    }
    if lock.root != root {
        return Err(ProjectError::RecoveryRequired);
    }
    validate_existing_project_root(root)?;
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ProjectError::InvalidProjectDocument);
    }
    let path = root.join(relative);
    let parent = path.parent().ok_or(ProjectError::InvalidProjectDocument)?;
    validate_project_ancestors(root, &path)?;
    let parent_metadata = metadata_if_exists(parent)?.ok_or(ProjectError::RecoveryRequired)?;
    validate_project_directory(parent, &parent_metadata)?;
    if let Some(existing) = read_private_project_metadata(root, relative_path)? {
        return (existing == bytes)
            .then_some(())
            .ok_or(ProjectError::RecoveryRequired);
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ProjectError::InvalidProjectDocument)?;
    atomic_write(parent, file_name, bytes, true)
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    sha256(bytes)
}

impl ProjectFileTransaction {
    pub(crate) fn apply(root: &Path, updates: &[ProjectFileUpdate]) -> Result<Self, ProjectError> {
        validate_existing_project_root(root)?;
        validate_project_file_updates(updates)?;
        let runtime = root.join(PROJECT_RUNTIME_DIR);
        ensure_private_directory_beneath(root, &runtime)?;
        let lock = acquire_lock(&runtime.join(CONSOLIDATION_LOCK_FILE))?;
        let transaction_dir = consolidation_transaction_directory(root);
        if project_metadata_if_exists(root, &transaction_dir)?.is_some() {
            return Err(ProjectError::RecoveryRequired);
        }

        let mut backups = Vec::with_capacity(updates.len());
        for update in updates {
            let previous_bytes = read_transaction_target(root, &update.relative_path)?;
            let previous_digest = previous_bytes.as_deref().map(sha256);
            if previous_digest != update.expected_digest {
                return Err(ProjectError::RevisionConflict);
            }
            backups.push(ProjectFileBackup {
                relative_path: update.relative_path.clone(),
                previous_bytes,
                next_digest: sha256(&update.next_bytes),
            });
        }

        create_private_directory(&transaction_dir)?;
        let mut transaction = Self {
            root: root.to_path_buf(),
            transaction_dir,
            backups,
            finalized: false,
            _lock: lock,
        };
        if let Err(error) = transaction.persist_recovery_evidence(updates) {
            let _ = transaction.rollback_in_place();
            return Err(error);
        }
        for update in updates {
            if let Err(error) = write_transaction_target(root, update) {
                return match transaction.rollback_in_place() {
                    Ok(()) => Err(error),
                    Err(_) => Err(ProjectError::RecoveryRequired),
                };
            }
        }
        Ok(transaction)
    }

    pub(crate) fn rollback(mut self) -> Result<(), ProjectError> {
        self.rollback_in_place()
    }

    pub(crate) fn commit(mut self) -> Result<(), ProjectError> {
        self.finalized = true;
        fs::remove_dir_all(&self.transaction_dir).map_err(map_io)?;
        sync_directory(
            self.transaction_dir
                .parent()
                .ok_or(ProjectError::RecoveryRequired)?,
        )
    }

    pub(crate) fn preserve_for_recovery(mut self) {
        self.finalized = true;
    }

    fn persist_recovery_evidence(&self, updates: &[ProjectFileUpdate]) -> Result<(), ProjectError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct JournalEntry<'a> {
            relative_path: &'a str,
            previous_digest: Option<String>,
            next_digest: String,
            backup_file: Option<String>,
        }

        let mut journal = Vec::with_capacity(self.backups.len());
        for (index, (backup, update)) in self.backups.iter().zip(updates).enumerate() {
            let backup_file = backup
                .previous_bytes
                .as_ref()
                .map(|_| format!("{index}.previous"));
            if let (Some(file_name), Some(bytes)) = (&backup_file, &backup.previous_bytes) {
                atomic_write(&self.transaction_dir, file_name, bytes, true)?;
            }
            journal.push(JournalEntry {
                relative_path: &backup.relative_path,
                previous_digest: backup.previous_bytes.as_deref().map(sha256),
                next_digest: sha256(&update.next_bytes),
                backup_file,
            });
        }
        let bytes = encode_document(&journal, false)?;
        atomic_write(&self.transaction_dir, "journal.json", &bytes, true)
    }

    fn rollback_in_place(&mut self) -> Result<(), ProjectError> {
        for backup in self.backups.iter().rev() {
            let target = self.root.join(&backup.relative_path);
            let current = project_metadata_if_exists(&self.root, &target)?
                .map(|metadata| {
                    read_bounded_project_file(
                        &self.root,
                        &target,
                        &metadata,
                        MAX_ARTIFACT_BYTES,
                        false,
                    )
                })
                .transpose()?;
            let current_digest = current.as_deref().map(sha256);
            let previous_digest = backup.previous_bytes.as_deref().map(sha256);
            if current_digest == previous_digest {
                continue;
            }
            if current_digest.as_deref() != Some(&backup.next_digest) {
                return Err(ProjectError::RecoveryRequired);
            }
            match &backup.previous_bytes {
                Some(bytes) => {
                    let parent = target.parent().ok_or(ProjectError::RecoveryRequired)?;
                    validate_project_ancestors(&self.root, &target)?;
                    let file_name = target
                        .file_name()
                        .and_then(|value| value.to_str())
                        .ok_or(ProjectError::RecoveryRequired)?;
                    atomic_write(parent, file_name, bytes, false)?;
                }
                None if current.is_some() => {
                    validate_project_ancestors(&self.root, &target)?;
                    fs::remove_file(&target).map_err(map_io)?;
                    sync_directory(target.parent().ok_or(ProjectError::RecoveryRequired)?)?;
                }
                None => {}
            }
        }
        fs::remove_dir_all(&self.transaction_dir).map_err(map_io)?;
        sync_directory(
            self.transaction_dir
                .parent()
                .ok_or(ProjectError::RecoveryRequired)?,
        )?;
        self.finalized = true;
        Ok(())
    }
}

impl Drop for ProjectFileTransaction {
    fn drop(&mut self) {
        if !self.finalized {
            let _ = self.rollback_in_place();
        }
    }
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
        if project_metadata_if_exists(root, &root.join(relative))?.is_none() {
            missing.push(kind);
        }
    }
    Ok(missing)
}

pub(crate) fn read_overview(root: &Path) -> Result<ProjectOverviewV1, ProjectError> {
    validate_existing_project_root(root)?;
    let mut overview = ProjectOverviewV1::empty();
    if let Some(bytes) =
        read_optional_project_artifact(root, &root.join("context/research_state.md"))?
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
    if let Some(bytes) =
        read_optional_project_artifact(root, &root.join("evidence/claim-evidence-ledger.csv"))?
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

fn read_optional_project_artifact(
    root: &Path,
    path: &Path,
) -> Result<Option<Vec<u8>>, ProjectError> {
    let Some(metadata) = project_metadata_if_exists(root, path)? else {
        return Ok(None);
    };
    read_bounded_project_file(root, path, &metadata, MAX_ARTIFACT_BYTES, false).map(Some)
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

fn capture_history_directory(root: &Path) -> PathBuf {
    CAPTURE_HISTORY_DIR
        .iter()
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

fn repository_capture_inbox_directory(root: &Path) -> PathBuf {
    REPOSITORY_CAPTURE_INBOX_DIR
        .iter()
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

fn consolidation_transaction_directory(root: &Path) -> PathBuf {
    root.join(PROJECT_RUNTIME_DIR)
        .join(CONSOLIDATION_TRANSACTION_DIR)
}

fn validate_project_file_updates(updates: &[ProjectFileUpdate]) -> Result<(), ProjectError> {
    const MAX_TRANSACTION_FILES: usize = 4;

    if updates.is_empty() || updates.len() > MAX_TRANSACTION_FILES {
        return Err(ProjectError::InvalidProjectDocument);
    }
    let mut relative_paths = Vec::with_capacity(updates.len());
    let mut total = 0usize;
    for update in updates {
        if !valid_transaction_target(&update.relative_path)
            || relative_paths.contains(&update.relative_path.as_str())
            || update.next_bytes.len() > MAX_ARTIFACT_BYTES
            || update.expected_digest.as_deref().is_some_and(|value| {
                value.len() != 64
                    || value
                        .bytes()
                        .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
            })
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        total = total
            .checked_add(update.next_bytes.len())
            .filter(|value| *value <= MAX_SEMANTIC_BYTES)
            .ok_or(ProjectError::DocumentTooLarge)?;
        relative_paths.push(update.relative_path.as_str());
    }
    Ok(())
}

fn valid_transaction_target(relative_path: &str) -> bool {
    if SEMANTIC_ARTIFACTS.contains(&relative_path)
        || relative_path == "context/project_manifest.json"
    {
        return true;
    }
    relative_path
        .strip_prefix("context/consolidations/")
        .and_then(|value| value.strip_suffix(".json"))
        .is_some_and(|value| CaptureId::parse(value.to_string()).is_ok())
}

fn read_transaction_target(
    root: &Path,
    relative_path: &str,
) -> Result<Option<Vec<u8>>, ProjectError> {
    let target = root.join(relative_path);
    let Some(metadata) = project_metadata_if_exists(root, &target)? else {
        return Ok(None);
    };
    read_bounded_project_file(root, &target, &metadata, MAX_ARTIFACT_BYTES, false).map(Some)
}

fn write_transaction_target(root: &Path, update: &ProjectFileUpdate) -> Result<(), ProjectError> {
    let target = root.join(&update.relative_path);
    let parent = target
        .parent()
        .ok_or(ProjectError::InvalidProjectDocument)?;
    ensure_project_directory_beneath(root, parent)?;
    let file_name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ProjectError::InvalidProjectDocument)?;
    atomic_write(parent, file_name, &update.next_bytes, false)
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

fn validate_project_ancestors(root: &Path, target: &Path) -> Result<(), ProjectError> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| ProjectError::UnsafeProjectRoot)?;
    let mut current = root.to_path_buf();
    let Some(parent) = relative.parent() else {
        return Ok(());
    };
    for component in parent.components() {
        let Component::Normal(value) = component else {
            return Err(ProjectError::UnsafeProjectRoot);
        };
        current.push(value);
        let Some(metadata) = metadata_if_exists(&current)? else {
            return Ok(());
        };
        validate_project_directory(&current, &metadata)?;
    }
    Ok(())
}

fn project_metadata_if_exists(
    root: &Path,
    target: &Path,
) -> Result<Option<Metadata>, ProjectError> {
    validate_project_ancestors(root, target)?;
    metadata_if_exists(target)
}

fn ensure_project_directory_beneath(root: &Path, path: &Path) -> Result<(), ProjectError> {
    validate_project_ancestors(root, path)?;
    ensure_project_directory(path)
}

fn ensure_private_directory_beneath(root: &Path, path: &Path) -> Result<(), ProjectError> {
    validate_project_ancestors(root, path)?;
    ensure_private_directory(path)
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

fn read_bounded_project_file(
    root: &Path,
    path: &Path,
    expected: &Metadata,
    max: usize,
    private: bool,
) -> Result<Vec<u8>, ProjectError> {
    validate_project_ancestors(root, path)?;
    read_bounded_file(path, expected, max, private)
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
