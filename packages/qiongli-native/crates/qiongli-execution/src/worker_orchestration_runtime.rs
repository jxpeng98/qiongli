use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

use qiongli_project::{ProjectError, ProjectId, ProjectStateService};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    AgentBackendErrorCode, AgentBackendFuture, AgentRunError, AgentRunInputV1, AgentRunResultV1,
    BackendId, BoundedAgentRunner, CancellationToken, ProjectExecutionScope, RunId, WorkerId,
    WorkerOrchestrationCheckpointV1, WorkerOrchestrationError, WorkerOrchestrationFailureCode,
    WorkerOrchestrationPlanV1, WorkerOrchestrationRunStatus, WorkerSpecV1, WorkerStatus,
};

const WORKER_RUN_DOCUMENT_SCHEMA_VERSION: u32 = 1;
const WORKER_AGENT_RUN_DOMAIN: &[u8] = b"qiongli-worker-orchestration-agent-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerOrchestrationInputError {
    Unavailable,
    Invalid,
}

impl WorkerOrchestrationInputError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Unavailable => "worker-orchestration-input-unavailable",
            Self::Invalid => "worker-orchestration-input-invalid",
        }
    }
}

impl Display for WorkerOrchestrationInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for WorkerOrchestrationInputError {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerOrchestrationAgentPhase {
    Worker,
    Synthesis,
    Review,
}

pub enum WorkerOrchestrationInputContextV1<'a> {
    Worker {
        orchestration_run_id: &'a RunId,
        agent_run_id: &'a RunId,
        project_id: &'a ProjectId,
        expected_project_revision: u64,
        task_id: &'a crate::OrchestrationTaskId,
        worker: &'a WorkerSpecV1,
        backend_id: &'a BackendId,
        attempt: u8,
    },
    Synthesis {
        orchestration_run_id: &'a RunId,
        agent_run_id: &'a RunId,
        project_id: &'a ProjectId,
        expected_project_revision: u64,
        task_id: &'a crate::OrchestrationTaskId,
        merge_policy: crate::WorkerMergePolicy,
        backend_id: &'a BackendId,
        worker_results: &'a [WorkerOrchestrationAgentResultV1],
    },
    Review {
        orchestration_run_id: &'a RunId,
        agent_run_id: &'a RunId,
        project_id: &'a ProjectId,
        expected_project_revision: u64,
        task_id: &'a crate::OrchestrationTaskId,
        backend_id: &'a BackendId,
        synthesis_result: &'a WorkerOrchestrationAgentResultV1,
    },
}

pub trait WorkerOrchestrationInputBuilder: Send + Sync {
    fn build(
        &self,
        context: WorkerOrchestrationInputContextV1<'_>,
    ) -> Result<AgentRunInputV1, WorkerOrchestrationInputError>;

    fn review_passed(
        &self,
        result: &AgentRunResultV1,
    ) -> Result<bool, WorkerOrchestrationInputError>;
}

#[derive(Clone)]
pub struct WorkerOrchestrationAgentResultV1 {
    pub phase: WorkerOrchestrationAgentPhase,
    pub worker_id: Option<WorkerId>,
    pub output_sha256: String,
    pub agent_result: AgentRunResultV1,
}

impl fmt::Debug for WorkerOrchestrationAgentResultV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerOrchestrationAgentResultV1")
            .field("phase", &self.phase)
            .field("worker_id", &self.worker_id)
            .field("output_sha256", &self.output_sha256)
            .field("agent_result", &"<private-agent-result>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerOrchestrationStepOutcome {
    WorkerRetryReady,
    RunCompleted,
    RunBlocked,
    RunCancelled,
}

#[derive(Clone)]
pub struct WorkerOrchestrationRunResultV1 {
    pub outcome: WorkerOrchestrationStepOutcome,
    pub agent_results: Vec<WorkerOrchestrationAgentResultV1>,
    pub persisted: PersistedWorkerOrchestrationCheckpointV1,
}

impl fmt::Debug for WorkerOrchestrationRunResultV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerOrchestrationRunResultV1")
            .field("outcome", &self.outcome)
            .field("agent_result_count", &self.agent_results.len())
            .field("persisted", &self.persisted)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerOrchestrationRuntimeError {
    Contract(WorkerOrchestrationError),
    Project(ProjectError),
    Input(WorkerOrchestrationInputError),
    BackendUnavailable,
    RecoveryRequired,
}

impl WorkerOrchestrationRuntimeError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.reason_code(),
            Self::Project(error) => error.reason_code(),
            Self::Input(error) => error.reason_code(),
            Self::BackendUnavailable => "worker-orchestration-backend-unavailable",
            Self::RecoveryRequired => "worker-orchestration-recovery-required",
        }
    }
}

impl Display for WorkerOrchestrationRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for WorkerOrchestrationRuntimeError {}

impl From<WorkerOrchestrationError> for WorkerOrchestrationRuntimeError {
    fn from(error: WorkerOrchestrationError) -> Self {
        Self::Contract(error)
    }
}

impl From<ProjectError> for WorkerOrchestrationRuntimeError {
    fn from(error: ProjectError) -> Self {
        Self::Project(error)
    }
}

impl From<WorkerOrchestrationInputError> for WorkerOrchestrationRuntimeError {
    fn from(error: WorkerOrchestrationInputError) -> Self {
        Self::Input(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedWorkerOrchestrationCheckpointV1 {
    checkpoint: WorkerOrchestrationCheckpointV1,
    document_sha256: String,
}

impl PersistedWorkerOrchestrationCheckpointV1 {
    #[must_use]
    pub const fn checkpoint(&self) -> &WorkerOrchestrationCheckpointV1 {
        &self.checkpoint
    }

    #[must_use]
    pub fn document_sha256(&self) -> &str {
        &self.document_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredWorkerOrchestrationRunV1 {
    plan: WorkerOrchestrationPlanV1,
    persisted: PersistedWorkerOrchestrationCheckpointV1,
}

impl DiscoveredWorkerOrchestrationRunV1 {
    #[must_use]
    pub const fn plan(&self) -> &WorkerOrchestrationPlanV1 {
        &self.plan
    }

    #[must_use]
    pub const fn persisted(&self) -> &PersistedWorkerOrchestrationCheckpointV1 {
        &self.persisted
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkerOrchestrationRunDocumentV1 {
    schema_version: u32,
    plan: WorkerOrchestrationPlanV1,
    checkpoint: WorkerOrchestrationCheckpointV1,
}

#[derive(Clone)]
pub struct WorkerOrchestrationCheckpointStore {
    projects: ProjectStateService,
}

impl WorkerOrchestrationCheckpointStore {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn create(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        checkpoint: WorkerOrchestrationCheckpointV1,
    ) -> Result<PersistedWorkerOrchestrationCheckpointV1, WorkerOrchestrationRuntimeError> {
        if checkpoint.generation != 0 || checkpoint.status != WorkerOrchestrationRunStatus::Planned
        {
            return Err(WorkerOrchestrationError::InvalidCheckpoint.into());
        }
        let bytes = encode_run_document(plan, &checkpoint)?;
        let commit = self.projects.replace_worker_orchestration_checkpoint(
            &checkpoint.project_id,
            checkpoint.expected_project_revision,
            checkpoint.run_id.as_str(),
            None,
            &bytes,
        )?;
        Ok(PersistedWorkerOrchestrationCheckpointV1 {
            checkpoint,
            document_sha256: commit.document_sha256,
        })
    }

    pub fn load(
        &self,
        plan: &WorkerOrchestrationPlanV1,
    ) -> Result<Option<PersistedWorkerOrchestrationCheckpointV1>, WorkerOrchestrationRuntimeError>
    {
        let Some(document) = self.projects.read_worker_orchestration_checkpoint(
            &plan.project_id,
            plan.expected_project_revision,
            plan.run_id.as_str(),
        )?
        else {
            return Ok(None);
        };
        let (stored_plan, checkpoint) = decode_run_document(document.bytes())?;
        if stored_plan != *plan {
            return Err(WorkerOrchestrationError::BindingMismatch.into());
        }
        Ok(Some(PersistedWorkerOrchestrationCheckpointV1 {
            checkpoint,
            document_sha256: document.sha256().to_owned(),
        }))
    }

    pub fn discover(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<Vec<DiscoveredWorkerOrchestrationRunV1>, WorkerOrchestrationRuntimeError> {
        self.projects
            .list_worker_orchestration_checkpoints(project_id, expected_project_revision)?
            .into_iter()
            .map(|entry| {
                let (plan, checkpoint) = decode_run_document(entry.document().bytes())?;
                if checkpoint.run_id.as_str() != entry.checkpoint_id()
                    || checkpoint.project_id != *project_id
                    || checkpoint.expected_project_revision != expected_project_revision
                {
                    return Err(WorkerOrchestrationError::BindingMismatch.into());
                }
                Ok(DiscoveredWorkerOrchestrationRunV1 {
                    plan,
                    persisted: PersistedWorkerOrchestrationCheckpointV1 {
                        checkpoint,
                        document_sha256: entry.document().sha256().to_owned(),
                    },
                })
            })
            .collect()
    }

    pub fn replace(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: &PersistedWorkerOrchestrationCheckpointV1,
        checkpoint: WorkerOrchestrationCheckpointV1,
    ) -> Result<PersistedWorkerOrchestrationCheckpointV1, WorkerOrchestrationRuntimeError> {
        current.checkpoint.to_canonical_json(plan)?;
        let next_generation = current
            .checkpoint
            .generation
            .checked_add(1)
            .ok_or(WorkerOrchestrationError::LimitExceeded)?;
        if checkpoint.run_id != current.checkpoint.run_id
            || checkpoint.project_id != current.checkpoint.project_id
            || checkpoint.expected_project_revision != current.checkpoint.expected_project_revision
            || checkpoint.generation != next_generation
        {
            return Err(WorkerOrchestrationError::InvalidCheckpoint.into());
        }
        let bytes = encode_run_document(plan, &checkpoint)?;
        let commit = self.projects.replace_worker_orchestration_checkpoint(
            &checkpoint.project_id,
            checkpoint.expected_project_revision,
            checkpoint.run_id.as_str(),
            Some(&current.document_sha256),
            &bytes,
        )?;
        Ok(PersistedWorkerOrchestrationCheckpointV1 {
            checkpoint,
            document_sha256: commit.document_sha256,
        })
    }

    pub fn verify_current(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: &PersistedWorkerOrchestrationCheckpointV1,
    ) -> Result<(), WorkerOrchestrationRuntimeError> {
        let loaded = self.load(plan)?.ok_or(ProjectError::RevisionConflict)?;
        if loaded == *current {
            Ok(())
        } else {
            Err(ProjectError::RevisionConflict.into())
        }
    }
}

pub struct WorkerOrchestrationExecutor {
    runners: BTreeMap<BackendId, BoundedAgentRunner>,
    input_builder: Arc<dyn WorkerOrchestrationInputBuilder>,
    store: WorkerOrchestrationCheckpointStore,
}

impl WorkerOrchestrationExecutor {
    pub fn try_new(
        runners: impl IntoIterator<Item = (BackendId, BoundedAgentRunner)>,
        input_builder: Arc<dyn WorkerOrchestrationInputBuilder>,
        store: WorkerOrchestrationCheckpointStore,
    ) -> Result<Self, WorkerOrchestrationRuntimeError> {
        let mut mapped = BTreeMap::new();
        for (backend_id, runner) in runners {
            let descriptor = runner.backend_descriptor();
            descriptor
                .validate()
                .map_err(|_| WorkerOrchestrationRuntimeError::BackendUnavailable)?;
            if descriptor.backend_id != backend_id || mapped.insert(backend_id, runner).is_some() {
                return Err(WorkerOrchestrationRuntimeError::BackendUnavailable);
            }
        }
        if mapped.is_empty() {
            return Err(WorkerOrchestrationRuntimeError::BackendUnavailable);
        }
        Ok(Self {
            runners: mapped,
            input_builder,
            store,
        })
    }

    pub fn run_to_completion<'a>(
        &'a self,
        plan: &'a WorkerOrchestrationPlanV1,
        mut persisted: PersistedWorkerOrchestrationCheckpointV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<
        'a,
        Result<WorkerOrchestrationRunResultV1, WorkerOrchestrationRuntimeError>,
    > {
        Box::pin(async move {
            self.store.verify_current(plan, &persisted)?;
            if persisted.checkpoint.status.is_terminal() {
                return Ok(terminal_result(persisted));
            }
            if cancellation.is_cancelled() {
                return self.cancel(plan, persisted, Vec::new());
            }
            if checkpoint_requires_recovery(&persisted.checkpoint) {
                return Err(WorkerOrchestrationRuntimeError::RecoveryRequired);
            }
            self.validate_runners(plan, &persisted.checkpoint)?;
            if persisted.checkpoint.status == WorkerOrchestrationRunStatus::Planned {
                let mut checkpoint = persisted.checkpoint.clone();
                checkpoint.start(plan, checkpoint.generation)?;
                persisted = self.store.replace(plan, &persisted, checkpoint)?;
            }

            let mut agent_results = Vec::new();
            for worker in &plan.workers {
                if persisted.checkpoint.status != WorkerOrchestrationRunStatus::Running {
                    break;
                }
                let status = persisted
                    .checkpoint
                    .workers
                    .iter()
                    .find(|state| state.worker_id == worker.worker_id)
                    .map(|state| state.status)
                    .ok_or(WorkerOrchestrationError::InvalidCheckpoint)?;
                if status != WorkerStatus::Planned {
                    continue;
                }
                if cancellation.is_cancelled() {
                    return self.cancel(plan, persisted, agent_results);
                }
                self.store.verify_current(plan, &persisted)?;
                let mut checkpoint = persisted.checkpoint.clone();
                checkpoint.begin_worker(plan, checkpoint.generation, &worker.worker_id)?;
                persisted = self.store.replace(plan, &persisted, checkpoint)?;
                let attempt = persisted
                    .checkpoint
                    .workers
                    .iter()
                    .find(|state| state.worker_id == worker.worker_id)
                    .map(|state| state.attempts)
                    .ok_or(WorkerOrchestrationError::InvalidCheckpoint)?;
                let agent_run_id = derive_agent_run_id(
                    plan,
                    WorkerOrchestrationAgentPhase::Worker,
                    Some(&worker.worker_id),
                    u64::from(attempt),
                )?;
                let input =
                    match self
                        .input_builder
                        .build(WorkerOrchestrationInputContextV1::Worker {
                            orchestration_run_id: &plan.run_id,
                            agent_run_id: &agent_run_id,
                            project_id: &plan.project_id,
                            expected_project_revision: plan.expected_project_revision,
                            task_id: &plan.task_id,
                            worker,
                            backend_id: &worker.backend_id,
                            attempt,
                        }) {
                        Ok(input) => input,
                        Err(_) => {
                            persisted = self.fail_worker(
                                plan,
                                persisted,
                                &worker.worker_id,
                                WorkerOrchestrationFailureCode::WorkerInputInvalid,
                                false,
                            )?;
                            continue;
                        }
                    };
                if !input_matches_plan(&input, plan, &agent_run_id) {
                    persisted = self.fail_worker(
                        plan,
                        persisted,
                        &worker.worker_id,
                        WorkerOrchestrationFailureCode::WorkerInputInvalid,
                        false,
                    )?;
                    continue;
                }
                let result = match self
                    .run_agent(&worker.backend_id, input, cancellation.clone())
                    .await
                {
                    Ok(result) if !result.content.trim().is_empty() => result,
                    Ok(_) => {
                        persisted = self.fail_worker(
                            plan,
                            persisted,
                            &worker.worker_id,
                            WorkerOrchestrationFailureCode::WorkerOutputInvalid,
                            false,
                        )?;
                        continue;
                    }
                    Err(AgentRunError::Cancelled) => {
                        return self.cancel(plan, persisted, agent_results);
                    }
                    Err(error) => {
                        let (failure_code, retryable) = map_worker_run_error(&error);
                        persisted = self.fail_worker(
                            plan,
                            persisted,
                            &worker.worker_id,
                            failure_code,
                            retryable,
                        )?;
                        if persisted
                            .checkpoint
                            .workers
                            .iter()
                            .any(|state| state.status == WorkerStatus::Planned)
                        {
                            return Ok(WorkerOrchestrationRunResultV1 {
                                outcome: WorkerOrchestrationStepOutcome::WorkerRetryReady,
                                agent_results,
                                persisted,
                            });
                        }
                        continue;
                    }
                };
                if result.backend_id != worker.backend_id {
                    persisted = self.fail_worker(
                        plan,
                        persisted,
                        &worker.worker_id,
                        WorkerOrchestrationFailureCode::WorkerOutputInvalid,
                        false,
                    )?;
                    continue;
                }
                let output_sha256 = sha256(result.content.as_bytes());
                let mut checkpoint = persisted.checkpoint.clone();
                checkpoint.complete_worker(
                    plan,
                    checkpoint.generation,
                    &worker.worker_id,
                    output_sha256.clone(),
                )?;
                persisted = self.store.replace(plan, &persisted, checkpoint)?;
                agent_results.push(WorkerOrchestrationAgentResultV1 {
                    phase: WorkerOrchestrationAgentPhase::Worker,
                    worker_id: Some(worker.worker_id.clone()),
                    output_sha256,
                    agent_result: result,
                });
            }

            if persisted.checkpoint.status == WorkerOrchestrationRunStatus::Blocked {
                return Ok(WorkerOrchestrationRunResultV1 {
                    outcome: WorkerOrchestrationStepOutcome::RunBlocked,
                    agent_results,
                    persisted,
                });
            }
            if persisted.checkpoint.status != WorkerOrchestrationRunStatus::SynthesisReady {
                return Err(WorkerOrchestrationError::InvalidCheckpoint.into());
            }
            let passed_count = persisted
                .checkpoint
                .workers
                .iter()
                .filter(|worker| worker.status == WorkerStatus::Passed)
                .count();
            if passed_count != agent_results.len() {
                return Err(WorkerOrchestrationRuntimeError::RecoveryRequired);
            }

            if cancellation.is_cancelled() {
                return self.cancel(plan, persisted, agent_results);
            }
            self.store.verify_current(plan, &persisted)?;
            let mut checkpoint = persisted.checkpoint.clone();
            checkpoint.begin_synthesis(plan, checkpoint.generation)?;
            persisted = self.store.replace(plan, &persisted, checkpoint)?;
            let synthesis_run_id = derive_agent_run_id(
                plan,
                WorkerOrchestrationAgentPhase::Synthesis,
                None,
                persisted.checkpoint.generation,
            )?;
            let synthesis_input =
                match self
                    .input_builder
                    .build(WorkerOrchestrationInputContextV1::Synthesis {
                        orchestration_run_id: &plan.run_id,
                        agent_run_id: &synthesis_run_id,
                        project_id: &plan.project_id,
                        expected_project_revision: plan.expected_project_revision,
                        task_id: &plan.task_id,
                        merge_policy: plan.merge_policy,
                        backend_id: &plan.synthesis_backend_id,
                        worker_results: &agent_results,
                    }) {
                    Ok(input) if input_matches_plan(&input, plan, &synthesis_run_id) => input,
                    _ => {
                        return self.fail_synthesis(plan, persisted, agent_results);
                    }
                };
            let synthesis_result = match self
                .run_agent(
                    &plan.synthesis_backend_id,
                    synthesis_input,
                    cancellation.clone(),
                )
                .await
            {
                Ok(result)
                    if result.backend_id == plan.synthesis_backend_id
                        && !result.content.trim().is_empty() =>
                {
                    result
                }
                Err(AgentRunError::Cancelled) => {
                    return self.cancel(plan, persisted, agent_results);
                }
                _ => return self.fail_synthesis(plan, persisted, agent_results),
            };
            let synthesis_sha256 = sha256(synthesis_result.content.as_bytes());
            let mut checkpoint = persisted.checkpoint.clone();
            checkpoint.complete_synthesis(plan, checkpoint.generation, synthesis_sha256.clone())?;
            persisted = self.store.replace(plan, &persisted, checkpoint)?;
            agent_results.push(WorkerOrchestrationAgentResultV1 {
                phase: WorkerOrchestrationAgentPhase::Synthesis,
                worker_id: None,
                output_sha256: synthesis_sha256,
                agent_result: synthesis_result,
            });

            if cancellation.is_cancelled() {
                return self.cancel(plan, persisted, agent_results);
            }
            self.store.verify_current(plan, &persisted)?;
            let mut checkpoint = persisted.checkpoint.clone();
            checkpoint.begin_review(plan, checkpoint.generation)?;
            persisted = self.store.replace(plan, &persisted, checkpoint)?;
            let review_run_id = derive_agent_run_id(
                plan,
                WorkerOrchestrationAgentPhase::Review,
                None,
                persisted.checkpoint.generation,
            )?;
            let synthesis_result = agent_results
                .last()
                .filter(|result| result.phase == WorkerOrchestrationAgentPhase::Synthesis)
                .ok_or(WorkerOrchestrationRuntimeError::RecoveryRequired)?;
            let review_input =
                match self
                    .input_builder
                    .build(WorkerOrchestrationInputContextV1::Review {
                        orchestration_run_id: &plan.run_id,
                        agent_run_id: &review_run_id,
                        project_id: &plan.project_id,
                        expected_project_revision: plan.expected_project_revision,
                        task_id: &plan.task_id,
                        backend_id: &plan.review_backend_id,
                        synthesis_result,
                    }) {
                    Ok(input) if input_matches_plan(&input, plan, &review_run_id) => input,
                    _ => return self.fail_review(plan, persisted, agent_results),
                };
            let review_result = match self
                .run_agent(&plan.review_backend_id, review_input, cancellation.clone())
                .await
            {
                Ok(result)
                    if result.backend_id == plan.review_backend_id
                        && !result.content.trim().is_empty() =>
                {
                    result
                }
                Err(AgentRunError::Cancelled) => {
                    return self.cancel(plan, persisted, agent_results);
                }
                _ => return self.fail_review(plan, persisted, agent_results),
            };
            let passed = match self.input_builder.review_passed(&review_result) {
                Ok(passed) => passed,
                Err(_) => return self.fail_review(plan, persisted, agent_results),
            };
            let review_sha256 = sha256(review_result.content.as_bytes());
            let mut checkpoint = persisted.checkpoint.clone();
            checkpoint.complete_review(
                plan,
                checkpoint.generation,
                review_sha256.clone(),
                passed,
            )?;
            persisted = self.store.replace(plan, &persisted, checkpoint)?;
            agent_results.push(WorkerOrchestrationAgentResultV1 {
                phase: WorkerOrchestrationAgentPhase::Review,
                worker_id: None,
                output_sha256: review_sha256,
                agent_result: review_result,
            });
            Ok(WorkerOrchestrationRunResultV1 {
                outcome: if passed {
                    WorkerOrchestrationStepOutcome::RunCompleted
                } else {
                    WorkerOrchestrationStepOutcome::RunBlocked
                },
                agent_results,
                persisted,
            })
        })
    }

    pub fn recover(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: PersistedWorkerOrchestrationCheckpointV1,
    ) -> Result<PersistedWorkerOrchestrationCheckpointV1, WorkerOrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.recover_interrupted(plan, checkpoint.generation)?;
        self.store.replace(plan, &current, checkpoint)
    }

    pub fn cancel_run(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: PersistedWorkerOrchestrationCheckpointV1,
    ) -> Result<PersistedWorkerOrchestrationCheckpointV1, WorkerOrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.cancel(plan, checkpoint.generation)?;
        self.store.replace(plan, &current, checkpoint)
    }

    fn run_agent<'a>(
        &'a self,
        backend_id: &'a BackendId,
        input: AgentRunInputV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<'a, Result<AgentRunResultV1, AgentRunError>> {
        Box::pin(async move {
            let runner = self
                .runners
                .get(backend_id)
                .ok_or(AgentRunError::InvalidRequest)?;
            runner.run(input, cancellation).await
        })
    }

    fn validate_runners(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        checkpoint: &WorkerOrchestrationCheckpointV1,
    ) -> Result<(), WorkerOrchestrationRuntimeError> {
        let required = plan
            .workers
            .iter()
            .map(|worker| &worker.backend_id)
            .chain([&plan.synthesis_backend_id, &plan.review_backend_id])
            .collect::<BTreeSet<_>>();
        let root = self
            .store
            .projects
            .resolve_project_root(&checkpoint.project_id)?;
        for backend_id in required {
            let runner = self
                .runners
                .get(backend_id)
                .ok_or(WorkerOrchestrationRuntimeError::BackendUnavailable)?;
            let Some(scope) = runner.project_scope() else {
                return Err(WorkerOrchestrationRuntimeError::BackendUnavailable);
            };
            if !scope_matches_checkpoint(scope, checkpoint, root.path()) {
                return Err(WorkerOrchestrationRuntimeError::BackendUnavailable);
            }
        }
        Ok(())
    }

    fn fail_worker(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: PersistedWorkerOrchestrationCheckpointV1,
        worker_id: &WorkerId,
        failure_code: WorkerOrchestrationFailureCode,
        retryable: bool,
    ) -> Result<PersistedWorkerOrchestrationCheckpointV1, WorkerOrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.fail_worker(
            plan,
            checkpoint.generation,
            worker_id,
            failure_code,
            retryable,
        )?;
        self.store.replace(plan, &current, checkpoint)
    }

    fn fail_synthesis(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: PersistedWorkerOrchestrationCheckpointV1,
        agent_results: Vec<WorkerOrchestrationAgentResultV1>,
    ) -> Result<WorkerOrchestrationRunResultV1, WorkerOrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.fail_synthesis(plan, checkpoint.generation)?;
        let persisted = self.store.replace(plan, &current, checkpoint)?;
        Ok(WorkerOrchestrationRunResultV1 {
            outcome: WorkerOrchestrationStepOutcome::RunBlocked,
            agent_results,
            persisted,
        })
    }

    fn fail_review(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: PersistedWorkerOrchestrationCheckpointV1,
        agent_results: Vec<WorkerOrchestrationAgentResultV1>,
    ) -> Result<WorkerOrchestrationRunResultV1, WorkerOrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.fail_review(plan, checkpoint.generation)?;
        let persisted = self.store.replace(plan, &current, checkpoint)?;
        Ok(WorkerOrchestrationRunResultV1 {
            outcome: WorkerOrchestrationStepOutcome::RunBlocked,
            agent_results,
            persisted,
        })
    }

    fn cancel(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        current: PersistedWorkerOrchestrationCheckpointV1,
        agent_results: Vec<WorkerOrchestrationAgentResultV1>,
    ) -> Result<WorkerOrchestrationRunResultV1, WorkerOrchestrationRuntimeError> {
        let persisted = self.cancel_run(plan, current)?;
        Ok(WorkerOrchestrationRunResultV1 {
            outcome: WorkerOrchestrationStepOutcome::RunCancelled,
            agent_results,
            persisted,
        })
    }
}

fn checkpoint_requires_recovery(checkpoint: &WorkerOrchestrationCheckpointV1) -> bool {
    match checkpoint.status {
        WorkerOrchestrationRunStatus::Running => checkpoint
            .workers
            .iter()
            .any(|worker| matches!(worker.status, WorkerStatus::Running | WorkerStatus::Passed)),
        WorkerOrchestrationRunStatus::SynthesisReady
        | WorkerOrchestrationRunStatus::Synthesizing
        | WorkerOrchestrationRunStatus::ReviewReady
        | WorkerOrchestrationRunStatus::Reviewing => true,
        WorkerOrchestrationRunStatus::Planned
        | WorkerOrchestrationRunStatus::Completed
        | WorkerOrchestrationRunStatus::Blocked
        | WorkerOrchestrationRunStatus::Cancelled => false,
    }
}

fn scope_matches_checkpoint(
    scope: &ProjectExecutionScope,
    checkpoint: &WorkerOrchestrationCheckpointV1,
    root: &std::path::Path,
) -> bool {
    scope.project_id == checkpoint.project_id
        && scope.semantic_revision == checkpoint.expected_project_revision
        && scope.canonical_root == root
}

fn input_matches_plan(
    input: &AgentRunInputV1,
    plan: &WorkerOrchestrationPlanV1,
    run: &RunId,
) -> bool {
    input.request.run_id == *run
        && input.project_id.as_ref() == Some(&plan.project_id)
        && input.expected_project_revision == Some(plan.expected_project_revision)
}

fn derive_agent_run_id(
    plan: &WorkerOrchestrationPlanV1,
    phase: WorkerOrchestrationAgentPhase,
    worker_id: Option<&WorkerId>,
    sequence: u64,
) -> Result<RunId, WorkerOrchestrationError> {
    let mut digest = Sha256::new();
    digest.update(WORKER_AGENT_RUN_DOMAIN);
    digest.update([0]);
    digest.update(plan.run_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(plan.project_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(plan.task_id.as_str().as_bytes());
    digest.update([0]);
    digest.update([match phase {
        WorkerOrchestrationAgentPhase::Worker => 1,
        WorkerOrchestrationAgentPhase::Synthesis => 2,
        WorkerOrchestrationAgentPhase::Review => 3,
    }]);
    digest.update([0]);
    if let Some(worker_id) = worker_id {
        digest.update(worker_id.as_str().as_bytes());
    }
    digest.update([0]);
    digest.update(sequence.to_be_bytes());
    let value = format!("run_{:x}", digest.finalize());
    RunId::parse(value[..36].to_owned()).map_err(|_| WorkerOrchestrationError::InvalidCheckpoint)
}

fn map_worker_run_error(error: &AgentRunError) -> (WorkerOrchestrationFailureCode, bool) {
    match error {
        AgentRunError::PreflightFailed(_) => {
            (WorkerOrchestrationFailureCode::BackendUnavailable, true)
        }
        AgentRunError::Backend(error) => {
            let code = match error.code {
                AgentBackendErrorCode::AuthenticationUnavailable
                | AgentBackendErrorCode::CapabilityUnavailable
                | AgentBackendErrorCode::TransportUnavailable => {
                    WorkerOrchestrationFailureCode::BackendUnavailable
                }
                AgentBackendErrorCode::ProviderRejected => {
                    WorkerOrchestrationFailureCode::BackendRejected
                }
                AgentBackendErrorCode::InvalidRequest
                | AgentBackendErrorCode::ResponseInvalid
                | AgentBackendErrorCode::Cancelled => WorkerOrchestrationFailureCode::BackendFailed,
            };
            (code, error.retry_class.is_some())
        }
        AgentRunError::InvalidRequest => {
            (WorkerOrchestrationFailureCode::WorkerInputInvalid, false)
        }
        AgentRunError::EventInvalid | AgentRunError::EventSequenceInvalid => {
            (WorkerOrchestrationFailureCode::WorkerOutputInvalid, false)
        }
        AgentRunError::PolicyDenied(_)
        | AgentRunError::ToolHost(_)
        | AgentRunError::LimitExceeded
        | AgentRunError::Cancelled => (WorkerOrchestrationFailureCode::BackendFailed, false),
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn terminal_result(
    persisted: PersistedWorkerOrchestrationCheckpointV1,
) -> WorkerOrchestrationRunResultV1 {
    let outcome = match persisted.checkpoint.status {
        WorkerOrchestrationRunStatus::Completed => WorkerOrchestrationStepOutcome::RunCompleted,
        WorkerOrchestrationRunStatus::Cancelled => WorkerOrchestrationStepOutcome::RunCancelled,
        WorkerOrchestrationRunStatus::Blocked => WorkerOrchestrationStepOutcome::RunBlocked,
        _ => WorkerOrchestrationStepOutcome::RunBlocked,
    };
    WorkerOrchestrationRunResultV1 {
        outcome,
        agent_results: Vec::new(),
        persisted,
    }
}

fn encode_run_document(
    plan: &WorkerOrchestrationPlanV1,
    checkpoint: &WorkerOrchestrationCheckpointV1,
) -> Result<Vec<u8>, WorkerOrchestrationError> {
    plan.to_canonical_json()?;
    checkpoint.to_canonical_json(plan)?;
    serde_json_canonicalizer::to_vec(&WorkerOrchestrationRunDocumentV1 {
        schema_version: WORKER_RUN_DOCUMENT_SCHEMA_VERSION,
        plan: plan.clone(),
        checkpoint: checkpoint.clone(),
    })
    .map_err(|_| WorkerOrchestrationError::SerializationFailed)
}

fn decode_run_document(
    input: &[u8],
) -> Result<(WorkerOrchestrationPlanV1, WorkerOrchestrationCheckpointV1), WorkerOrchestrationError>
{
    let document = serde_json::from_slice::<WorkerOrchestrationRunDocumentV1>(input)
        .map_err(|_| WorkerOrchestrationError::InvalidJson)?;
    if document.schema_version != WORKER_RUN_DOCUMENT_SCHEMA_VERSION {
        return Err(WorkerOrchestrationError::BindingMismatch);
    }
    document.plan.to_canonical_json()?;
    let checkpoint_bytes = document.checkpoint.to_canonical_json(&document.plan)?;
    let checkpoint = document.plan.restore_checkpoint(&checkpoint_bytes)?;
    let canonical = encode_run_document(&document.plan, &checkpoint)?;
    if canonical != input {
        return Err(WorkerOrchestrationError::NonCanonicalJson);
    }
    Ok((document.plan, checkpoint))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::future::Future;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};

    use qiongli_config::resolve_config_root;
    use qiongli_project::{
        ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions, ProjectStateService,
    };

    use crate::{
        AgentBackendError, AgentEventV1, AgentFinishReason, AgentMessageV1, AgentRequirementsV1,
        AgentResponseConstraintsV1, AgentRole, AgentUsageV1, DeterministicFakeBackend,
        ExecutionLimitsV1, ExecutionProfile, InProcessToolHost, OrchestrationTaskId,
        RedactionPolicyV1, ToolId, WorkerBarrierFailurePolicy, WorkerMergePolicy,
        WorkerOrchestrationMode, WorkerSpecV1,
    };

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        project_root: PathBuf,
        projects: ProjectStateService,
        project_id: ProjectId,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "qiongli-worker-orchestration-store-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            let projects = ProjectStateService::new(resolve_config_root(None, &home).unwrap());
            let project_root = root.join("article");
            let create = projects
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new("Worker paper", ProjectKind::Article),
                    1,
                )
                .unwrap();
            let project_id = create.preview().project_id.clone();
            projects
                .apply(
                    &create,
                    &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                    1,
                )
                .unwrap();
            Self {
                root,
                project_root,
                projects,
                project_id,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn backend_id() -> BackendId {
        BackendId::parse("deterministic-fake").unwrap()
    }

    fn plan_with_required(fixture: &Fixture, required_successes: u8) -> WorkerOrchestrationPlanV1 {
        WorkerOrchestrationPlanV1::try_new(
            RunId::parse(format!("run_{}", "6".repeat(32))).unwrap(),
            fixture.project_id.clone(),
            1,
            OrchestrationTaskId::parse("B1").unwrap(),
            WorkerOrchestrationMode::DelegatedWorkers,
            WorkerMergePolicy::SynthesizeWithConflictMatrix,
            WorkerBarrierFailurePolicy::Degrade,
            required_successes,
            2,
            backend_id(),
            backend_id(),
            vec![
                WorkerSpecV1::try_new(
                    "search_worker",
                    backend_id(),
                    "Search one bounded facet",
                    "search_worker",
                )
                .unwrap(),
                WorkerSpecV1::try_new(
                    "screening_worker",
                    backend_id(),
                    "Screen one bounded facet",
                    "screening_worker",
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn plan(fixture: &Fixture) -> WorkerOrchestrationPlanV1 {
        plan_with_required(fixture, 1)
    }

    struct TestWorkerInputBuilder {
        observations: Arc<Mutex<Vec<String>>>,
    }

    impl WorkerOrchestrationInputBuilder for TestWorkerInputBuilder {
        fn build(
            &self,
            context: WorkerOrchestrationInputContextV1<'_>,
        ) -> Result<AgentRunInputV1, WorkerOrchestrationInputError> {
            let (run_id, project_id, revision, content) = match context {
                WorkerOrchestrationInputContextV1::Worker {
                    agent_run_id,
                    project_id,
                    expected_project_revision,
                    worker,
                    ..
                } => (
                    agent_run_id,
                    project_id,
                    expected_project_revision,
                    format!("worker:{}", worker.worker_id.as_str()),
                ),
                WorkerOrchestrationInputContextV1::Synthesis {
                    agent_run_id,
                    project_id,
                    expected_project_revision,
                    worker_results,
                    ..
                } => (
                    agent_run_id,
                    project_id,
                    expected_project_revision,
                    format!(
                        "synthesis:{}",
                        worker_results
                            .iter()
                            .map(|result| result.agent_result.content.as_str())
                            .collect::<Vec<_>>()
                            .join("|")
                    ),
                ),
                WorkerOrchestrationInputContextV1::Review {
                    agent_run_id,
                    project_id,
                    expected_project_revision,
                    synthesis_result,
                    ..
                } => (
                    agent_run_id,
                    project_id,
                    expected_project_revision,
                    format!("review:{}", synthesis_result.agent_result.content),
                ),
            };
            self.observations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(content.clone());
            Ok(AgentRunInputV1 {
                request: crate::AgentRequestV1 {
                    schema_version: 1,
                    run_id: run_id.clone(),
                    model: "deterministic-v1".to_owned(),
                    messages: vec![AgentMessageV1 {
                        role: AgentRole::User,
                        content,
                        tool_call_id: None,
                    }],
                    attachments: Vec::new(),
                    response: AgentResponseConstraintsV1 {
                        maximum_output_tokens: 128,
                        structured_output_schema: None,
                    },
                    tools: Vec::new(),
                },
                requirements: AgentRequirementsV1 {
                    minimum_context_tokens: 1_024,
                    streaming: false,
                    structured_output: false,
                    tool_calls: false,
                    multimodal: false,
                    cancellation: true,
                },
                purpose: "Run one bounded worker orchestration phase.".to_owned(),
                project_id: Some(project_id.clone()),
                expected_project_revision: Some(revision),
            })
        }

        fn review_passed(
            &self,
            result: &AgentRunResultV1,
        ) -> Result<bool, WorkerOrchestrationInputError> {
            match result.content.as_str() {
                "PASS" => Ok(true),
                "BLOCK" => Ok(false),
                _ => Err(WorkerOrchestrationInputError::Invalid),
            }
        }
    }

    fn completed_script(content: &str) -> Vec<Result<AgentEventV1, AgentBackendError>> {
        vec![
            Ok(AgentEventV1::ContentDelta {
                content: content.to_owned(),
            }),
            Ok(AgentEventV1::Usage {
                usage: AgentUsageV1 {
                    input_tokens: 4,
                    output_tokens: 2,
                    cached_input_tokens: 0,
                },
            }),
            Ok(AgentEventV1::Completed {
                finish_reason: AgentFinishReason::Stop,
            }),
        ]
    }

    fn ready<T>(mut future: impl Future<Output = T> + Unpin) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        match std::pin::Pin::new(&mut future).poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("deterministic worker future must be immediately ready"),
        }
    }

    fn make_executor(
        fixture: &Fixture,
        backend: Arc<DeterministicFakeBackend>,
    ) -> (
        WorkerOrchestrationExecutor,
        WorkerOrchestrationCheckpointStore,
        Arc<Mutex<Vec<String>>>,
    ) {
        let scope =
            ProjectExecutionScope::new(fixture.project_id.clone(), fixture.project_root.clone(), 1)
                .unwrap();
        let policy = crate::AgentExecutionPolicy::locked(
            1,
            ExecutionProfile::Full,
            [ToolId::parse("fixture-read").unwrap()],
            Some(scope),
            ExecutionLimitsV1::bounded_default(),
            RedactionPolicyV1::strict_default(),
        )
        .unwrap();
        let runner = BoundedAgentRunner::new(backend, InProcessToolHost::new(), policy);
        let store = WorkerOrchestrationCheckpointStore::new(fixture.projects.clone());
        let observations = Arc::new(Mutex::new(Vec::new()));
        let input_builder = Arc::new(TestWorkerInputBuilder {
            observations: Arc::clone(&observations),
        });
        let executor = WorkerOrchestrationExecutor::try_new(
            [(backend_id(), runner)],
            input_builder,
            store.clone(),
        )
        .unwrap();
        (executor, store, observations)
    }

    #[test]
    fn checkpoint_store_round_trips_and_discovers_exact_plan() {
        let fixture = Fixture::new();
        let plan = plan(&fixture);
        let store = WorkerOrchestrationCheckpointStore::new(fixture.projects.clone());
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        assert_eq!(store.load(&plan).unwrap().unwrap(), persisted);
        let discovered = store.discover(&fixture.project_id, 1).unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].plan(), &plan);
        assert_eq!(discovered[0].persisted(), &persisted);

        let raw = fixture
            .projects
            .read_worker_orchestration_checkpoint(&fixture.project_id, 1, plan.run_id.as_str())
            .unwrap()
            .unwrap();
        assert!(
            !std::str::from_utf8(raw.bytes())
                .unwrap()
                .contains("model output canary")
        );
    }

    #[test]
    fn replacement_requires_exact_document_and_generation() {
        let fixture = Fixture::new();
        let plan = plan(&fixture);
        let store = WorkerOrchestrationCheckpointStore::new(fixture.projects.clone());
        let initial = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let stale = initial.clone();
        let mut started = initial.checkpoint().clone();
        started.start(&plan, 0).unwrap();
        let started = store.replace(&plan, &initial, started).unwrap();
        assert_eq!(started.checkpoint().generation, 1);

        let mut cancelled = stale.checkpoint().clone();
        cancelled.cancel(&plan, cancelled.generation).unwrap();
        assert_eq!(
            store.replace(&plan, &stale, cancelled),
            Err(WorkerOrchestrationRuntimeError::Project(
                ProjectError::RevisionConflict
            ))
        );
    }

    #[test]
    fn malformed_envelope_and_plan_substitution_fail_closed() {
        let fixture = Fixture::new();
        let plan = plan(&fixture);
        let store = WorkerOrchestrationCheckpointStore::new(fixture.projects.clone());
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let raw = fixture
            .projects
            .read_worker_orchestration_checkpoint(&fixture.project_id, 1, plan.run_id.as_str())
            .unwrap()
            .unwrap();
        let mut value = serde_json::from_slice::<serde_json::Value>(raw.bytes()).unwrap();
        value.as_object_mut().unwrap().insert(
            "privateCanary".to_owned(),
            serde_json::json!("model output canary"),
        );
        let bytes = serde_json_canonicalizer::to_vec(&value).unwrap();
        fixture
            .projects
            .replace_worker_orchestration_checkpoint(
                &fixture.project_id,
                1,
                plan.run_id.as_str(),
                Some(persisted.document_sha256()),
                &bytes,
            )
            .unwrap();
        assert_eq!(
            store.load(&plan),
            Err(WorkerOrchestrationRuntimeError::Contract(
                WorkerOrchestrationError::InvalidJson
            ))
        );
    }

    #[test]
    fn executor_runs_workers_synthesis_and_independent_review() {
        let fixture = Fixture::new();
        let plan = plan_with_required(&fixture, 2);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                completed_script("screening result canary"),
                completed_script("search result canary"),
                completed_script("merged result canary"),
                completed_script("PASS"),
            ])
            .unwrap(),
        );
        let (executor, store, observations) = make_executor(&fixture, Arc::clone(&backend));
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();

        let result =
            ready(executor.run_to_completion(&plan, persisted, CancellationToken::new())).unwrap();
        assert_eq!(result.outcome, WorkerOrchestrationStepOutcome::RunCompleted);
        assert_eq!(result.agent_results.len(), 4);
        assert_eq!(
            result.persisted.checkpoint().status,
            WorkerOrchestrationRunStatus::Completed
        );
        assert_eq!(backend.start_count(), 4);
        let observations = observations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(observations[2].contains("screening result canary"));
        assert!(observations[2].contains("search result canary"));
        assert!(observations[3].contains("merged result canary"));

        let raw = fixture
            .projects
            .read_worker_orchestration_checkpoint(&fixture.project_id, 1, plan.run_id.as_str())
            .unwrap()
            .unwrap();
        let raw = std::str::from_utf8(raw.bytes()).unwrap();
        assert!(!raw.contains("result canary"));
        assert!(!format!("{result:?}").contains("result canary"));
    }

    #[test]
    fn executor_degrades_or_blocks_at_the_deterministic_barrier() {
        let fixture = Fixture::new();
        let plan = plan(&fixture);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                completed_script("one worker passed"),
                vec![Err(AgentBackendError::new(
                    AgentBackendErrorCode::ProviderRejected,
                    None,
                ))],
                completed_script("degraded synthesis"),
                completed_script("PASS"),
            ])
            .unwrap(),
        );
        let (executor, store, _) = make_executor(&fixture, Arc::clone(&backend));
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let degraded =
            ready(executor.run_to_completion(&plan, persisted, CancellationToken::new())).unwrap();
        assert_eq!(
            degraded.persisted.checkpoint().barrier_status,
            Some(crate::WorkerBarrierStatus::Degraded)
        );
        assert_eq!(
            degraded.outcome,
            WorkerOrchestrationStepOutcome::RunCompleted
        );

        let fixture = Fixture::new();
        let plan = plan_with_required(&fixture, 2);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                completed_script("one worker passed"),
                vec![Err(AgentBackendError::new(
                    AgentBackendErrorCode::ProviderRejected,
                    None,
                ))],
            ])
            .unwrap(),
        );
        let (executor, store, _) = make_executor(&fixture, Arc::clone(&backend));
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let blocked =
            ready(executor.run_to_completion(&plan, persisted, CancellationToken::new())).unwrap();
        assert_eq!(blocked.outcome, WorkerOrchestrationStepOutcome::RunBlocked);
        assert_eq!(backend.start_count(), 2);
        assert_eq!(
            blocked.persisted.checkpoint().failure_code,
            Some(WorkerOrchestrationFailureCode::BarrierBlocked)
        );
    }

    #[test]
    fn executor_requires_recovery_then_replays_hash_only_outputs() {
        let fixture = Fixture::new();
        let plan = plan_with_required(&fixture, 2);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                completed_script("replayed screening"),
                completed_script("replayed search"),
                completed_script("replayed synthesis"),
                completed_script("PASS"),
            ])
            .unwrap(),
        );
        let (executor, store, _) = make_executor(&fixture, Arc::clone(&backend));
        let initial = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let mut checkpoint = initial.checkpoint().clone();
        checkpoint.start(&plan, checkpoint.generation).unwrap();
        let started = store.replace(&plan, &initial, checkpoint).unwrap();
        let worker_id = plan.workers[0].worker_id.clone();
        let mut checkpoint = started.checkpoint().clone();
        checkpoint
            .begin_worker(&plan, checkpoint.generation, &worker_id)
            .unwrap();
        let running = store.replace(&plan, &started, checkpoint).unwrap();
        let mut checkpoint = running.checkpoint().clone();
        checkpoint
            .complete_worker(&plan, checkpoint.generation, &worker_id, "a".repeat(64))
            .unwrap();
        let partial = store.replace(&plan, &running, checkpoint).unwrap();

        assert_eq!(
            ready(executor.run_to_completion(&plan, partial.clone(), CancellationToken::new()))
                .unwrap_err(),
            WorkerOrchestrationRuntimeError::RecoveryRequired
        );
        assert_eq!(backend.start_count(), 0);
        let recovered = executor.recover(&plan, partial).unwrap();
        let completed =
            ready(executor.run_to_completion(&plan, recovered, CancellationToken::new())).unwrap();
        assert_eq!(
            completed.outcome,
            WorkerOrchestrationStepOutcome::RunCompleted
        );
        assert_eq!(backend.start_count(), 4);
    }

    #[test]
    fn precancelled_executor_stops_before_backend_execution() {
        let fixture = Fixture::new();
        let plan = plan(&fixture);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![completed_script("unused")]).unwrap(),
        );
        let (executor, store, _) = make_executor(&fixture, Arc::clone(&backend));
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let result = ready(executor.run_to_completion(&plan, persisted, cancellation)).unwrap();
        assert_eq!(result.outcome, WorkerOrchestrationStepOutcome::RunCancelled);
        assert_eq!(backend.start_count(), 0);
    }

    #[test]
    fn retryable_worker_failure_resumes_without_partial_model_text() {
        let fixture = Fixture::new();
        let plan = plan_with_required(&fixture, 2);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                vec![Err(AgentBackendError::new(
                    AgentBackendErrorCode::TransportUnavailable,
                    Some(crate::AgentRetryClass::NetworkTransient),
                ))],
                completed_script("retried screening"),
                completed_script("search result"),
                completed_script("synthesis result"),
                completed_script("PASS"),
            ])
            .unwrap(),
        );
        let (executor, store, _) = make_executor(&fixture, Arc::clone(&backend));
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let first =
            ready(executor.run_to_completion(&plan, persisted, CancellationToken::new())).unwrap();
        assert_eq!(
            first.outcome,
            WorkerOrchestrationStepOutcome::WorkerRetryReady
        );
        assert_eq!(backend.start_count(), 1);
        assert!(first.agent_results.is_empty());

        let completed =
            ready(executor.run_to_completion(&plan, first.persisted, CancellationToken::new()))
                .unwrap();
        assert_eq!(
            completed.outcome,
            WorkerOrchestrationStepOutcome::RunCompleted
        );
        assert_eq!(backend.start_count(), 5);
    }

    #[test]
    fn mismatched_runner_scope_fails_before_worker_or_checkpoint_mutation() {
        let fixture = Fixture::new();
        let plan = plan(&fixture);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![completed_script("unused")]).unwrap(),
        );
        let scope =
            ProjectExecutionScope::new(fixture.project_id.clone(), fixture.project_root.clone(), 2)
                .unwrap();
        let policy = crate::AgentExecutionPolicy::locked(
            1,
            ExecutionProfile::Full,
            [ToolId::parse("fixture-read").unwrap()],
            Some(scope),
            ExecutionLimitsV1::bounded_default(),
            RedactionPolicyV1::strict_default(),
        )
        .unwrap();
        let runner = BoundedAgentRunner::new(
            Arc::clone(&backend) as Arc<dyn crate::AgentBackend>,
            InProcessToolHost::new(),
            policy,
        );
        let store = WorkerOrchestrationCheckpointStore::new(fixture.projects.clone());
        let observations = Arc::new(Mutex::new(Vec::new()));
        let executor = WorkerOrchestrationExecutor::try_new(
            [(backend_id(), runner)],
            Arc::new(TestWorkerInputBuilder { observations }),
            store.clone(),
        )
        .unwrap();
        let persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();

        assert_eq!(
            ready(executor.run_to_completion(&plan, persisted.clone(), CancellationToken::new()))
                .unwrap_err(),
            WorkerOrchestrationRuntimeError::BackendUnavailable
        );
        assert_eq!(backend.start_count(), 0);
        assert_eq!(store.load(&plan).unwrap().unwrap(), persisted);
    }
}
