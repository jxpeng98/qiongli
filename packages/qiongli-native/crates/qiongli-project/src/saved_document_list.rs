//! Read-only discovery of saved bindings through validated consolidation receipts.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::consolidation::read_consolidation_receipt;
use crate::model::valid_lower_hex;
use crate::storage::{
    list_consolidation_capture_ids, read_capture_document, read_manifest, read_project_source,
    semantic_digest, sha256_bytes,
};
use crate::{
    CaptureId, ConsolidationArtifact, ProjectError, ProjectId, ProjectStateService,
    SavedDocumentReadRequest,
};

const fn default_limit() -> usize {
    32
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SavedDocumentListRequest {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub expected_bindings_sha256: Option<String>,
}

impl SavedDocumentListRequest {
    pub fn validate(&self) -> Result<(), ProjectError> {
        self.project_id.validate()?;
        if self.expected_project_revision == 0
            || !(1..=64).contains(&self.limit)
            || (self.offset > 0 && self.expected_bindings_sha256.is_none())
            || self
                .expected_bindings_sha256
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SavedDocumentBindingState {
    Current,
    Missing,
    Changed,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDocumentBindingV1 {
    pub artifact: ConsolidationArtifact,
    pub relative_path: String,
    pub saved_sha256: String,
    pub saved_at_project_revision: u64,
    pub capture_id: CaptureId,
    pub receipt_sha256: String,
    pub state: SavedDocumentBindingState,
    pub reason_code: Option<String>,
    /// Present only for a safely read, current UTF-8 file. Uses the current revision.
    pub read_arguments: Option<SavedDocumentReadRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDocumentListV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    /// Binds the receipt/capture set, not current research-file bytes.
    pub bindings_sha256: String,
    pub total_documents: usize,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub truncated_before: bool,
    pub truncated_after: bool,
    pub documents: Vec<SavedDocumentBindingV1>,
}

impl ProjectStateService {
    pub fn list_saved_documents(
        &self,
        request: &SavedDocumentListRequest,
    ) -> Result<SavedDocumentListV1, ProjectError> {
        request.validate()?;
        let root = self.resolve_project_root(&request.project_id)?;
        let before = read_manifest(root.path())?.ok_or(ProjectError::ProjectManifestMissing)?;
        if before.0.project_id != request.project_id
            || before.0.semantic_revision != request.expected_project_revision
            || semantic_digest(root.path())? != before.0.semantic_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        let (bindings_sha256, bindings) = saved_bindings(root.path(), request)?;
        if request
            .expected_bindings_sha256
            .as_ref()
            .is_some_and(|expected| expected != &bindings_sha256)
        {
            return Err(ProjectError::RevisionConflict);
        }
        let total_documents = bindings.len();
        if request.offset > total_documents {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let mut documents: Vec<_> = bindings
            .into_values()
            .skip(request.offset)
            .take(request.limit)
            .collect();
        let mut observations = Vec::with_capacity(documents.len());
        for binding in &mut documents {
            let observed = observe_source(root.path(), &binding.relative_path);
            if observed == Err(ProjectError::RecoveryRequired) {
                return Err(ProjectError::RecoveryRequired);
            }
            bind_current_source(binding, &observed, request);
            observations.push(observed);
        }
        // Recheck only this page's file bodies, and the entire saved binding set.
        // Concurrent changes refuse; no lock, refresh or receipt is written.
        for (binding, observed) in documents.iter().zip(observations) {
            if observe_source(root.path(), &binding.relative_path) != observed {
                return Err(ProjectError::RevisionConflict);
            }
        }
        if saved_bindings(root.path(), request)?.0 != bindings_sha256
            || self.resolve_project_root(&request.project_id)?.path() != root.path()
            || read_manifest(root.path())?.as_ref() != Some(&before)
            || semantic_digest(root.path())? != before.0.semantic_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        let end = request.offset + documents.len();
        Ok(SavedDocumentListV1 {
            schema_version: 1,
            document_kind: "qiongli-saved-document-list".into(),
            project_id: request.project_id.clone(),
            project_revision: request.expected_project_revision,
            bindings_sha256,
            total_documents,
            offset: request.offset,
            next_offset: (end < total_documents).then_some(end),
            truncated_before: request.offset > 0,
            truncated_after: end < total_documents,
            documents,
        })
    }
}

type SourceObservation = Result<Option<(String, bool)>, ProjectError>;

fn observe_source(root: &Path, path: &str) -> SourceObservation {
    read_project_source(root, path)
        .map(|source| source.map(|(bytes, digest)| (digest, std::str::from_utf8(&bytes).is_ok())))
}

fn bind_current_source(
    binding: &mut SavedDocumentBindingV1,
    observed: &SourceObservation,
    request: &SavedDocumentListRequest,
) {
    use SavedDocumentBindingState::{Changed, Current, Missing, Unavailable};
    let (state, reason) = match observed {
        Ok(None) => (Missing, Some(ProjectError::RevisionConflict)),
        Ok(Some((digest, _))) if digest != &binding.saved_sha256 => {
            (Changed, Some(ProjectError::RevisionConflict))
        }
        Ok(Some((_, false))) => (
            Unavailable,
            Some(ProjectError::ProjectArtifactContentInvalid),
        ),
        Ok(Some((_, true))) => (Current, None),
        Err(error) => (Unavailable, Some(*error)),
    };
    binding.state = state;
    binding.reason_code = reason.map(|error| error.reason_code().to_string());
    binding.read_arguments = (state == Current).then(|| SavedDocumentReadRequest {
        json_pointer: None,
        project_id: request.project_id.clone(),
        expected_project_revision: request.expected_project_revision,
        relative_path: binding.relative_path.clone(),
        expected_sha256: binding.saved_sha256.clone(),
        offset_bytes: 0,
        max_bytes: 16 * 1024,
    });
}

fn saved_bindings(
    root: &Path,
    request: &SavedDocumentListRequest,
) -> Result<(String, BTreeMap<String, SavedDocumentBindingV1>), ProjectError> {
    let ids = list_consolidation_capture_ids(root)?;
    let mut revisions = BTreeSet::new();
    let mut history = Vec::with_capacity(ids.len());
    let mut bindings: BTreeMap<String, SavedDocumentBindingV1> = BTreeMap::new();
    for id in ids {
        let (receipt, bytes) =
            read_consolidation_receipt(root, &id)?.ok_or(ProjectError::InvalidProjectDocument)?;
        let (capture, capture_digest) =
            read_capture_document(root, &id)?.ok_or(ProjectError::CaptureNotFound)?;
        if receipt.project_id != request.project_id
            || capture.binding.project_id != request.project_id
            || receipt.source_capture_digest != capture_digest
            || receipt.from_project_revision != capture.binding.base_revision
            || receipt.project_stage != capture.binding.stage
            || receipt.to_project_revision > request.expected_project_revision
            || !revisions.insert(receipt.to_project_revision)
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let receipt_digest = sha256_bytes(&bytes);
        history.push((id.clone(), receipt_digest.clone(), capture_digest));
        for artifact in receipt.artifacts {
            if !matches!(
                artifact.artifact,
                ConsolidationArtifact::PaperNote
                    | ConsolidationArtifact::SourcePacket
                    | ConsolidationArtifact::RetrievalManifest
                    | ConsolidationArtifact::StageSummary
            ) {
                continue;
            }
            if bindings
                .get(&artifact.relative_path)
                .is_some_and(|prior| prior.saved_at_project_revision > receipt.to_project_revision)
            {
                continue;
            }
            bindings.insert(
                artifact.relative_path.clone(),
                SavedDocumentBindingV1 {
                    artifact: artifact.artifact,
                    relative_path: artifact.relative_path,
                    saved_sha256: artifact.digest,
                    saved_at_project_revision: receipt.to_project_revision,
                    capture_id: id.clone(),
                    receipt_sha256: receipt_digest.clone(),
                    state: SavedDocumentBindingState::Missing,
                    reason_code: None,
                    read_arguments: None,
                },
            );
        }
    }
    let fingerprint = serde_json::to_vec(&(
        &request.project_id,
        request.expected_project_revision,
        history,
    ))
    .map_err(|_| ProjectError::InvalidProjectDocument)?;
    Ok((sha256_bytes(&fingerprint), bindings))
}
