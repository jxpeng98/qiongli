use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;

use serde::Serialize;

use crate::ProjectError;
use crate::ProjectStateService;
use crate::capture::{
    CaptureDisposition, CaptureId, ProjectBindingV1, ResearchCaptureDraftV1, ResearchCaptureV1,
    classify_capture,
};
use crate::capture_delivery::{
    CaptureDeliveryDestinationV1, CaptureDeliveryEnvelopeV1, CaptureDeliveryReason,
    CaptureDeliveryState, DeliveryEnvelopeId,
};
use crate::capture_resolution::{
    CaptureAssignmentIntentBodyV1, CaptureAssignmentIntentId, CaptureAssignmentIntentV1,
    CaptureAssignmentOutcome, CaptureAssignmentReceiptId, CaptureAssignmentReceiptV1,
    CaptureAssignmentResultV1, CaptureResolutionArtifact, CaptureResolutionArtifactObservationV1,
};
use crate::capture_resolution_storage::{CaptureAssignmentTransactionV1, StoredCaptureAssignment};
use crate::model::{MAX_SEMANTIC_REVISION, ProjectId, ProjectLifecycle, ProjectStage};
use crate::storage::{
    list_capture_documents, project_root_from_string, read_consolidation_document, read_manifest,
    read_semantic_artifact, sha256_bytes, validate_existing_project_root,
};

pub const CAPTURE_ASSIGNMENT_SERVICE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureAssignmentDecision {
    Assign,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureAssignmentBindingEffect {
    Direct,
    Rebound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureAssignmentPreviewOutcome {
    Ready,
    Duplicate,
    ResolutionRequired,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureAssignmentStatusState {
    Pending,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureAssignmentPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub intent_id: CaptureAssignmentIntentId,
    pub decision: CaptureAssignmentDecision,
    pub outcome: CaptureAssignmentPreviewOutcome,
    pub binding_effect: CaptureAssignmentBindingEffect,
    pub source_disposition: CaptureDisposition,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_capture_id: CaptureId,
    pub source_record_state: CaptureDeliveryState,
    pub expected_source_generation: u64,
    pub expected_source_record_sha256: String,
    pub target_project_id: ProjectId,
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub target_stage: ProjectStage,
    pub target_manifest_sha256: String,
    pub observed_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub derived_capture_id: Option<CaptureId>,
    pub child_envelope_id: Option<DeliveryEnvelopeId>,
    pub resolution_required: bool,
    pub decided_at_unix: u64,
    pub approvals_required: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedCaptureAssignment {
    preview: CaptureAssignmentPreviewV1,
    transaction: CaptureAssignmentTransactionV1,
    source_capture: ResearchCaptureV1,
    target_root: PathBuf,
}

impl VerifiedCaptureAssignment {
    #[must_use]
    pub const fn preview(&self) -> &CaptureAssignmentPreviewV1 {
        &self.preview
    }
}

impl Debug for VerifiedCaptureAssignment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedCaptureAssignment")
            .field("preview", &self.preview)
            .field("transaction", &"<bounded-assignment-transaction>")
            .field("source_capture", &"<bounded-research-capture>")
            .field("target_root", &"<registered-project-root>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedCaptureAssignment {
    expected_plan_digest: String,
    assignment_write: bool,
}

impl ApprovedCaptureAssignment {
    #[must_use]
    pub fn new(expected_plan_digest: impl Into<String>, assignment_write: bool) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            assignment_write,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureAssignmentCommitV1 {
    pub schema_version: u32,
    pub intent_id: CaptureAssignmentIntentId,
    pub receipt_id: CaptureAssignmentReceiptId,
    pub outcome: CaptureAssignmentOutcome,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_capture_id: CaptureId,
    pub target_project_id: ProjectId,
    pub target_project_revision: u64,
    pub derived_capture_id: Option<CaptureId>,
    pub child_envelope_id: Option<DeliveryEnvelopeId>,
    pub source_state: CaptureDeliveryState,
    pub decided_at_unix: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureAssignmentStatusV1 {
    pub schema_version: u32,
    pub state: CaptureAssignmentStatusState,
    pub intent_id: CaptureAssignmentIntentId,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_capture_id: CaptureId,
    pub target_project_id: ProjectId,
    pub target_project_revision: u64,
    pub outcome: Option<CaptureAssignmentOutcome>,
    pub receipt_id: Option<CaptureAssignmentReceiptId>,
    pub derived_capture_id: Option<CaptureId>,
    pub child_envelope_id: Option<DeliveryEnvelopeId>,
    pub created_at_unix: u64,
    pub decided_at_unix: Option<u64>,
}

struct AssignmentTargetSnapshot {
    root: PathBuf,
    library_revision: u64,
    project_revision: u64,
    stage: ProjectStage,
    manifest_sha256: String,
    observations: Vec<CaptureResolutionArtifactObservationV1>,
}

impl ProjectStateService {
    pub fn preview_capture_assignment(
        &self,
        source_envelope_id: &DeliveryEnvelopeId,
        target_project_id: &ProjectId,
        decision: CaptureAssignmentDecision,
        decided_at_unix: u64,
    ) -> Result<VerifiedCaptureAssignment, ProjectError> {
        validate_assignment_timestamp(decided_at_unix)?;
        let target = self.assignment_target_snapshot(target_project_id)?;
        if let Some(existing) = self.existing_capture_assignment(
            source_envelope_id,
            target_project_id,
            decision,
            decided_at_unix,
            &target,
        )? {
            return Ok(existing);
        }

        let source = self
            .delivery_store
            .read(source_envelope_id)?
            .ok_or(ProjectError::DeliveryNotFound)?;
        if !matches!(
            source.record.state,
            CaptureDeliveryState::Queued
                | CaptureDeliveryState::RetryRequired
                | CaptureDeliveryState::Conflicted
        ) {
            return Err(ProjectError::InvalidDeliveryTransition);
        }
        let source_capture = source.envelope.capture.clone();
        let derived_capture = derive_capture_for_target(
            &source_capture,
            target_project_id,
            target.project_revision,
            target.stage,
        )?;
        let binding_effect = assignment_binding_effect(&source_capture, &derived_capture);
        let duplicate =
            match crate::storage::read_capture_document(&target.root, &derived_capture.capture_id)?
            {
                Some((existing, _)) if existing == derived_capture => true,
                Some(_) => return Err(ProjectError::CaptureIdentityConflict),
                None => false,
            };
        let source_disposition = classify_capture(&source_capture, false);
        let outcome =
            assignment_preview_outcome(decision, binding_effect, source_disposition, duplicate);
        let intent = CaptureAssignmentIntentV1::new(CaptureAssignmentIntentBodyV1 {
            source_envelope_id: source.envelope.envelope_id.clone(),
            source_envelope_sha256: source.envelope_sha256,
            source_record_state: source.record.state,
            source_record_generation: source.record.generation,
            source_record_sha256: source.record_sha256,
            source_capture_id: source.envelope.capture_id.clone(),
            source_capture_sha256: source.envelope.capture_sha256,
            target_project_id: target_project_id.clone(),
            expected_library_revision: target.library_revision,
            expected_project_revision: target.project_revision,
            target_stage: target.stage,
            target_manifest_sha256: target.manifest_sha256.clone(),
            observed_artifacts: target.observations.clone(),
            created_at_unix: decided_at_unix,
        })?;
        let source_record_after = source.record.transition(
            CaptureDeliveryState::Cancelled,
            decided_at_unix,
            CaptureDeliveryReason::DeliveryCancelled,
            None,
        )?;
        let source_record_sha256_after = sha256_bytes(&source_record_after.to_canonical_json()?);
        let (result, child_envelope) = match decision {
            CaptureAssignmentDecision::Assign => {
                let child = CaptureDeliveryEnvelopeV1::new(
                    derived_capture,
                    Some(CaptureDeliveryDestinationV1::new(
                        target_project_id.clone(),
                        target.project_revision,
                    )?),
                    decided_at_unix,
                )?;
                (
                    CaptureAssignmentResultV1::assigned(
                        child.capture_id.clone(),
                        child.capture_sha256.clone(),
                        child.envelope_id.clone(),
                    ),
                    Some(child),
                )
            }
            CaptureAssignmentDecision::Reject => (CaptureAssignmentResultV1::rejected(), None),
        };
        let receipt = CaptureAssignmentReceiptV1::new(
            &intent,
            result,
            source_record_after.generation,
            source_record_sha256_after,
            decided_at_unix,
        )?;
        let transaction = CaptureAssignmentTransactionV1::new(
            intent,
            receipt,
            child_envelope,
            source_record_after,
        )?;
        build_verified_assignment(
            transaction,
            source_capture,
            target.root,
            decision,
            outcome,
            binding_effect,
            source_disposition,
        )
    }

    pub fn apply_capture_assignment(
        &self,
        plan: &VerifiedCaptureAssignment,
        approval: &ApprovedCaptureAssignment,
    ) -> Result<CaptureAssignmentCommitV1, ProjectError> {
        validate_verified_assignment(plan)?;
        if !approval.assignment_write {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview.plan_digest {
            return Err(ProjectError::PlanMismatch);
        }
        let target = self.assignment_target_snapshot(&plan.preview.target_project_id)?;
        let intent = plan.transaction.intent();
        if target.root != plan.target_root
            || target.library_revision != intent.intent.expected_library_revision
            || target.project_revision != intent.intent.expected_project_revision
            || target.stage != intent.intent.target_stage
            || target.manifest_sha256 != intent.intent.target_manifest_sha256
            || target.observations != intent.intent.observed_artifacts
        {
            return Err(ProjectError::RevisionConflict);
        }
        let stored = self.resolution_store.commit_assignment(&plan.transaction)?;
        assignment_commit(&stored)
    }

    pub fn inspect_capture_assignment(
        &self,
        intent_id: &CaptureAssignmentIntentId,
    ) -> Result<Option<CaptureAssignmentStatusV1>, ProjectError> {
        self.resolution_store
            .read_assignment(intent_id)
            .map(|assignment| assignment.map(assignment_status))
    }

    pub fn list_capture_assignments(&self) -> Result<Vec<CaptureAssignmentStatusV1>, ProjectError> {
        self.resolution_store.rebuild().map(|snapshot| {
            snapshot
                .assignments
                .into_iter()
                .map(assignment_status)
                .collect()
        })
    }

    fn existing_capture_assignment(
        &self,
        source_envelope_id: &DeliveryEnvelopeId,
        target_project_id: &ProjectId,
        decision: CaptureAssignmentDecision,
        decided_at_unix: u64,
        target: &AssignmentTargetSnapshot,
    ) -> Result<Option<VerifiedCaptureAssignment>, ProjectError> {
        let assignment = self
            .resolution_store
            .rebuild()?
            .assignments
            .into_iter()
            .find(|assignment| {
                assignment.intent.intent.source_envelope_id == *source_envelope_id
                    && assignment.receipt.is_some()
            });
        let Some(assignment) = assignment else {
            return Ok(None);
        };
        let receipt = assignment
            .receipt
            .clone()
            .ok_or(ProjectError::RecoveryRequired)?;
        let expected_outcome = match decision {
            CaptureAssignmentDecision::Assign => CaptureAssignmentOutcome::Assigned,
            CaptureAssignmentDecision::Reject => CaptureAssignmentOutcome::Rejected,
        };
        if assignment.intent.intent.target_project_id != *target_project_id
            || receipt.receipt.result.outcome != expected_outcome
            || receipt.receipt.decided_at_unix != decided_at_unix
            || target.library_revision != assignment.intent.intent.expected_library_revision
            || target.project_revision != assignment.intent.intent.expected_project_revision
            || target.stage != assignment.intent.intent.target_stage
            || target.manifest_sha256 != assignment.intent.intent.target_manifest_sha256
            || target.observations != assignment.intent.intent.observed_artifacts
        {
            return Err(ProjectError::RevisionConflict);
        }
        let source = self
            .delivery_store
            .read(source_envelope_id)?
            .ok_or(ProjectError::DeliveryNotFound)?;
        let child_envelope = match receipt.receipt.result.outcome {
            CaptureAssignmentOutcome::Assigned => {
                let child_id = receipt
                    .receipt
                    .result
                    .child_envelope_id
                    .as_ref()
                    .ok_or(ProjectError::RecoveryRequired)?;
                Some(
                    self.delivery_store
                        .read(child_id)?
                        .ok_or(ProjectError::DeliveryNotFound)?
                        .envelope,
                )
            }
            CaptureAssignmentOutcome::Rejected => None,
        };
        let transaction = CaptureAssignmentTransactionV1::new(
            assignment.intent,
            receipt,
            child_envelope,
            source.record,
        )?;
        let source_capture = source.envelope.capture;
        let derived_capture = derive_capture_for_target(
            &source_capture,
            target_project_id,
            target.project_revision,
            target.stage,
        )?;
        let binding_effect = assignment_binding_effect(&source_capture, &derived_capture);
        let duplicate =
            match crate::storage::read_capture_document(&target.root, &derived_capture.capture_id)?
            {
                Some((existing, _)) if existing == derived_capture => true,
                Some(_) => return Err(ProjectError::CaptureIdentityConflict),
                None => false,
            };
        let source_disposition = classify_capture(&source_capture, false);
        let outcome =
            assignment_preview_outcome(decision, binding_effect, source_disposition, duplicate);
        build_verified_assignment(
            transaction,
            source_capture,
            target.root.clone(),
            decision,
            outcome,
            binding_effect,
            source_disposition,
        )
        .map(Some)
    }

    fn assignment_target_snapshot(
        &self,
        target_project_id: &ProjectId,
    ) -> Result<AssignmentTargetSnapshot, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == target_project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        if entry.lifecycle != ProjectLifecycle::Active {
            return Err(ProjectError::RevisionConflict);
        }
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        let (manifest, manifest_sha256) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *target_project_id
            || manifest.lifecycle != ProjectLifecycle::Active
            || manifest.project_kind != entry.project_kind
            || manifest.stage != entry.stage
            || manifest.semantic_revision != entry.semantic_revision
            || manifest.semantic_digest != entry.semantic_digest
            || manifest.display_name != entry.display_name
        {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(AssignmentTargetSnapshot {
            root: root.clone(),
            library_revision: library.revision,
            project_revision: manifest.semantic_revision,
            stage: manifest.stage,
            manifest_sha256,
            observations: assignment_artifact_observations(&root)?,
        })
    }
}

fn build_verified_assignment(
    transaction: CaptureAssignmentTransactionV1,
    source_capture: ResearchCaptureV1,
    target_root: PathBuf,
    decision: CaptureAssignmentDecision,
    outcome: CaptureAssignmentPreviewOutcome,
    binding_effect: CaptureAssignmentBindingEffect,
    source_disposition: CaptureDisposition,
) -> Result<VerifiedCaptureAssignment, ProjectError> {
    transaction.validate()?;
    let intent = transaction.intent();
    let receipt = transaction.receipt();
    let plan_digest = sha256_bytes(&transaction.to_canonical_json()?);
    let preview = CaptureAssignmentPreviewV1 {
        schema_version: CAPTURE_ASSIGNMENT_SERVICE_SCHEMA_VERSION,
        plan_digest,
        intent_id: intent.intent_id.clone(),
        decision,
        outcome,
        binding_effect,
        source_disposition,
        source_envelope_id: intent.intent.source_envelope_id.clone(),
        source_capture_id: intent.intent.source_capture_id.clone(),
        source_record_state: intent.intent.source_record_state,
        expected_source_generation: intent.intent.source_record_generation,
        expected_source_record_sha256: intent.intent.source_record_sha256.clone(),
        target_project_id: intent.intent.target_project_id.clone(),
        expected_library_revision: intent.intent.expected_library_revision,
        expected_project_revision: intent.intent.expected_project_revision,
        target_stage: intent.intent.target_stage,
        target_manifest_sha256: intent.intent.target_manifest_sha256.clone(),
        observed_artifacts: intent.intent.observed_artifacts.clone(),
        derived_capture_id: receipt.receipt.result.derived_capture_id.clone(),
        child_envelope_id: receipt.receipt.result.child_envelope_id.clone(),
        resolution_required: outcome == CaptureAssignmentPreviewOutcome::ResolutionRequired,
        decided_at_unix: receipt.receipt.decided_at_unix,
        approvals_required: vec!["assignment-write".to_string()],
    };
    Ok(VerifiedCaptureAssignment {
        preview,
        transaction,
        source_capture,
        target_root,
    })
}

fn validate_verified_assignment(plan: &VerifiedCaptureAssignment) -> Result<(), ProjectError> {
    plan.transaction.validate()?;
    let intent = plan.transaction.intent();
    let receipt = plan.transaction.receipt();
    if plan.preview.schema_version != CAPTURE_ASSIGNMENT_SERVICE_SCHEMA_VERSION
        || plan.preview.plan_digest != sha256_bytes(&plan.transaction.to_canonical_json()?)
        || plan.preview.intent_id != intent.intent_id
        || plan.preview.source_envelope_id != intent.intent.source_envelope_id
        || plan.preview.source_capture_id != intent.intent.source_capture_id
        || plan.preview.source_record_state != intent.intent.source_record_state
        || plan.preview.expected_source_generation != intent.intent.source_record_generation
        || plan.preview.expected_source_record_sha256 != intent.intent.source_record_sha256
        || plan.preview.target_project_id != intent.intent.target_project_id
        || plan.preview.expected_library_revision != intent.intent.expected_library_revision
        || plan.preview.expected_project_revision != intent.intent.expected_project_revision
        || plan.preview.target_stage != intent.intent.target_stage
        || plan.preview.target_manifest_sha256 != intent.intent.target_manifest_sha256
        || plan.preview.observed_artifacts != intent.intent.observed_artifacts
        || plan.preview.derived_capture_id != receipt.receipt.result.derived_capture_id
        || plan.preview.child_envelope_id != receipt.receipt.result.child_envelope_id
        || plan.preview.decided_at_unix != receipt.receipt.decided_at_unix
        || plan.preview.approvals_required != vec!["assignment-write".to_string()]
    {
        return Err(ProjectError::PlanMismatch);
    }
    let derived = derive_capture_for_target(
        &plan.source_capture,
        &intent.intent.target_project_id,
        intent.intent.expected_project_revision,
        intent.intent.target_stage,
    )?;
    let binding_effect = assignment_binding_effect(&plan.source_capture, &derived);
    let duplicate =
        match crate::storage::read_capture_document(&plan.target_root, &derived.capture_id)? {
            Some((existing, _)) if existing == derived => true,
            Some(_) => return Err(ProjectError::CaptureIdentityConflict),
            None => false,
        };
    let expected_outcome = assignment_preview_outcome(
        plan.preview.decision,
        binding_effect,
        classify_capture(&plan.source_capture, false),
        duplicate,
    );
    if binding_effect != plan.preview.binding_effect
        || classify_capture(&plan.source_capture, false) != plan.preview.source_disposition
        || expected_outcome != plan.preview.outcome
        || (receipt.receipt.result.outcome == CaptureAssignmentOutcome::Rejected)
            != (plan.preview.decision == CaptureAssignmentDecision::Reject)
        || plan.preview.resolution_required
            != (plan.preview.outcome == CaptureAssignmentPreviewOutcome::ResolutionRequired)
    {
        return Err(ProjectError::PlanMismatch);
    }
    match plan.preview.decision {
        CaptureAssignmentDecision::Assign => {
            let child = plan
                .transaction
                .child_envelope()
                .ok_or(ProjectError::PlanMismatch)?;
            if child.capture != derived
                || receipt.receipt.result.derived_capture_id.as_ref() != Some(&derived.capture_id)
            {
                return Err(ProjectError::PlanMismatch);
            }
        }
        CaptureAssignmentDecision::Reject => {
            if plan.transaction.child_envelope().is_some() {
                return Err(ProjectError::PlanMismatch);
            }
        }
    }
    if plan.transaction.source_record_after().state != CaptureDeliveryState::Cancelled {
        return Err(ProjectError::PlanMismatch);
    }
    Ok(())
}

fn derive_capture_for_target(
    source: &ResearchCaptureV1,
    target_project_id: &ProjectId,
    target_project_revision: u64,
    target_stage: ProjectStage,
) -> Result<ResearchCaptureV1, ProjectError> {
    ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            target_project_id.clone(),
            target_project_revision,
            target_stage,
            source.binding.task.clone(),
            source.binding.capture_policy,
        )?,
        source: source.source,
        delivery: source.delivery,
        captured_at_unix: source.captured_at_unix,
        summary: source.summary.clone(),
        changes: source.changes.clone(),
        decisions: source.decisions.clone(),
        evidence: source.evidence.clone(),
        contradictions: source.contradictions.clone(),
        next_actions: source.next_actions.clone(),
    }
    .into_capture()
}

fn assignment_binding_effect(
    source: &ResearchCaptureV1,
    derived: &ResearchCaptureV1,
) -> CaptureAssignmentBindingEffect {
    if source.capture_id == derived.capture_id {
        CaptureAssignmentBindingEffect::Direct
    } else {
        CaptureAssignmentBindingEffect::Rebound
    }
}

fn assignment_preview_outcome(
    decision: CaptureAssignmentDecision,
    binding_effect: CaptureAssignmentBindingEffect,
    disposition: CaptureDisposition,
    duplicate: bool,
) -> CaptureAssignmentPreviewOutcome {
    if decision == CaptureAssignmentDecision::Reject {
        return CaptureAssignmentPreviewOutcome::Rejected;
    }
    if duplicate {
        return CaptureAssignmentPreviewOutcome::Duplicate;
    }
    if binding_effect == CaptureAssignmentBindingEffect::Rebound
        || matches!(
            disposition,
            CaptureDisposition::Contradiction
                | CaptureDisposition::Supersession
                | CaptureDisposition::UnsupportedGap
        )
    {
        CaptureAssignmentPreviewOutcome::ResolutionRequired
    } else {
        CaptureAssignmentPreviewOutcome::Ready
    }
}

fn assignment_artifact_observations(
    root: &std::path::Path,
) -> Result<Vec<CaptureResolutionArtifactObservationV1>, ProjectError> {
    let research_state =
        read_semantic_artifact(root, "context/research_state.md")?.map(|(_, digest)| digest);
    let decision_log =
        read_semantic_artifact(root, "context/decision_log.md")?.map(|(_, digest)| digest);
    let captures = list_capture_documents(root)?;
    let capture_history = aggregate_history_digest(
        captures
            .iter()
            .map(|(capture, digest)| (capture.capture_id.as_str().to_string(), digest.clone()))
            .collect(),
    )?;
    let mut consolidations = Vec::new();
    for (capture, _) in &captures {
        if let Some(bytes) = read_consolidation_document(root, &capture.capture_id)? {
            consolidations.push((
                capture.capture_id.as_str().to_string(),
                sha256_bytes(&bytes),
            ));
        }
    }
    let consolidation_history = aggregate_history_digest(consolidations)?;
    Ok(vec![
        CaptureResolutionArtifactObservationV1::new(
            CaptureResolutionArtifact::ResearchState,
            research_state,
        ),
        CaptureResolutionArtifactObservationV1::new(
            CaptureResolutionArtifact::DecisionLog,
            decision_log,
        ),
        CaptureResolutionArtifactObservationV1::new(
            CaptureResolutionArtifact::CaptureHistory,
            capture_history,
        ),
        CaptureResolutionArtifactObservationV1::new(
            CaptureResolutionArtifact::ConsolidationHistory,
            consolidation_history,
        ),
    ])
}

fn aggregate_history_digest(
    mut entries: Vec<(String, String)>,
) -> Result<Option<String>, ProjectError> {
    if entries.is_empty() {
        return Ok(None);
    }
    entries.sort();
    let bytes = serde_json_canonicalizer::to_vec(&entries)
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    Ok(Some(sha256_bytes(&bytes)))
}

fn assignment_commit(
    stored: &StoredCaptureAssignment,
) -> Result<CaptureAssignmentCommitV1, ProjectError> {
    let receipt = stored
        .receipt
        .as_ref()
        .ok_or(ProjectError::RecoveryRequired)?;
    Ok(CaptureAssignmentCommitV1 {
        schema_version: CAPTURE_ASSIGNMENT_SERVICE_SCHEMA_VERSION,
        intent_id: stored.intent.intent_id.clone(),
        receipt_id: receipt.receipt_id.clone(),
        outcome: receipt.receipt.result.outcome,
        source_envelope_id: receipt.receipt.source_envelope_id.clone(),
        source_capture_id: receipt.receipt.source_capture_id.clone(),
        target_project_id: receipt.receipt.target_project_id.clone(),
        target_project_revision: receipt.receipt.target_project_revision,
        derived_capture_id: receipt.receipt.result.derived_capture_id.clone(),
        child_envelope_id: receipt.receipt.result.child_envelope_id.clone(),
        source_state: CaptureDeliveryState::Cancelled,
        decided_at_unix: receipt.receipt.decided_at_unix,
    })
}

fn assignment_status(stored: StoredCaptureAssignment) -> CaptureAssignmentStatusV1 {
    let receipt = stored.receipt;
    CaptureAssignmentStatusV1 {
        schema_version: CAPTURE_ASSIGNMENT_SERVICE_SCHEMA_VERSION,
        state: if receipt.is_some() {
            CaptureAssignmentStatusState::Completed
        } else {
            CaptureAssignmentStatusState::Pending
        },
        intent_id: stored.intent.intent_id,
        source_envelope_id: stored.intent.intent.source_envelope_id,
        source_capture_id: stored.intent.intent.source_capture_id,
        target_project_id: stored.intent.intent.target_project_id,
        target_project_revision: stored.intent.intent.expected_project_revision,
        outcome: receipt
            .as_ref()
            .map(|receipt| receipt.receipt.result.outcome),
        receipt_id: receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
        derived_capture_id: receipt
            .as_ref()
            .and_then(|receipt| receipt.receipt.result.derived_capture_id.clone()),
        child_envelope_id: receipt
            .as_ref()
            .and_then(|receipt| receipt.receipt.result.child_envelope_id.clone()),
        created_at_unix: stored.intent.intent.created_at_unix,
        decided_at_unix: receipt
            .as_ref()
            .map(|receipt| receipt.receipt.decided_at_unix),
    }
}

fn validate_assignment_timestamp(timestamp: u64) -> Result<(), ProjectError> {
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
        ApprovedCaptureIntake, ApprovedProjectMutation, CaptureArea, CaptureDelivery,
        CaptureDeliveryEnvelopeV1, CapturePolicy, CaptureSource, EvidenceLocatorKind,
        EvidenceReferenceV1, ProjectKind, ProjectRegistrationOptions, ResearchCaptureDraftV1,
        SemanticChangeV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        projects_root: PathBuf,
        config_root: ConfigRoot,
        service: ProjectStateService,
        source_project_id: ProjectId,
        target_project_id: ProjectId,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "qiongli-capture-assignment-service-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            let projects_root = root.join("projects");
            fs::create_dir(&home).unwrap();
            fs::create_dir(&projects_root).unwrap();
            let config_root = resolve_config_root(None, &home).unwrap();
            let service = ProjectStateService::new(config_root.clone());
            let source_project_id =
                ProjectId::parse("prj_11111111111111111111111111111111").unwrap();
            let target_project_id =
                ProjectId::parse("prj_22222222222222222222222222222222").unwrap();
            create_project(
                &service,
                &projects_root.join("source"),
                source_project_id.clone(),
                "Source project",
                100,
            );
            create_project(
                &service,
                &projects_root.join("target"),
                target_project_id.clone(),
                "Target project",
                200,
            );
            Self {
                root,
                projects_root,
                config_root,
                service,
                source_project_id,
                target_project_id,
            }
        }

        fn capture(
            &self,
            project_id: ProjectId,
            suffix: &str,
            captured_at_unix: u64,
        ) -> ResearchCaptureV1 {
            ResearchCaptureDraftV1 {
                binding: ProjectBindingV1::new(
                    project_id,
                    1,
                    ProjectStage::Literature,
                    format!("Assign capture {suffix}"),
                    CapturePolicy::ReviewRequired,
                )
                .unwrap(),
                source: CaptureSource::Codex,
                delivery: CaptureDelivery::Connected,
                captured_at_unix,
                summary: format!("Assignment service summary {suffix}."),
                changes: vec![SemanticChangeV1 {
                    area: CaptureArea::Method,
                    summary: format!("Preserve assignment evidence {suffix}."),
                }],
                decisions: Vec::new(),
                evidence: vec![EvidenceReferenceV1 {
                    locator_kind: EvidenceLocatorKind::Doi,
                    locator: format!("10.1000/assignment-{suffix}"),
                    relevance: "Binds the captured change to exact evidence.".to_string(),
                    limitation: None,
                }],
                contradictions: Vec::new(),
                next_actions: vec!["Review the assigned capture.".to_string()],
            }
            .into_capture()
            .unwrap()
        }

        fn enqueue(
            &self,
            capture: ResearchCaptureV1,
            destination: Option<CaptureDeliveryDestinationV1>,
            created_at_unix: u64,
        ) -> CaptureDeliveryEnvelopeV1 {
            let envelope =
                CaptureDeliveryEnvelopeV1::new(capture, destination, created_at_unix).unwrap();
            self.service
                .enqueue_capture_delivery(envelope.clone())
                .unwrap();
            envelope
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

    fn assert_target_unchanged(
        before: &AssignmentTargetSnapshot,
        after: &AssignmentTargetSnapshot,
    ) {
        assert_eq!(after.root, before.root);
        assert_eq!(after.library_revision, before.library_revision);
        assert_eq!(after.project_revision, before.project_revision);
        assert_eq!(after.stage, before.stage);
        assert_eq!(after.manifest_sha256, before.manifest_sha256);
        assert_eq!(after.observations, before.observations);
    }

    #[test]
    fn rebound_assignment_applies_replays_and_reopens_without_project_mutation() {
        let fixture = Fixture::new();
        let capture = fixture.capture(
            fixture.source_project_id.clone(),
            "rebound-replay",
            1_800_000_000,
        );
        let source = fixture.enqueue(capture.clone(), None, 1_800_000_010);
        let queued = fixture
            .service
            .inspect_capture_delivery(&source.envelope_id)
            .unwrap()
            .unwrap();
        let conflicted = fixture
            .service
            .begin_capture_delivery(
                &source.envelope_id,
                queued.generation,
                &queued.record_sha256,
                1_800_000_011,
            )
            .unwrap();
        assert_eq!(conflicted.state, CaptureDeliveryState::Conflicted);

        let target_before = fixture
            .service
            .assignment_target_snapshot(&fixture.target_project_id)
            .unwrap();
        let plan = fixture
            .service
            .preview_capture_assignment(
                &source.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Assign,
                1_800_000_020,
            )
            .unwrap();
        assert_eq!(
            plan.preview().binding_effect,
            CaptureAssignmentBindingEffect::Rebound
        );
        assert_eq!(
            plan.preview().outcome,
            CaptureAssignmentPreviewOutcome::ResolutionRequired
        );
        assert!(plan.preview().resolution_required);
        assert_ne!(
            plan.preview().derived_capture_id.as_ref(),
            Some(&capture.capture_id)
        );
        let commit = fixture
            .service
            .apply_capture_assignment(
                &plan,
                &ApprovedCaptureAssignment::new(plan.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(commit.outcome, CaptureAssignmentOutcome::Assigned);
        assert_eq!(commit.source_state, CaptureDeliveryState::Cancelled);
        assert_eq!(
            fixture
                .service
                .inspect_capture_delivery(&source.envelope_id)
                .unwrap()
                .unwrap()
                .state,
            CaptureDeliveryState::Cancelled
        );
        let child = fixture
            .service
            .inspect_capture_delivery(commit.child_envelope_id.as_ref().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(child.state, CaptureDeliveryState::Queued);
        assert_eq!(
            child.destination.as_ref().unwrap().project_id,
            fixture.target_project_id
        );
        assert_eq!(
            child
                .destination
                .as_ref()
                .unwrap()
                .expected_project_revision,
            1
        );
        let target_after = fixture
            .service
            .assignment_target_snapshot(&fixture.target_project_id)
            .unwrap();
        assert_target_unchanged(&target_before, &target_after);

        let status = fixture
            .service
            .inspect_capture_assignment(&commit.intent_id)
            .unwrap()
            .unwrap();
        assert_eq!(status.state, CaptureAssignmentStatusState::Completed);
        assert_eq!(status.receipt_id.as_ref(), Some(&commit.receipt_id));
        let restarted = ProjectStateService::new(fixture.config_root.clone());
        let replay_plan = restarted
            .preview_capture_assignment(
                &source.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Assign,
                1_800_000_020,
            )
            .unwrap();
        assert_eq!(replay_plan.preview(), plan.preview());
        let replay_commit = restarted
            .apply_capture_assignment(
                &replay_plan,
                &ApprovedCaptureAssignment::new(replay_plan.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(replay_commit, commit);
        assert_eq!(restarted.list_capture_assignments().unwrap().len(), 1);

        let public_text = format!(
            "{}\n{}\n{}\n{replay_plan:?}",
            serde_json::to_string(plan.preview()).unwrap(),
            serde_json::to_string(&commit).unwrap(),
            serde_json::to_string(&status).unwrap(),
        );
        assert!(!public_text.contains(fixture.root.to_str().unwrap()));
        assert!(!public_text.contains("Assignment service summary rebound-replay."));
    }

    #[test]
    fn direct_duplicate_and_reject_outcomes_keep_exact_lineage() {
        let fixture = Fixture::new();
        let direct_capture = fixture.capture(
            fixture.target_project_id.clone(),
            "direct-ready",
            1_800_000_100,
        );
        let direct_source = fixture.enqueue(direct_capture.clone(), None, 1_800_000_110);
        let direct = fixture
            .service
            .preview_capture_assignment(
                &direct_source.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Assign,
                1_800_000_120,
            )
            .unwrap();
        assert_eq!(
            direct.preview().binding_effect,
            CaptureAssignmentBindingEffect::Direct
        );
        assert_eq!(
            direct.preview().outcome,
            CaptureAssignmentPreviewOutcome::Ready
        );
        assert_eq!(
            direct.preview().derived_capture_id.as_ref(),
            Some(&direct_capture.capture_id)
        );
        let direct_commit = fixture
            .service
            .apply_capture_assignment(
                &direct,
                &ApprovedCaptureAssignment::new(direct.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(
            direct_commit.derived_capture_id.as_ref(),
            Some(&direct_capture.capture_id)
        );

        let duplicate_capture = fixture.capture(
            fixture.target_project_id.clone(),
            "direct-duplicate",
            1_800_000_200,
        );
        let intake = fixture
            .service
            .preview_capture(duplicate_capture.clone())
            .unwrap();
        fixture
            .service
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                1_800_000_201,
            )
            .unwrap();
        let duplicate_source = fixture.enqueue(duplicate_capture, None, 1_800_000_210);
        let duplicate = fixture
            .service
            .preview_capture_assignment(
                &duplicate_source.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Assign,
                1_800_000_220,
            )
            .unwrap();
        assert_eq!(
            duplicate.preview().outcome,
            CaptureAssignmentPreviewOutcome::Duplicate
        );

        let rejected_capture = fixture.capture(
            fixture.source_project_id.clone(),
            "explicit-reject",
            1_800_000_300,
        );
        let rejected_source = fixture.enqueue(rejected_capture, None, 1_800_000_310);
        let rejected = fixture
            .service
            .preview_capture_assignment(
                &rejected_source.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Reject,
                1_800_000_320,
            )
            .unwrap();
        assert_eq!(
            rejected.preview().outcome,
            CaptureAssignmentPreviewOutcome::Rejected
        );
        assert!(rejected.preview().derived_capture_id.is_none());
        assert!(rejected.preview().child_envelope_id.is_none());
        let rejected_commit = fixture
            .service
            .apply_capture_assignment(
                &rejected,
                &ApprovedCaptureAssignment::new(rejected.preview().plan_digest.clone(), true),
            )
            .unwrap();
        assert_eq!(rejected_commit.outcome, CaptureAssignmentOutcome::Rejected);
        assert!(rejected_commit.derived_capture_id.is_none());
        assert!(rejected_commit.child_envelope_id.is_none());
        assert_eq!(
            fixture
                .service
                .inspect_capture_delivery(&rejected_source.envelope_id)
                .unwrap()
                .unwrap()
                .state,
            CaptureDeliveryState::Cancelled
        );
    }

    #[test]
    fn approvals_and_target_drift_fail_before_assignment_writes() {
        let fixture = Fixture::new();
        let capture = fixture.capture(
            fixture.source_project_id.clone(),
            "target-drift",
            1_800_000_400,
        );
        let source = fixture.enqueue(capture, None, 1_800_000_410);
        let plan = fixture
            .service
            .preview_capture_assignment(
                &source.envelope_id,
                &fixture.target_project_id,
                CaptureAssignmentDecision::Assign,
                1_800_000_420,
            )
            .unwrap();
        assert_eq!(
            fixture.service.apply_capture_assignment(
                &plan,
                &ApprovedCaptureAssignment::new(plan.preview().plan_digest.clone(), false),
            ),
            Err(ProjectError::ApprovalRequired)
        );
        assert_eq!(
            fixture.service.apply_capture_assignment(
                &plan,
                &ApprovedCaptureAssignment::new("wrong-plan", true),
            ),
            Err(ProjectError::PlanMismatch)
        );
        create_project(
            &fixture.service,
            &fixture.projects_root.join("unrelated"),
            ProjectId::parse("prj_33333333333333333333333333333333").unwrap(),
            "Unrelated project",
            300,
        );
        assert_eq!(
            fixture.service.apply_capture_assignment(
                &plan,
                &ApprovedCaptureAssignment::new(plan.preview().plan_digest.clone(), true),
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert_eq!(
            fixture
                .service
                .inspect_capture_delivery(&source.envelope_id)
                .unwrap()
                .unwrap()
                .state,
            CaptureDeliveryState::Queued
        );
        assert!(
            fixture
                .service
                .inspect_capture_assignment(&plan.preview().intent_id)
                .unwrap()
                .is_none()
        );
        assert!(
            fixture
                .service
                .list_capture_assignments()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn missing_archived_and_nonassignable_sources_are_rejected() {
        let fixture = Fixture::new();
        let capture = fixture.capture(
            fixture.source_project_id.clone(),
            "invalid-target",
            1_800_000_500,
        );
        let source = fixture.enqueue(capture, None, 1_800_000_510);
        let missing = ProjectId::parse("prj_44444444444444444444444444444444").unwrap();
        assert_eq!(
            fixture
                .service
                .preview_capture_assignment(
                    &source.envelope_id,
                    &missing,
                    CaptureAssignmentDecision::Assign,
                    1_800_000_520,
                )
                .unwrap_err(),
            ProjectError::ProjectNotRegistered
        );

        let archive = fixture
            .service
            .preview_archive(&fixture.target_project_id)
            .unwrap();
        fixture
            .service
            .apply(
                &archive,
                &ApprovedProjectMutation::new(archive.preview().plan_digest.clone(), true),
                1_800_000_521,
            )
            .unwrap();
        assert_eq!(
            fixture
                .service
                .preview_capture_assignment(
                    &source.envelope_id,
                    &fixture.target_project_id,
                    CaptureAssignmentDecision::Assign,
                    1_800_000_522,
                )
                .unwrap_err(),
            ProjectError::RevisionConflict
        );

        let delivering_capture = fixture.capture(
            fixture.source_project_id.clone(),
            "already-delivering",
            1_800_000_600,
        );
        let delivering_source = fixture.enqueue(
            delivering_capture,
            Some(CaptureDeliveryDestinationV1::new(fixture.source_project_id.clone(), 1).unwrap()),
            1_800_000_610,
        );
        let queued = fixture
            .service
            .inspect_capture_delivery(&delivering_source.envelope_id)
            .unwrap()
            .unwrap();
        fixture
            .service
            .begin_capture_delivery(
                &delivering_source.envelope_id,
                queued.generation,
                &queued.record_sha256,
                1_800_000_611,
            )
            .unwrap();
        assert_eq!(
            fixture
                .service
                .preview_capture_assignment(
                    &delivering_source.envelope_id,
                    &fixture.source_project_id,
                    CaptureAssignmentDecision::Assign,
                    1_800_000_612,
                )
                .unwrap_err(),
            ProjectError::InvalidDeliveryTransition
        );
    }
}
