use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};

use crate::{
    BackendId, OrchestrationProfileV1, OrchestrationRole, OrchestrationRunStatus,
    OrchestrationTaskId, RunId,
};

pub const ALL_CHAT_STATE_SCHEMA_VERSION: u32 = 1;

const MAX_CHAT_EVENTS: usize = 1_024;
const MAX_CHAT_TEXT_BYTES: usize = 64 * 1_024;
const MAX_SESSION_ID_BYTES: usize = 256;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllChatStateError {
    InvalidState,
    InvalidEvent,
    StaleGeneration,
    UnexpectedSequence,
    InvalidTransition,
    TaskNotAssigned,
    LimitExceeded,
}

impl AllChatStateError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidState => "all-chat-state-invalid",
            Self::InvalidEvent => "all-chat-event-invalid",
            Self::StaleGeneration => "all-chat-generation-stale",
            Self::UnexpectedSequence => "all-chat-sequence-unexpected",
            Self::InvalidTransition => "all-chat-transition-invalid",
            Self::TaskNotAssigned => "all-chat-task-unassigned",
            Self::LimitExceeded => "all-chat-limit-exhausted",
        }
    }
}

impl Display for AllChatStateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for AllChatStateError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AllChatParticipantV1 {
    pub role: OrchestrationRole,
    pub backend_id: BackendId,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AllChatEventKindV1 {
    RunStarted {},
    UserMessage {
        content: String,
    },
    AgentSessionReady {
        role: OrchestrationRole,
        session_id: String,
    },
    TaskDelegated {
        by: OrchestrationRole,
        to: OrchestrationRole,
        task_id: OrchestrationTaskId,
        task_sha256: String,
    },
    TaskResult {
        by: OrchestrationRole,
        task_id: OrchestrationTaskId,
        result_sha256: String,
    },
    CoordinatorMessage {
        by: OrchestrationRole,
        content: String,
    },
    RunCompleted {
        by: OrchestrationRole,
    },
    RunFailed {},
    RunCancelled {},
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AllChatEventV1 {
    pub sequence: u64,
    pub generation: u64,
    pub kind: AllChatEventKindV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllChatStateV1 {
    schema_version: u32,
    run_id: RunId,
    project_id: ProjectId,
    expected_project_revision: u64,
    profile_sha256: String,
    generation: u64,
    status: OrchestrationRunStatus,
    participants: Vec<AllChatParticipantV1>,
    events: Vec<AllChatEventV1>,
}

impl AllChatStateV1 {
    pub fn try_new(
        run_id: RunId,
        project_id: ProjectId,
        expected_project_revision: u64,
        profile: &OrchestrationProfileV1,
    ) -> Result<Self, AllChatStateError> {
        if RunId::parse(run_id.as_str()).is_err()
            || ProjectId::parse(project_id.as_str()).is_err()
            || expected_project_revision == 0
            || expected_project_revision > MAX_SAFE_INTEGER
        {
            return Err(AllChatStateError::InvalidState);
        }

        let participants = profile
            .roles()
            .iter()
            .map(|role| {
                profile
                    .backend_for_role(*role)
                    .cloned()
                    .map(|backend_id| AllChatParticipantV1 {
                        role: *role,
                        backend_id,
                        session_id: None,
                    })
                    .ok_or(AllChatStateError::InvalidState)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if participants.len() > 3
            || participants
                .iter()
                .filter(|participant| participant.role == OrchestrationRole::Primary)
                .count()
                != 1
        {
            return Err(AllChatStateError::InvalidState);
        }

        Ok(Self {
            schema_version: ALL_CHAT_STATE_SCHEMA_VERSION,
            run_id,
            project_id,
            expected_project_revision,
            profile_sha256: profile
                .digest()
                .map_err(|_| AllChatStateError::InvalidState)?,
            generation: 0,
            status: OrchestrationRunStatus::Planned,
            participants,
            events: Vec::new(),
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    #[must_use]
    pub const fn run_id(&self) -> &RunId {
        &self.run_id
    }

    #[must_use]
    pub const fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    #[must_use]
    pub const fn expected_project_revision(&self) -> u64 {
        self.expected_project_revision
    }

    #[must_use]
    pub fn profile_sha256(&self) -> &str {
        &self.profile_sha256
    }

    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub const fn status(&self) -> OrchestrationRunStatus {
        self.status
    }

    #[must_use]
    pub fn participants(&self) -> &[AllChatParticipantV1] {
        &self.participants
    }

    #[must_use]
    pub fn events(&self) -> &[AllChatEventV1] {
        &self.events
    }

    pub fn append_event(
        &mut self,
        expected_generation: u64,
        sequence: u64,
        kind: AllChatEventKindV1,
    ) -> Result<(), AllChatStateError> {
        if expected_generation != self.generation {
            return Err(AllChatStateError::StaleGeneration);
        }
        if self.events.len() >= MAX_CHAT_EVENTS || self.generation >= MAX_SAFE_INTEGER {
            return Err(AllChatStateError::LimitExceeded);
        }
        let next_sequence = self
            .events
            .last()
            .map_or(Some(1), |event| event.sequence.checked_add(1))
            .filter(|next| *next <= MAX_SAFE_INTEGER)
            .ok_or(AllChatStateError::LimitExceeded)?;
        if sequence != next_sequence {
            return Err(AllChatStateError::UnexpectedSequence);
        }

        self.apply_event(&kind)?;
        self.generation += 1;
        self.events.push(AllChatEventV1 {
            sequence,
            generation: self.generation,
            kind,
        });
        Ok(())
    }

    fn apply_event(&mut self, kind: &AllChatEventKindV1) -> Result<(), AllChatStateError> {
        if self.status.is_terminal() {
            return Err(AllChatStateError::InvalidTransition);
        }
        match kind {
            AllChatEventKindV1::RunStarted {} => {
                if self.status != OrchestrationRunStatus::Planned || !self.events.is_empty() {
                    return Err(AllChatStateError::InvalidTransition);
                }
                self.status = OrchestrationRunStatus::Running;
            }
            _ if self.status != OrchestrationRunStatus::Running => {
                return Err(AllChatStateError::InvalidTransition);
            }
            AllChatEventKindV1::UserMessage { content } => {
                require_chat_text(content)?;
            }
            AllChatEventKindV1::AgentSessionReady { role, session_id } => {
                if !valid_session_id(session_id)
                    || self.participants.iter().any(|participant| {
                        participant.session_id.as_deref() == Some(session_id.as_str())
                    })
                {
                    return Err(AllChatStateError::InvalidEvent);
                }
                let participant = self
                    .participants
                    .iter_mut()
                    .find(|participant| participant.role == *role)
                    .ok_or(AllChatStateError::InvalidEvent)?;
                if participant.session_id.is_some() {
                    return Err(AllChatStateError::InvalidTransition);
                }
                participant.session_id = Some(session_id.clone());
            }
            AllChatEventKindV1::TaskDelegated {
                by,
                to,
                task_id,
                task_sha256,
            } => {
                if *by != OrchestrationRole::Primary
                    || *to == OrchestrationRole::Primary
                    || !self.participant_ready(*by)
                    || !self.participant_ready(*to)
                {
                    return Err(AllChatStateError::InvalidTransition);
                }
                if OrchestrationTaskId::parse(task_id.as_str()).is_err()
                    || !valid_sha256(task_sha256)
                {
                    return Err(AllChatStateError::InvalidEvent);
                }
                if self.assignment_for(task_id).is_some() {
                    return Err(AllChatStateError::InvalidTransition);
                }
            }
            AllChatEventKindV1::TaskResult {
                by,
                task_id,
                result_sha256,
            } => {
                if *by == OrchestrationRole::Primary || !self.participant_ready(*by) {
                    return Err(AllChatStateError::InvalidTransition);
                }
                if OrchestrationTaskId::parse(task_id.as_str()).is_err()
                    || !valid_sha256(result_sha256)
                {
                    return Err(AllChatStateError::InvalidEvent);
                }
                if self.assignment_for(task_id) != Some(*by) || self.has_result(task_id) {
                    return Err(AllChatStateError::TaskNotAssigned);
                }
            }
            AllChatEventKindV1::CoordinatorMessage { by, content } => {
                if *by != OrchestrationRole::Primary || !self.participant_ready(*by) {
                    return Err(AllChatStateError::InvalidTransition);
                }
                require_chat_text(content)?;
            }
            AllChatEventKindV1::RunCompleted { by } => {
                if *by != OrchestrationRole::Primary
                    || !self.participant_ready(*by)
                    || self.has_pending_assignments()
                {
                    return Err(AllChatStateError::InvalidTransition);
                }
                self.status = OrchestrationRunStatus::Completed;
            }
            AllChatEventKindV1::RunFailed {} => {
                self.status = OrchestrationRunStatus::Failed;
            }
            AllChatEventKindV1::RunCancelled {} => {
                self.status = OrchestrationRunStatus::Cancelled;
            }
        }
        Ok(())
    }

    fn participant_ready(&self, role: OrchestrationRole) -> bool {
        self.participants
            .iter()
            .any(|participant| participant.role == role && participant.session_id.is_some())
    }

    fn assignment_for(&self, task_id: &OrchestrationTaskId) -> Option<OrchestrationRole> {
        self.events.iter().find_map(|event| match &event.kind {
            AllChatEventKindV1::TaskDelegated {
                to,
                task_id: assigned_task,
                ..
            } if assigned_task == task_id => Some(*to),
            _ => None,
        })
    }

    fn has_result(&self, task_id: &OrchestrationTaskId) -> bool {
        self.events.iter().any(|event| {
            matches!(
                &event.kind,
                AllChatEventKindV1::TaskResult {
                    task_id: completed_task,
                    ..
                } if completed_task == task_id
            )
        })
    }

    fn has_pending_assignments(&self) -> bool {
        let mut pending = BTreeSet::new();
        for event in &self.events {
            match &event.kind {
                AllChatEventKindV1::TaskDelegated { task_id, .. } => {
                    pending.insert(task_id.clone());
                }
                AllChatEventKindV1::TaskResult { task_id, .. } => {
                    pending.remove(task_id);
                }
                _ => {}
            }
        }
        !pending.is_empty()
    }
}

fn require_chat_text(value: &str) -> Result<(), AllChatStateError> {
    (!value.is_empty()
        && value.len() <= MAX_CHAT_TEXT_BYTES
        && value
            .chars()
            .all(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t')))
    .then_some(())
    .ok_or(AllChatStateError::InvalidEvent)
}

fn valid_session_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SESSION_ID_BYTES
        && value.chars().all(|character| !character.is_control())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrchestrationExecutionMode;

    fn digest(character: char) -> String {
        character.to_string().repeat(64)
    }

    fn profile(mode: OrchestrationExecutionMode) -> OrchestrationProfileV1 {
        OrchestrationProfileV1::try_new(
            "all-chat-test",
            mode,
            BackendId::parse("codex-acp").unwrap(),
            (mode != OrchestrationExecutionMode::Solo)
                .then(|| BackendId::parse("claude-acp").unwrap()),
            (mode == OrchestrationExecutionMode::Triad)
                .then(|| BackendId::parse("codex-review-acp").unwrap()),
            1,
            true,
        )
        .unwrap()
    }

    fn new_state() -> AllChatStateV1 {
        AllChatStateV1::try_new(
            RunId::parse(format!("run_{}", "1".repeat(32))).unwrap(),
            ProjectId::parse(format!("prj_{}", "2".repeat(32))).unwrap(),
            7,
            &profile(OrchestrationExecutionMode::Triad),
        )
        .unwrap()
    }

    fn commit(state: &mut AllChatStateV1, kind: AllChatEventKindV1) {
        let generation = state.generation();
        let sequence = u64::try_from(state.events().len()).unwrap() + 1;
        state.append_event(generation, sequence, kind).unwrap();
    }

    fn running_state() -> AllChatStateV1 {
        let mut state = new_state();
        commit(&mut state, AllChatEventKindV1::RunStarted {});
        for (role, session_id) in [
            (OrchestrationRole::Primary, "session-primary"),
            (OrchestrationRole::Reviewer, "session-reviewer"),
            (OrchestrationRole::Verifier, "session-verifier"),
        ] {
            commit(
                &mut state,
                AllChatEventKindV1::AgentSessionReady {
                    role,
                    session_id: session_id.to_string(),
                },
            );
        }
        state
    }

    #[test]
    fn coordinator_and_two_collaborators_complete_one_ordered_flow() {
        let mut state = running_state();
        assert_eq!(
            state
                .participants()
                .iter()
                .filter(|participant| participant.role == OrchestrationRole::Primary)
                .count(),
            1
        );
        assert_eq!(state.participants().len(), 3);

        commit(
            &mut state,
            AllChatEventKindV1::UserMessage {
                content: "Compare the evidence.".to_string(),
            },
        );
        for (to, task_id, task_digest) in [
            (OrchestrationRole::Reviewer, "review", digest('a')),
            (OrchestrationRole::Verifier, "verify", digest('b')),
        ] {
            commit(
                &mut state,
                AllChatEventKindV1::TaskDelegated {
                    by: OrchestrationRole::Primary,
                    to,
                    task_id: OrchestrationTaskId::parse(task_id).unwrap(),
                    task_sha256: task_digest,
                },
            );
        }
        for (by, task_id, result_digest) in [
            (OrchestrationRole::Reviewer, "review", digest('c')),
            (OrchestrationRole::Verifier, "verify", digest('d')),
        ] {
            commit(
                &mut state,
                AllChatEventKindV1::TaskResult {
                    by,
                    task_id: OrchestrationTaskId::parse(task_id).unwrap(),
                    result_sha256: result_digest,
                },
            );
        }
        commit(
            &mut state,
            AllChatEventKindV1::CoordinatorMessage {
                by: OrchestrationRole::Primary,
                content: "Both bounded results are ready.".to_string(),
            },
        );
        commit(
            &mut state,
            AllChatEventKindV1::RunCompleted {
                by: OrchestrationRole::Primary,
            },
        );

        assert_eq!(state.status(), OrchestrationRunStatus::Completed);
        assert_eq!(state.generation(), 11);
        assert!(state.profile_sha256().len() == 64);
        assert!(state.events().iter().enumerate().all(|(index, event)| {
            event.sequence == u64::try_from(index).unwrap() + 1
                && event.generation == event.sequence
        }));
    }

    #[test]
    fn invalid_transition_table_fails_closed() {
        assert!(
            serde_json::from_str::<AllChatEventKindV1>(
                r#"{"type":"run_started","unexpected":true}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<AllChatEventKindV1>(
                r#"{"type":"user_message","content":"hello","unexpected":true}"#
            )
            .is_err()
        );

        type Case = fn(&mut AllChatStateV1) -> Result<(), AllChatStateError>;

        fn stale_generation(state: &mut AllChatStateV1) -> Result<(), AllChatStateError> {
            state.append_event(
                state.generation() + 1,
                u64::try_from(state.events().len()).unwrap() + 1,
                AllChatEventKindV1::UserMessage {
                    content: "stale".to_string(),
                },
            )
        }

        fn unexpected_sequence(state: &mut AllChatStateV1) -> Result<(), AllChatStateError> {
            state.append_event(
                state.generation(),
                u64::try_from(state.events().len()).unwrap() + 2,
                AllChatEventKindV1::UserMessage {
                    content: "out of order".to_string(),
                },
            )
        }

        fn worker_delegation(state: &mut AllChatStateV1) -> Result<(), AllChatStateError> {
            state.append_event(
                state.generation(),
                u64::try_from(state.events().len()).unwrap() + 1,
                AllChatEventKindV1::TaskDelegated {
                    by: OrchestrationRole::Reviewer,
                    to: OrchestrationRole::Verifier,
                    task_id: OrchestrationTaskId::parse("nested").unwrap(),
                    task_sha256: digest('e'),
                },
            )
        }

        fn unassigned_result(state: &mut AllChatStateV1) -> Result<(), AllChatStateError> {
            state.append_event(
                state.generation(),
                u64::try_from(state.events().len()).unwrap() + 1,
                AllChatEventKindV1::TaskResult {
                    by: OrchestrationRole::Reviewer,
                    task_id: OrchestrationTaskId::parse("missing").unwrap(),
                    result_sha256: digest('f'),
                },
            )
        }

        fn complete_with_pending_task(state: &mut AllChatStateV1) -> Result<(), AllChatStateError> {
            commit(
                state,
                AllChatEventKindV1::TaskDelegated {
                    by: OrchestrationRole::Primary,
                    to: OrchestrationRole::Reviewer,
                    task_id: OrchestrationTaskId::parse("pending").unwrap(),
                    task_sha256: digest('a'),
                },
            );
            state.append_event(
                state.generation(),
                u64::try_from(state.events().len()).unwrap() + 1,
                AllChatEventKindV1::RunCompleted {
                    by: OrchestrationRole::Primary,
                },
            )
        }

        fn event_after_terminal(state: &mut AllChatStateV1) -> Result<(), AllChatStateError> {
            commit(state, AllChatEventKindV1::RunCancelled {});
            state.append_event(
                state.generation(),
                u64::try_from(state.events().len()).unwrap() + 1,
                AllChatEventKindV1::UserMessage {
                    content: "too late".to_string(),
                },
            )
        }

        let cases: [(&str, Case, AllChatStateError); 6] = [
            (
                "stale generation",
                stale_generation,
                AllChatStateError::StaleGeneration,
            ),
            (
                "unexpected sequence",
                unexpected_sequence,
                AllChatStateError::UnexpectedSequence,
            ),
            (
                "worker delegation",
                worker_delegation,
                AllChatStateError::InvalidTransition,
            ),
            (
                "unassigned result",
                unassigned_result,
                AllChatStateError::TaskNotAssigned,
            ),
            (
                "completion with pending task",
                complete_with_pending_task,
                AllChatStateError::InvalidTransition,
            ),
            (
                "event after terminal state",
                event_after_terminal,
                AllChatStateError::InvalidTransition,
            ),
        ];

        for (name, apply, expected) in cases {
            assert_eq!(apply(&mut running_state()), Err(expected), "{name}");
        }
    }
}
