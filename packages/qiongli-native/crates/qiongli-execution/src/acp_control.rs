use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AgentBackendError, AgentBackendErrorCode, CancellationToken, OrchestrationRole, RunId,
};

const MAX_UPDATES: usize = 1_024;
const MAX_STREAM_BYTES: usize = 8 * 1024 * 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcpV1TurnBinding {
    pub connection_id: String,
    #[schemars(with = "String", regex(pattern = r"^run_[0-9a-f]{32}$"))]
    pub run_id: RunId,
    pub role: OrchestrationRole,
    pub session_id: String,
    #[schemars(range(min = 1, max = MAX_SAFE_INTEGER))]
    pub turn_id: u64,
}

#[derive(Clone, Debug, Default, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcpV1SessionInfo {
    pub adapter: Option<String>,
    // Session establishment is evidence of readiness, not a credential inspection.
    pub session_established: bool,
    pub authentication_required: bool,
    pub auth_method_ids: Vec<String>,
    pub load_advertised: bool,
    pub resume_advertised: bool,
    pub mode_ids: Vec<String>,
    pub current_mode_id: Option<String>,
    pub model_ids: Vec<String>,
    pub current_model_id: Option<String>,
    // Advertised support does not enable controls that Qiongli has not implemented.
    pub load_enabled: bool,
    pub resume_enabled: bool,
    pub mode_selection_enabled: bool,
    pub model_selection_enabled: bool,
}

#[derive(Clone, Copy, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpV1PermissionKind {
    AllowOnce,
    RejectOnce,
    AllowAlways,
    RejectAlways,
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcpV1PermissionOption {
    pub option_id: String,
    pub name: String,
    pub kind: AcpV1PermissionKind,
    pub enabled: bool,
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcpV1PermissionRequest {
    pub binding: AcpV1TurnBinding,
    #[schemars(range(min = 1, max = MAX_SAFE_INTEGER))]
    pub request_id: u64,
    pub tool_call_id: String,
    pub title: String,
    pub options: Vec<AcpV1PermissionOption>,
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AcpV1PermissionChoice {
    Cancel,
    Select { option_id: String },
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AcpV1ControlRequest {
    Cancel {
        binding: AcpV1TurnBinding,
    },
    Permission {
        binding: AcpV1TurnBinding,
        request_id: u64,
        choice: AcpV1PermissionChoice,
    },
}

#[derive(Clone, Copy, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpV1TurnStatus {
    Running,
    Completed,
    Cancelled,
    TimedOut,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpV1ActivityStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcpV1PlanEntry {
    pub content: String,
    pub status: AcpV1ActivityStatus,
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AcpV1UpdateKind {
    Session {
        info: AcpV1SessionInfo,
    },
    Turn {
        binding: AcpV1TurnBinding,
        status: AcpV1TurnStatus,
    },
    Text {
        binding: AcpV1TurnBinding,
        content: String,
    },
    Plan {
        binding: AcpV1TurnBinding,
        entries: Vec<AcpV1PlanEntry>,
    },
    Tool {
        binding: AcpV1TurnBinding,
        tool_call_id: String,
        title: Option<String>,
        status: Option<AcpV1ActivityStatus>,
    },
    PermissionPending {
        request: AcpV1PermissionRequest,
    },
    PermissionResolved {
        binding: AcpV1TurnBinding,
        request_id: u64,
        choice: AcpV1PermissionChoice,
    },
}

#[derive(Clone, Debug, JsonSchema, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcpV1Update {
    pub connection_id: String,
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    #[schemars(range(min = 1, max = MAX_SAFE_INTEGER))]
    pub sequence: u64,
    #[schemars(with = "String", regex(pattern = r"^run_[0-9a-f]{32}$"))]
    pub run_id: RunId,
    pub role: OrchestrationRole,
    pub kind: AcpV1UpdateKind,
}

/// One participant's transient controls. No SDK handles or project-write authority.
#[derive(Clone)]
pub struct AcpV1Control {
    shared: Arc<Shared>,
}

struct Shared {
    connection_id: String,
    run_id: RunId,
    role: OrchestrationRole,
    state: Mutex<State>,
    changed: event_listener::Event,
}

#[derive(Default)]
struct State {
    claimed: bool,
    closed: bool,
    active: Option<Active>,
    pending: Option<Pending>,
    next_request: u64,
    sequence: u64,
    bytes: usize,
    updates: VecDeque<(AcpV1Update, usize)>,
    failure: Option<AgentBackendErrorCode>,
    timed_out: bool,
    info: Option<AcpV1SessionInfo>,
}

struct Active {
    binding: AcpV1TurnBinding,
    cancellation: CancellationToken,
    stop: CancellationToken,
    deadline: Instant,
    requests: usize,
    read_requests: usize,
    read_bytes: usize,
}

struct Pending {
    request: AcpV1PermissionRequest,
    deadline: Instant,
    sender: oneshot::Sender<AcpV1PermissionChoice>,
}

pub(super) struct PermissionWait {
    pub request: AcpV1PermissionRequest,
    pub receiver: oneshot::Receiver<AcpV1PermissionChoice>,
    pub cancellation: CancellationToken,
    pub stop: CancellationToken,
    pub deadline: Instant,
}

impl AcpV1Control {
    pub fn new(run_id: RunId, role: OrchestrationRole) -> Result<Self, AgentBackendError> {
        RunId::parse(run_id.as_str()).map_err(|_| invalid_request())?;
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes)
            .map_err(|_| error(AgentBackendErrorCode::TransportUnavailable))?;
        let connection_id = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(Self {
            shared: Arc::new(Shared {
                connection_id,
                run_id,
                role,
                state: Mutex::new(State::default()),
                changed: event_listener::Event::new(),
            }),
        })
    }

    fn state(&self) -> MutexGuard<'_, State> {
        // No external callback or await runs under this lock.
        self.shared
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn apply(&self, request: AcpV1ControlRequest) -> Result<(), AgentBackendError> {
        let mut state = self.state();
        let binding = match &request {
            AcpV1ControlRequest::Cancel { binding }
            | AcpV1ControlRequest::Permission { binding, .. } => binding,
        }
        .clone();
        let active = state
            .active
            .as_ref()
            .filter(|active| {
                active.binding == binding
                    && !active.cancellation.is_cancelled()
                    && Instant::now() < active.deadline
            })
            .ok_or_else(invalid_request)?;
        match request {
            AcpV1ControlRequest::Cancel { .. } => {
                active.cancellation.cancel();
                Ok(())
            }
            AcpV1ControlRequest::Permission {
                request_id, choice, ..
            } => {
                let pending = state
                    .pending
                    .as_ref()
                    .filter(|pending| {
                        pending.request.request_id == request_id
                            && pending.request.binding == binding
                            && Instant::now() < pending.deadline
                    })
                    .ok_or_else(invalid_request)?;
                if let AcpV1PermissionChoice::Select { option_id } = &choice {
                    let option = pending
                        .request
                        .options
                        .iter()
                        .find(|option| &option.option_id == option_id)
                        .ok_or_else(invalid_request)?;
                    if !option.enabled {
                        return Err(error(AgentBackendErrorCode::CapabilityUnavailable));
                    }
                }
                state
                    .pending
                    .take()
                    .ok_or_else(invalid_request)?
                    .sender
                    .send(choice)
                    .map_err(|_| invalid_request())
            }
        }
    }

    pub fn try_next_update(&self) -> Option<AcpV1Update> {
        let mut state = self.state();
        let (update, bytes) = state.updates.pop_front()?;
        state.bytes -= bytes;
        Some(update)
    }

    pub async fn next_update(&self) -> Option<AcpV1Update> {
        loop {
            let listener = self.shared.changed.listen();
            if let Some(update) = self.try_next_update() {
                return Some(update);
            }
            if self.state().closed {
                return None;
            }
            listener.await;
        }
    }

    pub(super) fn close_unclaimed(&self) {
        let mut state = self.state();
        if !state.claimed {
            state.closed = true;
        }
        drop(state);
        self.shared.changed.notify(usize::MAX);
    }

    pub(super) fn claim(&self) -> Result<ControlScope, AgentBackendError> {
        let mut state = self.state();
        if state.claimed || state.closed {
            return Err(invalid_request());
        }
        state.claimed = true;
        Ok(ControlScope(self.clone()))
    }

    pub(super) fn failure(&self) -> Result<(), AgentBackendError> {
        self.state().failure.map_or(Ok(()), |code| Err(error(code)))
    }

    pub(super) fn authentication_required(&self) -> Result<(), AgentBackendError> {
        let mut info = self.state().info.clone().unwrap_or_default();
        info.authentication_required = true;
        info.session_established = false;
        self.emit(AcpV1UpdateKind::Session { info })
    }

    pub(super) fn timeout(&self) {
        let mut state = self.state();
        state.timed_out = true;
        state
            .failure
            .get_or_insert(AgentBackendErrorCode::TransportUnavailable);
    }

    pub(super) fn timed_out(&self) -> bool {
        self.state().timed_out
    }

    pub(super) fn fail(&self, code: AgentBackendErrorCode) {
        self.state().failure.get_or_insert(code);
        self.shared.changed.notify(usize::MAX);
    }

    pub(super) fn emit(&self, kind: AcpV1UpdateKind) -> Result<(), AgentBackendError> {
        let mut state = self.state();
        let sequence = state
            .sequence
            .checked_add(1)
            .filter(|n| *n <= MAX_SAFE_INTEGER)
            .ok_or_else(|| error(AgentBackendErrorCode::ResponseInvalid))?;
        let update = AcpV1Update {
            connection_id: self.shared.connection_id.clone(),
            schema_version: 1,
            sequence,
            run_id: self.shared.run_id.clone(),
            role: self.shared.role,
            kind,
        };
        let bytes = serde_json::to_vec(&update)
            .map_err(|_| error(AgentBackendErrorCode::ResponseInvalid))?
            .len();
        if state.updates.len() >= MAX_UPDATES
            || state.bytes.saturating_add(bytes) > MAX_STREAM_BYTES
        {
            state.failure = Some(AgentBackendErrorCode::ResponseInvalid);
            return Err(error(AgentBackendErrorCode::ResponseInvalid));
        }
        if let AcpV1UpdateKind::Session { info } = &update.kind {
            state.info = Some(info.clone());
        }
        state.bytes += bytes;
        state.sequence = sequence;
        state.updates.push_back((update, bytes));
        drop(state);
        self.shared.changed.notify(usize::MAX);
        Ok(())
    }

    pub(super) fn begin_turn(
        &self,
        session_id: &str,
        turn_id: u64,
        cancellation: CancellationToken,
        timeout: Duration,
    ) -> Result<TurnScope, AgentBackendError> {
        let binding = AcpV1TurnBinding {
            connection_id: self.shared.connection_id.clone(),
            run_id: self.shared.run_id.clone(),
            role: self.shared.role,
            session_id: session_id.to_owned(),
            turn_id,
        };
        {
            let mut state = self.state();
            if state.closed || state.active.is_some() {
                return Err(invalid_request());
            }
            state.active = Some(Active {
                binding: binding.clone(),
                cancellation,
                stop: CancellationToken::new(),
                deadline: Instant::now() + timeout,
                requests: 0,
                read_requests: 0,
                read_bytes: 0,
            });
        }
        let scope = TurnScope {
            control: self.clone(),
            binding: binding.clone(),
            status: AcpV1TurnStatus::Interrupted,
        };
        self.emit(AcpV1UpdateKind::Turn {
            binding,
            status: AcpV1TurnStatus::Running,
        })?;
        Ok(scope)
    }

    pub(super) fn admit_read(
        &self,
        session_id: &str,
        bytes: usize,
    ) -> Result<(), AgentBackendError> {
        let mut state = self.state();
        if state.closed || state.failure.is_some() {
            return Err(error(AgentBackendErrorCode::ResponseInvalid));
        }
        let active = state
            .active
            .as_mut()
            .filter(|active| active.binding.session_id == session_id)
            .ok_or_else(|| error(AgentBackendErrorCode::ResponseInvalid))?;
        if active.cancellation.is_cancelled()
            || active.stop.is_cancelled()
            || Instant::now() >= active.deadline
            || active.read_requests >= 16
            || bytes > (256 * 1024_usize).saturating_sub(active.read_bytes)
        {
            return Err(error(AgentBackendErrorCode::ResponseInvalid));
        }
        active.read_requests += 1;
        active.read_bytes += bytes;
        Ok(())
    }

    pub(super) fn begin_permission(
        &self,
        session_id: &str,
        tool_call_id: String,
        title: String,
        options: Vec<AcpV1PermissionOption>,
        timeout: Duration,
    ) -> Result<PermissionWait, AgentBackendError> {
        let (sender, receiver) = oneshot::channel();
        let mut state = self.state();
        if state.pending.is_some() {
            return Err(error(AgentBackendErrorCode::ResponseInvalid));
        }
        let active = state
            .active
            .as_mut()
            .filter(|active| active.binding.session_id == session_id)
            .ok_or_else(|| error(AgentBackendErrorCode::ResponseInvalid))?;
        if active.cancellation.is_cancelled() {
            return Err(error(AgentBackendErrorCode::Cancelled));
        }
        active.requests += 1;
        if active.requests > 64 {
            return Err(error(AgentBackendErrorCode::ResponseInvalid));
        }
        let binding = active.binding.clone();
        let cancellation = active.cancellation.clone();
        let stop = active.stop.clone();
        let deadline = active.deadline.min(Instant::now() + timeout);
        state.next_request = state
            .next_request
            .checked_add(1)
            .filter(|id| *id <= MAX_SAFE_INTEGER)
            .ok_or_else(|| error(AgentBackendErrorCode::ResponseInvalid))?;
        let request = AcpV1PermissionRequest {
            binding,
            request_id: state.next_request,
            tool_call_id,
            title,
            options,
        };
        state.pending = Some(Pending {
            request: request.clone(),
            deadline,
            sender,
        });
        drop(state);
        self.emit(AcpV1UpdateKind::PermissionPending {
            request: request.clone(),
        })?;
        Ok(PermissionWait {
            request,
            receiver,
            cancellation,
            stop,
            deadline,
        })
    }

    pub(super) fn permission_resolved(
        &self,
        request: &AcpV1PermissionRequest,
        choice: AcpV1PermissionChoice,
    ) -> Result<(), AgentBackendError> {
        let mut state = self.state();
        // A dropped turn still cancels its SDK responder, but emits no post-terminal event.
        if !state
            .active
            .as_ref()
            .is_some_and(|active| active.binding == request.binding)
        {
            return Ok(());
        }
        if state
            .pending
            .as_ref()
            .is_some_and(|pending| pending.request.request_id == request.request_id)
        {
            state.pending = None;
        }
        drop(state);
        self.emit(AcpV1UpdateKind::PermissionResolved {
            binding: request.binding.clone(),
            request_id: request.request_id,
            choice,
        })
    }

    pub(super) fn reject_pending_terminal(&self) -> bool {
        let state = self.state();
        state.pending.is_some()
            && state
                .active
                .as_ref()
                .is_some_and(|active| !active.cancellation.is_cancelled())
    }

    #[cfg(test)]
    pub(super) fn has_pending_permission(&self) -> bool {
        self.state().pending.is_some()
    }
}

pub(super) struct ControlScope(AcpV1Control);
impl Drop for ControlScope {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.closed = true;
        state.pending = None;
        if let Some(active) = state.active.take() {
            active.stop.cancel();
        }
        drop(state);
        self.0.shared.changed.notify(usize::MAX);
    }
}

pub(super) struct TurnScope {
    pub control: AcpV1Control,
    pub binding: AcpV1TurnBinding,
    pub status: AcpV1TurnStatus,
}
impl Drop for TurnScope {
    fn drop(&mut self) {
        let mut state = self.control.state();
        state.pending = None;
        if let Some(active) = state.active.take() {
            active.stop.cancel();
        }
        drop(state);
        let _ = self.control.emit(AcpV1UpdateKind::Turn {
            binding: self.binding.clone(),
            status: self.status,
        });
    }
}

fn error(code: AgentBackendErrorCode) -> AgentBackendError {
    AgentBackendError::new(code, None)
}
fn invalid_request() -> AgentBackendError {
    error(AgentBackendErrorCode::InvalidRequest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control() -> AcpV1Control {
        let mut control = AcpV1Control::new(
            RunId::parse(format!("run_{}", "a".repeat(32))).unwrap(),
            OrchestrationRole::Primary,
        )
        .unwrap();
        Arc::get_mut(&mut control.shared).unwrap().connection_id = "1".repeat(32);
        control
    }

    #[test]
    fn control_stream_fixture_is_generated_closed_and_replay_rejected() {
        let control = control();
        let scope = control.claim().unwrap();
        control
            .emit(AcpV1UpdateKind::Session {
                info: AcpV1SessionInfo {
                    adapter: Some("fixture-adapter@1.0.0".to_owned()),
                    session_established: true,
                    load_advertised: true,
                    ..Default::default()
                },
            })
            .unwrap();
        let mut turn = control
            .begin_turn(
                "fixture-session",
                1,
                CancellationToken::new(),
                Duration::from_secs(30),
            )
            .unwrap();
        control
            .emit(AcpV1UpdateKind::Text {
                binding: turn.binding.clone(),
                content: "Compare the selected sources.".to_owned(),
            })
            .unwrap();
        control
            .emit(AcpV1UpdateKind::Plan {
                binding: turn.binding.clone(),
                entries: vec![AcpV1PlanEntry {
                    content: "Read source references".to_owned(),
                    status: AcpV1ActivityStatus::InProgress,
                }],
            })
            .unwrap();
        control
            .emit(AcpV1UpdateKind::Tool {
                binding: turn.binding.clone(),
                tool_call_id: "read-source".to_owned(),
                title: Some("Read selected source".to_owned()),
                status: Some(AcpV1ActivityStatus::Pending),
            })
            .unwrap();
        let wait = control
            .begin_permission(
                "fixture-session",
                "read-source".to_owned(),
                "Read selected source".to_owned(),
                vec![
                    AcpV1PermissionOption {
                        option_id: "allow".to_owned(),
                        name: "Allow once".to_owned(),
                        kind: AcpV1PermissionKind::AllowOnce,
                        enabled: true,
                    },
                    AcpV1PermissionOption {
                        option_id: "deny".to_owned(),
                        name: "Deny once".to_owned(),
                        kind: AcpV1PermissionKind::RejectOnce,
                        enabled: true,
                    },
                    AcpV1PermissionOption {
                        option_id: "always".to_owned(),
                        name: "Allow always".to_owned(),
                        kind: AcpV1PermissionKind::AllowAlways,
                        enabled: false,
                    },
                ],
                Duration::from_secs(10),
            )
            .unwrap();
        let decision = AcpV1ControlRequest::Permission {
            binding: turn.binding.clone(),
            request_id: wait.request.request_id,
            choice: AcpV1PermissionChoice::Select {
                option_id: "deny".to_owned(),
            },
        };
        let cancel = AcpV1ControlRequest::Cancel {
            binding: turn.binding.clone(),
        };
        let mut unknown = serde_json::to_value(&decision).unwrap();
        unknown["unexpected"] = true.into();
        assert!(serde_json::from_value::<AcpV1ControlRequest>(unknown).is_err());
        control
            .apply(serde_json::from_value(serde_json::to_value(&decision).unwrap()).unwrap())
            .unwrap();
        let choice = futures::executor::block_on(wait.receiver).unwrap();
        control.permission_resolved(&wait.request, choice).unwrap();
        assert!(control.apply(decision.clone()).is_err());
        turn.status = AcpV1TurnStatus::Completed;
        drop(turn);
        drop(scope);
        assert!(control.apply(cancel.clone()).is_err());
        let updates: Vec<_> = std::iter::from_fn(|| control.try_next_update()).collect();
        let fixture = serde_json::json!({ "schemaVersion": 1, "controls": [decision, cancel], "updates": updates });
        let json = format!("{}\n", serde_json::to_string_pretty(&fixture).unwrap());
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/acp-control-stream-v1.json");
        if std::env::var_os("QIONGLI_UPDATE_ACP_FIXTURE").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &json).unwrap();
        }
        assert_eq!(std::fs::read_to_string(path).unwrap(), json);
        let decoded: Vec<AcpV1Update> = serde_json::from_value(fixture["updates"].clone()).unwrap();
        assert_eq!(decoded.len(), 8);
    }

    #[test]
    fn control_bounds_timeouts_and_recreated_connections_reject_stale_choices() {
        let control = control();
        let _scope = control.claim().unwrap();
        let turn = control
            .begin_turn(
                "fixture-session",
                1,
                CancellationToken::new(),
                Duration::from_secs(30),
            )
            .unwrap();
        let other = AcpV1Control::new(turn.binding.run_id.clone(), turn.binding.role).unwrap();
        let _other_scope = other.claim().unwrap();
        let _other_turn = other
            .begin_turn(
                "fixture-session",
                1,
                CancellationToken::new(),
                Duration::from_secs(30),
            )
            .unwrap();
        assert!(
            other
                .apply(AcpV1ControlRequest::Cancel {
                    binding: turn.binding.clone()
                })
                .is_err()
        );
        for _ in 0..MAX_UPDATES - 1 {
            control
                .emit(AcpV1UpdateKind::Turn {
                    binding: turn.binding.clone(),
                    status: AcpV1TurnStatus::Running,
                })
                .unwrap();
        }
        assert_eq!(
            control
                .emit(AcpV1UpdateKind::Turn {
                    binding: turn.binding.clone(),
                    status: AcpV1TurnStatus::Running
                })
                .unwrap_err()
                .code,
            AgentBackendErrorCode::ResponseInvalid
        );
        assert!(control.failure().is_err());

        let limited = self::control();
        let _scope = limited.claim().unwrap();
        let turn = limited
            .begin_turn(
                "fixture-session",
                1,
                CancellationToken::new(),
                Duration::from_secs(30),
            )
            .unwrap();
        let mut count = 0;
        while limited
            .emit(AcpV1UpdateKind::Text {
                binding: turn.binding.clone(),
                content: "x".repeat(65_536),
            })
            .is_ok()
        {
            count += 1;
        }
        assert!(count < 128);

        let expired = self::control();
        let _scope = expired.claim().unwrap();
        let turn = expired
            .begin_turn(
                "fixture-session",
                1,
                CancellationToken::new(),
                Duration::from_secs(30),
            )
            .unwrap();
        let wait = expired
            .begin_permission(
                "fixture-session",
                "tool".to_owned(),
                "Tool".to_owned(),
                vec![],
                Duration::ZERO,
            )
            .unwrap();
        assert!(
            expired
                .apply(AcpV1ControlRequest::Permission {
                    binding: turn.binding.clone(),
                    request_id: wait.request.request_id,
                    choice: AcpV1PermissionChoice::Cancel
                })
                .is_err()
        );
        drop(turn);
        assert!(futures::executor::block_on(wait.receiver).is_err());
        assert!(!expired.has_pending_permission());
        expired
            .permission_resolved(&wait.request, AcpV1PermissionChoice::Cancel)
            .unwrap();
        let updates: Vec<_> = std::iter::from_fn(|| expired.try_next_update()).collect();
        assert!(
            !updates
                .iter()
                .any(|update| matches!(update.kind, AcpV1UpdateKind::PermissionResolved { .. }))
        );
        assert!(matches!(
            updates.last().unwrap().kind,
            AcpV1UpdateKind::Turn {
                status: AcpV1TurnStatus::Interrupted,
                ..
            }
        ));
    }
}
