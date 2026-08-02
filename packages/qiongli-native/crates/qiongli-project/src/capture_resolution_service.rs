use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::cell::Cell;

use crate::ProjectError;
use crate::ProjectStateService;
use crate::capture::{
    CaptureId, ContradictionV1, DecisionCandidateV1, EvidenceReferenceV1, ResearchCaptureV1,
    SemanticChangeV1,
};
use crate::capture_assignment_service::assignment_artifact_observations;
use crate::capture_delivery::{CaptureDeliveryState, DeliveryAcknowledgementId};
use crate::capture_delivery_service::CaptureDeliveryAcknowledgementRequestV1;
use crate::capture_resolution::{
    CaptureAssignmentOutcome, CaptureAssignmentReceiptId, CaptureAssignmentReceiptV1,
    CaptureResolutionArtifact, CaptureResolutionArtifactObservationV1,
    CaptureResolutionCounterpartState, CaptureResolutionDisposition, CaptureResolutionItemKind,
    CaptureResolutionItemV1, CaptureResolutionPlanInputV1, CaptureResolutionPlanV1,
    CaptureResolutionReceiptId, CaptureResolutionReceiptV1, CaptureResolutionResultV1,
    CaptureResolutionSelectionV1,
};
use crate::consolidation::read_consolidation_receipt;
use crate::model::{
    ArticleProjectManifestV1, MAX_SEMANTIC_REVISION, ProjectId, ProjectLifecycle,
    RegisteredProjectV1,
};
use crate::storage::{
    ProjectFileTransaction, ProjectFileUpdate, assignment_receipt_relative_path,
    capture_history_relative_path, encode_project_document, list_capture_documents,
    list_resolution_receipt_documents, lock_capture_history, project_root_from_string,
    read_assignment_receipt_document, read_capture_document, read_manifest,
    read_resolution_receipt_document, read_semantic_artifact, resolution_receipt_relative_path,
    semantic_digest_with_overrides, sha256_bytes, validate_existing_project_root,
};

pub const CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION: u32 = 1;
const RESEARCH_STATE_PATH: &str = "context/research_state.md";
const DECISION_LOG_PATH: &str = "context/decision_log.md";
const PROJECT_MANIFEST_PATH: &str = "context/project_manifest.json";
const APPROVALS: [&str; 2] = ["academic-review", "filesystem-write"];
const ALL_DISPOSITIONS: [CaptureResolutionDisposition; 4] = [
    CaptureResolutionDisposition::AcceptCurrent,
    CaptureResolutionDisposition::AcceptCapture,
    CaptureResolutionDisposition::RetainBoth,
    CaptureResolutionDisposition::RejectCapture,
];

#[cfg(test)]
thread_local! {
    static DELIVERY_INTERRUPTION_BOUNDARY: Cell<u8> = const { Cell::new(0) };
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum CaptureResolutionItemContentV1 {
    SemanticChange(SemanticChangeV1),
    Decision(DecisionCandidateV1),
    Evidence(EvidenceReferenceV1),
    Contradiction(ContradictionV1),
    NextAction(String),
}

impl CaptureResolutionItemContentV1 {
    const fn kind(&self) -> CaptureResolutionItemKind {
        match self {
            Self::SemanticChange(_) => CaptureResolutionItemKind::SemanticChange,
            Self::Decision(_) => CaptureResolutionItemKind::Decision,
            Self::Evidence(_) => CaptureResolutionItemKind::Evidence,
            Self::Contradiction(_) => CaptureResolutionItemKind::Contradiction,
            Self::NextAction(_) => CaptureResolutionItemKind::NextAction,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResolutionItemPreviewV1 {
    pub item: CaptureResolutionItemV1,
    pub source: CaptureResolutionItemContentV1,
    pub current: Option<CaptureResolutionItemContentV1>,
    pub unavailable_dispositions: Vec<CaptureResolutionDisposition>,
    pub explanation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResolutionPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub assignment_receipt_id: CaptureAssignmentReceiptId,
    pub source_envelope_id: crate::DeliveryEnvelopeId,
    pub source_capture_id: CaptureId,
    pub derived_capture_id: CaptureId,
    pub child_envelope_id: crate::DeliveryEnvelopeId,
    pub target_project_id: ProjectId,
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub next_project_revision: u64,
    pub reviewed_at_unix: u64,
    pub items: Vec<CaptureResolutionItemPreviewV1>,
    pub approvals_required: Vec<String>,
    pub exact_replay: bool,
}

#[derive(Clone)]
pub struct VerifiedCaptureResolution {
    preview: CaptureResolutionPreviewV1,
    resolution_plan: CaptureResolutionPlanV1,
    assignment_receipt: CaptureAssignmentReceiptV1,
    derived_capture: ResearchCaptureV1,
    target_root: PathBuf,
    expected_child_generation: u64,
    expected_child_record_sha256: String,
    existing_receipt: Option<CaptureResolutionReceiptV1>,
}

impl VerifiedCaptureResolution {
    #[must_use]
    pub const fn preview(&self) -> &CaptureResolutionPreviewV1 {
        &self.preview
    }

    #[must_use]
    pub const fn resolution_plan(&self) -> &CaptureResolutionPlanV1 {
        &self.resolution_plan
    }
}

impl Debug for VerifiedCaptureResolution {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedCaptureResolution")
            .field("preview", &self.preview)
            .field("resolution_plan", &"<bounded-resolution-plan>")
            .field("assignment_receipt", &"<bounded-assignment-receipt>")
            .field("derived_capture", &"<bounded-research-capture>")
            .field("target_root", &"<registered-project-root>")
            .field("existing_receipt", &self.existing_receipt.is_some())
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionSelectionSetV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub selection_digest: String,
    pub selections: Vec<CaptureResolutionSelectionV1>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureResolutionSelectionIdentity<'a> {
    schema_version: u32,
    plan_digest: &'a str,
    selections: &'a [CaptureResolutionSelectionV1],
}

impl CaptureResolutionSelectionSetV1 {
    pub fn new(
        plan: &CaptureResolutionPlanV1,
        selections: Vec<CaptureResolutionSelectionV1>,
    ) -> Result<Self, ProjectError> {
        plan.validate()?;
        validate_selections(plan, &selections)?;
        let identity = CaptureResolutionSelectionIdentity {
            schema_version: CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION,
            plan_digest: &plan.plan_digest,
            selections: &selections,
        };
        let selection_digest = canonical_digest(&identity)?;
        Ok(Self {
            schema_version: CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION,
            plan_digest: plan.plan_digest.clone(),
            selection_digest,
            selections,
        })
    }

    fn validate_for(&self, plan: &CaptureResolutionPlanV1) -> Result<(), ProjectError> {
        plan.validate()?;
        validate_selections(plan, &self.selections)?;
        let identity = CaptureResolutionSelectionIdentity {
            schema_version: self.schema_version,
            plan_digest: &self.plan_digest,
            selections: &self.selections,
        };
        if self.schema_version != CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION
            || self.plan_digest != plan.plan_digest
            || self.selection_digest != canonical_digest(&identity)?
        {
            return Err(ProjectError::PlanMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedCaptureResolution {
    expected_plan_digest: String,
    expected_selection_digest: String,
    filesystem_write: bool,
    academic_review: bool,
}

impl ApprovedCaptureResolution {
    #[must_use]
    pub fn new(
        expected_plan_digest: impl Into<String>,
        expected_selection_digest: impl Into<String>,
        filesystem_write: bool,
        academic_review: bool,
    ) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            expected_selection_digest: expected_selection_digest.into(),
            filesystem_write,
            academic_review,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResolutionCommitV1 {
    pub schema_version: u32,
    pub receipt_id: CaptureResolutionReceiptId,
    pub assignment_receipt_id: CaptureAssignmentReceiptId,
    pub resolution_plan_digest: String,
    pub selection_digest: String,
    pub target_project_id: ProjectId,
    pub library_revision: u64,
    pub from_project_revision: u64,
    pub to_project_revision: u64,
    pub artifacts_updated: Vec<CaptureResolutionArtifact>,
    pub child_envelope_id: crate::DeliveryEnvelopeId,
    pub child_state: CaptureDeliveryState,
    pub acknowledgement_id: Option<DeliveryAcknowledgementId>,
    pub index_rebuild_required: bool,
    pub exact_replay: bool,
}

#[derive(Clone)]
struct ResolutionLineage {
    assignment_receipt: CaptureAssignmentReceiptV1,
    derived_capture: ResearchCaptureV1,
    child_generation: u64,
    child_record_sha256: String,
    child_state: CaptureDeliveryState,
}

#[derive(Clone)]
struct ResolutionTarget {
    root: PathBuf,
    library_revision: u64,
    manifest: ArticleProjectManifestV1,
    manifest_sha256: String,
    observations: Vec<CaptureResolutionArtifactObservationV1>,
}

type CurrentItems = BTreeMap<String, BTreeMap<String, CaptureResolutionItemContentV1>>;

#[derive(Clone)]
enum AcademicEvent {
    Consolidation {
        revision: u64,
        identity: String,
        capture: ResearchCaptureV1,
    },
    Resolution {
        revision: u64,
        identity: String,
        receipt: Box<CaptureResolutionReceiptV1>,
        capture: ResearchCaptureV1,
    },
}

impl AcademicEvent {
    fn key(&self) -> (u64, &str) {
        match self {
            Self::Consolidation {
                revision, identity, ..
            }
            | Self::Resolution {
                revision, identity, ..
            } => (*revision, identity),
        }
    }
}

#[derive(Clone)]
struct PlannedArtifact {
    artifact: CaptureResolutionArtifact,
    relative_path: &'static str,
    previous_sha256: Option<String>,
    next_bytes: Vec<u8>,
}

impl ProjectStateService {
    pub fn preview_capture_resolution(
        &self,
        assignment_receipt_id: &CaptureAssignmentReceiptId,
        reviewed_at_unix: u64,
    ) -> Result<VerifiedCaptureResolution, ProjectError> {
        validate_resolution_timestamp(reviewed_at_unix)?;
        let lineage = self.load_resolution_lineage(assignment_receipt_id)?;
        let target =
            self.resolution_target(&lineage.assignment_receipt.receipt.target_project_id)?;
        if reviewed_at_unix < lineage.assignment_receipt.receipt.decided_at_unix {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let existing = resolution_for_assignment(&target.root, assignment_receipt_id)?;
        if let Some(receipt) = existing {
            return self.build_resolution_replay(lineage, target, receipt, reviewed_at_unix);
        }
        if reviewed_at_unix < target.manifest.academically_updated_at_unix {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        if lineage.child_state != CaptureDeliveryState::Queued {
            return Err(ProjectError::RevisionConflict);
        }

        let current = current_academic_items(
            &target.root,
            &target.manifest.project_id,
            target.manifest.semantic_revision,
        )?;
        let items = compare_capture_items(
            &lineage.derived_capture,
            &lineage.assignment_receipt.receipt.source_envelope_id,
            &current,
        )?;
        let resolution_plan = CaptureResolutionPlanV1::new(
            &lineage.assignment_receipt,
            CaptureResolutionPlanInputV1 {
                expected_library_revision: target.library_revision,
                expected_project_revision: target.manifest.semantic_revision,
                target_stage: target.manifest.stage,
                target_manifest_sha256: target.manifest_sha256.clone(),
                observed_artifacts: target.observations,
                items: items.iter().map(|item| item.item.clone()).collect(),
                reviewed_at_unix,
            },
        )?;
        build_verified_resolution(resolution_plan, lineage, target.root, items, None)
    }

    pub fn apply_capture_resolution(
        &self,
        plan: &VerifiedCaptureResolution,
        selections: &CaptureResolutionSelectionSetV1,
        approval: &ApprovedCaptureResolution,
        resolved_at_unix: u64,
    ) -> Result<CaptureResolutionCommitV1, ProjectError> {
        validate_verified_resolution(plan)?;
        selections.validate_for(&plan.resolution_plan)?;
        validate_resolution_timestamp(resolved_at_unix)?;
        if resolved_at_unix < plan.resolution_plan.plan.reviewed_at_unix {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        if !approval.filesystem_write || !approval.academic_review {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.resolution_plan.plan_digest
            || approval.expected_selection_digest != selections.selection_digest
        {
            return Err(ProjectError::PlanMismatch);
        }
        if let Some(receipt) = &plan.existing_receipt {
            return self.replay_capture_resolution(plan, selections, receipt, resolved_at_unix);
        }

        let lineage = self.load_resolution_lineage(&plan.assignment_receipt.receipt_id)?;
        if lineage.assignment_receipt != plan.assignment_receipt
            || lineage.derived_capture != plan.derived_capture
            || lineage.child_state != CaptureDeliveryState::Queued
            || lineage.child_generation != plan.expected_child_generation
            || lineage.child_record_sha256 != plan.expected_child_record_sha256
        {
            return Err(ProjectError::RevisionConflict);
        }
        let target = self.resolution_target(&plan.resolution_plan.plan.target_project_id)?;
        validate_target_for_plan(plan, &target)?;
        if resolved_at_unix < target.manifest.academically_updated_at_unix {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        if resolution_for_assignment(&target.root, &plan.assignment_receipt.receipt_id)?.is_some() {
            return Err(ProjectError::CaptureResolutionAlreadyApplied);
        }

        let planned_artifacts =
            plan_resolution_artifacts(&target.root, plan, &selections.selections)?;
        let semantic_updates = planned_artifacts
            .iter()
            .map(|artifact| ProjectFileUpdate {
                relative_path: artifact.relative_path.to_string(),
                expected_digest: artifact.previous_sha256.clone(),
                next_bytes: artifact.next_bytes.clone(),
            })
            .collect::<Vec<_>>();
        let mut next_manifest = target.manifest.clone();
        next_manifest.semantic_revision = next_manifest
            .semantic_revision
            .checked_add(1)
            .filter(|revision| *revision <= MAX_SEMANTIC_REVISION)
            .ok_or(ProjectError::RevisionConflict)?;
        next_manifest.semantic_digest =
            semantic_digest_with_overrides(&target.root, &semantic_updates)?;
        next_manifest.academically_updated_at_unix = resolved_at_unix;
        next_manifest.validate()?;
        let next_manifest_bytes = encode_project_document(&next_manifest)?;
        let next_manifest_sha256 = sha256_bytes(&next_manifest_bytes);
        let resulting_observations = predicted_resulting_observations(
            &target.root,
            &target.observations,
            &planned_artifacts,
            &plan.derived_capture,
        )?;
        let receipt = CaptureResolutionReceiptV1::new(
            &plan.resolution_plan,
            selections.selections.clone(),
            CaptureResolutionResultV1 {
                resulting_manifest_sha256: next_manifest_sha256,
                resulting_artifacts: resulting_observations.clone(),
                resolved_at_unix,
            },
        )?;
        let mut updates = semantic_updates;
        add_capture_lineage_update(&target.root, &plan.derived_capture, &mut updates)?;
        add_assignment_lineage_update(&target.root, &plan.assignment_receipt, &mut updates)?;
        if read_resolution_receipt_document(&target.root, &receipt.receipt_id)?.is_some() {
            return Err(ProjectError::CaptureResolutionAlreadyApplied);
        }
        updates.push(ProjectFileUpdate {
            relative_path: resolution_receipt_relative_path(&receipt.receipt_id),
            expected_digest: None,
            next_bytes: receipt.to_canonical_json()?,
        });
        updates.push(ProjectFileUpdate {
            relative_path: PROJECT_MANIFEST_PATH.to_string(),
            expected_digest: Some(target.manifest_sha256.clone()),
            next_bytes: next_manifest_bytes,
        });

        let mut mutation = self.store.begin(target.library_revision)?;
        let _capture_lock = lock_capture_history(&target.root)?;
        let prior_entry = mutation
            .document
            .projects
            .iter()
            .find(|entry| entry.project_id == target.manifest.project_id)
            .cloned()
            .ok_or(ProjectError::RevisionConflict)?;
        validate_library_entry(&prior_entry, &target.manifest, &target.root)?;
        let transaction = ProjectFileTransaction::apply(&target.root, &updates)?;
        let next_entry = mutation
            .document
            .projects
            .iter_mut()
            .find(|entry| entry.project_id == target.manifest.project_id)
            .ok_or(ProjectError::RecoveryRequired)?;
        next_entry.semantic_revision = next_manifest.semantic_revision;
        next_entry
            .semantic_digest
            .clone_from(&next_manifest.semantic_digest);
        next_entry.academically_updated_at_unix = next_manifest.academically_updated_at_unix;
        let next_entry = next_entry.clone();
        let expected_next_library_revision = target
            .library_revision
            .checked_add(1)
            .ok_or(ProjectError::RevisionConflict)?;
        let library_revision = match mutation.commit() {
            Ok(revision) => revision,
            Err(error) => match self.store.load() {
                Ok(document)
                    if library_observation_matches(
                        &document,
                        expected_next_library_revision,
                        &next_entry,
                    ) =>
                {
                    expected_next_library_revision
                }
                Ok(document)
                    if library_observation_matches(
                        &document,
                        target.library_revision,
                        &prior_entry,
                    ) =>
                {
                    return match transaction.rollback() {
                        Ok(()) => Err(error),
                        Err(_) => Err(ProjectError::RecoveryRequired),
                    };
                }
                Ok(_) | Err(_) => {
                    transaction.preserve_for_recovery();
                    return Err(ProjectError::RecoveryRequired);
                }
            },
        };
        transaction.commit()?;
        let committed_observations = assignment_artifact_observations(&target.root)?;
        let (committed_manifest, committed_manifest_sha256) =
            read_manifest(&target.root)?.ok_or(ProjectError::RecoveryRequired)?;
        if committed_observations != resulting_observations
            || committed_manifest != next_manifest
            || committed_manifest_sha256 != receipt.receipt.resulting_manifest_sha256
        {
            return Err(ProjectError::RecoveryRequired);
        }
        let child = self
            .finalize_resolution_delivery(
                &plan.derived_capture,
                &plan.resolution_plan,
                next_manifest.semantic_revision,
                resolved_at_unix,
            )
            .map_err(|_| ProjectError::RecoveryRequired)?;
        Ok(resolution_commit(
            &receipt,
            selections,
            library_revision,
            planned_artifacts
                .iter()
                .map(|artifact| artifact.artifact)
                .collect(),
            child.state,
            child
                .acknowledgement
                .as_ref()
                .map(|acknowledgement| acknowledgement.acknowledgement_id.clone()),
            false,
        ))
    }

    pub fn inspect_capture_resolution(
        &self,
        project_id: &ProjectId,
        receipt_id: &CaptureResolutionReceiptId,
    ) -> Result<Option<CaptureResolutionReceiptV1>, ProjectError> {
        let target = self.resolution_target(project_id)?;
        let Some(bytes) = read_resolution_receipt_document(&target.root, receipt_id)? else {
            return Ok(None);
        };
        let receipt = CaptureResolutionReceiptV1::from_json_slice(&bytes)?;
        if &receipt.receipt_id != receipt_id {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
        validate_project_resolution_lineage(&target.root, &receipt)?;
        Ok(Some(receipt))
    }

    pub fn list_capture_resolutions(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<CaptureResolutionReceiptV1>, ProjectError> {
        let target = self.resolution_target(project_id)?;
        list_resolution_receipts(&target.root)
    }

    fn load_resolution_lineage(
        &self,
        assignment_receipt_id: &CaptureAssignmentReceiptId,
    ) -> Result<ResolutionLineage, ProjectError> {
        let assignment = self
            .resolution_store
            .read_assignment_by_receipt_id(assignment_receipt_id)?
            .ok_or(ProjectError::CaptureResolutionConflict)?;
        let receipt = assignment.receipt.ok_or(ProjectError::RecoveryRequired)?;
        if receipt.receipt_id != *assignment_receipt_id
            || receipt.receipt.result.outcome != CaptureAssignmentOutcome::Assigned
        {
            return Err(ProjectError::CaptureResolutionConflict);
        }
        let source = self
            .delivery_store
            .read(&receipt.receipt.source_envelope_id)?
            .ok_or(ProjectError::DeliveryNotFound)?;
        if source.envelope_sha256 != receipt.receipt.source_envelope_sha256
            || source.envelope.capture_id != receipt.receipt.source_capture_id
            || source.envelope.capture_sha256 != receipt.receipt.source_capture_sha256
            || source.record.state != CaptureDeliveryState::Cancelled
            || source.record.generation != receipt.receipt.source_record_generation_after
            || source.record_sha256 != receipt.receipt.source_record_sha256_after
        {
            return Err(ProjectError::RevisionConflict);
        }
        let child_id = receipt
            .receipt
            .result
            .child_envelope_id
            .as_ref()
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let child = self
            .delivery_store
            .read(child_id)?
            .ok_or(ProjectError::DeliveryNotFound)?;
        let derived_capture_id = receipt
            .receipt
            .result
            .derived_capture_id
            .as_ref()
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let derived_capture_sha256 = receipt
            .receipt
            .result
            .derived_capture_sha256
            .as_deref()
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let destination = child
            .envelope
            .destination
            .as_ref()
            .ok_or(ProjectError::ResolutionIdentityConflict)?;
        if child.envelope.envelope_id != *child_id
            || child.envelope.capture_id != *derived_capture_id
            || child.envelope.capture_sha256 != derived_capture_sha256
            || destination.project_id != receipt.receipt.target_project_id
            || destination.expected_project_revision != receipt.receipt.target_project_revision
            || child.envelope.capture.binding.project_id != receipt.receipt.target_project_id
            || child.envelope.capture.binding.base_revision
                != receipt.receipt.target_project_revision
            || child.envelope.capture.binding.stage != receipt.receipt.target_stage
        {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
        Ok(ResolutionLineage {
            assignment_receipt: receipt,
            derived_capture: child.envelope.capture,
            child_generation: child.record.generation,
            child_record_sha256: child.record_sha256,
            child_state: child.record.state,
        })
    }

    fn resolution_target(&self, project_id: &ProjectId) -> Result<ResolutionTarget, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        if entry.lifecycle != ProjectLifecycle::Active {
            return Err(ProjectError::RevisionConflict);
        }
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        let (manifest, manifest_sha256) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        validate_library_entry(entry, &manifest, &root)?;
        Ok(ResolutionTarget {
            root: root.clone(),
            library_revision: library.revision,
            observations: assignment_artifact_observations(&root)?,
            manifest,
            manifest_sha256,
        })
    }

    fn build_resolution_replay(
        &self,
        lineage: ResolutionLineage,
        target: ResolutionTarget,
        receipt: CaptureResolutionReceiptV1,
        reviewed_at_unix: u64,
    ) -> Result<VerifiedCaptureResolution, ProjectError> {
        if receipt.receipt.reviewed_at_unix != reviewed_at_unix
            || receipt.receipt.target_project_id != target.manifest.project_id
            || receipt.receipt.to_project_revision != target.manifest.semantic_revision
            || receipt.receipt.resulting_manifest_sha256 != target.manifest_sha256
            || receipt.receipt.resulting_artifacts != target.observations
        {
            return Err(ProjectError::CaptureResolutionAlreadyApplied);
        }
        let items = receipt
            .receipt
            .decisions
            .iter()
            .map(|decision| decision.item.clone())
            .collect::<Vec<_>>();
        let resolution_plan = CaptureResolutionPlanV1::new(
            &lineage.assignment_receipt,
            CaptureResolutionPlanInputV1 {
                expected_library_revision: receipt.receipt.expected_library_revision,
                expected_project_revision: receipt.receipt.from_project_revision,
                target_stage: receipt.receipt.target_stage,
                target_manifest_sha256: receipt.receipt.previous_manifest_sha256.clone(),
                observed_artifacts: receipt.receipt.observed_artifacts.clone(),
                items,
                reviewed_at_unix,
            },
        )?;
        if resolution_plan.plan_digest != receipt.receipt.resolution_plan_digest {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
        let all_contents = all_capture_contents_by_digest(&target.root)?;
        let item_previews = resolution_plan
            .plan
            .items
            .iter()
            .map(|item| {
                let source = content_at(&lineage.derived_capture, item.kind, item.source_index)?;
                if content_digest(&source)? != item.source_item_sha256 {
                    return Err(ProjectError::ResolutionIdentityConflict);
                }
                let current = item
                    .current_item_sha256
                    .as_ref()
                    .map(|digest| {
                        all_contents
                            .get(digest)
                            .cloned()
                            .ok_or(ProjectError::ResolutionIdentityConflict)
                    })
                    .transpose()?;
                item_preview(item.clone(), source, current)
            })
            .collect::<Result<Vec<_>, _>>()?;
        build_verified_resolution(
            resolution_plan,
            lineage,
            target.root,
            item_previews,
            Some(receipt),
        )
    }

    fn replay_capture_resolution(
        &self,
        plan: &VerifiedCaptureResolution,
        selections: &CaptureResolutionSelectionSetV1,
        receipt: &CaptureResolutionReceiptV1,
        resolved_at_unix: u64,
    ) -> Result<CaptureResolutionCommitV1, ProjectError> {
        let receipt_selections = receipt
            .receipt
            .decisions
            .iter()
            .map(|decision| CaptureResolutionSelectionV1 {
                item_id: decision.item.item_id.clone(),
                disposition: decision.disposition,
            })
            .collect::<Vec<_>>();
        if receipt.receipt.resolved_at_unix != resolved_at_unix
            || receipt.receipt.resolution_plan_digest != plan.resolution_plan.plan_digest
            || receipt_selections != selections.selections
        {
            return Err(ProjectError::CaptureResolutionAlreadyApplied);
        }
        let target = self.resolution_target(&plan.resolution_plan.plan.target_project_id)?;
        if target.manifest.semantic_revision != receipt.receipt.to_project_revision
            || target.manifest_sha256 != receipt.receipt.resulting_manifest_sha256
            || target.observations != receipt.receipt.resulting_artifacts
        {
            return Err(ProjectError::RevisionConflict);
        }
        let child = self
            .finalize_resolution_delivery(
                &plan.derived_capture,
                &plan.resolution_plan,
                receipt.receipt.to_project_revision,
                resolved_at_unix,
            )
            .map_err(|_| ProjectError::RecoveryRequired)?;
        let artifacts_updated = changed_artifacts(receipt);
        Ok(resolution_commit(
            receipt,
            selections,
            target.library_revision,
            artifacts_updated,
            child.state,
            child
                .acknowledgement
                .as_ref()
                .map(|acknowledgement| acknowledgement.acknowledgement_id.clone()),
            true,
        ))
    }

    fn finalize_resolution_delivery(
        &self,
        derived_capture: &ResearchCaptureV1,
        plan: &CaptureResolutionPlanV1,
        resulting_project_revision: u64,
        resolved_at_unix: u64,
    ) -> Result<crate::CaptureDeliveryStatusV1, ProjectError> {
        let envelope_id = &plan.plan.child_envelope_id;
        let mut status = self
            .inspect_capture_delivery(envelope_id)?
            .ok_or(ProjectError::DeliveryNotFound)?;
        if status.capture_id != derived_capture.capture_id {
            return Err(ProjectError::DeliveryIdentityConflict);
        }
        if status.state == CaptureDeliveryState::Queued {
            status = self.begin_capture_delivery(
                envelope_id,
                status.generation,
                &status.record_sha256,
                resolved_at_unix,
            )?;
            inject_delivery_interruption(1)?;
        }
        if status.state == CaptureDeliveryState::Delivering {
            status = self.record_capture_delivery(
                envelope_id,
                status.generation,
                &status.record_sha256,
                resolved_at_unix,
            )?;
            inject_delivery_interruption(2)?;
        }
        if status.state == CaptureDeliveryState::Delivered {
            status = self.acknowledge_capture_delivery(
                &CaptureDeliveryAcknowledgementRequestV1 {
                    envelope_id: envelope_id.clone(),
                    destination_project_id: plan.plan.target_project_id.clone(),
                    accepted_capture_id: derived_capture.capture_id.clone(),
                    expected_project_revision: plan.plan.assigned_project_revision,
                    resulting_project_revision,
                    acknowledged_at_unix: resolved_at_unix,
                },
                status.generation,
                &status.record_sha256,
            )?;
        }
        if status.state != CaptureDeliveryState::Acknowledged {
            return Err(ProjectError::InvalidDeliveryTransition);
        }
        let acknowledgement = status
            .acknowledgement
            .as_ref()
            .ok_or(ProjectError::DeliveryAcknowledgementConflict)?;
        if acknowledgement.destination_project_id != plan.plan.target_project_id
            || acknowledgement.accepted_capture_id != derived_capture.capture_id
            || acknowledgement.expected_project_revision != plan.plan.assigned_project_revision
            || acknowledgement.resulting_project_revision != resulting_project_revision
            || acknowledgement.acknowledged_at_unix != resolved_at_unix
        {
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }
        Ok(status)
    }
}

fn build_verified_resolution(
    resolution_plan: CaptureResolutionPlanV1,
    lineage: ResolutionLineage,
    target_root: PathBuf,
    items: Vec<CaptureResolutionItemPreviewV1>,
    existing_receipt: Option<CaptureResolutionReceiptV1>,
) -> Result<VerifiedCaptureResolution, ProjectError> {
    resolution_plan.validate()?;
    if items
        .iter()
        .map(|item| &item.item)
        .ne(resolution_plan.plan.items.iter())
    {
        return Err(ProjectError::PlanMismatch);
    }
    let preview = CaptureResolutionPreviewV1 {
        schema_version: CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION,
        plan_digest: resolution_plan.plan_digest.clone(),
        assignment_receipt_id: resolution_plan.plan.assignment_receipt_id.clone(),
        source_envelope_id: resolution_plan.plan.source_envelope_id.clone(),
        source_capture_id: resolution_plan.plan.source_capture_id.clone(),
        derived_capture_id: resolution_plan.plan.derived_capture_id.clone(),
        child_envelope_id: resolution_plan.plan.child_envelope_id.clone(),
        target_project_id: resolution_plan.plan.target_project_id.clone(),
        expected_library_revision: resolution_plan.plan.expected_library_revision,
        expected_project_revision: resolution_plan.plan.expected_project_revision,
        next_project_revision: resolution_plan.plan.expected_project_revision + 1,
        reviewed_at_unix: resolution_plan.plan.reviewed_at_unix,
        items,
        approvals_required: APPROVALS.iter().map(ToString::to_string).collect(),
        exact_replay: existing_receipt.is_some(),
    };
    Ok(VerifiedCaptureResolution {
        preview,
        resolution_plan,
        assignment_receipt: lineage.assignment_receipt,
        derived_capture: lineage.derived_capture,
        target_root,
        expected_child_generation: lineage.child_generation,
        expected_child_record_sha256: lineage.child_record_sha256,
        existing_receipt,
    })
}

fn validate_verified_resolution(plan: &VerifiedCaptureResolution) -> Result<(), ProjectError> {
    plan.resolution_plan.validate()?;
    if plan.preview.schema_version != CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION
        || plan.preview.plan_digest != plan.resolution_plan.plan_digest
        || plan.preview.assignment_receipt_id != plan.resolution_plan.plan.assignment_receipt_id
        || plan.preview.source_envelope_id != plan.resolution_plan.plan.source_envelope_id
        || plan.preview.source_capture_id != plan.resolution_plan.plan.source_capture_id
        || plan.preview.derived_capture_id != plan.resolution_plan.plan.derived_capture_id
        || plan.preview.child_envelope_id != plan.resolution_plan.plan.child_envelope_id
        || plan.preview.target_project_id != plan.resolution_plan.plan.target_project_id
        || plan.preview.expected_library_revision
            != plan.resolution_plan.plan.expected_library_revision
        || plan.preview.expected_project_revision
            != plan.resolution_plan.plan.expected_project_revision
        || plan.preview.next_project_revision
            != plan.resolution_plan.plan.expected_project_revision + 1
        || plan.preview.reviewed_at_unix != plan.resolution_plan.plan.reviewed_at_unix
        || plan.preview.approvals_required
            != APPROVALS
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        || plan.preview.exact_replay != plan.existing_receipt.is_some()
        || plan.preview.items.iter().map(|item| &item.item).ne(plan
            .resolution_plan
            .plan
            .items
            .iter())
    {
        return Err(ProjectError::PlanMismatch);
    }
    for item in &plan.preview.items {
        if item.source.kind() != item.item.kind
            || content_digest(&item.source)? != item.item.source_item_sha256
            || item
                .current
                .as_ref()
                .is_some_and(|current| current.kind() != item.item.kind)
            || item.current.as_ref().map(content_digest).transpose()?
                != item.item.current_item_sha256
        {
            return Err(ProjectError::PlanMismatch);
        }
    }
    Ok(())
}

fn validate_target_for_plan(
    plan: &VerifiedCaptureResolution,
    target: &ResolutionTarget,
) -> Result<(), ProjectError> {
    if target.root != plan.target_root
        || target.library_revision != plan.resolution_plan.plan.expected_library_revision
        || target.manifest.semantic_revision != plan.resolution_plan.plan.expected_project_revision
        || target.manifest.stage != plan.resolution_plan.plan.target_stage
        || target.manifest_sha256 != plan.resolution_plan.plan.target_manifest_sha256
        || target.observations != plan.resolution_plan.plan.observed_artifacts
    {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

fn validate_library_entry(
    entry: &RegisteredProjectV1,
    manifest: &ArticleProjectManifestV1,
    root: &Path,
) -> Result<(), ProjectError> {
    if entry.lifecycle != ProjectLifecycle::Active
        || manifest.lifecycle != ProjectLifecycle::Active
        || entry.project_id != manifest.project_id
        || entry.root_path != root.to_str().unwrap_or_default()
        || entry.project_kind != manifest.project_kind
        || entry.stage != manifest.stage
        || entry.semantic_revision != manifest.semantic_revision
        || entry.semantic_digest != manifest.semantic_digest
        || entry.display_name != manifest.display_name
    {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

fn compare_capture_items(
    capture: &ResearchCaptureV1,
    source_envelope_id: &crate::DeliveryEnvelopeId,
    current: &CurrentItems,
) -> Result<Vec<CaptureResolutionItemPreviewV1>, ProjectError> {
    capture_contents(capture)
        .into_iter()
        .map(|(kind, source_index, source)| {
            let source_digest = content_digest(&source)?;
            let identity = content_identity_digest(&source)?;
            let candidates = current.get(&identity);
            let (counterpart_state, current_digest, current_content) = match candidates {
                None => (CaptureResolutionCounterpartState::Absent, None, None),
                Some(candidates) if candidates.contains_key(&source_digest) => (
                    CaptureResolutionCounterpartState::ExactMatch,
                    Some(source_digest.clone()),
                    Some(source.clone()),
                ),
                Some(candidates) if candidates.len() == 1 => {
                    let (digest, content) = candidates
                        .first_key_value()
                        .ok_or(ProjectError::ResolutionIdentityConflict)?;
                    (
                        CaptureResolutionCounterpartState::ExactIdentityDivergent,
                        Some(digest.clone()),
                        Some(content.clone()),
                    )
                }
                Some(_) => return Err(ProjectError::ResolutionIdentityConflict),
            };
            let item = CaptureResolutionItemV1::new(
                source_envelope_id.clone(),
                kind,
                source_index,
                source_digest,
                counterpart_state,
                current_digest,
            )?;
            item_preview(item, source, current_content)
        })
        .collect()
}

fn item_preview(
    item: CaptureResolutionItemV1,
    source: CaptureResolutionItemContentV1,
    current: Option<CaptureResolutionItemContentV1>,
) -> Result<CaptureResolutionItemPreviewV1, ProjectError> {
    let unavailable_dispositions = ALL_DISPOSITIONS
        .into_iter()
        .filter(|disposition| !item.allowed_dispositions.contains(disposition))
        .collect();
    let explanation = match item.counterpart_state {
        CaptureResolutionCounterpartState::Absent => "no-exact-current-counterpart",
        CaptureResolutionCounterpartState::ExactMatch => "exact-current-counterpart",
        CaptureResolutionCounterpartState::ExactIdentityDivergent => {
            if item.kind == CaptureResolutionItemKind::SemanticChange {
                "exact-identity-diverges-replacement-only"
            } else {
                "exact-identity-diverges-coexistence-supported"
            }
        }
    }
    .to_string();
    Ok(CaptureResolutionItemPreviewV1 {
        item,
        source,
        current,
        unavailable_dispositions,
        explanation,
    })
}

fn current_academic_items(
    root: &Path,
    project_id: &ProjectId,
    current_revision: u64,
) -> Result<CurrentItems, ProjectError> {
    let captures = list_capture_documents(root)?;
    let capture_map = captures
        .iter()
        .map(|(capture, digest)| {
            (
                capture.capture_id.as_str().to_string(),
                (capture.clone(), digest.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut events = Vec::new();
    for (capture, digest) in &captures {
        if let Some((receipt, _)) = read_consolidation_receipt(root, &capture.capture_id)? {
            if receipt.project_id != *project_id
                || receipt.source_capture_digest != *digest
                || receipt.to_project_revision > current_revision
            {
                return Err(ProjectError::ResolutionIdentityConflict);
            }
            events.push(AcademicEvent::Consolidation {
                revision: receipt.to_project_revision,
                identity: format!("consolidation:{}", capture.capture_id.as_str()),
                capture: capture.clone(),
            });
        }
    }
    for receipt in list_resolution_receipts(root)? {
        validate_project_resolution_lineage(root, &receipt)?;
        if receipt.receipt.target_project_id != *project_id
            || receipt.receipt.to_project_revision > current_revision
        {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
        let (capture, digest) = capture_map
            .get(receipt.receipt.derived_capture_id.as_str())
            .ok_or(ProjectError::ResolutionIdentityConflict)?;
        if digest != &receipt.receipt.derived_capture_sha256 {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
        events.push(AcademicEvent::Resolution {
            revision: receipt.receipt.to_project_revision,
            identity: format!("resolution:{}", receipt.receipt_id.as_str()),
            receipt: Box::new(receipt),
            capture: capture.clone(),
        });
    }
    events.sort_by(|left, right| left.key().cmp(&right.key()));
    for pair in events.windows(2) {
        if pair[0].key().0 == pair[1].key().0 {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
    }
    let mut current = CurrentItems::new();
    for event in events {
        match event {
            AcademicEvent::Consolidation { capture, .. } => {
                for (_, _, content) in capture_contents(&capture) {
                    add_current_item(&mut current, content)?;
                }
            }
            AcademicEvent::Resolution {
                receipt, capture, ..
            } => {
                for decision in &receipt.receipt.decisions {
                    let content =
                        content_at(&capture, decision.item.kind, decision.item.source_index)?;
                    if content_digest(&content)? != decision.item.source_item_sha256 {
                        return Err(ProjectError::ResolutionIdentityConflict);
                    }
                    match decision.disposition {
                        CaptureResolutionDisposition::AcceptCapture => {
                            replace_current_item(&mut current, content)?;
                        }
                        CaptureResolutionDisposition::RetainBoth => {
                            add_current_item(&mut current, content)?;
                        }
                        CaptureResolutionDisposition::AcceptCurrent
                        | CaptureResolutionDisposition::RejectCapture => {}
                    }
                }
            }
        }
    }
    Ok(current)
}

fn add_current_item(
    current: &mut CurrentItems,
    content: CaptureResolutionItemContentV1,
) -> Result<(), ProjectError> {
    let identity = content_identity_digest(&content)?;
    let digest = content_digest(&content)?;
    current.entry(identity).or_default().insert(digest, content);
    Ok(())
}

fn replace_current_item(
    current: &mut CurrentItems,
    content: CaptureResolutionItemContentV1,
) -> Result<(), ProjectError> {
    let identity = content_identity_digest(&content)?;
    let digest = content_digest(&content)?;
    current.insert(identity, BTreeMap::from([(digest, content)]));
    Ok(())
}

fn capture_contents(
    capture: &ResearchCaptureV1,
) -> Vec<(
    CaptureResolutionItemKind,
    u16,
    CaptureResolutionItemContentV1,
)> {
    let mut contents = Vec::new();
    contents.extend(
        capture
            .changes
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| {
                (
                    CaptureResolutionItemKind::SemanticChange,
                    index as u16,
                    CaptureResolutionItemContentV1::SemanticChange(value),
                )
            }),
    );
    contents.extend(
        capture
            .decisions
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| {
                (
                    CaptureResolutionItemKind::Decision,
                    index as u16,
                    CaptureResolutionItemContentV1::Decision(value),
                )
            }),
    );
    contents.extend(
        capture
            .evidence
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| {
                (
                    CaptureResolutionItemKind::Evidence,
                    index as u16,
                    CaptureResolutionItemContentV1::Evidence(value),
                )
            }),
    );
    contents.extend(
        capture
            .contradictions
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| {
                (
                    CaptureResolutionItemKind::Contradiction,
                    index as u16,
                    CaptureResolutionItemContentV1::Contradiction(value),
                )
            }),
    );
    contents.extend(
        capture
            .next_actions
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, value)| {
                (
                    CaptureResolutionItemKind::NextAction,
                    index as u16,
                    CaptureResolutionItemContentV1::NextAction(value),
                )
            }),
    );
    contents
}

fn content_at(
    capture: &ResearchCaptureV1,
    kind: CaptureResolutionItemKind,
    source_index: u16,
) -> Result<CaptureResolutionItemContentV1, ProjectError> {
    let index = usize::from(source_index);
    match kind {
        CaptureResolutionItemKind::SemanticChange => capture
            .changes
            .get(index)
            .cloned()
            .map(CaptureResolutionItemContentV1::SemanticChange),
        CaptureResolutionItemKind::Decision => capture
            .decisions
            .get(index)
            .cloned()
            .map(CaptureResolutionItemContentV1::Decision),
        CaptureResolutionItemKind::Evidence => capture
            .evidence
            .get(index)
            .cloned()
            .map(CaptureResolutionItemContentV1::Evidence),
        CaptureResolutionItemKind::Contradiction => capture
            .contradictions
            .get(index)
            .cloned()
            .map(CaptureResolutionItemContentV1::Contradiction),
        CaptureResolutionItemKind::NextAction => capture
            .next_actions
            .get(index)
            .cloned()
            .map(CaptureResolutionItemContentV1::NextAction),
    }
    .ok_or(ProjectError::ResolutionIdentityConflict)
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "identity", rename_all = "kebab-case")]
enum ContentIdentity<'a> {
    SemanticChange {
        area: crate::CaptureArea,
    },
    DecisionTarget {
        target: &'a str,
    },
    DecisionCandidate {
        statement: &'a str,
    },
    Evidence {
        locator_kind: crate::EvidenceLocatorKind,
        locator: &'a str,
    },
    Contradiction {
        statement: &'a str,
        conflicts_with: &'a str,
    },
    NextAction {
        value: &'a str,
    },
}

fn content_identity_digest(
    content: &CaptureResolutionItemContentV1,
) -> Result<String, ProjectError> {
    let identity = match content {
        CaptureResolutionItemContentV1::SemanticChange(change) => {
            ContentIdentity::SemanticChange { area: change.area }
        }
        CaptureResolutionItemContentV1::Decision(decision) => match decision.target.as_deref() {
            Some(target) => ContentIdentity::DecisionTarget { target },
            None => ContentIdentity::DecisionCandidate {
                statement: &decision.statement,
            },
        },
        CaptureResolutionItemContentV1::Evidence(evidence) => ContentIdentity::Evidence {
            locator_kind: evidence.locator_kind,
            locator: &evidence.locator,
        },
        CaptureResolutionItemContentV1::Contradiction(contradiction) => {
            ContentIdentity::Contradiction {
                statement: &contradiction.statement,
                conflicts_with: &contradiction.conflicts_with,
            }
        }
        CaptureResolutionItemContentV1::NextAction(value) => ContentIdentity::NextAction { value },
    };
    canonical_digest(&identity)
}

fn content_digest(content: &CaptureResolutionItemContentV1) -> Result<String, ProjectError> {
    canonical_digest(content)
}

fn all_capture_contents_by_digest(
    root: &Path,
) -> Result<BTreeMap<String, CaptureResolutionItemContentV1>, ProjectError> {
    let mut contents = BTreeMap::new();
    for (capture, _) in list_capture_documents(root)? {
        for (_, _, content) in capture_contents(&capture) {
            let digest = content_digest(&content)?;
            if let Some(existing) = contents.insert(digest, content.clone())
                && existing != content
            {
                return Err(ProjectError::ResolutionIdentityConflict);
            }
        }
    }
    Ok(contents)
}

fn plan_resolution_artifacts(
    root: &Path,
    plan: &VerifiedCaptureResolution,
    selections: &[CaptureResolutionSelectionV1],
) -> Result<Vec<PlannedArtifact>, ProjectError> {
    let mut by_artifact = BTreeMap::<CaptureResolutionArtifact, Vec<usize>>::new();
    for (index, item) in plan.preview.items.iter().enumerate() {
        let artifact = match item.item.kind {
            CaptureResolutionItemKind::Decision => CaptureResolutionArtifact::DecisionLog,
            CaptureResolutionItemKind::SemanticChange
            | CaptureResolutionItemKind::Evidence
            | CaptureResolutionItemKind::Contradiction
            | CaptureResolutionItemKind::NextAction => CaptureResolutionArtifact::ResearchState,
        };
        by_artifact.entry(artifact).or_default().push(index);
    }
    let mut artifacts = Vec::new();
    for (artifact, indexes) in by_artifact {
        let relative_path =
            artifact_relative_path(artifact).ok_or(ProjectError::InvalidResolutionDocument)?;
        let observed = read_semantic_artifact(root, relative_path)?;
        let (previous, previous_sha256) = match observed {
            Some((bytes, digest)) => (
                String::from_utf8(bytes).map_err(|_| ProjectError::CaptureResolutionConflict)?,
                Some(digest),
            ),
            None => (String::new(), None),
        };
        let marker = format!(
            "<!-- qiongli:resolution-plan {} begin -->",
            plan.resolution_plan.plan_digest
        );
        if previous.contains(&marker) {
            return Err(ProjectError::ResolutionIdentityConflict);
        }
        let next = render_resolution_artifact(
            &previous,
            artifact,
            &plan.resolution_plan,
            &plan.preview.items,
            selections,
            &indexes,
        )?;
        artifacts.push(PlannedArtifact {
            artifact,
            relative_path,
            previous_sha256,
            next_bytes: next.into_bytes(),
        });
    }
    Ok(artifacts)
}

fn render_resolution_artifact(
    previous: &str,
    artifact: CaptureResolutionArtifact,
    plan: &CaptureResolutionPlanV1,
    items: &[CaptureResolutionItemPreviewV1],
    selections: &[CaptureResolutionSelectionV1],
    indexes: &[usize],
) -> Result<String, ProjectError> {
    let heading = match artifact {
        CaptureResolutionArtifact::ResearchState => "# Research State",
        CaptureResolutionArtifact::DecisionLog => "# Decision Log",
        CaptureResolutionArtifact::CaptureHistory
        | CaptureResolutionArtifact::ConsolidationHistory => {
            return Err(ProjectError::InvalidResolutionDocument);
        }
    };
    let mut output = prepare_document(previous, heading);
    let digest = &plan.plan_digest;
    output.push_str(&format!(
        "<!-- qiongli:resolution-plan {digest} begin -->\n"
    ));
    output.push_str(&format!("## Reviewed capture resolution `{digest}`\n\n"));
    output.push_str(&format!(
        "- Assignment receipt: `{}`\n",
        plan.plan.assignment_receipt_id.as_str()
    ));
    output.push_str(&format!(
        "- Derived capture: `{}`\n",
        plan.plan.derived_capture_id.as_str()
    ));
    output.push_str(&format!(
        "- Project revision: {} → {}\n\n",
        plan.plan.expected_project_revision,
        plan.plan.expected_project_revision + 1
    ));
    for index in indexes {
        let item = items
            .get(*index)
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let selection = selections
            .get(*index)
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        if selection.item_id != item.item.item_id {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        output.push_str(&format!(
            "### {} `{}`\n\n",
            item_kind_name(item.item.kind),
            item.item.item_id.as_str()
        ));
        output.push_str(&format!(
            "- Disposition: `{}`\n",
            disposition_name(selection.disposition)
        ));
        output.push_str(&format!(
            "- Source: {}\n",
            escape_markdown(
                &serde_json::to_string(&item.source)
                    .map_err(|_| { ProjectError::InvalidResolutionDocument })?
            )
        ));
        if let Some(current) = &item.current {
            output.push_str(&format!(
                "- Current: {}\n",
                escape_markdown(
                    &serde_json::to_string(current)
                        .map_err(|_| { ProjectError::InvalidResolutionDocument })?
                )
            ));
        } else {
            output.push_str("- Current: not present in exact typed lineage\n");
        }
        output.push('\n');
    }
    output.push_str(&format!("<!-- qiongli:resolution-plan {digest} end -->\n"));
    Ok(output)
}

fn predicted_resulting_observations(
    root: &Path,
    observed: &[CaptureResolutionArtifactObservationV1],
    planned_artifacts: &[PlannedArtifact],
    derived_capture: &ResearchCaptureV1,
) -> Result<Vec<CaptureResolutionArtifactObservationV1>, ProjectError> {
    let mut resulting = observed.to_vec();
    for planned in planned_artifacts {
        let observation = resulting
            .iter_mut()
            .find(|observation| observation.artifact == planned.artifact)
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        observation.sha256 = Some(sha256_bytes(&planned.next_bytes));
    }
    let capture_history = predicted_capture_history_digest(root, derived_capture)?;
    let observation = resulting
        .iter_mut()
        .find(|observation| observation.artifact == CaptureResolutionArtifact::CaptureHistory)
        .ok_or(ProjectError::InvalidResolutionDocument)?;
    observation.sha256 = Some(capture_history);
    Ok(resulting)
}

fn predicted_capture_history_digest(
    root: &Path,
    derived_capture: &ResearchCaptureV1,
) -> Result<String, ProjectError> {
    let mut entries = list_capture_documents(root)?
        .into_iter()
        .map(|(capture, digest)| (capture.capture_id.as_str().to_string(), digest))
        .collect::<Vec<_>>();
    let derived_digest = sha256_bytes(&derived_capture.to_canonical_json()?);
    match entries
        .iter()
        .find(|(capture_id, _)| capture_id == derived_capture.capture_id.as_str())
    {
        Some((_, digest)) if digest == &derived_digest => {}
        Some(_) => return Err(ProjectError::CaptureIdentityConflict),
        None => entries.push((
            derived_capture.capture_id.as_str().to_string(),
            derived_digest,
        )),
    }
    entries.sort();
    let bytes = serde_json_canonicalizer::to_vec(&entries)
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    Ok(sha256_bytes(&bytes))
}

fn add_capture_lineage_update(
    root: &Path,
    capture: &ResearchCaptureV1,
    updates: &mut Vec<ProjectFileUpdate>,
) -> Result<(), ProjectError> {
    match read_capture_document(root, &capture.capture_id)? {
        Some((existing, digest))
            if existing == *capture && digest == sha256_bytes(&capture.to_canonical_json()?) => {}
        Some(_) => return Err(ProjectError::CaptureIdentityConflict),
        None => updates.push(ProjectFileUpdate {
            relative_path: capture_history_relative_path(&capture.capture_id),
            expected_digest: None,
            next_bytes: capture.to_canonical_json()?,
        }),
    }
    Ok(())
}

fn add_assignment_lineage_update(
    root: &Path,
    receipt: &CaptureAssignmentReceiptV1,
    updates: &mut Vec<ProjectFileUpdate>,
) -> Result<(), ProjectError> {
    let bytes = receipt.to_canonical_json()?;
    match read_assignment_receipt_document(root, &receipt.receipt_id)? {
        Some(existing) if existing == bytes => {}
        Some(_) => return Err(ProjectError::ResolutionIdentityConflict),
        None => updates.push(ProjectFileUpdate {
            relative_path: assignment_receipt_relative_path(&receipt.receipt_id),
            expected_digest: None,
            next_bytes: bytes,
        }),
    }
    Ok(())
}

fn resolution_for_assignment(
    root: &Path,
    assignment_receipt_id: &CaptureAssignmentReceiptId,
) -> Result<Option<CaptureResolutionReceiptV1>, ProjectError> {
    let mut matches = list_resolution_receipts(root)?
        .into_iter()
        .filter(|receipt| &receipt.receipt.assignment_receipt_id == assignment_receipt_id);
    let result = matches.next();
    if matches.next().is_some() {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    Ok(result)
}

fn list_resolution_receipts(root: &Path) -> Result<Vec<CaptureResolutionReceiptV1>, ProjectError> {
    let mut receipts = list_resolution_receipt_documents(root)?
        .into_iter()
        .map(|(receipt_id, bytes)| {
            let receipt = CaptureResolutionReceiptV1::from_json_slice(&bytes)?;
            if receipt.receipt_id != receipt_id {
                return Err(ProjectError::ResolutionIdentityConflict);
            }
            validate_project_resolution_lineage(root, &receipt)?;
            Ok(receipt)
        })
        .collect::<Result<Vec<_>, _>>()?;
    receipts.sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
    Ok(receipts)
}

fn validate_project_resolution_lineage(
    root: &Path,
    receipt: &CaptureResolutionReceiptV1,
) -> Result<(), ProjectError> {
    receipt.validate()?;
    let assignment_bytes =
        read_assignment_receipt_document(root, &receipt.receipt.assignment_receipt_id)?
            .ok_or(ProjectError::ResolutionIdentityConflict)?;
    let assignment = CaptureAssignmentReceiptV1::from_json_slice(&assignment_bytes)?;
    let assignment_sha256 = sha256_bytes(&assignment_bytes);
    if assignment.receipt_id != receipt.receipt.assignment_receipt_id
        || assignment_sha256 != receipt.receipt.assignment_receipt_sha256
        || assignment.receipt.source_envelope_id != receipt.receipt.source_envelope_id
        || assignment.receipt.source_envelope_sha256 != receipt.receipt.source_envelope_sha256
        || assignment.receipt.source_capture_id != receipt.receipt.source_capture_id
        || assignment.receipt.source_capture_sha256 != receipt.receipt.source_capture_sha256
        || assignment.receipt.target_project_id != receipt.receipt.target_project_id
        || assignment.receipt.result.derived_capture_id.as_ref()
            != Some(&receipt.receipt.derived_capture_id)
        || assignment.receipt.result.derived_capture_sha256.as_deref()
            != Some(receipt.receipt.derived_capture_sha256.as_str())
        || assignment.receipt.result.child_envelope_id.as_ref()
            != Some(&receipt.receipt.child_envelope_id)
    {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    let (capture, capture_sha256) =
        read_capture_document(root, &receipt.receipt.derived_capture_id)?
            .ok_or(ProjectError::ResolutionIdentityConflict)?;
    if capture_sha256 != receipt.receipt.derived_capture_sha256
        || capture.binding.project_id != receipt.receipt.target_project_id
    {
        return Err(ProjectError::ResolutionIdentityConflict);
    }
    Ok(())
}

#[cfg(test)]
fn inject_delivery_interruption(boundary: u8) -> Result<(), ProjectError> {
    DELIVERY_INTERRUPTION_BOUNDARY.with(|selected| {
        if selected.get() == boundary {
            selected.set(0);
            Err(ProjectError::RecoveryRequired)
        } else {
            Ok(())
        }
    })
}

#[cfg(not(test))]
const fn inject_delivery_interruption(_boundary: u8) -> Result<(), ProjectError> {
    Ok(())
}

fn validate_selections(
    plan: &CaptureResolutionPlanV1,
    selections: &[CaptureResolutionSelectionV1],
) -> Result<(), ProjectError> {
    if selections.len() != plan.plan.items.len() {
        return Err(ProjectError::InvalidResolutionDocument);
    }
    for (item, selection) in plan.plan.items.iter().zip(selections) {
        if item.item_id != selection.item_id
            || !item.allowed_dispositions.contains(&selection.disposition)
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
    }
    Ok(())
}

fn resolution_commit(
    receipt: &CaptureResolutionReceiptV1,
    selections: &CaptureResolutionSelectionSetV1,
    library_revision: u64,
    artifacts_updated: Vec<CaptureResolutionArtifact>,
    child_state: CaptureDeliveryState,
    acknowledgement_id: Option<DeliveryAcknowledgementId>,
    exact_replay: bool,
) -> CaptureResolutionCommitV1 {
    CaptureResolutionCommitV1 {
        schema_version: CAPTURE_RESOLUTION_SERVICE_SCHEMA_VERSION,
        receipt_id: receipt.receipt_id.clone(),
        assignment_receipt_id: receipt.receipt.assignment_receipt_id.clone(),
        resolution_plan_digest: receipt.receipt.resolution_plan_digest.clone(),
        selection_digest: selections.selection_digest.clone(),
        target_project_id: receipt.receipt.target_project_id.clone(),
        library_revision,
        from_project_revision: receipt.receipt.from_project_revision,
        to_project_revision: receipt.receipt.to_project_revision,
        artifacts_updated,
        child_envelope_id: receipt.receipt.child_envelope_id.clone(),
        child_state,
        acknowledgement_id,
        index_rebuild_required: true,
        exact_replay,
    }
}

fn changed_artifacts(receipt: &CaptureResolutionReceiptV1) -> Vec<CaptureResolutionArtifact> {
    receipt
        .receipt
        .observed_artifacts
        .iter()
        .zip(&receipt.receipt.resulting_artifacts)
        .filter_map(|(before, after)| (before.sha256 != after.sha256).then_some(before.artifact))
        .filter(|artifact| {
            matches!(
                artifact,
                CaptureResolutionArtifact::ResearchState | CaptureResolutionArtifact::DecisionLog
            )
        })
        .collect()
}

fn library_observation_matches(
    document: &crate::model::ResearchLibraryDocumentV1,
    expected_revision: u64,
    expected_entry: &RegisteredProjectV1,
) -> bool {
    document.revision == expected_revision
        && document
            .projects
            .iter()
            .find(|entry| entry.project_id == expected_entry.project_id)
            == Some(expected_entry)
}

fn artifact_relative_path(artifact: CaptureResolutionArtifact) -> Option<&'static str> {
    match artifact {
        CaptureResolutionArtifact::ResearchState => Some(RESEARCH_STATE_PATH),
        CaptureResolutionArtifact::DecisionLog => Some(DECISION_LOG_PATH),
        CaptureResolutionArtifact::CaptureHistory
        | CaptureResolutionArtifact::ConsolidationHistory => None,
    }
}

fn prepare_document(previous: &str, heading: &str) -> String {
    let mut output = if previous.is_empty() {
        format!("{heading}\n")
    } else {
        previous.to_string()
    };
    if !output.ends_with('\n') {
        output.push('\n');
    }
    if !output.ends_with("\n\n") {
        output.push('\n');
    }
    output
}

fn escape_markdown(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '\\' | '*' | '_' | '{' | '}' | '[' | ']' | '<' | '>' | '#' | '|' | '`'
        ) {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

const fn item_kind_name(kind: CaptureResolutionItemKind) -> &'static str {
    match kind {
        CaptureResolutionItemKind::SemanticChange => "semantic change",
        CaptureResolutionItemKind::Decision => "decision",
        CaptureResolutionItemKind::Evidence => "evidence",
        CaptureResolutionItemKind::Contradiction => "contradiction",
        CaptureResolutionItemKind::NextAction => "next action",
    }
}

const fn disposition_name(disposition: CaptureResolutionDisposition) -> &'static str {
    match disposition {
        CaptureResolutionDisposition::AcceptCurrent => "accept-current",
        CaptureResolutionDisposition::AcceptCapture => "accept-capture",
        CaptureResolutionDisposition::RetainBoth => "retain-both",
        CaptureResolutionDisposition::RejectCapture => "reject-capture",
    }
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    Ok(sha256_bytes(&bytes))
}

fn validate_resolution_timestamp(timestamp: u64) -> Result<(), ProjectError> {
    if timestamp > MAX_SEMANTIC_REVISION {
        Err(ProjectError::InvalidResolutionDocument)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::{ConfigRoot, resolve_config_root};

    use crate::{
        ApprovedCaptureAssignment, ApprovedProjectMutation, CaptureArea, CaptureAssignmentCommitV1,
        CaptureAssignmentDecision, CaptureDelivery, CaptureDeliveryEnvelopeV1, CapturePolicy,
        CaptureSource, DecisionRelation, EvidenceLocatorKind, IncrementalPortfolioService,
        ProjectBindingV1, ProjectKind, ProjectRegistrationOptions, ProjectStage,
        ResearchCaptureDraftV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config_root: ConfigRoot,
        target_root: PathBuf,
        service: ProjectStateService,
        source_project_id: ProjectId,
        target_project_id: ProjectId,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "qiongli-capture-resolution-service-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            let projects = root.join("projects");
            fs::create_dir(&home).unwrap();
            fs::create_dir(&projects).unwrap();
            let config_root = resolve_config_root(None, &home).unwrap();
            let service = ProjectStateService::new(config_root.clone());
            let source_project_id =
                ProjectId::parse("prj_55555555555555555555555555555555").unwrap();
            let target_project_id =
                ProjectId::parse("prj_66666666666666666666666666666666").unwrap();
            create_project(
                &service,
                &projects.join("source"),
                source_project_id.clone(),
                "Resolution source",
                100,
            );
            let target_root = projects.join("target");
            create_project(
                &service,
                &target_root,
                target_project_id.clone(),
                "Resolution target",
                200,
            );
            Self {
                root,
                config_root,
                target_root,
                service,
                source_project_id,
                target_project_id,
            }
        }

        fn capture(&self, variant: u8, captured_at_unix: u64) -> ResearchCaptureV1 {
            let suffix = if variant == 1 { "baseline" } else { "revised" };
            ResearchCaptureDraftV1 {
                binding: ProjectBindingV1::new(
                    self.source_project_id.clone(),
                    1,
                    ProjectStage::Literature,
                    format!("Resolve {suffix} capture"),
                    CapturePolicy::ReviewRequired,
                )
                .unwrap(),
                source: CaptureSource::Codex,
                delivery: CaptureDelivery::Connected,
                captured_at_unix,
                summary: format!("Resolution {suffix} summary."),
                changes: vec![SemanticChangeV1 {
                    area: CaptureArea::Method,
                    summary: format!("{suffix} method specification."),
                }],
                decisions: vec![DecisionCandidateV1 {
                    relation: DecisionRelation::Candidate,
                    statement: "Use a panel model.".to_string(),
                    rationale: format!("{suffix} decision rationale."),
                    target: None,
                }],
                evidence: vec![EvidenceReferenceV1 {
                    locator_kind: EvidenceLocatorKind::Doi,
                    locator: "10.1000/resolution-shared".to_string(),
                    relevance: format!("{suffix} evidence relevance."),
                    limitation: None,
                }],
                contradictions: vec![ContradictionV1 {
                    statement: "The estimates disagree.".to_string(),
                    conflicts_with: "The registered baseline.".to_string(),
                    consequence: format!("{suffix} contradiction consequence."),
                }],
                next_actions: vec!["Run the shared robustness check.".to_string()],
            }
            .into_capture()
            .unwrap()
        }

        fn assign(
            &self,
            capture: ResearchCaptureV1,
            envelope_at_unix: u64,
            decided_at_unix: u64,
        ) -> CaptureAssignmentCommitV1 {
            let envelope = CaptureDeliveryEnvelopeV1::new(capture, None, envelope_at_unix).unwrap();
            self.service
                .enqueue_capture_delivery(envelope.clone())
                .unwrap();
            let assignment = self
                .service
                .preview_capture_assignment(
                    &envelope.envelope_id,
                    &self.target_project_id,
                    CaptureAssignmentDecision::Assign,
                    decided_at_unix,
                )
                .unwrap();
            self.service
                .apply_capture_assignment(
                    &assignment,
                    &ApprovedCaptureAssignment::new(assignment.preview().plan_digest.clone(), true),
                )
                .unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn create_project(
        service: &ProjectStateService,
        root: &Path,
        project_id: ProjectId,
        display_name: &str,
        now_unix: u64,
    ) {
        let plan = service
            .preview_create(
                root,
                ProjectRegistrationOptions::new(display_name, ProjectKind::Article)
                    .with_project_id(project_id)
                    .with_stage(ProjectStage::Literature),
                now_unix,
            )
            .unwrap();
        service
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                now_unix,
            )
            .unwrap();
    }

    fn selection_set(
        plan: &VerifiedCaptureResolution,
        disposition: impl Fn(CaptureResolutionItemKind) -> CaptureResolutionDisposition,
    ) -> CaptureResolutionSelectionSetV1 {
        CaptureResolutionSelectionSetV1::new(
            plan.resolution_plan(),
            plan.preview()
                .items
                .iter()
                .map(|item| CaptureResolutionSelectionV1 {
                    item_id: item.item.item_id.clone(),
                    disposition: disposition(item.item.kind),
                })
                .collect(),
        )
        .unwrap()
    }

    fn apply_resolution(
        service: &ProjectStateService,
        plan: &VerifiedCaptureResolution,
        selections: &CaptureResolutionSelectionSetV1,
        resolved_at_unix: u64,
    ) -> CaptureResolutionCommitV1 {
        service
            .apply_capture_resolution(
                plan,
                selections,
                &ApprovedCaptureResolution::new(
                    plan.preview().plan_digest.clone(),
                    selections.selection_digest.clone(),
                    true,
                    true,
                ),
                resolved_at_unix,
            )
            .unwrap()
    }

    #[test]
    fn resolution_applies_all_item_kinds_acknowledges_and_replays_after_restart() {
        let fixture = Fixture::new();
        let portfolio = IncrementalPortfolioService::new(fixture.service.clone());
        let baseline = portfolio.reconcile(1_800_000_999).unwrap();
        assert_eq!(baseline.rebuilt_project_count, 2);
        let assignment = fixture.assign(
            fixture.capture(1, 1_800_001_000),
            1_800_001_010,
            1_800_001_020,
        );
        let plan = fixture
            .service
            .preview_capture_resolution(&assignment.receipt_id, 1_800_001_030)
            .unwrap();
        assert_eq!(plan.preview().items.len(), 5);
        assert!(
            plan.preview().items.iter().all(
                |item| item.item.counterpart_state == CaptureResolutionCounterpartState::Absent
            )
        );
        assert_eq!(
            plan.preview()
                .items
                .iter()
                .map(|item| item.item.kind)
                .collect::<Vec<_>>(),
            [
                CaptureResolutionItemKind::SemanticChange,
                CaptureResolutionItemKind::Decision,
                CaptureResolutionItemKind::Evidence,
                CaptureResolutionItemKind::Contradiction,
                CaptureResolutionItemKind::NextAction,
            ]
        );
        let selections = selection_set(&plan, |_| CaptureResolutionDisposition::AcceptCapture);
        let commit = apply_resolution(&fixture.service, &plan, &selections, 1_800_001_040);
        assert_eq!(commit.from_project_revision, 1);
        assert_eq!(commit.to_project_revision, 2);
        assert_eq!(commit.child_state, CaptureDeliveryState::Acknowledged);
        assert!(commit.acknowledgement_id.is_some());
        assert!(!commit.exact_replay);
        let reconciled = portfolio.reconcile(1_800_001_041).unwrap();
        assert_eq!(
            reconciled.rebuilt_project_ids,
            vec![fixture.target_project_id.clone()]
        );
        assert_eq!(
            reconciled.reused_project_ids,
            vec![fixture.source_project_id.clone()]
        );
        assert_eq!(
            fixture
                .service
                .inspect_capture_delivery(&assignment.child_envelope_id.clone().unwrap())
                .unwrap()
                .unwrap()
                .state,
            CaptureDeliveryState::Acknowledged
        );
        let derived_capture_id = assignment.derived_capture_id.clone().unwrap();
        assert!(
            fixture
                .service
                .read_capture(&fixture.target_project_id, &derived_capture_id)
                .unwrap()
                .is_some()
        );
        assert!(
            read_assignment_receipt_document(&fixture.target_root, &assignment.receipt_id)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            fixture
                .service
                .inspect_capture_resolution(&fixture.target_project_id, &commit.receipt_id)
                .unwrap()
                .unwrap()
                .receipt_id,
            commit.receipt_id
        );
        assert_eq!(
            fixture
                .service
                .list_capture_resolutions(&fixture.target_project_id)
                .unwrap()
                .len(),
            1
        );
        let research_state =
            fs::read_to_string(fixture.target_root.join(RESEARCH_STATE_PATH)).unwrap();
        let decision_log = fs::read_to_string(fixture.target_root.join(DECISION_LOG_PATH)).unwrap();
        assert!(research_state.contains("semantic change"));
        assert!(research_state.contains("contradiction"));
        assert!(research_state.contains("accept-capture"));
        assert!(decision_log.contains("decision"));
        assert!(decision_log.contains("accept-capture"));

        let before_replay = fs::read(fixture.target_root.join(PROJECT_MANIFEST_PATH)).unwrap();
        let restarted = ProjectStateService::new(fixture.config_root.clone());
        let replay = restarted
            .preview_capture_resolution(&assignment.receipt_id, 1_800_001_030)
            .unwrap();
        assert!(replay.preview().exact_replay);
        assert_eq!(replay.preview().plan_digest, plan.preview().plan_digest);
        let replay_selections =
            selection_set(&replay, |_| CaptureResolutionDisposition::AcceptCapture);
        assert_eq!(
            replay_selections.selection_digest,
            selections.selection_digest
        );
        let replay_commit =
            apply_resolution(&restarted, &replay, &replay_selections, 1_800_001_040);
        assert!(replay_commit.exact_replay);
        assert_eq!(replay_commit.receipt_id, commit.receipt_id);
        assert_eq!(
            fs::read(fixture.target_root.join(PROJECT_MANIFEST_PATH)).unwrap(),
            before_replay
        );
        assert_eq!(
            restarted
                .list_capture_resolutions(&fixture.target_project_id)
                .unwrap()
                .len(),
            1
        );

        let package = fixture.root.join("resolved-portable-package");
        let export = restarted
            .preview_export(&fixture.target_project_id, &package)
            .unwrap();
        restarted
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                1_800_001_050,
            )
            .unwrap();
        let imported_home = fixture.root.join("imported-home");
        fs::create_dir(&imported_home).unwrap();
        let imported_service =
            ProjectStateService::new(resolve_config_root(None, &imported_home).unwrap());
        let imported_root = fixture.root.join("imported-resolution-project");
        let import = imported_service
            .preview_import(&package, &imported_root)
            .unwrap();
        imported_service
            .apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                1_800_001_060,
            )
            .unwrap();
        assert_eq!(
            imported_service
                .list_capture_resolutions(&fixture.target_project_id)
                .unwrap()
                .len(),
            1
        );
        assert!(
            imported_service
                .read_capture(&fixture.target_project_id, &derived_capture_id)
                .unwrap()
                .is_some()
        );
        let public_text = format!(
            "{}\n{}\n{replay:?}",
            serde_json::to_string(plan.preview()).unwrap(),
            serde_json::to_string(&commit).unwrap(),
        );
        assert!(!public_text.contains(fixture.root.to_str().unwrap()));
    }

    #[test]
    fn divergent_resolution_supports_every_frozen_disposition() {
        let fixture = Fixture::new();
        let first_assignment = fixture.assign(
            fixture.capture(1, 1_800_002_000),
            1_800_002_010,
            1_800_002_020,
        );
        let first = fixture
            .service
            .preview_capture_resolution(&first_assignment.receipt_id, 1_800_002_030)
            .unwrap();
        let first_selections =
            selection_set(&first, |_| CaptureResolutionDisposition::AcceptCapture);
        apply_resolution(&fixture.service, &first, &first_selections, 1_800_002_040);

        let second_assignment = fixture.assign(
            fixture.capture(2, 1_800_002_100),
            1_800_002_110,
            1_800_002_120,
        );
        let second = fixture
            .service
            .preview_capture_resolution(&second_assignment.receipt_id, 1_800_002_130)
            .unwrap();
        assert_eq!(
            second
                .preview()
                .items
                .iter()
                .map(|item| item.item.counterpart_state)
                .collect::<Vec<_>>(),
            [
                CaptureResolutionCounterpartState::ExactIdentityDivergent,
                CaptureResolutionCounterpartState::ExactIdentityDivergent,
                CaptureResolutionCounterpartState::ExactIdentityDivergent,
                CaptureResolutionCounterpartState::ExactIdentityDivergent,
                CaptureResolutionCounterpartState::ExactMatch,
            ]
        );
        let selections = selection_set(&second, |kind| match kind {
            CaptureResolutionItemKind::SemanticChange => {
                CaptureResolutionDisposition::AcceptCurrent
            }
            CaptureResolutionItemKind::Decision => CaptureResolutionDisposition::RetainBoth,
            CaptureResolutionItemKind::Evidence => CaptureResolutionDisposition::AcceptCapture,
            CaptureResolutionItemKind::Contradiction => CaptureResolutionDisposition::RetainBoth,
            CaptureResolutionItemKind::NextAction => CaptureResolutionDisposition::RejectCapture,
        });
        let commit = apply_resolution(&fixture.service, &second, &selections, 1_800_002_140);
        assert_eq!(commit.from_project_revision, 2);
        assert_eq!(commit.to_project_revision, 3);
        assert_eq!(commit.child_state, CaptureDeliveryState::Acknowledged);
        let receipt = fixture
            .service
            .inspect_capture_resolution(&fixture.target_project_id, &commit.receipt_id)
            .unwrap()
            .unwrap();
        let dispositions = receipt
            .receipt
            .decisions
            .iter()
            .map(|decision| decision.disposition)
            .collect::<Vec<_>>();
        for disposition in [
            CaptureResolutionDisposition::AcceptCurrent,
            CaptureResolutionDisposition::AcceptCapture,
            CaptureResolutionDisposition::RetainBoth,
            CaptureResolutionDisposition::RejectCapture,
        ] {
            assert!(dispositions.contains(&disposition));
        }
        let research_state =
            fs::read_to_string(fixture.target_root.join(RESEARCH_STATE_PATH)).unwrap();
        assert!(research_state.contains("accept-current"));
        assert!(research_state.contains("accept-capture"));
        assert!(research_state.contains("retain-both"));
        assert!(research_state.contains("reject-capture"));
    }

    #[test]
    fn invalid_selection_approval_and_artifact_drift_fail_before_resolution_write() {
        let fixture = Fixture::new();
        let assignment = fixture.assign(
            fixture.capture(1, 1_800_003_000),
            1_800_003_010,
            1_800_003_020,
        );
        let plan = fixture
            .service
            .preview_capture_resolution(&assignment.receipt_id, 1_800_003_030)
            .unwrap();
        let unsupported = plan
            .preview()
            .items
            .iter()
            .map(|item| CaptureResolutionSelectionV1 {
                item_id: item.item.item_id.clone(),
                disposition: CaptureResolutionDisposition::RetainBoth,
            })
            .collect();
        assert_eq!(
            CaptureResolutionSelectionSetV1::new(plan.resolution_plan(), unsupported).unwrap_err(),
            ProjectError::InvalidResolutionDocument
        );
        let selections = selection_set(&plan, |_| CaptureResolutionDisposition::AcceptCapture);
        assert_eq!(
            fixture.service.apply_capture_resolution(
                &plan,
                &selections,
                &ApprovedCaptureResolution::new(
                    plan.preview().plan_digest.clone(),
                    selections.selection_digest.clone(),
                    false,
                    true,
                ),
                1_800_003_040,
            ),
            Err(ProjectError::ApprovalRequired)
        );
        let mut changed_selections = selections.clone();
        changed_selections.selection_digest = "0".repeat(64);
        assert_eq!(
            fixture.service.apply_capture_resolution(
                &plan,
                &changed_selections,
                &ApprovedCaptureResolution::new(
                    plan.preview().plan_digest.clone(),
                    changed_selections.selection_digest.clone(),
                    true,
                    true,
                ),
                1_800_003_040,
            ),
            Err(ProjectError::PlanMismatch)
        );

        let manifest_before = fs::read(fixture.target_root.join(PROJECT_MANIFEST_PATH)).unwrap();
        fs::write(
            fixture.target_root.join(RESEARCH_STATE_PATH),
            "# Externally changed research state\n",
        )
        .unwrap();
        assert_eq!(
            fixture.service.apply_capture_resolution(
                &plan,
                &selections,
                &ApprovedCaptureResolution::new(
                    plan.preview().plan_digest.clone(),
                    selections.selection_digest.clone(),
                    true,
                    true,
                ),
                1_800_003_040,
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert_eq!(
            fs::read(fixture.target_root.join(PROJECT_MANIFEST_PATH)).unwrap(),
            manifest_before
        );
        assert!(
            fixture
                .service
                .list_capture_resolutions(&fixture.target_project_id)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            fixture
                .service
                .inspect_capture_delivery(&assignment.child_envelope_id.unwrap())
                .unwrap()
                .unwrap()
                .state,
            CaptureDeliveryState::Queued
        );
    }

    #[test]
    fn rejected_or_missing_assignment_cannot_enter_academic_resolution() {
        let fixture = Fixture::new();
        let capture = fixture.capture(1, 1_800_004_000);
        let envelope = CaptureDeliveryEnvelopeV1::new(capture, None, 1_800_004_010).unwrap();
        fixture
            .service
            .enqueue_capture_delivery(envelope.clone())
            .unwrap();
        let rejection = fixture
            .service
            .preview_capture_assignment(
                &envelope.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Reject,
                1_800_004_020,
            )
            .unwrap();
        let rejected = fixture
            .service
            .apply_capture_assignment(
                &rejection,
                &ApprovedCaptureAssignment::new(rejection.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(
            fixture
                .service
                .preview_capture_resolution(&rejected.receipt_id, 1_800_004_030)
                .unwrap_err(),
            ProjectError::CaptureResolutionConflict
        );
        let missing = CaptureAssignmentReceiptId::parse(format!("car_{}", "f".repeat(64))).unwrap();
        assert_eq!(
            fixture
                .service
                .preview_capture_resolution(&missing, 1_800_004_030)
                .unwrap_err(),
            ProjectError::CaptureResolutionConflict
        );
    }

    #[test]
    fn duplicate_source_values_keep_distinct_item_identities() {
        let fixture = Fixture::new();
        let capture = fixture.capture(1, 1_800_005_000);
        let duplicated = ResearchCaptureDraftV1 {
            binding: capture.binding,
            source: capture.source,
            delivery: capture.delivery,
            captured_at_unix: capture.captured_at_unix,
            summary: capture.summary,
            changes: capture.changes,
            decisions: capture.decisions,
            evidence: capture.evidence,
            contradictions: capture.contradictions,
            next_actions: vec![
                "Repeat the exact bounded action.".to_string(),
                "Repeat the exact bounded action.".to_string(),
            ],
        }
        .into_capture()
        .unwrap();
        let envelope =
            CaptureDeliveryEnvelopeV1::new(duplicated.clone(), None, 1_800_005_010).unwrap();
        let items = compare_capture_items(&duplicated, &envelope.envelope_id, &CurrentItems::new())
            .unwrap();
        let next_actions = items
            .iter()
            .filter(|item| item.item.kind == CaptureResolutionItemKind::NextAction)
            .collect::<Vec<_>>();
        assert_eq!(next_actions.len(), 2);
        assert_eq!(
            next_actions[0].item.source_item_sha256,
            next_actions[1].item.source_item_sha256
        );
        assert_ne!(next_actions[0].item.item_id, next_actions[1].item.item_id);
        assert_eq!(next_actions[0].item.source_index, 0);
        assert_eq!(next_actions[1].item.source_index, 1);
    }

    #[test]
    fn delivery_interruption_after_each_durable_transition_replays_to_acknowledged() {
        for boundary in [1, 2] {
            let fixture = Fixture::new();
            let base = 1_800_006_000 + u64::from(boundary) * 100;
            let assignment = fixture.assign(fixture.capture(1, base), base + 10, base + 20);
            let plan = fixture
                .service
                .preview_capture_resolution(&assignment.receipt_id, base + 30)
                .unwrap();
            let selections = selection_set(&plan, |_| CaptureResolutionDisposition::AcceptCapture);
            DELIVERY_INTERRUPTION_BOUNDARY.with(|selected| selected.set(boundary));
            assert_eq!(
                fixture.service.apply_capture_resolution(
                    &plan,
                    &selections,
                    &ApprovedCaptureResolution::new(
                        plan.preview().plan_digest.clone(),
                        selections.selection_digest.clone(),
                        true,
                        true,
                    ),
                    base + 40,
                ),
                Err(ProjectError::RecoveryRequired)
            );
            let interrupted = fixture
                .service
                .inspect_capture_delivery(&assignment.child_envelope_id.clone().unwrap())
                .unwrap()
                .unwrap();
            assert_eq!(
                interrupted.state,
                if boundary == 1 {
                    CaptureDeliveryState::Delivering
                } else {
                    CaptureDeliveryState::Delivered
                }
            );
            assert_eq!(
                fixture
                    .service
                    .list_capture_resolutions(&fixture.target_project_id)
                    .unwrap()
                    .len(),
                1
            );

            let restarted = ProjectStateService::new(fixture.config_root.clone());
            let replay = restarted
                .preview_capture_resolution(&assignment.receipt_id, base + 30)
                .unwrap();
            assert!(replay.preview().exact_replay);
            let replay_selections =
                selection_set(&replay, |_| CaptureResolutionDisposition::AcceptCapture);
            let commit = apply_resolution(&restarted, &replay, &replay_selections, base + 40);
            assert!(commit.exact_replay);
            assert_eq!(commit.child_state, CaptureDeliveryState::Acknowledged);
            assert_eq!(
                restarted
                    .inspect_capture_delivery(&assignment.child_envelope_id.unwrap())
                    .unwrap()
                    .unwrap()
                    .state,
                CaptureDeliveryState::Acknowledged
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn project_lineage_reader_rejects_hardlinked_assignment_receipt() {
        let fixture = Fixture::new();
        let assignment = fixture.assign(
            fixture.capture(1, 1_800_007_000),
            1_800_007_010,
            1_800_007_020,
        );
        let plan = fixture
            .service
            .preview_capture_resolution(&assignment.receipt_id, 1_800_007_030)
            .unwrap();
        let selections = selection_set(&plan, |_| CaptureResolutionDisposition::AcceptCapture);
        apply_resolution(&fixture.service, &plan, &selections, 1_800_007_040);
        let assignment_path = fixture
            .target_root
            .join(assignment_receipt_relative_path(&assignment.receipt_id));
        let external = fixture.root.join("hardlinked-assignment.json");
        fs::copy(&assignment_path, &external).unwrap();
        fs::remove_file(&assignment_path).unwrap();
        fs::hard_link(&external, &assignment_path).unwrap();
        assert_eq!(
            fixture
                .service
                .list_capture_resolutions(&fixture.target_project_id),
            Err(ProjectError::UnsafeProjectRoot)
        );
    }
}
