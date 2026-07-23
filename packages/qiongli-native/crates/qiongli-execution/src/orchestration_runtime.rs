use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Arc;

use qiongli_project::{ProjectError, ProjectId, ProjectStateService};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    AgentBackendFuture, AgentRunError, AgentRunInputV1, AgentRunResultV1, BackendId,
    BoundedAgentRunner, CancellationToken, OrchestrationCheckpointV1, OrchestrationError,
    OrchestrationFailureCode, OrchestrationPlanV1, OrchestrationProfileV1, OrchestrationRole,
    OrchestrationRunStatus, OrchestrationTaskGraphV1, OrchestrationTaskId, OrchestrationTaskState,
    ProjectExecutionScope, RunId,
};

const ROLE_RUN_DOMAIN: &[u8] = b"qiongli-orchestration-role-v1";
const RUN_DOCUMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationRoleInputError {
    Unavailable,
    Invalid,
}

impl OrchestrationRoleInputError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Unavailable => "orchestration-role-input-unavailable",
            Self::Invalid => "orchestration-role-input-invalid",
        }
    }
}

impl Display for OrchestrationRoleInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for OrchestrationRoleInputError {}

pub struct OrchestrationRoleInputContextV1<'a> {
    pub orchestration_run_id: &'a RunId,
    pub role_run_id: &'a RunId,
    pub project_id: &'a ProjectId,
    pub expected_project_revision: u64,
    pub task_id: &'a OrchestrationTaskId,
    pub attempt: u8,
    pub role: OrchestrationRole,
    pub backend_id: &'a BackendId,
    pub prior_role_results: &'a [OrchestrationRoleResultV1],
}

pub trait OrchestrationRoleInputBuilder: Send + Sync {
    fn build(
        &self,
        context: OrchestrationRoleInputContextV1<'_>,
    ) -> Result<AgentRunInputV1, OrchestrationRoleInputError>;
}

#[derive(Clone)]
pub struct OrchestrationRoleResultV1 {
    pub task_id: OrchestrationTaskId,
    pub role: OrchestrationRole,
    pub output_sha256: String,
    pub agent_result: AgentRunResultV1,
}

impl Debug for OrchestrationRoleResultV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OrchestrationRoleResultV1")
            .field("task_id", &self.task_id)
            .field("role", &self.role)
            .field("output_sha256", &self.output_sha256)
            .field("agent_result", &"<private-agent-result>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedOrchestrationCheckpointV1 {
    checkpoint: OrchestrationCheckpointV1,
    document_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredOrchestrationRunV1 {
    plan: OrchestrationPlanV1,
    persisted: PersistedOrchestrationCheckpointV1,
}

impl DiscoveredOrchestrationRunV1 {
    #[must_use]
    pub const fn plan(&self) -> &OrchestrationPlanV1 {
        &self.plan
    }

    #[must_use]
    pub const fn persisted(&self) -> &PersistedOrchestrationCheckpointV1 {
        &self.persisted
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OrchestrationRunDocumentV1 {
    schema_version: u32,
    profile: OrchestrationProfileV1,
    checkpoint: OrchestrationCheckpointV1,
}

impl PersistedOrchestrationCheckpointV1 {
    #[must_use]
    pub const fn checkpoint(&self) -> &OrchestrationCheckpointV1 {
        &self.checkpoint
    }

    #[must_use]
    pub fn document_sha256(&self) -> &str {
        &self.document_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationStepOutcome {
    TaskCompleted,
    TaskRetryReady,
    TaskFailed,
    RunCompleted,
    RunFailed,
    RunCancelled,
    Paused,
}

#[derive(Clone)]
pub struct OrchestrationTaskRunResultV1 {
    pub outcome: OrchestrationStepOutcome,
    pub task_id: Option<OrchestrationTaskId>,
    pub role_results: Vec<OrchestrationRoleResultV1>,
    pub persisted: PersistedOrchestrationCheckpointV1,
}

impl Debug for OrchestrationTaskRunResultV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OrchestrationTaskRunResultV1")
            .field("outcome", &self.outcome)
            .field("task_id", &self.task_id)
            .field("role_result_count", &self.role_results.len())
            .field("persisted", &self.persisted)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrchestrationRuntimeError {
    Contract(OrchestrationError),
    Project(ProjectError),
    BackendUnavailable,
    RecoveryRequired,
}

impl OrchestrationRuntimeError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.reason_code(),
            Self::Project(error) => error.reason_code(),
            Self::BackendUnavailable => "orchestration-backend-unavailable",
            Self::RecoveryRequired => "orchestration-recovery-required",
        }
    }
}

impl Display for OrchestrationRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for OrchestrationRuntimeError {}

impl From<OrchestrationError> for OrchestrationRuntimeError {
    fn from(error: OrchestrationError) -> Self {
        Self::Contract(error)
    }
}

impl From<ProjectError> for OrchestrationRuntimeError {
    fn from(error: ProjectError) -> Self {
        Self::Project(error)
    }
}

#[derive(Clone)]
pub struct OrchestrationCheckpointStore {
    projects: ProjectStateService,
}

impl OrchestrationCheckpointStore {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn create(
        &self,
        plan: &OrchestrationPlanV1,
        checkpoint: OrchestrationCheckpointV1,
    ) -> Result<PersistedOrchestrationCheckpointV1, OrchestrationRuntimeError> {
        if checkpoint.generation != 0 || checkpoint.status != OrchestrationRunStatus::Planned {
            return Err(OrchestrationError::InvalidCheckpoint.into());
        }
        let bytes = encode_run_document(plan, &checkpoint)?;
        let commit = self.projects.replace_orchestration_checkpoint(
            &checkpoint.project_id,
            checkpoint.expected_project_revision,
            checkpoint.run_id.as_str(),
            None,
            &bytes,
        )?;
        Ok(PersistedOrchestrationCheckpointV1 {
            checkpoint,
            document_sha256: commit.document_sha256,
        })
    }

    pub fn load(
        &self,
        plan: &OrchestrationPlanV1,
        project_id: &ProjectId,
        expected_project_revision: u64,
        run_id: &RunId,
    ) -> Result<Option<PersistedOrchestrationCheckpointV1>, OrchestrationRuntimeError> {
        let Some(document) = self.projects.read_orchestration_checkpoint(
            project_id,
            expected_project_revision,
            run_id.as_str(),
        )?
        else {
            return Ok(None);
        };
        let checkpoint = decode_run_document(
            document.bytes(),
            plan,
            project_id,
            expected_project_revision,
        )?;
        if checkpoint.run_id != *run_id {
            return Err(OrchestrationError::BindingMismatch.into());
        }
        Ok(Some(PersistedOrchestrationCheckpointV1 {
            checkpoint,
            document_sha256: document.sha256().to_owned(),
        }))
    }

    pub fn discover(
        &self,
        graph: &OrchestrationTaskGraphV1,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<Vec<DiscoveredOrchestrationRunV1>, OrchestrationRuntimeError> {
        self.projects
            .list_orchestration_checkpoints(project_id, expected_project_revision)?
            .into_iter()
            .map(|entry| {
                let document =
                    serde_json::from_slice::<OrchestrationRunDocumentV1>(entry.document().bytes())
                        .map_err(|_| OrchestrationError::InvalidJson)?;
                let plan = OrchestrationPlanV1::try_new(graph.clone(), document.profile.clone())?;
                let checkpoint = decode_run_document(
                    entry.document().bytes(),
                    &plan,
                    project_id,
                    expected_project_revision,
                )?;
                if checkpoint.run_id.as_str() != entry.checkpoint_id() {
                    return Err(OrchestrationError::BindingMismatch.into());
                }
                Ok(DiscoveredOrchestrationRunV1 {
                    plan,
                    persisted: PersistedOrchestrationCheckpointV1 {
                        checkpoint,
                        document_sha256: entry.document().sha256().to_owned(),
                    },
                })
            })
            .collect()
    }

    pub fn replace(
        &self,
        plan: &OrchestrationPlanV1,
        current: &PersistedOrchestrationCheckpointV1,
        checkpoint: OrchestrationCheckpointV1,
    ) -> Result<PersistedOrchestrationCheckpointV1, OrchestrationRuntimeError> {
        current.checkpoint.to_canonical_json(plan)?;
        let next_generation = current
            .checkpoint
            .generation
            .checked_add(1)
            .ok_or(OrchestrationError::LimitExceeded)?;
        if checkpoint.run_id != current.checkpoint.run_id
            || checkpoint.project_id != current.checkpoint.project_id
            || checkpoint.expected_project_revision != current.checkpoint.expected_project_revision
            || checkpoint.generation != next_generation
        {
            return Err(OrchestrationError::InvalidCheckpoint.into());
        }
        let bytes = encode_run_document(plan, &checkpoint)?;
        let commit = self.projects.replace_orchestration_checkpoint(
            &checkpoint.project_id,
            checkpoint.expected_project_revision,
            checkpoint.run_id.as_str(),
            Some(&current.document_sha256),
            &bytes,
        )?;
        Ok(PersistedOrchestrationCheckpointV1 {
            checkpoint,
            document_sha256: commit.document_sha256,
        })
    }

    fn verify_current(
        &self,
        plan: &OrchestrationPlanV1,
        current: &PersistedOrchestrationCheckpointV1,
    ) -> Result<(), OrchestrationRuntimeError> {
        let loaded = self
            .load(
                plan,
                &current.checkpoint.project_id,
                current.checkpoint.expected_project_revision,
                &current.checkpoint.run_id,
            )?
            .ok_or(ProjectError::RevisionConflict)?;
        if loaded == *current {
            Ok(())
        } else {
            Err(ProjectError::RevisionConflict.into())
        }
    }
}

fn encode_run_document(
    plan: &OrchestrationPlanV1,
    checkpoint: &OrchestrationCheckpointV1,
) -> Result<Vec<u8>, OrchestrationError> {
    checkpoint.to_canonical_json(plan)?;
    serde_json_canonicalizer::to_vec(&OrchestrationRunDocumentV1 {
        schema_version: RUN_DOCUMENT_SCHEMA_VERSION,
        profile: plan.profile().clone(),
        checkpoint: checkpoint.clone(),
    })
    .map_err(|_| OrchestrationError::SerializationFailed)
}

fn decode_run_document(
    input: &[u8],
    plan: &OrchestrationPlanV1,
    project_id: &ProjectId,
    expected_project_revision: u64,
) -> Result<OrchestrationCheckpointV1, OrchestrationError> {
    let document = serde_json::from_slice::<OrchestrationRunDocumentV1>(input)
        .map_err(|_| OrchestrationError::InvalidJson)?;
    if document.schema_version != RUN_DOCUMENT_SCHEMA_VERSION || document.profile != *plan.profile()
    {
        return Err(OrchestrationError::BindingMismatch);
    }
    let checkpoint_bytes = document.checkpoint.to_canonical_json(plan)?;
    let checkpoint =
        plan.restore_checkpoint(&checkpoint_bytes, project_id, expected_project_revision)?;
    let canonical = serde_json_canonicalizer::to_vec(&OrchestrationRunDocumentV1 {
        schema_version: document.schema_version,
        profile: document.profile,
        checkpoint: checkpoint.clone(),
    })
    .map_err(|_| OrchestrationError::SerializationFailed)?;
    if canonical != input {
        return Err(OrchestrationError::NonCanonicalJson);
    }
    Ok(checkpoint)
}

pub struct OrchestrationTaskExecutor {
    runners: BTreeMap<BackendId, BoundedAgentRunner>,
    input_builder: Arc<dyn OrchestrationRoleInputBuilder>,
    store: OrchestrationCheckpointStore,
}

impl OrchestrationTaskExecutor {
    pub fn try_new(
        runners: impl IntoIterator<Item = (BackendId, BoundedAgentRunner)>,
        input_builder: Arc<dyn OrchestrationRoleInputBuilder>,
        store: OrchestrationCheckpointStore,
    ) -> Result<Self, OrchestrationRuntimeError> {
        let mut mapped = BTreeMap::new();
        for (backend_id, runner) in runners {
            let descriptor = runner.backend_descriptor();
            descriptor
                .validate()
                .map_err(|_| OrchestrationRuntimeError::BackendUnavailable)?;
            if descriptor.backend_id != backend_id || mapped.insert(backend_id, runner).is_some() {
                return Err(OrchestrationRuntimeError::BackendUnavailable);
            }
        }
        if mapped.is_empty() {
            return Err(OrchestrationRuntimeError::BackendUnavailable);
        }
        Ok(Self {
            runners: mapped,
            input_builder,
            store,
        })
    }

    pub fn run_next<'a>(
        &'a self,
        plan: &'a OrchestrationPlanV1,
        mut persisted: PersistedOrchestrationCheckpointV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<'a, Result<OrchestrationTaskRunResultV1, OrchestrationRuntimeError>>
    {
        Box::pin(async move {
            self.store.verify_current(plan, &persisted)?;
            if persisted.checkpoint.status.is_terminal() {
                return Ok(no_task_result(persisted));
            }
            if cancellation.is_cancelled() {
                return self.cancel(plan, persisted);
            }
            if persisted.checkpoint.status == OrchestrationRunStatus::Paused {
                return Ok(OrchestrationTaskRunResultV1 {
                    outcome: OrchestrationStepOutcome::Paused,
                    task_id: None,
                    role_results: Vec::new(),
                    persisted,
                });
            }
            if persisted
                .checkpoint
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Running)
            {
                return Err(OrchestrationRuntimeError::RecoveryRequired);
            }
            self.validate_profile_runners(plan, &persisted.checkpoint)?;
            if persisted.checkpoint.status == OrchestrationRunStatus::Planned {
                let mut checkpoint = persisted.checkpoint.clone();
                checkpoint.start(plan, checkpoint.generation)?;
                persisted = self.store.replace(plan, &persisted, checkpoint)?;
            }
            let task_id = persisted
                .checkpoint
                .next_ready_task()
                .cloned()
                .ok_or(OrchestrationError::InvalidCheckpoint)?;
            let mut checkpoint = persisted.checkpoint.clone();
            checkpoint.begin_task(plan, checkpoint.generation, &task_id)?;
            persisted = self.store.replace(plan, &persisted, checkpoint)?;

            let mut role_results = Vec::new();
            for role in plan.profile().roles() {
                if cancellation.is_cancelled() {
                    return self.cancel(plan, persisted);
                }
                self.store.verify_current(plan, &persisted)?;
                let backend_id = plan
                    .profile()
                    .backend_for_role(*role)
                    .ok_or(OrchestrationRuntimeError::BackendUnavailable)?;
                let runner = self
                    .runners
                    .get(backend_id)
                    .ok_or(OrchestrationRuntimeError::BackendUnavailable)?;
                let attempt = persisted
                    .checkpoint
                    .tasks
                    .iter()
                    .find(|task| task.task_id == task_id)
                    .map(|task| task.attempts)
                    .ok_or(OrchestrationError::InvalidCheckpoint)?;
                let role_run_id = derive_role_run_id(
                    &persisted.checkpoint.run_id,
                    &persisted.checkpoint.project_id,
                    &task_id,
                    attempt,
                    *role,
                )?;
                let context = OrchestrationRoleInputContextV1 {
                    orchestration_run_id: &persisted.checkpoint.run_id,
                    role_run_id: &role_run_id,
                    project_id: &persisted.checkpoint.project_id,
                    expected_project_revision: persisted.checkpoint.expected_project_revision,
                    task_id: &task_id,
                    attempt,
                    role: *role,
                    backend_id,
                    prior_role_results: &role_results,
                };
                let mut input = match self.input_builder.build(context) {
                    Ok(input) => input,
                    Err(_) => {
                        return self.fail(
                            plan,
                            persisted,
                            task_id,
                            role_results,
                            OrchestrationFailureCode::RoleInputInvalid,
                            false,
                        );
                    }
                };
                input.request.run_id = role_run_id;
                if input.project_id.as_ref() != Some(&persisted.checkpoint.project_id)
                    || input.expected_project_revision
                        != Some(persisted.checkpoint.expected_project_revision)
                {
                    return self.fail(
                        plan,
                        persisted,
                        task_id,
                        role_results,
                        OrchestrationFailureCode::RoleInputInvalid,
                        false,
                    );
                }
                let result = match runner.run(input, cancellation.clone()).await {
                    Ok(result) if !result.content.trim().is_empty() => result,
                    Ok(_) => {
                        return self.fail(
                            plan,
                            persisted,
                            task_id,
                            role_results,
                            OrchestrationFailureCode::RoleOutputInvalid,
                            false,
                        );
                    }
                    Err(AgentRunError::Cancelled) => return self.cancel(plan, persisted),
                    Err(error) => {
                        let (failure_code, retryable) = map_run_error(&error);
                        return self.fail(
                            plan,
                            persisted,
                            task_id,
                            role_results,
                            failure_code,
                            retryable,
                        );
                    }
                };
                if result.backend_id != *backend_id {
                    return self.fail(
                        plan,
                        persisted,
                        task_id,
                        role_results,
                        OrchestrationFailureCode::RoleOutputInvalid,
                        false,
                    );
                }
                let output_sha256 = sha256(result.content.as_bytes());
                let mut checkpoint = persisted.checkpoint.clone();
                checkpoint.complete_role(
                    plan,
                    checkpoint.generation,
                    &task_id,
                    output_sha256.clone(),
                )?;
                persisted = self.store.replace(plan, &persisted, checkpoint)?;
                role_results.push(OrchestrationRoleResultV1 {
                    task_id: task_id.clone(),
                    role: *role,
                    output_sha256,
                    agent_result: result,
                });
            }

            let outcome = match persisted.checkpoint.status {
                OrchestrationRunStatus::Completed => OrchestrationStepOutcome::RunCompleted,
                OrchestrationRunStatus::Failed => OrchestrationStepOutcome::RunFailed,
                _ => OrchestrationStepOutcome::TaskCompleted,
            };
            Ok(OrchestrationTaskRunResultV1 {
                outcome,
                task_id: Some(task_id),
                role_results,
                persisted,
            })
        })
    }

    pub fn recover(
        &self,
        plan: &OrchestrationPlanV1,
        current: PersistedOrchestrationCheckpointV1,
    ) -> Result<PersistedOrchestrationCheckpointV1, OrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.recover_interrupted(plan, checkpoint.generation)?;
        self.store.replace(plan, &current, checkpoint)
    }

    pub fn resume(
        &self,
        plan: &OrchestrationPlanV1,
        current: PersistedOrchestrationCheckpointV1,
    ) -> Result<PersistedOrchestrationCheckpointV1, OrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.resume(plan, checkpoint.generation)?;
        self.store.replace(plan, &current, checkpoint)
    }

    pub fn pause(
        &self,
        plan: &OrchestrationPlanV1,
        current: PersistedOrchestrationCheckpointV1,
    ) -> Result<PersistedOrchestrationCheckpointV1, OrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.pause(plan, checkpoint.generation)?;
        self.store.replace(plan, &current, checkpoint)
    }

    pub fn cancel_run(
        &self,
        plan: &OrchestrationPlanV1,
        current: PersistedOrchestrationCheckpointV1,
    ) -> Result<PersistedOrchestrationCheckpointV1, OrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.cancel(plan, checkpoint.generation)?;
        self.store.replace(plan, &current, checkpoint)
    }

    fn validate_profile_runners(
        &self,
        plan: &OrchestrationPlanV1,
        checkpoint: &OrchestrationCheckpointV1,
    ) -> Result<(), OrchestrationRuntimeError> {
        let required = plan
            .profile()
            .roles()
            .iter()
            .map(|role| {
                plan.profile()
                    .backend_for_role(*role)
                    .ok_or(OrchestrationRuntimeError::BackendUnavailable)
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let root = self
            .store
            .projects
            .resolve_project_root(&checkpoint.project_id)?;
        for backend_id in required {
            let runner = self
                .runners
                .get(backend_id)
                .ok_or(OrchestrationRuntimeError::BackendUnavailable)?;
            let Some(scope) = runner.project_scope() else {
                return Err(OrchestrationRuntimeError::BackendUnavailable);
            };
            if !scope_matches_checkpoint(scope, checkpoint, root.path()) {
                return Err(OrchestrationRuntimeError::BackendUnavailable);
            }
        }
        Ok(())
    }

    fn fail(
        &self,
        plan: &OrchestrationPlanV1,
        current: PersistedOrchestrationCheckpointV1,
        task_id: OrchestrationTaskId,
        role_results: Vec<OrchestrationRoleResultV1>,
        failure_code: OrchestrationFailureCode,
        retryable: bool,
    ) -> Result<OrchestrationTaskRunResultV1, OrchestrationRuntimeError> {
        let mut checkpoint = current.checkpoint.clone();
        checkpoint.fail_task(
            plan,
            checkpoint.generation,
            &task_id,
            failure_code,
            retryable,
        )?;
        let persisted = self.store.replace(plan, &current, checkpoint)?;
        let task_state = persisted
            .checkpoint
            .tasks
            .iter()
            .find(|task| task.task_id == task_id)
            .map(|task| task.state)
            .ok_or(OrchestrationError::InvalidCheckpoint)?;
        let outcome = if task_state == OrchestrationTaskState::Ready {
            OrchestrationStepOutcome::TaskRetryReady
        } else if persisted.checkpoint.status == OrchestrationRunStatus::Failed {
            OrchestrationStepOutcome::RunFailed
        } else {
            OrchestrationStepOutcome::TaskFailed
        };
        Ok(OrchestrationTaskRunResultV1 {
            outcome,
            task_id: Some(task_id),
            role_results,
            persisted,
        })
    }

    fn cancel(
        &self,
        plan: &OrchestrationPlanV1,
        current: PersistedOrchestrationCheckpointV1,
    ) -> Result<OrchestrationTaskRunResultV1, OrchestrationRuntimeError> {
        let persisted = self.cancel_run(plan, current)?;
        Ok(OrchestrationTaskRunResultV1 {
            outcome: OrchestrationStepOutcome::RunCancelled,
            task_id: None,
            role_results: Vec::new(),
            persisted,
        })
    }
}

fn scope_matches_checkpoint(
    scope: &ProjectExecutionScope,
    checkpoint: &OrchestrationCheckpointV1,
    root: &std::path::Path,
) -> bool {
    scope.project_id == checkpoint.project_id
        && scope.semantic_revision == checkpoint.expected_project_revision
        && scope.canonical_root == root
}

fn no_task_result(persisted: PersistedOrchestrationCheckpointV1) -> OrchestrationTaskRunResultV1 {
    let outcome = match persisted.checkpoint.status {
        OrchestrationRunStatus::Completed => OrchestrationStepOutcome::RunCompleted,
        OrchestrationRunStatus::Failed => OrchestrationStepOutcome::RunFailed,
        OrchestrationRunStatus::Cancelled => OrchestrationStepOutcome::RunCancelled,
        OrchestrationRunStatus::Paused => OrchestrationStepOutcome::Paused,
        OrchestrationRunStatus::Planned | OrchestrationRunStatus::Running => {
            OrchestrationStepOutcome::TaskFailed
        }
    };
    OrchestrationTaskRunResultV1 {
        outcome,
        task_id: None,
        role_results: Vec::new(),
        persisted,
    }
}

fn derive_role_run_id(
    orchestration_run_id: &RunId,
    project_id: &ProjectId,
    task_id: &OrchestrationTaskId,
    attempt: u8,
    role: OrchestrationRole,
) -> Result<RunId, OrchestrationError> {
    let mut digest = Sha256::new();
    digest.update(ROLE_RUN_DOMAIN);
    digest.update([0]);
    digest.update(orchestration_run_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(project_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(task_id.as_str().as_bytes());
    digest.update([0, attempt]);
    digest.update([match role {
        OrchestrationRole::Primary => 1,
        OrchestrationRole::Reviewer => 2,
        OrchestrationRole::Verifier => 3,
    }]);
    let value = format!("run_{:x}", digest.finalize());
    RunId::parse(value[..36].to_owned()).map_err(|_| OrchestrationError::InvalidCheckpoint)
}

fn map_run_error(error: &AgentRunError) -> (OrchestrationFailureCode, bool) {
    match error {
        AgentRunError::PreflightFailed(_) => (OrchestrationFailureCode::BackendUnavailable, true),
        AgentRunError::Backend(error) => {
            let failure_code = match error.code {
                crate::AgentBackendErrorCode::AuthenticationUnavailable
                | crate::AgentBackendErrorCode::CapabilityUnavailable
                | crate::AgentBackendErrorCode::TransportUnavailable => {
                    OrchestrationFailureCode::BackendUnavailable
                }
                crate::AgentBackendErrorCode::ProviderRejected => {
                    OrchestrationFailureCode::BackendRejected
                }
                crate::AgentBackendErrorCode::InvalidRequest
                | crate::AgentBackendErrorCode::ResponseInvalid
                | crate::AgentBackendErrorCode::Cancelled => {
                    OrchestrationFailureCode::BackendFailed
                }
            };
            (failure_code, error.retry_class.is_some())
        }
        AgentRunError::PolicyDenied(_) => (OrchestrationFailureCode::ToolDenied, false),
        AgentRunError::ToolHost(_) => (OrchestrationFailureCode::ToolFailed, false),
        AgentRunError::InvalidRequest => (OrchestrationFailureCode::RoleInputInvalid, false),
        AgentRunError::EventInvalid | AgentRunError::EventSequenceInvalid => {
            (OrchestrationFailureCode::RoleOutputInvalid, false)
        }
        AgentRunError::LimitExceeded => (OrchestrationFailureCode::BackendFailed, false),
        AgentRunError::Cancelled => (OrchestrationFailureCode::BackendFailed, false),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::future::Future;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::task::{Context, Poll, Waker};

    use qiongli_config::resolve_config_root;
    use qiongli_project::{
        ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions, ProjectStateService,
    };

    use crate::{
        AgentBackend, AgentBackendError, AgentBackendErrorCode, AgentEventV1, AgentFinishReason,
        AgentMessageV1, AgentRequirementsV1, AgentResponseConstraintsV1, AgentRetryClass,
        AgentRole, AgentUsageV1, DeterministicFakeBackend, ExecutionLimitsV1, ExecutionProfile,
        InProcessToolHost, OrchestrationExecutionMode, OrchestrationProfileV1,
        OrchestrationTaskGraphV1, OrchestrationTaskSpecV1, ProjectExecutionScope,
        RedactionPolicyV1, ToolId,
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
                "qiongli-orchestration-runtime-{}-{}",
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
                    ProjectRegistrationOptions::new("Orchestration paper", ProjectKind::Article),
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

    struct TestInputBuilder;

    impl OrchestrationRoleInputBuilder for TestInputBuilder {
        fn build(
            &self,
            context: OrchestrationRoleInputContextV1<'_>,
        ) -> Result<AgentRunInputV1, OrchestrationRoleInputError> {
            let prior = context
                .prior_role_results
                .last()
                .map_or("none", |result| result.agent_result.content.as_str());
            Ok(AgentRunInputV1 {
                request: crate::AgentRequestV1 {
                    schema_version: 1,
                    run_id: context.role_run_id.clone(),
                    model: "deterministic-v1".to_owned(),
                    messages: vec![AgentMessageV1 {
                        role: AgentRole::User,
                        content: format!(
                            "Task {} role {:?}; prior output: {prior}",
                            context.task_id.as_str(),
                            context.role
                        ),
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
                purpose: "Execute one bounded orchestration role.".to_owned(),
                project_id: Some(context.project_id.clone()),
                expected_project_revision: Some(context.expected_project_revision),
            })
        }
    }

    fn ready<T>(mut future: impl Future<Output = T> + Unpin) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        match std::pin::Pin::new(&mut future).poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("deterministic orchestration future must be immediately ready"),
        }
    }

    fn run_id() -> RunId {
        RunId::parse(format!("run_{}", "8".repeat(32))).unwrap()
    }

    fn plan(mode: OrchestrationExecutionMode, attempts: u8) -> OrchestrationPlanV1 {
        let graph = OrchestrationTaskGraphV1::try_new(vec![
            OrchestrationTaskSpecV1::try_new("A1", [] as [&str; 0], [] as [&str; 0]).unwrap(),
        ])
        .unwrap();
        let backend = BackendId::parse("deterministic-fake").unwrap();
        let profile = OrchestrationProfileV1::try_new(
            "test",
            mode,
            backend.clone(),
            matches!(
                mode,
                OrchestrationExecutionMode::Duo | OrchestrationExecutionMode::Triad
            )
            .then_some(backend.clone()),
            (mode == OrchestrationExecutionMode::Triad).then_some(backend),
            attempts,
            false,
        )
        .unwrap();
        OrchestrationPlanV1::try_new(graph, profile).unwrap()
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

    fn executor(
        fixture: &Fixture,
        backend: Arc<DeterministicFakeBackend>,
    ) -> (OrchestrationTaskExecutor, OrchestrationCheckpointStore) {
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
        let backend_id = backend.descriptor().backend_id;
        let runner = BoundedAgentRunner::new(backend, InProcessToolHost::new(), policy);
        let store = OrchestrationCheckpointStore::new(fixture.projects.clone());
        let executor = OrchestrationTaskExecutor::try_new(
            [(backend_id, runner)],
            Arc::new(TestInputBuilder),
            store.clone(),
        )
        .unwrap();
        (executor, store)
    }

    fn create_checkpoint(
        fixture: &Fixture,
        store: &OrchestrationCheckpointStore,
        plan: &OrchestrationPlanV1,
    ) -> PersistedOrchestrationCheckpointV1 {
        store
            .create(
                plan,
                plan.new_checkpoint(run_id(), fixture.project_id.clone(), 1)
                    .unwrap(),
            )
            .unwrap()
    }

    #[test]
    fn duo_task_runs_in_order_and_persists_hashes_without_model_text() {
        let fixture = Fixture::new();
        let plan = plan(OrchestrationExecutionMode::Duo, 1);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                completed_script("primary draft"),
                completed_script("reviewed draft"),
            ])
            .unwrap(),
        );
        let (executor, store) = executor(&fixture, backend.clone());
        let persisted = create_checkpoint(&fixture, &store, &plan);

        let result = ready(executor.run_next(&plan, persisted, CancellationToken::new())).unwrap();

        assert_eq!(result.outcome, OrchestrationStepOutcome::RunCompleted);
        assert_eq!(result.role_results.len(), 2);
        assert_ne!(
            result.role_results[0].agent_result.run_id,
            result.role_results[1].agent_result.run_id
        );
        assert_eq!(
            result.persisted.checkpoint().status,
            OrchestrationRunStatus::Completed
        );
        assert_eq!(result.persisted.checkpoint().tasks[0].role_outputs.len(), 2);
        let reviewer_request = backend.last_request().unwrap();
        assert!(
            reviewer_request.messages[0]
                .content
                .contains("primary draft")
        );

        let raw = fixture
            .projects
            .read_orchestration_checkpoint(&fixture.project_id, 1, run_id().as_str())
            .unwrap()
            .unwrap();
        let raw_text = std::str::from_utf8(raw.bytes()).unwrap();
        assert!(!raw_text.contains("primary draft"));
        assert!(!raw_text.contains("reviewed draft"));
        assert!(!format!("{result:?}").contains("primary draft"));
        assert_eq!(
            store
                .load(&plan, &fixture.project_id, 1, &run_id())
                .unwrap()
                .unwrap(),
            result.persisted
        );
        let discovered = store
            .discover(plan.graph(), &fixture.project_id, 1)
            .unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].plan(), &plan);
        assert_eq!(discovered[0].persisted(), &result.persisted);
    }

    #[test]
    fn retryable_backend_failure_is_persisted_and_replayed_as_a_whole_task() {
        let fixture = Fixture::new();
        let plan = plan(OrchestrationExecutionMode::Solo, 2);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![
                vec![Err(AgentBackendError::new(
                    AgentBackendErrorCode::TransportUnavailable,
                    Some(AgentRetryClass::NetworkTransient),
                ))],
                completed_script("recovered output"),
            ])
            .unwrap(),
        );
        let (executor, store) = executor(&fixture, backend.clone());
        let persisted = create_checkpoint(&fixture, &store, &plan);

        let first = ready(executor.run_next(&plan, persisted, CancellationToken::new())).unwrap();
        assert_eq!(first.outcome, OrchestrationStepOutcome::TaskRetryReady);
        assert_eq!(
            first.persisted.checkpoint().tasks[0].failure_code,
            Some(OrchestrationFailureCode::BackendUnavailable)
        );
        assert_eq!(first.persisted.checkpoint().tasks[0].attempts, 1);

        let second =
            ready(executor.run_next(&plan, first.persisted, CancellationToken::new())).unwrap();
        assert_eq!(second.outcome, OrchestrationStepOutcome::RunCompleted);
        assert_eq!(second.persisted.checkpoint().tasks[0].attempts, 2);
        assert_eq!(backend.start_count(), 2);
    }

    #[test]
    fn interrupted_role_requires_explicit_recovery_before_reexecution() {
        let fixture = Fixture::new();
        let plan = plan(OrchestrationExecutionMode::Solo, 2);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![completed_script("recovered")]).unwrap(),
        );
        let (executor, store) = executor(&fixture, backend.clone());
        let initial = create_checkpoint(&fixture, &store, &plan);
        let mut started_checkpoint = initial.checkpoint().clone();
        started_checkpoint
            .start(&plan, started_checkpoint.generation)
            .unwrap();
        let started = store.replace(&plan, &initial, started_checkpoint).unwrap();
        let mut running_checkpoint = started.checkpoint().clone();
        running_checkpoint
            .begin_task(
                &plan,
                running_checkpoint.generation,
                &OrchestrationTaskId::parse("A1").unwrap(),
            )
            .unwrap();
        let running = store.replace(&plan, &started, running_checkpoint).unwrap();

        assert_eq!(
            ready(executor.run_next(&plan, running.clone(), CancellationToken::new())).unwrap_err(),
            OrchestrationRuntimeError::RecoveryRequired
        );
        assert_eq!(backend.start_count(), 0);
        let paused = executor.recover(&plan, running).unwrap();
        assert_eq!(paused.checkpoint().status, OrchestrationRunStatus::Paused);
        assert_eq!(
            paused.checkpoint().tasks[0].failure_code,
            Some(OrchestrationFailureCode::TaskInterrupted)
        );
        let resumed = executor.resume(&plan, paused).unwrap();
        let completed = ready(executor.run_next(&plan, resumed, CancellationToken::new())).unwrap();
        assert_eq!(completed.outcome, OrchestrationStepOutcome::RunCompleted);
        assert_eq!(completed.persisted.checkpoint().tasks[0].attempts, 2);
    }

    #[test]
    fn cancellation_and_stale_checkpoint_writes_stop_before_backend_execution() {
        let fixture = Fixture::new();
        let plan = plan(OrchestrationExecutionMode::Solo, 1);
        let backend = Arc::new(
            DeterministicFakeBackend::from_turns(vec![completed_script("unused")]).unwrap(),
        );
        let (executor, store) = executor(&fixture, backend.clone());
        let initial = create_checkpoint(&fixture, &store, &plan);
        let stale = initial.clone();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let cancelled = ready(executor.run_next(&plan, initial, cancellation)).unwrap();
        assert_eq!(cancelled.outcome, OrchestrationStepOutcome::RunCancelled);
        assert_eq!(backend.start_count(), 0);

        let mut stale_next = stale.checkpoint().clone();
        stale_next.cancel(&plan, stale_next.generation).unwrap();
        assert_eq!(
            store.replace(&plan, &stale, stale_next).unwrap_err(),
            OrchestrationRuntimeError::Project(ProjectError::RevisionConflict)
        );
    }

    #[test]
    fn run_document_rejects_unknown_envelope_fields_without_exposing_values() {
        let fixture = Fixture::new();
        let plan = plan(OrchestrationExecutionMode::Solo, 1);
        let store = OrchestrationCheckpointStore::new(fixture.projects.clone());
        let persisted = create_checkpoint(&fixture, &store, &plan);
        let raw = fixture
            .projects
            .read_orchestration_checkpoint(&fixture.project_id, 1, run_id().as_str())
            .unwrap()
            .unwrap();
        let mut value = serde_json::from_slice::<serde_json::Value>(raw.bytes()).unwrap();
        value.as_object_mut().unwrap().insert(
            "privateCanary".to_owned(),
            serde_json::json!("do-not-render"),
        );
        let bytes = serde_json_canonicalizer::to_vec(&value).unwrap();
        fixture
            .projects
            .replace_orchestration_checkpoint(
                &fixture.project_id,
                1,
                run_id().as_str(),
                Some(persisted.document_sha256()),
                &bytes,
            )
            .unwrap();

        let error = store
            .load(&plan, &fixture.project_id, 1, &run_id())
            .unwrap_err();
        assert_eq!(
            error,
            OrchestrationRuntimeError::Contract(OrchestrationError::InvalidJson)
        );
        assert!(!error.to_string().contains("do-not-render"));
    }

    #[test]
    fn mismatched_runner_scope_fails_before_checkpoint_or_backend_mutation() {
        let fixture = Fixture::new();
        let plan = plan(OrchestrationExecutionMode::Solo, 1);
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
        let runner = BoundedAgentRunner::new(backend.clone(), InProcessToolHost::new(), policy);
        let store = OrchestrationCheckpointStore::new(fixture.projects.clone());
        let executor = OrchestrationTaskExecutor::try_new(
            [(backend.descriptor().backend_id, runner)],
            Arc::new(TestInputBuilder),
            store.clone(),
        )
        .unwrap();
        let persisted = create_checkpoint(&fixture, &store, &plan);

        assert_eq!(
            ready(executor.run_next(&plan, persisted.clone(), CancellationToken::new()))
                .unwrap_err(),
            OrchestrationRuntimeError::BackendUnavailable
        );
        assert_eq!(backend.start_count(), 0);
        assert_eq!(
            store
                .load(&plan, &fixture.project_id, 1, &run_id())
                .unwrap()
                .unwrap(),
            persisted
        );
    }
}
