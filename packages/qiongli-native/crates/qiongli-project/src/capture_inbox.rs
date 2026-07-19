use serde::Serialize;

use crate::capture::{CaptureDisposition, CaptureId, classify_capture};
use crate::model::{ProjectId, ProjectLifecycle, ProjectStage};
use crate::storage::{
    capture_history_relative_path, list_capture_documents, project_root_from_string, read_manifest,
    validate_existing_project_root,
};
use crate::{CaptureDelivery, CapturePolicy, CaptureSource, ProjectError, ProjectStateService};

pub const CAPTURE_INBOX_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureInboxState {
    PendingReview,
    Stale,
    Conflicted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInboxEntryV1 {
    pub capture_id: CaptureId,
    pub state: CaptureInboxState,
    pub disposition: CaptureDisposition,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub captured_at_unix: u64,
    pub base_revision: u64,
    pub bound_stage: ProjectStage,
    pub task: String,
    pub capture_policy: CapturePolicy,
    pub summary: String,
    pub change_count: usize,
    pub decision_count: usize,
    pub evidence_count: usize,
    pub contradiction_count: usize,
    pub next_action_count: usize,
    pub history_entry: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInboxSnapshotV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub project_stage: ProjectStage,
    pub pending_review_count: usize,
    pub stale_count: usize,
    pub conflicted_count: usize,
    pub entries: Vec<CaptureInboxEntryV1>,
}

impl ProjectStateService {
    pub fn capture_inbox(
        &self,
        project_id: &ProjectId,
    ) -> Result<CaptureInboxSnapshotV1, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        let (manifest, _) = read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id
            || entry.semantic_revision != manifest.semantic_revision
            || entry.semantic_digest != manifest.semantic_digest
            || entry.stage != manifest.stage
            || entry.lifecycle != manifest.lifecycle
        {
            return Err(ProjectError::RevisionConflict);
        }

        let mut entries = list_capture_documents(&root)?
            .into_iter()
            .map(|(capture, _)| {
                if capture.binding.project_id != *project_id {
                    return Err(ProjectError::CaptureIdentityConflict);
                }
                let state = capture_state(
                    entry.lifecycle,
                    manifest.semantic_revision,
                    manifest.stage,
                    capture.binding.base_revision,
                    capture.binding.stage,
                );
                Ok(CaptureInboxEntryV1 {
                    capture_id: capture.capture_id.clone(),
                    state,
                    disposition: classify_capture(&capture, false),
                    source: capture.source,
                    delivery: capture.delivery,
                    captured_at_unix: capture.captured_at_unix,
                    base_revision: capture.binding.base_revision,
                    bound_stage: capture.binding.stage,
                    task: capture.binding.task.clone(),
                    capture_policy: capture.binding.capture_policy,
                    summary: capture.summary.clone(),
                    change_count: capture.changes.len(),
                    decision_count: capture.decisions.len(),
                    evidence_count: capture.evidence.len(),
                    contradiction_count: capture.contradictions.len(),
                    next_action_count: capture.next_actions.len(),
                    history_entry: capture_history_relative_path(&capture.capture_id),
                })
            })
            .collect::<Result<Vec<_>, ProjectError>>()?;
        entries.sort_by(|left, right| {
            right
                .captured_at_unix
                .cmp(&left.captured_at_unix)
                .then_with(|| left.capture_id.cmp(&right.capture_id))
        });

        Ok(CaptureInboxSnapshotV1 {
            schema_version: CAPTURE_INBOX_SCHEMA_VERSION,
            project_id: project_id.clone(),
            project_revision: manifest.semantic_revision,
            project_stage: manifest.stage,
            pending_review_count: count_state(&entries, CaptureInboxState::PendingReview),
            stale_count: count_state(&entries, CaptureInboxState::Stale),
            conflicted_count: count_state(&entries, CaptureInboxState::Conflicted),
            entries,
        })
    }
}

fn capture_state(
    lifecycle: ProjectLifecycle,
    current_revision: u64,
    current_stage: ProjectStage,
    base_revision: u64,
    bound_stage: ProjectStage,
) -> CaptureInboxState {
    if lifecycle != ProjectLifecycle::Active
        || base_revision > current_revision
        || (base_revision == current_revision && bound_stage != current_stage)
    {
        CaptureInboxState::Conflicted
    } else if base_revision < current_revision {
        CaptureInboxState::Stale
    } else {
        CaptureInboxState::PendingReview
    }
}

fn count_state(entries: &[CaptureInboxEntryV1], state: CaptureInboxState) -> usize {
    entries.iter().filter(|entry| entry.state == state).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_projection_distinguishes_pending_stale_and_conflicted() {
        assert_eq!(
            capture_state(
                ProjectLifecycle::Active,
                2,
                ProjectStage::Writing,
                2,
                ProjectStage::Writing,
            ),
            CaptureInboxState::PendingReview
        );
        assert_eq!(
            capture_state(
                ProjectLifecycle::Active,
                2,
                ProjectStage::Writing,
                1,
                ProjectStage::Literature,
            ),
            CaptureInboxState::Stale
        );
        assert_eq!(
            capture_state(
                ProjectLifecycle::Archived,
                2,
                ProjectStage::Writing,
                2,
                ProjectStage::Writing,
            ),
            CaptureInboxState::Conflicted
        );
    }
}
