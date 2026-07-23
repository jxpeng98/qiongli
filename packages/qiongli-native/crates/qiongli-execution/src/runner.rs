use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    AgentBackend, AgentBackendError, AgentBackendErrorCode, AgentBackendFuture, AgentEventV1,
    AgentExecutionPolicy, AgentFinishReason, AgentMessageV1, AgentRequirementsV1, AgentRole,
    AgentRunError::EventSequenceInvalid, AgentToolRequestV1, AgentUsageV1, CancellationToken,
    ExecutionError, ExecutionUsageV1, InProcessToolHost, PolicyReasonCode, PolicyToolRequestV1,
    RunId, ToolAuditRecordV1, ToolExecutionKind, ToolId, ToolResultStatus, preflight_backend,
};

pub const BOUNDED_AGENT_RUN_SCHEMA_VERSION: u32 = 1;

const MAX_RUN_PURPOSE_BYTES: usize = 512;

pub struct AgentRunInputV1 {
    pub request: crate::AgentRequestV1,
    pub requirements: AgentRequirementsV1,
    pub purpose: String,
    pub project_id: Option<ProjectId>,
    pub expected_project_revision: Option<u64>,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRunResultV1 {
    pub schema_version: u32,
    pub run_id: RunId,
    pub backend_id: crate::BackendId,
    pub model: String,
    pub finish_reason: AgentFinishReason,
    pub content: String,
    pub provider_usage: AgentUsageV1,
    pub execution_usage: ExecutionUsageV1,
    pub tool_audits: Vec<ToolAuditRecordV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentRunError {
    InvalidRequest,
    PreflightFailed(Vec<String>),
    Backend(AgentBackendError),
    EventInvalid,
    EventSequenceInvalid,
    PolicyDenied(PolicyReasonCode),
    ToolHost(ExecutionError),
    LimitExceeded,
    Cancelled,
}

impl AgentRunError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "agent-request-invalid",
            Self::PreflightFailed(_) => "agent-run-preflight-failed",
            Self::Backend(error) => error.code.reason_code(),
            Self::EventInvalid => "agent-event-invalid",
            Self::EventSequenceInvalid => "agent-event-sequence-invalid",
            Self::PolicyDenied(reason) => reason.as_str(),
            Self::ToolHost(error) => error.reason_code(),
            Self::LimitExceeded => "agent-run-limit-exhausted",
            Self::Cancelled => "agent-run-cancelled",
        }
    }

    #[must_use]
    pub fn preflight_reason_codes(&self) -> &[String] {
        match self {
            Self::PreflightFailed(reasons) => reasons,
            _ => &[],
        }
    }
}

impl Display for AgentRunError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for AgentRunError {}

#[derive(Clone)]
pub struct BoundedAgentRunner {
    backend: Arc<dyn AgentBackend>,
    host: InProcessToolHost,
    policy: AgentExecutionPolicy,
}

impl BoundedAgentRunner {
    #[must_use]
    pub fn new(
        backend: Arc<dyn AgentBackend>,
        host: InProcessToolHost,
        policy: AgentExecutionPolicy,
    ) -> Self {
        Self {
            backend,
            host,
            policy,
        }
    }

    pub fn run<'a>(
        &'a self,
        input: AgentRunInputV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<'a, Result<AgentRunResultV1, AgentRunError>> {
        Box::pin(async move {
            let cleanup = RunCleanup {
                backend: self.backend.as_ref(),
                run_id: input.request.run_id.clone(),
            };
            let result = self.run_inner(input, cancellation).await;
            drop(cleanup);
            result
        })
    }

    #[must_use]
    pub fn backend_descriptor(&self) -> crate::AgentBackendDescriptorV1 {
        self.backend.descriptor()
    }

    #[must_use]
    pub const fn project_scope(&self) -> Option<&crate::ProjectExecutionScope> {
        self.policy.project_scope()
    }

    async fn run_inner(
        &self,
        mut input: AgentRunInputV1,
        cancellation: CancellationToken,
    ) -> Result<AgentRunResultV1, AgentRunError> {
        self.validate_input(&input)?;
        let started = Instant::now();
        let mut execution_usage = ExecutionUsageV1::default();
        let mut provider_usage = AgentUsageV1 {
            input_tokens: 0,
            output_tokens: 0,
            cached_input_tokens: 0,
        };
        let mut content = String::new();
        let mut tool_audits = Vec::new();
        let mut seen_call_ids = BTreeSet::new();

        loop {
            self.refresh_elapsed(&mut execution_usage, started, &cancellation)?;
            let descriptor = self.backend.descriptor();
            let preflight = preflight_backend(&descriptor, &input.request, &input.requirements);
            if !preflight.ready {
                return Err(AgentRunError::PreflightFailed(preflight.reason_codes));
            }
            self.reserve_model_turn(&input.request, &mut execution_usage)?;

            let mut stream = self
                .backend
                .start(input.request.clone(), cancellation.clone())
                .await
                .map_err(map_backend_error)?;
            let mut turn_text = String::new();
            let mut tool_requests = Vec::new();
            let mut usage_seen = false;
            let finish_reason = loop {
                self.refresh_elapsed(&mut execution_usage, started, &cancellation)?;
                let event = stream
                    .next_event(&cancellation)
                    .await
                    .ok_or(EventSequenceInvalid)?
                    .map_err(map_backend_error)?;
                event.validate().map_err(|_| AgentRunError::EventInvalid)?;
                match event {
                    AgentEventV1::ContentDelta { content: delta } => {
                        reserve_bytes(
                            &mut execution_usage.output_bytes,
                            delta.len() as u64,
                            self.policy.limits().output_bytes,
                        )?;
                        turn_text.push_str(&delta);
                        content.push_str(&delta);
                    }
                    AgentEventV1::ReasoningStatus { .. } => {}
                    AgentEventV1::ToolRequest { request } => {
                        if !seen_call_ids.insert(request.call_id.as_str().to_owned()) {
                            return Err(EventSequenceInvalid);
                        }
                        tool_requests.push(request);
                    }
                    AgentEventV1::Usage { usage } => {
                        if usage_seen {
                            return Err(EventSequenceInvalid);
                        }
                        usage_seen = true;
                        add_provider_usage(&mut provider_usage, &usage)?;
                    }
                    AgentEventV1::RetryableError { class, code } => {
                        return Err(map_backend_error(AgentBackendError::new(code, Some(class))));
                    }
                    AgentEventV1::TerminalError { code } => {
                        return Err(map_backend_error(AgentBackendError::new(code, None)));
                    }
                    AgentEventV1::Cancelled => return Err(AgentRunError::Cancelled),
                    AgentEventV1::Completed { finish_reason } => break finish_reason,
                }
            };
            if let Some(event) = stream.next_event(&cancellation).await {
                event.map_err(map_backend_error)?;
                return Err(EventSequenceInvalid);
            }

            match finish_reason {
                AgentFinishReason::ToolRequest if !tool_requests.is_empty() => {
                    if !turn_text.is_empty() {
                        input.request.messages.push(AgentMessageV1 {
                            role: AgentRole::Assistant,
                            content: turn_text,
                            tool_call_id: None,
                        });
                    }
                    for request in tool_requests {
                        let message = self.dispatch_tool(
                            &input,
                            request,
                            &mut execution_usage,
                            &mut tool_audits,
                            started,
                            &cancellation,
                        )?;
                        input.request.messages.push(message);
                    }
                }
                AgentFinishReason::ToolRequest => return Err(EventSequenceInvalid),
                AgentFinishReason::Stop | AgentFinishReason::Length if tool_requests.is_empty() => {
                    self.refresh_elapsed(&mut execution_usage, started, &cancellation)?;
                    return Ok(AgentRunResultV1 {
                        schema_version: BOUNDED_AGENT_RUN_SCHEMA_VERSION,
                        run_id: input.request.run_id,
                        backend_id: descriptor.backend_id,
                        model: input.request.model,
                        finish_reason,
                        content,
                        provider_usage,
                        execution_usage,
                        tool_audits,
                    });
                }
                AgentFinishReason::Stop | AgentFinishReason::Length => {
                    return Err(EventSequenceInvalid);
                }
            }
        }
    }

    fn validate_input(&self, input: &AgentRunInputV1) -> Result<(), AgentRunError> {
        if input.request.validate().is_err()
            || input.purpose.trim().is_empty()
            || input.purpose.len() > MAX_RUN_PURPOSE_BYTES
            || input.project_id.is_some() != input.expected_project_revision.is_some()
            || input
                .request
                .messages
                .iter()
                .any(|message| message.role == AgentRole::Tool)
        {
            return Err(AgentRunError::InvalidRequest);
        }
        for tool in &input.request.tools {
            let tool_id =
                ToolId::parse(tool.name.clone()).map_err(|_| AgentRunError::InvalidRequest)?;
            let registration = self
                .host
                .registry()
                .registration(&tool_id)
                .ok_or(AgentRunError::InvalidRequest)?;
            if registration.execution != ToolExecutionKind::InProcessReadOnly
                || !registration.read_only
                || !self.policy.is_tool_allowlisted(&tool_id)
            {
                return Err(AgentRunError::InvalidRequest);
            }
        }
        Ok(())
    }

    fn reserve_model_turn(
        &self,
        request: &crate::AgentRequestV1,
        usage: &mut ExecutionUsageV1,
    ) -> Result<(), AgentRunError> {
        if usage.model_turns >= self.policy.limits().model_turns
            || usage.network_requests >= self.policy.limits().network_requests
        {
            return Err(AgentRunError::LimitExceeded);
        }
        let input_bytes = serde_json::to_vec(request)
            .map_err(|_| AgentRunError::InvalidRequest)?
            .len() as u64;
        reserve_bytes(
            &mut usage.input_bytes,
            input_bytes,
            self.policy.limits().input_bytes,
        )?;
        usage.model_turns += 1;
        usage.network_requests += 1;
        Ok(())
    }

    fn dispatch_tool(
        &self,
        input: &AgentRunInputV1,
        request: AgentToolRequestV1,
        usage: &mut ExecutionUsageV1,
        audits: &mut Vec<ToolAuditRecordV1>,
        started: Instant,
        cancellation: &CancellationToken,
    ) -> Result<AgentMessageV1, AgentRunError> {
        self.refresh_elapsed(usage, started, cancellation)?;
        let tool_id = ToolId::parse(request.tool_name).map_err(|_| EventSequenceInvalid)?;
        let registration = self
            .host
            .registry()
            .registration(&tool_id)
            .ok_or(EventSequenceInvalid)?;
        let argument_bytes = serde_json::to_vec(&request.arguments)
            .map_err(|_| EventSequenceInvalid)?
            .len() as u64;
        reserve_bytes(
            &mut usage.input_bytes,
            argument_bytes,
            self.policy.limits().input_bytes,
        )?;
        let policy_request = PolicyToolRequestV1 {
            schema_version: 1,
            run_id: input.request.run_id.clone(),
            call_id: request.call_id.clone(),
            purpose: input.purpose.clone(),
            tool_id,
            arguments: request.arguments,
            project_id: registration
                .requires_project
                .then(|| input.project_id.clone())
                .flatten(),
            expected_project_revision: registration
                .requires_project
                .then_some(input.expected_project_revision)
                .flatten(),
            declared_artifacts: Vec::new(),
        };
        let decision = self
            .policy
            .evaluate(registration, &policy_request, usage, None, now_unix());
        if !decision.is_allowed() {
            if decision.reason == PolicyReasonCode::LimitExhausted {
                return Err(AgentRunError::LimitExceeded);
            }
            return Err(AgentRunError::PolicyDenied(decision.reason));
        }
        let invocation = self
            .host
            .registry()
            .prepare(&self.policy, policy_request, &decision)
            .map_err(AgentRunError::ToolHost)?;
        usage.tool_calls = usage
            .tool_calls
            .checked_add(1)
            .ok_or(AgentRunError::LimitExceeded)?;
        let result = self
            .host
            .dispatch(&invocation, cancellation)
            .map_err(AgentRunError::ToolHost)?;
        match result.status {
            ToolResultStatus::Cancelled => return Err(AgentRunError::Cancelled),
            ToolResultStatus::LimitExceeded => return Err(AgentRunError::LimitExceeded),
            ToolResultStatus::Completed | ToolResultStatus::Failed => {}
        }
        reserve_bytes(
            &mut usage.output_bytes,
            result.audit.output_bytes,
            self.policy.limits().output_bytes,
        )?;
        let message_content = serde_json::to_string(&json!({
            "schemaVersion": 1,
            "status": result.status,
            "content": result.content,
            "truncated": result.truncated,
            "reasonCode": result.audit.reason_code,
        }))
        .map_err(|_| AgentRunError::ToolHost(ExecutionError::ToolHostContractInvalid))?;
        audits.push(result.audit);
        Ok(AgentMessageV1 {
            role: AgentRole::Tool,
            content: message_content,
            tool_call_id: Some(request.call_id),
        })
    }

    fn refresh_elapsed(
        &self,
        usage: &mut ExecutionUsageV1,
        started: Instant,
        cancellation: &CancellationToken,
    ) -> Result<(), AgentRunError> {
        if cancellation.is_cancelled() {
            return Err(AgentRunError::Cancelled);
        }
        usage.elapsed_seconds = started.elapsed().as_secs();
        let limits = self.policy.limits();
        if usage.elapsed_seconds >= limits.wall_clock_seconds
            || usage.model_turns > limits.model_turns
            || usage.tool_calls > limits.tool_calls
            || usage.processes > limits.processes
            || usage.input_bytes > limits.input_bytes
            || usage.output_bytes > limits.output_bytes
            || usage.network_requests > limits.network_requests
            || usage.artifacts > limits.artifacts
        {
            return Err(AgentRunError::LimitExceeded);
        }
        Ok(())
    }
}

struct RunCleanup<'a> {
    backend: &'a dyn AgentBackend,
    run_id: RunId,
}

impl Drop for RunCleanup<'_> {
    fn drop(&mut self) {
        self.backend.forget_run(&self.run_id);
    }
}

fn reserve_bytes(current: &mut u64, additional: u64, limit: u64) -> Result<(), AgentRunError> {
    let next = current
        .checked_add(additional)
        .ok_or(AgentRunError::LimitExceeded)?;
    if next > limit {
        return Err(AgentRunError::LimitExceeded);
    }
    *current = next;
    Ok(())
}

fn add_provider_usage(
    aggregate: &mut AgentUsageV1,
    turn: &AgentUsageV1,
) -> Result<(), AgentRunError> {
    aggregate.input_tokens = aggregate
        .input_tokens
        .checked_add(turn.input_tokens)
        .ok_or(EventSequenceInvalid)?;
    aggregate.output_tokens = aggregate
        .output_tokens
        .checked_add(turn.output_tokens)
        .ok_or(EventSequenceInvalid)?;
    aggregate.cached_input_tokens = aggregate
        .cached_input_tokens
        .checked_add(turn.cached_input_tokens)
        .ok_or(EventSequenceInvalid)?;
    Ok(())
}

fn map_backend_error(error: AgentBackendError) -> AgentRunError {
    if error.code == AgentBackendErrorCode::Cancelled {
        AgentRunError::Cancelled
    } else {
        AgentRunError::Backend(error)
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Waker};

    use serde_json::{Value, json};

    use crate::{
        AgentBackendError, AgentResponseConstraintsV1, AgentToolSchemaV1, DeterministicFakeBackend,
        ExecutionLimitsV1, ExecutionProfile, ReadOnlyToolRequest, ReadOnlyToolService,
        RedactionPolicyV1, ToolClass, ToolRegistrationV1, ToolServiceError,
    };

    use super::*;

    const RUN_ID: &str = "run_11111111111111111111111111111111";
    const CALL_ID: &str = "call_22222222222222222222222222222222";
    const TOOL_NAME: &str = "qiongli_project_list";

    struct FixedService {
        calls: Arc<AtomicUsize>,
    }

    struct TrackingBackend {
        inner: DeterministicFakeBackend,
        forgotten: Arc<AtomicUsize>,
    }

    impl AgentBackend for TrackingBackend {
        fn descriptor(&self) -> crate::AgentBackendDescriptorV1 {
            self.inner.descriptor()
        }

        fn start<'a>(
            &'a self,
            request: crate::AgentRequestV1,
            cancellation: CancellationToken,
        ) -> AgentBackendFuture<'a, Result<Box<dyn crate::AgentEventStream>, AgentBackendError>>
        {
            self.inner.start(request, cancellation)
        }

        fn forget_run(&self, run_id: &RunId) {
            self.forgotten.fetch_add(1, Ordering::AcqRel);
            self.inner.forget_run(run_id);
        }
    }

    impl ReadOnlyToolService for FixedService {
        fn invoke(
            &self,
            _request: ReadOnlyToolRequest<'_>,
            _cancellation: &CancellationToken,
        ) -> Result<Value, ToolServiceError> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            Ok(json!({"projectCount": 1, "privatePath": "/private/project"}))
        }
    }

    fn ready<T>(mut future: impl Future<Output = T> + Unpin) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        match std::pin::Pin::new(&mut future).poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("deterministic runner future must be immediately ready"),
        }
    }

    fn tool_registration(requires_project: bool) -> ToolRegistrationV1 {
        ToolRegistrationV1 {
            schema_version: 1,
            tool_id: ToolId::parse(TOOL_NAME).unwrap(),
            class: ToolClass::Read,
            execution: ToolExecutionKind::InProcessReadOnly,
            read_only: true,
            requires_project,
            requires_approval: false,
            allows_network: false,
        }
    }

    fn tool_schema() -> AgentToolSchemaV1 {
        AgentToolSchemaV1 {
            name: TOOL_NAME.to_owned(),
            description: "List registered projects.".to_owned(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        }
    }

    fn input(project_id: Option<ProjectId>, revision: Option<u64>) -> AgentRunInputV1 {
        AgentRunInputV1 {
            request: crate::AgentRequestV1 {
                schema_version: 1,
                run_id: RunId::parse(RUN_ID).unwrap(),
                model: "deterministic-v1".to_owned(),
                messages: vec![AgentMessageV1 {
                    role: AgentRole::User,
                    content: "How many projects are registered?".to_owned(),
                    tool_call_id: None,
                }],
                attachments: Vec::new(),
                response: AgentResponseConstraintsV1 {
                    maximum_output_tokens: 128,
                    structured_output_schema: None,
                },
                tools: vec![tool_schema()],
            },
            requirements: AgentRequirementsV1 {
                minimum_context_tokens: 1_024,
                streaming: false,
                structured_output: false,
                tool_calls: true,
                multimodal: false,
                cancellation: true,
            },
            purpose: "Answer one read-only Research Library question.".to_owned(),
            project_id,
            expected_project_revision: revision,
        }
    }

    fn fake_turns() -> Vec<Vec<Result<AgentEventV1, AgentBackendError>>> {
        vec![
            vec![
                Ok(AgentEventV1::Usage {
                    usage: AgentUsageV1 {
                        input_tokens: 10,
                        output_tokens: 2,
                        cached_input_tokens: 1,
                    },
                }),
                Ok(AgentEventV1::ToolRequest {
                    request: AgentToolRequestV1 {
                        call_id: crate::ToolCallId::parse(CALL_ID).unwrap(),
                        tool_name: TOOL_NAME.to_owned(),
                        arguments: json!({}),
                    },
                }),
                Ok(AgentEventV1::Completed {
                    finish_reason: AgentFinishReason::ToolRequest,
                }),
            ],
            vec![
                Ok(AgentEventV1::ContentDelta {
                    content: "One project is registered.".to_owned(),
                }),
                Ok(AgentEventV1::Usage {
                    usage: AgentUsageV1 {
                        input_tokens: 12,
                        output_tokens: 4,
                        cached_input_tokens: 2,
                    },
                }),
                Ok(AgentEventV1::Completed {
                    finish_reason: AgentFinishReason::Stop,
                }),
            ],
        ]
    }

    fn runner(
        backend: Arc<dyn AgentBackend>,
        requires_project: bool,
        project_scope: Option<crate::ProjectExecutionScope>,
        calls: Arc<AtomicUsize>,
        limits: ExecutionLimitsV1,
    ) -> BoundedAgentRunner {
        let mut host = InProcessToolHost::new();
        host.register_read_only(
            tool_registration(requires_project),
            Arc::new(FixedService { calls }),
        )
        .unwrap();
        let policy = AgentExecutionPolicy::locked(
            7,
            ExecutionProfile::Full,
            [ToolId::parse(TOOL_NAME).unwrap()],
            project_scope,
            limits,
            RedactionPolicyV1::strict_default(),
        )
        .unwrap();
        BoundedAgentRunner::new(backend, host, policy)
    }

    #[test]
    fn bounded_runner_completes_the_backend_policy_toolhost_loop() {
        let fake = Arc::new(DeterministicFakeBackend::from_turns(fake_turns()).unwrap());
        let calls = Arc::new(AtomicUsize::new(0));
        let runner = runner(
            fake.clone(),
            false,
            None,
            calls.clone(),
            ExecutionLimitsV1::bounded_default(),
        );

        let result = ready(runner.run(input(None, None), CancellationToken::new())).unwrap();

        assert_eq!(result.schema_version, BOUNDED_AGENT_RUN_SCHEMA_VERSION);
        assert_eq!(result.content, "One project is registered.");
        assert_eq!(result.finish_reason, AgentFinishReason::Stop);
        assert_eq!(result.execution_usage.model_turns, 2);
        assert_eq!(result.execution_usage.network_requests, 2);
        assert_eq!(result.execution_usage.tool_calls, 1);
        assert_eq!(result.provider_usage.input_tokens, 22);
        assert_eq!(result.provider_usage.output_tokens, 6);
        assert_eq!(result.provider_usage.cached_input_tokens, 3);
        assert_eq!(result.tool_audits.len(), 1);
        assert_eq!(result.tool_audits[0].reason_code, "tool-completed");
        assert_eq!(calls.load(Ordering::Acquire), 1);
        assert_eq!(fake.start_count(), 2);
        let continuation = fake.last_request().unwrap();
        let tool_message = continuation.messages.last().unwrap();
        assert_eq!(tool_message.role, AgentRole::Tool);
        assert!(tool_message.content.contains("<redacted>"));
        assert!(!tool_message.content.contains("/private/project"));
    }

    #[test]
    fn project_scope_mismatch_denies_before_the_service_runs() {
        let fake =
            Arc::new(DeterministicFakeBackend::from_turns(vec![fake_turns().remove(0)]).unwrap());
        let calls = Arc::new(AtomicUsize::new(0));
        let policy_project = ProjectId::parse(format!("prj_{}", "3".repeat(32))).unwrap();
        let requested_project = ProjectId::parse(format!("prj_{}", "4".repeat(32))).unwrap();
        let scope =
            crate::ProjectExecutionScope::new(policy_project, std::env::current_dir().unwrap(), 9)
                .unwrap();
        let runner = runner(
            fake,
            true,
            Some(scope),
            calls.clone(),
            ExecutionLimitsV1::bounded_default(),
        );

        let error = ready(runner.run(
            input(Some(requested_project), Some(9)),
            CancellationToken::new(),
        ))
        .err()
        .unwrap();

        assert_eq!(
            error,
            AgentRunError::PolicyDenied(PolicyReasonCode::ProjectScopeMismatch)
        );
        assert_eq!(calls.load(Ordering::Acquire), 0);
    }

    #[test]
    fn model_turn_limit_stops_before_a_second_provider_request() {
        let fake = Arc::new(DeterministicFakeBackend::from_turns(fake_turns()).unwrap());
        let calls = Arc::new(AtomicUsize::new(0));
        let mut limits = ExecutionLimitsV1::bounded_default();
        limits.model_turns = 1;
        let runner = runner(fake.clone(), false, None, calls.clone(), limits);

        let error = ready(runner.run(input(None, None), CancellationToken::new()))
            .err()
            .unwrap();

        assert_eq!(error, AgentRunError::LimitExceeded);
        assert_eq!(fake.start_count(), 1);
        assert_eq!(calls.load(Ordering::Acquire), 0);
    }

    #[test]
    fn unregistered_tools_and_precancelled_runs_never_start_the_backend() {
        let fake = Arc::new(DeterministicFakeBackend::from_turns(fake_turns()).unwrap());
        let calls = Arc::new(AtomicUsize::new(0));
        let runner = runner(
            fake.clone(),
            false,
            None,
            calls,
            ExecutionLimitsV1::bounded_default(),
        );
        let mut invalid = input(None, None);
        invalid.request.tools[0].name = "unregistered_tool".to_owned();
        let error = ready(runner.run(invalid, CancellationToken::new()))
            .err()
            .unwrap();
        assert_eq!(error, AgentRunError::InvalidRequest);
        assert_eq!(fake.start_count(), 0);

        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let error = ready(runner.run(input(None, None), cancellation))
            .err()
            .unwrap();
        assert_eq!(error, AgentRunError::Cancelled);
        assert_eq!(fake.start_count(), 0);
    }

    #[test]
    fn run_scoped_backend_state_is_forgotten_after_completion() {
        let forgotten = Arc::new(AtomicUsize::new(0));
        let backend = Arc::new(TrackingBackend {
            inner: DeterministicFakeBackend::from_turns(fake_turns()).unwrap(),
            forgotten: forgotten.clone(),
        });
        let runner = runner(
            backend,
            false,
            None,
            Arc::new(AtomicUsize::new(0)),
            ExecutionLimitsV1::bounded_default(),
        );

        ready(runner.run(input(None, None), CancellationToken::new())).unwrap();

        assert_eq!(forgotten.load(Ordering::Acquire), 1);
    }
}
