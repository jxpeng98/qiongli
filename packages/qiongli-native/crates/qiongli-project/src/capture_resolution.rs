use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ProjectError;
use crate::capture::CaptureId;
use crate::capture_delivery::{CaptureDeliveryState, DeliveryEnvelopeId};
use crate::model::{MAX_SEMANTIC_REVISION, ProjectId, ProjectStage, valid_lower_hex};

pub const CAPTURE_RESOLUTION_SCHEMA_VERSION: u32 = 1;
pub const CAPTURE_ASSIGNMENT_INTENT_DOCUMENT_KIND: &str = "qiongli-capture-assignment-intent";
pub const CAPTURE_ASSIGNMENT_RECEIPT_DOCUMENT_KIND: &str = "qiongli-capture-assignment-receipt";
pub const CAPTURE_RESOLUTION_PLAN_DOCUMENT_KIND: &str = "qiongli-capture-resolution-plan";
pub const CAPTURE_RESOLUTION_RECEIPT_DOCUMENT_KIND: &str = "qiongli-capture-resolution-receipt";
pub const CAPTURE_ASSIGNMENT_INTENT_ID_PREFIX: &str = "cai_";
pub const CAPTURE_ASSIGNMENT_RECEIPT_ID_PREFIX: &str = "car_";
pub const CAPTURE_RESOLUTION_ITEM_ID_PREFIX: &str = "cri_";
pub const CAPTURE_RESOLUTION_RECEIPT_ID_PREFIX: &str = "crr_";

const MAX_ASSIGNMENT_INTENT_BYTES: usize = 16 * 1024;
const MAX_ASSIGNMENT_RECEIPT_BYTES: usize = 24 * 1024;
const MAX_RESOLUTION_PLAN_BYTES: usize = 64 * 1024;
const MAX_RESOLUTION_RECEIPT_BYTES: usize = 64 * 1024;
const MAX_ITEMS_PER_CAPTURE_FIELD: u16 = 16;
const MAX_RESOLUTION_ITEMS: usize = 80;

macro_rules! define_resolution_id {
    ($name:ident, $prefix:expr) => {
        #[derive(Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, ProjectError> {
                let value = value.into();
                if value.len() != $prefix.len() + 64
                    || !value.starts_with($prefix)
                    || !valid_lower_hex(&value[$prefix.len()..], 64)
                {
                    return Err(ProjectError::InvalidResolutionDocument);
                }
                Ok(Self(value))
            }

            fn from_digest(digest: &str) -> Self {
                Self(format!("{}{}", $prefix, digest))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            fn validate(&self) -> Result<(), ProjectError> {
                Self::parse(self.0.clone()).map(|_| ())
            }
        }

        impl Debug for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }
    };
}

define_resolution_id!(
    CaptureAssignmentIntentId,
    CAPTURE_ASSIGNMENT_INTENT_ID_PREFIX
);
define_resolution_id!(
    CaptureAssignmentReceiptId,
    CAPTURE_ASSIGNMENT_RECEIPT_ID_PREFIX
);
define_resolution_id!(CaptureResolutionItemId, CAPTURE_RESOLUTION_ITEM_ID_PREFIX);
define_resolution_id!(
    CaptureResolutionReceiptId,
    CAPTURE_RESOLUTION_RECEIPT_ID_PREFIX
);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureResolutionArtifact {
    ResearchState,
    DecisionLog,
    CaptureHistory,
    ConsolidationHistory,
}

const REQUIRED_ARTIFACTS: [CaptureResolutionArtifact; 4] = [
    CaptureResolutionArtifact::ResearchState,
    CaptureResolutionArtifact::DecisionLog,
    CaptureResolutionArtifact::CaptureHistory,
    CaptureResolutionArtifact::ConsolidationHistory,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionArtifactObservationV1 {
    pub artifact: CaptureResolutionArtifact,
    pub sha256: Option<String>,
}

impl CaptureResolutionArtifactObservationV1 {
    #[must_use]
    pub fn new(artifact: CaptureResolutionArtifact, sha256: Option<String>) -> Self {
        Self { artifact, sha256 }
    }

    fn validate(&self) -> Result<(), ProjectError> {
        if self
            .sha256
            .as_deref()
            .is_some_and(|digest| !valid_lower_hex(digest, 64))
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureResolutionItemKind {
    SemanticChange,
    Decision,
    Evidence,
    Contradiction,
    NextAction,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureResolutionCounterpartState {
    Absent,
    ExactMatch,
    ExactIdentityDivergent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureResolutionDisposition {
    AcceptCurrent,
    AcceptCapture,
    RetainBoth,
    RejectCapture,
}

const ABSENT_DISPOSITIONS: [CaptureResolutionDisposition; 2] = [
    CaptureResolutionDisposition::AcceptCapture,
    CaptureResolutionDisposition::RejectCapture,
];
const EXACT_MATCH_DISPOSITIONS: [CaptureResolutionDisposition; 2] = [
    CaptureResolutionDisposition::AcceptCurrent,
    CaptureResolutionDisposition::RejectCapture,
];
const DIVERGENT_REPLACEMENT_DISPOSITIONS: [CaptureResolutionDisposition; 3] = [
    CaptureResolutionDisposition::AcceptCurrent,
    CaptureResolutionDisposition::AcceptCapture,
    CaptureResolutionDisposition::RejectCapture,
];
const DIVERGENT_COEXISTENCE_DISPOSITIONS: [CaptureResolutionDisposition; 4] = [
    CaptureResolutionDisposition::AcceptCurrent,
    CaptureResolutionDisposition::AcceptCapture,
    CaptureResolutionDisposition::RetainBoth,
    CaptureResolutionDisposition::RejectCapture,
];

#[must_use]
pub fn capture_resolution_allowed_dispositions(
    kind: CaptureResolutionItemKind,
    counterpart_state: CaptureResolutionCounterpartState,
) -> &'static [CaptureResolutionDisposition] {
    match counterpart_state {
        CaptureResolutionCounterpartState::Absent => &ABSENT_DISPOSITIONS,
        CaptureResolutionCounterpartState::ExactMatch => &EXACT_MATCH_DISPOSITIONS,
        CaptureResolutionCounterpartState::ExactIdentityDivergent => match kind {
            CaptureResolutionItemKind::SemanticChange => &DIVERGENT_REPLACEMENT_DISPOSITIONS,
            CaptureResolutionItemKind::Decision
            | CaptureResolutionItemKind::Evidence
            | CaptureResolutionItemKind::Contradiction
            | CaptureResolutionItemKind::NextAction => &DIVERGENT_COEXISTENCE_DISPOSITIONS,
        },
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionItemV1 {
    pub item_id: CaptureResolutionItemId,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub kind: CaptureResolutionItemKind,
    pub source_index: u16,
    pub source_item_sha256: String,
    pub counterpart_state: CaptureResolutionCounterpartState,
    pub current_item_sha256: Option<String>,
    pub allowed_dispositions: Vec<CaptureResolutionDisposition>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureResolutionItemIdentity<'a> {
    schema_version: u32,
    source_envelope_id: &'a DeliveryEnvelopeId,
    kind: CaptureResolutionItemKind,
    source_index: u16,
    source_item_sha256: &'a str,
}

impl CaptureResolutionItemV1 {
    pub fn new(
        source_envelope_id: DeliveryEnvelopeId,
        kind: CaptureResolutionItemKind,
        source_index: u16,
        source_item_sha256: impl Into<String>,
        counterpart_state: CaptureResolutionCounterpartState,
        current_item_sha256: Option<String>,
    ) -> Result<Self, ProjectError> {
        let source_item_sha256 = source_item_sha256.into();
        let identity = CaptureResolutionItemIdentity {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            source_envelope_id: &source_envelope_id,
            kind,
            source_index,
            source_item_sha256: &source_item_sha256,
        };
        let item = Self {
            item_id: CaptureResolutionItemId::from_digest(&canonical_digest(&identity)?),
            source_envelope_id,
            kind,
            source_index,
            source_item_sha256,
            counterpart_state,
            current_item_sha256,
            allowed_dispositions: capture_resolution_allowed_dispositions(kind, counterpart_state)
                .to_vec(),
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        self.item_id.validate()?;
        DeliveryEnvelopeId::parse(self.source_envelope_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        if self.source_index >= MAX_ITEMS_PER_CAPTURE_FIELD
            || !valid_lower_hex(&self.source_item_sha256, 64)
            || self
                .current_item_sha256
                .as_deref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        match self.counterpart_state {
            CaptureResolutionCounterpartState::Absent if self.current_item_sha256.is_none() => {}
            CaptureResolutionCounterpartState::ExactMatch
                if self.current_item_sha256.as_deref()
                    == Some(self.source_item_sha256.as_str()) => {}
            CaptureResolutionCounterpartState::ExactIdentityDivergent
                if self
                    .current_item_sha256
                    .as_deref()
                    .is_some_and(|digest| digest != self.source_item_sha256) => {}
            _ => return Err(ProjectError::InvalidResolutionDocument),
        }
        if self.allowed_dispositions
            != capture_resolution_allowed_dispositions(self.kind, self.counterpart_state)
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let identity = CaptureResolutionItemIdentity {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            source_envelope_id: &self.source_envelope_id,
            kind: self.kind,
            source_index: self.source_index,
            source_item_sha256: &self.source_item_sha256,
        };
        if self.item_id != CaptureResolutionItemId::from_digest(&canonical_digest(&identity)?) {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureAssignmentIntentBodyV1 {
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_envelope_sha256: String,
    pub source_record_state: CaptureDeliveryState,
    pub source_record_generation: u64,
    pub source_record_sha256: String,
    pub source_capture_id: CaptureId,
    pub source_capture_sha256: String,
    pub target_project_id: ProjectId,
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub target_stage: ProjectStage,
    pub target_manifest_sha256: String,
    pub observed_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub created_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureAssignmentIntentV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub intent_id: CaptureAssignmentIntentId,
    pub intent: CaptureAssignmentIntentBodyV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureAssignmentIntentIdentity<'a> {
    schema_version: u32,
    intent: &'a CaptureAssignmentIntentBodyV1,
}

impl CaptureAssignmentIntentV1 {
    pub fn new(mut intent: CaptureAssignmentIntentBodyV1) -> Result<Self, ProjectError> {
        intent
            .observed_artifacts
            .sort_by_key(|observation| observation.artifact);
        validate_assignment_intent_body(&intent)?;
        let identity = CaptureAssignmentIntentIdentity {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            intent: &intent,
        };
        let document = Self {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            document_kind: CAPTURE_ASSIGNMENT_INTENT_DOCUMENT_KIND.to_string(),
            intent_id: CaptureAssignmentIntentId::from_digest(&canonical_digest(&identity)?),
            intent,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        let document: Self = parse_document(bytes, MAX_ASSIGNMENT_INTENT_BYTES)?;
        document.validate()?;
        Ok(document)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_ASSIGNMENT_INTENT_BYTES)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        self.intent_id.validate()?;
        validate_assignment_intent_body(&self.intent)?;
        if self.schema_version != CAPTURE_RESOLUTION_SCHEMA_VERSION
            || self.document_kind != CAPTURE_ASSIGNMENT_INTENT_DOCUMENT_KIND
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let identity = CaptureAssignmentIntentIdentity {
            schema_version: self.schema_version,
            intent: &self.intent,
        };
        if self.intent_id != CaptureAssignmentIntentId::from_digest(&canonical_digest(&identity)?) {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        canonical_json(self, MAX_ASSIGNMENT_INTENT_BYTES).map(|_| ())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureAssignmentOutcome {
    Assigned,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureAssignmentResultV1 {
    pub outcome: CaptureAssignmentOutcome,
    pub derived_capture_id: Option<CaptureId>,
    pub derived_capture_sha256: Option<String>,
    pub child_envelope_id: Option<DeliveryEnvelopeId>,
}

impl CaptureAssignmentResultV1 {
    #[must_use]
    pub fn assigned(
        derived_capture_id: CaptureId,
        derived_capture_sha256: impl Into<String>,
        child_envelope_id: DeliveryEnvelopeId,
    ) -> Self {
        Self {
            outcome: CaptureAssignmentOutcome::Assigned,
            derived_capture_id: Some(derived_capture_id),
            derived_capture_sha256: Some(derived_capture_sha256.into()),
            child_envelope_id: Some(child_envelope_id),
        }
    }

    #[must_use]
    pub const fn rejected() -> Self {
        Self {
            outcome: CaptureAssignmentOutcome::Rejected,
            derived_capture_id: None,
            derived_capture_sha256: None,
            child_envelope_id: None,
        }
    }

    fn validate(&self) -> Result<(), ProjectError> {
        match (
            self.outcome,
            self.derived_capture_id.as_ref(),
            self.derived_capture_sha256.as_deref(),
            self.child_envelope_id.as_ref(),
        ) {
            (
                CaptureAssignmentOutcome::Assigned,
                Some(capture_id),
                Some(capture_sha256),
                Some(envelope_id),
            ) => {
                CaptureId::parse(capture_id.as_str().to_owned())
                    .map_err(|_| ProjectError::InvalidResolutionDocument)?;
                DeliveryEnvelopeId::parse(envelope_id.as_str().to_owned())
                    .map_err(|_| ProjectError::InvalidResolutionDocument)?;
                if !valid_lower_hex(capture_sha256, 64) {
                    return Err(ProjectError::InvalidResolutionDocument);
                }
            }
            (CaptureAssignmentOutcome::Rejected, None, None, None) => {}
            _ => return Err(ProjectError::InvalidResolutionDocument),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureAssignmentReceiptBodyV1 {
    pub intent_id: CaptureAssignmentIntentId,
    pub intent_sha256: String,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_envelope_sha256: String,
    pub source_capture_id: CaptureId,
    pub source_capture_sha256: String,
    pub target_project_id: ProjectId,
    pub target_library_revision: u64,
    pub target_project_revision: u64,
    pub target_stage: ProjectStage,
    pub target_manifest_sha256: String,
    pub observed_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub source_record_generation_before: u64,
    pub source_record_sha256_before: String,
    pub source_record_generation_after: u64,
    pub source_record_sha256_after: String,
    pub result: CaptureAssignmentResultV1,
    pub intent_created_at_unix: u64,
    pub decided_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureAssignmentReceiptV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub receipt_id: CaptureAssignmentReceiptId,
    pub receipt: CaptureAssignmentReceiptBodyV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureAssignmentReceiptIdentity<'a> {
    schema_version: u32,
    receipt: &'a CaptureAssignmentReceiptBodyV1,
}

impl CaptureAssignmentReceiptV1 {
    pub fn new(
        intent: &CaptureAssignmentIntentV1,
        result: CaptureAssignmentResultV1,
        source_record_generation_after: u64,
        source_record_sha256_after: impl Into<String>,
        decided_at_unix: u64,
    ) -> Result<Self, ProjectError> {
        intent.validate()?;
        let receipt = CaptureAssignmentReceiptBodyV1 {
            intent_id: intent.intent_id.clone(),
            intent_sha256: canonical_digest(intent)?,
            source_envelope_id: intent.intent.source_envelope_id.clone(),
            source_envelope_sha256: intent.intent.source_envelope_sha256.clone(),
            source_capture_id: intent.intent.source_capture_id.clone(),
            source_capture_sha256: intent.intent.source_capture_sha256.clone(),
            target_project_id: intent.intent.target_project_id.clone(),
            target_library_revision: intent.intent.expected_library_revision,
            target_project_revision: intent.intent.expected_project_revision,
            target_stage: intent.intent.target_stage,
            target_manifest_sha256: intent.intent.target_manifest_sha256.clone(),
            observed_artifacts: intent.intent.observed_artifacts.clone(),
            source_record_generation_before: intent.intent.source_record_generation,
            source_record_sha256_before: intent.intent.source_record_sha256.clone(),
            source_record_generation_after,
            source_record_sha256_after: source_record_sha256_after.into(),
            result,
            intent_created_at_unix: intent.intent.created_at_unix,
            decided_at_unix,
        };
        let identity = CaptureAssignmentReceiptIdentity {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            receipt: &receipt,
        };
        let document = Self {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            document_kind: CAPTURE_ASSIGNMENT_RECEIPT_DOCUMENT_KIND.to_string(),
            receipt_id: CaptureAssignmentReceiptId::from_digest(&canonical_digest(&identity)?),
            receipt,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        let document: Self = parse_document(bytes, MAX_ASSIGNMENT_RECEIPT_BYTES)?;
        document.validate()?;
        Ok(document)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_ASSIGNMENT_RECEIPT_BYTES)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        self.receipt_id.validate()?;
        self.receipt.intent_id.validate()?;
        DeliveryEnvelopeId::parse(self.receipt.source_envelope_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        CaptureId::parse(self.receipt.source_capture_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        self.receipt
            .target_project_id
            .validate()
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        self.receipt.result.validate()?;
        let expected_record_generation_after = self
            .receipt
            .source_record_generation_before
            .checked_add(1)
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        if self.schema_version != CAPTURE_RESOLUTION_SCHEMA_VERSION
            || self.document_kind != CAPTURE_ASSIGNMENT_RECEIPT_DOCUMENT_KIND
            || !valid_lower_hex(&self.receipt.intent_sha256, 64)
            || !valid_lower_hex(&self.receipt.source_envelope_sha256, 64)
            || !valid_lower_hex(&self.receipt.source_capture_sha256, 64)
            || !valid_revision(self.receipt.target_library_revision)
            || !valid_revision(self.receipt.target_project_revision)
            || !valid_lower_hex(&self.receipt.target_manifest_sha256, 64)
            || self.receipt.source_record_generation_before == 0
            || self.receipt.source_record_generation_after != expected_record_generation_after
            || !valid_lower_hex(&self.receipt.source_record_sha256_before, 64)
            || !valid_lower_hex(&self.receipt.source_record_sha256_after, 64)
            || self.receipt.source_record_sha256_before == self.receipt.source_record_sha256_after
            || self.receipt.intent_created_at_unix > self.receipt.decided_at_unix
            || self.receipt.decided_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        validate_observations(&self.receipt.observed_artifacts)?;
        let identity = CaptureAssignmentReceiptIdentity {
            schema_version: self.schema_version,
            receipt: &self.receipt,
        };
        if self.receipt_id != CaptureAssignmentReceiptId::from_digest(&canonical_digest(&identity)?)
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        canonical_json(self, MAX_ASSIGNMENT_RECEIPT_BYTES).map(|_| ())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionPlanInputV1 {
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub target_stage: ProjectStage,
    pub target_manifest_sha256: String,
    pub observed_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub items: Vec<CaptureResolutionItemV1>,
    pub reviewed_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionPlanBodyV1 {
    pub assignment_receipt_id: CaptureAssignmentReceiptId,
    pub assignment_receipt_sha256: String,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_envelope_sha256: String,
    pub source_record_generation: u64,
    pub source_record_sha256: String,
    pub source_capture_id: CaptureId,
    pub source_capture_sha256: String,
    pub derived_capture_id: CaptureId,
    pub derived_capture_sha256: String,
    pub child_envelope_id: DeliveryEnvelopeId,
    pub target_project_id: ProjectId,
    pub assigned_library_revision: u64,
    pub assigned_project_revision: u64,
    pub expected_library_revision: u64,
    pub expected_project_revision: u64,
    pub target_stage: ProjectStage,
    pub target_manifest_sha256: String,
    pub observed_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub item_set_sha256: String,
    pub items: Vec<CaptureResolutionItemV1>,
    pub assignment_decided_at_unix: u64,
    pub reviewed_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionPlanV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub plan_digest: String,
    pub plan: CaptureResolutionPlanBodyV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureResolutionPlanIdentity<'a> {
    schema_version: u32,
    plan: &'a CaptureResolutionPlanBodyV1,
}

impl CaptureResolutionPlanV1 {
    pub fn new(
        assignment: &CaptureAssignmentReceiptV1,
        mut input: CaptureResolutionPlanInputV1,
    ) -> Result<Self, ProjectError> {
        assignment.validate()?;
        input
            .observed_artifacts
            .sort_by_key(|observation| observation.artifact);
        input
            .items
            .sort_by_key(|item| (item.kind, item.source_index));
        let derived_capture_id = assignment
            .receipt
            .result
            .derived_capture_id
            .clone()
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let child_envelope_id = assignment
            .receipt
            .result
            .child_envelope_id
            .clone()
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let plan = CaptureResolutionPlanBodyV1 {
            assignment_receipt_id: assignment.receipt_id.clone(),
            assignment_receipt_sha256: canonical_digest(assignment)?,
            source_envelope_id: assignment.receipt.source_envelope_id.clone(),
            source_envelope_sha256: assignment.receipt.source_envelope_sha256.clone(),
            source_record_generation: assignment.receipt.source_record_generation_after,
            source_record_sha256: assignment.receipt.source_record_sha256_after.clone(),
            source_capture_id: assignment.receipt.source_capture_id.clone(),
            source_capture_sha256: assignment.receipt.source_capture_sha256.clone(),
            derived_capture_id,
            derived_capture_sha256: assignment
                .receipt
                .result
                .derived_capture_sha256
                .clone()
                .ok_or(ProjectError::InvalidResolutionDocument)?,
            child_envelope_id,
            target_project_id: assignment.receipt.target_project_id.clone(),
            assigned_library_revision: assignment.receipt.target_library_revision,
            assigned_project_revision: assignment.receipt.target_project_revision,
            expected_library_revision: input.expected_library_revision,
            expected_project_revision: input.expected_project_revision,
            target_stage: input.target_stage,
            target_manifest_sha256: input.target_manifest_sha256,
            observed_artifacts: input.observed_artifacts,
            item_set_sha256: canonical_digest(&input.items)?,
            items: input.items,
            assignment_decided_at_unix: assignment.receipt.decided_at_unix,
            reviewed_at_unix: input.reviewed_at_unix,
        };
        let identity = CaptureResolutionPlanIdentity {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            plan: &plan,
        };
        let document = Self {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            document_kind: CAPTURE_RESOLUTION_PLAN_DOCUMENT_KIND.to_string(),
            plan_digest: canonical_digest(&identity)?,
            plan,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        let document: Self = parse_document(bytes, MAX_RESOLUTION_PLAN_BYTES)?;
        document.validate()?;
        Ok(document)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_RESOLUTION_PLAN_BYTES)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        self.plan.assignment_receipt_id.validate()?;
        DeliveryEnvelopeId::parse(self.plan.source_envelope_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        DeliveryEnvelopeId::parse(self.plan.child_envelope_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        CaptureId::parse(self.plan.source_capture_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        CaptureId::parse(self.plan.derived_capture_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        self.plan
            .target_project_id
            .validate()
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        if self.schema_version != CAPTURE_RESOLUTION_SCHEMA_VERSION
            || self.document_kind != CAPTURE_RESOLUTION_PLAN_DOCUMENT_KIND
            || !valid_lower_hex(&self.plan_digest, 64)
            || !valid_lower_hex(&self.plan.assignment_receipt_sha256, 64)
            || !valid_lower_hex(&self.plan.source_envelope_sha256, 64)
            || self.plan.source_record_generation == 0
            || !valid_lower_hex(&self.plan.source_record_sha256, 64)
            || !valid_lower_hex(&self.plan.source_capture_sha256, 64)
            || !valid_lower_hex(&self.plan.derived_capture_sha256, 64)
            || !valid_revision(self.plan.assigned_library_revision)
            || !valid_revision(self.plan.assigned_project_revision)
            || !valid_revision(self.plan.expected_library_revision)
            || !valid_revision(self.plan.expected_project_revision)
            || self.plan.expected_project_revision >= MAX_SEMANTIC_REVISION
            || self.plan.expected_library_revision < self.plan.assigned_library_revision
            || self.plan.expected_project_revision < self.plan.assigned_project_revision
            || !valid_lower_hex(&self.plan.target_manifest_sha256, 64)
            || !valid_lower_hex(&self.plan.item_set_sha256, 64)
            || self.plan.assignment_decided_at_unix > self.plan.reviewed_at_unix
            || self.plan.reviewed_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        validate_observations(&self.plan.observed_artifacts)?;
        validate_items(&self.plan.items, &self.plan.source_envelope_id)?;
        if canonical_digest(&self.plan.items)? != self.plan.item_set_sha256 {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let identity = CaptureResolutionPlanIdentity {
            schema_version: self.schema_version,
            plan: &self.plan,
        };
        if canonical_digest(&identity)? != self.plan_digest {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        canonical_json(self, MAX_RESOLUTION_PLAN_BYTES).map(|_| ())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionSelectionV1 {
    pub item_id: CaptureResolutionItemId,
    pub disposition: CaptureResolutionDisposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionDecisionV1 {
    pub item: CaptureResolutionItemV1,
    pub disposition: CaptureResolutionDisposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionResultV1 {
    pub resulting_manifest_sha256: String,
    pub resulting_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub resolved_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionReceiptBodyV1 {
    pub resolution_plan_digest: String,
    pub assignment_receipt_id: CaptureAssignmentReceiptId,
    pub assignment_receipt_sha256: String,
    pub source_envelope_id: DeliveryEnvelopeId,
    pub source_envelope_sha256: String,
    pub source_record_generation: u64,
    pub source_record_sha256: String,
    pub source_capture_id: CaptureId,
    pub source_capture_sha256: String,
    pub derived_capture_id: CaptureId,
    pub derived_capture_sha256: String,
    pub child_envelope_id: DeliveryEnvelopeId,
    pub target_project_id: ProjectId,
    pub assigned_library_revision: u64,
    pub assigned_project_revision: u64,
    pub expected_library_revision: u64,
    pub target_stage: ProjectStage,
    pub from_project_revision: u64,
    pub to_project_revision: u64,
    pub previous_manifest_sha256: String,
    pub resulting_manifest_sha256: String,
    pub observed_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub resulting_artifacts: Vec<CaptureResolutionArtifactObservationV1>,
    pub item_set_sha256: String,
    pub decisions: Vec<CaptureResolutionDecisionV1>,
    pub reviewed_at_unix: u64,
    pub resolved_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureResolutionReceiptV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub receipt_id: CaptureResolutionReceiptId,
    pub receipt: CaptureResolutionReceiptBodyV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureResolutionReceiptIdentity<'a> {
    schema_version: u32,
    receipt: &'a CaptureResolutionReceiptBodyV1,
}

impl CaptureResolutionReceiptV1 {
    pub fn new(
        plan: &CaptureResolutionPlanV1,
        selections: Vec<CaptureResolutionSelectionV1>,
        mut result: CaptureResolutionResultV1,
    ) -> Result<Self, ProjectError> {
        plan.validate()?;
        result
            .resulting_artifacts
            .sort_by_key(|observation| observation.artifact);
        if selections.len() != plan.plan.items.len() {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let mut decisions = Vec::with_capacity(selections.len());
        for (item, selection) in plan.plan.items.iter().zip(selections) {
            selection.item_id.validate()?;
            if selection.item_id != item.item_id
                || !item.allowed_dispositions.contains(&selection.disposition)
            {
                return Err(ProjectError::InvalidResolutionDocument);
            }
            decisions.push(CaptureResolutionDecisionV1 {
                item: item.clone(),
                disposition: selection.disposition,
            });
        }
        let to_project_revision = plan
            .plan
            .expected_project_revision
            .checked_add(1)
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        let receipt = CaptureResolutionReceiptBodyV1 {
            resolution_plan_digest: plan.plan_digest.clone(),
            assignment_receipt_id: plan.plan.assignment_receipt_id.clone(),
            assignment_receipt_sha256: plan.plan.assignment_receipt_sha256.clone(),
            source_envelope_id: plan.plan.source_envelope_id.clone(),
            source_envelope_sha256: plan.plan.source_envelope_sha256.clone(),
            source_record_generation: plan.plan.source_record_generation,
            source_record_sha256: plan.plan.source_record_sha256.clone(),
            source_capture_id: plan.plan.source_capture_id.clone(),
            source_capture_sha256: plan.plan.source_capture_sha256.clone(),
            derived_capture_id: plan.plan.derived_capture_id.clone(),
            derived_capture_sha256: plan.plan.derived_capture_sha256.clone(),
            child_envelope_id: plan.plan.child_envelope_id.clone(),
            target_project_id: plan.plan.target_project_id.clone(),
            assigned_library_revision: plan.plan.assigned_library_revision,
            assigned_project_revision: plan.plan.assigned_project_revision,
            expected_library_revision: plan.plan.expected_library_revision,
            target_stage: plan.plan.target_stage,
            from_project_revision: plan.plan.expected_project_revision,
            to_project_revision,
            previous_manifest_sha256: plan.plan.target_manifest_sha256.clone(),
            resulting_manifest_sha256: result.resulting_manifest_sha256,
            observed_artifacts: plan.plan.observed_artifacts.clone(),
            resulting_artifacts: result.resulting_artifacts,
            item_set_sha256: plan.plan.item_set_sha256.clone(),
            decisions,
            reviewed_at_unix: plan.plan.reviewed_at_unix,
            resolved_at_unix: result.resolved_at_unix,
        };
        let identity = CaptureResolutionReceiptIdentity {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            receipt: &receipt,
        };
        let document = Self {
            schema_version: CAPTURE_RESOLUTION_SCHEMA_VERSION,
            document_kind: CAPTURE_RESOLUTION_RECEIPT_DOCUMENT_KIND.to_string(),
            receipt_id: CaptureResolutionReceiptId::from_digest(&canonical_digest(&identity)?),
            receipt,
        };
        document.validate()?;
        Ok(document)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ProjectError> {
        let document: Self = parse_document(bytes, MAX_RESOLUTION_RECEIPT_BYTES)?;
        document.validate()?;
        Ok(document)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ProjectError> {
        self.validate()?;
        canonical_json(self, MAX_RESOLUTION_RECEIPT_BYTES)
    }

    fn validate(&self) -> Result<(), ProjectError> {
        self.receipt_id.validate()?;
        self.receipt.assignment_receipt_id.validate()?;
        DeliveryEnvelopeId::parse(self.receipt.source_envelope_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        DeliveryEnvelopeId::parse(self.receipt.child_envelope_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        CaptureId::parse(self.receipt.source_capture_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        CaptureId::parse(self.receipt.derived_capture_id.as_str().to_owned())
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        self.receipt
            .target_project_id
            .validate()
            .map_err(|_| ProjectError::InvalidResolutionDocument)?;
        let expected_to_project_revision = self
            .receipt
            .from_project_revision
            .checked_add(1)
            .ok_or(ProjectError::InvalidResolutionDocument)?;
        if self.schema_version != CAPTURE_RESOLUTION_SCHEMA_VERSION
            || self.document_kind != CAPTURE_RESOLUTION_RECEIPT_DOCUMENT_KIND
            || !valid_lower_hex(&self.receipt.resolution_plan_digest, 64)
            || !valid_lower_hex(&self.receipt.assignment_receipt_sha256, 64)
            || !valid_lower_hex(&self.receipt.source_envelope_sha256, 64)
            || self.receipt.source_record_generation == 0
            || !valid_lower_hex(&self.receipt.source_record_sha256, 64)
            || !valid_lower_hex(&self.receipt.source_capture_sha256, 64)
            || !valid_lower_hex(&self.receipt.derived_capture_sha256, 64)
            || !valid_revision(self.receipt.assigned_library_revision)
            || !valid_revision(self.receipt.assigned_project_revision)
            || !valid_revision(self.receipt.expected_library_revision)
            || self.receipt.expected_library_revision < self.receipt.assigned_library_revision
            || !valid_revision(self.receipt.from_project_revision)
            || self.receipt.from_project_revision < self.receipt.assigned_project_revision
            || self.receipt.to_project_revision != expected_to_project_revision
            || !valid_revision(self.receipt.to_project_revision)
            || !valid_lower_hex(&self.receipt.previous_manifest_sha256, 64)
            || !valid_lower_hex(&self.receipt.resulting_manifest_sha256, 64)
            || self.receipt.previous_manifest_sha256 == self.receipt.resulting_manifest_sha256
            || !valid_lower_hex(&self.receipt.item_set_sha256, 64)
            || self.receipt.reviewed_at_unix > self.receipt.resolved_at_unix
            || self.receipt.resolved_at_unix > MAX_SEMANTIC_REVISION
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        validate_observations(&self.receipt.observed_artifacts)?;
        validate_observations(&self.receipt.resulting_artifacts)?;
        let items = self
            .receipt
            .decisions
            .iter()
            .map(|decision| decision.item.clone())
            .collect::<Vec<_>>();
        validate_items(&items, &self.receipt.source_envelope_id)?;
        if canonical_digest(&items)? != self.receipt.item_set_sha256 {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        for decision in &self.receipt.decisions {
            if !decision
                .item
                .allowed_dispositions
                .contains(&decision.disposition)
            {
                return Err(ProjectError::InvalidResolutionDocument);
            }
        }
        let identity = CaptureResolutionReceiptIdentity {
            schema_version: self.schema_version,
            receipt: &self.receipt,
        };
        if self.receipt_id != CaptureResolutionReceiptId::from_digest(&canonical_digest(&identity)?)
        {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        canonical_json(self, MAX_RESOLUTION_RECEIPT_BYTES).map(|_| ())
    }
}

fn validate_assignment_intent_body(
    intent: &CaptureAssignmentIntentBodyV1,
) -> Result<(), ProjectError> {
    DeliveryEnvelopeId::parse(intent.source_envelope_id.as_str().to_owned())
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    CaptureId::parse(intent.source_capture_id.as_str().to_owned())
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    intent
        .target_project_id
        .validate()
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    if !matches!(
        intent.source_record_state,
        CaptureDeliveryState::Queued
            | CaptureDeliveryState::RetryRequired
            | CaptureDeliveryState::Conflicted
    ) || !valid_lower_hex(&intent.source_envelope_sha256, 64)
        || intent.source_record_generation == 0
        || !valid_lower_hex(&intent.source_record_sha256, 64)
        || !valid_lower_hex(&intent.source_capture_sha256, 64)
        || !valid_revision(intent.expected_library_revision)
        || !valid_revision(intent.expected_project_revision)
        || !valid_lower_hex(&intent.target_manifest_sha256, 64)
        || intent.created_at_unix > MAX_SEMANTIC_REVISION
    {
        return Err(ProjectError::InvalidResolutionDocument);
    }
    validate_observations(&intent.observed_artifacts)
}

fn validate_observations(
    observations: &[CaptureResolutionArtifactObservationV1],
) -> Result<(), ProjectError> {
    if observations.len() != REQUIRED_ARTIFACTS.len() {
        return Err(ProjectError::InvalidResolutionDocument);
    }
    for (observation, required) in observations.iter().zip(REQUIRED_ARTIFACTS) {
        observation.validate()?;
        if observation.artifact != required {
            return Err(ProjectError::InvalidResolutionDocument);
        }
    }
    Ok(())
}

fn validate_items(
    items: &[CaptureResolutionItemV1],
    source_envelope_id: &DeliveryEnvelopeId,
) -> Result<(), ProjectError> {
    if items.len() > MAX_RESOLUTION_ITEMS {
        return Err(ProjectError::InvalidResolutionDocument);
    }
    let mut previous = None;
    for item in items {
        item.validate()?;
        if &item.source_envelope_id != source_envelope_id {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        let key = (item.kind, item.source_index);
        if previous.is_some_and(|previous| previous >= key) {
            return Err(ProjectError::InvalidResolutionDocument);
        }
        previous = Some(key);
    }
    Ok(())
}

fn valid_revision(revision: u64) -> bool {
    revision > 0 && revision <= MAX_SEMANTIC_REVISION
}

fn parse_document<T>(bytes: &[u8], maximum_bytes: usize) -> Result<T, ProjectError>
where
    T: for<'de> Deserialize<'de>,
{
    if bytes.len() > maximum_bytes {
        return Err(ProjectError::DocumentTooLarge);
    }
    let value = crate::json::parse_unique_json(bytes)
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    serde_json::from_value(value).map_err(|_| ProjectError::InvalidResolutionDocument)
}

fn canonical_json<T: Serialize>(value: &T, maximum_bytes: usize) -> Result<Vec<u8>, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    if bytes.len() > maximum_bytes {
        return Err(ProjectError::DocumentTooLarge);
    }
    Ok(bytes)
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidResolutionDocument)?;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn digest(character: char) -> String {
        character.to_string().repeat(64)
    }

    fn envelope_id(character: char) -> DeliveryEnvelopeId {
        DeliveryEnvelopeId::parse(format!("env_{}", digest(character))).unwrap()
    }

    fn capture_id(character: char) -> CaptureId {
        CaptureId::parse(format!("cap_{}", digest(character))).unwrap()
    }

    fn project_id(character: char) -> ProjectId {
        ProjectId::parse(format!("prj_{}", character.to_string().repeat(32))).unwrap()
    }

    fn observations(characters: [char; 4]) -> Vec<CaptureResolutionArtifactObservationV1> {
        REQUIRED_ARTIFACTS
            .into_iter()
            .zip(characters)
            .map(|(artifact, character)| {
                CaptureResolutionArtifactObservationV1::new(artifact, Some(digest(character)))
            })
            .collect()
    }

    fn intent() -> CaptureAssignmentIntentV1 {
        CaptureAssignmentIntentV1::new(CaptureAssignmentIntentBodyV1 {
            source_envelope_id: envelope_id('a'),
            source_envelope_sha256: digest('0'),
            source_record_state: CaptureDeliveryState::Conflicted,
            source_record_generation: 3,
            source_record_sha256: digest('1'),
            source_capture_id: capture_id('b'),
            source_capture_sha256: digest('2'),
            target_project_id: project_id('c'),
            expected_library_revision: 7,
            expected_project_revision: 4,
            target_stage: ProjectStage::Literature,
            target_manifest_sha256: digest('3'),
            observed_artifacts: observations(['4', '5', '6', '7']),
            created_at_unix: 1_800_000_010,
        })
        .unwrap()
    }

    fn assigned_receipt() -> CaptureAssignmentReceiptV1 {
        CaptureAssignmentReceiptV1::new(
            &intent(),
            CaptureAssignmentResultV1::assigned(capture_id('d'), digest('8'), envelope_id('e')),
            4,
            digest('9'),
            1_800_000_011,
        )
        .unwrap()
    }

    fn resolution_items(source_envelope_id: &DeliveryEnvelopeId) -> Vec<CaptureResolutionItemV1> {
        vec![
            CaptureResolutionItemV1::new(
                source_envelope_id.clone(),
                CaptureResolutionItemKind::Evidence,
                0,
                digest('b'),
                CaptureResolutionCounterpartState::ExactMatch,
                Some(digest('b')),
            )
            .unwrap(),
            CaptureResolutionItemV1::new(
                source_envelope_id.clone(),
                CaptureResolutionItemKind::SemanticChange,
                0,
                digest('a'),
                CaptureResolutionCounterpartState::Absent,
                None,
            )
            .unwrap(),
            CaptureResolutionItemV1::new(
                source_envelope_id.clone(),
                CaptureResolutionItemKind::Decision,
                0,
                digest('c'),
                CaptureResolutionCounterpartState::ExactIdentityDivergent,
                Some(digest('d')),
            )
            .unwrap(),
        ]
    }

    fn resolution_plan() -> CaptureResolutionPlanV1 {
        let assignment = assigned_receipt();
        CaptureResolutionPlanV1::new(
            &assignment,
            CaptureResolutionPlanInputV1 {
                expected_library_revision: 8,
                expected_project_revision: 4,
                target_stage: ProjectStage::Literature,
                target_manifest_sha256: digest('e'),
                observed_artifacts: observations(['1', '2', '3', '4']),
                items: resolution_items(&assignment.receipt.source_envelope_id),
                reviewed_at_unix: 1_800_000_012,
            },
        )
        .unwrap()
    }

    fn valid_selections(plan: &CaptureResolutionPlanV1) -> Vec<CaptureResolutionSelectionV1> {
        plan.plan
            .items
            .iter()
            .map(|item| CaptureResolutionSelectionV1 {
                item_id: item.item_id.clone(),
                disposition: match item.kind {
                    CaptureResolutionItemKind::SemanticChange => {
                        CaptureResolutionDisposition::AcceptCapture
                    }
                    CaptureResolutionItemKind::Decision => CaptureResolutionDisposition::RetainBoth,
                    CaptureResolutionItemKind::Evidence => {
                        CaptureResolutionDisposition::AcceptCurrent
                    }
                    CaptureResolutionItemKind::Contradiction
                    | CaptureResolutionItemKind::NextAction => unreachable!(),
                },
            })
            .collect()
    }

    fn resolution_result() -> CaptureResolutionResultV1 {
        CaptureResolutionResultV1 {
            resulting_manifest_sha256: digest('f'),
            resulting_artifacts: observations(['5', '6', '7', '8']),
            resolved_at_unix: 1_800_000_013,
        }
    }

    #[test]
    fn assignment_intent_is_content_addressed_strict_and_path_free() {
        let intent = intent();
        assert!(
            intent
                .intent_id
                .as_str()
                .starts_with(CAPTURE_ASSIGNMENT_INTENT_ID_PREFIX)
        );
        let bytes = intent.to_canonical_json().unwrap();
        assert_eq!(
            CaptureAssignmentIntentV1::from_json_slice(&bytes).unwrap(),
            intent
        );
        assert!(!String::from_utf8_lossy(&bytes).contains("/Users/"));
        assert!(!format!("{intent:?}").contains("rootPath"));

        let mut changed_body = intent.intent.clone();
        changed_body.expected_project_revision += 1;
        let changed = CaptureAssignmentIntentV1::new(changed_body).unwrap();
        assert_ne!(changed.intent_id, intent.intent_id);

        let mut unknown: Value = serde_json::from_slice(&bytes).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("rootPath".to_string(), json!("/Users/private/project"));
        assert_eq!(
            CaptureAssignmentIntentV1::from_json_slice(&serde_json::to_vec(&unknown).unwrap()),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let json = String::from_utf8(bytes).unwrap();
        let duplicate = json.replacen(
            "\"schemaVersion\":1",
            "\"schemaVersion\":1,\"schemaVersion\":1",
            1,
        );
        assert_eq!(
            CaptureAssignmentIntentV1::from_json_slice(duplicate.as_bytes()),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut invalid_state = intent.intent.clone();
        invalid_state.source_record_state = CaptureDeliveryState::Delivered;
        assert_eq!(
            CaptureAssignmentIntentV1::new(invalid_state),
            Err(ProjectError::InvalidResolutionDocument)
        );
    }

    #[test]
    fn item_identity_preserves_duplicates_and_freezes_disposition_capabilities() {
        let envelope_id = envelope_id('a');
        let first = CaptureResolutionItemV1::new(
            envelope_id.clone(),
            CaptureResolutionItemKind::SemanticChange,
            0,
            digest('1'),
            CaptureResolutionCounterpartState::Absent,
            None,
        )
        .unwrap();
        let duplicate_at_next_index = CaptureResolutionItemV1::new(
            envelope_id.clone(),
            CaptureResolutionItemKind::SemanticChange,
            1,
            digest('1'),
            CaptureResolutionCounterpartState::Absent,
            None,
        )
        .unwrap();
        assert_ne!(first.item_id, duplicate_at_next_index.item_id);
        assert_eq!(
            first.allowed_dispositions,
            vec![
                CaptureResolutionDisposition::AcceptCapture,
                CaptureResolutionDisposition::RejectCapture,
            ]
        );

        let divergent_decision = CaptureResolutionItemV1::new(
            envelope_id,
            CaptureResolutionItemKind::Decision,
            0,
            digest('2'),
            CaptureResolutionCounterpartState::ExactIdentityDivergent,
            Some(digest('3')),
        )
        .unwrap();
        assert!(
            divergent_decision
                .allowed_dispositions
                .contains(&CaptureResolutionDisposition::RetainBoth)
        );

        let mut invalid_counterpart = first.clone();
        invalid_counterpart.counterpart_state = CaptureResolutionCounterpartState::ExactMatch;
        assert_eq!(
            invalid_counterpart.validate(),
            Err(ProjectError::InvalidResolutionDocument)
        );
        let mut broadened = first;
        broadened
            .allowed_dispositions
            .push(CaptureResolutionDisposition::RetainBoth);
        assert_eq!(
            broadened.validate(),
            Err(ProjectError::InvalidResolutionDocument)
        );
    }

    #[test]
    fn assignment_receipts_bind_assigned_or_rejected_outcomes_exactly_once() {
        let intent = intent();
        let assigned = assigned_receipt();
        let bytes = assigned.to_canonical_json().unwrap();
        assert_eq!(
            CaptureAssignmentReceiptV1::from_json_slice(&bytes).unwrap(),
            assigned
        );
        assert_eq!(
            assigned.receipt.result.outcome,
            CaptureAssignmentOutcome::Assigned
        );

        let rejected = CaptureAssignmentReceiptV1::new(
            &intent,
            CaptureAssignmentResultV1::rejected(),
            4,
            digest('a'),
            1_800_000_011,
        )
        .unwrap();
        assert_eq!(
            rejected.receipt.result.outcome,
            CaptureAssignmentOutcome::Rejected
        );
        assert!(rejected.receipt.result.child_envelope_id.is_none());
        assert_ne!(assigned.receipt_id, rejected.receipt_id);

        assert_eq!(
            CaptureAssignmentReceiptV1::new(
                &intent,
                CaptureAssignmentResultV1::rejected(),
                5,
                digest('a'),
                1_800_000_011,
            ),
            Err(ProjectError::InvalidResolutionDocument)
        );
        assert_eq!(
            CaptureAssignmentReceiptV1::new(
                &intent,
                CaptureAssignmentResultV1::rejected(),
                4,
                digest('a'),
                1_800_000_009,
            ),
            Err(ProjectError::InvalidResolutionDocument)
        );
        let mut exhausted_intent_body = intent.intent.clone();
        exhausted_intent_body.source_record_generation = u64::MAX;
        let exhausted_intent = CaptureAssignmentIntentV1::new(exhausted_intent_body).unwrap();
        assert_eq!(
            CaptureAssignmentReceiptV1::new(
                &exhausted_intent,
                CaptureAssignmentResultV1::rejected(),
                u64::MAX,
                digest('a'),
                1_800_000_011,
            ),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut tampered = assigned;
        tampered.receipt.result.child_envelope_id = None;
        assert_eq!(
            tampered.to_canonical_json(),
            Err(ProjectError::InvalidResolutionDocument)
        );
    }

    #[test]
    fn resolution_plan_binds_complete_sorted_items_and_target_evidence() {
        let plan = resolution_plan();
        assert_eq!(
            plan.plan
                .items
                .iter()
                .map(|item| item.kind)
                .collect::<Vec<_>>(),
            vec![
                CaptureResolutionItemKind::SemanticChange,
                CaptureResolutionItemKind::Decision,
                CaptureResolutionItemKind::Evidence,
            ]
        );
        let bytes = plan.to_canonical_json().unwrap();
        assert_eq!(
            CaptureResolutionPlanV1::from_json_slice(&bytes).unwrap(),
            plan
        );

        let assignment = assigned_receipt();
        let duplicate = resolution_items(&assignment.receipt.source_envelope_id)[0].clone();
        let duplicate_plan = CaptureResolutionPlanV1::new(
            &assignment,
            CaptureResolutionPlanInputV1 {
                expected_library_revision: 8,
                expected_project_revision: 4,
                target_stage: ProjectStage::Literature,
                target_manifest_sha256: digest('e'),
                observed_artifacts: observations(['1', '2', '3', '4']),
                items: vec![duplicate.clone(), duplicate],
                reviewed_at_unix: 1_800_000_012,
            },
        );
        assert_eq!(duplicate_plan, Err(ProjectError::InvalidResolutionDocument));

        let rejected = CaptureAssignmentReceiptV1::new(
            &intent(),
            CaptureAssignmentResultV1::rejected(),
            4,
            digest('9'),
            1_800_000_011,
        )
        .unwrap();
        assert_eq!(
            CaptureResolutionPlanV1::new(
                &rejected,
                CaptureResolutionPlanInputV1 {
                    expected_library_revision: 8,
                    expected_project_revision: 4,
                    target_stage: ProjectStage::Literature,
                    target_manifest_sha256: digest('e'),
                    observed_artifacts: observations(['1', '2', '3', '4']),
                    items: Vec::new(),
                    reviewed_at_unix: 1_800_000_012,
                },
            ),
            Err(ProjectError::InvalidResolutionDocument)
        );
        assert_eq!(
            CaptureResolutionPlanV1::new(
                &assignment,
                CaptureResolutionPlanInputV1 {
                    expected_library_revision: 8,
                    expected_project_revision: MAX_SEMANTIC_REVISION,
                    target_stage: ProjectStage::Literature,
                    target_manifest_sha256: digest('e'),
                    observed_artifacts: observations(['1', '2', '3', '4']),
                    items: Vec::new(),
                    reviewed_at_unix: 1_800_000_012,
                },
            ),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut changed_input = CaptureResolutionPlanInputV1 {
            expected_library_revision: 8,
            expected_project_revision: 4,
            target_stage: ProjectStage::Literature,
            target_manifest_sha256: digest('e'),
            observed_artifacts: observations(['1', '2', '3', '4']),
            items: resolution_items(&assignment.receipt.source_envelope_id),
            reviewed_at_unix: 1_800_000_012,
        };
        let original = CaptureResolutionPlanV1::new(&assignment, changed_input.clone()).unwrap();
        changed_input.observed_artifacts[0].sha256 = Some(digest('9'));
        let changed = CaptureResolutionPlanV1::new(&assignment, changed_input).unwrap();
        assert_ne!(original.plan_digest, changed.plan_digest);
    }

    #[test]
    fn academic_receipt_requires_every_supported_decision_and_exact_results() {
        let plan = resolution_plan();
        let selections = valid_selections(&plan);
        let receipt =
            CaptureResolutionReceiptV1::new(&plan, selections.clone(), resolution_result())
                .unwrap();
        let bytes = receipt.to_canonical_json().unwrap();
        assert_eq!(
            CaptureResolutionReceiptV1::from_json_slice(&bytes).unwrap(),
            receipt
        );
        assert_eq!(receipt.receipt.decisions.len(), plan.plan.items.len());
        assert_eq!(
            receipt.receipt.to_project_revision,
            receipt.receipt.from_project_revision + 1
        );

        let mut incomplete = selections.clone();
        incomplete.pop();
        assert_eq!(
            CaptureResolutionReceiptV1::new(&plan, incomplete, resolution_result()),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut unsupported = selections;
        let semantic = plan
            .plan
            .items
            .iter()
            .position(|item| item.kind == CaptureResolutionItemKind::SemanticChange)
            .unwrap();
        unsupported[semantic].disposition = CaptureResolutionDisposition::RetainBoth;
        assert_eq!(
            CaptureResolutionReceiptV1::new(&plan, unsupported, resolution_result()),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut early = resolution_result();
        early.resolved_at_unix = plan.plan.reviewed_at_unix - 1;
        assert_eq!(
            CaptureResolutionReceiptV1::new(&plan, valid_selections(&plan), early),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut changed_result = resolution_result();
        changed_result.resulting_artifacts[0].sha256 = Some(digest('9'));
        let changed =
            CaptureResolutionReceiptV1::new(&plan, valid_selections(&plan), changed_result)
                .unwrap();
        assert_ne!(changed.receipt_id, receipt.receipt_id);

        let mut exhausted = receipt.clone();
        exhausted.receipt.from_project_revision = u64::MAX;
        exhausted.receipt.to_project_revision = u64::MAX;
        assert_eq!(
            exhausted.to_canonical_json(),
            Err(ProjectError::InvalidResolutionDocument)
        );

        let mut unknown: Value = serde_json::from_slice(&bytes).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("projectPath".to_string(), json!("/private/project"));
        assert_eq!(
            CaptureResolutionReceiptV1::from_json_slice(&serde_json::to_vec(&unknown).unwrap()),
            Err(ProjectError::InvalidResolutionDocument)
        );
    }

    #[test]
    fn every_resolution_document_reader_rejects_oversized_input() {
        assert_eq!(
            CaptureAssignmentIntentV1::from_json_slice(&vec![
                b' ';
                MAX_ASSIGNMENT_INTENT_BYTES + 1
            ]),
            Err(ProjectError::DocumentTooLarge)
        );
        assert_eq!(
            CaptureAssignmentReceiptV1::from_json_slice(&vec![
                b' ';
                MAX_ASSIGNMENT_RECEIPT_BYTES + 1
            ]),
            Err(ProjectError::DocumentTooLarge)
        );
        assert_eq!(
            CaptureResolutionPlanV1::from_json_slice(&vec![b' '; MAX_RESOLUTION_PLAN_BYTES + 1]),
            Err(ProjectError::DocumentTooLarge)
        );
        assert_eq!(
            CaptureResolutionReceiptV1::from_json_slice(&vec![
                b' ';
                MAX_RESOLUTION_RECEIPT_BYTES + 1
            ]),
            Err(ProjectError::DocumentTooLarge)
        );
    }
}
