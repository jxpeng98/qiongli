use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use qiongli_config::{GlobalSettings, SecretStore};
use qiongli_content::EmbeddedContent;
use qiongli_execution::{
    AgentBackend, AgentExecutionPolicy, BackendId, BackendReadinessV1, BoundedAgentRunner,
    CancellationToken, EmbeddedWorkerOrchestrationInputBuilder, EmbeddedWorkflowRoleInputBuilder,
    ExecutionProfile, InProcessToolHost, OpenAiBackendConfigV1, OpenAiResponsesBackend,
    OrchestrationCheckpointStore, OrchestrationExecutionMode, OrchestrationPlanV1,
    OrchestrationProfileV1, OrchestrationRole, OrchestrationRunStatus, OrchestrationStepOutcome,
    OrchestrationTaskExecutor, OrchestrationTaskGraphV1, OrchestrationTaskId,
    OrchestrationTaskState, ProjectExecutionScope, RedactionPolicyV1, RunId,
    WorkerBarrierFailurePolicy, WorkerBarrierStatus, WorkerMergePolicy,
    WorkerOrchestrationAgentPhase, WorkerOrchestrationCheckpointStore, WorkerOrchestrationExecutor,
    WorkerOrchestrationMode, WorkerOrchestrationPlanV1, WorkerOrchestrationRunStatus,
    WorkerOrchestrationStepOutcome, WorkerSpecV1, WorkerStatus, openai_backend_status,
};
use qiongli_project::{ProjectId, ProjectStateService};
use qiongli_runtime::{FullProjectService, FullProjectToolRegistry};
use serde::Serialize;

use crate::agent_run::{
    block_on, execution_limits, new_run_id, project_scoped_read_tools, readiness_reason_code,
};

const ORCHESTRATION_VIEW_SCHEMA_VERSION: u32 = 1;
const POLICY_REVISION: u64 = 1;
const MAX_TASK_ATTEMPTS: u8 = 2;
const OPENAI_BACKEND_ID: &str = "openai-responses";
const OPENAI_MODEL: &str = "gpt-5.6-sol";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FullOrchestrationError {
    reason_code: &'static str,
}

impl FullOrchestrationError {
    pub(crate) const fn new(reason_code: &'static str) -> Self {
        Self { reason_code }
    }

    pub(crate) const fn reason_code(self) -> &'static str {
        self.reason_code
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrchestrationDoctorViewV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub workflow_contract_status: &'static str,
    pub backend_readiness: BackendReadinessV1,
    pub run_count: usize,
    pub active_run_count: usize,
    pub recovery_required_count: usize,
    pub runnable: bool,
    pub reason_codes: Vec<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrchestrationRunListViewV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub runs: Vec<OrchestrationRunSummaryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrchestrationRunSummaryV1 {
    pub run_id: RunId,
    pub profile_id: String,
    pub execution_mode: OrchestrationExecutionMode,
    pub status: OrchestrationRunStatus,
    pub generation: u64,
    pub document_sha256: String,
    pub completed_task_count: usize,
    pub total_task_count: usize,
    pub next_task_id: Option<String>,
    pub active_task_id: Option<String>,
    pub recovery_required: bool,
    pub can_continue: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_recover: bool,
    pub can_cancel: bool,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrchestrationRoleOutputViewV1 {
    pub task_id: String,
    pub role: OrchestrationRole,
    pub output_sha256: String,
    pub model: String,
    pub finish_reason: qiongli_execution::AgentFinishReason,
    pub content: String,
    pub model_turns: u32,
    pub tool_calls: u32,
    pub network_requests: u32,
}

impl Debug for OrchestrationRoleOutputViewV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OrchestrationRoleOutputViewV1")
            .field("task_id", &self.task_id)
            .field("role", &self.role)
            .field("output_sha256", &self.output_sha256)
            .field("model", &self.model)
            .field("finish_reason", &self.finish_reason)
            .field("content", &"<private-orchestration-role-output>")
            .field("model_turns", &self.model_turns)
            .field("tool_calls", &self.tool_calls)
            .field("network_requests", &self.network_requests)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrchestrationExecutionViewV1 {
    pub schema_version: u32,
    pub outcome: &'static str,
    pub task_id: Option<String>,
    pub run: OrchestrationRunSummaryV1,
    pub role_outputs: Vec<OrchestrationRoleOutputViewV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerOrchestrationRunListViewV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub runs: Vec<WorkerOrchestrationRunSummaryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerOrchestrationRunSummaryV1 {
    pub run_id: RunId,
    pub task_id: String,
    pub mode: WorkerOrchestrationMode,
    pub status: WorkerOrchestrationRunStatus,
    pub barrier_status: Option<WorkerBarrierStatus>,
    pub generation: u64,
    pub document_sha256: String,
    pub passed_worker_count: usize,
    pub total_worker_count: usize,
    pub recovery_required: bool,
    pub can_continue: bool,
    pub can_recover: bool,
    pub can_cancel: bool,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerOrchestrationAgentOutputViewV1 {
    pub phase: WorkerOrchestrationAgentPhase,
    pub worker_id: Option<String>,
    pub output_sha256: String,
    pub model: String,
    pub finish_reason: qiongli_execution::AgentFinishReason,
    pub content: String,
    pub model_turns: u32,
    pub tool_calls: u32,
    pub network_requests: u32,
}

impl Debug for WorkerOrchestrationAgentOutputViewV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerOrchestrationAgentOutputViewV1")
            .field("phase", &self.phase)
            .field("worker_id", &self.worker_id)
            .field("output_sha256", &self.output_sha256)
            .field("model", &self.model)
            .field("finish_reason", &self.finish_reason)
            .field("content", &"<private-worker-orchestration-output>")
            .field("model_turns", &self.model_turns)
            .field("tool_calls", &self.tool_calls)
            .field("network_requests", &self.network_requests)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerOrchestrationExecutionViewV1 {
    pub schema_version: u32,
    pub outcome: &'static str,
    pub run: WorkerOrchestrationRunSummaryV1,
    pub agent_outputs: Vec<WorkerOrchestrationAgentOutputViewV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrchestrationControlAction {
    Pause,
    Recover,
    Resume,
    Cancel,
}

#[derive(Clone)]
pub(crate) struct OrchestrationRunReference {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub run_id: RunId,
    pub expected_generation: u64,
    pub expected_document_sha256: String,
}

#[derive(Clone)]
pub(crate) struct WorkerOrchestrationRunReference {
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub run_id: RunId,
    pub expected_generation: u64,
    pub expected_document_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerOrchestrationControlAction {
    Recover,
    Cancel,
}

#[derive(Clone)]
pub(crate) struct FullOrchestrationService {
    projects: ProjectStateService,
    graph: OrchestrationTaskGraphV1,
    input_builder: Arc<EmbeddedWorkflowRoleInputBuilder>,
    worker_input_builder: Arc<EmbeddedWorkerOrchestrationInputBuilder>,
    tool_registry: FullProjectToolRegistry,
}

impl FullOrchestrationService {
    pub(crate) fn from_embedded_content(
        projects: ProjectStateService,
        tool_registry: FullProjectToolRegistry,
        content: &EmbeddedContent,
    ) -> Result<Self, FullOrchestrationError> {
        let graph = OrchestrationTaskGraphV1::from_embedded_content(content)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let host =
            InProcessToolHost::with_full_project_service(FullProjectService::new(projects.clone()))
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let tools = project_scoped_read_tools(&tool_registry, &host)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let backend_id = BackendId::parse(OPENAI_BACKEND_ID)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let backend_models = BTreeMap::from([(backend_id, OPENAI_MODEL.to_owned())]);
        let input_builder = EmbeddedWorkflowRoleInputBuilder::from_embedded_content(
            content,
            backend_models.clone(),
            tools.clone(),
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let worker_input_builder =
            EmbeddedWorkerOrchestrationInputBuilder::try_new(backend_models, tools)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        Ok(Self {
            projects,
            graph,
            input_builder: Arc::new(input_builder),
            worker_input_builder: Arc::new(worker_input_builder),
            tool_registry,
        })
    }

    pub(crate) fn doctor_openai(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        settings: &GlobalSettings,
        secrets: &dyn SecretStore,
    ) -> Result<OrchestrationDoctorViewV1, FullOrchestrationError> {
        self.verify_project(project_id, expected_project_revision)?;
        let status = openai_backend_status(settings, secrets);
        let runs = self.list_runs(project_id, expected_project_revision)?;
        let recovery_required_count = runs.runs.iter().filter(|run| run.recovery_required).count();
        let active_run_count = runs
            .runs
            .iter()
            .filter(|run| !run.status.is_terminal())
            .count();
        let mut reason_codes = Vec::new();
        if status.readiness != BackendReadinessV1::Ready {
            reason_codes.push(readiness_reason_code(status.readiness));
        }
        if recovery_required_count > 0 {
            reason_codes.push("orchestration-recovery-required");
        } else if active_run_count > 0 {
            reason_codes.push("orchestration-active-run-exists");
        }
        Ok(OrchestrationDoctorViewV1 {
            schema_version: ORCHESTRATION_VIEW_SCHEMA_VERSION,
            project_id: project_id.clone(),
            expected_project_revision,
            workflow_contract_status: "ready",
            backend_readiness: status.readiness,
            run_count: runs.runs.len(),
            active_run_count,
            recovery_required_count,
            runnable: status.readiness == BackendReadinessV1::Ready && active_run_count == 0,
            reason_codes,
        })
    }

    pub(crate) fn list_runs(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<OrchestrationRunListViewV1, FullOrchestrationError> {
        self.verify_project(project_id, expected_project_revision)?;
        let store = OrchestrationCheckpointStore::new(self.projects.clone());
        let runs = store
            .discover(&self.graph, project_id, expected_project_revision)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?
            .into_iter()
            .map(|run| summarize_run(run.plan(), run.persisted()))
            .collect();
        Ok(OrchestrationRunListViewV1 {
            schema_version: ORCHESTRATION_VIEW_SCHEMA_VERSION,
            project_id: project_id.clone(),
            expected_project_revision,
            runs,
        })
    }

    pub(crate) fn list_worker_runs(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<WorkerOrchestrationRunListViewV1, FullOrchestrationError> {
        self.verify_project(project_id, expected_project_revision)?;
        let runs = WorkerOrchestrationCheckpointStore::new(self.projects.clone())
            .discover(project_id, expected_project_revision)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?
            .into_iter()
            .map(|run| summarize_worker_run(run.plan(), run.persisted()))
            .collect();
        Ok(WorkerOrchestrationRunListViewV1 {
            schema_version: ORCHESTRATION_VIEW_SCHEMA_VERSION,
            project_id: project_id.clone(),
            expected_project_revision,
            runs,
        })
    }

    pub(crate) fn start_openai_worker_test(
        &self,
        project_id: ProjectId,
        expected_project_revision: u64,
        task_id: OrchestrationTaskId,
        confirm_network_request: bool,
        settings: &GlobalSettings,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<WorkerOrchestrationExecutionViewV1, FullOrchestrationError> {
        if !confirm_network_request {
            return Err(FullOrchestrationError::new(
                "orchestration-network-confirmation-required",
            ));
        }
        self.verify_project(&project_id, expected_project_revision)?;
        if self
            .list_worker_runs(&project_id, expected_project_revision)?
            .runs
            .iter()
            .any(|run| !run.status.is_terminal())
        {
            return Err(FullOrchestrationError::new(
                "worker-orchestration-active-run-exists",
            ));
        }
        let backend = self.openai_backend(settings, secrets)?;
        self.start_worker_with_backend(project_id, expected_project_revision, task_id, backend)
    }

    pub(crate) fn continue_openai_worker(
        &self,
        reference: &WorkerOrchestrationRunReference,
        confirm_network_request: bool,
        settings: &GlobalSettings,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<WorkerOrchestrationExecutionViewV1, FullOrchestrationError> {
        if !confirm_network_request {
            return Err(FullOrchestrationError::new(
                "orchestration-network-confirmation-required",
            ));
        }
        let (plan, persisted) = self.resolve_worker_run(reference)?;
        let backend = self.openai_backend(settings, secrets)?;
        self.run_worker_with_backend(&plan, persisted, backend)
    }

    pub(crate) fn control_worker(
        &self,
        reference: &WorkerOrchestrationRunReference,
        action: WorkerOrchestrationControlAction,
    ) -> Result<WorkerOrchestrationRunSummaryV1, FullOrchestrationError> {
        let (plan, persisted) = self.resolve_worker_run(reference)?;
        let mut checkpoint = persisted.checkpoint().clone();
        let generation = checkpoint.generation;
        match action {
            WorkerOrchestrationControlAction::Recover => checkpoint
                .recover_interrupted(&plan, generation)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?,
            WorkerOrchestrationControlAction::Cancel => checkpoint
                .cancel(&plan, generation)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?,
        }
        let persisted = WorkerOrchestrationCheckpointStore::new(self.projects.clone())
            .replace(&plan, &persisted, checkpoint)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        Ok(summarize_worker_run(&plan, &persisted))
    }

    fn start_worker_with_backend(
        &self,
        project_id: ProjectId,
        expected_project_revision: u64,
        task_id: OrchestrationTaskId,
        backend: Arc<dyn AgentBackend>,
    ) -> Result<WorkerOrchestrationExecutionViewV1, FullOrchestrationError> {
        let backend_id = backend.descriptor().backend_id;
        let run_id =
            new_run_id().map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let plan = worker_plan(
            run_id,
            project_id,
            expected_project_revision,
            task_id,
            backend_id,
        )?;
        let checkpoint = plan
            .new_checkpoint()
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let persisted = WorkerOrchestrationCheckpointStore::new(self.projects.clone())
            .create(&plan, checkpoint)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        self.run_worker_with_backend(&plan, persisted, backend)
    }

    pub(crate) fn start_openai_test(
        &self,
        project_id: ProjectId,
        expected_project_revision: u64,
        execution_mode: OrchestrationExecutionMode,
        confirm_network_request: bool,
        settings: &GlobalSettings,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<OrchestrationExecutionViewV1, FullOrchestrationError> {
        if !confirm_network_request {
            return Err(FullOrchestrationError::new(
                "orchestration-network-confirmation-required",
            ));
        }
        self.verify_project(&project_id, expected_project_revision)?;
        if self
            .list_runs(&project_id, expected_project_revision)?
            .runs
            .iter()
            .any(|run| !run.status.is_terminal())
        {
            return Err(FullOrchestrationError::new(
                "orchestration-active-run-exists",
            ));
        }
        let backend = self.openai_backend(settings, secrets)?;
        self.start_with_backend(
            project_id,
            expected_project_revision,
            execution_mode,
            backend,
        )
    }

    fn start_with_backend(
        &self,
        project_id: ProjectId,
        expected_project_revision: u64,
        execution_mode: OrchestrationExecutionMode,
        backend: Arc<dyn AgentBackend>,
    ) -> Result<OrchestrationExecutionViewV1, FullOrchestrationError> {
        let backend_id = backend.descriptor().backend_id;
        let profile = profile(execution_mode, backend_id)?;
        let plan = OrchestrationPlanV1::try_new(self.graph.clone(), profile)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let run_id =
            new_run_id().map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let checkpoint = plan
            .new_checkpoint(run_id, project_id, expected_project_revision)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let store = OrchestrationCheckpointStore::new(self.projects.clone());
        let persisted = store
            .create(&plan, checkpoint)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        self.run_next_with_backend(&plan, persisted, backend)
    }

    pub(crate) fn continue_openai(
        &self,
        reference: &OrchestrationRunReference,
        confirm_network_request: bool,
        settings: &GlobalSettings,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<OrchestrationExecutionViewV1, FullOrchestrationError> {
        if !confirm_network_request {
            return Err(FullOrchestrationError::new(
                "orchestration-network-confirmation-required",
            ));
        }
        let (plan, persisted) = self.resolve_run(reference)?;
        let backend = self.openai_backend(settings, secrets)?;
        self.run_next_with_backend(&plan, persisted, backend)
    }

    pub(crate) fn control(
        &self,
        reference: &OrchestrationRunReference,
        action: OrchestrationControlAction,
    ) -> Result<OrchestrationRunSummaryV1, FullOrchestrationError> {
        let (plan, persisted) = self.resolve_run(reference)?;
        let mut checkpoint = persisted.checkpoint().clone();
        let generation = checkpoint.generation;
        match action {
            OrchestrationControlAction::Pause => checkpoint
                .pause(&plan, generation)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?,
            OrchestrationControlAction::Recover => checkpoint
                .recover_interrupted(&plan, generation)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?,
            OrchestrationControlAction::Resume => checkpoint
                .resume(&plan, generation)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?,
            OrchestrationControlAction::Cancel => checkpoint
                .cancel(&plan, generation)
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?,
        }
        let persisted = OrchestrationCheckpointStore::new(self.projects.clone())
            .replace(&plan, &persisted, checkpoint)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        Ok(summarize_run(&plan, &persisted))
    }

    fn run_next_with_backend(
        &self,
        plan: &OrchestrationPlanV1,
        persisted: qiongli_execution::PersistedOrchestrationCheckpointV1,
        backend: Arc<dyn AgentBackend>,
    ) -> Result<OrchestrationExecutionViewV1, FullOrchestrationError> {
        let checkpoint = persisted.checkpoint();
        let root = self
            .projects
            .resolve_project_root(&checkpoint.project_id)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let scope = ProjectExecutionScope::new(
            checkpoint.project_id.clone(),
            root.path().to_path_buf(),
            checkpoint.expected_project_revision,
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let host = InProcessToolHost::with_full_project_service(FullProjectService::new(
            self.projects.clone(),
        ))
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let tools = project_scoped_read_tools(&self.tool_registry, &host)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let allowed_tools = tools
            .iter()
            .map(|tool| {
                qiongli_execution::ToolId::parse(tool.name.clone())
                    .map_err(|error| FullOrchestrationError::new(error.reason_code()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let policy = AgentExecutionPolicy::locked(
            POLICY_REVISION,
            ExecutionProfile::Full,
            allowed_tools,
            Some(scope),
            execution_limits(),
            RedactionPolicyV1::strict_default(),
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let backend_id = backend.descriptor().backend_id;
        let runner = BoundedAgentRunner::new(backend, host, policy);
        let executor = OrchestrationTaskExecutor::try_new(
            [(backend_id, runner)],
            Arc::clone(&self.input_builder)
                as Arc<dyn qiongli_execution::OrchestrationRoleInputBuilder>,
            OrchestrationCheckpointStore::new(self.projects.clone()),
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let result = block_on(executor.run_next(plan, persisted, CancellationToken::new()))
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let run = summarize_run(plan, &result.persisted);
        let role_outputs = result
            .role_results
            .into_iter()
            .map(|result| OrchestrationRoleOutputViewV1 {
                task_id: result.task_id.as_str().to_owned(),
                role: result.role,
                output_sha256: result.output_sha256,
                model: result.agent_result.model,
                finish_reason: result.agent_result.finish_reason,
                content: result.agent_result.content,
                model_turns: result.agent_result.execution_usage.model_turns,
                tool_calls: result.agent_result.execution_usage.tool_calls,
                network_requests: result.agent_result.execution_usage.network_requests,
            })
            .collect();
        Ok(OrchestrationExecutionViewV1 {
            schema_version: ORCHESTRATION_VIEW_SCHEMA_VERSION,
            outcome: outcome_name(result.outcome),
            task_id: result.task_id.map(|task_id| task_id.as_str().to_owned()),
            run,
            role_outputs,
        })
    }

    fn run_worker_with_backend(
        &self,
        plan: &WorkerOrchestrationPlanV1,
        persisted: qiongli_execution::PersistedWorkerOrchestrationCheckpointV1,
        backend: Arc<dyn AgentBackend>,
    ) -> Result<WorkerOrchestrationExecutionViewV1, FullOrchestrationError> {
        let checkpoint = persisted.checkpoint();
        let root = self
            .projects
            .resolve_project_root(&checkpoint.project_id)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let scope = ProjectExecutionScope::new(
            checkpoint.project_id.clone(),
            root.path().to_path_buf(),
            checkpoint.expected_project_revision,
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let host = InProcessToolHost::with_full_project_service(FullProjectService::new(
            self.projects.clone(),
        ))
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let tools = project_scoped_read_tools(&self.tool_registry, &host)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let allowed_tools = tools
            .iter()
            .map(|tool| {
                qiongli_execution::ToolId::parse(tool.name.clone())
                    .map_err(|error| FullOrchestrationError::new(error.reason_code()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let policy = AgentExecutionPolicy::locked(
            POLICY_REVISION,
            ExecutionProfile::Full,
            allowed_tools,
            Some(scope),
            execution_limits(),
            RedactionPolicyV1::strict_default(),
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let backend_id = backend.descriptor().backend_id;
        let runner = BoundedAgentRunner::new(backend, host, policy);
        let executor = WorkerOrchestrationExecutor::try_new(
            [(backend_id, runner)],
            Arc::clone(&self.worker_input_builder)
                as Arc<dyn qiongli_execution::WorkerOrchestrationInputBuilder>,
            WorkerOrchestrationCheckpointStore::new(self.projects.clone()),
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let result =
            block_on(executor.run_to_completion(plan, persisted, CancellationToken::new()))
                .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let run = summarize_worker_run(plan, &result.persisted);
        let agent_outputs = result
            .agent_results
            .into_iter()
            .map(|result| WorkerOrchestrationAgentOutputViewV1 {
                phase: result.phase,
                worker_id: result
                    .worker_id
                    .map(|worker_id| worker_id.as_str().to_owned()),
                output_sha256: result.output_sha256,
                model: result.agent_result.model,
                finish_reason: result.agent_result.finish_reason,
                content: result.agent_result.content,
                model_turns: result.agent_result.execution_usage.model_turns,
                tool_calls: result.agent_result.execution_usage.tool_calls,
                network_requests: result.agent_result.execution_usage.network_requests,
            })
            .collect();
        Ok(WorkerOrchestrationExecutionViewV1 {
            schema_version: ORCHESTRATION_VIEW_SCHEMA_VERSION,
            outcome: worker_outcome_name(result.outcome),
            run,
            agent_outputs,
        })
    }

    fn openai_backend(
        &self,
        settings: &GlobalSettings,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<Arc<dyn AgentBackend>, FullOrchestrationError> {
        let status = openai_backend_status(settings, secrets.as_ref());
        if status.readiness != BackendReadinessV1::Ready {
            return Err(FullOrchestrationError::new(readiness_reason_code(
                status.readiness,
            )));
        }
        let secret_ref = settings
            .agent_backends
            .openai
            .api_key_ref
            .clone()
            .ok_or_else(|| FullOrchestrationError::new("agent-backend-secret-reference-missing"))?;
        let backend = OpenAiResponsesBackend::for_bounded_run(
            OpenAiBackendConfigV1::gpt_5_6_sol(secret_ref),
            secrets,
        )
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        Ok(Arc::new(backend))
    }

    fn resolve_run(
        &self,
        reference: &OrchestrationRunReference,
    ) -> Result<
        (
            OrchestrationPlanV1,
            qiongli_execution::PersistedOrchestrationCheckpointV1,
        ),
        FullOrchestrationError,
    > {
        if reference.expected_project_revision == 0
            || reference.expected_document_sha256.len() != 64
            || !reference
                .expected_document_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(FullOrchestrationError::new(
                "orchestration-run-reference-invalid",
            ));
        }
        let run = OrchestrationCheckpointStore::new(self.projects.clone())
            .discover(
                &self.graph,
                &reference.project_id,
                reference.expected_project_revision,
            )
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?
            .into_iter()
            .find(|run| run.persisted().checkpoint().run_id == reference.run_id)
            .ok_or_else(|| FullOrchestrationError::new("orchestration-run-not-found"))?;
        if run.persisted().checkpoint().generation != reference.expected_generation
            || run.persisted().document_sha256() != reference.expected_document_sha256
        {
            return Err(FullOrchestrationError::new(
                "orchestration-run-reference-stale",
            ));
        }
        Ok((run.plan().clone(), run.persisted().clone()))
    }

    fn resolve_worker_run(
        &self,
        reference: &WorkerOrchestrationRunReference,
    ) -> Result<
        (
            WorkerOrchestrationPlanV1,
            qiongli_execution::PersistedWorkerOrchestrationCheckpointV1,
        ),
        FullOrchestrationError,
    > {
        if reference.expected_project_revision == 0
            || reference.expected_document_sha256.len() != 64
            || !reference
                .expected_document_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(FullOrchestrationError::new(
                "worker-orchestration-run-reference-invalid",
            ));
        }
        let run = WorkerOrchestrationCheckpointStore::new(self.projects.clone())
            .discover(&reference.project_id, reference.expected_project_revision)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?
            .into_iter()
            .find(|run| run.persisted().checkpoint().run_id == reference.run_id)
            .ok_or_else(|| FullOrchestrationError::new("worker-orchestration-run-not-found"))?;
        if run.persisted().checkpoint().generation != reference.expected_generation
            || run.persisted().document_sha256() != reference.expected_document_sha256
        {
            return Err(FullOrchestrationError::new(
                "worker-orchestration-run-reference-stale",
            ));
        }
        Ok((run.plan().clone(), run.persisted().clone()))
    }

    fn verify_project(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
    ) -> Result<(), FullOrchestrationError> {
        if expected_project_revision == 0 {
            return Err(FullOrchestrationError::new(
                "orchestration-project-reference-invalid",
            ));
        }
        let snapshot = self
            .projects
            .snapshot()
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        let project = snapshot
            .projects
            .iter()
            .find(|project| &project.project_id == project_id)
            .ok_or_else(|| FullOrchestrationError::new("project-not-registered"))?;
        if project.semantic_revision != expected_project_revision {
            return Err(FullOrchestrationError::new("revision-conflict"));
        }
        self.projects
            .resolve_project_root(project_id)
            .map_err(|error| FullOrchestrationError::new(error.reason_code()))?;
        Ok(())
    }
}

fn profile(
    mode: OrchestrationExecutionMode,
    backend_id: BackendId,
) -> Result<OrchestrationProfileV1, FullOrchestrationError> {
    let (profile_id, reviewer, verifier) = match mode {
        OrchestrationExecutionMode::Solo => ("openai-solo-v1", None, None),
        OrchestrationExecutionMode::Duo => ("openai-duo-v1", Some(backend_id.clone()), None),
        OrchestrationExecutionMode::Triad => (
            "openai-triad-v1",
            Some(backend_id.clone()),
            Some(backend_id.clone()),
        ),
    };
    OrchestrationProfileV1::try_new(
        profile_id,
        mode,
        backend_id,
        reviewer,
        verifier,
        MAX_TASK_ATTEMPTS,
        true,
    )
    .map_err(|error| FullOrchestrationError::new(error.reason_code()))
}

fn worker_plan(
    run_id: RunId,
    project_id: ProjectId,
    expected_project_revision: u64,
    task_id: OrchestrationTaskId,
    backend_id: BackendId,
) -> Result<WorkerOrchestrationPlanV1, FullOrchestrationError> {
    let (mode, merge_policy, failure_policy, required_successes, workers) = match task_id.as_str() {
        "B1" => (
            WorkerOrchestrationMode::DelegatedWorkers,
            WorkerMergePolicy::SynthesizeWithConflictMatrix,
            WorkerBarrierFailurePolicy::Degrade,
            2,
            vec![
                worker_spec(
                    "literature_search_worker",
                    "Build the bounded search-strategy facet and report provenance gaps.",
                    &backend_id,
                )?,
                worker_spec(
                    "screening_worker",
                    "Screen the bounded evidence facet and report inclusion conflicts.",
                    &backend_id,
                )?,
                worker_spec(
                    "extraction_worker",
                    "Extract the bounded evidence facet and report missing fields.",
                    &backend_id,
                )?,
            ],
        ),
        "H3" => (
            WorkerOrchestrationMode::ReviewSwarm,
            WorkerMergePolicy::ControllerAdjudication,
            WorkerBarrierFailurePolicy::Block,
            3,
            vec![
                worker_spec(
                    "methodologist",
                    "Review statistical validity, identification, and robustness.",
                    &backend_id,
                )?,
                worker_spec(
                    "domain_expert",
                    "Review theoretical grounding, literature coverage, and contribution.",
                    &backend_id,
                )?,
                worker_spec(
                    "reviewer_2",
                    "Review presentation, reproducibility, and reporting clarity.",
                    &backend_id,
                )?,
            ],
        ),
        _ => {
            return Err(FullOrchestrationError::new(
                "worker-orchestration-task-unsupported",
            ));
        }
    };
    WorkerOrchestrationPlanV1::try_new(
        run_id,
        project_id,
        expected_project_revision,
        task_id,
        mode,
        merge_policy,
        failure_policy,
        required_successes,
        MAX_TASK_ATTEMPTS,
        backend_id.clone(),
        backend_id,
        workers,
    )
    .map_err(|error| FullOrchestrationError::new(error.reason_code()))
}

fn worker_spec(
    worker_id: &str,
    goal: &str,
    backend_id: &BackendId,
) -> Result<WorkerSpecV1, FullOrchestrationError> {
    WorkerSpecV1::try_new(worker_id, backend_id.clone(), goal, worker_id)
        .map_err(|error| FullOrchestrationError::new(error.reason_code()))
}

fn summarize_run(
    plan: &OrchestrationPlanV1,
    persisted: &qiongli_execution::PersistedOrchestrationCheckpointV1,
) -> OrchestrationRunSummaryV1 {
    let checkpoint = persisted.checkpoint();
    let active_task_id = checkpoint
        .tasks
        .iter()
        .find(|task| task.state == OrchestrationTaskState::Running)
        .map(|task| task.task_id.as_str().to_owned());
    let recovery_required =
        checkpoint.status == OrchestrationRunStatus::Running && active_task_id.is_some();
    OrchestrationRunSummaryV1 {
        run_id: checkpoint.run_id.clone(),
        profile_id: plan.profile().profile_id.as_str().to_owned(),
        execution_mode: plan.profile().execution_mode,
        status: checkpoint.status,
        generation: checkpoint.generation,
        document_sha256: persisted.document_sha256().to_owned(),
        completed_task_count: checkpoint
            .tasks
            .iter()
            .filter(|task| task.state == OrchestrationTaskState::Completed)
            .count(),
        total_task_count: checkpoint.tasks.len(),
        next_task_id: checkpoint
            .next_ready_task()
            .map(|task_id| task_id.as_str().to_owned()),
        active_task_id,
        recovery_required,
        can_continue: checkpoint.status == OrchestrationRunStatus::Planned
            || (checkpoint.status == OrchestrationRunStatus::Running && !recovery_required),
        can_pause: checkpoint.status == OrchestrationRunStatus::Running && !recovery_required,
        can_resume: checkpoint.status == OrchestrationRunStatus::Paused,
        can_recover: recovery_required,
        can_cancel: !checkpoint.status.is_terminal(),
    }
}

fn summarize_worker_run(
    plan: &WorkerOrchestrationPlanV1,
    persisted: &qiongli_execution::PersistedWorkerOrchestrationCheckpointV1,
) -> WorkerOrchestrationRunSummaryV1 {
    let checkpoint = persisted.checkpoint();
    let recovery_required = match checkpoint.status {
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
    };
    WorkerOrchestrationRunSummaryV1 {
        run_id: checkpoint.run_id.clone(),
        task_id: checkpoint.task_id.as_str().to_owned(),
        mode: plan.mode,
        status: checkpoint.status,
        barrier_status: checkpoint.barrier_status,
        generation: checkpoint.generation,
        document_sha256: persisted.document_sha256().to_owned(),
        passed_worker_count: checkpoint
            .workers
            .iter()
            .filter(|worker| worker.status == WorkerStatus::Passed)
            .count(),
        total_worker_count: checkpoint.workers.len(),
        recovery_required,
        can_continue: checkpoint.status == WorkerOrchestrationRunStatus::Planned
            || (checkpoint.status == WorkerOrchestrationRunStatus::Running && !recovery_required),
        can_recover: recovery_required,
        can_cancel: !checkpoint.status.is_terminal(),
    }
}

const fn outcome_name(outcome: OrchestrationStepOutcome) -> &'static str {
    match outcome {
        OrchestrationStepOutcome::TaskCompleted => "task-completed",
        OrchestrationStepOutcome::TaskRetryReady => "task-retry-ready",
        OrchestrationStepOutcome::TaskFailed => "task-failed",
        OrchestrationStepOutcome::RunCompleted => "run-completed",
        OrchestrationStepOutcome::RunFailed => "run-failed",
        OrchestrationStepOutcome::RunCancelled => "run-cancelled",
        OrchestrationStepOutcome::Paused => "paused",
    }
}

const fn worker_outcome_name(outcome: WorkerOrchestrationStepOutcome) -> &'static str {
    match outcome {
        WorkerOrchestrationStepOutcome::WorkerRetryReady => "worker-retry-ready",
        WorkerOrchestrationStepOutcome::RunCompleted => "worker-run-completed",
        WorkerOrchestrationStepOutcome::RunBlocked => "worker-run-blocked",
        WorkerOrchestrationStepOutcome::RunCancelled => "worker-run-cancelled",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_execution::{
        AgentEventV1, AgentFinishReason, DeterministicFakeBackend, OrchestrationTaskId,
    };
    use qiongli_project::{ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions};

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        projects: ProjectStateService,
        project_id: ProjectId,
        service: FullOrchestrationService,
    }

    impl Fixture {
        fn new() -> Self {
            let native_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(std::path::Path::parent)
                .unwrap()
                .to_path_buf();
            let root = native_root
                .join("target/qiongli-orchestration-control-tests")
                .join(format!(
                    "{}-{}",
                    std::process::id(),
                    NEXT_ID.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir_all(&root).unwrap();
            let config =
                qiongli_config::resolve_config_root(Some(root.join("config").as_os_str()), &root)
                    .unwrap();
            let projects = ProjectStateService::new(config);
            let project_root = root.join("article");
            let create = projects
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new(
                        "Orchestration Control Article",
                        ProjectKind::Article,
                    ),
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
            let content = crate::embedded_content().unwrap();
            let tools = FullProjectToolRegistry::from_embedded_content(&content).unwrap();
            let service =
                FullOrchestrationService::from_embedded_content(projects.clone(), tools, &content)
                    .unwrap();
            Self {
                root,
                projects,
                project_id,
                service,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn backend(outputs: &[&str]) -> Arc<DeterministicFakeBackend> {
        let turns = outputs
            .iter()
            .map(|output| {
                vec![
                    Ok(AgentEventV1::ContentDelta {
                        content: (*output).to_owned(),
                    }),
                    Ok(AgentEventV1::Completed {
                        finish_reason: AgentFinishReason::Stop,
                    }),
                ]
            })
            .collect();
        Arc::new(
            DeterministicFakeBackend::from_turns(turns)
                .unwrap()
                .with_identity(
                    BackendId::parse(OPENAI_BACKEND_ID).unwrap(),
                    vec![OPENAI_MODEL.to_owned()],
                )
                .unwrap(),
        )
    }

    fn reference(
        run: &OrchestrationRunSummaryV1,
        project_id: &ProjectId,
    ) -> OrchestrationRunReference {
        OrchestrationRunReference {
            project_id: project_id.clone(),
            expected_project_revision: 1,
            run_id: run.run_id.clone(),
            expected_generation: run.generation,
            expected_document_sha256: run.document_sha256.clone(),
        }
    }

    fn worker_reference(
        run: &WorkerOrchestrationRunSummaryV1,
        project_id: &ProjectId,
    ) -> WorkerOrchestrationRunReference {
        WorkerOrchestrationRunReference {
            project_id: project_id.clone(),
            expected_project_revision: 1,
            run_id: run.run_id.clone(),
            expected_generation: run.generation,
            expected_document_sha256: run.document_sha256.clone(),
        }
    }

    #[test]
    fn single_task_test_returns_role_content_but_persists_only_its_hash() {
        let fixture = Fixture::new();
        let fake = backend(&["private model candidate"]);
        let result = fixture
            .service
            .start_with_backend(
                fixture.project_id.clone(),
                1,
                OrchestrationExecutionMode::Solo,
                fake.clone(),
            )
            .unwrap();

        assert_eq!(result.outcome, "task-completed");
        assert_eq!(result.task_id.as_deref(), Some("A1"));
        assert_eq!(result.role_outputs.len(), 1);
        assert_eq!(result.role_outputs[0].content, "private model candidate");
        assert_eq!(result.run.completed_task_count, 1);
        let document = fixture
            .projects
            .read_orchestration_checkpoint(&fixture.project_id, 1, result.run.run_id.as_str())
            .unwrap()
            .unwrap();
        let raw = String::from_utf8(document.bytes().to_vec()).unwrap();
        assert!(!raw.contains("private model candidate"));
        assert!(raw.contains(&result.role_outputs[0].output_sha256));
        assert_eq!(fake.start_count(), 1);
        let request = fake.last_request().unwrap();
        let rendered = request
            .messages
            .iter()
            .map(|message| message.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("research-question"));
    }

    #[test]
    fn pause_resume_and_cancel_require_the_exact_current_reference() {
        let fixture = Fixture::new();
        let result = fixture
            .service
            .start_with_backend(
                fixture.project_id.clone(),
                1,
                OrchestrationExecutionMode::Solo,
                backend(&["candidate"]),
            )
            .unwrap();
        let first_reference = reference(&result.run, &fixture.project_id);

        let paused = fixture
            .service
            .control(&first_reference, OrchestrationControlAction::Pause)
            .unwrap();
        assert_eq!(paused.status, OrchestrationRunStatus::Paused);
        assert_eq!(
            fixture
                .service
                .control(&first_reference, OrchestrationControlAction::Cancel)
                .unwrap_err()
                .reason_code(),
            "orchestration-run-reference-stale"
        );

        let resumed = fixture
            .service
            .control(
                &reference(&paused, &fixture.project_id),
                OrchestrationControlAction::Resume,
            )
            .unwrap();
        assert_eq!(resumed.status, OrchestrationRunStatus::Running);
        let cancelled = fixture
            .service
            .control(
                &reference(&resumed, &fixture.project_id),
                OrchestrationControlAction::Cancel,
            )
            .unwrap();
        assert_eq!(cancelled.status, OrchestrationRunStatus::Cancelled);
    }

    #[test]
    fn interrupted_task_is_discovered_and_explicitly_recovered_without_a_backend() {
        let fixture = Fixture::new();
        let profile = profile(
            OrchestrationExecutionMode::Solo,
            BackendId::parse(OPENAI_BACKEND_ID).unwrap(),
        )
        .unwrap();
        let plan = OrchestrationPlanV1::try_new(fixture.service.graph.clone(), profile).unwrap();
        let run_id = RunId::parse(format!("run_{}", "f".repeat(32))).unwrap();
        let checkpoint = plan
            .new_checkpoint(run_id, fixture.project_id.clone(), 1)
            .unwrap();
        let store = OrchestrationCheckpointStore::new(fixture.projects.clone());
        let mut persisted = store.create(&plan, checkpoint).unwrap();
        let mut checkpoint = persisted.checkpoint().clone();
        checkpoint.start(&plan, checkpoint.generation).unwrap();
        persisted = store.replace(&plan, &persisted, checkpoint).unwrap();
        let mut checkpoint = persisted.checkpoint().clone();
        checkpoint
            .begin_task(
                &plan,
                checkpoint.generation,
                &OrchestrationTaskId::parse("A1").unwrap(),
            )
            .unwrap();
        store.replace(&plan, &persisted, checkpoint).unwrap();

        let runs = fixture.service.list_runs(&fixture.project_id, 1).unwrap();
        assert_eq!(runs.runs.len(), 1);
        assert!(runs.runs[0].recovery_required);
        assert!(runs.runs[0].can_recover);
        let recovered = fixture
            .service
            .control(
                &reference(&runs.runs[0], &fixture.project_id),
                OrchestrationControlAction::Recover,
            )
            .unwrap();
        assert_eq!(recovered.status, OrchestrationRunStatus::Paused);
        assert!(!recovered.recovery_required);
        assert!(recovered.can_resume);
    }

    #[test]
    fn worker_test_returns_all_phase_content_but_persists_only_hashes() {
        let fixture = Fixture::new();
        let fake = backend(&[
            "extraction output canary",
            "search output canary",
            "screening output canary",
            "synthesis output canary",
            "ACCEPT",
        ]);
        let result = fixture
            .service
            .start_worker_with_backend(
                fixture.project_id.clone(),
                1,
                OrchestrationTaskId::parse("B1").unwrap(),
                fake.clone(),
            )
            .unwrap();

        assert_eq!(result.outcome, "worker-run-completed");
        assert_eq!(result.run.status, WorkerOrchestrationRunStatus::Completed);
        assert_eq!(result.run.barrier_status, Some(WorkerBarrierStatus::Passed));
        assert_eq!(result.agent_outputs.len(), 5);
        assert_eq!(
            result.agent_outputs[3].phase,
            WorkerOrchestrationAgentPhase::Synthesis
        );
        assert_eq!(
            result.agent_outputs[4].phase,
            WorkerOrchestrationAgentPhase::Review
        );
        assert_eq!(fake.start_count(), 5);

        let document = fixture
            .projects
            .read_worker_orchestration_checkpoint(
                &fixture.project_id,
                1,
                result.run.run_id.as_str(),
            )
            .unwrap()
            .unwrap();
        let raw = String::from_utf8(document.bytes().to_vec()).unwrap();
        assert!(!raw.contains("output canary"));
        assert!(raw.contains(&result.agent_outputs[3].output_sha256));
        assert!(!format!("{result:?}").contains("output canary"));
    }

    #[test]
    fn interrupted_worker_run_is_discovered_replayed_and_cancelled_by_exact_reference() {
        let fixture = Fixture::new();
        let plan = worker_plan(
            RunId::parse(format!("run_{}", "e".repeat(32))).unwrap(),
            fixture.project_id.clone(),
            1,
            OrchestrationTaskId::parse("B1").unwrap(),
            BackendId::parse(OPENAI_BACKEND_ID).unwrap(),
        )
        .unwrap();
        let store = WorkerOrchestrationCheckpointStore::new(fixture.projects.clone());
        let mut persisted = store.create(&plan, plan.new_checkpoint().unwrap()).unwrap();
        let mut checkpoint = persisted.checkpoint().clone();
        checkpoint.start(&plan, checkpoint.generation).unwrap();
        persisted = store.replace(&plan, &persisted, checkpoint).unwrap();
        let worker_id = plan.workers[0].worker_id.clone();
        let mut checkpoint = persisted.checkpoint().clone();
        checkpoint
            .begin_worker(&plan, checkpoint.generation, &worker_id)
            .unwrap();
        persisted = store.replace(&plan, &persisted, checkpoint).unwrap();
        let mut checkpoint = persisted.checkpoint().clone();
        checkpoint
            .complete_worker(&plan, checkpoint.generation, &worker_id, "a".repeat(64))
            .unwrap();
        store.replace(&plan, &persisted, checkpoint).unwrap();

        let runs = fixture
            .service
            .list_worker_runs(&fixture.project_id, 1)
            .unwrap();
        assert_eq!(runs.runs.len(), 1);
        assert!(runs.runs[0].recovery_required);
        assert!(runs.runs[0].can_recover);
        let recovered = fixture
            .service
            .control_worker(
                &worker_reference(&runs.runs[0], &fixture.project_id),
                WorkerOrchestrationControlAction::Recover,
            )
            .unwrap();
        assert_eq!(recovered.status, WorkerOrchestrationRunStatus::Running);
        assert!(recovered.can_continue);
        assert_eq!(recovered.passed_worker_count, 0);

        let cancelled = fixture
            .service
            .control_worker(
                &worker_reference(&recovered, &fixture.project_id),
                WorkerOrchestrationControlAction::Cancel,
            )
            .unwrap();
        assert_eq!(cancelled.status, WorkerOrchestrationRunStatus::Cancelled);
        assert_eq!(
            fixture
                .service
                .control_worker(
                    &worker_reference(&recovered, &fixture.project_id),
                    WorkerOrchestrationControlAction::Cancel,
                )
                .unwrap_err()
                .reason_code(),
            "worker-orchestration-run-reference-stale"
        );
    }

    #[test]
    fn only_frozen_b1_and_h3_worker_profiles_are_available() {
        let fixture = Fixture::new();
        let backend = BackendId::parse(OPENAI_BACKEND_ID).unwrap();
        let h3 = worker_plan(
            RunId::parse(format!("run_{}", "d".repeat(32))).unwrap(),
            fixture.project_id.clone(),
            1,
            OrchestrationTaskId::parse("H3").unwrap(),
            backend.clone(),
        )
        .unwrap();
        assert_eq!(h3.mode, WorkerOrchestrationMode::ReviewSwarm);
        assert_eq!(h3.barrier_failure_policy, WorkerBarrierFailurePolicy::Block);
        assert_eq!(
            worker_plan(
                RunId::parse(format!("run_{}", "c".repeat(32))).unwrap(),
                fixture.project_id.clone(),
                1,
                OrchestrationTaskId::parse("A1").unwrap(),
                backend,
            )
            .unwrap_err()
            .reason_code(),
            "worker-orchestration-task-unsupported"
        );
    }
}
