use qiongli_execution::{
    AgentFinishReason, AllChatEventKindV1, AllChatStateV1, BackendId, OrchestrationExecutionMode,
    OrchestrationProfileV1, OrchestrationRole, OrchestrationRunStatus, RunId,
};
use qiongli_project::ProjectId;
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::Serialize;

pub const ALL_CHAT_APP_CONTRACT_SCHEMA_VERSION: u32 = 1;
pub const ALL_CHAT_APP_CONTRACT_SCHEMA_ID: &str =
    "https://qiongli.dev/schemas/app/all-chat-app-v1.json";

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[schemars(title = "Qiongli All Chat App Contract v1")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AllChatAppSnapshotV1 {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    #[schemars(length(equal = 36), regex(pattern = r"^run_[0-9a-f]{32}$"))]
    run_id: String,
    #[schemars(length(equal = 36), regex(pattern = r"^prj_[0-9a-f]{32}$"))]
    project_id: String,
    #[schemars(range(min = 1, max = MAX_SAFE_INTEGER))]
    expected_project_revision: u64,
    #[schemars(range(max = MAX_SAFE_INTEGER))]
    generation: u64,
    status: AllChatAppRunStatusV1,
    #[schemars(length(min = 1, max = 3))]
    participants: Vec<AllChatAppParticipantV1>,
    #[schemars(length(max = 1_024))]
    events: Vec<AllChatAppEventV1>,
}

#[derive(Clone, Copy, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
enum AllChatAppRunStatusV1 {
    Planned,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
enum AllChatAppRoleV1 {
    Primary,
    Reviewer,
    Verifier,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AllChatAppParticipantV1 {
    role: AllChatAppRoleV1,
    #[schemars(
        length(min = 1, max = 64),
        regex(pattern = r"^[a-z0-9][a-z0-9._-]{0,63}$")
    )]
    backend_id: String,
    session_id: Option<AllChatAppSessionIdV1>,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(transparent)]
struct AllChatAppSessionIdV1(#[schemars(length(min = 1, max = 256))] String);

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AllChatAppEventV1 {
    #[schemars(range(min = 1, max = MAX_SAFE_INTEGER))]
    sequence: u64,
    #[schemars(range(min = 1, max = MAX_SAFE_INTEGER))]
    generation: u64,
    kind: AllChatAppEventKindV1,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum AllChatAppEventKindV1 {
    RunStarted {},
    UserMessage {
        #[schemars(length(min = 1, max = 65_536))]
        content: String,
    },
    AgentSessionReady {
        role: AllChatAppRoleV1,
        session_id: AllChatAppSessionIdV1,
    },
    TaskDelegated {
        by: AllChatAppRoleV1,
        to: AllChatAppRoleV1,
        #[schemars(
            length(min = 1, max = 64),
            regex(pattern = r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")
        )]
        task_id: String,
        #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
        task_sha256: String,
    },
    TaskResult {
        by: AllChatAppRoleV1,
        #[schemars(
            length(min = 1, max = 64),
            regex(pattern = r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")
        )]
        task_id: String,
        #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
        result_sha256: String,
    },
    CoordinatorMessage {
        by: AllChatAppRoleV1,
        #[schemars(length(min = 1, max = 65_536))]
        content: String,
    },
    AgentTurnCompleted {
        by: AllChatAppRoleV1,
        finish_reason: AllChatAppFinishReasonV1,
    },
    AgentTurnCancelled {
        by: AllChatAppRoleV1,
    },
    RunCompleted {
        by: AllChatAppRoleV1,
    },
    RunFailed {},
    RunCancelled {},
}

#[derive(Clone, Copy, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
enum AllChatAppFinishReasonV1 {
    Stop,
    Length,
}

pub fn project_all_chat_app_snapshot(
    state: &AllChatStateV1,
) -> Result<AllChatAppSnapshotV1, &'static str> {
    let participants = state
        .participants()
        .iter()
        .map(|participant| AllChatAppParticipantV1 {
            role: participant.role.into(),
            backend_id: participant.backend_id.as_str().to_owned(),
            session_id: participant
                .session_id
                .as_ref()
                .map(|session_id| AllChatAppSessionIdV1(session_id.clone())),
        })
        .collect();
    let events = state
        .events()
        .iter()
        .map(|event| {
            Ok(AllChatAppEventV1 {
                sequence: event.sequence,
                generation: event.generation,
                kind: project_event_kind(&event.kind)?,
            })
        })
        .collect::<Result<Vec<_>, &'static str>>()?;

    Ok(AllChatAppSnapshotV1 {
        schema_version: ALL_CHAT_APP_CONTRACT_SCHEMA_VERSION,
        run_id: state.run_id().as_str().to_owned(),
        project_id: state.project_id().as_str().to_owned(),
        expected_project_revision: state.expected_project_revision(),
        generation: state.generation(),
        status: state.status().into(),
        participants,
        events,
    })
}

pub fn all_chat_app_contract_schema_json() -> Result<String, &'static str> {
    let mut schema = SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator()
        .into_root_schema_for::<AllChatAppSnapshotV1>();
    schema.insert("$id".to_owned(), ALL_CHAT_APP_CONTRACT_SCHEMA_ID.into());
    serde_json::to_string_pretty(&schema)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|_| "all-chat-app-schema-serialization-failed")
}

pub fn all_chat_app_contract_completed_fixture_json() -> Result<String, &'static str> {
    serialize_fixture(false)
}

pub fn all_chat_app_contract_cancelled_fixture_json() -> Result<String, &'static str> {
    serialize_fixture(true)
}

fn project_event_kind(kind: &AllChatEventKindV1) -> Result<AllChatAppEventKindV1, &'static str> {
    Ok(match kind {
        AllChatEventKindV1::RunStarted {} => AllChatAppEventKindV1::RunStarted {},
        AllChatEventKindV1::UserMessage { content } => AllChatAppEventKindV1::UserMessage {
            content: content.clone(),
        },
        AllChatEventKindV1::AgentSessionReady { role, session_id } => {
            AllChatAppEventKindV1::AgentSessionReady {
                role: (*role).into(),
                session_id: AllChatAppSessionIdV1(session_id.clone()),
            }
        }
        AllChatEventKindV1::TaskDelegated {
            by,
            to,
            task_id,
            task_sha256,
        } => AllChatAppEventKindV1::TaskDelegated {
            by: (*by).into(),
            to: (*to).into(),
            task_id: task_id.as_str().to_owned(),
            task_sha256: task_sha256.clone(),
        },
        AllChatEventKindV1::TaskResult {
            by,
            task_id,
            result_sha256,
        } => AllChatAppEventKindV1::TaskResult {
            by: (*by).into(),
            task_id: task_id.as_str().to_owned(),
            result_sha256: result_sha256.clone(),
        },
        AllChatEventKindV1::CoordinatorMessage { by, content } => {
            AllChatAppEventKindV1::CoordinatorMessage {
                by: (*by).into(),
                content: content.clone(),
            }
        }
        AllChatEventKindV1::AgentTurnCompleted { by, finish_reason } => {
            AllChatAppEventKindV1::AgentTurnCompleted {
                by: (*by).into(),
                finish_reason: match finish_reason {
                    AgentFinishReason::Stop => AllChatAppFinishReasonV1::Stop,
                    AgentFinishReason::Length => AllChatAppFinishReasonV1::Length,
                    AgentFinishReason::ToolRequest => {
                        return Err("all-chat-app-event-unsupported");
                    }
                },
            }
        }
        AllChatEventKindV1::AgentTurnCancelled { by } => {
            AllChatAppEventKindV1::AgentTurnCancelled { by: (*by).into() }
        }
        AllChatEventKindV1::RunCompleted { by } => {
            AllChatAppEventKindV1::RunCompleted { by: (*by).into() }
        }
        AllChatEventKindV1::RunFailed {} => AllChatAppEventKindV1::RunFailed {},
        AllChatEventKindV1::RunCancelled {} => AllChatAppEventKindV1::RunCancelled {},
    })
}

fn serialize_fixture(cancelled: bool) -> Result<String, &'static str> {
    let profile = OrchestrationProfileV1::try_new(
        "all-chat-app-contract",
        OrchestrationExecutionMode::Triad,
        BackendId::parse("codex-acp").map_err(|_| "all-chat-app-fixture-invalid")?,
        Some(BackendId::parse("claude-agent-acp").map_err(|_| "all-chat-app-fixture-invalid")?),
        Some(BackendId::parse("codex-review-acp").map_err(|_| "all-chat-app-fixture-invalid")?),
        1,
        true,
    )
    .map_err(|_| "all-chat-app-fixture-invalid")?;
    let run_digit = if cancelled { '2' } else { '1' };
    let mut state = AllChatStateV1::try_new(
        RunId::parse(format!("run_{}", run_digit.to_string().repeat(32)))
            .map_err(|_| "all-chat-app-fixture-invalid")?,
        ProjectId::parse(format!("prj_{}", "3".repeat(32)))
            .map_err(|_| "all-chat-app-fixture-invalid")?,
        7,
        &profile,
    )
    .map_err(|_| "all-chat-app-fixture-invalid")?;
    append_fixture_event(&mut state, AllChatEventKindV1::RunStarted {})?;
    append_fixture_event(
        &mut state,
        AllChatEventKindV1::AgentSessionReady {
            role: OrchestrationRole::Primary,
            session_id: if cancelled {
                "opaque-session-cancelled"
            } else {
                "opaque-session-completed"
            }
            .to_owned(),
        },
    )?;
    if cancelled {
        append_fixture_event(
            &mut state,
            AllChatEventKindV1::AgentTurnCancelled {
                by: OrchestrationRole::Primary,
            },
        )?;
    } else {
        append_fixture_event(
            &mut state,
            AllChatEventKindV1::UserMessage {
                content: "Compare the available evidence.".to_owned(),
            },
        )?;
        append_fixture_event(
            &mut state,
            AllChatEventKindV1::CoordinatorMessage {
                by: OrchestrationRole::Primary,
                content: "The first bounded comparison is ready.".to_owned(),
            },
        )?;
        append_fixture_event(
            &mut state,
            AllChatEventKindV1::AgentTurnCompleted {
                by: OrchestrationRole::Primary,
                finish_reason: AgentFinishReason::Stop,
            },
        )?;
    }
    serde_json::to_string_pretty(&project_all_chat_app_snapshot(&state)?)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|_| "all-chat-app-fixture-serialization-failed")
}

fn append_fixture_event(
    state: &mut AllChatStateV1,
    kind: AllChatEventKindV1,
) -> Result<(), &'static str> {
    let sequence = u64::try_from(state.events().len())
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or("all-chat-app-fixture-invalid")?;
    state
        .append_event(state.generation(), sequence, kind)
        .map_err(|_| "all-chat-app-fixture-invalid")
}

impl From<OrchestrationRole> for AllChatAppRoleV1 {
    fn from(value: OrchestrationRole) -> Self {
        match value {
            OrchestrationRole::Primary => Self::Primary,
            OrchestrationRole::Reviewer => Self::Reviewer,
            OrchestrationRole::Verifier => Self::Verifier,
        }
    }
}

impl From<OrchestrationRunStatus> for AllChatAppRunStatusV1 {
    fn from(value: OrchestrationRunStatus) -> Self {
        match value {
            OrchestrationRunStatus::Planned => Self::Planned,
            OrchestrationRunStatus::Running => Self::Running,
            OrchestrationRunStatus::Paused => Self::Paused,
            OrchestrationRunStatus::Completed => Self::Completed,
            OrchestrationRunStatus::Failed => Self::Failed,
            OrchestrationRunStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const SCHEMA_GOLDEN: &str = include_str!("../schemas/all-chat-app-v1.schema.json");
    const COMPLETED_GOLDEN: &str = include_str!("../tests/fixtures/all-chat-app-v1.completed.json");
    const CANCELLED_GOLDEN: &str = include_str!("../tests/fixtures/all-chat-app-v1.cancelled.json");

    #[test]
    fn generated_schema_and_representative_fixtures_are_stable() {
        let schema = all_chat_app_contract_schema_json().expect("schema must serialize");
        let completed = all_chat_app_contract_completed_fixture_json()
            .expect("completed fixture must serialize");
        let cancelled = all_chat_app_contract_cancelled_fixture_json()
            .expect("cancelled fixture must serialize");

        assert_eq!(schema, SCHEMA_GOLDEN);
        assert_eq!(completed, COMPLETED_GOLDEN);
        assert_eq!(cancelled, CANCELLED_GOLDEN);

        let schema: Value = serde_json::from_str(&schema).expect("schema must be JSON");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema["$id"], ALL_CHAT_APP_CONTRACT_SCHEMA_ID);
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["properties"]["generation"]["maximum"],
            MAX_SAFE_INTEGER
        );
        for definition in ["AllChatAppParticipantV1", "AllChatAppEventV1"] {
            assert_eq!(schema["$defs"][definition]["additionalProperties"], false);
        }
        assert!(
            schema["$defs"]["AllChatAppEventKindV1"]["oneOf"]
                .as_array()
                .expect("event union must be an array")
                .iter()
                .all(|variant| variant["additionalProperties"] == false)
        );

        let completed: Value = serde_json::from_str(&completed).expect("fixture must be JSON");
        assert_eq!(
            completed["schemaVersion"],
            ALL_CHAT_APP_CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(completed["status"], "running");
        assert_eq!(
            completed["events"][3]["kind"]["type"],
            "coordinator_message"
        );
        assert_eq!(
            completed["events"][4]["kind"]["type"],
            "agent_turn_completed"
        );

        let cancelled: Value = serde_json::from_str(&cancelled).expect("fixture must be JSON");
        assert_eq!(cancelled["status"], "running");
        assert_eq!(
            cancelled["events"][2]["kind"]["type"],
            "agent_turn_cancelled"
        );
        assert!(
            cancelled["events"]
                .as_array()
                .expect("cancelled events must be an array")
                .iter()
                .all(|event| event["kind"]["type"] != "coordinator_message")
        );
    }
}
