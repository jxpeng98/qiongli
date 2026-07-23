use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{BackendId, OrchestrationTaskId, RunId, WorkerId};

pub const WORKER_ORCHESTRATION_SCHEMA_VERSION: u32 = 1;

const MAX_PLAN_BYTES: usize = 262_144;
const MAX_CHECKPOINT_BYTES: usize = 262_144;
const MAX_WORKERS: usize = 4;
const MAX_WORKER_ATTEMPTS: u8 = 3;
const MAX_GOAL_BYTES: usize = 512;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerOrchestrationMode {
    DelegatedWorkers,
    ReviewSwarm,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerMergePolicy {
    SynthesizeWithConflictMatrix,
    ConsensusThenGaps,
    ControllerAdjudication,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerBarrierFailurePolicy {
    Degrade,
    Block,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerBarrierStatus {
    Passed,
    Degraded,
    Blocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Planned,
    Running,
    Passed,
    Failed,
    Blocked,
    Skipped,
}

impl WorkerStatus {
    const fn is_settled(self) -> bool {
        matches!(
            self,
            Self::Passed | Self::Failed | Self::Blocked | Self::Skipped
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerOrchestrationRunStatus {
    Planned,
    Running,
    SynthesisReady,
    Synthesizing,
    ReviewReady,
    Reviewing,
    Completed,
    Blocked,
    Cancelled,
}

impl WorkerOrchestrationRunStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Blocked | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkerOrchestrationFailureCode {
    #[serde(rename = "worker-input-invalid")]
    WorkerInputInvalid,
    #[serde(rename = "worker-backend-unavailable")]
    BackendUnavailable,
    #[serde(rename = "worker-backend-rejected")]
    BackendRejected,
    #[serde(rename = "worker-backend-failed")]
    BackendFailed,
    #[serde(rename = "worker-output-invalid")]
    WorkerOutputInvalid,
    #[serde(rename = "worker-interrupted")]
    WorkerInterrupted,
    #[serde(rename = "worker-barrier-blocked")]
    BarrierBlocked,
    #[serde(rename = "worker-synthesis-failed")]
    SynthesisFailed,
    #[serde(rename = "worker-review-failed")]
    ReviewFailed,
    #[serde(rename = "worker-review-blocked")]
    ReviewBlocked,
    #[serde(rename = "worker-run-cancelled")]
    RunCancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerOrchestrationError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidPlan,
    InvalidCheckpoint,
    BindingMismatch,
    StaleGeneration,
    InvalidTransition,
    WorkerNotReady,
    LimitExceeded,
    SerializationFailed,
}

impl WorkerOrchestrationError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "worker-orchestration-input-too-large",
            Self::InvalidJson => "worker-orchestration-json-invalid",
            Self::NonCanonicalJson => "worker-orchestration-json-noncanonical",
            Self::InvalidPlan => "worker-orchestration-plan-invalid",
            Self::InvalidCheckpoint => "worker-orchestration-checkpoint-invalid",
            Self::BindingMismatch => "worker-orchestration-checkpoint-binding-mismatch",
            Self::StaleGeneration => "worker-orchestration-generation-stale",
            Self::InvalidTransition => "worker-orchestration-transition-invalid",
            Self::WorkerNotReady => "worker-orchestration-worker-not-ready",
            Self::LimitExceeded => "worker-orchestration-limit-exhausted",
            Self::SerializationFailed => "worker-orchestration-serialization-failed",
        }
    }
}

impl Display for WorkerOrchestrationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for WorkerOrchestrationError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkerSpecV1 {
    pub worker_id: WorkerId,
    pub backend_id: BackendId,
    pub goal: String,
    pub functional_role: WorkerId,
}

impl WorkerSpecV1 {
    pub fn try_new(
        worker_id: impl Into<String>,
        backend_id: BackendId,
        goal: impl Into<String>,
        functional_role: impl Into<String>,
    ) -> Result<Self, WorkerOrchestrationError> {
        let worker = Self {
            worker_id: WorkerId::parse(worker_id.into())
                .map_err(|_| WorkerOrchestrationError::InvalidPlan)?,
            backend_id,
            goal: goal.into(),
            functional_role: WorkerId::parse(functional_role.into())
                .map_err(|_| WorkerOrchestrationError::InvalidPlan)?,
        };
        worker.validate()?;
        Ok(worker)
    }

    fn validate(&self) -> Result<(), WorkerOrchestrationError> {
        if WorkerId::parse(self.worker_id.as_str()).is_err()
            || BackendId::parse(self.backend_id.as_str()).is_err()
            || WorkerId::parse(self.functional_role.as_str()).is_err()
            || !valid_goal(&self.goal)
        {
            return Err(WorkerOrchestrationError::InvalidPlan);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkerOrchestrationPlanV1 {
    pub schema_version: u32,
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub task_id: OrchestrationTaskId,
    pub mode: WorkerOrchestrationMode,
    pub merge_policy: WorkerMergePolicy,
    pub barrier_failure_policy: WorkerBarrierFailurePolicy,
    pub required_successes: u8,
    pub max_worker_attempts: u8,
    pub synthesis_backend_id: BackendId,
    pub review_backend_id: BackendId,
    pub workers: Vec<WorkerSpecV1>,
}

impl WorkerOrchestrationPlanV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        run_id: RunId,
        project_id: ProjectId,
        expected_project_revision: u64,
        task_id: OrchestrationTaskId,
        mode: WorkerOrchestrationMode,
        merge_policy: WorkerMergePolicy,
        barrier_failure_policy: WorkerBarrierFailurePolicy,
        required_successes: u8,
        max_worker_attempts: u8,
        synthesis_backend_id: BackendId,
        review_backend_id: BackendId,
        mut workers: Vec<WorkerSpecV1>,
    ) -> Result<Self, WorkerOrchestrationError> {
        workers.sort_by(|left, right| left.worker_id.cmp(&right.worker_id));
        let plan = Self {
            schema_version: WORKER_ORCHESTRATION_SCHEMA_VERSION,
            run_id,
            project_id,
            expected_project_revision,
            task_id,
            mode,
            merge_policy,
            barrier_failure_policy,
            required_successes,
            max_worker_attempts,
            synthesis_backend_id,
            review_backend_id,
            workers,
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn from_canonical_json(input: &[u8]) -> Result<Self, WorkerOrchestrationError> {
        if input.len() > MAX_PLAN_BYTES {
            return Err(WorkerOrchestrationError::InputTooLarge);
        }
        let plan = serde_json::from_slice::<Self>(input)
            .map_err(|_| WorkerOrchestrationError::InvalidJson)?;
        plan.validate()?;
        if plan.to_canonical_json()? != input {
            return Err(WorkerOrchestrationError::NonCanonicalJson);
        }
        Ok(plan)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, WorkerOrchestrationError> {
        self.validate()?;
        canonical_json(self, MAX_PLAN_BYTES)
    }

    pub fn digest(&self) -> Result<String, WorkerOrchestrationError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    pub fn new_checkpoint(
        &self,
    ) -> Result<WorkerOrchestrationCheckpointV1, WorkerOrchestrationError> {
        let checkpoint = WorkerOrchestrationCheckpointV1 {
            schema_version: WORKER_ORCHESTRATION_SCHEMA_VERSION,
            run_id: self.run_id.clone(),
            project_id: self.project_id.clone(),
            expected_project_revision: self.expected_project_revision,
            task_id: self.task_id.clone(),
            plan_sha256: self.digest()?,
            generation: 0,
            status: WorkerOrchestrationRunStatus::Planned,
            barrier_status: None,
            workers: self
                .workers
                .iter()
                .map(|worker| WorkerCheckpointV1 {
                    worker_id: worker.worker_id.clone(),
                    backend_id: worker.backend_id.clone(),
                    status: WorkerStatus::Planned,
                    attempts: 0,
                    output_sha256: None,
                    failure_code: None,
                })
                .collect(),
            synthesis_output_sha256: None,
            review_output_sha256: None,
            failure_code: None,
        };
        self.validate_checkpoint(&checkpoint)?;
        Ok(checkpoint)
    }

    pub fn restore_checkpoint(
        &self,
        input: &[u8],
    ) -> Result<WorkerOrchestrationCheckpointV1, WorkerOrchestrationError> {
        if input.len() > MAX_CHECKPOINT_BYTES {
            return Err(WorkerOrchestrationError::InputTooLarge);
        }
        let checkpoint = serde_json::from_slice::<WorkerOrchestrationCheckpointV1>(input)
            .map_err(|_| WorkerOrchestrationError::InvalidJson)?;
        if checkpoint.run_id != self.run_id
            || checkpoint.project_id != self.project_id
            || checkpoint.expected_project_revision != self.expected_project_revision
            || checkpoint.task_id != self.task_id
            || checkpoint.plan_sha256 != self.digest()?
        {
            return Err(WorkerOrchestrationError::BindingMismatch);
        }
        self.validate_checkpoint(&checkpoint)?;
        if checkpoint.to_canonical_json(self)? != input {
            return Err(WorkerOrchestrationError::NonCanonicalJson);
        }
        Ok(checkpoint)
    }

    fn validate(&self) -> Result<(), WorkerOrchestrationError> {
        let worker_ids = self
            .workers
            .iter()
            .map(|worker| worker.worker_id.clone())
            .collect::<BTreeSet<_>>();
        if self.schema_version != WORKER_ORCHESTRATION_SCHEMA_VERSION
            || RunId::parse(self.run_id.as_str()).is_err()
            || ProjectId::parse(self.project_id.as_str()).is_err()
            || OrchestrationTaskId::parse(self.task_id.as_str()).is_err()
            || self.expected_project_revision == 0
            || self.expected_project_revision > MAX_SAFE_INTEGER
            || self.workers.is_empty()
            || self.workers.len() > MAX_WORKERS
            || worker_ids.len() != self.workers.len()
            || !strictly_sorted_workers(&self.workers)
            || self.required_successes == 0
            || usize::from(self.required_successes) > self.workers.len()
            || self.max_worker_attempts == 0
            || self.max_worker_attempts > MAX_WORKER_ATTEMPTS
            || BackendId::parse(self.synthesis_backend_id.as_str()).is_err()
            || BackendId::parse(self.review_backend_id.as_str()).is_err()
            || self.workers.iter().any(|worker| worker.validate().is_err())
        {
            return Err(WorkerOrchestrationError::InvalidPlan);
        }
        if self.barrier_failure_policy == WorkerBarrierFailurePolicy::Block
            && usize::from(self.required_successes) != self.workers.len()
        {
            return Err(WorkerOrchestrationError::InvalidPlan);
        }
        if self.mode == WorkerOrchestrationMode::ReviewSwarm
            && (self.barrier_failure_policy != WorkerBarrierFailurePolicy::Block
                || usize::from(self.required_successes) != self.workers.len())
        {
            return Err(WorkerOrchestrationError::InvalidPlan);
        }
        Ok(())
    }

    fn validate_checkpoint(
        &self,
        checkpoint: &WorkerOrchestrationCheckpointV1,
    ) -> Result<(), WorkerOrchestrationError> {
        if checkpoint.schema_version != WORKER_ORCHESTRATION_SCHEMA_VERSION
            || checkpoint.run_id != self.run_id
            || checkpoint.project_id != self.project_id
            || checkpoint.expected_project_revision != self.expected_project_revision
            || checkpoint.task_id != self.task_id
            || checkpoint.plan_sha256 != self.digest()?
            || checkpoint.generation > MAX_SAFE_INTEGER
            || checkpoint.workers.len() != self.workers.len()
            || checkpoint
                .workers
                .iter()
                .zip(&self.workers)
                .any(|(checkpoint_worker, worker)| {
                    checkpoint_worker.worker_id != worker.worker_id
                        || checkpoint_worker.backend_id != worker.backend_id
                })
            || checkpoint
                .workers
                .iter()
                .any(|worker| !valid_worker_checkpoint(worker, self.max_worker_attempts))
            || checkpoint
                .synthesis_output_sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
            || checkpoint
                .review_output_sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
        {
            return Err(WorkerOrchestrationError::InvalidCheckpoint);
        }

        let all_settled = checkpoint
            .workers
            .iter()
            .all(|worker| worker.status.is_settled());
        let expected_barrier = all_settled.then(|| self.evaluate_barrier(&checkpoint.workers));
        if checkpoint.barrier_status != expected_barrier {
            return Err(WorkerOrchestrationError::InvalidCheckpoint);
        }

        let status_valid = match checkpoint.status {
            WorkerOrchestrationRunStatus::Planned => {
                checkpoint.generation == 0
                    && checkpoint.workers.iter().all(|worker| {
                        worker.status == WorkerStatus::Planned && worker.attempts == 0
                    })
                    && checkpoint.barrier_status.is_none()
                    && checkpoint.failure_code.is_none()
                    && checkpoint.synthesis_output_sha256.is_none()
                    && checkpoint.review_output_sha256.is_none()
            }
            WorkerOrchestrationRunStatus::Running => {
                !all_settled
                    && checkpoint.workers.iter().any(|worker| {
                        matches!(worker.status, WorkerStatus::Planned | WorkerStatus::Running)
                    })
                    && checkpoint.failure_code.is_none()
                    && checkpoint.synthesis_output_sha256.is_none()
                    && checkpoint.review_output_sha256.is_none()
            }
            WorkerOrchestrationRunStatus::SynthesisReady
            | WorkerOrchestrationRunStatus::Synthesizing => {
                matches!(
                    checkpoint.barrier_status,
                    Some(WorkerBarrierStatus::Passed | WorkerBarrierStatus::Degraded)
                ) && checkpoint.failure_code.is_none()
                    && checkpoint.synthesis_output_sha256.is_none()
                    && checkpoint.review_output_sha256.is_none()
            }
            WorkerOrchestrationRunStatus::ReviewReady | WorkerOrchestrationRunStatus::Reviewing => {
                matches!(
                    checkpoint.barrier_status,
                    Some(WorkerBarrierStatus::Passed | WorkerBarrierStatus::Degraded)
                ) && checkpoint.failure_code.is_none()
                    && checkpoint.synthesis_output_sha256.is_some()
                    && checkpoint.review_output_sha256.is_none()
            }
            WorkerOrchestrationRunStatus::Completed => {
                matches!(
                    checkpoint.barrier_status,
                    Some(WorkerBarrierStatus::Passed | WorkerBarrierStatus::Degraded)
                ) && checkpoint.failure_code.is_none()
                    && checkpoint.synthesis_output_sha256.is_some()
                    && checkpoint.review_output_sha256.is_some()
            }
            WorkerOrchestrationRunStatus::Blocked => match checkpoint.failure_code {
                Some(WorkerOrchestrationFailureCode::BarrierBlocked) => {
                    checkpoint.barrier_status == Some(WorkerBarrierStatus::Blocked)
                        && checkpoint.synthesis_output_sha256.is_none()
                        && checkpoint.review_output_sha256.is_none()
                }
                Some(WorkerOrchestrationFailureCode::SynthesisFailed) => {
                    matches!(
                        checkpoint.barrier_status,
                        Some(WorkerBarrierStatus::Passed | WorkerBarrierStatus::Degraded)
                    ) && checkpoint.synthesis_output_sha256.is_none()
                        && checkpoint.review_output_sha256.is_none()
                }
                Some(
                    WorkerOrchestrationFailureCode::ReviewFailed
                    | WorkerOrchestrationFailureCode::ReviewBlocked,
                ) => {
                    matches!(
                        checkpoint.barrier_status,
                        Some(WorkerBarrierStatus::Passed | WorkerBarrierStatus::Degraded)
                    ) && checkpoint.synthesis_output_sha256.is_some()
                }
                _ => false,
            },
            WorkerOrchestrationRunStatus::Cancelled => {
                checkpoint.failure_code == Some(WorkerOrchestrationFailureCode::RunCancelled)
                    && checkpoint.workers.iter().all(|worker| {
                        !matches!(worker.status, WorkerStatus::Planned | WorkerStatus::Running)
                    })
            }
        };
        status_valid
            .then_some(())
            .ok_or(WorkerOrchestrationError::InvalidCheckpoint)
    }

    fn evaluate_barrier(&self, workers: &[WorkerCheckpointV1]) -> WorkerBarrierStatus {
        let passed = workers
            .iter()
            .filter(|worker| worker.status == WorkerStatus::Passed)
            .count();
        if passed == workers.len() {
            WorkerBarrierStatus::Passed
        } else if self.barrier_failure_policy == WorkerBarrierFailurePolicy::Degrade
            && passed >= usize::from(self.required_successes)
        {
            WorkerBarrierStatus::Degraded
        } else {
            WorkerBarrierStatus::Blocked
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkerCheckpointV1 {
    pub worker_id: WorkerId,
    pub backend_id: BackendId,
    pub status: WorkerStatus,
    pub attempts: u8,
    pub output_sha256: Option<String>,
    pub failure_code: Option<WorkerOrchestrationFailureCode>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkerOrchestrationCheckpointV1 {
    pub schema_version: u32,
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub task_id: OrchestrationTaskId,
    pub plan_sha256: String,
    pub generation: u64,
    pub status: WorkerOrchestrationRunStatus,
    pub barrier_status: Option<WorkerBarrierStatus>,
    pub workers: Vec<WorkerCheckpointV1>,
    pub synthesis_output_sha256: Option<String>,
    pub review_output_sha256: Option<String>,
    pub failure_code: Option<WorkerOrchestrationFailureCode>,
}

impl WorkerOrchestrationCheckpointV1 {
    #[must_use]
    pub fn next_planned_worker(&self) -> Option<&WorkerId> {
        self.workers
            .iter()
            .find(|worker| worker.status == WorkerStatus::Planned)
            .map(|worker| &worker.worker_id)
    }

    pub fn to_canonical_json(
        &self,
        plan: &WorkerOrchestrationPlanV1,
    ) -> Result<Vec<u8>, WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        canonical_json(self, MAX_CHECKPOINT_BYTES)
    }

    pub fn start(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::Planned {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.status = WorkerOrchestrationRunStatus::Running;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn begin_worker(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
        worker_id: &WorkerId,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::Running {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        let worker = self.worker_mut(worker_id)?;
        if worker.status != WorkerStatus::Planned {
            return Err(WorkerOrchestrationError::WorkerNotReady);
        }
        if worker.attempts >= plan.max_worker_attempts {
            return Err(WorkerOrchestrationError::LimitExceeded);
        }
        worker.attempts += 1;
        worker.status = WorkerStatus::Running;
        worker.output_sha256 = None;
        worker.failure_code = None;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn complete_worker(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
        worker_id: &WorkerId,
        output_sha256: impl Into<String>,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::Running {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        let output_sha256 = output_sha256.into();
        if !valid_sha256(&output_sha256) {
            return Err(WorkerOrchestrationError::InvalidCheckpoint);
        }
        let worker = self.worker_mut(worker_id)?;
        if worker.status != WorkerStatus::Running {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        worker.status = WorkerStatus::Passed;
        worker.output_sha256 = Some(output_sha256);
        worker.failure_code = None;
        self.advance_generation()?;
        self.recompute_barrier(plan)?;
        plan.validate_checkpoint(self)
    }

    pub fn fail_worker(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
        worker_id: &WorkerId,
        failure_code: WorkerOrchestrationFailureCode,
        retryable: bool,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::Running
            || !worker_failure_allowed(failure_code)
        {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        let worker = self.worker_mut(worker_id)?;
        if worker.status != WorkerStatus::Running {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        worker.output_sha256 = None;
        worker.failure_code = Some(failure_code);
        worker.status = if retryable && worker.attempts < plan.max_worker_attempts {
            WorkerStatus::Planned
        } else {
            WorkerStatus::Failed
        };
        self.advance_generation()?;
        self.recompute_barrier(plan)?;
        plan.validate_checkpoint(self)
    }

    pub fn begin_synthesis(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::SynthesisReady {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.status = WorkerOrchestrationRunStatus::Synthesizing;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn complete_synthesis(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
        output_sha256: impl Into<String>,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        let output_sha256 = output_sha256.into();
        if self.status != WorkerOrchestrationRunStatus::Synthesizing
            || !valid_sha256(&output_sha256)
        {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.synthesis_output_sha256 = Some(output_sha256);
        self.status = WorkerOrchestrationRunStatus::ReviewReady;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn fail_synthesis(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::Synthesizing {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.status = WorkerOrchestrationRunStatus::Blocked;
        self.failure_code = Some(WorkerOrchestrationFailureCode::SynthesisFailed);
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn begin_review(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::ReviewReady {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.status = WorkerOrchestrationRunStatus::Reviewing;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn complete_review(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
        output_sha256: impl Into<String>,
        passed: bool,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        let output_sha256 = output_sha256.into();
        if self.status != WorkerOrchestrationRunStatus::Reviewing || !valid_sha256(&output_sha256) {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.review_output_sha256 = Some(output_sha256);
        if passed {
            self.status = WorkerOrchestrationRunStatus::Completed;
            self.failure_code = None;
        } else {
            self.status = WorkerOrchestrationRunStatus::Blocked;
            self.failure_code = Some(WorkerOrchestrationFailureCode::ReviewBlocked);
        }
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn fail_review(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != WorkerOrchestrationRunStatus::Reviewing {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.status = WorkerOrchestrationRunStatus::Blocked;
        self.failure_code = Some(WorkerOrchestrationFailureCode::ReviewFailed);
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn recover_interrupted(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        let mut recovered = false;
        let mut recovered_workers = false;
        match self.status {
            WorkerOrchestrationRunStatus::Running => {
                for worker in &mut self.workers {
                    if worker.status == WorkerStatus::Running {
                        worker.status = if worker.attempts < plan.max_worker_attempts {
                            WorkerStatus::Planned
                        } else {
                            WorkerStatus::Failed
                        };
                        worker.output_sha256 = None;
                        worker.failure_code =
                            Some(WorkerOrchestrationFailureCode::WorkerInterrupted);
                        recovered = true;
                        recovered_workers = true;
                    }
                }
            }
            WorkerOrchestrationRunStatus::Synthesizing => {
                self.status = WorkerOrchestrationRunStatus::SynthesisReady;
                recovered = true;
            }
            WorkerOrchestrationRunStatus::Reviewing => {
                self.status = WorkerOrchestrationRunStatus::ReviewReady;
                recovered = true;
            }
            _ => {}
        }
        if !recovered {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        self.advance_generation()?;
        if recovered_workers {
            self.recompute_barrier(plan)?;
        }
        plan.validate_checkpoint(self)
    }

    pub fn cancel(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), WorkerOrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status.is_terminal() {
            return Err(WorkerOrchestrationError::InvalidTransition);
        }
        for worker in &mut self.workers {
            if matches!(worker.status, WorkerStatus::Planned | WorkerStatus::Running) {
                worker.status = WorkerStatus::Skipped;
                worker.output_sha256 = None;
                worker.failure_code = Some(WorkerOrchestrationFailureCode::RunCancelled);
            }
        }
        self.status = WorkerOrchestrationRunStatus::Cancelled;
        self.failure_code = Some(WorkerOrchestrationFailureCode::RunCancelled);
        self.barrier_status = self
            .workers
            .iter()
            .all(|worker| worker.status.is_settled())
            .then(|| plan.evaluate_barrier(&self.workers));
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    fn recompute_barrier(
        &mut self,
        plan: &WorkerOrchestrationPlanV1,
    ) -> Result<(), WorkerOrchestrationError> {
        if !self.workers.iter().all(|worker| worker.status.is_settled()) {
            self.barrier_status = None;
            return Ok(());
        }
        let barrier_status = plan.evaluate_barrier(&self.workers);
        self.barrier_status = Some(barrier_status);
        match barrier_status {
            WorkerBarrierStatus::Passed | WorkerBarrierStatus::Degraded => {
                self.status = WorkerOrchestrationRunStatus::SynthesisReady;
            }
            WorkerBarrierStatus::Blocked => {
                self.status = WorkerOrchestrationRunStatus::Blocked;
                self.failure_code = Some(WorkerOrchestrationFailureCode::BarrierBlocked);
            }
        }
        Ok(())
    }

    fn require_generation(&self, expected: u64) -> Result<(), WorkerOrchestrationError> {
        if self.generation != expected {
            return Err(WorkerOrchestrationError::StaleGeneration);
        }
        (self.generation < MAX_SAFE_INTEGER)
            .then_some(())
            .ok_or(WorkerOrchestrationError::LimitExceeded)
    }

    fn advance_generation(&mut self) -> Result<(), WorkerOrchestrationError> {
        self.generation = self
            .generation
            .checked_add(1)
            .filter(|generation| *generation <= MAX_SAFE_INTEGER)
            .ok_or(WorkerOrchestrationError::LimitExceeded)?;
        Ok(())
    }

    fn worker_mut(
        &mut self,
        worker_id: &WorkerId,
    ) -> Result<&mut WorkerCheckpointV1, WorkerOrchestrationError> {
        self.workers
            .iter_mut()
            .find(|worker| &worker.worker_id == worker_id)
            .ok_or(WorkerOrchestrationError::InvalidCheckpoint)
    }
}

fn valid_worker_checkpoint(checkpoint: &WorkerCheckpointV1, max_attempts: u8) -> bool {
    if WorkerId::parse(checkpoint.worker_id.as_str()).is_err()
        || BackendId::parse(checkpoint.backend_id.as_str()).is_err()
        || checkpoint.attempts > max_attempts
        || checkpoint
            .output_sha256
            .as_ref()
            .is_some_and(|value| !valid_sha256(value))
    {
        return false;
    }
    match checkpoint.status {
        WorkerStatus::Planned => {
            checkpoint.output_sha256.is_none()
                && (checkpoint.attempts == 0 && checkpoint.failure_code.is_none()
                    || checkpoint.attempts > 0
                        && checkpoint.failure_code.is_some_and(worker_failure_allowed))
        }
        WorkerStatus::Running => {
            checkpoint.attempts > 0
                && checkpoint.output_sha256.is_none()
                && checkpoint.failure_code.is_none()
        }
        WorkerStatus::Passed => {
            checkpoint.attempts > 0
                && checkpoint.output_sha256.is_some()
                && checkpoint.failure_code.is_none()
        }
        WorkerStatus::Failed | WorkerStatus::Blocked => {
            checkpoint.attempts > 0
                && checkpoint.output_sha256.is_none()
                && checkpoint.failure_code.is_some_and(worker_failure_allowed)
        }
        WorkerStatus::Skipped => {
            checkpoint.output_sha256.is_none()
                && checkpoint.failure_code == Some(WorkerOrchestrationFailureCode::RunCancelled)
        }
    }
}

const fn worker_failure_allowed(code: WorkerOrchestrationFailureCode) -> bool {
    matches!(
        code,
        WorkerOrchestrationFailureCode::WorkerInputInvalid
            | WorkerOrchestrationFailureCode::BackendUnavailable
            | WorkerOrchestrationFailureCode::BackendRejected
            | WorkerOrchestrationFailureCode::BackendFailed
            | WorkerOrchestrationFailureCode::WorkerOutputInvalid
            | WorkerOrchestrationFailureCode::WorkerInterrupted
    )
}

fn strictly_sorted_workers(workers: &[WorkerSpecV1]) -> bool {
    workers
        .windows(2)
        .all(|pair| pair[0].worker_id < pair[1].worker_id)
}

fn valid_goal(goal: &str) -> bool {
    let trimmed = goal.trim();
    !trimmed.is_empty()
        && trimmed == goal
        && goal.len() <= MAX_GOAL_BYTES
        && goal.chars().all(|character| !character.is_control())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn canonical_json<T: Serialize>(
    value: &T,
    maximum: usize,
) -> Result<Vec<u8>, WorkerOrchestrationError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| WorkerOrchestrationError::SerializationFailed)?;
    if bytes.len() > maximum {
        return Err(WorkerOrchestrationError::InputTooLarge);
    }
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: char) -> String {
        byte.to_string().repeat(64)
    }

    fn worker(id: &str) -> WorkerSpecV1 {
        WorkerSpecV1::try_new(
            id,
            BackendId::parse("fake").unwrap(),
            format!("Complete the {id} work unit"),
            id,
        )
        .unwrap()
    }

    fn delegated_plan(required_successes: u8) -> WorkerOrchestrationPlanV1 {
        WorkerOrchestrationPlanV1::try_new(
            RunId::parse(format!("run_{}", "a".repeat(32))).unwrap(),
            ProjectId::parse(format!("prj_{}", "1".repeat(32))).unwrap(),
            7,
            OrchestrationTaskId::parse("B1").unwrap(),
            WorkerOrchestrationMode::DelegatedWorkers,
            WorkerMergePolicy::SynthesizeWithConflictMatrix,
            WorkerBarrierFailurePolicy::Degrade,
            required_successes,
            2,
            BackendId::parse("fake").unwrap(),
            BackendId::parse("fake").unwrap(),
            vec![
                worker("screening_worker"),
                worker("literature_search_worker"),
                worker("extraction_worker"),
            ],
        )
        .unwrap()
    }

    fn settle_worker(
        checkpoint: &mut WorkerOrchestrationCheckpointV1,
        plan: &WorkerOrchestrationPlanV1,
        worker_id: &str,
        passed: bool,
    ) {
        let worker_id = WorkerId::parse(worker_id).unwrap();
        checkpoint
            .begin_worker(plan, checkpoint.generation, &worker_id)
            .unwrap();
        if passed {
            checkpoint
                .complete_worker(plan, checkpoint.generation, &worker_id, hash('b'))
                .unwrap();
        } else {
            checkpoint
                .fail_worker(
                    plan,
                    checkpoint.generation,
                    &worker_id,
                    WorkerOrchestrationFailureCode::BackendFailed,
                    false,
                )
                .unwrap();
        }
    }

    #[test]
    fn plan_is_bounded_sorted_and_canonical() {
        let plan = delegated_plan(2);
        assert_eq!(
            plan.workers
                .iter()
                .map(|worker| worker.worker_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "extraction_worker",
                "literature_search_worker",
                "screening_worker"
            ]
        );
        let bytes = plan.to_canonical_json().unwrap();
        assert_eq!(
            WorkerOrchestrationPlanV1::from_canonical_json(&bytes).unwrap(),
            plan
        );
        let pretty = serde_json::to_vec_pretty(&plan).unwrap();
        assert_eq!(
            WorkerOrchestrationPlanV1::from_canonical_json(&pretty),
            Err(WorkerOrchestrationError::NonCanonicalJson)
        );
    }

    #[test]
    fn plan_rejects_excess_workers_and_unsafe_goal() {
        let too_many = (0..=MAX_WORKERS)
            .map(|index| worker(&format!("worker-{index}")))
            .collect::<Vec<_>>();
        assert_eq!(
            WorkerOrchestrationPlanV1::try_new(
                RunId::parse(format!("run_{}", "a".repeat(32))).unwrap(),
                ProjectId::parse(format!("prj_{}", "1".repeat(32))).unwrap(),
                7,
                OrchestrationTaskId::parse("B1").unwrap(),
                WorkerOrchestrationMode::DelegatedWorkers,
                WorkerMergePolicy::ConsensusThenGaps,
                WorkerBarrierFailurePolicy::Degrade,
                2,
                2,
                BackendId::parse("fake").unwrap(),
                BackendId::parse("fake").unwrap(),
                too_many,
            ),
            Err(WorkerOrchestrationError::InvalidPlan)
        );
        assert_eq!(
            WorkerSpecV1::try_new(
                "worker",
                BackendId::parse("fake").unwrap(),
                " unsafe",
                "worker"
            ),
            Err(WorkerOrchestrationError::InvalidPlan)
        );
    }

    #[test]
    fn barrier_degrades_when_minimum_success_count_is_met() {
        let plan = delegated_plan(2);
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.start(&plan, 0).unwrap();
        settle_worker(&mut checkpoint, &plan, "extraction_worker", true);
        settle_worker(&mut checkpoint, &plan, "literature_search_worker", true);
        settle_worker(&mut checkpoint, &plan, "screening_worker", false);
        assert_eq!(
            checkpoint.status,
            WorkerOrchestrationRunStatus::SynthesisReady
        );
        assert_eq!(
            checkpoint.barrier_status,
            Some(WorkerBarrierStatus::Degraded)
        );
    }

    #[test]
    fn barrier_blocks_below_minimum_success_count() {
        let plan = delegated_plan(2);
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.start(&plan, 0).unwrap();
        settle_worker(&mut checkpoint, &plan, "extraction_worker", true);
        settle_worker(&mut checkpoint, &plan, "literature_search_worker", false);
        settle_worker(&mut checkpoint, &plan, "screening_worker", false);
        assert_eq!(checkpoint.status, WorkerOrchestrationRunStatus::Blocked);
        assert_eq!(
            checkpoint.barrier_status,
            Some(WorkerBarrierStatus::Blocked)
        );
        assert_eq!(
            checkpoint.failure_code,
            Some(WorkerOrchestrationFailureCode::BarrierBlocked)
        );
    }

    #[test]
    fn review_swarm_requires_every_worker() {
        let result = WorkerOrchestrationPlanV1::try_new(
            RunId::parse(format!("run_{}", "a".repeat(32))).unwrap(),
            ProjectId::parse(format!("prj_{}", "1".repeat(32))).unwrap(),
            7,
            OrchestrationTaskId::parse("H3").unwrap(),
            WorkerOrchestrationMode::ReviewSwarm,
            WorkerMergePolicy::ControllerAdjudication,
            WorkerBarrierFailurePolicy::Degrade,
            2,
            2,
            BackendId::parse("fake").unwrap(),
            BackendId::parse("fake").unwrap(),
            vec![
                worker("methodologist"),
                worker("domain_expert"),
                worker("reviewer_2"),
            ],
        );
        assert_eq!(result, Err(WorkerOrchestrationError::InvalidPlan));
    }

    #[test]
    fn synthesis_and_review_store_only_hashes() {
        let plan = delegated_plan(3);
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.start(&plan, 0).unwrap();
        for id in [
            "extraction_worker",
            "literature_search_worker",
            "screening_worker",
        ] {
            settle_worker(&mut checkpoint, &plan, id, true);
        }
        checkpoint
            .begin_synthesis(&plan, checkpoint.generation)
            .unwrap();
        checkpoint
            .complete_synthesis(&plan, checkpoint.generation, hash('c'))
            .unwrap();
        checkpoint
            .begin_review(&plan, checkpoint.generation)
            .unwrap();
        checkpoint
            .complete_review(&plan, checkpoint.generation, hash('d'), true)
            .unwrap();
        assert_eq!(checkpoint.status, WorkerOrchestrationRunStatus::Completed);
        assert_eq!(checkpoint.synthesis_output_sha256, Some(hash('c')));
        assert_eq!(checkpoint.review_output_sha256, Some(hash('d')));
        let serialized = String::from_utf8(checkpoint.to_canonical_json(&plan).unwrap()).unwrap();
        assert!(!serialized.contains("Complete the"));
    }

    #[test]
    fn retry_and_interruption_recovery_are_generation_bound() {
        let plan = delegated_plan(2);
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.start(&plan, 0).unwrap();
        let worker_id = WorkerId::parse("extraction_worker").unwrap();
        checkpoint
            .begin_worker(&plan, checkpoint.generation, &worker_id)
            .unwrap();
        let stale_generation = checkpoint.generation - 1;
        assert_eq!(
            checkpoint.complete_worker(&plan, stale_generation, &worker_id, hash('a')),
            Err(WorkerOrchestrationError::StaleGeneration)
        );
        checkpoint
            .recover_interrupted(&plan, checkpoint.generation)
            .unwrap();
        let recovered = checkpoint
            .workers
            .iter()
            .find(|worker| worker.worker_id == worker_id)
            .unwrap();
        assert_eq!(recovered.status, WorkerStatus::Planned);
        assert_eq!(
            recovered.failure_code,
            Some(WorkerOrchestrationFailureCode::WorkerInterrupted)
        );
    }

    #[test]
    fn synthesis_and_review_recovery_return_to_ready_states() {
        let plan = delegated_plan(3);
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.start(&plan, 0).unwrap();
        for id in [
            "extraction_worker",
            "literature_search_worker",
            "screening_worker",
        ] {
            settle_worker(&mut checkpoint, &plan, id, true);
        }
        checkpoint
            .begin_synthesis(&plan, checkpoint.generation)
            .unwrap();
        checkpoint
            .recover_interrupted(&plan, checkpoint.generation)
            .unwrap();
        assert_eq!(
            checkpoint.status,
            WorkerOrchestrationRunStatus::SynthesisReady
        );
        checkpoint
            .begin_synthesis(&plan, checkpoint.generation)
            .unwrap();
        checkpoint
            .complete_synthesis(&plan, checkpoint.generation, hash('c'))
            .unwrap();
        checkpoint
            .begin_review(&plan, checkpoint.generation)
            .unwrap();
        checkpoint
            .recover_interrupted(&plan, checkpoint.generation)
            .unwrap();
        assert_eq!(checkpoint.status, WorkerOrchestrationRunStatus::ReviewReady);
    }

    #[test]
    fn cancellation_settles_unfinished_workers() {
        let plan = delegated_plan(2);
        let mut checkpoint = plan.new_checkpoint().unwrap();
        checkpoint.start(&plan, 0).unwrap();
        let worker_id = WorkerId::parse("extraction_worker").unwrap();
        checkpoint
            .begin_worker(&plan, checkpoint.generation, &worker_id)
            .unwrap();
        checkpoint.cancel(&plan, checkpoint.generation).unwrap();
        assert_eq!(checkpoint.status, WorkerOrchestrationRunStatus::Cancelled);
        assert!(checkpoint.workers.iter().all(|worker| {
            !matches!(worker.status, WorkerStatus::Planned | WorkerStatus::Running)
        }));
    }

    #[test]
    fn checkpoint_is_canonical_and_plan_bound() {
        let plan = delegated_plan(2);
        let checkpoint = plan.new_checkpoint().unwrap();
        let bytes = checkpoint.to_canonical_json(&plan).unwrap();
        assert_eq!(plan.restore_checkpoint(&bytes).unwrap(), checkpoint);

        let mut other_plan = delegated_plan(2);
        other_plan.task_id = OrchestrationTaskId::parse("H3").unwrap();
        assert_eq!(
            other_plan.restore_checkpoint(&bytes),
            Err(WorkerOrchestrationError::BindingMismatch)
        );
    }
}
