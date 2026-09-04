use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use agent_client_protocol::schema::{
    ProtocolVersion,
    v1::{
        CancelNotification, ContentBlock, ContentChunk, InitializeRequest,
        RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
        SessionNotification, SessionUpdate, StopReason,
    },
};
use agent_client_protocol::{
    AcpAgent, AcpAgentConfig, Client, ConnectTo, Dispatch, DynConnectTo, Error as AcpError,
    ErrorCode as AcpErrorCode, JsonRpcMessage,
};

use crate::all_chat::MAX_CHAT_TEXT_BYTES;
use crate::{
    AgentBackendError, AgentBackendErrorCode, AgentEventV1, AgentFinishReason, AllChatEventKindV1,
    AllChatStateError, AllChatStateV1, CancellationToken, OrchestrationRole,
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
        if self.protocol_version != ProtocolVersion::V1.as_u16() {
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

/// Single-use stable ACP v1 client boundary.
///
/// Its public constructor is intentionally development-only: it launches one
/// exact, pinned adapter with `npx`, requires Node.js and `npx` on `PATH`, and is
/// not evidence of packaged provider support. Missing process prerequisites fail
/// closed as [`AgentBackendErrorCode::TransportUnavailable`].
pub struct AcpV1Client {
    transport: DynConnectTo<Client>,
}

impl AcpV1Client {
    /// Builds a development-only client for a fixed, pinned Codex or Claude ACP adapter.
    ///
    /// The SDK launches `npx -y <pinned-package>` without an explicitly configured
    /// shell; callers cannot supply a program, arguments, or environment overrides.
    #[must_use]
    pub fn for_development_npx(preset: AcpDevelopmentPresetV1) -> Self {
        Self::from_transport(AcpAgent::new(development_config(preset)))
    }

    fn from_transport(transport: impl ConnectTo<Client>) -> Self {
        Self {
            transport: DynConnectTo::new(transport),
        }
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
        validate_input(&cwd, &prompt)?;
        if cancellation.is_cancelled() {
            return Err(backend_error(AgentBackendErrorCode::Cancelled));
        }

        let permission_requested = Arc::new(AtomicBool::new(false));
        let permission_flag = Arc::clone(&permission_requested);
        let result = Client
            .builder()
            .name(ACP_CLIENT_NAME)
            .on_receive_request(
                async move |_request: RequestPermissionRequest, responder, _connection| {
                    permission_flag.store(true, Ordering::Release);
                    responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ))
                },
                agent_client_protocol::on_receive_request!(),
            )
            .connect_with(self.transport, async move |connection| {
                Ok(run_turn_on_connection(connection, cwd, prompt, cancellation).await)
            })
            .await;

        if permission_requested.load(Ordering::Acquire) {
            return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable));
        }

        result.map_err(map_acp_error)?
    }
}

fn development_config(preset: AcpDevelopmentPresetV1) -> AcpAgentConfig {
    AcpAgentConfig::new(NPX_PROGRAM).args(["-y", preset.package_reference()])
}

async fn run_turn_on_connection(
    connection: agent_client_protocol::ConnectionTo<agent_client_protocol::Agent>,
    cwd: PathBuf,
    prompt: String,
    cancellation: CancellationToken,
) -> Result<AcpV1TurnOutcome, AgentBackendError> {
    let initialized = connection
        .send_request(InitializeRequest::new(ProtocolVersion::V1))
        .block_task()
        .await
        .map_err(map_acp_error)?;
    if initialized.protocol_version != ProtocolVersion::V1 {
        return Err(backend_error(AgentBackendErrorCode::CapabilityUnavailable));
    }
    if cancellation.is_cancelled() {
        return Err(backend_error(AgentBackendErrorCode::Cancelled));
    }

    let mut session = connection
        .build_session(cwd)
        .block_task()
        .start_session()
        .await
        .map_err(map_acp_error)?;
    let session_id = session.session_id().to_string();
    validate_session_id(&session_id)?;
    if cancellation.is_cancelled() {
        return Err(backend_error(AgentBackendErrorCode::Cancelled));
    }

    session.send_prompt(prompt).map_err(map_acp_error)?;
    let mut events = Vec::new();
    let mut accepted_content_bytes = 0_usize;
    let mut update_count = 0_usize;
    let mut cancel_sent = false;

    loop {
        if cancellation.is_cancelled() && !cancel_sent {
            send_cancel(&session, &session_id)?;
            cancel_sent = true;
        }

        // ponytail: CancellationToken is polling-only. A wakeable cancellation
        // runtime is deferred until Qiongli owns a broader ACP session lifecycle.
        let message = session.read_update().await.map_err(map_acp_error)?;
        if cancellation.is_cancelled() && !cancel_sent {
            send_cancel(&session, &session_id)?;
            cancel_sent = true;
        }
        update_count = update_count
            .checked_add(1)
            .filter(|count| *count <= MAX_TURN_UPDATES)
            .ok_or_else(response_invalid)?;

        match message {
            agent_client_protocol::SessionMessage::SessionMessage(dispatch) => {
                let content = text_delta(dispatch, &session_id)?;
                if cancel_sent {
                    continue;
                }
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
                return Ok(AcpV1TurnOutcome {
                    protocol_version: ProtocolVersion::V1.as_u16(),
                    session_id,
                    events,
                });
            }
            _ => return Err(response_invalid()),
        }
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

fn validate_input(cwd: &Path, prompt: &str) -> Result<(), AgentBackendError> {
    let cwd_bytes = cwd.as_os_str().as_encoded_bytes();
    if !cwd.is_absolute()
        || cwd_bytes.is_empty()
        || cwd_bytes.len() > MAX_CWD_BYTES
        || cwd_bytes.contains(&0)
        || prompt.is_empty()
        || prompt.len() > MAX_PROMPT_BYTES
    {
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

    fn fixture_agent(requests_permission: bool) -> impl ConnectTo<Client> {
        Agent
            .builder()
            .on_receive_request(
                async |request: InitializeRequest,
                       responder: Responder<InitializeResponse>,
                       _connection: ConnectionTo<Client>| {
                    assert_eq!(request.protocol_version, ProtocolVersion::V1);
                    responder.respond(InitializeResponse::new(ProtocolVersion::V1))
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async |_request: NewSessionRequest,
                       responder: Responder<NewSessionResponse>,
                       _connection: ConnectionTo<Client>| {
                    responder.respond(NewSessionResponse::new("fixture-session"))
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |request: PromptRequest,
                            responder: Responder<PromptResponse>,
                            connection: ConnectionTo<Client>| {
                    if requests_permission {
                        let session_id = request.session_id;
                        let update_session_id = session_id.clone();
                        connection
                            .send_request(RequestPermissionRequest::new(
                                session_id,
                                ToolCallUpdate::new("fixture-tool", ToolCallUpdateFields::new()),
                                vec![PermissionOption::new(
                                    "allow",
                                    "Allow",
                                    PermissionOptionKind::AllowOnce,
                                )],
                            ))
                            .on_receiving_result(async move |response| {
                                assert_eq!(response?.outcome, RequestPermissionOutcome::Cancelled);
                                connection.send_notification(SessionNotification::new(
                                    update_session_id,
                                    SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                        ContentBlock::Text(TextContent::new("must not escape")),
                                    )),
                                ))?;
                                responder.respond(PromptResponse::new(StopReason::EndTurn))
                            })
                    } else {
                        connection.send_notification(SessionNotification::new(
                            request.session_id,
                            SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                ContentBlock::Text(TextContent::new("fixture reply")),
                            )),
                        ))?;
                        responder.respond(PromptResponse::new(StopReason::EndTurn))
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

            let outcome = AcpV1Client::from_transport(fixture_agent(false))
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

            let permission_error = match AcpV1Client::from_transport(fixture_agent(true))
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
    fn first_coordinator_projection_is_atomic_and_turn_scoped() {
        let completed = AcpV1TurnOutcome {
            protocol_version: 1,
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
