use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ProjectError;
use crate::capture::{CaptureDelivery, CaptureId, CaptureSource, ResearchCaptureV1};
use crate::model::{MAX_SEMANTIC_REVISION, ProjectId, valid_lower_hex};

pub const CAPTURE_DELIVERY_ENVELOPE_SCHEMA_VERSION: u32 = 1;
pub const CAPTURE_DELIVERY_ENVELOPE_DOCUMENT_KIND: &str = "qiongli-capture-delivery-envelope";
pub const CAPTURE_DELIVERY_RECORD_SCHEMA_VERSION: u32 = 1;
pub const CAPTURE_DELIVERY_RECORD_DOCUMENT_KIND: &str = "qiongli-capture-delivery-record";
pub const CAPTURE_DELIVERY_ACKNOWLEDGEMENT_SCHEMA_VERSION: u32 = 1;
pub const CAPTURE_DELIVERY_ACKNOWLEDGEMENT_DOCUMENT_KIND: &str =
    "qiongli-capture-delivery-acknowledgement";
pub const DELIVERY_ENVELOPE_ID_PREFIX: &str = "env_";
pub const DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX: &str = "dack_";

pub(crate) const MAX_DELIVERY_ENVELOPE_BYTES: usize = 72 * 1024;
pub(crate) const MAX_DELIVERY_RECORD_BYTES: usize = 64 * 1024;
pub(crate) const MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES: usize = 8 * 1024;
pub(crate) const MAX_DELIVERY_TRANSITIONS: usize = 64;
pub(crate) const MAX_DELIVERY_ATTEMPTS: u32 = 32;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeliveryEnvelopeId(String);

impl DeliveryEnvelopeId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectError> {
        let value = value.into();
        if value.len() != DELIVERY_ENVELOPE_ID_PREFIX.len() + 64
            || !value.starts_with(DELIVERY_ENVELOPE_ID_PREFIX)
            || !valid_lower_hex(&value[DELIVERY_ENVELOPE_ID_PREFIX.len()..], 64)
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        Ok(Self(value))
    }

    fn from_digest(digest: &str) -> Self {
        Self(format!("{DELIVERY_ENVELOPE_ID_PREFIX}{digest}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), ProjectError> {
        Self::parse(self.0.clone()).map(|_| ())
    }
}

impl Debug for DeliveryEnvelopeId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DeliveryEnvelopeId")
            .field(&self.0)
            .finish()
    }
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeliveryAcknowledgementId(String);

impl DeliveryAcknowledgementId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ProjectError> {
        let value = value.into();
        if value.len() != DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX.len() + 64
            || !value.starts_with(DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX)
            || !valid_lower_hex(&value[DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX.len()..], 64)
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        Ok(Self(value))
    }

    fn from_digest(digest: &str) -> Self {
        Self(format!("{DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX}{digest}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), ProjectError> {
        Self::parse(self.0.clone()).map(|_| ())
    }
}

impl Debug for DeliveryAcknowledgementId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DeliveryAcknowledgementId")
            .field(&self.0)
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDeliveryDestinationV1 {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
}

impl CaptureDeliveryDestinationV1 {
    pub fn new(
        project_id: ProjectId,
        expected_project_revision: u64,
    ) -> Result<Self, ProjectError> {
        let destination = Self {
            project_id,
            expected_project_revision,
        };
        destination.validate()?;
        Ok(destination)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        self.project_id
            .validate()
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        if self.expected_project_revision == 0
            || self.expected_project_revision > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDeliveryEnvelopeV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub envelope_id: DeliveryEnvelopeId,
    pub capture_id: CaptureId,
    pub capture_sha256: String,
    pub source: CaptureSource,
    pub delivery: CaptureDelivery,
    pub destination: Option<CaptureDeliveryDestinationV1>,
    pub created_at_unix: u64,
    pub capture: ResearchCaptureV1,
}

impl Debug for CaptureDeliveryEnvelopeV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CaptureDeliveryEnvelopeV1")
            .field("schema_version", &self.schema_version)
            .field("document_kind", &self.document_kind)
            .field("envelope_id", &self.envelope_id)
            .field("capture_id", &self.capture_id)
            .field("capture_sha256", &self.capture_sha256)
            .field("source", &self.source)
            .field("delivery", &self.delivery)
            .field("destination", &self.destination)
            .field("created_at_unix", &self.created_at_unix)
            .field("capture", &"<bounded-research-capture>")
            .finish()
    }
}

impl CaptureDeliveryEnvelopeV1 {
    pub fn new(
        capture: ResearchCaptureV1,
        destination: Option<CaptureDeliveryDestinationV1>,
        created_at_unix: u64,
    ) -> Result<Self, ProjectError> {
        capture
            .validate()
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        if created_at_unix < capture.captured_at_unix || created_at_unix > MAX_SEMANTIC_REVISION {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        if let Some(destination) = destination.as_ref() {
            destination.validate()?;
        }
        let capture_sha256 = canonical_digest(&capture)?;
        let semantics = CaptureDeliveryEnvelopeSemantics {
            schema_version: CAPTURE_DELIVERY_ENVELOPE_SCHEMA_VERSION,
            capture_id: &capture.capture_id,
            capture_sha256: &capture_sha256,
            source: capture.source,
            delivery: capture.delivery,
            destination: destination.as_ref(),
            created_at_unix,
        };
        let envelope_id = DeliveryEnvelopeId::from_digest(&canonical_digest(&semantics)?);
        let envelope = Self {
            schema_version: CAPTURE_DELIVERY_ENVELOPE_SCHEMA_VERSION,
            document_kind: CAPTURE_DELIVERY_ENVELOPE_DOCUMENT_KIND.to_string(),
            envelope_id,
            capture_id: capture.capture_id.clone(),
            capture_sha256,
            source: capture.source,
            delivery: capture.delivery,
            destination,
            created_at_unix,
            capture,
        };
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_DELIVERY_ENVELOPE_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = crate::json::parse_unique_json(bytes)
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        let envelope: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_DELIVERY_ENVELOPE_BYTES)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != CAPTURE_DELIVERY_ENVELOPE_SCHEMA_VERSION
            || self.document_kind != CAPTURE_DELIVERY_ENVELOPE_DOCUMENT_KIND
            || self.capture_id != self.capture.capture_id
            || self.source != self.capture.source
            || self.delivery != self.capture.delivery
            || self.created_at_unix < self.capture.captured_at_unix
            || self.created_at_unix > MAX_SEMANTIC_REVISION
            || !valid_lower_hex(&self.capture_sha256, 64)
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        self.envelope_id.validate()?;
        self.capture
            .validate()
            .map_err(|_| ProjectError::DeliveryIdentityConflict)?;
        if let Some(destination) = self.destination.as_ref() {
            destination.validate()?;
        }
        if canonical_digest(&self.capture)? != self.capture_sha256 {
            return Err(ProjectError::DeliveryIdentityConflict);
        }
        let semantics = CaptureDeliveryEnvelopeSemantics {
            schema_version: self.schema_version,
            capture_id: &self.capture_id,
            capture_sha256: &self.capture_sha256,
            source: self.source,
            delivery: self.delivery,
            destination: self.destination.as_ref(),
            created_at_unix: self.created_at_unix,
        };
        if DeliveryEnvelopeId::from_digest(&canonical_digest(&semantics)?) != self.envelope_id {
            return Err(ProjectError::DeliveryIdentityConflict);
        }
        canonical_json(self, MAX_DELIVERY_ENVELOPE_BYTES).map(|_| ())
    }
}

#[derive(Serialize)]
struct CaptureDeliveryEnvelopeSemantics<'a> {
    schema_version: u32,
    capture_id: &'a CaptureId,
    capture_sha256: &'a str,
    source: CaptureSource,
    delivery: CaptureDelivery,
    destination: Option<&'a CaptureDeliveryDestinationV1>,
    created_at_unix: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureDeliveryState {
    Queued,
    Delivering,
    Delivered,
    Acknowledged,
    RetryRequired,
    Conflicted,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureDeliveryReason {
    DeliveryEnqueued,
    DeliveryAttemptStarted,
    DeliveryRetryStarted,
    DeliveryAccepted,
    DeliveryProcessInterrupted,
    DeliveryTransportUnavailable,
    DeliveryDestinationUnavailable,
    DeliveryDestinationConflict,
    DeliveryRevisionConflict,
    DeliveryRetryRequested,
    DeliveryAcknowledged,
    DeliveryCancelled,
    DeliveryRecoveryRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDeliveryTransitionV1 {
    pub generation: u64,
    pub from_state: Option<CaptureDeliveryState>,
    pub to_state: CaptureDeliveryState,
    pub transitioned_at_unix: u64,
    pub reason_code: CaptureDeliveryReason,
    pub acknowledgement_id: Option<DeliveryAcknowledgementId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDeliveryRecordV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub envelope_id: DeliveryEnvelopeId,
    pub envelope_sha256: String,
    pub state: CaptureDeliveryState,
    pub generation: u64,
    pub attempt_count: u32,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
    pub transitions: Vec<CaptureDeliveryTransitionV1>,
}

impl CaptureDeliveryRecordV1 {
    pub fn queued(
        envelope: &CaptureDeliveryEnvelopeV1,
        created_at_unix: u64,
    ) -> Result<Self, ProjectError> {
        envelope.validate()?;
        if created_at_unix < envelope.created_at_unix || created_at_unix > MAX_SEMANTIC_REVISION {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        let record = Self {
            schema_version: CAPTURE_DELIVERY_RECORD_SCHEMA_VERSION,
            document_kind: CAPTURE_DELIVERY_RECORD_DOCUMENT_KIND.to_string(),
            envelope_id: envelope.envelope_id.clone(),
            envelope_sha256: canonical_digest(envelope)?,
            state: CaptureDeliveryState::Queued,
            generation: 1,
            attempt_count: 0,
            created_at_unix,
            updated_at_unix: created_at_unix,
            transitions: vec![CaptureDeliveryTransitionV1 {
                generation: 1,
                from_state: None,
                to_state: CaptureDeliveryState::Queued,
                transitioned_at_unix: created_at_unix,
                reason_code: CaptureDeliveryReason::DeliveryEnqueued,
                acknowledgement_id: None,
            }],
        };
        record.validate()?;
        Ok(record)
    }

    pub fn transition(
        &self,
        next_state: CaptureDeliveryState,
        transitioned_at_unix: u64,
        reason_code: CaptureDeliveryReason,
        acknowledgement_id: Option<DeliveryAcknowledgementId>,
    ) -> Result<Self, ProjectError> {
        self.validate()?;
        if !valid_transition(self.state, next_state)
            || transitioned_at_unix < self.updated_at_unix
            || transitioned_at_unix > MAX_SEMANTIC_REVISION
            || self.transitions.len() >= MAX_DELIVERY_TRANSITIONS
            || (next_state == CaptureDeliveryState::Acknowledged) != acknowledgement_id.is_some()
            || !valid_transition_reason(self.state, next_state, reason_code)
        {
            return Err(ProjectError::InvalidDeliveryTransition);
        }
        if let Some(acknowledgement_id) = acknowledgement_id.as_ref() {
            acknowledgement_id.validate()?;
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ProjectError::InvalidDeliveryTransition)?;
        let attempt_count = self
            .attempt_count
            .checked_add(u32::from(next_state == CaptureDeliveryState::Delivering))
            .ok_or(ProjectError::InvalidDeliveryTransition)?;
        if attempt_count > MAX_DELIVERY_ATTEMPTS {
            return Err(ProjectError::InvalidDeliveryTransition);
        }
        let mut transitions = self.transitions.clone();
        transitions.push(CaptureDeliveryTransitionV1 {
            generation,
            from_state: Some(self.state),
            to_state: next_state,
            transitioned_at_unix,
            reason_code,
            acknowledgement_id,
        });
        let next = Self {
            schema_version: self.schema_version,
            document_kind: self.document_kind.clone(),
            envelope_id: self.envelope_id.clone(),
            envelope_sha256: self.envelope_sha256.clone(),
            state: next_state,
            generation,
            attempt_count,
            created_at_unix: self.created_at_unix,
            updated_at_unix: transitioned_at_unix,
            transitions,
        };
        next.validate()?;
        Ok(next)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_DELIVERY_RECORD_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = crate::json::parse_unique_json(bytes)
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        let record: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        record.validate()?;
        Ok(record)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_DELIVERY_RECORD_BYTES)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != CAPTURE_DELIVERY_RECORD_SCHEMA_VERSION
            || self.document_kind != CAPTURE_DELIVERY_RECORD_DOCUMENT_KIND
            || !valid_lower_hex(&self.envelope_sha256, 64)
            || self.generation == 0
            || self.generation > MAX_SEMANTIC_REVISION
            || self.attempt_count > MAX_DELIVERY_ATTEMPTS
            || self.created_at_unix > MAX_SEMANTIC_REVISION
            || self.updated_at_unix < self.created_at_unix
            || self.updated_at_unix > MAX_SEMANTIC_REVISION
            || self.transitions.is_empty()
            || self.transitions.len() > MAX_DELIVERY_TRANSITIONS
            || self.generation != self.transitions.len() as u64
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        self.envelope_id.validate()?;
        let mut previous_state = None;
        let mut previous_timestamp = self.created_at_unix;
        let mut observed_attempts = 0_u32;
        for (index, transition) in self.transitions.iter().enumerate() {
            let expected_generation = index as u64 + 1;
            if transition.generation != expected_generation
                || transition.from_state != previous_state
                || transition.transitioned_at_unix < previous_timestamp
                || transition.transitioned_at_unix > MAX_SEMANTIC_REVISION
                || (transition.to_state == CaptureDeliveryState::Acknowledged)
                    != transition.acknowledgement_id.is_some()
                || (index == 0
                    && (transition.from_state.is_some()
                        || transition.to_state != CaptureDeliveryState::Queued
                        || transition.reason_code != CaptureDeliveryReason::DeliveryEnqueued
                        || transition.transitioned_at_unix != self.created_at_unix))
                || (index > 0
                    && !transition.from_state.is_some_and(|from_state| {
                        valid_transition(from_state, transition.to_state)
                            && valid_transition_reason(
                                from_state,
                                transition.to_state,
                                transition.reason_code,
                            )
                    }))
            {
                return Err(ProjectError::InvalidDeliveryDocument);
            }
            if let Some(acknowledgement_id) = transition.acknowledgement_id.as_ref() {
                acknowledgement_id.validate()?;
            }
            if transition.to_state == CaptureDeliveryState::Delivering {
                observed_attempts = observed_attempts
                    .checked_add(1)
                    .ok_or(ProjectError::InvalidDeliveryDocument)?;
            }
            previous_state = Some(transition.to_state);
            previous_timestamp = transition.transitioned_at_unix;
        }
        if previous_state != Some(self.state)
            || previous_timestamp != self.updated_at_unix
            || observed_attempts != self.attempt_count
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        canonical_json(self, MAX_DELIVERY_RECORD_BYTES).map(|_| ())
    }

    #[must_use]
    pub fn acknowledgement_id(&self) -> Option<&DeliveryAcknowledgementId> {
        self.transitions
            .last()
            .and_then(|transition| transition.acknowledgement_id.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDeliveryAcknowledgementV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub acknowledgement_id: DeliveryAcknowledgementId,
    pub envelope_id: DeliveryEnvelopeId,
    pub destination_project_id: ProjectId,
    pub accepted_capture_id: CaptureId,
    pub expected_project_revision: u64,
    pub resulting_project_revision: u64,
    pub acknowledged_at_unix: u64,
}

impl CaptureDeliveryAcknowledgementV1 {
    pub fn new(
        envelope: &CaptureDeliveryEnvelopeV1,
        accepted_capture_id: CaptureId,
        resulting_project_revision: u64,
        acknowledged_at_unix: u64,
    ) -> Result<Self, ProjectError> {
        envelope.validate()?;
        let destination = envelope
            .destination
            .as_ref()
            .ok_or(ProjectError::DeliveryAcknowledgementConflict)?;
        if resulting_project_revision < destination.expected_project_revision
            || resulting_project_revision > MAX_SEMANTIC_REVISION
            || acknowledged_at_unix < envelope.created_at_unix
            || acknowledged_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::DeliveryAcknowledgementConflict);
        }
        let semantics = CaptureDeliveryAcknowledgementSemantics {
            schema_version: CAPTURE_DELIVERY_ACKNOWLEDGEMENT_SCHEMA_VERSION,
            envelope_id: &envelope.envelope_id,
            destination_project_id: &destination.project_id,
            accepted_capture_id: &accepted_capture_id,
            expected_project_revision: destination.expected_project_revision,
            resulting_project_revision,
            acknowledged_at_unix,
        };
        let acknowledgement_id =
            DeliveryAcknowledgementId::from_digest(&canonical_digest(&semantics)?);
        let acknowledgement = Self {
            schema_version: CAPTURE_DELIVERY_ACKNOWLEDGEMENT_SCHEMA_VERSION,
            document_kind: CAPTURE_DELIVERY_ACKNOWLEDGEMENT_DOCUMENT_KIND.to_string(),
            acknowledgement_id,
            envelope_id: envelope.envelope_id.clone(),
            destination_project_id: destination.project_id.clone(),
            accepted_capture_id,
            expected_project_revision: destination.expected_project_revision,
            resulting_project_revision,
            acknowledged_at_unix,
        };
        acknowledgement.validate()?;
        Ok(acknowledgement)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        if bytes.len() > MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES {
            return Err(ProjectError::DocumentTooLarge);
        }
        let value = crate::json::parse_unique_json(bytes)
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        let acknowledgement: Self =
            serde_json::from_value(value).map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        acknowledgement.validate()?;
        Ok(acknowledgement)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES)
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != CAPTURE_DELIVERY_ACKNOWLEDGEMENT_SCHEMA_VERSION
            || self.document_kind != CAPTURE_DELIVERY_ACKNOWLEDGEMENT_DOCUMENT_KIND
            || self.expected_project_revision == 0
            || self.expected_project_revision > MAX_SEMANTIC_REVISION
            || self.resulting_project_revision < self.expected_project_revision
            || self.resulting_project_revision > MAX_SEMANTIC_REVISION
            || self.acknowledged_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidDeliveryDocument);
        }
        self.acknowledgement_id.validate()?;
        self.envelope_id.validate()?;
        self.destination_project_id
            .validate()
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        CaptureId::parse(self.accepted_capture_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
        let semantics = CaptureDeliveryAcknowledgementSemantics {
            schema_version: self.schema_version,
            envelope_id: &self.envelope_id,
            destination_project_id: &self.destination_project_id,
            accepted_capture_id: &self.accepted_capture_id,
            expected_project_revision: self.expected_project_revision,
            resulting_project_revision: self.resulting_project_revision,
            acknowledged_at_unix: self.acknowledged_at_unix,
        };
        if DeliveryAcknowledgementId::from_digest(&canonical_digest(&semantics)?)
            != self.acknowledgement_id
        {
            return Err(ProjectError::DeliveryIdentityConflict);
        }
        canonical_json(self, MAX_DELIVERY_ACKNOWLEDGEMENT_BYTES).map(|_| ())
    }
}

#[derive(Serialize)]
struct CaptureDeliveryAcknowledgementSemantics<'a> {
    schema_version: u32,
    envelope_id: &'a DeliveryEnvelopeId,
    destination_project_id: &'a ProjectId,
    accepted_capture_id: &'a CaptureId,
    expected_project_revision: u64,
    resulting_project_revision: u64,
    acknowledged_at_unix: u64,
}

fn valid_transition(from: CaptureDeliveryState, to: CaptureDeliveryState) -> bool {
    matches!(
        (from, to),
        (
            CaptureDeliveryState::Queued,
            CaptureDeliveryState::Delivering
                | CaptureDeliveryState::Conflicted
                | CaptureDeliveryState::Cancelled
        ) | (
            CaptureDeliveryState::Delivering,
            CaptureDeliveryState::Delivered
                | CaptureDeliveryState::RetryRequired
                | CaptureDeliveryState::Conflicted
                | CaptureDeliveryState::Cancelled
        ) | (
            CaptureDeliveryState::Delivered,
            CaptureDeliveryState::Acknowledged
                | CaptureDeliveryState::RetryRequired
                | CaptureDeliveryState::Conflicted
                | CaptureDeliveryState::Cancelled
        ) | (
            CaptureDeliveryState::RetryRequired,
            CaptureDeliveryState::Delivering
                | CaptureDeliveryState::Conflicted
                | CaptureDeliveryState::Cancelled
        ) | (
            CaptureDeliveryState::Conflicted,
            CaptureDeliveryState::RetryRequired | CaptureDeliveryState::Cancelled
        )
    )
}

fn valid_transition_reason(
    from: CaptureDeliveryState,
    to: CaptureDeliveryState,
    reason: CaptureDeliveryReason,
) -> bool {
    match to {
        CaptureDeliveryState::Delivering => matches!(
            (from, reason),
            (
                CaptureDeliveryState::Queued,
                CaptureDeliveryReason::DeliveryAttemptStarted
            ) | (
                CaptureDeliveryState::RetryRequired,
                CaptureDeliveryReason::DeliveryRetryStarted
            )
        ),
        CaptureDeliveryState::Delivered => {
            from == CaptureDeliveryState::Delivering
                && reason == CaptureDeliveryReason::DeliveryAccepted
        }
        CaptureDeliveryState::Acknowledged => {
            from == CaptureDeliveryState::Delivered
                && reason == CaptureDeliveryReason::DeliveryAcknowledged
        }
        CaptureDeliveryState::RetryRequired => {
            matches!(
                from,
                CaptureDeliveryState::Delivering | CaptureDeliveryState::Delivered
            ) && matches!(
                reason,
                CaptureDeliveryReason::DeliveryProcessInterrupted
                    | CaptureDeliveryReason::DeliveryTransportUnavailable
                    | CaptureDeliveryReason::DeliveryDestinationUnavailable
                    | CaptureDeliveryReason::DeliveryRecoveryRequired
            ) || from == CaptureDeliveryState::Conflicted
                && reason == CaptureDeliveryReason::DeliveryRetryRequested
        }
        CaptureDeliveryState::Conflicted => {
            matches!(
                from,
                CaptureDeliveryState::Queued
                    | CaptureDeliveryState::Delivering
                    | CaptureDeliveryState::Delivered
                    | CaptureDeliveryState::RetryRequired
            ) && matches!(
                reason,
                CaptureDeliveryReason::DeliveryDestinationConflict
                    | CaptureDeliveryReason::DeliveryRevisionConflict
            )
        }
        CaptureDeliveryState::Cancelled => {
            matches!(
                from,
                CaptureDeliveryState::Queued
                    | CaptureDeliveryState::Delivering
                    | CaptureDeliveryState::Delivered
                    | CaptureDeliveryState::RetryRequired
                    | CaptureDeliveryState::Conflicted
            ) && reason == CaptureDeliveryReason::DeliveryCancelled
        }
        CaptureDeliveryState::Queued => false,
    }
}

fn canonical_json<T: Serialize>(value: &T, maximum_bytes: usize) -> Result<Vec<u8>, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
    if bytes.len() > maximum_bytes {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok(bytes)
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidDeliveryDocument)?;
    Ok(sha256(&bytes))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CaptureArea, CapturePolicy, DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind,
        EvidenceReferenceV1, ProjectBindingV1, ProjectStage, ResearchCaptureDraftV1,
        SemanticChangeV1,
    };

    fn project_id(value: &str) -> ProjectId {
        ProjectId::parse(value).unwrap()
    }

    fn capture() -> ResearchCaptureV1 {
        ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                project_id("prj_0123456789abcdef0123456789abcdef"),
                4,
                ProjectStage::Literature,
                "Route the verified capture",
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix: 1_800_000_000,
            summary: "The methods evidence requires a separate validity branch.".to_string(),
            changes: vec![SemanticChangeV1 {
                area: CaptureArea::Method,
                summary: "Separate validity from reliability evidence.".to_string(),
            }],
            decisions: vec![DecisionCandidateV1 {
                relation: DecisionRelation::Candidate,
                statement: "Track validity as an independent method concern.".to_string(),
                rationale: "The source set treats it independently.".to_string(),
                target: None,
            }],
            evidence: vec![EvidenceReferenceV1 {
                locator_kind: EvidenceLocatorKind::Doi,
                locator: "10.1000/delivery".to_string(),
                relevance: "Supports the proposed distinction.".to_string(),
                limitation: None,
            }],
            contradictions: Vec::new(),
            next_actions: vec!["Review the candidate before consolidation.".to_string()],
        }
        .into_capture()
        .unwrap()
    }

    fn bound_envelope() -> CaptureDeliveryEnvelopeV1 {
        CaptureDeliveryEnvelopeV1::new(
            capture(),
            Some(
                CaptureDeliveryDestinationV1::new(
                    project_id("prj_0123456789abcdef0123456789abcdef"),
                    4,
                )
                .unwrap(),
            ),
            1_800_000_010,
        )
        .unwrap()
    }

    #[test]
    fn envelope_is_content_addressed_strict_and_path_free() {
        let envelope = bound_envelope();
        let bytes = envelope.to_canonical_json().unwrap();
        assert_eq!(
            CaptureDeliveryEnvelopeV1::from_json_slice(&bytes).unwrap(),
            envelope
        );
        assert_eq!(
            envelope.envelope_id.as_str().len(),
            DELIVERY_ENVELOPE_ID_PREFIX.len() + 64
        );
        let text = String::from_utf8(bytes).unwrap();
        for forbidden in ["root_path", "/Users/", "session", "transcript", "prompt"] {
            assert!(!text.contains(forbidden));
        }
        let debug = format!("{envelope:?}");
        assert!(debug.contains("<bounded-research-capture>"));
        assert!(!debug.contains(&envelope.capture.summary));

        let mut changed = envelope.clone();
        changed.created_at_unix += 1;
        assert_eq!(
            changed.validate(),
            Err(ProjectError::DeliveryIdentityConflict)
        );

        let value = serde_json::to_value(&envelope).unwrap();
        let mut object = value.as_object().unwrap().clone();
        object.insert("unknown".to_string(), serde_json::json!(true));
        assert!(serde_json::from_value::<CaptureDeliveryEnvelopeV1>(object.into()).is_err());

        let canonical = String::from_utf8(envelope.to_canonical_json().unwrap()).unwrap();
        let duplicate = format!("{{\"schema_version\":1,{}", &canonical[1..]);
        assert_eq!(
            CaptureDeliveryEnvelopeV1::from_json_slice(duplicate.as_bytes()),
            Err(ProjectError::InvalidDeliveryDocument)
        );
        assert_eq!(
            CaptureDeliveryEnvelopeV1::from_json_slice(&vec![
                b' ';
                MAX_DELIVERY_ENVELOPE_BYTES + 1
            ]),
            Err(ProjectError::DocumentTooLarge)
        );
    }

    #[test]
    fn envelope_preserves_declared_binding_and_optional_routing_destination() {
        let capture = capture();
        let unbound = CaptureDeliveryEnvelopeV1::new(capture.clone(), None, 1_800_000_010).unwrap();
        assert!(unbound.destination.is_none());

        let routed_elsewhere = CaptureDeliveryEnvelopeV1::new(
            capture,
            Some(
                CaptureDeliveryDestinationV1::new(
                    project_id("prj_abcdef0123456789abcdef0123456789"),
                    7,
                )
                .unwrap(),
            ),
            1_800_000_010,
        )
        .unwrap();
        assert_ne!(unbound.envelope_id, routed_elsewhere.envelope_id);
        assert_ne!(
            routed_elsewhere.capture.binding.project_id,
            routed_elsewhere.destination.unwrap().project_id
        );
    }

    #[test]
    fn record_enforces_one_bounded_causal_transition_chain() {
        let envelope = bound_envelope();
        let queued = CaptureDeliveryRecordV1::queued(&envelope, 1_800_000_011).unwrap();
        let delivering = queued
            .transition(
                CaptureDeliveryState::Delivering,
                1_800_000_012,
                CaptureDeliveryReason::DeliveryAttemptStarted,
                None,
            )
            .unwrap();
        let retry = delivering
            .transition(
                CaptureDeliveryState::RetryRequired,
                1_800_000_013,
                CaptureDeliveryReason::DeliveryProcessInterrupted,
                None,
            )
            .unwrap();
        let delivering = retry
            .transition(
                CaptureDeliveryState::Delivering,
                1_800_000_014,
                CaptureDeliveryReason::DeliveryRetryStarted,
                None,
            )
            .unwrap();
        let delivered = delivering
            .transition(
                CaptureDeliveryState::Delivered,
                1_800_000_015,
                CaptureDeliveryReason::DeliveryAccepted,
                None,
            )
            .unwrap();
        assert_eq!(delivered.generation, 5);
        assert_eq!(delivered.attempt_count, 2);
        assert_eq!(
            CaptureDeliveryRecordV1::from_json_slice(&delivered.to_canonical_json().unwrap())
                .unwrap(),
            delivered
        );

        assert_eq!(
            delivered.transition(
                CaptureDeliveryState::Delivering,
                1_800_000_016,
                CaptureDeliveryReason::DeliveryAttemptStarted,
                None,
            ),
            Err(ProjectError::InvalidDeliveryTransition)
        );
        assert_eq!(
            delivered.transition(
                CaptureDeliveryState::Acknowledged,
                1_800_000_016,
                CaptureDeliveryReason::DeliveryAcknowledged,
                None,
            ),
            Err(ProjectError::InvalidDeliveryTransition)
        );
        assert_eq!(
            delivered.transition(
                CaptureDeliveryState::Cancelled,
                1_800_000_010,
                CaptureDeliveryReason::DeliveryCancelled,
                None,
            ),
            Err(ProjectError::InvalidDeliveryTransition)
        );
    }

    #[test]
    fn acknowledgement_binds_destination_capture_and_revisions() {
        let envelope = bound_envelope();
        let acknowledgement = CaptureDeliveryAcknowledgementV1::new(
            &envelope,
            envelope.capture_id.clone(),
            5,
            1_800_000_020,
        )
        .unwrap();
        assert_eq!(
            CaptureDeliveryAcknowledgementV1::from_json_slice(
                &acknowledgement.to_canonical_json().unwrap()
            )
            .unwrap(),
            acknowledgement
        );
        assert_eq!(
            acknowledgement.acknowledgement_id.as_str().len(),
            DELIVERY_ACKNOWLEDGEMENT_ID_PREFIX.len() + 64
        );

        let queued = CaptureDeliveryRecordV1::queued(&envelope, 1_800_000_011).unwrap();
        let delivered = queued
            .transition(
                CaptureDeliveryState::Delivering,
                1_800_000_012,
                CaptureDeliveryReason::DeliveryAttemptStarted,
                None,
            )
            .unwrap()
            .transition(
                CaptureDeliveryState::Delivered,
                1_800_000_013,
                CaptureDeliveryReason::DeliveryAccepted,
                None,
            )
            .unwrap();
        let acknowledged = delivered
            .transition(
                CaptureDeliveryState::Acknowledged,
                1_800_000_020,
                CaptureDeliveryReason::DeliveryAcknowledged,
                Some(acknowledgement.acknowledgement_id.clone()),
            )
            .unwrap();
        assert_eq!(
            acknowledged.acknowledgement_id(),
            Some(&acknowledgement.acknowledgement_id)
        );
        assert_eq!(
            acknowledged.transition(
                CaptureDeliveryState::Cancelled,
                1_800_000_021,
                CaptureDeliveryReason::DeliveryCancelled,
                None,
            ),
            Err(ProjectError::InvalidDeliveryTransition)
        );

        let unbound = CaptureDeliveryEnvelopeV1::new(
            envelope.capture.clone(),
            None,
            envelope.created_at_unix,
        )
        .unwrap();
        assert_eq!(
            CaptureDeliveryAcknowledgementV1::new(
                &unbound,
                unbound.capture_id.clone(),
                5,
                1_800_000_020,
            ),
            Err(ProjectError::DeliveryAcknowledgementConflict)
        );

        let mut tampered = acknowledgement;
        tampered.resulting_project_revision += 1;
        assert_eq!(
            tampered.validate(),
            Err(ProjectError::DeliveryIdentityConflict)
        );
    }

    #[test]
    fn record_parser_rejects_unknown_fields_and_impossible_history() {
        let envelope = bound_envelope();
        let record = CaptureDeliveryRecordV1::queued(&envelope, 1_800_000_011).unwrap();
        let mut value = serde_json::to_value(record).unwrap();
        value["generation"] = serde_json::json!(2);
        let bytes = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            CaptureDeliveryRecordV1::from_json_slice(&bytes),
            Err(ProjectError::InvalidDeliveryDocument)
        );

        value["generation"] = serde_json::json!(1);
        value["transitions"][0]["reason_code"] = serde_json::json!("/Users/example/private");
        let bytes = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            CaptureDeliveryRecordV1::from_json_slice(&bytes),
            Err(ProjectError::InvalidDeliveryDocument)
        );
    }
}
