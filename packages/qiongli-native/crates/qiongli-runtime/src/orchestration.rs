use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::contract::LiteOrchestrationHandler;

const MAX_ROUTE_REQUEST_BYTES: usize = 4_096;
const MAX_TASK_ID_BYTES: usize = 256;
const MAX_PAPER_TYPE_BYTES: usize = 256;
const MAX_TOPIC_BYTES: usize = 4_096;
const ROUTE_ARGUMENTS: [&str; 2] = ["request", "platform"];
const TASK_PLAN_ARGUMENTS: [&str; 3] = ["task_id", "paper_type", "topic"];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LitePlatform {
    Codex,
    ClaudeCode,
    Claude,
    Antigravity,
    Cli,
    Unknown,
}

impl LitePlatform {
    fn from_identifier(identifier: &str) -> Result<Self, OrchestrationError> {
        match identifier {
            "codex" => Ok(Self::Codex),
            "claude_code" => Ok(Self::ClaudeCode),
            "claude" => Ok(Self::Claude),
            "antigravity" => Ok(Self::Antigravity),
            "cli" => Ok(Self::Cli),
            "unknown" => Ok(Self::Unknown),
            _ => Err(OrchestrationError::UnsupportedPlatform),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum OrchestrationError {
    #[error("tool arguments must be an object")]
    ArgumentsNotObject,
    #[error("Unsupported argument")]
    UnsupportedArgument,
    #[error("Missing request")]
    MissingRequest,
    #[error("request must be a string")]
    RequestNotString,
    #[error("request must not be empty")]
    EmptyRequest,
    #[error("request exceeds the byte limit")]
    RequestTooLong,
    #[error("platform must be a string")]
    PlatformNotString,
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("Missing task_id")]
    MissingTaskId,
    #[error("task_id must be a string")]
    TaskIdNotString,
    #[error("task_id must not be empty")]
    EmptyTaskId,
    #[error("task_id exceeds the byte limit")]
    TaskIdTooLong,
    #[error("Missing paper_type")]
    MissingPaperType,
    #[error("paper_type must be a string")]
    PaperTypeNotString,
    #[error("paper_type must not be empty")]
    EmptyPaperType,
    #[error("paper_type exceeds the byte limit")]
    PaperTypeTooLong,
    #[error("Missing topic")]
    MissingTopic,
    #[error("topic must be a string")]
    TopicNotString,
    #[error("topic must not be empty")]
    EmptyTopic,
    #[error("topic exceeds the byte limit")]
    TopicTooLong,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrchestratorRouteInput {
    request: String,
    platform: LitePlatform,
}

impl OrchestratorRouteInput {
    pub fn try_new(
        request: impl Into<String>,
        platform: Option<&str>,
    ) -> Result<Self, OrchestrationError> {
        let request = request.into();
        validate_route_request(&request)?;
        let platform = platform
            .map(LitePlatform::from_identifier)
            .transpose()?
            .unwrap_or(LitePlatform::Unknown);
        Ok(Self { request, platform })
    }

    pub fn from_arguments(arguments: &Value) -> Result<Self, OrchestrationError> {
        let entries = arguments
            .as_object()
            .ok_or(OrchestrationError::ArgumentsNotObject)?;
        reject_unknown_arguments(entries, &ROUTE_ARGUMENTS)?;
        let request = entries
            .get("request")
            .ok_or(OrchestrationError::MissingRequest)?
            .as_str()
            .ok_or(OrchestrationError::RequestNotString)?;
        let platform = entries
            .get("platform")
            .map(|value| value.as_str().ok_or(OrchestrationError::PlatformNotString))
            .transpose()?;
        Self::try_new(request, platform)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TaskPlanInput {
    task_id: String,
    paper_type: String,
    topic: String,
}

impl TaskPlanInput {
    pub fn try_new(
        task_id: impl Into<String>,
        paper_type: impl Into<String>,
        topic: impl Into<String>,
    ) -> Result<Self, OrchestrationError> {
        let task_id = normalize_task_field(
            task_id.into(),
            MAX_TASK_ID_BYTES,
            OrchestrationError::EmptyTaskId,
            OrchestrationError::TaskIdTooLong,
        )?;
        let paper_type = normalize_task_field(
            paper_type.into(),
            MAX_PAPER_TYPE_BYTES,
            OrchestrationError::EmptyPaperType,
            OrchestrationError::PaperTypeTooLong,
        )?;
        let topic = normalize_task_field(
            topic.into(),
            MAX_TOPIC_BYTES,
            OrchestrationError::EmptyTopic,
            OrchestrationError::TopicTooLong,
        )?;
        Ok(Self {
            task_id,
            paper_type,
            topic,
        })
    }

    pub fn from_arguments(arguments: &Value) -> Result<Self, OrchestrationError> {
        let entries = arguments
            .as_object()
            .ok_or(OrchestrationError::ArgumentsNotObject)?;
        reject_unknown_arguments(entries, &TASK_PLAN_ARGUMENTS)?;
        let task_id = required_string(
            entries,
            "task_id",
            OrchestrationError::MissingTaskId,
            OrchestrationError::TaskIdNotString,
        )?;
        let paper_type = required_string(
            entries,
            "paper_type",
            OrchestrationError::MissingPaperType,
            OrchestrationError::PaperTypeNotString,
        )?;
        let topic = required_string(
            entries,
            "topic",
            OrchestrationError::MissingTopic,
            OrchestrationError::TopicNotString,
        )?;
        Self::try_new(task_id, paper_type, topic)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct FullUpgradeRecommendation {
    pub required_for_execution: bool,
    pub runtime_profile: &'static str,
    pub command: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct OrchestratorRoutePreview {
    pub mode: &'static str,
    pub preview_only: bool,
    pub runtime_profile: &'static str,
    pub run_agents_allowed: bool,
    pub shell_execution_allowed: bool,
    pub project_writes_allowed: bool,
    pub request: String,
    pub platform: LitePlatform,
    pub recommended_runtime: &'static str,
    pub upgrade: FullUpgradeRecommendation,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct TaskPlanPreview {
    pub mode: &'static str,
    pub preview_only: bool,
    pub runtime_profile: &'static str,
    pub run_agents_allowed: bool,
    pub shell_execution_allowed: bool,
    pub project_writes_allowed: bool,
    pub recommended_runtime: &'static str,
    pub upgrade: FullUpgradeRecommendation,
    pub task_id: String,
    pub paper_type: String,
    pub topic: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum LiteOrchestrationPreview {
    Route(OrchestratorRoutePreview),
    TaskPlan(TaskPlanPreview),
}

#[must_use]
pub fn build_orchestrator_route(input: OrchestratorRouteInput) -> OrchestratorRoutePreview {
    OrchestratorRoutePreview {
        mode: "preview",
        preview_only: true,
        runtime_profile: "marketplace_lite",
        run_agents_allowed: false,
        shell_execution_allowed: false,
        project_writes_allowed: false,
        request: input.request,
        platform: input.platform,
        recommended_runtime: "full_cli_for_execution",
        upgrade: full_upgrade_recommendation(),
    }
}

#[must_use]
pub fn build_task_plan(input: TaskPlanInput) -> TaskPlanPreview {
    TaskPlanPreview {
        mode: "preview",
        preview_only: true,
        runtime_profile: "marketplace_lite",
        run_agents_allowed: false,
        shell_execution_allowed: false,
        project_writes_allowed: false,
        recommended_runtime: "full_cli_for_execution",
        upgrade: full_upgrade_recommendation(),
        task_id: input.task_id,
        paper_type: input.paper_type,
        topic: input.topic,
    }
}

pub fn dispatch_lite_orchestration(
    handler: LiteOrchestrationHandler,
    arguments: &Value,
) -> Result<LiteOrchestrationPreview, OrchestrationError> {
    match handler {
        LiteOrchestrationHandler::Route => Ok(LiteOrchestrationPreview::Route(
            build_orchestrator_route(OrchestratorRouteInput::from_arguments(arguments)?),
        )),
        LiteOrchestrationHandler::TaskPlan => Ok(LiteOrchestrationPreview::TaskPlan(
            build_task_plan(TaskPlanInput::from_arguments(arguments)?),
        )),
    }
}

fn full_upgrade_recommendation() -> FullUpgradeRecommendation {
    FullUpgradeRecommendation {
        required_for_execution: true,
        runtime_profile: "full_cli",
        command: "qiongli mcp serve --transport stdio",
    }
}

fn validate_route_request(request: &str) -> Result<(), OrchestrationError> {
    if request.trim().is_empty() {
        return Err(OrchestrationError::EmptyRequest);
    }
    if request.len() > MAX_ROUTE_REQUEST_BYTES {
        return Err(OrchestrationError::RequestTooLong);
    }
    Ok(())
}

fn normalize_task_field(
    value: String,
    max_bytes: usize,
    empty_error: OrchestrationError,
    too_long_error: OrchestrationError,
) -> Result<String, OrchestrationError> {
    if value.len() > max_bytes {
        return Err(too_long_error);
    }
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(empty_error);
    }
    Ok(normalized.to_owned())
}

fn reject_unknown_arguments(
    entries: &Map<String, Value>,
    allowed: &[&str],
) -> Result<(), OrchestrationError> {
    if entries.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(OrchestrationError::UnsupportedArgument);
    }
    Ok(())
}

fn required_string<'a>(
    entries: &'a Map<String, Value>,
    field: &str,
    missing_error: OrchestrationError,
    type_error: OrchestrationError,
) -> Result<&'a str, OrchestrationError> {
    entries
        .get(field)
        .ok_or(missing_error)?
        .as_str()
        .ok_or(type_error)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const CANARY: &str = "private-orchestration-canary";

    fn with_unknown_argument(mut arguments: Value) -> Value {
        arguments
            .as_object_mut()
            .unwrap()
            .insert(CANARY.to_owned(), Value::Bool(true));
        arguments
    }

    #[test]
    fn route_dispatch_defaults_platform_and_declares_only_preview_permissions() {
        let output = dispatch_lite_orchestration(
            LiteOrchestrationHandler::Route,
            &json!({"request": "run a full paper workflow"}),
        )
        .unwrap();
        let LiteOrchestrationPreview::Route(route) = output else {
            panic!("route handler returned the wrong preview variant");
        };

        assert_eq!(route.mode, "preview");
        assert!(route.preview_only);
        assert_eq!(route.runtime_profile, "marketplace_lite");
        assert!(!route.run_agents_allowed);
        assert!(!route.shell_execution_allowed);
        assert!(!route.project_writes_allowed);
        assert_eq!(route.platform, LitePlatform::Unknown);
        assert_eq!(route.recommended_runtime, "full_cli_for_execution");
        assert_eq!(route.upgrade.runtime_profile, "full_cli");
        assert_eq!(route.upgrade.command, "qiongli mcp serve --transport stdio");
    }

    #[test]
    fn route_accepts_only_the_contract_platforms() {
        for (identifier, expected) in [
            ("codex", LitePlatform::Codex),
            ("claude_code", LitePlatform::ClaudeCode),
            ("claude", LitePlatform::Claude),
            ("antigravity", LitePlatform::Antigravity),
            ("cli", LitePlatform::Cli),
            ("unknown", LitePlatform::Unknown),
        ] {
            let input = OrchestratorRouteInput::from_arguments(
                &json!({"request": "plan", "platform": identifier}),
            )
            .unwrap();
            assert_eq!(input.platform, expected);
        }

        assert_eq!(
            OrchestratorRouteInput::from_arguments(&json!({"request": "plan", "platform": CANARY}))
                .unwrap_err(),
            OrchestrationError::UnsupportedPlatform
        );
    }

    #[test]
    fn task_plan_dispatch_trims_fields_and_never_enables_execution() {
        let output = dispatch_lite_orchestration(
            LiteOrchestrationHandler::TaskPlan,
            &json!({
                "task_id": " B1 ",
                "paper_type": " systematic-review ",
                "topic": " ai-feedback "
            }),
        )
        .unwrap();
        let LiteOrchestrationPreview::TaskPlan(plan) = output else {
            panic!("task-plan handler returned the wrong preview variant");
        };

        assert_eq!(plan.task_id, "B1");
        assert_eq!(plan.paper_type, "systematic-review");
        assert_eq!(plan.topic, "ai-feedback");
        assert!(!plan.run_agents_allowed);
        assert!(!plan.shell_execution_allowed);
        assert!(!plan.project_writes_allowed);
    }

    #[test]
    fn route_rejects_malformed_empty_unknown_and_oversized_input() {
        for (arguments, expected) in [
            (json!([]), OrchestrationError::ArgumentsNotObject),
            (json!({}), OrchestrationError::MissingRequest),
            (json!({"request": 7}), OrchestrationError::RequestNotString),
            (json!({"request": "  "}), OrchestrationError::EmptyRequest),
            (
                with_unknown_argument(json!({"request": "plan"})),
                OrchestrationError::UnsupportedArgument,
            ),
            (
                json!({"request": "plan", "platform": 7}),
                OrchestrationError::PlatformNotString,
            ),
        ] {
            assert_eq!(
                OrchestratorRouteInput::from_arguments(&arguments).unwrap_err(),
                expected
            );
        }

        let maximum = "é".repeat(MAX_ROUTE_REQUEST_BYTES / 2);
        assert!(OrchestratorRouteInput::try_new(maximum, None).is_ok());
        assert_eq!(
            OrchestratorRouteInput::try_new("é".repeat(MAX_ROUTE_REQUEST_BYTES / 2 + 1), None)
                .unwrap_err(),
            OrchestrationError::RequestTooLong
        );
    }

    #[test]
    fn task_plan_rejects_missing_wrong_blank_unknown_and_oversized_fields() {
        for (arguments, expected) in [
            (json!([]), OrchestrationError::ArgumentsNotObject),
            (
                json!({"paper_type": "review", "topic": "topic"}),
                OrchestrationError::MissingTaskId,
            ),
            (
                json!({"task_id": 1, "paper_type": "review", "topic": "topic"}),
                OrchestrationError::TaskIdNotString,
            ),
            (
                json!({"task_id": " ", "paper_type": "review", "topic": "topic"}),
                OrchestrationError::EmptyTaskId,
            ),
            (
                json!({"task_id": "B1", "paper_type": " ", "topic": "topic"}),
                OrchestrationError::EmptyPaperType,
            ),
            (
                json!({"task_id": "B1", "paper_type": "review", "topic": " "}),
                OrchestrationError::EmptyTopic,
            ),
            (
                with_unknown_argument(json!({
                    "task_id": "B1",
                    "paper_type": "review",
                    "topic": "topic"
                })),
                OrchestrationError::UnsupportedArgument,
            ),
        ] {
            assert_eq!(
                TaskPlanInput::from_arguments(&arguments).unwrap_err(),
                expected
            );
        }

        assert_eq!(
            TaskPlanInput::try_new("x".repeat(MAX_TASK_ID_BYTES + 1), "review", "topic")
                .unwrap_err(),
            OrchestrationError::TaskIdTooLong
        );
        assert_eq!(
            TaskPlanInput::try_new("B1", "x".repeat(MAX_PAPER_TYPE_BYTES + 1), "topic")
                .unwrap_err(),
            OrchestrationError::PaperTypeTooLong
        );
        assert_eq!(
            TaskPlanInput::try_new("B1", "review", "é".repeat(MAX_TOPIC_BYTES / 2 + 1))
                .unwrap_err(),
            OrchestrationError::TopicTooLong
        );
    }

    #[test]
    fn validation_errors_do_not_echo_private_values_or_unknown_keys() {
        for error in [
            OrchestratorRouteInput::from_arguments(&json!({"request": "plan", "platform": CANARY}))
                .unwrap_err(),
            TaskPlanInput::from_arguments(&with_unknown_argument(json!({
                "task_id": "B1",
                "paper_type": "review",
                "topic": "topic"
            })))
            .unwrap_err(),
        ] {
            assert!(!error.to_string().contains(CANARY));
        }
    }
}
