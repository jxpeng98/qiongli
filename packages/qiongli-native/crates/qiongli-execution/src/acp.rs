use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::FutureExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use agent_client_protocol::schema::{
    ProtocolVersion,
    v1::{
        CancelNotification, ClientCapabilities, ContentBlock, ContentChunk, FileSystemCapabilities,
        InitializeRequest, NewSessionRequest, PromptRequest, ReadTextFileRequest,
        ReadTextFileResponse, RequestPermissionOutcome, RequestPermissionRequest,
        RequestPermissionResponse, SessionNotification, SessionUpdate, StopReason,
    },
};
use agent_client_protocol::{
    AcpAgent, AcpAgentConfig, Client, ConnectTo, Dispatch, DynConnectTo, Error as AcpError,
    ErrorCode as AcpErrorCode, Handled, JsonRpcMessage,
};

use crate::acp_control::*;
use crate::all_chat::MAX_CHAT_TEXT_BYTES;
use crate::{
    AgentBackendError, AgentBackendErrorCode, AgentEventV1, AgentFinishReason, AllChatEventKindV1,
    AllChatStateError, AllChatStateV1, CancellationToken, OrchestrationRole,
};
use agent_client_protocol::schema::v1::{
    InitializeResponse, PermissionOptionKind, PlanEntryStatus, SelectedPermissionOutcome,
    SessionConfigKind, SessionConfigOptionCategory, SessionConfigSelectOptions, ToolCallStatus,
};

const ACP_CLIENT_NAME: &str = "qiongli-acp-v1";
const NPX_PROGRAM: &str = "npx";
const CODEX_ACP_PACKAGE: &str = "@agentclientprotocol/codex-acp@1.9.0";
const CLAUDE_ACP_PACKAGE: &str = "@agentclientprotocol/claude-agent-acp@0.74.0";
const MAX_CWD_BYTES: usize = 4 * 1024;
const MAX_PROMPT_BYTES: usize = 256 * 1024;
const MAX_SESSION_ID_BYTES: usize = 256;
const MAX_CONTENT_DELTA_BYTES: usize = 64 * 1024;
const MAX_TURN_CONTENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_TURN_UPDATES: usize = 1024;

// Strict raw decoding precedes the SDK's permissive DefaultOnError range fields.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadViewRequest {
    session_id: String,
    path: String,
    line: Option<u32>,
    limit: Option<u32>,
    #[serde(rename = "_meta")]
    _meta: Option<serde_json::Value>,
}

fn read_view_content(
    view: &BTreeMap<String, String>,
    request: &ReadViewRequest,
) -> Result<String, AcpError> {
    let content = view
        .get(&request.path)
        .ok_or_else(AcpError::invalid_params)?;
    if request.line.is_none() && request.limit.is_none() {
        return Ok(content.clone());
    }
    let start = request
        .line
        .unwrap_or(1)
        .checked_sub(1)
        .ok_or_else(AcpError::invalid_params)? as usize;
    let lines: Vec<_> = content.split_inclusive('\n').collect();
    let count = request
        .limit
        .map_or(lines.len().saturating_sub(start), |n| n as usize);
    let end = start.saturating_add(count).min(lines.len());
    if count == 0 || start >= lines.len() {
        return Err(AcpError::invalid_params());
    }
    Ok(lines[start..end].concat())
}

/// Phase budgets; cancellation acknowledgement is bounded independently of output.
#[derive(Clone, Copy)]
pub struct AcpV1Timeouts {
    pub initialization: Duration,
    pub session_creation: Duration,
    pub prompt: Duration,
    pub permission: Duration,
    pub cancellation_grace: Duration,
}

impl Default for AcpV1Timeouts {
    fn default() -> Self {
        Self {
            initialization: Duration::from_secs(30),
            session_creation: Duration::from_secs(30),
            prompt: Duration::from_secs(300),
            permission: Duration::from_secs(120),
            cancellation_grace: Duration::from_secs(2),
        }
    }
}

impl AcpV1Timeouts {
    fn validate(self) -> Result<(), AgentBackendError> {
        if [
            self.initialization,
            self.session_creation,
            self.prompt,
            self.permission,
            self.cancellation_grace,
        ]
        .into_iter()
        .any(|duration| duration.is_zero() || duration > Duration::from_secs(86_400))
        {
            return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
        }
        Ok(())
    }
}

/// Fixed ACP adapters available only for local development through Node.js `npx`.
///
/// These presets are not packaged provider support. Qiongli neither discovers nor
/// accepts arbitrary commands through this API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpDevelopmentPresetV1 {
    Codex,
    Claude,
}

impl AcpDevelopmentPresetV1 {
    const fn package_reference(self) -> &'static str {
        match self {
            Self::Codex => CODEX_ACP_PACKAGE,
            Self::Claude => CLAUDE_ACP_PACKAGE,
        }
    }
}

/// Qiongli-owned result of one stable ACP v1 prompt turn.
#[derive(Clone, PartialEq)]
pub struct AcpV1TurnOutcome {
    protocol_version: u16,
    turn_id: u64,
    session_id: String,
    events: Vec<AgentEventV1>,
}

impl AcpV1TurnOutcome {
    #[must_use]
    pub const fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Monotonic prompt identity within this provider session (starting at 1).
    #[must_use]
    pub const fn turn_id(&self) -> u64 {
        self.turn_id
    }

    #[must_use]
    pub fn events(&self) -> &[AgentEventV1] {
        &self.events
    }

    /// Atomically projects the first coordinator session and turn into All Chat.
    ///
    /// Stream fragments are aggregated only for a completed turn. A confirmed
    /// cancelled turn records no partial message, and no ACP SDK type enters the
    /// shared state. Any validation or append error leaves `state` unchanged.
    pub fn project_first_coordinator_turn(
        &self,
        state: &mut AllChatStateV1,
        expected_generation: u64,
    ) -> Result<(), AllChatStateError> {
        if expected_generation != state.generation() {
            return Err(AllChatStateError::StaleGeneration);
        }
        if self.protocol_version != ProtocolVersion::V1.as_u16() || self.turn_id != 1 {
            return Err(AllChatStateError::InvalidEvent);
        }

        let (terminal, deltas) = self
            .events
            .split_last()
            .ok_or(AllChatStateError::InvalidEvent)?;
        if deltas
            .iter()
            .any(|event| !matches!(event, AgentEventV1::ContentDelta { .. }))
        {
            return Err(AllChatStateError::InvalidEvent);
        }

        let mut kinds = vec![AllChatEventKindV1::AgentSessionReady {
            role: OrchestrationRole::Primary,
            session_id: self.session_id.clone(),
        }];
        match terminal {
            AgentEventV1::Completed { finish_reason }
                if matches!(
                    finish_reason,
                    AgentFinishReason::Stop | AgentFinishReason::Length
                ) =>
            {
                let content = aggregate_deltas(deltas)?;
                kinds.push(AllChatEventKindV1::CoordinatorMessage {
                    by: OrchestrationRole::Primary,
                    content,
                });
                kinds.push(AllChatEventKindV1::AgentTurnCompleted {
                    by: OrchestrationRole::Primary,
                    finish_reason: *finish_reason,
                });
            }
            AgentEventV1::Cancelled => {
                kinds.push(AllChatEventKindV1::AgentTurnCancelled {
                    by: OrchestrationRole::Primary,
                });
            }
            _ => return Err(AllChatStateError::InvalidEvent),
        }

        let mut candidate = state.clone();
        for (offset, kind) in kinds.into_iter().enumerate() {
            let offset = u64::try_from(offset).map_err(|_| AllChatStateError::LimitExceeded)?;
            let generation = expected_generation
                .checked_add(offset)
                .ok_or(AllChatStateError::LimitExceeded)?;
            let sequence = candidate
                .events()
                .last()
                .map_or(Some(1), |event| event.sequence.checked_add(1))
                .ok_or(AllChatStateError::LimitExceeded)?;
            candidate.append_event(generation, sequence, kind)?;
        }
        *state = candidate;
        Ok(())
    }
}

fn aggregate_deltas(events: &[AgentEventV1]) -> Result<String, AllChatStateError> {
    let mut content = String::new();
    for event in events {
        let AgentEventV1::ContentDelta { content: delta } = event else {
            return Err(AllChatStateError::InvalidEvent);
        };
        content
            .len()
            .checked_add(delta.len())
            .filter(|length| *length <= MAX_CHAT_TEXT_BYTES)
            .ok_or(AllChatStateError::InvalidEvent)?;
        content.push_str(delta);
    }
    if content.is_empty() {
        return Err(AllChatStateError::InvalidEvent);
    }
    Ok(content)
}

/// Stable ACP v1 connection factory with a scoped, reusable session.
///
/// Its public constructor is intentionally development-only: it launches one
/// exact, pinned adapter with `npx`, requires Node.js and `npx` on `PATH`, and is
/// not evidence of packaged provider support. Missing process prerequisites fail
/// closed as [`AgentBackendErrorCode::TransportUnavailable`].
pub struct AcpV1Client {
    transport: DynConnectTo<Client>,
    timeouts: AcpV1Timeouts,
    control: Option<AcpV1Control>,
    preset: Option<AcpDevelopmentPresetV1>,
    read_view: Option<BTreeMap<String, String>>,
}

/// One live session, borrowed only while `AcpV1Client::with_session` drives it.
/// SDK handles are private and mutable borrowing serializes prompts.
pub struct AcpV1Session {
    session: agent_client_protocol::ActiveSession<'static, agent_client_protocol::Agent>,
    session_id: String,
    last_turn_id: u64,
    timeouts: AcpV1Timeouts,
    ready: bool,
    info: AcpV1SessionInfo,
    control: Option<AcpV1Control>,
    invalid_notification: Arc<AtomicBool>,
    permission_requested: Arc<AtomicBool>,
    accept_updates: Arc<AtomicBool>,
}

impl AcpV1Session {
    #[must_use]
    pub fn info(&self) -> &AcpV1SessionInfo {
        &self.info
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Runs one prompt without reinitializing or replacing the provider session.
    /// A failed or abandoned in-flight turn retires this scoped session.
    pub async fn run_turn(
        &mut self,
        prompt: impl Into<String>,
        cancellation: CancellationToken,
    ) -> Result<AcpV1TurnOutcome, AgentBackendError> {
        let prompt = prompt.into();
        validate_prompt(&prompt)?;
        if cancellation.is_cancelled() {
            return Err(backend_error(AgentBackendErrorCode::Cancelled));
        }
        self.check_failure()?;
        if !self.ready {
            return Err(backend_error(AgentBackendErrorCode::TransportUnavailable));
        }
        let turn_id = self
            .last_turn_id
            .checked_add(1)
            .filter(|id| *id <= 9_007_199_254_740_991)
            .ok_or_else(|| backend_error(AgentBackendErrorCode::InvalidRequest))?;
        // Stay unavailable if this future is dropped before its terminal result.
        self.ready = false;
        self.last_turn_id = turn_id;
        self.accept_updates.store(true, Ordering::Release);
        let mut scope = self
            .control
            .as_ref()
            .map(|control| {
                control.begin_turn(
                    &self.session_id,
                    turn_id,
                    cancellation.clone(),
                    self.timeouts.prompt,
                )
            })
            .transpose()?;
        let result = run_turn_on_session(
            &mut self.session,
            prompt,
            cancellation,
            turn_id,
            self.timeouts,
            &mut self.info,
            scope.as_mut(),
        )
        .await;
        let result = self.check_failure().and(result);
        if let Some(scope) = scope.as_mut() {
            if scope.control.timed_out() {
                scope.status = AcpV1TurnStatus::TimedOut;
            }
            if scope.status != AcpV1TurnStatus::TimedOut {
                scope.status = match &result {
                    Ok(outcome)
                        if matches!(outcome.events.last(), Some(AgentEventV1::Cancelled)) =>
                    {
                        AcpV1TurnStatus::Cancelled
                    }
                    Ok(_) => AcpV1TurnStatus::Completed,
                    Err(_) => AcpV1TurnStatus::Failed,
                };
            }
        }
        self.ready = result.is_ok();
        drop(scope);
        self.check_failure()?;
        result
    }

    fn check_failure(&self) -> Result<(), AgentBackendError> {
        if let Some(control) = &self.control {
            control.failure()?;
        }
        if self.invalid_notification.load(Ordering::Acquire) {
            return Err(response_invalid());
        }
        if self.permission_requested.load(Ordering::Acquire) {
            return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable));
        }
        Ok(())
    }
}

impl AcpV1Client {
    /// Builds a development-only client for a fixed, pinned Codex or Claude ACP adapter.
    ///
    /// The SDK launches `npx -y <pinned-package>` without an explicitly configured
    /// shell; callers cannot supply a program, arguments, or environment overrides.
    #[must_use]
    pub fn for_development_npx(preset: AcpDevelopmentPresetV1) -> Self {
        let mut client = Self::from_transport(AcpAgent::new(development_config(preset)));
        client.preset = Some(preset);
        client
    }

    /// Offline UI development fixture, driven through the same ACP SDK connection.
    /// It never launches a process or reads a project. Release builds omit it.
    #[cfg(debug_assertions)]
    pub fn for_development_demo() -> Self {
        use agent_client_protocol::schema::v1::{
            NewSessionResponse, PermissionOption, PromptResponse, TextContent, ToolCallUpdate,
            ToolCallUpdateFields,
        };
        use agent_client_protocol::{Agent, ConnectionTo, Responder};
        Self::from_transport(Agent.builder()
            .on_receive_request(async |_: InitializeRequest, response: Responder<InitializeResponse>, _: ConnectionTo<Client>| {
                response.respond(InitializeResponse::new(ProtocolVersion::V1))
            }, agent_client_protocol::on_receive_request!())
            .on_receive_request(async |_: NewSessionRequest, response: Responder<NewSessionResponse>, _: ConnectionTo<Client>| {
                response.respond(NewSessionResponse::new("offline-demo"))
            }, agent_client_protocol::on_receive_request!())
            .on_receive_request(async |request: PromptRequest, response: Responder<PromptResponse>, connection: ConnectionTo<Client>| {
                let session = request.session_id;
                connection.send_notification(SessionNotification::new(session.clone(),
                    serde_json::from_value(serde_json::json!({"sessionUpdate":"plan", "entries":[{"content":"Demonstrate permission and activity without reading sources", "priority":"medium", "status":"in_progress"}]}))?))?;
                connection.send_request(RequestPermissionRequest::new(session.clone(),
                    ToolCallUpdate::new("demo-tool", ToolCallUpdateFields::new().title("Offline demonstration")),
                    vec![PermissionOption::new("allow", "Allow once", PermissionOptionKind::AllowOnce),
                         PermissionOption::new("deny", "Deny once", PermissionOptionKind::RejectOnce)]))
                    .on_receiving_result(async move |result| {
                        let allowed = matches!(result?.outcome, RequestPermissionOutcome::Selected(choice) if choice.option_id.to_string() == "allow");
                        connection.send_notification(SessionNotification::new(session.clone(),
                            serde_json::from_value(serde_json::json!({"sessionUpdate":"tool_call_update", "toolCallId":"demo-tool", "title":"Offline demonstration", "status":"completed"}))?))?;
                        connection.send_notification(SessionNotification::new(session, SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
                            if allowed { "Offline demonstration completed. No source was read and no file was changed. You can send another message in this session." }
                            else { "Permission was declined. No source was read and no file was changed." }
                        ))))))?;
                        response.respond(PromptResponse::new(StopReason::EndTurn))
                    })
            }, agent_client_protocol::on_receive_request!()))
    }

    /// Bounded credential-free responses for the native research integration fixture.
    /// Uses the actual ACP connection; never launches an executable or a model.
    #[cfg(debug_assertions)]
    pub fn for_development_responses(responses: Vec<String>) -> Result<Self, AgentBackendError> {
        Self::development_responses(responses, Vec::new())
    }

    /// Reads every approved snapshot over ACP before each deterministic response.
    /// Read failures or substituted content prevent a successful fixture outcome.
    #[cfg(debug_assertions)]
    pub fn for_development_read_responses(
        responses: Vec<String>,
        files: Vec<(String, String)>,
    ) -> Result<Self, AgentBackendError> {
        Self::development_responses(responses, files.clone())?.with_read_view(files)
    }

    #[cfg(debug_assertions)]
    fn development_responses(
        responses: Vec<String>,
        files: Vec<(String, String)>,
    ) -> Result<Self, AgentBackendError> {
        use agent_client_protocol::schema::v1::{NewSessionResponse, PromptResponse, TextContent};
        use agent_client_protocol::{Agent, ConnectionTo, Responder};
        if responses.is_empty()
            || responses.len() > 64
            || responses
                .iter()
                .any(|s| s.is_empty() || s.len() > 64 * 1024)
        {
            return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
        }
        let responses = std::sync::Mutex::new(std::collections::VecDeque::from(responses));
        let mut fixture_turn = 0_u64;
        Ok(Self::from_transport(
            Agent
                .builder()
                .on_receive_request(
                    async |_: InitializeRequest,
                           responder: Responder<InitializeResponse>,
                           _: ConnectionTo<Client>| {
                        responder.respond(InitializeResponse::new(ProtocolVersion::V1))
                    },
                    agent_client_protocol::on_receive_request!(),
                )
                .on_receive_request(
                    async |_: NewSessionRequest,
                           responder: Responder<NewSessionResponse>,
                           _: ConnectionTo<Client>| {
                        responder.respond(NewSessionResponse::new("offline-research"))
                    },
                    agent_client_protocol::on_receive_request!(),
                )
                .on_receive_request(
                    async move |request: PromptRequest,
                                responder: Responder<PromptResponse>,
                                connection: ConnectionTo<Client>| {
                        let text = responses
                            .lock()
                            .map_err(|_| AcpError::internal_error())?
                            .pop_front()
                            .ok_or_else(AcpError::invalid_params)?;
                        let files = files.clone();
                        fixture_turn += 1;
                        let turn = fixture_turn;
                        connection.clone().spawn(async move {
                            for (index, (path, expected)) in files.into_iter().enumerate() {
                                let update = |status: &str| -> Result<SessionNotification, AcpError> {
                                    Ok(SessionNotification::new(request.session_id.clone(),
                                        serde_json::from_value(serde_json::json!({
                                            "sessionUpdate": if status == "in_progress" { "tool_call" } else { "tool_call_update" },
                                            "toolCallId": format!("context-{turn}-{}", index + 1),
                                            "title": if index == 2 { "Read selected method" } else { "Read selected excerpt" },
                                            "status": status
                                        }))?))
                                };
                                connection.send_notification(update("in_progress")?)?;
                                let read = connection.send_request(ReadTextFileRequest::new(request.session_id.clone(), path)).block_task().await?;
                                if read.content != expected { return Err(AcpError::invalid_params()); }
                                connection.send_notification(update("completed")?)?;
                            }
                            connection.send_notification(SessionNotification::new(
                                request.session_id,
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(text)))),
                            ))?;
                            responder.respond(PromptResponse::new(StopReason::EndTurn))
                        })
                    },
                    agent_client_protocol::on_receive_request!(),
                ),
        ))
    }

    fn from_transport(transport: impl ConnectTo<Client>) -> Self {
        Self {
            transport: DynConnectTo::new(transport),
            timeouts: AcpV1Timeouts::default(),
            control: None,
            preset: None,
            read_view: None,
        }
    }

    /// Enables transient controls for exactly one caller-owned run participant.
    pub fn with_control(mut self, control: AcpV1Control) -> Self {
        self.control = Some(control);
        self
    }

    /// Installs up to three approved text snapshots under exact virtual paths.
    /// Requires `with_control`; never reads the filesystem or grants provider OS isolation.
    /// All other agent-to-client requests are forbidden in this mode, including permissions.
    pub fn with_read_view(
        mut self,
        files: Vec<(String, String)>,
    ) -> Result<Self, AgentBackendError> {
        let mut view = BTreeMap::new();
        if files.is_empty() || files.len() > 3 {
            if let Some(control) = &self.control {
                control.close_unclaimed();
            }
            return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
        }
        for (path, content) in files {
            if !path.starts_with("/qiongli-context/")
                || path.len() > MAX_CWD_BYTES
                || path.chars().any(char::is_control)
                || path.contains('\\')
                || path
                    .split('/')
                    .skip(1)
                    .any(|part| matches!(part, "" | "." | ".."))
                || content.is_empty()
                || content.len() > 64 * 1024
                || view.insert(path, content).is_some()
            {
                if let Some(control) = &self.control {
                    control.close_unclaimed();
                }
                return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
            }
        }
        self.read_view = Some(view);
        Ok(self)
    }

    pub fn with_timeouts(mut self, timeouts: AcpV1Timeouts) -> Result<Self, AgentBackendError> {
        if let Err(error) = timeouts.validate() {
            if let Some(control) = &self.control {
                control.close_unclaimed();
            }
            return Err(error);
        }
        self.timeouts = timeouts;
        Ok(self)
    }

    /// Initializes ACP v1, creates one new session, and runs one prompt turn.
    pub async fn run_turn(
        self,
        cwd: impl AsRef<Path>,
        prompt: impl Into<String>,
        cancellation: CancellationToken,
    ) -> Result<AcpV1TurnOutcome, AgentBackendError> {
        let cwd = cwd.as_ref().to_path_buf();
        let prompt = prompt.into();
        if let Err(error) = validate_cwd(&cwd).and_then(|()| validate_prompt(&prompt)) {
            if let Some(control) = &self.control {
                control.close_unclaimed();
            }
            return Err(error);
        }
        self.with_session(cwd, cancellation.clone(), async move |session| {
            session.run_turn(prompt, cancellation).await
        })
        .await
    }

    /// Drives one connection and session for the callback's entire lifetime.
    /// Returning from the callback releases the session; it cannot escape its borrow.
    /// The token covers startup; each prompt supplies its own cancellation token.
    pub async fn with_session<R>(
        self,
        cwd: impl AsRef<Path>,
        cancellation: CancellationToken,
        use_session: impl AsyncFnOnce(&mut AcpV1Session) -> Result<R, AgentBackendError>,
    ) -> Result<R, AgentBackendError> {
        let _control_scope = self.control.as_ref().map(AcpV1Control::claim).transpose()?;
        if self.read_view.is_some() && self.control.is_none() {
            return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
        }
        let cwd = cwd.as_ref().to_path_buf();
        validate_cwd(&cwd)?;
        if cancellation.is_cancelled() {
            return Err(backend_error(AgentBackendErrorCode::Cancelled));
        }

        // The SDK only kills the direct child on Windows. A wrapper may orphan
        // descendants; keep this development launcher unavailable until it has a Job owner.
        #[cfg(windows)]
        if self.preset.is_some() {
            return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable));
        }
        let control = self.control.clone();
        let permission_control = control.clone();
        let dispatch_control = control.clone();
        let session_control = control.clone();
        let permission_timeout = self.timeouts.permission;
        let invalid_notification = Arc::new(AtomicBool::new(false));
        let invalid_flag = Arc::clone(&invalid_notification);
        let mut owned_session: Option<String> = None;
        // The first session may emit owned updates directly after session/new.
        let accept_updates = Arc::new(AtomicBool::new(true));
        let update_flag = Arc::clone(&accept_updates);
        let permission_requested = Arc::new(AtomicBool::new(false));
        let permission_flag = Arc::clone(&permission_requested);
        let session_invalid = Arc::clone(&invalid_notification);
        let session_permission = Arc::clone(&permission_requested);
        let read_enabled = self.read_view.is_some();
        let read_view = self.read_view;
        let result = Client
            .builder()
            .name(ACP_CLIENT_NAME)
            .on_receive_dispatch(
                async move |message: Dispatch, connection| {
                    let message = match message {
                        Dispatch::Request(request, responder) if read_enabled => {
                            let result = (|| {
                                if !ReadTextFileRequest::matches_method(request.method())
                                    || !update_flag.load(Ordering::Acquire)
                                    || request.params().to_string().len() > 8 * 1024
                                {
                                    return Err(AcpError::invalid_params());
                                }
                                let request: ReadViewRequest =
                                    serde_json::from_value(request.params().clone())
                                        .map_err(|_| AcpError::invalid_params())?;
                                if owned_session.as_deref() != Some(request.session_id.as_str()) {
                                    return Err(AcpError::invalid_params());
                                }
                                let content = read_view_content(
                                    read_view.as_ref().ok_or_else(AcpError::invalid_params)?,
                                    &request,
                                )?;
                                dispatch_control
                                    .as_ref()
                                    .ok_or_else(AcpError::invalid_params)?
                                    .admit_read(&request.session_id, content.len())
                                    .map_err(|_| AcpError::invalid_params())?;
                                serde_json::to_value(ReadTextFileResponse::new(content))
                                    .map_err(|_| AcpError::internal_error())
                            })();
                            match result {
                                Ok(value) => responder.respond(value)?,
                                Err(error) => {
                                    invalid_flag.store(true, Ordering::Release);
                                    responder.respond_with_error(error)?;
                                    connection.spawn(async { Err(AcpError::invalid_params()) })?;
                                }
                            }
                            return Ok(Handled::Yes);
                        }
                        message => message,
                    };
                    if let Dispatch::Response(Err(error), _) = &message
                        && error.code == AcpErrorCode::AuthRequired
                        && let Some(control) = &dispatch_control
                    {
                        control
                            .authentication_required()
                            .map_err(|_| AcpError::invalid_params())?;
                    }
                    match &message {
                        Dispatch::Response(Ok(value), router)
                            if NewSessionRequest::matches_method(router.method()) =>
                        {
                            let session_id = value
                                .get("sessionId")
                                .and_then(serde_json::Value::as_str)
                                .ok_or_else(AcpError::invalid_params)?;
                            validate_session_id(session_id)
                                .map_err(|_| AcpError::invalid_params())?;
                            // Bind during response dispatch, before immediate session updates.
                            owned_session = Some(session_id.to_owned());
                        }
                        Dispatch::Response(_, router)
                            if PromptRequest::matches_method(router.method()) =>
                        {
                            // ponytail: v1 updates have no turn ID. Reject idle traffic;
                            // causal attribution during the next prompt needs protocol support.
                            update_flag.store(false, Ordering::Release);
                            if dispatch_control
                                .as_ref()
                                .is_some_and(AcpV1Control::reject_pending_terminal)
                            {
                                invalid_flag.store(true, Ordering::Release);
                                connection.spawn(async { Err(AcpError::invalid_params()) })?;
                            }
                        }
                        Dispatch::Notification(notification)
                            if SessionNotification::matches_method(notification.method()) =>
                        {
                            let valid = SessionNotification::parse_message(
                                notification.method(),
                                notification.params(),
                            )
                            .is_ok_and(|update| {
                                update_flag.load(Ordering::Acquire)
                                    && owned_session.as_deref()
                                        == Some(update.session_id.to_string().as_str())
                            });
                            if !valid {
                                // Notification handler errors are only logged by the SDK.
                                // Abort its runner and latch rejection against a racing EndTurn.
                                invalid_flag.store(true, Ordering::Release);
                                connection.spawn(async { Err(AcpError::invalid_params()) })?;
                                return Ok(Handled::Yes);
                            }
                        }
                        _ => {}
                    }
                    // Leave valid updates and responses to the SDK's session/request routing.
                    Ok(Handled::No {
                        message,
                        retry: false,
                    })
                },
                agent_client_protocol::on_receive_dispatch!(),
            )
            .on_receive_request(
                async move |request: RequestPermissionRequest, responder, connection| {
                    let Some(control) = &permission_control else {
                        permission_flag.store(true, Ordering::Release);
                        return responder.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Cancelled,
                        ));
                    };
                    let wait = match prepare_permission(control, request, permission_timeout) {
                        Ok(wait) => wait,
                        Err(error) => {
                            if error.code != AgentBackendErrorCode::Cancelled {
                                control.fail(error.code);
                                connection.spawn(async { Err(AcpError::invalid_params()) })?;
                            }
                            return responder.respond(RequestPermissionResponse::new(
                                RequestPermissionOutcome::Cancelled,
                            ));
                        }
                    };
                    let control = control.clone();
                    connection.spawn(async move {
                        let cancelled = wait.cancellation.cancelled().fuse();
                        let stopped = wait.stop.cancelled().fuse();
                        let timer = async_io::Timer::at(wait.deadline).fuse();
                        let receiver = wait.receiver.fuse();
                        futures::pin_mut!(cancelled, stopped, timer, receiver);
                        let choice = futures::select_biased! {
                            _ = cancelled => AcpV1PermissionChoice::Cancel,
                            _ = stopped => AcpV1PermissionChoice::Cancel,
                            _ = timer => {
                                control.timeout();
                                wait.cancellation.cancel();
                                AcpV1PermissionChoice::Cancel
                            },
                            result = receiver => result.unwrap_or(AcpV1PermissionChoice::Cancel),
                        };
                        control
                            .permission_resolved(&wait.request, choice.clone())
                            .map_err(|_| AcpError::invalid_params())?;
                        let outcome = match choice {
                            AcpV1PermissionChoice::Cancel => RequestPermissionOutcome::Cancelled,
                            AcpV1PermissionChoice::Select { option_id } => {
                                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                                    option_id,
                                ))
                            }
                        };
                        responder.respond(RequestPermissionResponse::new(outcome))
                    })
                },
                agent_client_protocol::on_receive_request!(),
            )
            .connect_with(self.transport, async move |connection| {
                let (session, info) = match start_session_on_connection(
                    connection,
                    cwd,
                    cancellation,
                    self.timeouts,
                    self.preset,
                    session_control.as_ref(),
                    read_enabled,
                )
                .await
                {
                    Ok(session) => session,
                    Err(error) => return Ok(Err(error)),
                };
                let mut session = AcpV1Session {
                    session_id: session.session_id().to_string(),
                    session,
                    last_turn_id: 0,
                    timeouts: self.timeouts,
                    ready: true,
                    info,
                    control: session_control,
                    invalid_notification: session_invalid,
                    permission_requested: session_permission,
                    accept_updates,
                };
                Ok(use_session(&mut session).await)
            })
            .await;

        if let Some(control) = &control {
            control.failure()?;
        }
        if invalid_notification.load(Ordering::Acquire) {
            return Err(response_invalid());
        }
        if permission_requested.load(Ordering::Acquire) {
            return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable));
        }

        result.map_err(map_acp_error)?
    }
}

fn development_config(preset: AcpDevelopmentPresetV1) -> AcpAgentConfig {
    AcpAgentConfig::new(NPX_PROGRAM).args(["-y", preset.package_reference()])
}

async fn start_session_on_connection(
    connection: agent_client_protocol::ConnectionTo<agent_client_protocol::Agent>,
    cwd: PathBuf,
    cancellation: CancellationToken,
    timeouts: AcpV1Timeouts,
    preset: Option<AcpDevelopmentPresetV1>,
    control: Option<&AcpV1Control>,
    read_enabled: bool,
) -> Result<
    (
        agent_client_protocol::ActiveSession<'static, agent_client_protocol::Agent>,
        AcpV1SessionInfo,
    ),
    AgentBackendError,
> {
    let initialized = bounded_phase(
        connection
            .send_request(
                InitializeRequest::new(ProtocolVersion::V1).client_capabilities(
                    ClientCapabilities::new()
                        .fs(FileSystemCapabilities::new().read_text_file(read_enabled)),
                ),
            )
            .block_task(),
        &cancellation,
        timeouts.initialization,
    )
    .await?;
    if initialized.protocol_version != ProtocolVersion::V1 {
        return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable));
    }
    if cancellation.is_cancelled() {
        return Err(backend_error(AgentBackendErrorCode::Cancelled));
    }

    let mut info = session_info(&initialized, preset)?;
    if let Some(control) = control {
        control.emit(AcpV1UpdateKind::Session { info: info.clone() })?;
    }
    let session = match bounded_phase(
        connection.build_session(cwd).block_task().start_session(),
        &cancellation,
        timeouts.session_creation,
    )
    .await
    {
        Ok(session) => session,
        Err(error) => {
            if error.code == AgentBackendErrorCode::AuthenticationUnavailable {
                info.authentication_required = true;
                if let Some(control) = control {
                    control.emit(AcpV1UpdateKind::Session { info })?;
                }
            }
            return Err(error);
        }
    };
    info.session_established = true;
    if let Some(modes) = session.modes() {
        info.mode_ids = checked_ids(modes.available_modes.iter().map(|mode| mode.id.to_string()))?;
        let current = modes.current_mode_id.to_string();
        if !info.mode_ids.contains(&current) {
            return Err(response_invalid());
        }
        info.current_mode_id = Some(current);
    }
    apply_config(&mut info, session.config_options().unwrap_or_default())?;
    if let Some(control) = control {
        control.emit(AcpV1UpdateKind::Session { info: info.clone() })?;
    }
    let session_id = session.session_id().to_string();
    validate_session_id(&session_id)?;
    if cancellation.is_cancelled() {
        return Err(backend_error(AgentBackendErrorCode::Cancelled));
    }

    Ok((session, info))
}

async fn run_turn_on_session(
    session: &mut agent_client_protocol::ActiveSession<'_, agent_client_protocol::Agent>,
    prompt: String,
    cancellation: CancellationToken,
    turn_id: u64,
    timeouts: AcpV1Timeouts,
    info: &mut AcpV1SessionInfo,
    mut scope: Option<&mut crate::acp_control::TurnScope>,
) -> Result<AcpV1TurnOutcome, AgentBackendError> {
    let session_id = session.session_id().to_string();
    session.send_prompt(prompt).map_err(map_acp_error)?;
    let connection = session.connection().clone();
    let mut events = Vec::new();
    let mut accepted_content_bytes = 0_usize;
    let mut update_count = 0_usize;
    let mut cancel_sent = false;
    let mut timed_out = false;
    let mut deadline = Instant::now() + timeouts.prompt;

    loop {
        let message = {
            let update = session.read_update().fuse();
            let cancelled = async {
                if cancel_sent {
                    std::future::pending::<()>().await;
                }
                cancellation.cancelled().await;
            }
            .fuse();
            let timer = async_io::Timer::at(deadline).fuse();
            futures::pin_mut!(update, cancelled, timer);
            futures::select_biased! {
                _ = cancelled => None,
                _ = timer => {
                    if cancel_sent {
                        // Abort even if the callback catches this error and keeps waiting.
                        connection.spawn(async { Err(AcpError::internal_error()) })
                            .map_err(map_acp_error)?;
                        return Err(backend_error(AgentBackendErrorCode::TransportUnavailable));
                    }
                    timed_out = true;
                    if let Some(scope) = scope.as_mut() { scope.status = AcpV1TurnStatus::TimedOut; }
                    None
                },
                result = update => Some(result.map_err(map_acp_error)?),
            }
        };
        let Some(message) = message else {
            send_cancel(session, &session_id)?;
            cancel_sent = true;
            deadline = Instant::now() + timeouts.cancellation_grace;
            continue;
        };
        update_count = update_count
            .checked_add(1)
            .filter(|count| *count <= MAX_TURN_UPDATES)
            .ok_or_else(response_invalid)?;

        match message {
            agent_client_protocol::SessionMessage::SessionMessage(dispatch) => {
                let content =
                    project_update(dispatch, &session_id, info, scope.as_deref(), cancel_sent)?;
                let Some(content) = content else {
                    continue;
                };
                accepted_content_bytes = accepted_content_bytes
                    .checked_add(content.len())
                    .filter(|bytes| *bytes <= MAX_TURN_CONTENT_BYTES)
                    .ok_or_else(response_invalid)?;
                let event = AgentEventV1::ContentDelta { content };
                event.validate().map_err(|_| response_invalid())?;
                events.push(event);
            }
            agent_client_protocol::SessionMessage::StopReason(reason) => {
                finish_turn(reason, cancel_sent, &mut events)?;
                if timed_out {
                    connection
                        .spawn(async { Err(AcpError::internal_error()) })
                        .map_err(map_acp_error)?;
                    return Err(backend_error(AgentBackendErrorCode::TransportUnavailable));
                }
                return Ok(AcpV1TurnOutcome {
                    protocol_version: ProtocolVersion::V1.as_u16(),
                    turn_id,
                    session_id,
                    events,
                });
            }
            _ => return Err(response_invalid()),
        }
    }
}

async fn bounded_phase<T>(
    operation: impl std::future::Future<Output = Result<T, AcpError>>,
    cancellation: &CancellationToken,
    timeout: Duration,
) -> Result<T, AgentBackendError> {
    let operation = operation.fuse();
    let cancelled = cancellation.cancelled().fuse();
    let timer = async_io::Timer::after(timeout).fuse();
    futures::pin_mut!(operation, cancelled, timer);
    futures::select_biased! {
        _ = cancelled => Err(backend_error(AgentBackendErrorCode::Cancelled)),
        _ = timer => Err(backend_error(AgentBackendErrorCode::TransportUnavailable)),
        result = operation => result.map_err(map_acp_error),
    }
}

fn send_cancel(
    session: &agent_client_protocol::ActiveSession<'_, agent_client_protocol::Agent>,
    session_id: &str,
) -> Result<(), AgentBackendError> {
    session
        .connection()
        .send_notification(CancelNotification::new(session_id.to_owned()))
        .map_err(map_acp_error)
}

fn checked_ids(values: impl IntoIterator<Item = String>) -> Result<Vec<String>, AgentBackendError> {
    let mut ids = Vec::new();
    for id in values {
        validate_session_id(&id)?;
        if ids.len() >= 64 || ids.contains(&id) {
            return Err(response_invalid());
        }
        ids.push(id);
    }
    Ok(ids)
}

fn session_info(
    initialized: &InitializeResponse,
    preset: Option<AcpDevelopmentPresetV1>,
) -> Result<AcpV1SessionInfo, AgentBackendError> {
    Ok(AcpV1SessionInfo {
        adapter: preset.map(|preset| preset.package_reference().to_owned()),
        session_established: false,
        authentication_required: false,
        auth_method_ids: checked_ids(
            initialized
                .auth_methods
                .iter()
                .map(|method| method.id().to_string()),
        )?,
        load_advertised: initialized.agent_capabilities.load_session,
        resume_advertised: initialized
            .agent_capabilities
            .session_capabilities
            .resume
            .is_some(),
        mode_ids: Vec::new(),
        current_mode_id: None,
        model_ids: Vec::new(),
        current_model_id: None,
        load_enabled: false,
        resume_enabled: false,
        mode_selection_enabled: false,
        model_selection_enabled: false,
    })
}

fn apply_config(
    info: &mut AcpV1SessionInfo,
    options: &[agent_client_protocol::schema::v1::SessionConfigOption],
) -> Result<(), AgentBackendError> {
    if options.len() > 64 {
        return Err(response_invalid());
    }
    for option in options {
        let (ids, current) = match option.category {
            Some(SessionConfigOptionCategory::Model) => {
                (&mut info.model_ids, &mut info.current_model_id)
            }
            Some(SessionConfigOptionCategory::Mode) => {
                (&mut info.mode_ids, &mut info.current_mode_id)
            }
            _ => continue,
        };
        let SessionConfigKind::Select(select) = &option.kind else {
            return Err(response_invalid());
        };
        *ids = match &select.options {
            SessionConfigSelectOptions::Ungrouped(options) => {
                checked_ids(options.iter().map(|option| option.value.to_string()))?
            }
            SessionConfigSelectOptions::Grouped(groups) => checked_ids(
                groups
                    .iter()
                    .flat_map(|group| group.options.iter())
                    .map(|option| option.value.to_string()),
            )?,
            _ => return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
        };
        let value = select.current_value.to_string();
        if !ids.contains(&value) {
            return Err(response_invalid());
        }
        *current = Some(value);
    }
    Ok(())
}

fn prepare_permission(
    control: &AcpV1Control,
    request: RequestPermissionRequest,
    timeout: Duration,
) -> Result<crate::acp_control::PermissionWait, AgentBackendError> {
    let session_id = request.session_id.to_string();
    validate_session_id(&session_id)?;
    let tool_call_id = request.tool_call.tool_call_id.to_string();
    validate_session_id(&tool_call_id)?;
    let title = request
        .tool_call
        .fields
        .title
        .unwrap_or_else(|| "Tool permission".to_owned());
    validate_label(&title)?;
    if request.options.is_empty() || request.options.len() > 16 {
        return Err(response_invalid());
    }
    let mut options = Vec::new();
    for option in request.options {
        let option_id = option.option_id.to_string();
        validate_session_id(&option_id)?;
        validate_label(&option.name)?;
        if options
            .iter()
            .any(|prior: &AcpV1PermissionOption| prior.option_id == option_id)
        {
            return Err(response_invalid());
        }
        let kind = match option.kind {
            PermissionOptionKind::AllowOnce => AcpV1PermissionKind::AllowOnce,
            PermissionOptionKind::RejectOnce => AcpV1PermissionKind::RejectOnce,
            PermissionOptionKind::AllowAlways => AcpV1PermissionKind::AllowAlways,
            PermissionOptionKind::RejectAlways => AcpV1PermissionKind::RejectAlways,
            _ => return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
        };
        options.push(AcpV1PermissionOption {
            option_id,
            name: option.name,
            kind,
            enabled: matches!(
                kind,
                AcpV1PermissionKind::AllowOnce | AcpV1PermissionKind::RejectOnce
            ),
        });
    }
    control.begin_permission(&session_id, tool_call_id, title, options, timeout)
}

fn validate_label(label: &str) -> Result<(), AgentBackendError> {
    if label.is_empty() || label.len() > 4_096 || label.chars().any(char::is_control) {
        return Err(response_invalid());
    }
    Ok(())
}

fn tool_status(status: ToolCallStatus) -> Result<AcpV1ActivityStatus, AgentBackendError> {
    match status {
        ToolCallStatus::Pending => Ok(AcpV1ActivityStatus::Pending),
        ToolCallStatus::InProgress => Ok(AcpV1ActivityStatus::InProgress),
        ToolCallStatus::Completed => Ok(AcpV1ActivityStatus::Completed),
        ToolCallStatus::Failed => Ok(AcpV1ActivityStatus::Failed),
        _ => Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
    }
}

fn project_update(
    dispatch: Dispatch,
    session_id: &str,
    info: &mut AcpV1SessionInfo,
    scope: Option<&crate::acp_control::TurnScope>,
    cancelled: bool,
) -> Result<Option<String>, AgentBackendError> {
    let Dispatch::Notification(notification) = &dispatch else {
        return Err(response_invalid());
    };
    let notification =
        SessionNotification::parse_message(notification.method(), notification.params())
            .map_err(|_| response_invalid())?;
    if notification.session_id.to_string() != session_id {
        return Err(response_invalid());
    }
    if serde_json::to_vec(&notification.update)
        .map_err(|_| response_invalid())?
        .len()
        > MAX_CONTENT_DELTA_BYTES + 4_096
    {
        return Err(response_invalid());
    }
    let emit = |kind| {
        if let Some(scope) = scope {
            scope.control.emit(kind)
        } else {
            Ok(())
        }
    };
    let binding = scope.map(|scope| scope.binding.clone());
    match notification.update {
        SessionUpdate::AgentMessageChunk(_) => {
            let content = text_delta(dispatch, session_id)?;
            if cancelled {
                return Ok(None);
            }
            if let Some(binding) = binding {
                emit(AcpV1UpdateKind::Text {
                    binding,
                    content: content.clone(),
                })?;
            }
            Ok(Some(content))
        }
        // Hidden reasoning is neither streamed, persisted nor copied into outcomes.
        SessionUpdate::AgentThoughtChunk(_) => Ok(None),
        SessionUpdate::Plan(plan) => {
            if plan.entries.len() > 64 {
                return Err(response_invalid());
            }
            let mut entries = Vec::new();
            for entry in plan.entries {
                validate_label(&entry.content)?;
                let status = match entry.status {
                    PlanEntryStatus::Pending => AcpV1ActivityStatus::Pending,
                    PlanEntryStatus::InProgress => AcpV1ActivityStatus::InProgress,
                    PlanEntryStatus::Completed => AcpV1ActivityStatus::Completed,
                    _ => return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
                };
                entries.push(AcpV1PlanEntry {
                    content: entry.content,
                    status,
                });
            }
            if !cancelled && let Some(binding) = binding {
                emit(AcpV1UpdateKind::Plan { binding, entries })?;
            }
            Ok(None)
        }
        SessionUpdate::ToolCall(tool) => {
            let id = tool.tool_call_id.to_string();
            validate_session_id(&id)?;
            validate_label(&tool.title)?;
            let status = tool_status(tool.status)?;
            if !cancelled && let Some(binding) = binding {
                emit(AcpV1UpdateKind::Tool {
                    binding,
                    tool_call_id: id,
                    title: Some(tool.title),
                    status: Some(status),
                })?;
            }
            Ok(None)
        }
        SessionUpdate::ToolCallUpdate(tool) => {
            let id = tool.tool_call_id.to_string();
            validate_session_id(&id)?;
            if let Some(title) = &tool.fields.title {
                validate_label(title)?;
            }
            let status = tool.fields.status.map(tool_status).transpose()?;
            if !cancelled && let Some(binding) = binding {
                emit(AcpV1UpdateKind::Tool {
                    binding,
                    tool_call_id: id,
                    title: tool.fields.title,
                    status,
                })?;
            }
            Ok(None)
        }
        SessionUpdate::CurrentModeUpdate(update) => {
            let current = update.current_mode_id.to_string();
            if !info.mode_ids.contains(&current) {
                return Err(response_invalid());
            }
            info.current_mode_id = Some(current);
            emit(AcpV1UpdateKind::Session { info: info.clone() })?;
            Ok(None)
        }
        SessionUpdate::ConfigOptionUpdate(update) => {
            apply_config(info, &update.config_options)?;
            emit(AcpV1UpdateKind::Session { info: info.clone() })?;
            Ok(None)
        }
        _ => Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
    }
}

fn text_delta(dispatch: Dispatch, expected_session_id: &str) -> Result<String, AgentBackendError> {
    let notification = match dispatch {
        Dispatch::Notification(notification) => {
            SessionNotification::parse_message(notification.method(), notification.params())
                .map_err(|_| response_invalid())?
        }
        Dispatch::Request(_, _) | Dispatch::Response(_, _) => return Err(response_invalid()),
    };
    if notification.session_id.to_string() != expected_session_id {
        return Err(response_invalid());
    }

    match notification.update {
        SessionUpdate::AgentMessageChunk(ContentChunk {
            content: ContentBlock::Text(text),
            ..
        }) if !text.text.is_empty() && text.text.len() <= MAX_CONTENT_DELTA_BYTES => Ok(text.text),
        SessionUpdate::AgentMessageChunk(_) => Err(response_invalid()),
        _ => Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
    }
}

fn finish_turn(
    reason: StopReason,
    cancel_sent: bool,
    events: &mut Vec<AgentEventV1>,
) -> Result<(), AgentBackendError> {
    let event = match reason {
        StopReason::EndTurn if !cancel_sent => AgentEventV1::Completed {
            finish_reason: AgentFinishReason::Stop,
        },
        StopReason::MaxTokens | StopReason::MaxTurnRequests if !cancel_sent => {
            AgentEventV1::Completed {
                finish_reason: AgentFinishReason::Length,
            }
        }
        StopReason::Cancelled if cancel_sent => AgentEventV1::Cancelled,
        StopReason::Refusal if !cancel_sent => {
            return Err(backend_error(AgentBackendErrorCode::ProviderRejected));
        }
        StopReason::Cancelled
        | StopReason::EndTurn
        | StopReason::MaxTokens
        | StopReason::MaxTurnRequests
        | StopReason::Refusal => return Err(response_invalid()),
        _ => return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable)),
    };
    events.push(event);
    Ok(())
}

fn validate_cwd(cwd: &Path) -> Result<(), AgentBackendError> {
    let cwd_bytes = cwd.as_os_str().as_encoded_bytes();
    if !cwd.is_absolute()
        || cwd_bytes.is_empty()
        || cwd_bytes.len() > MAX_CWD_BYTES
        || cwd_bytes.contains(&0)
    {
        return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
    }
    Ok(())
}

fn validate_prompt(prompt: &str) -> Result<(), AgentBackendError> {
    if prompt.is_empty() || prompt.len() > MAX_PROMPT_BYTES {
        return Err(backend_error(AgentBackendErrorCode::InvalidRequest));
    }
    Ok(())
}

fn validate_session_id(session_id: &str) -> Result<(), AgentBackendError> {
    if session_id.is_empty()
        || session_id.len() > MAX_SESSION_ID_BYTES
        || session_id.chars().any(char::is_control)
    {
        return Err(response_invalid());
    }
    Ok(())
}

fn map_acp_error(error: AcpError) -> AgentBackendError {
    let code = match error.code {
        AcpErrorCode::AuthRequired => AgentBackendErrorCode::AuthenticationUnavailable,
        AcpErrorCode::MethodNotFound => AgentBackendErrorCode::CapabilityUnavailable,
        AcpErrorCode::ParseError | AcpErrorCode::InvalidRequest | AcpErrorCode::InvalidParams => {
            AgentBackendErrorCode::ResponseInvalid
        }
        // ACP permits request cancellation for peer shutdown and resource limits too.
        // Only a confirmed StopReason::Cancelled after our session/cancel is user cancellation.
        AcpErrorCode::RequestCancelled => AgentBackendErrorCode::TransportUnavailable,
        AcpErrorCode::ResourceNotFound => AgentBackendErrorCode::ProviderRejected,
        AcpErrorCode::InternalError | AcpErrorCode::Other(_) => {
            AgentBackendErrorCode::TransportUnavailable
        }
        _ => AgentBackendErrorCode::TransportUnavailable,
    };
    backend_error(code)
}

const fn backend_error(code: AgentBackendErrorCode) -> AgentBackendError {
    AgentBackendError::new(code, None)
}

const fn response_invalid() -> AgentBackendError {
    backend_error(AgentBackendErrorCode::ResponseInvalid)
}

#[cfg(test)]
mod tests {
    use agent_client_protocol::schema::v1::{
        InitializeResponse, NewSessionRequest, NewSessionResponse, PermissionOption,
        PermissionOptionKind, PromptRequest, PromptResponse, TextContent, ToolCallUpdate,
        ToolCallUpdateFields,
    };
    use agent_client_protocol::{Agent, ConnectionTo, Responder};
    use qiongli_project::ProjectId;

    use super::*;
    use crate::{
        BackendId, OrchestrationExecutionMode, OrchestrationProfileV1, OrchestrationRunStatus,
        RunId,
    };

    #[derive(Clone, Copy)]
    enum FixtureBehavior {
        Normal,
        Permission,
        UnknownSession,
        UnknownSessionWithoutStop,
        EarlyOwnedUpdate,
        EarlyUnknownUpdate,
        RepeatedTurns,
        MalformedSecondTurn,
        Silent,
        LateUpdate,
        SilentInitialize,
        SilentSession,
        CancelAcknowledged,
        Disconnected,
        PermissionInteractive,
        PermissionWaiting,
        PermissionWrongSession,
        PermissionExit,
        PermissionPrematureStop,
        RichUpdates,
        AuthRequired,
    }

    fn fixture_agent(behavior: FixtureBehavior) -> impl ConnectTo<Client> {
        let pending_prompt = Arc::new(std::sync::Mutex::new(None::<Responder<PromptResponse>>));
        let cancel_prompt = Arc::clone(&pending_prompt);
        let mut initialize_count = 0;
        let mut session_count = 0;
        let mut prompt_count = 0;
        Agent
            .builder()
            .on_receive_notification(
                async move |request: CancelNotification, _connection| {
                    assert_eq!(request.session_id.to_string(), "fixture-session");
                    if let Some(responder) = cancel_prompt.lock().unwrap().take() {
                        responder.respond(PromptResponse::new(StopReason::Cancelled))?;
                    }
                    Ok(())
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .on_receive_request(
                async move |request: InitializeRequest,
                            responder: Responder<InitializeResponse>,
                            _connection: ConnectionTo<Client>| {
                    initialize_count += 1;
                    assert_eq!(initialize_count, 1, "connection must initialize only once");
                    assert_eq!(request.protocol_version, ProtocolVersion::V1);
                    if matches!(behavior, FixtureBehavior::SilentInitialize) {
                        return _connection.spawn(async move {
                            std::future::pending::<()>().await;
                            responder.respond(InitializeResponse::new(ProtocolVersion::V1))
                        });
                    }
                    let mut response = InitializeResponse::new(ProtocolVersion::V1);
                    if matches!(behavior, FixtureBehavior::RichUpdates) {
                        response.auth_methods = serde_json::from_value(serde_json::json!([
                            {"id": "fixture-auth", "name": "Fixture authentication"}
                        ])).unwrap();
                        response.agent_capabilities.load_session = true;
                        response.agent_capabilities.session_capabilities.resume = Some(Default::default());
                    }
                    responder.respond(response)
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |_request: NewSessionRequest,
                            responder: Responder<NewSessionResponse>,
                            connection: ConnectionTo<Client>| {
                    session_count += 1;
                    assert_eq!(session_count, 1, "prompts must retain one provider session");
                    if matches!(behavior, FixtureBehavior::SilentSession) {
                        return connection.spawn(async move {
                            std::future::pending::<()>().await;
                            responder.respond(NewSessionResponse::new("fixture-session"))
                        });
                    }
                    if matches!(behavior, FixtureBehavior::AuthRequired) {
                        return responder.respond_with_error(AcpError::auth_required());
                    }
                    let mut response = NewSessionResponse::new("fixture-session");
                    if matches!(behavior, FixtureBehavior::RichUpdates) {
                        response = serde_json::from_value(serde_json::json!({
                            "sessionId": "fixture-session",
                            "modes": {"currentModeId": "ask", "availableModes": [{"id": "ask", "name": "Ask"}, {"id": "plan", "name": "Plan"}]},
                            "configOptions": [{"id": "model", "name": "Model", "category": "model", "type": "select", "currentValue": "model-a", "options": [{"value": "model-a", "name": "Model A"}]}]
                        })).unwrap();
                    }
                    responder.respond(response)?;
                    if matches!(
                        behavior,
                        FixtureBehavior::EarlyOwnedUpdate | FixtureBehavior::EarlyUnknownUpdate
                    ) {
                        let session_id = if matches!(behavior, FixtureBehavior::EarlyOwnedUpdate) {
                            "fixture-session"
                        } else {
                            "unknown-session"
                        };
                        connection.send_notification(SessionNotification::new(
                            session_id,
                            SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                ContentBlock::Text(TextContent::new("early reply")),
                            )),
                        ))?;
                    }
                    Ok(())
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |request: PromptRequest,
                            responder: Responder<PromptResponse>,
                            connection: ConnectionTo<Client>| {
                    if matches!(behavior, FixtureBehavior::Disconnected) {
                        return connection.spawn(async { Err(AcpError::internal_error()) });
                    }
                    if matches!(behavior, FixtureBehavior::CancelAcknowledged) {
                        *pending_prompt.lock().unwrap() = Some(responder);
                        return Ok(());
                    }
                    prompt_count += 1;
                    assert_eq!(request.session_id.to_string(), "fixture-session");
                    if matches!(
                        behavior,
                        FixtureBehavior::RepeatedTurns | FixtureBehavior::MalformedSecondTurn
                    ) {
                        let [ContentBlock::Text(text)] = request.prompt.as_slice() else {
                            panic!("expected one text prompt");
                        };
                        assert_eq!(text.text, format!("prompt {prompt_count}"));
                    }
                    if matches!(
                        behavior,
                        FixtureBehavior::UnknownSession
                            | FixtureBehavior::UnknownSessionWithoutStop
                    ) {
                        for session_id in ["fixture-session", "unknown-session"] {
                            connection.send_notification(SessionNotification::new(
                                session_id,
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new("must not escape")),
                                )),
                            ))?;
                        }
                    }
                    if matches!(
                        behavior,
                        FixtureBehavior::UnknownSessionWithoutStop | FixtureBehavior::Silent
                    ) {
                        return connection.spawn(async move {
                            std::future::pending::<()>().await;
                            responder.respond(PromptResponse::new(StopReason::EndTurn))
                        });
                    }
                    if matches!(behavior, FixtureBehavior::PermissionExit | FixtureBehavior::PermissionPrematureStop) {
                        connection.send_request(RequestPermissionRequest::new(
                            request.session_id.clone(), ToolCallUpdate::new("fixture-tool", ToolCallUpdateFields::new()),
                            vec![PermissionOption::new("allow", "Allow", PermissionOptionKind::AllowOnce)],
                        )).on_receiving_result(async |_result| Ok(()))?;
                        if matches!(behavior, FixtureBehavior::PermissionPrematureStop) {
                            return responder.respond(PromptResponse::new(StopReason::EndTurn));
                        }
                        return connection.spawn(async move {
                            let _responder = responder;
                            async_io::Timer::after(Duration::from_millis(20)).await;
                            Err(AcpError::internal_error())
                        });
                    }
                    if matches!(behavior, FixtureBehavior::Permission | FixtureBehavior::PermissionInteractive
                        | FixtureBehavior::PermissionWaiting | FixtureBehavior::PermissionWrongSession
                        | FixtureBehavior::PermissionExit | FixtureBehavior::PermissionPrematureStop) {
                        let session_id = if matches!(behavior, FixtureBehavior::PermissionWrongSession) {
                            "unowned-session".into()
                        } else { request.session_id };
                        let update_session_id = session_id.clone();
                        connection
                            .send_request(RequestPermissionRequest::new(
                                session_id,
                                ToolCallUpdate::new("fixture-tool", ToolCallUpdateFields::new()),
                                vec![
                                    PermissionOption::new("allow", "Allow once", PermissionOptionKind::AllowOnce),
                                    PermissionOption::new("deny", "Deny once", PermissionOptionKind::RejectOnce),
                                    PermissionOption::new("always", "Allow always", PermissionOptionKind::AllowAlways),
                                ],
                            ))
                            .on_receiving_result(async move |response| {
                                let response = response?;
                                if matches!(behavior, FixtureBehavior::PermissionInteractive) {
                                    assert!(matches!(&response.outcome, RequestPermissionOutcome::Selected(choice) if choice.option_id.to_string() == "allow" || choice.option_id.to_string() == "deny"));
                                } else { assert_eq!(response.outcome, RequestPermissionOutcome::Cancelled); }
                                if matches!(behavior, FixtureBehavior::PermissionWaiting) {
                                    return responder.respond(PromptResponse::new(StopReason::Cancelled));
                                }
                                connection.send_notification(SessionNotification::new(
                                    update_session_id,
                                    SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                        ContentBlock::Text(TextContent::new("must not escape")),
                                    )),
                                ))?;
                                responder.respond(PromptResponse::new(StopReason::EndTurn))
                            })
                    } else {
                        if matches!(behavior, FixtureBehavior::RichUpdates) {
                            for update in [
                                serde_json::json!({"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"hidden-reasoning-canary"}}),
                                serde_json::json!({"sessionUpdate":"plan","entries":[{"content":"Compare sources","priority":"high","status":"in_progress"}]}),
                                serde_json::json!({"sessionUpdate":"tool_call","toolCallId":"read-source","title":"Read source","status":"pending","rawInput":{"secret":"raw-input-canary"}}),
                                serde_json::json!({"sessionUpdate":"tool_call_update","toolCallId":"read-source","status":"completed","rawOutput":"raw-output-canary"}),
                                serde_json::json!({"sessionUpdate":"current_mode_update","currentModeId":"plan"}),
                            ] {
                                connection.send_notification(SessionNotification::new(request.session_id.clone(), serde_json::from_value(update).unwrap()))?;
                            }
                        }
                        let reply = match behavior {
                            FixtureBehavior::MalformedSecondTurn if prompt_count == 2 => {
                                String::new()
                            }
                            FixtureBehavior::RepeatedTurns
                            | FixtureBehavior::MalformedSecondTurn => {
                                format!("reply {prompt_count}")
                            }
                            _ => "fixture reply".to_string(),
                        };
                        connection.send_notification(SessionNotification::new(
                            request.session_id.clone(),
                            SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                ContentBlock::Text(TextContent::new(reply)),
                            )),
                        ))?;
                        responder.respond(PromptResponse::new(StopReason::EndTurn))?;
                        if matches!(behavior, FixtureBehavior::LateUpdate) {
                            connection.send_notification(SessionNotification::new(
                                request.session_id,
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new("late previous-turn text")),
                                )),
                            ))?;
                        }
                        Ok(())
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
    }

    fn running_all_chat_state() -> AllChatStateV1 {
        let profile = OrchestrationProfileV1::try_new(
            "acp-projection-test",
            OrchestrationExecutionMode::Solo,
            BackendId::parse("codex-acp").unwrap(),
            None,
            None,
            1,
            true,
        )
        .unwrap();
        let mut state = AllChatStateV1::try_new(
            RunId::parse(format!("run_{}", "3".repeat(32))).unwrap(),
            ProjectId::parse(format!("prj_{}", "4".repeat(32))).unwrap(),
            11,
            &profile,
        )
        .unwrap();
        state
            .append_event(0, 1, AllChatEventKindV1::RunStarted {})
            .unwrap();
        state
    }

    #[test]
    fn fixed_development_presets_and_v1_turn_share_the_production_boundary() {
        futures::executor::block_on(async {
            let codex = development_config(AcpDevelopmentPresetV1::Codex);
            assert_eq!(codex.command(), Path::new("npx"));
            assert_eq!(
                codex.arguments(),
                ["-y", "@agentclientprotocol/codex-acp@1.9.0"]
            );
            assert!(codex.environment().is_empty());

            let claude = development_config(AcpDevelopmentPresetV1::Claude);
            assert_eq!(claude.command(), Path::new("npx"));
            assert_eq!(
                claude.arguments(),
                ["-y", "@agentclientprotocol/claude-agent-acp@0.74.0"]
            );
            assert!(claude.environment().is_empty());

            let outcome = AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Normal))
                .run_turn(
                    std::env::current_dir().expect("fixture cwd"),
                    "fixture prompt",
                    CancellationToken::new(),
                )
                .await
                .expect("credential-free ACP v1 fixture should complete");

            assert_eq!(outcome.protocol_version(), 1);
            assert_eq!(outcome.session_id(), "fixture-session");
            assert_eq!(outcome.events().len(), 2);
            assert!(matches!(
                &outcome.events()[0],
                AgentEventV1::ContentDelta { content } if content == "fixture reply"
            ));
            assert!(matches!(
                outcome.events()[1],
                AgentEventV1::Completed {
                    finish_reason: AgentFinishReason::Stop,
                }
            ));

            let permission_error =
                match AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Permission))
                    .run_turn(
                        std::env::current_dir().expect("fixture cwd"),
                        "permission prompt",
                        CancellationToken::new(),
                    )
                    .await
                {
                    Ok(_) => panic!("content after a permission request must not escape"),
                    Err(error) => error,
                };
            assert_eq!(
                permission_error.code,
                AgentBackendErrorCode::CapabilityUnavailable
            );

            let mut terminal_events = Vec::new();
            assert_eq!(
                finish_turn(StopReason::Cancelled, false, &mut terminal_events)
                    .expect_err("an unsolicited cancelled reason must fail closed")
                    .code,
                AgentBackendErrorCode::ResponseInvalid
            );
            assert_eq!(
                finish_turn(StopReason::EndTurn, true, &mut terminal_events)
                    .expect_err("completion after a sent cancellation must fail closed")
                    .code,
                AgentBackendErrorCode::ResponseInvalid
            );
            assert!(terminal_events.is_empty());
            assert_eq!(
                map_acp_error(AcpError::request_cancelled()).code,
                AgentBackendErrorCode::TransportUnavailable
            );
        });
    }

    #[test]
    fn connection_rejects_unknown_sessions_without_losing_early_owned_updates() {
        futures::executor::block_on(async {
            let outcome =
                AcpV1Client::from_transport(fixture_agent(FixtureBehavior::EarlyOwnedUpdate))
                    .run_turn(
                        std::env::current_dir().unwrap(),
                        "prompt",
                        CancellationToken::new(),
                    )
                    .await
                    .unwrap_or_else(|error| {
                        panic!("owned early update rejected: {:?}", error.code)
                    });
            assert_eq!(outcome.events().len(), 3);
            assert!(
                matches!(&outcome.events()[0], AgentEventV1::ContentDelta { content } if content == "early reply")
            );

            for behavior in [
                FixtureBehavior::UnknownSession,
                FixtureBehavior::EarlyUnknownUpdate,
                FixtureBehavior::UnknownSessionWithoutStop,
            ] {
                let (send, receive) = std::sync::mpsc::sync_channel(1);
                let worker = std::thread::spawn(move || {
                    let result = futures::executor::block_on(
                        AcpV1Client::from_transport(fixture_agent(behavior)).run_turn(
                            std::env::current_dir().unwrap(),
                            "prompt",
                            CancellationToken::new(),
                        ),
                    );
                    send.send(result.map(|_| ()).map_err(|error| error.code))
                        .unwrap();
                });
                // Bound a broken error-wakeup path so the regression cannot hang CI.
                let result = receive
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .expect("invalid notification must stop the turn without waiting for EndTurn");
                worker.join().unwrap();
                assert_eq!(result, Err(AgentBackendErrorCode::ResponseInvalid));
            }
        });
    }

    #[test]
    fn retained_session_keeps_two_prompt_outcomes_independent() {
        futures::executor::block_on(async {
            let (first, second) =
                AcpV1Client::from_transport(fixture_agent(FixtureBehavior::RepeatedTurns))
                    .with_session(
                        std::env::current_dir().unwrap(),
                        CancellationToken::new(),
                        async |session| {
                            assert_eq!(session.session_id(), "fixture-session");
                            assert_eq!(
                                session
                                    .run_turn("", CancellationToken::new())
                                    .await
                                    .err()
                                    .unwrap()
                                    .code,
                                AgentBackendErrorCode::InvalidRequest
                            );
                            let cancelled = CancellationToken::new();
                            cancelled.cancel();
                            assert_eq!(
                                session
                                    .run_turn("not sent", cancelled)
                                    .await
                                    .err()
                                    .unwrap()
                                    .code,
                                AgentBackendErrorCode::Cancelled
                            );
                            let first = session
                                .run_turn("prompt 1", CancellationToken::new())
                                .await?;
                            let saved = first.clone();
                            let second = session
                                .run_turn("prompt 2", CancellationToken::new())
                                .await?;
                            assert!(first == saved);
                            // An exhausted local turn counter must not wrap or send another prompt.
                            session.last_turn_id = 9_007_199_254_740_991;
                            assert_eq!(
                                session
                                    .run_turn("not sent", CancellationToken::new())
                                    .await
                                    .err()
                                    .unwrap()
                                    .code,
                                AgentBackendErrorCode::InvalidRequest
                            );
                            Ok((first, second))
                        },
                    )
                    .await
                    .unwrap();
            assert_eq!((first.turn_id(), second.turn_id()), (1, 2));
            assert_eq!(first.session_id(), second.session_id());
            for (outcome, reply) in [(&first, "reply 1"), (&second, "reply 2")] {
                assert_eq!(outcome.events().len(), 2);
                assert!(
                    matches!(&outcome.events()[0], AgentEventV1::ContentDelta { content } if content == reply)
                );
                assert!(matches!(
                    outcome.events()[1],
                    AgentEventV1::Completed {
                        finish_reason: AgentFinishReason::Stop
                    }
                ));
            }
            let mut state = running_all_chat_state();
            first.project_first_coordinator_turn(&mut state, 1).unwrap();
            let before = state.clone();
            let generation = state.generation();
            assert_eq!(
                second.project_first_coordinator_turn(&mut state, generation),
                Err(AllChatStateError::InvalidEvent)
            );
            assert_eq!(state, before);
            assert_eq!(state.status(), OrchestrationRunStatus::Running);
        });
    }

    #[test]
    fn failed_or_abandoned_turn_retires_the_retained_session() {
        futures::executor::block_on(async {
            AcpV1Client::from_transport(fixture_agent(FixtureBehavior::MalformedSecondTurn))
                .with_session(
                    std::env::current_dir().unwrap(),
                    CancellationToken::new(),
                    async |session| {
                        let first = session
                            .run_turn("prompt 1", CancellationToken::new())
                            .await?;
                        let saved = first.clone();
                        assert_eq!(
                            session
                                .run_turn("prompt 2", CancellationToken::new())
                                .await
                                .err()
                                .unwrap()
                                .code,
                            AgentBackendErrorCode::ResponseInvalid
                        );
                        assert_eq!(
                            session
                                .run_turn("not sent", CancellationToken::new())
                                .await
                                .err()
                                .unwrap()
                                .code,
                            AgentBackendErrorCode::TransportUnavailable
                        );
                        assert!(first == saved);
                        Ok(())
                    },
                )
                .await
                .unwrap();
            AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Silent))
                .with_session(
                    std::env::current_dir().unwrap(),
                    CancellationToken::new(),
                    async |session| {
                        let mut turn =
                            Box::pin(session.run_turn("abandoned", CancellationToken::new()));
                        assert!(futures::poll!(&mut turn).is_pending());
                        drop(turn);
                        assert_eq!(
                            session
                                .run_turn("not sent", CancellationToken::new())
                                .await
                                .err()
                                .unwrap()
                                .code,
                            AgentBackendErrorCode::TransportUnavailable
                        );
                        Ok(())
                    },
                )
                .await
                .unwrap();
        });
    }

    #[test]
    fn retained_session_rejects_updates_while_idle() {
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let result: Result<(), AgentBackendError> = futures::executor::block_on(
                AcpV1Client::from_transport(fixture_agent(FixtureBehavior::LateUpdate))
                    .with_session(
                        std::env::current_dir().unwrap(),
                        CancellationToken::new(),
                        async |session| {
                            session.run_turn("prompt", CancellationToken::new()).await?;
                            std::future::pending().await
                        },
                    ),
            );
            send.send(result.map_err(|error| error.code)).unwrap();
        });
        let result = receive
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("late idle update must abort the scoped runner");
        worker.join().unwrap();
        assert_eq!(result, Err(AgentBackendErrorCode::ResponseInvalid));
    }

    fn control() -> AcpV1Control {
        AcpV1Control::new(
            RunId::parse(format!("run_{}", "a".repeat(32))).unwrap(),
            OrchestrationRole::Primary,
        )
        .unwrap()
    }

    #[test]
    fn read_view_is_exact_bounded_and_turn_owned_over_acp() {
        use agent_client_protocol::UntypedMessage;
        use serde_json::json;
        const PATH: &str = "/qiongli-context/paper-a.txt";
        const TEXT: &str = "方法 α\r\n结论 β\n";
        futures::executor::block_on(async {
            let base = json!({"sessionId":"reader", "path":PATH});
            let mut cases = vec![("fs/read_text_file", base.clone(), Some(TEXT.to_owned()), 16)];
            cases.push((
                "fs/read_text_file",
                json!({"sessionId":"reader", "path":PATH, "line":2, "limit":1}),
                Some("结论 β\n".to_owned()),
                1,
            ));
            cases.push((
                "fs/read_text_file",
                json!({"sessionId":"reader", "path":PATH, "line":1, "limit":2000}),
                Some(TEXT.to_owned()),
                1,
            ));
            for (field, value) in [
                ("line", json!(0)),
                ("line", json!(-1)),
                ("line", json!("2")),
                ("line", json!(4294967296_u64)),
                ("line", json!(3)),
                ("limit", json!(0)),
                ("limit", json!(true)),
                ("sessionId", json!("other")),
                ("path", json!("/etc/passwd")),
                ("path", json!("/qiongli-context/./paper-a.txt")),
                ("path", json!("/qiongli-context//paper-a.txt")),
                ("path", json!("/qiongli-context/../paper-a.txt")),
                ("path", json!("qiongli-context/paper-a.txt")),
                ("unknown", json!(true)),
                ("_meta", json!("x".repeat(8192))),
            ] {
                let mut params = base.clone();
                params[field] = value;
                cases.push(("fs/read_text_file", params, None, 1));
            }
            for method in [
                "fs/write_text_file",
                "terminal/create",
                "session/request_permission",
                "unknown/tool",
            ] {
                cases.push((method, base.clone(), None, 1));
            }
            // A seventeenth read must retire the turn, even if the Agent ignores its error.
            cases.push(("fs/read_text_file", base, None, 17));
            for (method, params, expected, reads) in cases {
                let answer = expected.clone();
                let agent = Agent
                    .builder()
                    .on_receive_request(
                        async |request: InitializeRequest,
                               responder: Responder<InitializeResponse>,
                               _: ConnectionTo<Client>| {
                            assert!(request.client_capabilities.fs.read_text_file);
                            assert!(!request.client_capabilities.fs.write_text_file);
                            assert!(!request.client_capabilities.terminal);
                            responder.respond(InitializeResponse::new(ProtocolVersion::V1))
                        },
                        agent_client_protocol::on_receive_request!(),
                    )
                    .on_receive_request(
                        async |_: NewSessionRequest,
                               responder: Responder<NewSessionResponse>,
                               _: ConnectionTo<Client>| {
                            responder.respond(NewSessionResponse::new("reader"))
                        },
                        agent_client_protocol::on_receive_request!(),
                    )
                    .on_receive_request(
                        async move |request: PromptRequest,
                                    responder: Responder<PromptResponse>,
                                    connection: ConnectionTo<Client>| {
                            let params = params.clone();
                            let answer = answer.clone();
                            connection.clone().spawn(async move {
                                for _ in 0..reads {
                                    let response = connection
                                        .send_request(UntypedMessage::new(method, params.clone())?)
                                        .block_task()
                                        .await;
                                    if let Some(answer) = &answer {
                                        assert_eq!(response.unwrap()["content"], *answer);
                                    }
                                }
                                connection.send_notification(SessionNotification::new(
                                    request.session_id,
                                    SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                        ContentBlock::Text(TextContent::new("done")),
                                    )),
                                ))?;
                                responder.respond(PromptResponse::new(StopReason::EndTurn))
                            })
                        },
                        agent_client_protocol::on_receive_request!(),
                    );
                let result = AcpV1Client::from_transport(agent)
                    .with_control(control())
                    .with_read_view(vec![(PATH.to_owned(), TEXT.to_owned())])
                    .unwrap()
                    .with_timeouts(AcpV1Timeouts {
                        prompt: Duration::from_secs(1),
                        ..Default::default()
                    })
                    .unwrap()
                    .with_session(
                        std::env::current_dir().unwrap(),
                        CancellationToken::new(),
                        async |session| {
                            for turn in 1..=2 {
                                assert_eq!(
                                    session
                                        .run_turn("read", CancellationToken::new())
                                        .await?
                                        .turn_id(),
                                    turn
                                );
                            }
                            Ok(())
                        },
                    )
                    .await;
                if expected.is_some() {
                    result.unwrap();
                } else {
                    assert_eq!(
                        result.unwrap_err().code,
                        AgentBackendErrorCode::ResponseInvalid,
                        "{method}"
                    );
                }
            }

            for files in [
                vec![],
                vec![(PATH.into(), String::new())],
                vec![(PATH.into(), "x".repeat(65537))],
                vec![(PATH.into(), TEXT.into()); 2],
                vec![("/outside/file".into(), TEXT.into())],
            ] {
                let rejected_control = control();
                assert!(
                    AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Normal))
                        .with_control(rejected_control.clone())
                        .with_read_view(files)
                        .is_err()
                );
                assert!(rejected_control.next_update().await.is_none());
            }
            let result = AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Normal))
                .with_read_view(vec![(PATH.into(), TEXT.into())])
                .unwrap()
                .run_turn(
                    std::env::current_dir().unwrap(),
                    "no control",
                    CancellationToken::new(),
                )
                .await;
            assert_eq!(
                result.err().unwrap().code,
                AgentBackendErrorCode::InvalidRequest
            );
        });
    }

    #[test]
    fn read_view_rejects_startup_idle_and_cancelled_protocol_requests() {
        for phase in ["startup", "idle", "cancelled"] {
            // Watchdog also detects a lost dispatch wakeup while the callback is idle.
            let (send, receive) = std::sync::mpsc::channel();
            let worker = std::thread::spawn(move || {
                let result: Result<(), AgentBackendError> = futures::executor::block_on(async {
                    let token = CancellationToken::new();
                    let agent_token = token.clone();
                    let agent = Agent.builder()
                        .on_receive_request(async |_: InitializeRequest, responder: Responder<InitializeResponse>, _: ConnectionTo<Client>| {
                            responder.respond(InitializeResponse::new(ProtocolVersion::V1))
                        }, agent_client_protocol::on_receive_request!())
                        .on_receive_request(async move |_: NewSessionRequest, responder: Responder<NewSessionResponse>, connection: ConnectionTo<Client>| {
                            if phase == "startup" {
                                return connection.clone().spawn(async move {
                                    let _ = connection.send_request(ReadTextFileRequest::new("reader", "/qiongli-context/paper.txt")).block_task().await;
                                    responder.respond(NewSessionResponse::new("reader"))
                                });
                            }
                            responder.respond(NewSessionResponse::new("reader"))
                        }, agent_client_protocol::on_receive_request!())
                        .on_receive_request(async move |_: PromptRequest, responder: Responder<PromptResponse>, connection: ConnectionTo<Client>| {
                            if phase == "cancelled" { agent_token.cancel(); }
                            else { responder.respond(PromptResponse::new(StopReason::EndTurn))?; }
                            connection.clone().spawn(async move {
                                let _ = connection.send_request(ReadTextFileRequest::new("reader", "/qiongli-context/paper.txt")).block_task().await;
                                Ok(())
                            })
                        }, agent_client_protocol::on_receive_request!());
                    AcpV1Client::from_transport(agent)
                        .with_control(control())
                        .with_read_view(vec![(
                            "/qiongli-context/paper.txt".into(),
                            "approved".into(),
                        )])
                        .unwrap()
                        .with_session(
                            std::env::current_dir().unwrap(),
                            CancellationToken::new(),
                            async |session| {
                                let _ = session.run_turn("read", token).await;
                                std::future::pending().await
                            },
                        )
                        .await
                });
                send.send(result.map_err(|error| error.code)).unwrap();
            });
            assert_eq!(
                receive.recv_timeout(Duration::from_secs(3)).expect(phase),
                Err(AgentBackendErrorCode::ResponseInvalid)
            );
            worker.join().unwrap();
        }
    }

    #[test]
    fn read_view_admission_reuses_cancel_deadline_and_scope_drop() {
        let control = control();
        assert!(control.admit_read("reader", 1).is_err());
        let token = CancellationToken::new();
        let scope = control
            .begin_turn("reader", 1, token.clone(), Duration::from_secs(1))
            .unwrap();
        assert!(control.admit_read("other", 1).is_err());
        control.admit_read("reader", 256 * 1024).unwrap();
        assert!(control.admit_read("reader", 1).is_err());
        token.cancel();
        assert!(control.admit_read("reader", 0).is_err());
        drop(scope);
        assert!(control.admit_read("reader", 0).is_err());
        let _scope = control
            .begin_turn("reader", 2, CancellationToken::new(), Duration::ZERO)
            .unwrap();
        assert!(control.admit_read("reader", 1).is_err());
    }

    #[test]
    fn rejected_startup_closes_control_stream_without_launching_an_agent() {
        futures::executor::block_on(async {
            for case in 0..4 {
                let control = control();
                let token = CancellationToken::new();
                if case == 2 {
                    token.cancel();
                }
                let client = AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Normal))
                    .with_control(control.clone());
                if case == 3 {
                    assert!(
                        client
                            .with_timeouts(AcpV1Timeouts {
                                prompt: Duration::ZERO,
                                ..Default::default()
                            })
                            .is_err()
                    );
                } else {
                    let cwd = if case == 0 {
                        PathBuf::from("relative")
                    } else {
                        std::env::current_dir().unwrap()
                    };
                    assert!(
                        client
                            .run_turn(cwd, if case == 1 { "" } else { "prompt" }, token)
                            .await
                            .is_err()
                    );
                }
                assert!(control.next_update().await.is_none());
            }
        });
    }

    #[test]
    fn interactive_permissions_are_exact_once_and_bound_to_connection_participant_and_turn() {
        futures::executor::block_on(async {
            for option_id in ["allow", "deny"] {
                let control = control();
                let observer = control.clone();
                let (result, requests) = futures::join!(
                    AcpV1Client::from_transport(fixture_agent(
                        FixtureBehavior::PermissionInteractive
                    ))
                    .with_control(control.clone())
                    .with_session(
                        std::env::current_dir().unwrap(),
                        CancellationToken::new(),
                        async |session| {
                            let first = session.run_turn("one", CancellationToken::new()).await?;
                            let second = session.run_turn("two", CancellationToken::new()).await?;
                            Ok((first, second))
                        }
                    ),
                    async {
                        let mut requests: Vec<AcpV1ControlRequest> = Vec::new();
                        while let Some(update) = observer.next_update().await {
                            if let AcpV1UpdateKind::PermissionPending { request } = update.kind {
                                for field in 0..5 {
                                    let mut binding = request.binding.clone();
                                    match field {
                                        0 => binding.turn_id += 1,
                                        1 => binding.role = OrchestrationRole::Reviewer,
                                        2 => binding.session_id = "different-session".to_owned(),
                                        3 => binding.connection_id = "0".repeat(32),
                                        _ => {
                                            binding.run_id =
                                                RunId::parse(format!("run_{}", "b".repeat(32)))
                                                    .unwrap()
                                        }
                                    }
                                    assert_eq!(
                                        observer
                                            .apply(AcpV1ControlRequest::Permission {
                                                binding,
                                                request_id: request.request_id,
                                                choice: AcpV1PermissionChoice::Cancel
                                            })
                                            .unwrap_err()
                                            .code,
                                        AgentBackendErrorCode::InvalidRequest
                                    );
                                }
                                for (id, expected) in [
                                    ("missing", AgentBackendErrorCode::InvalidRequest),
                                    ("always", AgentBackendErrorCode::CapabilityUnavailable),
                                ] {
                                    assert_eq!(
                                        observer
                                            .apply(AcpV1ControlRequest::Permission {
                                                binding: request.binding.clone(),
                                                request_id: request.request_id,
                                                choice: AcpV1PermissionChoice::Select {
                                                    option_id: id.to_owned()
                                                }
                                            })
                                            .unwrap_err()
                                            .code,
                                        expected
                                    );
                                }
                                assert_eq!(
                                    observer
                                        .apply(AcpV1ControlRequest::Permission {
                                            binding: request.binding.clone(),
                                            request_id: request.request_id + 1,
                                            choice: AcpV1PermissionChoice::Cancel
                                        })
                                        .unwrap_err()
                                        .code,
                                    AgentBackendErrorCode::InvalidRequest
                                );
                                if let Some(previous) = requests.last() {
                                    assert_eq!(
                                        observer.apply(previous.clone()).unwrap_err().code,
                                        AgentBackendErrorCode::InvalidRequest
                                    );
                                }
                                let response = AcpV1ControlRequest::Permission {
                                    binding: request.binding,
                                    request_id: request.request_id,
                                    choice: AcpV1PermissionChoice::Select {
                                        option_id: option_id.to_owned(),
                                    },
                                };
                                observer.apply(response.clone()).unwrap();
                                assert_eq!(
                                    observer.apply(response.clone()).unwrap_err().code,
                                    AgentBackendErrorCode::InvalidRequest
                                );
                                requests.push(response);
                            }
                        }
                        requests
                    }
                );
                let (first, second) = result.unwrap();
                assert_eq!((first.turn_id(), second.turn_id()), (1, 2));
                assert_eq!(requests.len(), 2);
                assert!(control.apply(requests[1].clone()).is_err());
                let reused = AcpV1Client::from_transport(fixture_agent(FixtureBehavior::Normal))
                    .with_control(control)
                    .run_turn(
                        std::env::current_dir().unwrap(),
                        "not sent",
                        CancellationToken::new(),
                    )
                    .await;
                assert_eq!(
                    reused.err().unwrap().code,
                    AgentBackendErrorCode::InvalidRequest
                );
            }
        });
    }

    #[test]
    fn pending_permission_cancel_timeout_exit_and_unowned_requests_fail_closed() {
        futures::executor::block_on(async {
            for (behavior, cancel, expected) in [
                (FixtureBehavior::PermissionWaiting, true, None),
                (
                    FixtureBehavior::PermissionWaiting,
                    false,
                    Some(AgentBackendErrorCode::TransportUnavailable),
                ),
                (
                    FixtureBehavior::PermissionExit,
                    false,
                    Some(AgentBackendErrorCode::TransportUnavailable),
                ),
                (
                    FixtureBehavior::PermissionWrongSession,
                    false,
                    Some(AgentBackendErrorCode::ResponseInvalid),
                ),
                (
                    FixtureBehavior::PermissionPrematureStop,
                    false,
                    Some(AgentBackendErrorCode::ResponseInvalid),
                ),
            ] {
                let control = control();
                let observer = control.clone();
                let start = Instant::now();
                let (result, updates) = futures::join!(
                    AcpV1Client::from_transport(fixture_agent(behavior))
                        .with_control(control.clone())
                        .with_timeouts(short_timeouts())
                        .unwrap()
                        .run_turn(
                            std::env::current_dir().unwrap(),
                            "prompt",
                            CancellationToken::new()
                        ),
                    async {
                        let mut updates = Vec::new();
                        while let Some(update) = observer.next_update().await {
                            if cancel
                                && let AcpV1UpdateKind::PermissionPending { request } = &update.kind
                            {
                                observer
                                    .apply(AcpV1ControlRequest::Cancel {
                                        binding: request.binding.clone(),
                                    })
                                    .unwrap();
                                assert!(
                                    observer
                                        .apply(AcpV1ControlRequest::Permission {
                                            binding: request.binding.clone(),
                                            request_id: request.request_id,
                                            choice: AcpV1PermissionChoice::Select {
                                                option_id: "allow".to_owned()
                                            }
                                        })
                                        .is_err()
                                );
                            }
                            updates.push(update);
                        }
                        updates
                    }
                );
                match expected {
                    Some(code) => assert_eq!(result.err().unwrap().code, code),
                    None => assert!(matches!(
                        result.unwrap().events().last(),
                        Some(AgentEventV1::Cancelled)
                    )),
                }
                assert!(start.elapsed() < Duration::from_secs(2));
                assert!(!control.has_pending_permission());
                if matches!(behavior, FixtureBehavior::PermissionWaiting) && !cancel {
                    assert!(updates.iter().any(|update| matches!(
                        update.kind,
                        AcpV1UpdateKind::Turn {
                            status: AcpV1TurnStatus::TimedOut,
                            ..
                        }
                    )));
                }
            }
        });
    }

    #[test]
    fn capabilities_and_activity_are_owned_bounded_and_exclude_hidden_or_raw_content() {
        futures::executor::block_on(async {
            let control = control();
            let result = AcpV1Client::from_transport(fixture_agent(FixtureBehavior::RichUpdates))
                .with_control(control.clone())
                .with_session(
                    std::env::current_dir().unwrap(),
                    CancellationToken::new(),
                    async |session| {
                        assert!(session.info().session_established);
                        assert_eq!(session.info().auth_method_ids, ["fixture-auth"]);
                        assert!(session.info().load_advertised && session.info().resume_advertised);
                        assert!(!session.info().load_enabled && !session.info().resume_enabled);
                        assert!(
                            !session.info().mode_selection_enabled
                                && !session.info().model_selection_enabled
                        );
                        assert_eq!(session.info().current_model_id.as_deref(), Some("model-a"));
                        let result = session.run_turn("prompt", CancellationToken::new()).await?;
                        assert_eq!(session.info().current_mode_id.as_deref(), Some("plan"));
                        Ok(result)
                    },
                )
                .await
                .unwrap();
            assert_eq!(result.events().len(), 2);
            let mut updates = Vec::new();
            while let Some(update) = control.next_update().await {
                updates.push(update);
            }
            assert!(
                updates
                    .iter()
                    .any(|update| matches!(update.kind, AcpV1UpdateKind::Plan { .. }))
            );
            assert_eq!(
                updates
                    .iter()
                    .filter(|update| matches!(update.kind, AcpV1UpdateKind::Tool { .. }))
                    .count(),
                2
            );
            let json = serde_json::to_string(&updates).unwrap();
            for secret in [
                "hidden-reasoning-canary",
                "raw-input-canary",
                "raw-output-canary",
            ] {
                assert!(!json.contains(secret));
            }
            for (index, update) in updates.iter().enumerate() {
                assert_eq!(update.sequence, index as u64 + 1);
            }

            let auth_control = self::control();
            let result = AcpV1Client::from_transport(fixture_agent(FixtureBehavior::AuthRequired))
                .with_control(auth_control.clone())
                .run_turn(
                    std::env::current_dir().unwrap(),
                    "not sent",
                    CancellationToken::new(),
                )
                .await;
            assert_eq!(
                result.err().unwrap().code,
                AgentBackendErrorCode::AuthenticationUnavailable
            );
            let mut required = false;
            while let Some(update) = auth_control.next_update().await {
                if let AcpV1UpdateKind::Session { info } = update.kind {
                    required |= info.authentication_required && !info.session_established;
                }
            }
            assert!(required);
        });
    }

    fn short_timeouts() -> AcpV1Timeouts {
        AcpV1Timeouts {
            initialization: Duration::from_millis(80),
            session_creation: Duration::from_millis(80),
            prompt: Duration::from_millis(80),
            permission: Duration::from_millis(40),
            cancellation_grace: Duration::from_millis(40),
        }
    }

    #[test]
    fn silent_phases_cancel_timeout_and_transport_loss_are_bounded() {
        futures::executor::block_on(async {
            for behavior in [
                FixtureBehavior::SilentInitialize,
                FixtureBehavior::SilentSession,
                FixtureBehavior::Silent,
                FixtureBehavior::Disconnected,
            ] {
                let start = Instant::now();
                let result = AcpV1Client::from_transport(fixture_agent(behavior))
                    .with_timeouts(short_timeouts())
                    .unwrap()
                    .run_turn(
                        std::env::current_dir().unwrap(),
                        "prompt",
                        CancellationToken::new(),
                    )
                    .await;
                assert_eq!(
                    result.err().unwrap().code,
                    AgentBackendErrorCode::TransportUnavailable
                );
                assert!(start.elapsed() < Duration::from_secs(2));
            }
            for behavior in [
                FixtureBehavior::SilentInitialize,
                FixtureBehavior::SilentSession,
                FixtureBehavior::CancelAcknowledged,
                FixtureBehavior::Silent,
            ] {
                let cancel = CancellationToken::new();
                let observer = cancel.clone();
                let start = Instant::now();
                let (result, ()) = futures::join!(
                    AcpV1Client::from_transport(fixture_agent(behavior))
                        .with_timeouts(short_timeouts())
                        .unwrap()
                        .run_turn(std::env::current_dir().unwrap(), "prompt", observer),
                    async {
                        async_io::Timer::after(Duration::from_millis(20)).await;
                        cancel.cancel();
                    }
                );
                if matches!(behavior, FixtureBehavior::CancelAcknowledged) {
                    assert!(matches!(
                        result.unwrap().events().last(),
                        Some(AgentEventV1::Cancelled)
                    ));
                } else {
                    assert_eq!(
                        result.err().unwrap().code,
                        if matches!(behavior, FixtureBehavior::Silent) {
                            AgentBackendErrorCode::TransportUnavailable
                        } else {
                            AgentBackendErrorCode::Cancelled
                        }
                    );
                }
                assert!(start.elapsed() < Duration::from_secs(2));
            }
        });
    }

    #[cfg(unix)]
    #[test]
    #[allow(
        clippy::disallowed_methods,
        reason = "Test-only read-only process probe verifies the SDK-owned launcher cleanup"
    )]
    fn timed_out_owned_launcher_and_descendant_exit() {
        let path = std::env::temp_dir().join(format!(
            "qiongli-acp-owned-{}-{}.pid",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config = AcpAgentConfig::new("/bin/sh").args([
            "-c",
            "printf '%s\n' \"$$\" > \"$1\"; sleep 60 & printf '%s\n' \"$!\" >> \"$1\"; wait",
            "qiongli-owned-fixture",
            path.to_str().unwrap(),
        ]);
        let start = Instant::now();
        let result = futures::executor::block_on(
            AcpV1Client::from_transport(AcpAgent::new(config))
                .with_timeouts(AcpV1Timeouts {
                    initialization: Duration::from_millis(300),
                    ..short_timeouts()
                })
                .unwrap()
                .run_turn(
                    std::env::current_dir().unwrap(),
                    "prompt",
                    CancellationToken::new(),
                ),
        );
        assert_eq!(
            result.err().unwrap().code,
            AgentBackendErrorCode::TransportUnavailable
        );
        let pids = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        for pid in pids.lines() {
            loop {
                let output = std::process::Command::new("ps")
                    .args(["-p", pid, "-o", "stat="])
                    .output()
                    .unwrap();
                let status = String::from_utf8(output.stdout).unwrap();
                if !output.status.success()
                    || status.trim().is_empty()
                    || status.trim().starts_with('Z')
                {
                    break;
                }
                assert!(
                    start.elapsed() < Duration::from_secs(3),
                    "owned process {pid} remains alive"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    #[test]
    fn first_coordinator_projection_is_atomic_and_turn_scoped() {
        let completed = AcpV1TurnOutcome {
            protocol_version: 1,
            turn_id: 1,
            session_id: "coordinator-session".to_string(),
            events: vec![
                AgentEventV1::ContentDelta {
                    content: "bounded ".to_string(),
                },
                AgentEventV1::ContentDelta {
                    content: "answer".to_string(),
                },
                AgentEventV1::Completed {
                    finish_reason: AgentFinishReason::Length,
                },
            ],
        };
        let mut state = running_all_chat_state();
        completed
            .project_first_coordinator_turn(&mut state, 1)
            .unwrap();

        assert_eq!(state.status(), OrchestrationRunStatus::Running);
        assert_eq!(state.generation(), 4);
        assert!(matches!(
            &state.events()[1].kind,
            AllChatEventKindV1::AgentSessionReady {
                role: OrchestrationRole::Primary,
                session_id,
            } if session_id == "coordinator-session"
        ));
        assert!(matches!(
            &state.events()[2].kind,
            AllChatEventKindV1::CoordinatorMessage {
                by: OrchestrationRole::Primary,
                content,
            } if content == "bounded answer"
        ));
        assert!(matches!(
            state.events()[3].kind,
            AllChatEventKindV1::AgentTurnCompleted {
                by: OrchestrationRole::Primary,
                finish_reason: AgentFinishReason::Length,
            }
        ));
        assert!(
            !state
                .events()
                .iter()
                .any(|event| matches!(event.kind, AllChatEventKindV1::RunCompleted { .. }))
        );

        let cancelled = AcpV1TurnOutcome {
            protocol_version: 1,
            turn_id: 1,
            session_id: "cancelled-session".to_string(),
            events: vec![
                AgentEventV1::ContentDelta {
                    content: "discard this partial text".to_string(),
                },
                AgentEventV1::Cancelled,
            ],
        };
        let mut cancelled_state = running_all_chat_state();
        cancelled
            .project_first_coordinator_turn(&mut cancelled_state, 1)
            .unwrap();
        assert_eq!(cancelled_state.status(), OrchestrationRunStatus::Running);
        assert_eq!(cancelled_state.events().len(), 3);
        assert!(matches!(
            cancelled_state.events()[2].kind,
            AllChatEventKindV1::AgentTurnCancelled {
                by: OrchestrationRole::Primary,
            }
        ));
        assert!(
            !cancelled_state
                .events()
                .iter()
                .any(|event| matches!(event.kind, AllChatEventKindV1::CoordinatorMessage { .. }))
        );
        let original = cancelled_state.clone();
        assert_eq!(
            cancelled_state
                .append_event(
                    cancelled_state.generation(),
                    4,
                    AllChatEventKindV1::AgentTurnCancelled {
                        by: OrchestrationRole::Primary,
                    },
                )
                .unwrap_err(),
            AllChatStateError::InvalidTransition
        );
        assert_eq!(cancelled_state, original);

        let mut out_of_context = running_all_chat_state();
        let original = out_of_context.clone();
        assert_eq!(
            out_of_context
                .append_event(
                    1,
                    2,
                    AllChatEventKindV1::AgentTurnCancelled {
                        by: OrchestrationRole::Primary,
                    },
                )
                .unwrap_err(),
            AllChatStateError::InvalidTransition
        );
        assert_eq!(out_of_context, original);

        let mut completion_without_message = running_all_chat_state();
        completion_without_message
            .append_event(
                1,
                2,
                AllChatEventKindV1::AgentSessionReady {
                    role: OrchestrationRole::Primary,
                    session_id: "ready-without-message".to_string(),
                },
            )
            .unwrap();
        let original = completion_without_message.clone();
        assert_eq!(
            completion_without_message
                .append_event(
                    2,
                    3,
                    AllChatEventKindV1::AgentTurnCompleted {
                        by: OrchestrationRole::Primary,
                        finish_reason: AgentFinishReason::Stop,
                    },
                )
                .unwrap_err(),
            AllChatStateError::InvalidTransition
        );
        assert_eq!(completion_without_message, original);

        let invalid = AcpV1TurnOutcome {
            protocol_version: 1,
            turn_id: 1,
            session_id: "invalid-session".to_string(),
            events: vec![
                AgentEventV1::ContentDelta {
                    content: "invalid\0content".to_string(),
                },
                AgentEventV1::Completed {
                    finish_reason: AgentFinishReason::Length,
                },
            ],
        };
        let mut invalid_state = running_all_chat_state();
        let original = invalid_state.clone();
        assert_eq!(
            invalid
                .project_first_coordinator_turn(&mut invalid_state, 1)
                .unwrap_err(),
            AllChatStateError::InvalidEvent
        );
        assert_eq!(invalid_state, original);

        let mut stale_state = running_all_chat_state();
        let original = stale_state.clone();
        assert_eq!(
            invalid
                .project_first_coordinator_turn(&mut stale_state, 2)
                .unwrap_err(),
            AllChatStateError::StaleGeneration
        );
        assert_eq!(stale_state, original);

        let mut event_limit_state = running_all_chat_state();
        for sequence in 2..=1_023 {
            event_limit_state
                .append_event(
                    sequence - 1,
                    sequence,
                    AllChatEventKindV1::UserMessage {
                        content: "bounded filler".to_string(),
                    },
                )
                .unwrap();
        }
        let original = event_limit_state.clone();
        assert_eq!(
            completed
                .project_first_coordinator_turn(&mut event_limit_state, 1_023)
                .unwrap_err(),
            AllChatStateError::LimitExceeded
        );
        assert_eq!(event_limit_state, original);
    }
}
