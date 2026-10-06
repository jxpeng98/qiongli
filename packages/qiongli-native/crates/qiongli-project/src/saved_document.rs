//! Revision- and digest-bound reads of saved research documents, outside Graph.
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::model::valid_lower_hex;
use crate::paper_note::valid_note_path;
use crate::source_packet::valid_packet_path;
use crate::storage::{read_manifest, read_project_source, semantic_digest};
use crate::{ProjectError, ProjectId, ProjectStateService};

pub const MAX_SAVED_DOCUMENT_VIEW_BYTES: usize = 64 * 1024;

const fn default_max_bytes() -> usize {
    16 * 1024
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SavedDocumentReadRequest {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub relative_path: String,
    pub expected_sha256: String,
    #[serde(default)]
    pub offset_bytes: u64,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
}

impl SavedDocumentReadRequest {
    pub fn validate(&self) -> Result<(), ProjectError> {
        self.project_id.validate()?;
        if self.expected_project_revision == 0
            || !valid_lower_hex(&self.expected_sha256, 64)
            || !(4..=MAX_SAVED_DOCUMENT_VIEW_BYTES).contains(&self.max_bytes)
        {
            return Err(ProjectError::InvalidProjectDocument);
        }
        if self.relative_path != "retrieval_manifest.csv"
            && !valid_note_path(&self.relative_path)
            && !valid_packet_path(&self.relative_path)
        {
            return Err(ProjectError::ProjectArtifactUnsupported);
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDocumentViewV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub relative_path: String,
    /// Digest of the complete file, never of just the returned window.
    pub sha256: String,
    pub source_size_bytes: u64,
    pub content_size_bytes: u64,
    pub offset_bytes: u64,
    pub next_offset_bytes: Option<u64>,
    pub truncated_before: bool,
    pub truncated_after: bool,
    pub content: String,
}

impl fmt::Debug for SavedDocumentViewV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SavedDocumentViewV1")
            .field("project_id", &self.project_id)
            .field("project_revision", &self.project_revision)
            .field("sha256", &self.sha256)
            .field("source_size_bytes", &self.source_size_bytes)
            .field("content", &"<saved-document-content>")
            .finish_non_exhaustive()
    }
}

impl ProjectStateService {
    /// Read one explicit saved file. No discovery, Graph projection or write occurs.
    /// Sources are rechecked snapshots, not locks against external editors.
    pub fn read_saved_document(
        &self,
        request: &SavedDocumentReadRequest,
    ) -> Result<SavedDocumentViewV1, ProjectError> {
        request.validate()?;
        let root = self.resolve_project_root(&request.project_id)?;
        let before = read_manifest(root.path())?.ok_or(ProjectError::ProjectManifestMissing)?;
        if before.0.project_id != request.project_id
            || before.0.semantic_revision != request.expected_project_revision
            || semantic_digest(root.path())? != before.0.semantic_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        let (bytes, digest) = read_project_source(root.path(), &request.relative_path)?
            .ok_or(ProjectError::RevisionConflict)?;
        if digest != request.expected_sha256
            || (valid_packet_path(&request.relative_path)
                && !request.relative_path.ends_with(&format!("/{digest}.json")))
        {
            return Err(ProjectError::RevisionConflict);
        }
        let source =
            std::str::from_utf8(&bytes).map_err(|_| ProjectError::ProjectArtifactContentInvalid)?;
        let start = usize::try_from(request.offset_bytes)
            .map_err(|_| ProjectError::InvalidProjectDocument)?;
        if start > source.len() || !source.is_char_boundary(start) {
            return Err(ProjectError::InvalidProjectDocument);
        }
        let mut end = start.saturating_add(request.max_bytes).min(source.len());
        while !source.is_char_boundary(end) {
            end -= 1;
        }

        // Reuse the same bounded, link/ownership/recovery-checked source owner.
        let (_, current_digest) = read_project_source(root.path(), &request.relative_path)?
            .ok_or(ProjectError::RevisionConflict)?;
        if current_digest != digest
            || self.resolve_project_root(&request.project_id)?.path() != root.path()
            || read_manifest(root.path())?.as_ref() != Some(&before)
            || semantic_digest(root.path())? != before.0.semantic_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(SavedDocumentViewV1 {
            schema_version: 1,
            document_kind: "qiongli-saved-document-view".into(),
            project_id: request.project_id.clone(),
            project_revision: request.expected_project_revision,
            relative_path: request.relative_path.clone(),
            sha256: digest,
            source_size_bytes: bytes.len() as u64,
            content_size_bytes: (end - start) as u64,
            offset_bytes: request.offset_bytes,
            next_offset_bytes: (end < source.len()).then_some(end as u64),
            truncated_before: start > 0,
            truncated_after: end < source.len(),
            content: source[start..end].to_owned(),
        })
    }
}
