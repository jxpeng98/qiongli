use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{OrchestrationTaskId, RunId};

pub const ARTIFACT_REVIEW_SCHEMA_VERSION: u32 = 1;

const MAX_PLAN_BYTES: usize = 262_144;
const MAX_CHECKPOINT_BYTES: usize = 262_144;
const MAX_ARTIFACTS: usize = 32;
const MAX_ARTIFACT_PATH_BYTES: usize = 256;
const MAX_ARTIFACT_BYTES: u64 = 1_048_576;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactReviewSourceKind {
    SingleTask,
    WorkerSynthesis,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCandidateOperation {
    Create,
    Update,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum QualityGateId {
    Q1,
    Q2,
    Q3,
    Q4,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityGateStatus {
    Pending,
    Pass,
    Warn,
    Fail,
    Blocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerVerdict {
    Accept,
    Revise,
    Block,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactReviewRunStatus {
    PendingReview,
    Reviewing,
    ReadyForApply,
    RevisionRequired,
    Blocked,
    Cancelled,
}

impl ArtifactReviewRunStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::ReadyForApply | Self::RevisionRequired | Self::Blocked | Self::Cancelled
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactReviewError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidPlan,
    InvalidCheckpoint,
    BindingMismatch,
    StaleGeneration,
    InvalidTransition,
    GateNotRequired,
    SerializationFailed,
}

impl ArtifactReviewError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "artifact-review-input-too-large",
            Self::InvalidJson => "artifact-review-json-invalid",
            Self::NonCanonicalJson => "artifact-review-json-noncanonical",
            Self::InvalidPlan => "artifact-review-plan-invalid",
            Self::InvalidCheckpoint => "artifact-review-checkpoint-invalid",
            Self::BindingMismatch => "artifact-review-checkpoint-binding-mismatch",
            Self::StaleGeneration => "artifact-review-generation-stale",
            Self::InvalidTransition => "artifact-review-transition-invalid",
            Self::GateNotRequired => "artifact-review-gate-not-required",
            Self::SerializationFailed => "artifact-review-serialization-failed",
        }
    }
}

impl Display for ArtifactReviewError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for ArtifactReviewError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactCandidateV1 {
    pub relative_path: String,
    pub operation: ArtifactCandidateOperation,
    pub prior_sha256: Option<String>,
    pub candidate_sha256: String,
    pub byte_count: u64,
}

impl ArtifactCandidateV1 {
    pub fn try_new(
        relative_path: impl Into<String>,
        operation: ArtifactCandidateOperation,
        prior_sha256: Option<String>,
        candidate_sha256: impl Into<String>,
        byte_count: u64,
    ) -> Result<Self, ArtifactReviewError> {
        let candidate = Self {
            relative_path: relative_path.into(),
            operation,
            prior_sha256,
            candidate_sha256: candidate_sha256.into(),
            byte_count,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    fn validate(&self) -> Result<(), ArtifactReviewError> {
        let operation_valid = match self.operation {
            ArtifactCandidateOperation::Create => self.prior_sha256.is_none(),
            ArtifactCandidateOperation::Update => self
                .prior_sha256
                .as_ref()
                .is_some_and(|digest| valid_sha256(digest) && digest != &self.candidate_sha256),
        };
        if !valid_relative_artifact_path(&self.relative_path)
            || !operation_valid
            || !valid_sha256(&self.candidate_sha256)
            || self.byte_count == 0
            || self.byte_count > MAX_ARTIFACT_BYTES
        {
            return Err(ArtifactReviewError::InvalidPlan);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactReviewPlanV1 {
    pub schema_version: u32,
    pub review_run_id: RunId,
    pub source_run_id: RunId,
    pub source_kind: ArtifactReviewSourceKind,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub task_id: OrchestrationTaskId,
    pub source_output_sha256: String,
    pub workflow_contract_sha256: String,
    pub capability_map_sha256: String,
    pub quality_gate_contract_sha256: String,
    pub artifacts: Vec<ArtifactCandidateV1>,
    pub required_gates: Vec<QualityGateId>,
}

impl ArtifactReviewPlanV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        review_run_id: RunId,
        source_run_id: RunId,
        source_kind: ArtifactReviewSourceKind,
        project_id: ProjectId,
        expected_project_revision: u64,
        task_id: OrchestrationTaskId,
        source_output_sha256: impl Into<String>,
        workflow_contract_sha256: impl Into<String>,
        capability_map_sha256: impl Into<String>,
        quality_gate_contract_sha256: impl Into<String>,
        mut artifacts: Vec<ArtifactCandidateV1>,
        mut required_gates: Vec<QualityGateId>,
    ) -> Result<Self, ArtifactReviewError> {
        artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        required_gates.sort_unstable();
        let plan = Self {
            schema_version: ARTIFACT_REVIEW_SCHEMA_VERSION,
            review_run_id,
            source_run_id,
            source_kind,
            project_id,
            expected_project_revision,
            task_id,
            source_output_sha256: source_output_sha256.into(),
            workflow_contract_sha256: workflow_contract_sha256.into(),
            capability_map_sha256: capability_map_sha256.into(),
            quality_gate_contract_sha256: quality_gate_contract_sha256.into(),
            artifacts,
            required_gates,
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn from_canonical_json(input: &[u8]) -> Result<Self, ArtifactReviewError> {
        if input.len() > MAX_PLAN_BYTES {
            return Err(ArtifactReviewError::InputTooLarge);
        }
        let plan =
            serde_json::from_slice::<Self>(input).map_err(|_| ArtifactReviewError::InvalidJson)?;
        plan.validate()?;
        if plan.to_canonical_json()? != input {
            return Err(ArtifactReviewError::NonCanonicalJson);
        }
        Ok(plan)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ArtifactReviewError> {
        self.validate()?;
        canonical_json(self, MAX_PLAN_BYTES)
    }

    pub fn digest(&self) -> Result<String, ArtifactReviewError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    pub fn new_checkpoint(&self) -> Result<ArtifactReviewCheckpointV1, ArtifactReviewError> {
        let checkpoint = ArtifactReviewCheckpointV1 {
            schema_version: ARTIFACT_REVIEW_SCHEMA_VERSION,
            review_run_id: self.review_run_id.clone(),
            source_run_id: self.source_run_id.clone(),
            project_id: self.project_id.clone(),
            expected_project_revision: self.expected_project_revision,
            task_id: self.task_id.clone(),
            plan_sha256: self.digest()?,
            generation: 0,
            status: ArtifactReviewRunStatus::PendingReview,
            gates: self
                .required_gates
                .iter()
                .map(|gate_id| QualityGateCheckpointV1 {
                    gate_id: *gate_id,
                    status: QualityGateStatus::Pending,
                    evidence_sha256: None,
                })
                .collect(),
            reviewer_verdict: None,
            review_sha256: None,
        };
        self.validate_checkpoint(&checkpoint)?;
        Ok(checkpoint)
    }

    pub fn restore_checkpoint(
        &self,
        input: &[u8],
    ) -> Result<ArtifactReviewCheckpointV1, ArtifactReviewError> {
        if input.len() > MAX_CHECKPOINT_BYTES {
            return Err(ArtifactReviewError::InputTooLarge);
        }
        let checkpoint = serde_json::from_slice::<ArtifactReviewCheckpointV1>(input)
            .map_err(|_| ArtifactReviewError::InvalidJson)?;
        if checkpoint.review_run_id != self.review_run_id
            || checkpoint.source_run_id != self.source_run_id
            || checkpoint.project_id != self.project_id
            || checkpoint.expected_project_revision != self.expected_project_revision
            || checkpoint.task_id != self.task_id
            || checkpoint.plan_sha256 != self.digest()?
        {
            return Err(ArtifactReviewError::BindingMismatch);
        }
        self.validate_checkpoint(&checkpoint)?;
        if checkpoint.to_canonical_json(self)? != input {
            return Err(ArtifactReviewError::NonCanonicalJson);
        }
        Ok(checkpoint)
    }

    fn validate(&self) -> Result<(), ArtifactReviewError> {
        let artifact_paths = self
            .artifacts
            .iter()
            .map(|artifact| artifact.relative_path.as_str())
            .collect::<BTreeSet<_>>();
        let gates = self.required_gates.iter().copied().collect::<BTreeSet<_>>();
        if self.schema_version != ARTIFACT_REVIEW_SCHEMA_VERSION
            || RunId::parse(self.review_run_id.as_str()).is_err()
            || RunId::parse(self.source_run_id.as_str()).is_err()
            || self.review_run_id == self.source_run_id
            || ProjectId::parse(self.project_id.as_str()).is_err()
            || OrchestrationTaskId::parse(self.task_id.as_str()).is_err()
            || self.expected_project_revision == 0
            || self.expected_project_revision > MAX_SAFE_INTEGER
            || !valid_sha256(&self.source_output_sha256)
            || !valid_sha256(&self.workflow_contract_sha256)
            || !valid_sha256(&self.capability_map_sha256)
            || !valid_sha256(&self.quality_gate_contract_sha256)
            || self.artifacts.is_empty()
            || self.artifacts.len() > MAX_ARTIFACTS
            || artifact_paths.len() != self.artifacts.len()
            || !strictly_sorted_artifacts(&self.artifacts)
            || self
                .artifacts
                .iter()
                .any(|artifact| artifact.validate().is_err())
            || self.required_gates.is_empty()
            || gates.len() != self.required_gates.len()
            || !strictly_sorted_gates(&self.required_gates)
        {
            return Err(ArtifactReviewError::InvalidPlan);
        }
        Ok(())
    }

    fn validate_checkpoint(
        &self,
        checkpoint: &ArtifactReviewCheckpointV1,
    ) -> Result<(), ArtifactReviewError> {
        if checkpoint.schema_version != ARTIFACT_REVIEW_SCHEMA_VERSION
            || checkpoint.review_run_id != self.review_run_id
            || checkpoint.source_run_id != self.source_run_id
            || checkpoint.project_id != self.project_id
            || checkpoint.expected_project_revision != self.expected_project_revision
            || checkpoint.task_id != self.task_id
            || checkpoint.plan_sha256 != self.digest()?
            || checkpoint.generation > MAX_SAFE_INTEGER
            || checkpoint.gates.len() != self.required_gates.len()
            || checkpoint
                .gates
                .iter()
                .zip(&self.required_gates)
                .any(|(checkpoint_gate, required_gate)| checkpoint_gate.gate_id != *required_gate)
            || checkpoint.gates.iter().any(|gate| !valid_gate(gate))
            || checkpoint
                .review_sha256
                .as_ref()
                .is_some_and(|digest| !valid_sha256(digest))
        {
            return Err(ArtifactReviewError::InvalidCheckpoint);
        }

        let all_pending = checkpoint
            .gates
            .iter()
            .all(|gate| gate.status == QualityGateStatus::Pending);
        let all_settled = checkpoint
            .gates
            .iter()
            .all(|gate| gate.status != QualityGateStatus::Pending);
        let all_pass = checkpoint
            .gates
            .iter()
            .all(|gate| gate.status == QualityGateStatus::Pass);
        let settled_count = checkpoint
            .gates
            .iter()
            .filter(|gate| gate.status != QualityGateStatus::Pending)
            .count() as u64;
        let review_absent =
            checkpoint.reviewer_verdict.is_none() && checkpoint.review_sha256.is_none();
        let status_valid = match checkpoint.status {
            ArtifactReviewRunStatus::PendingReview => {
                checkpoint.generation == 0 && all_pending && review_absent
            }
            ArtifactReviewRunStatus::Reviewing => {
                checkpoint.generation == 1 + settled_count && review_absent
            }
            ArtifactReviewRunStatus::ReadyForApply => {
                all_settled
                    && all_pass
                    && checkpoint.generation == 2 + settled_count
                    && checkpoint.reviewer_verdict == Some(ReviewerVerdict::Accept)
                    && checkpoint.review_sha256.is_some()
            }
            ArtifactReviewRunStatus::RevisionRequired => {
                all_settled
                    && checkpoint.generation == 2 + settled_count
                    && checkpoint.reviewer_verdict == Some(ReviewerVerdict::Revise)
                    && checkpoint.review_sha256.is_some()
            }
            ArtifactReviewRunStatus::Blocked => {
                all_settled
                    && checkpoint.generation == 2 + settled_count
                    && checkpoint.reviewer_verdict == Some(ReviewerVerdict::Block)
                    && checkpoint.review_sha256.is_some()
            }
            ArtifactReviewRunStatus::Cancelled => {
                review_absent
                    && if settled_count == 0 {
                        matches!(checkpoint.generation, 1 | 2)
                    } else {
                        checkpoint.generation == 2 + settled_count
                    }
            }
        };
        status_valid
            .then_some(())
            .ok_or(ArtifactReviewError::InvalidCheckpoint)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QualityGateCheckpointV1 {
    pub gate_id: QualityGateId,
    pub status: QualityGateStatus,
    pub evidence_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactReviewCheckpointV1 {
    pub schema_version: u32,
    pub review_run_id: RunId,
    pub source_run_id: RunId,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub task_id: OrchestrationTaskId,
    pub plan_sha256: String,
    pub generation: u64,
    pub status: ArtifactReviewRunStatus,
    pub gates: Vec<QualityGateCheckpointV1>,
    pub reviewer_verdict: Option<ReviewerVerdict>,
    pub review_sha256: Option<String>,
}

impl ArtifactReviewCheckpointV1 {
    pub fn to_canonical_json(
        &self,
        plan: &ArtifactReviewPlanV1,
    ) -> Result<Vec<u8>, ArtifactReviewError> {
        plan.validate_checkpoint(self)?;
        canonical_json(self, MAX_CHECKPOINT_BYTES)
    }

    pub fn begin_review(
        &mut self,
        plan: &ArtifactReviewPlanV1,
        expected_generation: u64,
    ) -> Result<(), ArtifactReviewError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != ArtifactReviewRunStatus::PendingReview {
            return Err(ArtifactReviewError::InvalidTransition);
        }
        self.status = ArtifactReviewRunStatus::Reviewing;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn record_gate(
        &mut self,
        plan: &ArtifactReviewPlanV1,
        expected_generation: u64,
        gate_id: QualityGateId,
        status: QualityGateStatus,
        evidence_sha256: impl Into<String>,
    ) -> Result<(), ArtifactReviewError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        let evidence_sha256 = evidence_sha256.into();
        if self.status != ArtifactReviewRunStatus::Reviewing
            || status == QualityGateStatus::Pending
            || !valid_sha256(&evidence_sha256)
        {
            return Err(ArtifactReviewError::InvalidTransition);
        }
        let gate = self
            .gates
            .iter_mut()
            .find(|gate| gate.gate_id == gate_id)
            .ok_or(ArtifactReviewError::GateNotRequired)?;
        if gate.status != QualityGateStatus::Pending {
            return Err(ArtifactReviewError::InvalidTransition);
        }
        gate.status = status;
        gate.evidence_sha256 = Some(evidence_sha256);
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn complete_review(
        &mut self,
        plan: &ArtifactReviewPlanV1,
        expected_generation: u64,
        verdict: ReviewerVerdict,
        review_sha256: impl Into<String>,
    ) -> Result<(), ArtifactReviewError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        let review_sha256 = review_sha256.into();
        let all_settled = self
            .gates
            .iter()
            .all(|gate| gate.status != QualityGateStatus::Pending);
        let all_pass = self
            .gates
            .iter()
            .all(|gate| gate.status == QualityGateStatus::Pass);
        if self.status != ArtifactReviewRunStatus::Reviewing
            || !all_settled
            || !valid_sha256(&review_sha256)
            || (verdict == ReviewerVerdict::Accept && !all_pass)
        {
            return Err(ArtifactReviewError::InvalidTransition);
        }
        self.reviewer_verdict = Some(verdict);
        self.review_sha256 = Some(review_sha256);
        self.status = match verdict {
            ReviewerVerdict::Accept => ArtifactReviewRunStatus::ReadyForApply,
            ReviewerVerdict::Revise => ArtifactReviewRunStatus::RevisionRequired,
            ReviewerVerdict::Block => ArtifactReviewRunStatus::Blocked,
        };
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn cancel(
        &mut self,
        plan: &ArtifactReviewPlanV1,
        expected_generation: u64,
    ) -> Result<(), ArtifactReviewError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status.is_terminal() {
            return Err(ArtifactReviewError::InvalidTransition);
        }
        self.status = ArtifactReviewRunStatus::Cancelled;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    fn require_generation(&self, expected_generation: u64) -> Result<(), ArtifactReviewError> {
        if self.generation != expected_generation {
            return Err(ArtifactReviewError::StaleGeneration);
        }
        Ok(())
    }

    fn advance_generation(&mut self) -> Result<(), ArtifactReviewError> {
        self.generation = self
            .generation
            .checked_add(1)
            .filter(|generation| *generation <= MAX_SAFE_INTEGER)
            .ok_or(ArtifactReviewError::InvalidCheckpoint)?;
        Ok(())
    }
}

fn valid_gate(gate: &QualityGateCheckpointV1) -> bool {
    match gate.status {
        QualityGateStatus::Pending => gate.evidence_sha256.is_none(),
        QualityGateStatus::Pass
        | QualityGateStatus::Warn
        | QualityGateStatus::Fail
        | QualityGateStatus::Blocked => gate
            .evidence_sha256
            .as_ref()
            .is_some_and(|digest| valid_sha256(digest)),
    }
}

fn strictly_sorted_artifacts(artifacts: &[ArtifactCandidateV1]) -> bool {
    artifacts
        .windows(2)
        .all(|pair| pair[0].relative_path < pair[1].relative_path)
}

fn strictly_sorted_gates(gates: &[QualityGateId]) -> bool {
    gates.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_relative_artifact_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= MAX_ARTIFACT_PATH_BYTES
        && !path.starts_with('/')
        && path != ".qiongli"
        && !path.starts_with(".qiongli/")
        && !path.ends_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && !path.contains('\0')
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

fn canonical_json<T: Serialize>(
    value: &T,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ArtifactReviewError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ArtifactReviewError::SerializationFailed)?;
    if bytes.len() > maximum_bytes {
        return Err(ArtifactReviewError::InputTooLarge);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_id(character: char) -> RunId {
        RunId::parse(format!("run_{}", character.to_string().repeat(32))).unwrap()
    }

    fn digest(character: char) -> String {
        character.to_string().repeat(64)
    }

    fn candidate(path: &str) -> ArtifactCandidateV1 {
        ArtifactCandidateV1::try_new(
            path,
            ArtifactCandidateOperation::Create,
            None,
            digest('a'),
            128,
        )
        .unwrap()
    }

    fn plan() -> ArtifactReviewPlanV1 {
        ArtifactReviewPlanV1::try_new(
            run_id('b'),
            run_id('a'),
            ArtifactReviewSourceKind::WorkerSynthesis,
            ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap(),
            7,
            OrchestrationTaskId::parse("B1").unwrap(),
            digest('1'),
            digest('2'),
            digest('3'),
            digest('4'),
            vec![candidate("synthesis.md"), candidate("search_strategy.md")],
            vec![QualityGateId::Q4, QualityGateId::Q2, QualityGateId::Q3],
        )
        .unwrap()
    }

    #[test]
    fn plan_is_canonical_and_checkpoint_stores_hashes_only() {
        let plan = plan();
        assert_eq!(plan.artifacts[0].relative_path, "search_strategy.md");
        assert_eq!(
            plan.required_gates,
            vec![QualityGateId::Q2, QualityGateId::Q3, QualityGateId::Q4]
        );
        let bytes = plan.to_canonical_json().unwrap();
        assert_eq!(
            ArtifactReviewPlanV1::from_canonical_json(&bytes).unwrap(),
            plan
        );

        let checkpoint = plan.new_checkpoint().unwrap();
        let serialized = String::from_utf8(checkpoint.to_canonical_json(&plan).unwrap()).unwrap();
        assert!(!serialized.contains("candidate body canary"));
        assert!(!serialized.contains("/Users/example/project"));
        assert!(String::from_utf8(bytes).unwrap().contains(&digest('a')));
    }

    #[test]
    fn all_pass_and_accept_is_the_only_ready_for_apply_path() {
        let plan = plan();
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.begin_review(&plan, 0).unwrap();
        checkpoint
            .record_gate(
                &plan,
                1,
                QualityGateId::Q2,
                QualityGateStatus::Pass,
                digest('5'),
            )
            .unwrap();
        checkpoint
            .record_gate(
                &plan,
                2,
                QualityGateId::Q3,
                QualityGateStatus::Pass,
                digest('6'),
            )
            .unwrap();
        checkpoint
            .record_gate(
                &plan,
                3,
                QualityGateId::Q4,
                QualityGateStatus::Pass,
                digest('7'),
            )
            .unwrap();
        checkpoint
            .complete_review(&plan, 4, ReviewerVerdict::Accept, digest('8'))
            .unwrap();

        assert_eq!(checkpoint.status, ArtifactReviewRunStatus::ReadyForApply);
        assert_eq!(checkpoint.generation, 5);
        let bytes = checkpoint.to_canonical_json(&plan).unwrap();
        assert_eq!(plan.restore_checkpoint(&bytes).unwrap(), checkpoint);
    }

    #[test]
    fn warnings_cannot_be_overridden_by_an_accept_verdict() {
        let plan = plan();
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.begin_review(&plan, 0).unwrap();
        checkpoint
            .record_gate(
                &plan,
                1,
                QualityGateId::Q2,
                QualityGateStatus::Pass,
                digest('5'),
            )
            .unwrap();
        checkpoint
            .record_gate(
                &plan,
                2,
                QualityGateId::Q3,
                QualityGateStatus::Warn,
                digest('6'),
            )
            .unwrap();
        checkpoint
            .record_gate(
                &plan,
                3,
                QualityGateId::Q4,
                QualityGateStatus::Pass,
                digest('7'),
            )
            .unwrap();
        assert_eq!(
            checkpoint.complete_review(&plan, 4, ReviewerVerdict::Accept, digest('8')),
            Err(ArtifactReviewError::InvalidTransition)
        );
        checkpoint
            .complete_review(&plan, 4, ReviewerVerdict::Revise, digest('8'))
            .unwrap();
        assert_eq!(checkpoint.status, ArtifactReviewRunStatus::RevisionRequired);
    }

    #[test]
    fn stale_and_duplicate_gate_mutations_fail_closed() {
        let plan = plan();
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.begin_review(&plan, 0).unwrap();
        assert_eq!(
            checkpoint.record_gate(
                &plan,
                0,
                QualityGateId::Q2,
                QualityGateStatus::Pass,
                digest('5')
            ),
            Err(ArtifactReviewError::StaleGeneration)
        );
        checkpoint
            .record_gate(
                &plan,
                1,
                QualityGateId::Q2,
                QualityGateStatus::Pass,
                digest('5'),
            )
            .unwrap();
        assert_eq!(
            checkpoint.record_gate(
                &plan,
                2,
                QualityGateId::Q2,
                QualityGateStatus::Pass,
                digest('6')
            ),
            Err(ArtifactReviewError::InvalidTransition)
        );
        assert_eq!(
            checkpoint.record_gate(
                &plan,
                2,
                QualityGateId::Q1,
                QualityGateStatus::Pass,
                digest('6')
            ),
            Err(ArtifactReviewError::GateNotRequired)
        );
    }

    #[test]
    fn canonical_restore_rejects_unknown_fields_and_plan_substitution() {
        let plan = plan();
        let checkpoint = plan.new_checkpoint().unwrap();
        let mut value = serde_json::from_slice::<serde_json::Value>(
            &checkpoint.to_canonical_json(&plan).unwrap(),
        )
        .unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("prompt".to_owned(), serde_json::json!("secret canary"));
        let unknown = serde_json_canonicalizer::to_vec(&value).unwrap();
        assert_eq!(
            plan.restore_checkpoint(&unknown),
            Err(ArtifactReviewError::InvalidJson)
        );

        let mut substituted = plan.clone();
        substituted.expected_project_revision = 8;
        assert_eq!(
            substituted.restore_checkpoint(&checkpoint.to_canonical_json(&plan).unwrap()),
            Err(ArtifactReviewError::BindingMismatch)
        );
    }

    #[test]
    fn invalid_artifact_boundaries_and_operations_are_rejected() {
        for path in [
            "",
            "/tmp/paper.md",
            "../paper.md",
            "notes/../paper.md",
            ".qiongli",
            ".qiongli/private.json",
            "C:/private/paper.md",
            "notes\\paper.md",
            "notes/",
        ] {
            assert_eq!(
                ArtifactCandidateV1::try_new(
                    path,
                    ArtifactCandidateOperation::Create,
                    None,
                    digest('a'),
                    1
                ),
                Err(ArtifactReviewError::InvalidPlan),
                "{path}"
            );
        }
        assert_eq!(
            ArtifactCandidateV1::try_new(
                "paper.md",
                ArtifactCandidateOperation::Update,
                None,
                digest('a'),
                1
            ),
            Err(ArtifactReviewError::InvalidPlan)
        );
        assert_eq!(
            ArtifactCandidateV1::try_new(
                "paper.md",
                ArtifactCandidateOperation::Update,
                Some(digest('a')),
                digest('a'),
                1
            ),
            Err(ArtifactReviewError::InvalidPlan)
        );
    }

    #[test]
    fn cancellation_is_terminal_and_retains_no_review_material() {
        let plan = plan();
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.begin_review(&plan, 0).unwrap();
        checkpoint.cancel(&plan, 1).unwrap();
        assert_eq!(checkpoint.status, ArtifactReviewRunStatus::Cancelled);
        assert!(checkpoint.reviewer_verdict.is_none());
        assert!(checkpoint.review_sha256.is_none());
        assert_eq!(
            checkpoint.cancel(&plan, 2),
            Err(ArtifactReviewError::InvalidTransition)
        );
    }
}
