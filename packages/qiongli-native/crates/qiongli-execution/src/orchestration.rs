use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use qiongli_content::EmbeddedContent;
use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{BackendId, OrchestrationProfileId, OrchestrationTaskId, RunId};

pub const ORCHESTRATION_SCHEMA_VERSION: u32 = 1;
pub const ORCHESTRATION_WORKFLOW_SOURCE_PATH: &str = "standards/research-workflow-contract.yaml";

const EMBEDDED_WORKFLOW_TASK_GRAPH_V1: &str =
    include_str!("../resources/research-workflow-task-graph-v1.json");

const MAX_GRAPH_BYTES: usize = 1_048_576;
const MAX_CHECKPOINT_BYTES: usize = 1_048_576;
const MAX_PROFILE_BYTES: usize = 65_536;
const MAX_TASKS: usize = 128;
const MAX_DEPENDENCIES_PER_TASK: usize = 64;
const MAX_TASK_ATTEMPTS: u8 = 3;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationExecutionMode {
    Solo,
    Duo,
    Triad,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationRunStatus {
    Planned,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl OrchestrationRunStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationTaskState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Blocked,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum OrchestrationFailureCode {
    #[serde(rename = "orchestration-role-input-invalid")]
    RoleInputInvalid,
    #[serde(rename = "orchestration-backend-unavailable")]
    BackendUnavailable,
    #[serde(rename = "orchestration-backend-rejected")]
    BackendRejected,
    #[serde(rename = "orchestration-backend-failed")]
    BackendFailed,
    #[serde(rename = "orchestration-tool-denied")]
    ToolDenied,
    #[serde(rename = "orchestration-tool-failed")]
    ToolFailed,
    #[serde(rename = "orchestration-role-output-invalid")]
    RoleOutputInvalid,
    #[serde(rename = "orchestration-task-interrupted")]
    TaskInterrupted,
    #[serde(rename = "orchestration-run-cancelled")]
    RunCancelled,
    #[serde(rename = "orchestration-prerequisite-unavailable")]
    PrerequisiteUnavailable,
    #[serde(rename = "orchestration-stopped-after-failure")]
    StoppedAfterFailure,
}

#[derive(Clone, Copy, Debug, schemars::JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationRole {
    Primary,
    Reviewer,
    Verifier,
}

impl OrchestrationTaskState {
    const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Blocked | Self::Cancelled,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidGraph,
    GraphCycle,
    InvalidProfile,
    InvalidCheckpoint,
    BindingMismatch,
    StaleGeneration,
    InvalidTransition,
    TaskNotReady,
    LimitExceeded,
    SerializationFailed,
    WorkflowContractUnavailable,
    WorkflowContractMismatch,
    WorkflowProjectionInvalid,
}

impl OrchestrationError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "orchestration-input-too-large",
            Self::InvalidJson => "orchestration-json-invalid",
            Self::NonCanonicalJson => "orchestration-json-noncanonical",
            Self::InvalidGraph => "orchestration-graph-invalid",
            Self::GraphCycle => "orchestration-graph-cycle",
            Self::InvalidProfile => "orchestration-profile-invalid",
            Self::InvalidCheckpoint => "orchestration-checkpoint-invalid",
            Self::BindingMismatch => "orchestration-checkpoint-binding-mismatch",
            Self::StaleGeneration => "orchestration-generation-stale",
            Self::InvalidTransition => "orchestration-transition-invalid",
            Self::TaskNotReady => "orchestration-task-not-ready",
            Self::LimitExceeded => "orchestration-limit-exhausted",
            Self::SerializationFailed => "orchestration-serialization-failed",
            Self::WorkflowContractUnavailable => "orchestration-workflow-contract-unavailable",
            Self::WorkflowContractMismatch => "orchestration-workflow-contract-mismatch",
            Self::WorkflowProjectionInvalid => "orchestration-workflow-projection-invalid",
        }
    }
}

impl Display for OrchestrationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for OrchestrationError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrchestrationTaskSpecV1 {
    pub task_id: OrchestrationTaskId,
    pub prerequisites_all: Vec<OrchestrationTaskId>,
    pub prerequisites_any: Vec<OrchestrationTaskId>,
}

impl OrchestrationTaskSpecV1 {
    pub fn try_new(
        task_id: impl Into<String>,
        prerequisites_all: impl IntoIterator<Item = impl Into<String>>,
        prerequisites_any: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, OrchestrationError> {
        let task_id = OrchestrationTaskId::parse(task_id.into())
            .map_err(|_| OrchestrationError::InvalidGraph)?;
        let prerequisites_all = parse_dependencies(prerequisites_all)?;
        let prerequisites_any = parse_dependencies(prerequisites_any)?;
        let task = Self {
            task_id,
            prerequisites_all,
            prerequisites_any,
        };
        task.validate_local()?;
        Ok(task)
    }

    fn validate_local(&self) -> Result<(), OrchestrationError> {
        OrchestrationTaskId::parse(self.task_id.as_str())
            .map_err(|_| OrchestrationError::InvalidGraph)?;
        if self.prerequisites_all.len() > MAX_DEPENDENCIES_PER_TASK
            || self.prerequisites_any.len() > MAX_DEPENDENCIES_PER_TASK
            || !strictly_sorted_unique(&self.prerequisites_all)
            || !strictly_sorted_unique(&self.prerequisites_any)
            || self.prerequisites_all.contains(&self.task_id)
            || self.prerequisites_any.contains(&self.task_id)
            || self
                .prerequisites_all
                .iter()
                .any(|dependency| self.prerequisites_any.contains(dependency))
        {
            return Err(OrchestrationError::InvalidGraph);
        }
        for dependency in self.prerequisites_all.iter().chain(&self.prerequisites_any) {
            OrchestrationTaskId::parse(dependency.as_str())
                .map_err(|_| OrchestrationError::InvalidGraph)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrchestrationTaskGraphV1 {
    pub schema_version: u32,
    pub tasks: Vec<OrchestrationTaskSpecV1>,
}

impl OrchestrationTaskGraphV1 {
    pub fn try_new(tasks: Vec<OrchestrationTaskSpecV1>) -> Result<Self, OrchestrationError> {
        let graph = Self {
            schema_version: ORCHESTRATION_SCHEMA_VERSION,
            tasks,
        };
        graph.validate()?;
        Ok(graph)
    }

    pub fn from_canonical_json(input: &[u8]) -> Result<Self, OrchestrationError> {
        if input.len() > MAX_GRAPH_BYTES {
            return Err(OrchestrationError::InputTooLarge);
        }
        let graph =
            serde_json::from_slice::<Self>(input).map_err(|_| OrchestrationError::InvalidJson)?;
        graph.validate()?;
        if graph.to_canonical_json()? != input {
            return Err(OrchestrationError::NonCanonicalJson);
        }
        Ok(graph)
    }

    pub fn from_embedded_content(content: &EmbeddedContent) -> Result<Self, OrchestrationError> {
        let projection = embedded_workflow_projection()?;
        let resource = content
            .read_profile_resource("full", &projection.source_path)
            .map_err(|_| OrchestrationError::WorkflowContractUnavailable)?
            .ok_or(OrchestrationError::WorkflowContractUnavailable)?;
        if resource.entry().sha256 != projection.source_sha256
            || sha256(resource.bytes()) != projection.source_sha256
        {
            return Err(OrchestrationError::WorkflowContractMismatch);
        }
        Ok(projection.graph)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, OrchestrationError> {
        self.validate()?;
        canonical_json(self, MAX_GRAPH_BYTES)
    }

    pub fn digest(&self) -> Result<String, OrchestrationError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    fn validate(&self) -> Result<(), OrchestrationError> {
        if self.schema_version != ORCHESTRATION_SCHEMA_VERSION
            || self.tasks.is_empty()
            || self.tasks.len() > MAX_TASKS
        {
            return Err(OrchestrationError::InvalidGraph);
        }
        let task_ids = self
            .tasks
            .iter()
            .map(|task| task.task_id.clone())
            .collect::<BTreeSet<_>>();
        if task_ids.len() != self.tasks.len() {
            return Err(OrchestrationError::InvalidGraph);
        }
        for task in &self.tasks {
            task.validate_local()?;
            if task
                .prerequisites_all
                .iter()
                .chain(&task.prerequisites_any)
                .any(|dependency| !task_ids.contains(dependency))
            {
                return Err(OrchestrationError::InvalidGraph);
            }
        }
        self.validate_acyclic()
    }

    fn validate_acyclic(&self) -> Result<(), OrchestrationError> {
        let mut indegrees = self
            .tasks
            .iter()
            .map(|task| {
                (
                    task.task_id.clone(),
                    task.prerequisites_all.len() + task.prerequisites_any.len(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut ready = self
            .tasks
            .iter()
            .filter(|task| indegrees[&task.task_id] == 0)
            .map(|task| task.task_id.clone())
            .collect::<Vec<_>>();
        let mut visited = 0_usize;
        while let Some(task_id) = ready.pop() {
            visited += 1;
            for dependent in &self.tasks {
                if dependent.prerequisites_all.contains(&task_id)
                    || dependent.prerequisites_any.contains(&task_id)
                {
                    let Some(indegree) = indegrees.get_mut(&dependent.task_id) else {
                        return Err(OrchestrationError::InvalidGraph);
                    };
                    *indegree = indegree
                        .checked_sub(1)
                        .ok_or(OrchestrationError::InvalidGraph)?;
                    if *indegree == 0 {
                        ready.push(dependent.task_id.clone());
                    }
                }
            }
        }
        if visited == self.tasks.len() {
            Ok(())
        } else {
            Err(OrchestrationError::GraphCycle)
        }
    }

    fn task(&self, task_id: &OrchestrationTaskId) -> Option<&OrchestrationTaskSpecV1> {
        self.tasks.iter().find(|task| &task.task_id == task_id)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EmbeddedWorkflowProjectionV1 {
    schema_version: u32,
    source_path: String,
    source_sha256: String,
    graph: OrchestrationTaskGraphV1,
}

fn embedded_workflow_projection() -> Result<EmbeddedWorkflowProjectionV1, OrchestrationError> {
    let input = EMBEDDED_WORKFLOW_TASK_GRAPH_V1
        .strip_suffix('\n')
        .unwrap_or(EMBEDDED_WORKFLOW_TASK_GRAPH_V1)
        .as_bytes();
    let projection = serde_json::from_slice::<EmbeddedWorkflowProjectionV1>(input)
        .map_err(|_| OrchestrationError::WorkflowProjectionInvalid)?;
    if projection.schema_version != ORCHESTRATION_SCHEMA_VERSION
        || projection.source_path != ORCHESTRATION_WORKFLOW_SOURCE_PATH
        || !valid_sha256(&projection.source_sha256)
        || projection.graph.validate().is_err()
    {
        return Err(OrchestrationError::WorkflowProjectionInvalid);
    }
    Ok(projection)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrchestrationProfileV1 {
    pub schema_version: u32,
    pub profile_id: OrchestrationProfileId,
    pub execution_mode: OrchestrationExecutionMode,
    pub primary_backend: BackendId,
    pub reviewer_backend: Option<BackendId>,
    pub verifier_backend: Option<BackendId>,
    pub max_task_attempts: u8,
    pub stop_on_failure: bool,
}

impl OrchestrationProfileV1 {
    pub fn try_new(
        profile_id: impl Into<String>,
        execution_mode: OrchestrationExecutionMode,
        primary_backend: BackendId,
        reviewer_backend: Option<BackendId>,
        verifier_backend: Option<BackendId>,
        max_task_attempts: u8,
        stop_on_failure: bool,
    ) -> Result<Self, OrchestrationError> {
        let profile = Self {
            schema_version: ORCHESTRATION_SCHEMA_VERSION,
            profile_id: OrchestrationProfileId::parse(profile_id.into())
                .map_err(|_| OrchestrationError::InvalidProfile)?,
            execution_mode,
            primary_backend,
            reviewer_backend,
            verifier_backend,
            max_task_attempts,
            stop_on_failure,
        };
        profile.validate()?;
        Ok(profile)
    }

    pub fn from_canonical_json(input: &[u8]) -> Result<Self, OrchestrationError> {
        if input.len() > MAX_PROFILE_BYTES {
            return Err(OrchestrationError::InputTooLarge);
        }
        let profile =
            serde_json::from_slice::<Self>(input).map_err(|_| OrchestrationError::InvalidJson)?;
        profile.validate()?;
        if profile.to_canonical_json()? != input {
            return Err(OrchestrationError::NonCanonicalJson);
        }
        Ok(profile)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, OrchestrationError> {
        self.validate()?;
        canonical_json(self, MAX_PROFILE_BYTES)
    }

    pub fn digest(&self) -> Result<String, OrchestrationError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    #[must_use]
    pub const fn roles(&self) -> &'static [OrchestrationRole] {
        match self.execution_mode {
            OrchestrationExecutionMode::Solo => &[OrchestrationRole::Primary],
            OrchestrationExecutionMode::Duo => {
                &[OrchestrationRole::Primary, OrchestrationRole::Reviewer]
            }
            OrchestrationExecutionMode::Triad => &[
                OrchestrationRole::Primary,
                OrchestrationRole::Reviewer,
                OrchestrationRole::Verifier,
            ],
        }
    }

    #[must_use]
    pub fn backend_for_role(&self, role: OrchestrationRole) -> Option<&BackendId> {
        match role {
            OrchestrationRole::Primary => Some(&self.primary_backend),
            OrchestrationRole::Reviewer => self.reviewer_backend.as_ref(),
            OrchestrationRole::Verifier => self.verifier_backend.as_ref(),
        }
    }

    fn validate(&self) -> Result<(), OrchestrationError> {
        if self.schema_version != ORCHESTRATION_SCHEMA_VERSION
            || OrchestrationProfileId::parse(self.profile_id.as_str()).is_err()
            || BackendId::parse(self.primary_backend.as_str()).is_err()
            || self.max_task_attempts == 0
            || self.max_task_attempts > MAX_TASK_ATTEMPTS
        {
            return Err(OrchestrationError::InvalidProfile);
        }
        if self
            .reviewer_backend
            .as_ref()
            .is_some_and(|backend| BackendId::parse(backend.as_str()).is_err())
            || self
                .verifier_backend
                .as_ref()
                .is_some_and(|backend| BackendId::parse(backend.as_str()).is_err())
        {
            return Err(OrchestrationError::InvalidProfile);
        }
        let roles_valid = match self.execution_mode {
            OrchestrationExecutionMode::Solo => {
                self.reviewer_backend.is_none() && self.verifier_backend.is_none()
            }
            OrchestrationExecutionMode::Duo => {
                self.reviewer_backend.is_some() && self.verifier_backend.is_none()
            }
            OrchestrationExecutionMode::Triad => {
                self.reviewer_backend.is_some() && self.verifier_backend.is_some()
            }
        };
        roles_valid
            .then_some(())
            .ok_or(OrchestrationError::InvalidProfile)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskCheckpointV1 {
    pub task_id: OrchestrationTaskId,
    pub state: OrchestrationTaskState,
    pub attempts: u8,
    pub active_role: Option<OrchestrationRole>,
    pub role_outputs: Vec<RoleCheckpointV1>,
    pub output_sha256: Option<String>,
    pub failure_code: Option<OrchestrationFailureCode>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoleCheckpointV1 {
    pub role: OrchestrationRole,
    pub backend_id: BackendId,
    pub output_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrchestrationCheckpointV1 {
    pub schema_version: u32,
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub graph_sha256: String,
    pub profile_sha256: String,
    pub generation: u64,
    pub status: OrchestrationRunStatus,
    pub tasks: Vec<TaskCheckpointV1>,
}

impl OrchestrationCheckpointV1 {
    #[must_use]
    pub fn next_ready_task(&self) -> Option<&OrchestrationTaskId> {
        self.tasks
            .iter()
            .find(|task| task.state == OrchestrationTaskState::Ready)
            .map(|task| &task.task_id)
    }

    pub fn to_canonical_json(
        &self,
        plan: &OrchestrationPlanV1,
    ) -> Result<Vec<u8>, OrchestrationError> {
        plan.validate_checkpoint(self)?;
        canonical_json(self, MAX_CHECKPOINT_BYTES)
    }

    pub fn start(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Planned {
            return Err(OrchestrationError::InvalidTransition);
        }
        self.status = OrchestrationRunStatus::Running;
        self.advance_generation()?;
        plan.recompute(self)?;
        plan.validate_checkpoint(self)
    }

    pub fn begin_task(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
        task_id: &OrchestrationTaskId,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Running
            || self
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Running)
        {
            return Err(OrchestrationError::InvalidTransition);
        }
        let task = self.task_mut(task_id)?;
        if task.state != OrchestrationTaskState::Ready {
            return Err(OrchestrationError::TaskNotReady);
        }
        if task.attempts >= plan.profile.max_task_attempts {
            return Err(OrchestrationError::LimitExceeded);
        }
        task.attempts += 1;
        task.state = OrchestrationTaskState::Running;
        task.active_role = Some(OrchestrationRole::Primary);
        task.role_outputs.clear();
        task.output_sha256 = None;
        task.failure_code = None;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn complete_role(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
        task_id: &OrchestrationTaskId,
        output_sha256: impl Into<String>,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Running {
            return Err(OrchestrationError::InvalidTransition);
        }
        let output_sha256 = output_sha256.into();
        if !valid_sha256(&output_sha256) {
            return Err(OrchestrationError::InvalidCheckpoint);
        }
        let current = self
            .tasks
            .iter()
            .find(|task| &task.task_id == task_id)
            .ok_or(OrchestrationError::InvalidCheckpoint)?;
        let active_role = current
            .active_role
            .ok_or(OrchestrationError::InvalidTransition)?;
        let role_index = current.role_outputs.len();
        if plan.profile.roles().get(role_index) != Some(&active_role) {
            return Err(OrchestrationError::InvalidCheckpoint);
        }
        let backend_id = plan
            .profile
            .backend_for_role(active_role)
            .cloned()
            .ok_or(OrchestrationError::InvalidProfile)?;
        let next_role = plan.profile.roles().get(role_index + 1).copied();
        let task = self.task_mut(task_id)?;
        if task.state != OrchestrationTaskState::Running {
            return Err(OrchestrationError::InvalidTransition);
        }
        task.role_outputs.push(RoleCheckpointV1 {
            role: active_role,
            backend_id,
            output_sha256: output_sha256.clone(),
        });
        task.active_role = next_role;
        if next_role.is_none() {
            task.state = OrchestrationTaskState::Completed;
            task.output_sha256 = Some(output_sha256);
        }
        task.failure_code = None;
        self.advance_generation()?;
        if next_role.is_none() {
            plan.recompute(self)?;
        }
        plan.validate_checkpoint(self)
    }

    pub fn fail_task(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
        task_id: &OrchestrationTaskId,
        failure_code: OrchestrationFailureCode,
        retryable: bool,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Running {
            return Err(OrchestrationError::InvalidTransition);
        }
        let task = self.task_mut(task_id)?;
        if task.state != OrchestrationTaskState::Running {
            return Err(OrchestrationError::InvalidTransition);
        }
        task.output_sha256 = None;
        task.active_role = None;
        task.role_outputs.clear();
        task.failure_code = Some(failure_code);
        if retryable && task.attempts < plan.profile.max_task_attempts {
            task.state = OrchestrationTaskState::Ready;
        } else {
            task.state = OrchestrationTaskState::Failed;
        }
        self.advance_generation()?;
        plan.recompute(self)?;
        plan.validate_checkpoint(self)
    }

    pub fn pause(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Running
            || self
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Running)
        {
            return Err(OrchestrationError::InvalidTransition);
        }
        self.status = OrchestrationRunStatus::Paused;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    pub fn recover_interrupted(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Running
            || !self
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Running)
        {
            return Err(OrchestrationError::InvalidTransition);
        }
        for task in &mut self.tasks {
            if task.state == OrchestrationTaskState::Running {
                task.output_sha256 = None;
                task.active_role = None;
                task.role_outputs.clear();
                task.failure_code = Some(OrchestrationFailureCode::TaskInterrupted);
                task.state = if task.attempts < plan.profile.max_task_attempts {
                    OrchestrationTaskState::Ready
                } else {
                    OrchestrationTaskState::Failed
                };
            }
        }
        self.status = OrchestrationRunStatus::Paused;
        self.advance_generation()?;
        plan.recompute_dependencies(self)?;
        plan.validate_checkpoint(self)
    }

    pub fn resume(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status != OrchestrationRunStatus::Paused {
            return Err(OrchestrationError::InvalidTransition);
        }
        self.status = OrchestrationRunStatus::Running;
        self.advance_generation()?;
        plan.recompute(self)?;
        plan.validate_checkpoint(self)
    }

    pub fn cancel(
        &mut self,
        plan: &OrchestrationPlanV1,
        expected_generation: u64,
    ) -> Result<(), OrchestrationError> {
        plan.validate_checkpoint(self)?;
        self.require_generation(expected_generation)?;
        if self.status.is_terminal() {
            return Err(OrchestrationError::InvalidTransition);
        }
        for task in &mut self.tasks {
            if !task.state.is_terminal() {
                task.state = OrchestrationTaskState::Cancelled;
                task.output_sha256 = None;
                task.active_role = None;
                task.role_outputs.clear();
                task.failure_code = Some(OrchestrationFailureCode::RunCancelled);
            }
        }
        self.status = OrchestrationRunStatus::Cancelled;
        self.advance_generation()?;
        plan.validate_checkpoint(self)
    }

    fn require_generation(&self, expected: u64) -> Result<(), OrchestrationError> {
        if self.generation != expected {
            return Err(OrchestrationError::StaleGeneration);
        }
        (self.generation < MAX_SAFE_INTEGER)
            .then_some(())
            .ok_or(OrchestrationError::LimitExceeded)
    }

    fn advance_generation(&mut self) -> Result<(), OrchestrationError> {
        self.generation = self
            .generation
            .checked_add(1)
            .filter(|generation| *generation <= MAX_SAFE_INTEGER)
            .ok_or(OrchestrationError::LimitExceeded)?;
        Ok(())
    }

    fn task_mut(
        &mut self,
        task_id: &OrchestrationTaskId,
    ) -> Result<&mut TaskCheckpointV1, OrchestrationError> {
        self.tasks
            .iter_mut()
            .find(|task| &task.task_id == task_id)
            .ok_or(OrchestrationError::InvalidCheckpoint)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationPlanV1 {
    graph: OrchestrationTaskGraphV1,
    profile: OrchestrationProfileV1,
    graph_sha256: String,
    profile_sha256: String,
}

impl OrchestrationPlanV1 {
    pub fn try_new(
        graph: OrchestrationTaskGraphV1,
        profile: OrchestrationProfileV1,
    ) -> Result<Self, OrchestrationError> {
        graph.validate()?;
        profile.validate()?;
        let graph_sha256 = graph.digest()?;
        let profile_sha256 = profile.digest()?;
        Ok(Self {
            graph,
            profile,
            graph_sha256,
            profile_sha256,
        })
    }

    #[must_use]
    pub fn graph(&self) -> &OrchestrationTaskGraphV1 {
        &self.graph
    }

    #[must_use]
    pub fn profile(&self) -> &OrchestrationProfileV1 {
        &self.profile
    }

    pub fn new_checkpoint(
        &self,
        run_id: RunId,
        project_id: ProjectId,
        expected_project_revision: u64,
    ) -> Result<OrchestrationCheckpointV1, OrchestrationError> {
        if expected_project_revision == 0
            || expected_project_revision > MAX_SAFE_INTEGER
            || RunId::parse(run_id.as_str()).is_err()
            || ProjectId::parse(project_id.as_str()).is_err()
        {
            return Err(OrchestrationError::InvalidCheckpoint);
        }
        let checkpoint = OrchestrationCheckpointV1 {
            schema_version: ORCHESTRATION_SCHEMA_VERSION,
            run_id,
            project_id,
            expected_project_revision,
            graph_sha256: self.graph_sha256.clone(),
            profile_sha256: self.profile_sha256.clone(),
            generation: 0,
            status: OrchestrationRunStatus::Planned,
            tasks: self
                .graph
                .tasks
                .iter()
                .map(|task| TaskCheckpointV1 {
                    task_id: task.task_id.clone(),
                    state: OrchestrationTaskState::Pending,
                    attempts: 0,
                    active_role: None,
                    role_outputs: Vec::new(),
                    output_sha256: None,
                    failure_code: None,
                })
                .collect(),
        };
        self.validate_checkpoint(&checkpoint)?;
        Ok(checkpoint)
    }

    pub fn restore_checkpoint(
        &self,
        input: &[u8],
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<OrchestrationCheckpointV1, OrchestrationError> {
        if input.len() > MAX_CHECKPOINT_BYTES {
            return Err(OrchestrationError::InputTooLarge);
        }
        let checkpoint = serde_json::from_slice::<OrchestrationCheckpointV1>(input)
            .map_err(|_| OrchestrationError::InvalidJson)?;
        if checkpoint.project_id != *project_id
            || checkpoint.expected_project_revision != expected_project_revision
            || checkpoint.graph_sha256 != self.graph_sha256
            || checkpoint.profile_sha256 != self.profile_sha256
        {
            return Err(OrchestrationError::BindingMismatch);
        }
        self.validate_checkpoint(&checkpoint)?;
        if checkpoint.to_canonical_json(self)? != input {
            return Err(OrchestrationError::NonCanonicalJson);
        }
        Ok(checkpoint)
    }

    fn validate_checkpoint(
        &self,
        checkpoint: &OrchestrationCheckpointV1,
    ) -> Result<(), OrchestrationError> {
        if checkpoint.schema_version != ORCHESTRATION_SCHEMA_VERSION
            || checkpoint.expected_project_revision == 0
            || checkpoint.expected_project_revision > MAX_SAFE_INTEGER
            || checkpoint.generation > MAX_SAFE_INTEGER
            || checkpoint.graph_sha256 != self.graph_sha256
            || checkpoint.profile_sha256 != self.profile_sha256
            || RunId::parse(checkpoint.run_id.as_str()).is_err()
            || ProjectId::parse(checkpoint.project_id.as_str()).is_err()
            || checkpoint.tasks.len() != self.graph.tasks.len()
            || checkpoint
                .tasks
                .iter()
                .zip(&self.graph.tasks)
                .any(|(state, spec)| state.task_id != spec.task_id)
        {
            return Err(OrchestrationError::InvalidCheckpoint);
        }
        let running_count = checkpoint
            .tasks
            .iter()
            .filter(|task| task.state == OrchestrationTaskState::Running)
            .count();
        if running_count > 1 {
            return Err(OrchestrationError::InvalidCheckpoint);
        }
        if checkpoint.status != OrchestrationRunStatus::Cancelled
            && checkpoint
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Cancelled)
        {
            return Err(OrchestrationError::InvalidCheckpoint);
        }
        for task in &checkpoint.tasks {
            if task.attempts > self.profile.max_task_attempts
                || !valid_task_checkpoint(task, &self.profile)
                || task.state == OrchestrationTaskState::Ready
                    && task.attempts >= self.profile.max_task_attempts
            {
                return Err(OrchestrationError::InvalidCheckpoint);
            }
            if matches!(
                task.state,
                OrchestrationTaskState::Completed | OrchestrationTaskState::Running
            ) && !self.dependencies_satisfied(checkpoint, &task.task_id)?
            {
                return Err(OrchestrationError::InvalidCheckpoint);
            }
        }
        if checkpoint.status != OrchestrationRunStatus::Planned {
            self.validate_dependency_states(checkpoint)?;
        }
        let status_valid = match checkpoint.status {
            OrchestrationRunStatus::Planned => {
                checkpoint.generation == 0
                    && checkpoint.tasks.iter().all(|task| {
                        task.state == OrchestrationTaskState::Pending && task.attempts == 0
                    })
            }
            OrchestrationRunStatus::Running => checkpoint
                .tasks
                .iter()
                .any(|task| !task.state.is_terminal()),
            OrchestrationRunStatus::Paused => running_count == 0,
            OrchestrationRunStatus::Completed => checkpoint
                .tasks
                .iter()
                .all(|task| task.state == OrchestrationTaskState::Completed),
            OrchestrationRunStatus::Failed => {
                checkpoint.tasks.iter().all(|task| {
                    matches!(
                        task.state,
                        OrchestrationTaskState::Completed
                            | OrchestrationTaskState::Failed
                            | OrchestrationTaskState::Blocked
                    )
                }) && checkpoint.tasks.iter().any(|task| {
                    matches!(
                        task.state,
                        OrchestrationTaskState::Failed | OrchestrationTaskState::Blocked
                    )
                })
            }
            OrchestrationRunStatus::Cancelled => {
                checkpoint.tasks.iter().all(|task| task.state.is_terminal())
                    && checkpoint
                        .tasks
                        .iter()
                        .any(|task| task.state == OrchestrationTaskState::Cancelled)
            }
        };
        status_valid
            .then_some(())
            .ok_or(OrchestrationError::InvalidCheckpoint)
    }

    fn validate_dependency_states(
        &self,
        checkpoint: &OrchestrationCheckpointV1,
    ) -> Result<(), OrchestrationError> {
        let states = checkpoint
            .tasks
            .iter()
            .map(|task| (task.task_id.clone(), task.state))
            .collect::<BTreeMap<_, _>>();
        let stopped_after_failure = self.profile.stop_on_failure
            && checkpoint
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Failed);
        for task in &checkpoint.tasks {
            let spec = self
                .graph
                .task(&task.task_id)
                .ok_or(OrchestrationError::InvalidCheckpoint)?;
            let dependency = dependency_state(spec, &states)?;
            let valid = match task.state {
                OrchestrationTaskState::Pending => dependency == DependencyState::Waiting,
                OrchestrationTaskState::Ready => dependency == DependencyState::Ready,
                OrchestrationTaskState::Blocked => match task.failure_code {
                    Some(OrchestrationFailureCode::PrerequisiteUnavailable) => {
                        dependency == DependencyState::Impossible
                    }
                    Some(OrchestrationFailureCode::StoppedAfterFailure) => stopped_after_failure,
                    _ => false,
                },
                _ => true,
            };
            if !valid {
                return Err(OrchestrationError::InvalidCheckpoint);
            }
        }
        Ok(())
    }

    fn recompute(
        &self,
        checkpoint: &mut OrchestrationCheckpointV1,
    ) -> Result<(), OrchestrationError> {
        self.recompute_dependencies(checkpoint)?;
        if checkpoint.status != OrchestrationRunStatus::Running {
            return Ok(());
        }
        if self.profile.stop_on_failure
            && checkpoint
                .tasks
                .iter()
                .any(|task| task.state == OrchestrationTaskState::Failed)
        {
            for task in &mut checkpoint.tasks {
                if matches!(
                    task.state,
                    OrchestrationTaskState::Pending | OrchestrationTaskState::Ready
                ) {
                    task.state = OrchestrationTaskState::Blocked;
                    task.failure_code = Some(OrchestrationFailureCode::StoppedAfterFailure);
                }
            }
        }
        if checkpoint
            .tasks
            .iter()
            .all(|task| matches!(task.state, OrchestrationTaskState::Completed))
        {
            checkpoint.status = OrchestrationRunStatus::Completed;
        } else if checkpoint.tasks.iter().all(|task| task.state.is_terminal()) {
            checkpoint.status = OrchestrationRunStatus::Failed;
        }
        Ok(())
    }

    fn recompute_dependencies(
        &self,
        checkpoint: &mut OrchestrationCheckpointV1,
    ) -> Result<(), OrchestrationError> {
        loop {
            let states = checkpoint
                .tasks
                .iter()
                .map(|task| (task.task_id.clone(), task.state))
                .collect::<BTreeMap<_, _>>();
            let mut changed = false;
            for task in &mut checkpoint.tasks {
                if !matches!(
                    task.state,
                    OrchestrationTaskState::Pending | OrchestrationTaskState::Ready
                ) {
                    continue;
                }
                let spec = self
                    .graph
                    .task(&task.task_id)
                    .ok_or(OrchestrationError::InvalidCheckpoint)?;
                let dependency = dependency_state(spec, &states)?;
                let next = match dependency {
                    DependencyState::Waiting => OrchestrationTaskState::Pending,
                    DependencyState::Ready => OrchestrationTaskState::Ready,
                    DependencyState::Impossible => OrchestrationTaskState::Blocked,
                };
                if task.state != next {
                    task.state = next;
                    changed = true;
                }
                if next == OrchestrationTaskState::Blocked {
                    task.failure_code = Some(OrchestrationFailureCode::PrerequisiteUnavailable);
                }
            }
            if !changed {
                return Ok(());
            }
        }
    }

    fn dependencies_satisfied(
        &self,
        checkpoint: &OrchestrationCheckpointV1,
        task_id: &OrchestrationTaskId,
    ) -> Result<bool, OrchestrationError> {
        let spec = self
            .graph
            .task(task_id)
            .ok_or(OrchestrationError::InvalidCheckpoint)?;
        let states = checkpoint
            .tasks
            .iter()
            .map(|task| (task.task_id.clone(), task.state))
            .collect::<BTreeMap<_, _>>();
        Ok(dependency_state(spec, &states)? == DependencyState::Ready)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DependencyState {
    Waiting,
    Ready,
    Impossible,
}

fn dependency_state(
    task: &OrchestrationTaskSpecV1,
    states: &BTreeMap<OrchestrationTaskId, OrchestrationTaskState>,
) -> Result<DependencyState, OrchestrationError> {
    let all_states = task
        .prerequisites_all
        .iter()
        .map(|dependency| {
            states
                .get(dependency)
                .copied()
                .ok_or(OrchestrationError::InvalidCheckpoint)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if all_states
        .iter()
        .any(|state| dependency_terminal_without_output(*state))
    {
        return Ok(DependencyState::Impossible);
    }
    if all_states
        .iter()
        .any(|state| *state != OrchestrationTaskState::Completed)
    {
        return Ok(DependencyState::Waiting);
    }
    if task.prerequisites_any.is_empty() {
        return Ok(DependencyState::Ready);
    }
    let any_states = task
        .prerequisites_any
        .iter()
        .map(|dependency| {
            states
                .get(dependency)
                .copied()
                .ok_or(OrchestrationError::InvalidCheckpoint)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if any_states.contains(&OrchestrationTaskState::Completed) {
        Ok(DependencyState::Ready)
    } else if any_states
        .iter()
        .all(|state| dependency_terminal_without_output(*state))
    {
        Ok(DependencyState::Impossible)
    } else {
        Ok(DependencyState::Waiting)
    }
}

fn dependency_terminal_without_output(state: OrchestrationTaskState) -> bool {
    matches!(
        state,
        OrchestrationTaskState::Failed
            | OrchestrationTaskState::Blocked
            | OrchestrationTaskState::Cancelled
    )
}

fn valid_task_checkpoint(task: &TaskCheckpointV1, profile: &OrchestrationProfileV1) -> bool {
    if OrchestrationTaskId::parse(task.task_id.as_str()).is_err()
        || task
            .output_sha256
            .as_ref()
            .is_some_and(|digest| !valid_sha256(digest))
    {
        return false;
    }
    let roles = profile.roles();
    if task.role_outputs.len() > roles.len()
        || task
            .role_outputs
            .iter()
            .zip(roles)
            .any(|(output, expected_role)| {
                output.role != *expected_role
                    || profile.backend_for_role(output.role) != Some(&output.backend_id)
                    || !valid_sha256(&output.output_sha256)
            })
    {
        return false;
    }
    match task.state {
        OrchestrationTaskState::Pending => {
            task.attempts == 0
                && task.active_role.is_none()
                && task.role_outputs.is_empty()
                && task.output_sha256.is_none()
                && task.failure_code.is_none()
        }
        OrchestrationTaskState::Ready => {
            task.active_role.is_none()
                && task.role_outputs.is_empty()
                && task.output_sha256.is_none()
        }
        OrchestrationTaskState::Running => {
            task.attempts > 0
                && task.role_outputs.len() < roles.len()
                && task.active_role == roles.get(task.role_outputs.len()).copied()
                && task.output_sha256.is_none()
                && task.failure_code.is_none()
        }
        OrchestrationTaskState::Completed => {
            task.attempts > 0
                && task.active_role.is_none()
                && task.role_outputs.len() == roles.len()
                && task.output_sha256.as_deref()
                    == task
                        .role_outputs
                        .last()
                        .map(|output| output.output_sha256.as_str())
                && task.failure_code.is_none()
        }
        OrchestrationTaskState::Failed => {
            task.attempts > 0
                && task.active_role.is_none()
                && task.role_outputs.is_empty()
                && task.output_sha256.is_none()
                && task.failure_code.is_some()
        }
        OrchestrationTaskState::Blocked => {
            task.active_role.is_none()
                && task.role_outputs.is_empty()
                && task.output_sha256.is_none()
                && task.failure_code.is_some()
        }
        OrchestrationTaskState::Cancelled => {
            task.active_role.is_none()
                && task.role_outputs.is_empty()
                && task.output_sha256.is_none()
                && task.failure_code.is_some()
        }
    }
}

fn parse_dependencies(
    dependencies: impl IntoIterator<Item = impl Into<String>>,
) -> Result<Vec<OrchestrationTaskId>, OrchestrationError> {
    let mut parsed = dependencies
        .into_iter()
        .map(|value| {
            OrchestrationTaskId::parse(value.into()).map_err(|_| OrchestrationError::InvalidGraph)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let input_len = parsed.len();
    parsed.sort();
    parsed.dedup();
    if parsed.len() != input_len || parsed.len() > MAX_DEPENDENCIES_PER_TASK {
        return Err(OrchestrationError::InvalidGraph);
    }
    Ok(parsed)
}

fn strictly_sorted_unique(values: &[OrchestrationTaskId]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn canonical_json(value: &impl Serialize, maximum: usize) -> Result<Vec<u8>, OrchestrationError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| OrchestrationError::SerializationFailed)?;
    if bytes.len() > maximum {
        return Err(OrchestrationError::InputTooLarge);
    }
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use qiongli_content::{
        QIONGLI_CORE_RESOURCE_PACK_LOCK_V1, ResourcePackLockV1, build_resource_pack,
        collect_canonical_sources,
    };
    use serde_json::json;

    use super::*;

    fn task(
        id: &str,
        all: &[&str],
        any: &[&str],
    ) -> Result<OrchestrationTaskSpecV1, OrchestrationError> {
        OrchestrationTaskSpecV1::try_new(id, all.iter().copied(), any.iter().copied())
    }

    fn graph() -> OrchestrationTaskGraphV1 {
        OrchestrationTaskGraphV1::try_new(vec![
            task("A1", &[], &[]).unwrap(),
            task("B1", &["A1"], &[]).unwrap(),
            task("B2", &["A1"], &[]).unwrap(),
            task("F3", &[], &["B1", "B2"]).unwrap(),
        ])
        .unwrap()
    }

    fn profile(stop_on_failure: bool, attempts: u8) -> OrchestrationProfileV1 {
        let backend = BackendId::parse("openai-direct").unwrap();
        OrchestrationProfileV1::try_new(
            "default",
            OrchestrationExecutionMode::Duo,
            backend.clone(),
            Some(backend),
            None,
            attempts,
            stop_on_failure,
        )
        .unwrap()
    }

    fn plan(stop_on_failure: bool, attempts: u8) -> OrchestrationPlanV1 {
        OrchestrationPlanV1::try_new(graph(), profile(stop_on_failure, attempts)).unwrap()
    }

    fn run_id() -> RunId {
        RunId::parse(format!("run_{}", "a".repeat(32))).unwrap()
    }

    fn project_id() -> ProjectId {
        ProjectId::parse(format!("prj_{}", "b".repeat(32))).unwrap()
    }

    fn digest(byte: u8) -> String {
        format!("{byte:02x}").repeat(32)
    }

    fn task_id(value: &str) -> OrchestrationTaskId {
        OrchestrationTaskId::parse(value).unwrap()
    }

    fn complete_active_task(
        checkpoint: &mut OrchestrationCheckpointV1,
        plan: &OrchestrationPlanV1,
        task_id: &OrchestrationTaskId,
        output_sha256: &str,
    ) {
        while checkpoint
            .tasks
            .iter()
            .find(|task| &task.task_id == task_id)
            .is_some_and(|task| task.state == OrchestrationTaskState::Running)
        {
            checkpoint
                .complete_role(plan, checkpoint.generation, task_id, output_sha256)
                .unwrap();
        }
    }

    fn repository_embedded_content() -> EmbeddedContent {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let content_root = crate_root.join("../../../../content");
        let lock = ResourcePackLockV1::from_json(QIONGLI_CORE_RESOURCE_PACK_LOCK_V1).unwrap();
        let resources = collect_canonical_sources(content_root).unwrap();
        let built = build_resource_pack(&lock.metadata().unwrap(), &resources).unwrap();
        lock.verify(&built).unwrap();
        let digest = built.pack_sha256().to_owned();
        let bytes = Box::leak(built.into_core_bytes());
        EmbeddedContent::load(bytes, &digest).unwrap()
    }

    #[test]
    fn graph_is_bounded_closed_and_acyclic() {
        let canonical = graph().to_canonical_json().unwrap();
        assert_eq!(
            OrchestrationTaskGraphV1::from_canonical_json(&canonical).unwrap(),
            graph()
        );
        assert_eq!(
            OrchestrationTaskGraphV1::from_canonical_json(
                &serde_json::to_vec_pretty(&graph()).unwrap()
            )
            .unwrap_err(),
            OrchestrationError::NonCanonicalJson
        );
        assert_eq!(
            OrchestrationTaskGraphV1::try_new(vec![
                task("A1", &["B1"], &[]).unwrap(),
                task("B1", &["A1"], &[]).unwrap(),
            ])
            .unwrap_err(),
            OrchestrationError::GraphCycle
        );
        assert_eq!(
            OrchestrationTaskGraphV1::try_new(vec![task("A1", &["missing"], &[]).unwrap()])
                .unwrap_err(),
            OrchestrationError::InvalidGraph
        );
        assert_eq!(
            OrchestrationTaskGraphV1::try_new(vec![
                task("A1", &[], &[]).unwrap(),
                task("A1", &[], &[]).unwrap(),
            ])
            .unwrap_err(),
            OrchestrationError::InvalidGraph
        );
        assert_eq!(
            OrchestrationTaskSpecV1::try_new("B1", ["A1", "A1"], [] as [&str; 0]).unwrap_err(),
            OrchestrationError::InvalidGraph
        );
    }

    #[test]
    fn embedded_projection_binds_all_frozen_tasks_to_the_verified_full_content() {
        let projection = embedded_workflow_projection().unwrap();
        assert_eq!(projection.source_path, ORCHESTRATION_WORKFLOW_SOURCE_PATH);
        assert_eq!(projection.graph.tasks.len(), 76);
        assert_eq!(projection.graph.tasks[0].task_id.as_str(), "A1");
        assert_eq!(projection.graph.tasks[75].task_id.as_str(), "M7");

        let content = repository_embedded_content();
        let graph = OrchestrationTaskGraphV1::from_embedded_content(&content).unwrap();
        assert_eq!(graph, projection.graph);
    }

    #[test]
    fn embedded_projection_matches_the_checked_ctr_201_inventory() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let inventory_path =
            crate_root.join("../../../../tooling/migration/ctr-201-orchestrator.json");
        let inventory: serde_json::Value =
            serde_json::from_slice(&fs::read(inventory_path).unwrap()).unwrap();
        let source_sha256 = inventory["source"]["blob_anchors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|anchor| anchor["role"] == "workflow-contract")
            .and_then(|anchor| anchor["sha256"].as_str())
            .unwrap();
        let tasks = inventory["workflow"]["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|task| {
                let strings = |field: &str| {
                    task["dependencies"][field]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_str().unwrap().to_owned())
                        .collect::<Vec<_>>()
                };
                OrchestrationTaskSpecV1::try_new(
                    task["task_id"].as_str().unwrap(),
                    strings("prerequisites_all"),
                    strings("prerequisites_any"),
                )
                .unwrap()
            })
            .collect();
        let projection = embedded_workflow_projection().unwrap();
        assert_eq!(projection.source_sha256, source_sha256);
        assert_eq!(
            projection.graph,
            OrchestrationTaskGraphV1::try_new(tasks).unwrap()
        );
    }

    #[test]
    fn profile_requires_exact_role_shape_and_canonical_round_trip() {
        let profile = profile(true, 2);
        let canonical = profile.to_canonical_json().unwrap();
        assert_eq!(
            OrchestrationProfileV1::from_canonical_json(&canonical).unwrap(),
            profile
        );
        let backend = BackendId::parse("openai-direct").unwrap();
        assert_eq!(
            OrchestrationProfileV1::try_new(
                "solo",
                OrchestrationExecutionMode::Solo,
                backend.clone(),
                Some(backend),
                None,
                1,
                true,
            )
            .unwrap_err(),
            OrchestrationError::InvalidProfile
        );
        assert_eq!(
            OrchestrationProfileV1::try_new(
                "default",
                OrchestrationExecutionMode::Duo,
                BackendId::parse("openai-direct").unwrap(),
                Some(BackendId::parse("openai-direct").unwrap()),
                None,
                MAX_TASK_ATTEMPTS + 1,
                true,
            )
            .unwrap_err(),
            OrchestrationError::InvalidProfile
        );
        let backend = BackendId::parse("openai-direct").unwrap();
        let triad = OrchestrationProfileV1::try_new(
            "strict-review",
            OrchestrationExecutionMode::Triad,
            backend.clone(),
            Some(backend.clone()),
            Some(backend),
            1,
            true,
        )
        .unwrap();
        assert_eq!(
            triad.roles(),
            &[
                OrchestrationRole::Primary,
                OrchestrationRole::Reviewer,
                OrchestrationRole::Verifier,
            ]
        );
    }

    #[test]
    fn duo_role_progress_is_checkpointed_without_model_text() {
        let plan = plan(false, 1);
        let mut checkpoint = plan.new_checkpoint(run_id(), project_id(), 4).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        checkpoint.begin_task(&plan, 1, &task_id("A1")).unwrap();
        assert_eq!(
            checkpoint.tasks[0].active_role,
            Some(OrchestrationRole::Primary)
        );

        checkpoint
            .complete_role(&plan, 2, &task_id("A1"), digest(1))
            .unwrap();
        assert_eq!(checkpoint.tasks[0].state, OrchestrationTaskState::Running);
        assert_eq!(
            checkpoint.tasks[0].active_role,
            Some(OrchestrationRole::Reviewer)
        );
        assert_eq!(checkpoint.tasks[0].role_outputs.len(), 1);
        assert!(checkpoint.tasks[0].output_sha256.is_none());

        checkpoint
            .complete_role(&plan, 3, &task_id("A1"), digest(2))
            .unwrap();
        assert_eq!(checkpoint.tasks[0].state, OrchestrationTaskState::Completed);
        assert_eq!(checkpoint.tasks[0].active_role, None);
        assert_eq!(checkpoint.tasks[0].role_outputs.len(), 2);
        assert_eq!(checkpoint.tasks[0].output_sha256, Some(digest(2)));
        let serialized = String::from_utf8(checkpoint.to_canonical_json(&plan).unwrap()).unwrap();
        assert!(!serialized.contains("model text"));
    }

    #[test]
    fn task_graph_progresses_in_declaration_order_with_all_and_any_dependencies() {
        let plan = plan(false, 1);
        let mut checkpoint = plan.new_checkpoint(run_id(), project_id(), 7).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        assert_eq!(checkpoint.next_ready_task(), Some(&task_id("A1")));

        checkpoint.begin_task(&plan, 1, &task_id("A1")).unwrap();
        complete_active_task(&mut checkpoint, &plan, &task_id("A1"), &digest(1));
        assert_eq!(checkpoint.next_ready_task(), Some(&task_id("B1")));

        checkpoint
            .begin_task(&plan, checkpoint.generation, &task_id("B1"))
            .unwrap();
        complete_active_task(&mut checkpoint, &plan, &task_id("B1"), &digest(2));
        assert_eq!(checkpoint.next_ready_task(), Some(&task_id("B2")));

        checkpoint
            .begin_task(&plan, checkpoint.generation, &task_id("B2"))
            .unwrap();
        checkpoint
            .fail_task(
                &plan,
                checkpoint.generation,
                &task_id("B2"),
                OrchestrationFailureCode::BackendFailed,
                false,
            )
            .unwrap();
        assert_eq!(checkpoint.next_ready_task(), Some(&task_id("F3")));

        checkpoint
            .begin_task(&plan, checkpoint.generation, &task_id("F3"))
            .unwrap();
        complete_active_task(&mut checkpoint, &plan, &task_id("F3"), &digest(3));
        assert_eq!(checkpoint.status, OrchestrationRunStatus::Failed);
        assert_eq!(checkpoint.generation, 12);
    }

    #[test]
    fn retry_is_bounded_and_optimistic_generation_rejects_stale_updates() {
        let plan = plan(false, 2);
        let mut checkpoint = plan.new_checkpoint(run_id(), project_id(), 1).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        checkpoint.begin_task(&plan, 1, &task_id("A1")).unwrap();
        assert_eq!(
            checkpoint
                .fail_task(
                    &plan,
                    1,
                    &task_id("A1"),
                    OrchestrationFailureCode::BackendUnavailable,
                    true,
                )
                .unwrap_err(),
            OrchestrationError::StaleGeneration
        );
        checkpoint
            .fail_task(
                &plan,
                2,
                &task_id("A1"),
                OrchestrationFailureCode::BackendUnavailable,
                true,
            )
            .unwrap();
        assert_eq!(checkpoint.next_ready_task(), Some(&task_id("A1")));
        checkpoint.begin_task(&plan, 3, &task_id("A1")).unwrap();
        checkpoint
            .fail_task(
                &plan,
                4,
                &task_id("A1"),
                OrchestrationFailureCode::BackendUnavailable,
                true,
            )
            .unwrap();
        assert_eq!(checkpoint.tasks[0].state, OrchestrationTaskState::Failed);

        let mut exhausted = plan.new_checkpoint(run_id(), project_id(), 1).unwrap();
        exhausted.start(&plan, 0).unwrap();
        exhausted.generation = MAX_SAFE_INTEGER;
        let before = exhausted.clone();
        assert_eq!(
            exhausted
                .begin_task(&plan, MAX_SAFE_INTEGER, &task_id("A1"))
                .unwrap_err(),
            OrchestrationError::LimitExceeded
        );
        assert_eq!(exhausted, before);
    }

    #[test]
    fn stop_on_failure_blocks_remaining_work() {
        let plan = plan(true, 1);
        let mut checkpoint = plan.new_checkpoint(run_id(), project_id(), 1).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        checkpoint.begin_task(&plan, 1, &task_id("A1")).unwrap();
        checkpoint
            .fail_task(
                &plan,
                2,
                &task_id("A1"),
                OrchestrationFailureCode::BackendFailed,
                false,
            )
            .unwrap();
        assert_eq!(checkpoint.status, OrchestrationRunStatus::Failed);
        assert!(checkpoint.tasks[1..].iter().all(|task| {
            task.state == OrchestrationTaskState::Blocked
                && matches!(
                    task.failure_code,
                    Some(
                        OrchestrationFailureCode::PrerequisiteUnavailable
                            | OrchestrationFailureCode::StoppedAfterFailure
                    )
                )
        }));
    }

    #[test]
    fn interrupted_task_can_be_recovered_paused_and_resumed() {
        let plan = plan(false, 2);
        let mut checkpoint = plan.new_checkpoint(run_id(), project_id(), 3).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        checkpoint.begin_task(&plan, 1, &task_id("A1")).unwrap();
        checkpoint.recover_interrupted(&plan, 2).unwrap();
        assert_eq!(checkpoint.status, OrchestrationRunStatus::Paused);
        assert_eq!(checkpoint.tasks[0].state, OrchestrationTaskState::Ready);
        assert_eq!(
            checkpoint.tasks[0].failure_code,
            Some(OrchestrationFailureCode::TaskInterrupted)
        );
        checkpoint.resume(&plan, 3).unwrap();
        assert_eq!(checkpoint.status, OrchestrationRunStatus::Running);
        checkpoint.begin_task(&plan, 4, &task_id("A1")).unwrap();
    }

    #[test]
    fn canonical_checkpoint_resume_requires_exact_project_graph_and_profile_binding() {
        let plan = plan(false, 2);
        let project = project_id();
        let mut checkpoint = plan.new_checkpoint(run_id(), project.clone(), 11).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        checkpoint.pause(&plan, 1).unwrap();
        let canonical = checkpoint.to_canonical_json(&plan).unwrap();
        assert_eq!(
            plan.restore_checkpoint(&canonical, &project, 11).unwrap(),
            checkpoint
        );
        assert_eq!(
            plan.restore_checkpoint(&canonical, &project, 12)
                .unwrap_err(),
            OrchestrationError::BindingMismatch
        );
        let other_plan = OrchestrationPlanV1::try_new(graph(), profile(true, 2)).unwrap();
        assert_eq!(
            other_plan
                .restore_checkpoint(&canonical, &project, 11)
                .unwrap_err(),
            OrchestrationError::BindingMismatch
        );
    }

    #[test]
    fn checkpoint_rejects_unknown_fields_open_reason_values_and_invalid_state() {
        let plan = plan(false, 2);
        let checkpoint = plan.new_checkpoint(run_id(), project_id(), 1).unwrap();
        let mut value = serde_json::to_value(&checkpoint).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("private-canary".to_owned(), json!(true));
        assert_eq!(
            plan.restore_checkpoint(
                &serde_json_canonicalizer::to_vec(&value).unwrap(),
                &project_id(),
                1
            )
            .unwrap_err(),
            OrchestrationError::InvalidJson
        );

        let mut open_reason = serde_json::to_value(&checkpoint).unwrap();
        open_reason["tasks"][0]["failureCode"] = json!("private-reason");
        assert_eq!(
            plan.restore_checkpoint(
                &serde_json_canonicalizer::to_vec(&open_reason).unwrap(),
                &project_id(),
                1,
            )
            .unwrap_err(),
            OrchestrationError::InvalidJson
        );
        let mut impossible = plan.new_checkpoint(run_id(), project_id(), 1).unwrap();
        impossible.start(&plan, 0).unwrap();
        impossible.tasks[0].state = OrchestrationTaskState::Pending;
        assert_eq!(
            impossible.to_canonical_json(&plan).unwrap_err(),
            OrchestrationError::InvalidCheckpoint
        );
        let mut impossible_cancel = plan.new_checkpoint(run_id(), project_id(), 1).unwrap();
        impossible_cancel.start(&plan, 0).unwrap();
        impossible_cancel.tasks[0].state = OrchestrationTaskState::Cancelled;
        impossible_cancel.tasks[0].failure_code = Some(OrchestrationFailureCode::RunCancelled);
        assert_eq!(
            impossible_cancel.to_canonical_json(&plan).unwrap_err(),
            OrchestrationError::InvalidCheckpoint
        );
        for error in [
            OrchestrationError::InvalidJson,
            OrchestrationError::InvalidCheckpoint,
            OrchestrationError::BindingMismatch,
        ] {
            assert!(!error.to_string().contains("private"));
        }
    }

    #[test]
    fn cancellation_is_terminal_and_preserves_completed_output_digests() {
        let plan = plan(false, 1);
        let mut checkpoint = plan.new_checkpoint(run_id(), project_id(), 2).unwrap();
        checkpoint.start(&plan, 0).unwrap();
        checkpoint.begin_task(&plan, 1, &task_id("A1")).unwrap();
        complete_active_task(&mut checkpoint, &plan, &task_id("A1"), &digest(9));
        checkpoint.cancel(&plan, checkpoint.generation).unwrap();
        assert_eq!(checkpoint.status, OrchestrationRunStatus::Cancelled);
        assert_eq!(checkpoint.tasks[0].output_sha256, Some(digest(9)));
        assert_eq!(checkpoint.tasks[0].role_outputs.len(), 2);
        assert!(checkpoint.tasks[1..].iter().all(|task| {
            task.state == OrchestrationTaskState::Cancelled
                && task.failure_code == Some(OrchestrationFailureCode::RunCancelled)
        }));
        assert_eq!(
            checkpoint.cancel(&plan, checkpoint.generation).unwrap_err(),
            OrchestrationError::InvalidTransition
        );
    }
}
