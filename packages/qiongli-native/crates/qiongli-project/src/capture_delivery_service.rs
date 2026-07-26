use serde::{Deserialize, Serialize};

use crate::capture_delivery::{
    CaptureDeliveryAcknowledgementV1, CaptureDeliveryEnvelopeV1, CaptureDeliveryReason,
    CaptureDeliveryState, DeliveryAcknowledgementId, DeliveryEnvelopeId,
};
use crate::capture_delivery_storage::StoredCaptureDelivery;
use crate::model::{MAX_SEMANTIC_REVISION, ProjectId};
use crate::storage::{read_manifest, sha256_bytes};
use crate::{CaptureDelivery, CaptureId, CaptureSource, ProjectError, ProjectStateService};

pub const CAPTURE_DELIVERY_SERVICE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureDeliveryRetryCause {
    ProcessInterrupted,
    TransportUnavailable,
    DestinationUnavailable,
    RecoveryRequired,
    ConflictResolved,
}

impl CaptureDeliveryRetryCause {
    const fn reason(self) -> CaptureDeliveryReason {
        match self {
            Self::ProcessInterrupted => CaptureDeliveryReason::DeliveryProcessInterrupted,
            Self::TransportUnavailable => CaptureDeliveryReason::DeliveryTransportUnavailable,
            Self::DestinationUnavailable => CaptureDeliveryReason::DeliveryDestinationUnavailable,
            Self::RecoveryRequired => CaptureDeliveryReason::DeliveryRecoveryRequired,
            Self::ConflictResolved => CaptureDeliveryReason::DeliveryRetryRequested,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDeliveryAcknowledgementRequestV1 {
    pub envelope_id: DeliveryEnvelopeId,
    pub destination_project_id: ProjectId,
    pub accepted_capture_id: CaptureId,
    pub expected_project_revision: u64,
    pub resulting_project_revision: u64,
    pub acknowledged_at_unix: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDeliveryAcknowledgementSummaryV1 {
    pub acknowledgement_id: DeliveryAcknowledgementId,
    pub destination_project_id: ProjectId,
    pub accepted_capture_id: CaptureId,
    pub expected_project_revision: u64,
    pub resulting_project_revision: u64,
    pub acknowledged_at_unix: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDeliveryDestinationSummaryV1 {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDeliveryStatusV1 {
    pub schema_version: u32,
    pub envelope_id: DeliveryEnvelopeId,
    pub capture_id: CaptureId,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub destination: Option<CaptureDeliveryDestinationSummaryV1>,
    pub state: CaptureDeliveryState,
    pub generation: u64,
    pub attempt_count: u32,
    pub retry_count: u32,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
    pub last_reason: CaptureDeliveryReason,
    pub envelope_sha256: String,
    pub record_sha256: String,
    pub acknowledgement: Option<CaptureDeliveryAcknowledgementSummaryV1>,
}

impl ProjectStateService {
    pub fn enqueue_capture_delivery(
        &self,
        envelope: CaptureDeliveryEnvelopeV1,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        self.delivery_store.enqueue(&envelope).map(delivery_status)
    }

    pub fn inspect_capture_delivery(
        &self,
        envelope_id: &DeliveryEnvelopeId,
    ) -> Result<Option<CaptureDeliveryStatusV1>, ProjectError> {
        self.delivery_store
            .read(envelope_id)
            .map(|entry| entry.map(delivery_status))
    }

    pub fn list_capture_deliveries(&self) -> Result<Vec<CaptureDeliveryStatusV1>, ProjectError> {
        self.delivery_store
            .rebuild()
            .map(|snapshot| snapshot.entries.into_iter().map(delivery_status).collect())
    }

    pub fn list_capture_deliveries_for_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<CaptureDeliveryStatusV1>, ProjectError> {
        self.resolve_project_root(project_id)?;
        self.delivery_store.rebuild().map(|snapshot| {
            snapshot
                .entries
                .into_iter()
                .filter(|entry| {
                    entry.envelope.capture.binding.project_id == *project_id
                        || entry
                            .envelope
                            .destination
                            .as_ref()
                            .is_some_and(|destination| destination.project_id == *project_id)
                })
                .map(delivery_status)
                .collect()
        })
    }

    pub fn begin_capture_delivery(
        &self,
        envelope_id: &DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        started_at_unix: u64,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        let current = self.delivery_entry(envelope_id)?;
        if current.envelope.destination.is_none() {
            return self.transition_capture_delivery(
                current,
                expected_generation,
                expected_record_sha256,
                CaptureDeliveryState::Conflicted,
                started_at_unix,
                CaptureDeliveryReason::DeliveryDestinationConflict,
            );
        }
        let reason = if current.record.state == CaptureDeliveryState::Delivering {
            current.record.transitions.last().map_or(
                CaptureDeliveryReason::DeliveryAttemptStarted,
                |transition| transition.reason_code,
            )
        } else if current.record.state == CaptureDeliveryState::RetryRequired {
            CaptureDeliveryReason::DeliveryRetryStarted
        } else {
            CaptureDeliveryReason::DeliveryAttemptStarted
        };
        self.transition_capture_delivery(
            current,
            expected_generation,
            expected_record_sha256,
            CaptureDeliveryState::Delivering,
            started_at_unix,
            reason,
        )
    }

    pub fn record_capture_delivery(
        &self,
        envelope_id: &DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        delivered_at_unix: u64,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        let current = self.delivery_entry(envelope_id)?;
        if current.envelope.destination.is_none() {
            return self.transition_capture_delivery(
                current,
                expected_generation,
                expected_record_sha256,
                CaptureDeliveryState::Conflicted,
                delivered_at_unix,
                CaptureDeliveryReason::DeliveryDestinationConflict,
            );
        }
        self.transition_capture_delivery(
            current,
            expected_generation,
            expected_record_sha256,
            CaptureDeliveryState::Delivered,
            delivered_at_unix,
            CaptureDeliveryReason::DeliveryAccepted,
        )
    }

    pub fn retry_capture_delivery(
        &self,
        envelope_id: &DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        retried_at_unix: u64,
        cause: CaptureDeliveryRetryCause,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        let current = self.delivery_entry(envelope_id)?;
        self.transition_capture_delivery(
            current,
            expected_generation,
            expected_record_sha256,
            CaptureDeliveryState::RetryRequired,
            retried_at_unix,
            cause.reason(),
        )
    }

    pub fn cancel_capture_delivery(
        &self,
        envelope_id: &DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        cancelled_at_unix: u64,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        let current = self.delivery_entry(envelope_id)?;
        self.transition_capture_delivery(
            current,
            expected_generation,
            expected_record_sha256,
            CaptureDeliveryState::Cancelled,
            cancelled_at_unix,
            CaptureDeliveryReason::DeliveryCancelled,
        )
    }

    pub fn acknowledge_capture_delivery(
        &self,
        request: &CaptureDeliveryAcknowledgementRequestV1,
        expected_generation: u64,
        expected_record_sha256: &str,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        let current = self.delivery_entry(&request.envelope_id)?;
        validate_service_timestamp(request.acknowledged_at_unix)?;

        if let Some(existing) = current.acknowledgement.as_ref() {
            if acknowledgement_matches_request(existing, request) {
                return self
                    .delivery_store
                    .acknowledge(existing, expected_generation, expected_record_sha256)
                    .map(delivery_status);
            }
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }

        let Some(destination) = current.envelope.destination.as_ref() else {
            self.persist_acknowledgement_conflict(
                current,
                expected_generation,
                expected_record_sha256,
                request.acknowledged_at_unix,
                CaptureDeliveryReason::DeliveryDestinationConflict,
            )?;
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        };
        if request.destination_project_id != destination.project_id
            || request.accepted_capture_id != current.envelope.capture_id
        {
            self.persist_acknowledgement_conflict(
                current,
                expected_generation,
                expected_record_sha256,
                request.acknowledged_at_unix,
                CaptureDeliveryReason::DeliveryDestinationConflict,
            )?;
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }
        if request.expected_project_revision != destination.expected_project_revision
            || request.resulting_project_revision < request.expected_project_revision
            || request.resulting_project_revision > MAX_SEMANTIC_REVISION
        {
            self.persist_acknowledgement_conflict(
                current,
                expected_generation,
                expected_record_sha256,
                request.acknowledged_at_unix,
                CaptureDeliveryReason::DeliveryRevisionConflict,
            )?;
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }

        let root = self.resolve_project_root(&destination.project_id)?;
        let (manifest, _) =
            read_manifest(root.path())?.ok_or(ProjectError::ProjectManifestMissing)?;
        let accepted_capture =
            self.read_capture(&destination.project_id, &request.accepted_capture_id)?;
        if manifest.semantic_revision != request.resulting_project_revision
            || accepted_capture.as_ref() != Some(&current.envelope.capture)
        {
            self.persist_acknowledgement_conflict(
                current,
                expected_generation,
                expected_record_sha256,
                request.acknowledged_at_unix,
                if manifest.semantic_revision != request.resulting_project_revision {
                    CaptureDeliveryReason::DeliveryRevisionConflict
                } else {
                    CaptureDeliveryReason::DeliveryDestinationConflict
                },
            )?;
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }

        let acknowledgement = CaptureDeliveryAcknowledgementV1::new(
            &current.envelope,
            request.accepted_capture_id.clone(),
            request.resulting_project_revision,
            request.acknowledged_at_unix,
        )?;
        self.delivery_store
            .acknowledge(
                &acknowledgement,
                expected_generation,
                expected_record_sha256,
            )
            .map(delivery_status)
    }

    fn delivery_entry(
        &self,
        envelope_id: &DeliveryEnvelopeId,
    ) -> Result<StoredCaptureDelivery, ProjectError> {
        self.delivery_store
            .read(envelope_id)?
            .ok_or(ProjectError::DeliveryNotFound)
    }

    fn transition_capture_delivery(
        &self,
        current: StoredCaptureDelivery,
        expected_generation: u64,
        expected_record_sha256: &str,
        next_state: CaptureDeliveryState,
        transitioned_at_unix: u64,
        reason: CaptureDeliveryReason,
    ) -> Result<CaptureDeliveryStatusV1, ProjectError> {
        validate_service_timestamp(transitioned_at_unix)?;
        if exact_transition_replay(
            &current,
            expected_generation,
            expected_record_sha256,
            next_state,
            transitioned_at_unix,
            reason,
        )? {
            return Ok(delivery_status(current));
        }
        let next = current
            .record
            .transition(next_state, transitioned_at_unix, reason, None)?;
        self.delivery_store
            .replace_record(
                &current.envelope.envelope_id,
                expected_generation,
                expected_record_sha256,
                &next,
            )
            .map(delivery_status)
    }

    fn persist_acknowledgement_conflict(
        &self,
        current: StoredCaptureDelivery,
        expected_generation: u64,
        expected_record_sha256: &str,
        transitioned_at_unix: u64,
        reason: CaptureDeliveryReason,
    ) -> Result<(), ProjectError> {
        self.transition_capture_delivery(
            current,
            expected_generation,
            expected_record_sha256,
            CaptureDeliveryState::Conflicted,
            transitioned_at_unix,
            reason,
        )
        .map(|_| ())
    }
}

fn delivery_status(entry: StoredCaptureDelivery) -> CaptureDeliveryStatusV1 {
    let last_reason = entry
        .record
        .transitions
        .last()
        .map_or(CaptureDeliveryReason::DeliveryEnqueued, |transition| {
            transition.reason_code
        });
    let acknowledgement =
        entry
            .acknowledgement
            .map(|document| CaptureDeliveryAcknowledgementSummaryV1 {
                acknowledgement_id: document.acknowledgement_id,
                destination_project_id: document.destination_project_id,
                accepted_capture_id: document.accepted_capture_id,
                expected_project_revision: document.expected_project_revision,
                resulting_project_revision: document.resulting_project_revision,
                acknowledged_at_unix: document.acknowledged_at_unix,
            });
    CaptureDeliveryStatusV1 {
        schema_version: CAPTURE_DELIVERY_SERVICE_SCHEMA_VERSION,
        envelope_id: entry.envelope.envelope_id,
        capture_id: entry.envelope.capture_id,
        source: entry.envelope.source,
        delivery: entry.envelope.delivery,
        destination: entry.envelope.destination.map(|destination| {
            CaptureDeliveryDestinationSummaryV1 {
                project_id: destination.project_id,
                expected_project_revision: destination.expected_project_revision,
            }
        }),
        state: entry.record.state,
        generation: entry.record.generation,
        attempt_count: entry.record.attempt_count,
        retry_count: entry.record.attempt_count.saturating_sub(1),
        created_at_unix: entry.record.created_at_unix,
        updated_at_unix: entry.record.updated_at_unix,
        last_reason,
        envelope_sha256: entry.envelope_sha256,
        record_sha256: entry.record_sha256,
        acknowledgement,
    }
}

fn exact_transition_replay(
    current: &StoredCaptureDelivery,
    expected_generation: u64,
    expected_record_sha256: &str,
    next_state: CaptureDeliveryState,
    transitioned_at_unix: u64,
    reason: CaptureDeliveryReason,
) -> Result<bool, ProjectError> {
    if current.record.state != next_state {
        return Ok(false);
    }
    let transition = current
        .record
        .transitions
        .last()
        .ok_or(ProjectError::InvalidDeliveryDocument)?;
    let previous = current.record.previous()?;
    let previous_sha256 = sha256_bytes(&previous.to_canonical_json()?);
    if transition.to_state == next_state
        && transition.transitioned_at_unix == transitioned_at_unix
        && transition.reason_code == reason
        && transition.acknowledgement_id.is_none()
        && previous.generation == expected_generation
        && previous_sha256 == expected_record_sha256
    {
        return Ok(true);
    }
    Err(ProjectError::RevisionConflict)
}

fn acknowledgement_matches_request(
    acknowledgement: &CaptureDeliveryAcknowledgementV1,
    request: &CaptureDeliveryAcknowledgementRequestV1,
) -> bool {
    acknowledgement.envelope_id == request.envelope_id
        && acknowledgement.destination_project_id == request.destination_project_id
        && acknowledgement.accepted_capture_id == request.accepted_capture_id
        && acknowledgement.expected_project_revision == request.expected_project_revision
        && acknowledgement.resulting_project_revision == request.resulting_project_revision
        && acknowledgement.acknowledged_at_unix == request.acknowledged_at_unix
}

fn validate_service_timestamp(timestamp: u64) -> Result<(), ProjectError> {
    if timestamp > MAX_SEMANTIC_REVISION {
        Err(ProjectError::InvalidDeliveryDocument)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::{ConfigRoot, resolve_config_root};

    use crate::{
        ApprovedCaptureIntake, ApprovedProjectMutation, CaptureArea, CaptureDeliveryDestinationV1,
        CapturePolicy, ContradictionV1, DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind,
        EvidenceReferenceV1, ProjectBindingV1, ProjectKind, ProjectRegistrationOptions,
        ProjectStage, ResearchCaptureDraftV1, ResearchCaptureV1, SemanticChangeV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        project_root: PathBuf,
        config_root: ConfigRoot,
        service: ProjectStateService,
        project_id: ProjectId,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "qiongli-capture-delivery-service-{}-{}",
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
            let project_id = ProjectId::parse("prj_abcdef0123456789abcdef0123456789").unwrap();
            let project_root = projects.join("delivery-project");
            let create = service
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new("Delivery project", ProjectKind::Article)
                        .with_project_id(project_id.clone())
                        .with_stage(ProjectStage::Literature),
                    100,
                )
                .unwrap();
            service
                .apply(
                    &create,
                    &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                    100,
                )
                .unwrap();
            Self {
                root,
                project_root,
                config_root,
                service,
                project_id,
            }
        }

        fn capture(&self, suffix: &str, captured_at_unix: u64) -> ResearchCaptureV1 {
            ResearchCaptureDraftV1 {
                binding: ProjectBindingV1::new(
                    self.project_id.clone(),
                    1,
                    ProjectStage::Literature,
                    format!("Route capture {suffix}"),
                    CapturePolicy::ReviewRequired,
                )
                .unwrap(),
                source: CaptureSource::Codex,
                delivery: CaptureDelivery::Connected,
                captured_at_unix,
                summary: format!("Capture delivery summary {suffix}."),
                changes: vec![SemanticChangeV1 {
                    area: CaptureArea::Method,
                    summary: "Separate validity from reliability.".to_string(),
                }],
                decisions: vec![DecisionCandidateV1 {
                    relation: DecisionRelation::Candidate,
                    statement: "Track validity independently.".to_string(),
                    rationale: "The sources distinguish it.".to_string(),
                    target: None,
                }],
                evidence: vec![EvidenceReferenceV1 {
                    locator_kind: EvidenceLocatorKind::Doi,
                    locator: format!("10.1000/service-{suffix}"),
                    relevance: "Supports the distinction.".to_string(),
                    limitation: None,
                }],
                contradictions: Vec::<ContradictionV1>::new(),
                next_actions: vec!["Review the candidate.".to_string()],
            }
            .into_capture()
            .unwrap()
        }

        fn envelope(&self, suffix: &str, created_at_unix: u64) -> CaptureDeliveryEnvelopeV1 {
            CaptureDeliveryEnvelopeV1::new(
                self.capture(suffix, created_at_unix - 10),
                Some(CaptureDeliveryDestinationV1::new(self.project_id.clone(), 1).unwrap()),
                created_at_unix,
            )
            .unwrap()
        }

        fn apply_capture(&self, capture: ResearchCaptureV1, accepted_at_unix: u64) {
            let plan = self.service.preview_capture(capture).unwrap();
            self.service
                .apply_capture(
                    &plan,
                    &ApprovedCaptureIntake::new(plan.preview().plan_digest.clone(), true),
                    accepted_at_unix,
                )
                .unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn acknowledgement_request(
        envelope: &CaptureDeliveryEnvelopeV1,
        acknowledged_at_unix: u64,
    ) -> CaptureDeliveryAcknowledgementRequestV1 {
        let destination = envelope.destination.as_ref().unwrap();
        CaptureDeliveryAcknowledgementRequestV1 {
            envelope_id: envelope.envelope_id.clone(),
            destination_project_id: destination.project_id.clone(),
            accepted_capture_id: envelope.capture_id.clone(),
            expected_project_revision: destination.expected_project_revision,
            resulting_project_revision: destination.expected_project_revision,
            acknowledged_at_unix,
        }
    }

    #[test]
    fn service_delivers_acknowledges_replays_and_reopens_one_capture() {
        let fixture = Fixture::new();
        let envelope = fixture.envelope("complete", 1_800_000_010);
        let queued = fixture
            .service
            .enqueue_capture_delivery(envelope.clone())
            .unwrap();
        assert_eq!(
            fixture
                .service
                .list_capture_deliveries_for_project(&fixture.project_id)
                .unwrap(),
            vec![queued.clone()]
        );
        assert_eq!(
            fixture
                .service
                .list_capture_deliveries_for_project(
                    &ProjectId::parse("prj_11111111111111111111111111111111").unwrap(),
                )
                .unwrap_err(),
            ProjectError::ProjectNotRegistered
        );
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            queued
        );
        let delivering = fixture
            .service
            .begin_capture_delivery(
                &envelope.envelope_id,
                queued.generation,
                &queued.record_sha256,
                1_800_000_011,
            )
            .unwrap();
        assert_eq!(delivering.state, CaptureDeliveryState::Delivering);
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            delivering
        );
        assert_eq!(
            fixture
                .service
                .begin_capture_delivery(
                    &envelope.envelope_id,
                    queued.generation,
                    &queued.record_sha256,
                    1_800_000_011,
                )
                .unwrap(),
            delivering
        );

        fixture.apply_capture(envelope.capture.clone(), 1_800_000_012);
        let duplicate = fixture
            .service
            .preview_capture(envelope.capture.clone())
            .unwrap();
        assert_eq!(
            duplicate.preview().effect,
            crate::CaptureIntakeEffect::NoChange
        );

        let restarted_after_capture = ProjectStateService::new(fixture.config_root.clone());
        let delivered = restarted_after_capture
            .record_capture_delivery(
                &envelope.envelope_id,
                delivering.generation,
                &delivering.record_sha256,
                1_800_000_013,
            )
            .unwrap();
        assert_eq!(
            restarted_after_capture
                .record_capture_delivery(
                    &envelope.envelope_id,
                    delivering.generation,
                    &delivering.record_sha256,
                    1_800_000_013,
                )
                .unwrap(),
            delivered
        );
        let restarted_after_delivery = ProjectStateService::new(fixture.config_root.clone());
        assert_eq!(
            restarted_after_delivery
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            delivered
        );
        let request = acknowledgement_request(&envelope, 1_800_000_014);
        let acknowledged = restarted_after_delivery
            .acknowledge_capture_delivery(&request, delivered.generation, &delivered.record_sha256)
            .unwrap();
        assert_eq!(acknowledged.state, CaptureDeliveryState::Acknowledged);
        assert_eq!(
            restarted_after_delivery
                .acknowledge_capture_delivery(
                    &request,
                    delivered.generation,
                    &delivered.record_sha256,
                )
                .unwrap(),
            acknowledged
        );

        let reopened = ProjectStateService::new(fixture.config_root.clone())
            .inspect_capture_delivery(&envelope.envelope_id)
            .unwrap()
            .unwrap();
        assert_eq!(reopened, acknowledged);
        assert_eq!(
            fs::read_dir(fixture.project_root.join("context/captures"))
                .unwrap()
                .count(),
            1
        );
        let json = serde_json::to_string(&reopened).unwrap();
        for forbidden in [
            fixture.project_root.to_string_lossy().into_owned(),
            envelope.capture.summary.clone(),
            "/Users/".to_string(),
            "rootPath".to_string(),
        ] {
            assert!(!json.contains(&forbidden));
        }
    }

    #[test]
    fn service_retry_cancel_and_unbound_conflict_are_causal_and_idempotent() {
        let fixture = Fixture::new();
        let envelope = fixture.envelope("retry", 1_800_000_010);
        let queued = fixture
            .service
            .enqueue_capture_delivery(envelope.clone())
            .unwrap();
        let delivering = fixture
            .service
            .begin_capture_delivery(
                &envelope.envelope_id,
                queued.generation,
                &queued.record_sha256,
                1_800_000_011,
            )
            .unwrap();
        let retry = fixture
            .service
            .retry_capture_delivery(
                &envelope.envelope_id,
                delivering.generation,
                &delivering.record_sha256,
                1_800_000_012,
                CaptureDeliveryRetryCause::TransportUnavailable,
            )
            .unwrap();
        assert_eq!(retry.state, CaptureDeliveryState::RetryRequired);
        assert_eq!(
            fixture
                .service
                .retry_capture_delivery(
                    &envelope.envelope_id,
                    delivering.generation,
                    &delivering.record_sha256,
                    1_800_000_012,
                    CaptureDeliveryRetryCause::TransportUnavailable,
                )
                .unwrap(),
            retry
        );
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            retry
        );
        let delivering_again = fixture
            .service
            .begin_capture_delivery(
                &envelope.envelope_id,
                retry.generation,
                &retry.record_sha256,
                1_800_000_013,
            )
            .unwrap();
        assert_eq!(delivering_again.retry_count, 1);
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            delivering_again
        );
        let cancelled = fixture
            .service
            .cancel_capture_delivery(
                &envelope.envelope_id,
                delivering_again.generation,
                &delivering_again.record_sha256,
                1_800_000_014,
            )
            .unwrap();
        assert_eq!(cancelled.state, CaptureDeliveryState::Cancelled);
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            cancelled
        );

        let unbound = CaptureDeliveryEnvelopeV1::new(
            fixture.capture("unbound", 1_800_000_020),
            None,
            1_800_000_030,
        )
        .unwrap();
        let unbound_queued = fixture
            .service
            .enqueue_capture_delivery(unbound.clone())
            .unwrap();
        let conflicted = fixture
            .service
            .begin_capture_delivery(
                &unbound.envelope_id,
                unbound_queued.generation,
                &unbound_queued.record_sha256,
                1_800_000_031,
            )
            .unwrap();
        assert_eq!(conflicted.state, CaptureDeliveryState::Conflicted);
        assert_eq!(
            conflicted.last_reason,
            CaptureDeliveryReason::DeliveryDestinationConflict
        );
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&unbound.envelope_id)
                .unwrap()
                .unwrap(),
            conflicted
        );
    }

    #[test]
    fn wrong_revision_acknowledgement_conflicts_without_second_capture() {
        let fixture = Fixture::new();
        let envelope = fixture.envelope("wrong-revision", 1_800_000_010);
        let queued = fixture
            .service
            .enqueue_capture_delivery(envelope.clone())
            .unwrap();
        let delivering = fixture
            .service
            .begin_capture_delivery(
                &envelope.envelope_id,
                queued.generation,
                &queued.record_sha256,
                1_800_000_011,
            )
            .unwrap();
        fixture.apply_capture(envelope.capture.clone(), 1_800_000_012);
        let delivered = fixture
            .service
            .record_capture_delivery(
                &envelope.envelope_id,
                delivering.generation,
                &delivering.record_sha256,
                1_800_000_013,
            )
            .unwrap();

        let mut wrong = acknowledgement_request(&envelope, 1_800_000_014);
        wrong.expected_project_revision += 1;
        assert_eq!(
            fixture.service.acknowledge_capture_delivery(
                &wrong,
                delivered.generation,
                &delivered.record_sha256,
            ),
            Err(ProjectError::DeliveryAcknowledgementConflict)
        );
        let conflicted = fixture
            .service
            .inspect_capture_delivery(&envelope.envelope_id)
            .unwrap()
            .unwrap();
        assert_eq!(conflicted.state, CaptureDeliveryState::Conflicted);
        assert_eq!(
            conflicted.last_reason,
            CaptureDeliveryReason::DeliveryRevisionConflict
        );
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            conflicted
        );
        let retry = fixture
            .service
            .retry_capture_delivery(
                &envelope.envelope_id,
                conflicted.generation,
                &conflicted.record_sha256,
                1_800_000_015,
                CaptureDeliveryRetryCause::ConflictResolved,
            )
            .unwrap();
        assert_eq!(retry.state, CaptureDeliveryState::RetryRequired);
        assert_eq!(
            fs::read_dir(fixture.project_root.join("context/captures"))
                .unwrap()
                .count(),
            1
        );
    }

    #[test]
    fn wrong_project_acknowledgement_is_a_stable_destination_conflict() {
        let fixture = Fixture::new();
        let envelope = fixture.envelope("wrong-project", 1_800_000_010);
        let queued = fixture
            .service
            .enqueue_capture_delivery(envelope.clone())
            .unwrap();
        let delivering = fixture
            .service
            .begin_capture_delivery(
                &envelope.envelope_id,
                queued.generation,
                &queued.record_sha256,
                1_800_000_011,
            )
            .unwrap();
        fixture.apply_capture(envelope.capture.clone(), 1_800_000_012);
        let delivered = fixture
            .service
            .record_capture_delivery(
                &envelope.envelope_id,
                delivering.generation,
                &delivering.record_sha256,
                1_800_000_013,
            )
            .unwrap();
        let mut wrong = acknowledgement_request(&envelope, 1_800_000_014);
        wrong.destination_project_id =
            ProjectId::parse("prj_11111111111111111111111111111111").unwrap();
        assert_eq!(
            fixture.service.acknowledge_capture_delivery(
                &wrong,
                delivered.generation,
                &delivered.record_sha256,
            ),
            Err(ProjectError::DeliveryAcknowledgementConflict)
        );
        let conflicted = fixture
            .service
            .inspect_capture_delivery(&envelope.envelope_id)
            .unwrap()
            .unwrap();
        assert_eq!(conflicted.state, CaptureDeliveryState::Conflicted);
        assert_eq!(
            conflicted.last_reason,
            CaptureDeliveryReason::DeliveryDestinationConflict
        );
        assert_eq!(
            ProjectStateService::new(fixture.config_root.clone())
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            conflicted
        );
        assert_eq!(
            fixture.service.acknowledge_capture_delivery(
                &wrong,
                delivered.generation,
                &delivered.record_sha256,
            ),
            Err(ProjectError::DeliveryAcknowledgementConflict)
        );
        assert_eq!(
            fixture
                .service
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            conflicted
        );
        assert_eq!(
            fs::read_dir(fixture.project_root.join("context/captures"))
                .unwrap()
                .count(),
            1
        );
    }
}
