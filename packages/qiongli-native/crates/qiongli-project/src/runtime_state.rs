use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::storage::{
    acquire_lock, atomic_write, ensure_private_directory_beneath, project_metadata_if_exists,
    read_bounded_project_file, read_manifest, sha256_bytes, validate_private_directory,
};
use crate::{ProjectError, ProjectId, ProjectStateService};

pub const PROJECT_RUNTIME_CHECKPOINT_SCHEMA_VERSION: u32 = 1;

const PROJECT_RUNTIME_DIRECTORY: &str = ".qiongli";
const ORCHESTRATION_DIRECTORY: &str = "orchestration";
const ORCHESTRATION_LOCK_FILE: &str = ".orchestration.lock";
const WORKER_ORCHESTRATION_DIRECTORY: &str = "worker-orchestration";
const WORKER_ORCHESTRATION_LOCK_FILE: &str = ".worker-orchestration.lock";
const MAX_CHECKPOINT_BYTES: usize = 1024 * 1024;
const MAX_CHECKPOINTS_PER_PROJECT: usize = 128;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Eq, PartialEq)]
pub struct ProjectRuntimeCheckpointDocument {
    bytes: Vec<u8>,
    sha256: String,
}

impl ProjectRuntimeCheckpointDocument {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

impl Debug for ProjectRuntimeCheckpointDocument {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProjectRuntimeCheckpointDocument")
            .field("bytes", &"<private-checkpoint-bytes>")
            .field("sha256", &self.sha256)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ProjectRuntimeCheckpointEntry {
    checkpoint_id: String,
    document: ProjectRuntimeCheckpointDocument,
}

impl ProjectRuntimeCheckpointEntry {
    #[must_use]
    pub fn checkpoint_id(&self) -> &str {
        &self.checkpoint_id
    }

    #[must_use]
    pub const fn document(&self) -> &ProjectRuntimeCheckpointDocument {
        &self.document
    }
}

impl Debug for ProjectRuntimeCheckpointEntry {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProjectRuntimeCheckpointEntry")
            .field("checkpoint_id", &self.checkpoint_id)
            .field("document", &self.document)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRuntimeCheckpointCommitV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub checkpoint_id: String,
    pub document_sha256: String,
}

impl ProjectStateService {
    pub fn list_orchestration_checkpoints(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<Vec<ProjectRuntimeCheckpointEntry>, ProjectError> {
        let root = self.resolve_root_at_revision(project_id, expected_project_revision)?;
        list_checkpoints_from_root(
            root.path(),
            ORCHESTRATION_DIRECTORY,
            ORCHESTRATION_LOCK_FILE,
        )
    }

    pub fn read_orchestration_checkpoint(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        checkpoint_id: &str,
    ) -> Result<Option<ProjectRuntimeCheckpointDocument>, ProjectError> {
        validate_checkpoint_id(checkpoint_id)?;
        let root = self.resolve_root_at_revision(project_id, expected_project_revision)?;
        read_checkpoint_from_root(root.path(), ORCHESTRATION_DIRECTORY, checkpoint_id)
    }

    pub fn replace_orchestration_checkpoint(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        checkpoint_id: &str,
        expected_document_sha256: Option<&str>,
        bytes: &[u8],
    ) -> Result<ProjectRuntimeCheckpointCommitV1, ProjectError> {
        self.replace_runtime_checkpoint(
            project_id,
            expected_project_revision,
            checkpoint_id,
            expected_document_sha256,
            bytes,
            ORCHESTRATION_DIRECTORY,
            ORCHESTRATION_LOCK_FILE,
        )
    }

    pub fn list_worker_orchestration_checkpoints(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<Vec<ProjectRuntimeCheckpointEntry>, ProjectError> {
        let root = self.resolve_root_at_revision(project_id, expected_project_revision)?;
        list_checkpoints_from_root(
            root.path(),
            WORKER_ORCHESTRATION_DIRECTORY,
            WORKER_ORCHESTRATION_LOCK_FILE,
        )
    }

    pub fn read_worker_orchestration_checkpoint(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        checkpoint_id: &str,
    ) -> Result<Option<ProjectRuntimeCheckpointDocument>, ProjectError> {
        validate_checkpoint_id(checkpoint_id)?;
        let root = self.resolve_root_at_revision(project_id, expected_project_revision)?;
        read_checkpoint_from_root(root.path(), WORKER_ORCHESTRATION_DIRECTORY, checkpoint_id)
    }

    pub fn replace_worker_orchestration_checkpoint(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        checkpoint_id: &str,
        expected_document_sha256: Option<&str>,
        bytes: &[u8],
    ) -> Result<ProjectRuntimeCheckpointCommitV1, ProjectError> {
        self.replace_runtime_checkpoint(
            project_id,
            expected_project_revision,
            checkpoint_id,
            expected_document_sha256,
            bytes,
            WORKER_ORCHESTRATION_DIRECTORY,
            WORKER_ORCHESTRATION_LOCK_FILE,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn replace_runtime_checkpoint(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        checkpoint_id: &str,
        expected_document_sha256: Option<&str>,
        bytes: &[u8],
        directory_name: &str,
        lock_file_name: &str,
    ) -> Result<ProjectRuntimeCheckpointCommitV1, ProjectError> {
        validate_checkpoint_id(checkpoint_id)?;
        if bytes.is_empty() {
            return Err(ProjectError::InvalidProjectDocument);
        }
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        if expected_document_sha256.is_some_and(|digest| !valid_sha256(digest)) {
            return Err(ProjectError::InvalidProjectDocument);
        }

        let root = self.resolve_root_at_revision(project_id, expected_project_revision)?;
        let root_path = root.path().to_path_buf();
        let runtime = root_path.join(PROJECT_RUNTIME_DIRECTORY);
        ensure_private_directory_beneath(&root_path, &runtime)?;
        let _lock = acquire_lock(&runtime.join(lock_file_name))?;

        let revalidated = self.resolve_root_at_revision(project_id, expected_project_revision)?;
        if revalidated.path() != root_path {
            return Err(ProjectError::RevisionConflict);
        }
        let directory = runtime.join(directory_name);
        ensure_private_directory_beneath(&root_path, &directory)?;
        let existing = read_checkpoint_from_root(&root_path, directory_name, checkpoint_id)?;
        match (existing.as_ref(), expected_document_sha256) {
            (None, None) => {}
            (Some(document), Some(expected)) if document.sha256() == expected => {}
            _ => return Err(ProjectError::RevisionConflict),
        }

        let digest = sha256_bytes(bytes);
        if existing
            .as_ref()
            .is_some_and(|document| document.sha256() == digest)
        {
            return Ok(commit(
                project_id,
                expected_project_revision,
                checkpoint_id,
                digest,
            ));
        }

        let file_name = checkpoint_file_name(checkpoint_id);
        atomic_write(&directory, &file_name, bytes, true)?;
        let committed = read_checkpoint_from_root(&root_path, directory_name, checkpoint_id)?
            .ok_or(ProjectError::RecoveryRequired)?;
        if committed.bytes() != bytes || committed.sha256() != digest {
            return Err(ProjectError::RecoveryRequired);
        }
        Ok(commit(
            project_id,
            expected_project_revision,
            checkpoint_id,
            digest,
        ))
    }

    fn resolve_root_at_revision(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<crate::RegisteredProjectRoot, ProjectError> {
        if expected_project_revision == 0 || expected_project_revision > MAX_SAFE_INTEGER {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let root = self.resolve_project_root(project_id)?;
        let (manifest, _) =
            read_manifest(root.path())?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id
            || manifest.semantic_revision != expected_project_revision
        {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(root)
    }
}

fn list_checkpoints_from_root(
    root: &Path,
    directory_name: &str,
    lock_file_name: &str,
) -> Result<Vec<ProjectRuntimeCheckpointEntry>, ProjectError> {
    let runtime = root.join(PROJECT_RUNTIME_DIRECTORY);
    let Some(runtime_metadata) = project_metadata_if_exists(root, &runtime)? else {
        return Ok(Vec::new());
    };
    validate_private_directory(&runtime, &runtime_metadata)?;
    let directory = runtime.join(directory_name);
    let Some(directory_metadata) = project_metadata_if_exists(root, &directory)? else {
        return Ok(Vec::new());
    };
    validate_private_directory(&directory, &directory_metadata)?;
    let lock_path = runtime.join(lock_file_name);
    if project_metadata_if_exists(root, &lock_path)?.is_none() {
        return Err(ProjectError::RecoveryRequired);
    }
    let _lock = acquire_lock(&lock_path)?;

    let mut checkpoint_ids = Vec::new();
    for entry in fs::read_dir(&directory).map_err(map_io)? {
        let file_name = entry
            .map_err(map_io)?
            .file_name()
            .into_string()
            .map_err(|_| ProjectError::InvalidProjectDocument)?;
        let checkpoint_id = file_name
            .strip_suffix(".json")
            .ok_or(ProjectError::InvalidProjectDocument)?;
        validate_checkpoint_id(checkpoint_id)?;
        checkpoint_ids.push(checkpoint_id.to_owned());
        if checkpoint_ids.len() > MAX_CHECKPOINTS_PER_PROJECT {
            return Err(ProjectError::DocumentTooLarge);
        }
    }
    checkpoint_ids.sort();
    checkpoint_ids
        .into_iter()
        .map(|checkpoint_id| {
            let document = read_checkpoint_from_root(root, directory_name, &checkpoint_id)?
                .ok_or(ProjectError::RecoveryRequired)?;
            Ok(ProjectRuntimeCheckpointEntry {
                checkpoint_id,
                document,
            })
        })
        .collect()
}

fn read_checkpoint_from_root(
    root: &Path,
    directory_name: &str,
    checkpoint_id: &str,
) -> Result<Option<ProjectRuntimeCheckpointDocument>, ProjectError> {
    let runtime = root.join(PROJECT_RUNTIME_DIRECTORY);
    let Some(runtime_metadata) = project_metadata_if_exists(root, &runtime)? else {
        return Ok(None);
    };
    validate_private_directory(&runtime, &runtime_metadata)?;
    let directory = runtime.join(directory_name);
    let Some(directory_metadata) = project_metadata_if_exists(root, &directory)? else {
        return Ok(None);
    };
    validate_private_directory(&directory, &directory_metadata)?;
    let path = directory.join(checkpoint_file_name(checkpoint_id));
    let Some(metadata) = project_metadata_if_exists(root, &path)? else {
        return Ok(None);
    };
    let bytes = read_bounded_project_file(root, &path, &metadata, MAX_CHECKPOINT_BYTES, true)?;
    let sha256 = sha256_bytes(&bytes);
    Ok(Some(ProjectRuntimeCheckpointDocument { bytes, sha256 }))
}

fn commit(
    project_id: &ProjectId,
    expected_project_revision: u64,
    checkpoint_id: &str,
    document_sha256: String,
) -> ProjectRuntimeCheckpointCommitV1 {
    ProjectRuntimeCheckpointCommitV1 {
        schema_version: PROJECT_RUNTIME_CHECKPOINT_SCHEMA_VERSION,
        project_id: project_id.clone(),
        expected_project_revision,
        checkpoint_id: checkpoint_id.to_owned(),
        document_sha256,
    }
}

fn checkpoint_file_name(checkpoint_id: &str) -> String {
    format!("{checkpoint_id}.json")
}

fn validate_checkpoint_id(value: &str) -> Result<(), ProjectError> {
    if value.len() == 36
        && value.starts_with("run_")
        && value[4..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ProjectError::InvalidProjectDocument)
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn map_io(error: std::io::Error) -> ProjectError {
    ProjectError::PersistenceFailed(error.kind())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::resolve_config_root;

    use crate::storage::{read_manifest, write_manifest};
    use crate::{
        ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions, ProjectStateService,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn fixture() -> (PathBuf, PathBuf, ProjectStateService, ProjectId) {
        let root = std::env::temp_dir().join(format!(
            "qiongli-runtime-checkpoint-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let service = ProjectStateService::new(resolve_config_root(None, &home).unwrap());
        let project_root = root.join("article");
        let plan = service
            .preview_create(
                &project_root,
                ProjectRegistrationOptions::new("Checkpoint paper", ProjectKind::Article),
                1,
            )
            .unwrap();
        let project_id = plan.preview().project_id.clone();
        service
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        (root, project_root, service, project_id)
    }

    fn checkpoint_id(byte: char) -> String {
        format!("run_{}", byte.to_string().repeat(32))
    }

    #[test]
    fn checkpoint_replace_is_private_atomic_and_revision_bound() {
        let (_fixture, project_root, service, project_id) = fixture();
        let checkpoint_id = checkpoint_id('a');
        assert!(
            service
                .read_orchestration_checkpoint(&project_id, 1, &checkpoint_id)
                .unwrap()
                .is_none()
        );

        let first = br#"{"generation":0}"#;
        let commit = service
            .replace_orchestration_checkpoint(&project_id, 1, &checkpoint_id, None, first)
            .unwrap();
        let loaded = service
            .read_orchestration_checkpoint(&project_id, 1, &checkpoint_id)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.bytes(), first);
        assert_eq!(loaded.sha256(), commit.document_sha256);
        assert!(!format!("{loaded:?}").contains("generation"));
        let listed = service
            .list_orchestration_checkpoints(&project_id, 1)
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].checkpoint_id(), checkpoint_id);
        assert_eq!(listed[0].document(), &loaded);

        let second = br#"{"generation":1}"#;
        let replaced = service
            .replace_orchestration_checkpoint(
                &project_id,
                1,
                &checkpoint_id,
                Some(loaded.sha256()),
                second,
            )
            .unwrap();
        assert_ne!(replaced.document_sha256, commit.document_sha256);
        assert_eq!(
            service.replace_orchestration_checkpoint(
                &project_id,
                1,
                &checkpoint_id,
                Some(&commit.document_sha256),
                first,
            ),
            Err(ProjectError::RevisionConflict)
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(
                project_root
                    .join(PROJECT_RUNTIME_DIRECTORY)
                    .join(ORCHESTRATION_DIRECTORY)
                    .join(checkpoint_file_name(&checkpoint_id)),
            )
            .unwrap();
            assert_eq!(metadata.permissions().mode() & 0o077, 0);
        }
    }

    #[test]
    fn worker_checkpoints_use_an_isolated_private_cas_namespace() {
        let (_fixture, project_root, service, project_id) = fixture();
        let checkpoint_id = checkpoint_id('c');
        let task_document = br#"{"kind":"task"}"#;
        let worker_document = br#"{"kind":"worker"}"#;
        service
            .replace_orchestration_checkpoint(&project_id, 1, &checkpoint_id, None, task_document)
            .unwrap();
        let worker_commit = service
            .replace_worker_orchestration_checkpoint(
                &project_id,
                1,
                &checkpoint_id,
                None,
                worker_document,
            )
            .unwrap();

        let worker = service
            .read_worker_orchestration_checkpoint(&project_id, 1, &checkpoint_id)
            .unwrap()
            .unwrap();
        assert_eq!(worker.bytes(), worker_document);
        assert_eq!(worker.sha256(), worker_commit.document_sha256);
        assert_eq!(
            service
                .read_orchestration_checkpoint(&project_id, 1, &checkpoint_id)
                .unwrap()
                .unwrap()
                .bytes(),
            task_document
        );
        assert_eq!(
            service
                .list_worker_orchestration_checkpoints(&project_id, 1)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            service.replace_worker_orchestration_checkpoint(
                &project_id,
                1,
                &checkpoint_id,
                Some(&"0".repeat(64)),
                br#"{"kind":"stale"}"#,
            ),
            Err(ProjectError::RevisionConflict)
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(
                project_root
                    .join(PROJECT_RUNTIME_DIRECTORY)
                    .join(WORKER_ORCHESTRATION_DIRECTORY)
                    .join(checkpoint_file_name(&checkpoint_id)),
            )
            .unwrap();
            assert_eq!(metadata.permissions().mode() & 0o077, 0);
        }
    }

    #[test]
    fn checkpoint_rejects_revision_drift_links_and_invalid_identity() {
        let (_fixture, project_root, service, project_id) = fixture();
        let checkpoint_id = checkpoint_id('b');
        assert_eq!(
            service.read_orchestration_checkpoint(&project_id, 1, "../private"),
            Err(ProjectError::InvalidProjectDocument)
        );

        let (mut manifest, digest) = read_manifest(&project_root).unwrap().unwrap();
        manifest.semantic_revision = 2;
        manifest.academically_updated_at_unix = 2;
        write_manifest(&project_root, &manifest, Some(&digest)).unwrap();
        assert_eq!(
            service.replace_orchestration_checkpoint(
                &project_id,
                1,
                &checkpoint_id,
                None,
                br#"{"generation":0}"#,
            ),
            Err(ProjectError::RevisionConflict)
        );

        manifest.semantic_revision = 1;
        manifest.academically_updated_at_unix = 1;
        let (_, drift_digest) = read_manifest(&project_root).unwrap().unwrap();
        write_manifest(&project_root, &manifest, Some(&drift_digest)).unwrap();
        let runtime = project_root.join(PROJECT_RUNTIME_DIRECTORY);
        fs::create_dir(&runtime).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            project_root.join("context"),
            runtime.join(ORCHESTRATION_DIRECTORY),
        )
        .unwrap();
        #[cfg(unix)]
        assert_eq!(
            service.read_orchestration_checkpoint(&project_id, 1, &checkpoint_id),
            Err(ProjectError::UnsafeProjectRoot)
        );
    }
}
