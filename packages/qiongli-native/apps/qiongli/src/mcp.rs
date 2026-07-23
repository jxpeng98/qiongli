use std::io::{BufRead, Write};

use qiongli_config::{GlobalSettingsStore, SecretStore};
use qiongli_content::EmbeddedContent;
use qiongli_execution::{
    BackendControlService, CancellationToken, OrchestrationExecutionMode, RunId,
};
use qiongli_project::ProjectStateService;
use qiongli_runtime::mcp::LiteMcpServer;
use qiongli_runtime::protocol::{read_message, write_message};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{
    FullProjectService, FullProjectServiceErrorKind, FullProjectToolId, FullProjectToolRegistry,
    LiteToolRegistry, RuntimeError, RuntimeErrorCode,
};
use serde_json::{Value, json};

use crate::agent_run::{FullAgentRunRequest, FullAgentRunService};
use crate::command::{CommandEnvironment, config_root, config_store};
use crate::credential_store::native_secret_store;
use crate::orchestration_control::{
    FullOrchestrationService, OrchestrationControlAction, OrchestrationRunReference,
};

pub fn serve_lite_mcp<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), RuntimeError> {
    lite_server(environment, content)?.serve(reader, writer)
}

pub fn serve_full_mcp<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), RuntimeError> {
    let projects = config_root(environment).ok().map(ProjectStateService::new);
    let backend_store = config_store(environment).ok();
    let secret_store = native_secret_store();
    let registry = FullProjectToolRegistry::from_embedded_content(content)?;
    let orchestration = projects
        .as_ref()
        .map(|projects| {
            FullOrchestrationService::from_embedded_content(
                projects.clone(),
                registry.clone(),
                content,
            )
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::InvalidFullProjectContract))
        })
        .transpose()?;
    FullMcpServer::new(
        lite_server(environment, content)?,
        registry,
        projects,
        backend_store,
        secret_store,
        orchestration,
    )
    .serve(reader, writer)
}

fn lite_server(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<LiteMcpServer, RuntimeError> {
    let registry = LiteToolRegistry::from_embedded_content(content)?;
    let server = match config_store(environment).and_then(|store| store.load()) {
        Ok(loaded) => {
            let secret_store = native_secret_store();
            let access =
                ProviderAccess::from_global_settings(&loaded.settings, secret_store.as_ref());
            LiteMcpServer::production("qiongli", env!("CARGO_PKG_VERSION"), registry, access)
        }
        Err(_) => LiteMcpServer::config_unavailable("qiongli", env!("CARGO_PKG_VERSION"), registry),
    };
    Ok(server)
}

#[derive(Clone)]
pub struct FullMcpServer {
    lite: LiteMcpServer,
    registry: FullProjectToolRegistry,
    projects: Option<FullProjectService>,
    project_state: Option<ProjectStateService>,
    backend_store: Option<GlobalSettingsStore>,
    secret_store: std::sync::Arc<dyn SecretStore>,
    orchestration: Option<FullOrchestrationService>,
}

impl FullMcpServer {
    #[must_use]
    pub fn new(
        lite: LiteMcpServer,
        registry: FullProjectToolRegistry,
        projects: Option<ProjectStateService>,
        backend_store: Option<GlobalSettingsStore>,
        secret_store: std::sync::Arc<dyn SecretStore>,
        orchestration: Option<FullOrchestrationService>,
    ) -> Self {
        let project_state = projects.clone();
        Self {
            lite,
            registry,
            projects: projects.map(FullProjectService::new),
            project_state,
            backend_store,
            secret_store,
            orchestration,
        }
    }

    pub fn serve<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<(), RuntimeError> {
        while let Some(message) = read_message(reader)? {
            let response = match serde_json::from_str::<Value>(&message.payload) {
                Ok(request) => self.handle(request),
                Err(_) => Some(json_rpc_error(None, -32700, "Parse error")),
            };
            if let Some(response) = response {
                write_message(writer, &response, message.framing)?;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn handle(&self, request: Value) -> Option<Value> {
        match request.get("method").and_then(Value::as_str) {
            Some("tools/list") => self.list_tools(request),
            Some("tools/call") => self.handle_tool_call(request),
            _ => self.lite.handle(request),
        }
    }

    fn list_tools(&self, request: Value) -> Option<Value> {
        let mut response = self.lite.handle(request)?;
        let tools = response
            .pointer_mut("/result/tools")
            .and_then(Value::as_array_mut);
        if let Some(tools) = tools {
            tools.extend(self.registry.tools().iter().map(|tool| json!(tool)));
            tools.extend(backend_control_tools());
            tools.extend(orchestration_control_tools());
        }
        Some(response)
    }

    fn handle_tool_call(&self, request: Value) -> Option<Value> {
        let requested_name = request.pointer("/params/name").and_then(Value::as_str);
        let project_tool = requested_name.and_then(|name| self.registry.resolve(name));
        let backend_tool = requested_name.and_then(BackendControlTool::resolve);
        let orchestration_tool = requested_name.and_then(OrchestrationControlTool::resolve);
        if project_tool.is_none() && backend_tool.is_none() && orchestration_tool.is_none() {
            return self.lite.handle(request);
        }
        let validation = self.lite.handle(request.clone())?;
        if validation.pointer("/error/code").and_then(Value::as_i64) != Some(-32601) {
            return Some(validation);
        }
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let arguments = request
            .pointer("/params/arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(tool) = project_tool {
            Some(self.dispatch_project(id, tool, &arguments))
        } else if let Some(tool) = backend_tool {
            Some(self.dispatch_backend(id, tool, &arguments))
        } else {
            Some(self.dispatch_orchestration(
                id,
                orchestration_tool.expect("validated orchestration tool remains present"),
                &arguments,
            ))
        }
    }

    fn dispatch_project(&self, id: Value, tool: FullProjectToolId, arguments: &Value) -> Value {
        let Some(projects) = self.projects.as_ref() else {
            return tool_error(
                id,
                "project-service-unavailable",
                "native Research Library is unavailable",
            );
        };
        match projects.dispatch(tool, arguments) {
            Ok(result) => tool_result(id, result),
            Err(error) if error.kind() == FullProjectServiceErrorKind::InvalidArguments => {
                json_rpc_error(Some(id), -32602, error.public_message())
            }
            Err(error) => tool_error(id, error.reason_code(), error.public_message()),
        }
    }

    fn dispatch_backend(&self, id: Value, tool: BackendControlTool, arguments: &Value) -> Value {
        let Some(arguments) = arguments.as_object() else {
            return json_rpc_error(Some(id), -32602, "Invalid backend control arguments");
        };
        let valid = match tool {
            BackendControlTool::Status => arguments.is_empty(),
            BackendControlTool::Test => {
                arguments.len() == 1
                    && arguments
                        .get("confirmNetworkRequest")
                        .and_then(Value::as_bool)
                        == Some(true)
            }
            BackendControlTool::Run => parse_agent_run_request(arguments).is_ok(),
        };
        if !valid {
            return json_rpc_error(Some(id), -32602, "Invalid backend control arguments");
        }
        let Some(store) = self.backend_store.as_ref() else {
            return tool_error(
                id,
                "agent-backend-config-unavailable",
                "agent backend configuration is unavailable",
            );
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return tool_error(
                    id,
                    error.reason_code(),
                    "agent backend configuration is unavailable",
                );
            }
        };
        let control = BackendControlService::from_global_settings(
            &loaded.settings,
            std::sync::Arc::clone(&self.secret_store),
        );
        match tool {
            BackendControlTool::Status => tool_result(id, json!(control.openai_status())),
            BackendControlTool::Test => {
                match control.test_openai_connection(&CancellationToken::new()) {
                    Ok(result) => tool_result(id, json!(result)),
                    Err(error) => tool_error(
                        id,
                        error.reason_code(),
                        "agent backend connection test failed",
                    ),
                }
            }
            BackendControlTool::Run => {
                let Some(projects) = self.project_state.as_ref() else {
                    return tool_error(
                        id,
                        "project-service-unavailable",
                        "native Research Library is unavailable",
                    );
                };
                let request = match parse_agent_run_request(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(
                            Some(id),
                            -32602,
                            "Invalid backend control arguments",
                        );
                    }
                };
                let service = FullAgentRunService::new(projects.clone(), self.registry.clone());
                match service.run_openai(
                    request,
                    &loaded.settings,
                    std::sync::Arc::clone(&self.secret_store),
                ) {
                    Ok(result) => tool_result(id, json!(result)),
                    Err(error) => tool_error(id, error.reason_code(), "agent backend run failed"),
                }
            }
        }
    }

    fn dispatch_orchestration(
        &self,
        id: Value,
        tool: OrchestrationControlTool,
        arguments: &Value,
    ) -> Value {
        let Some(service) = self.orchestration.as_ref() else {
            return tool_error(
                id,
                "orchestration-service-unavailable",
                "native orchestration service is unavailable",
            );
        };
        let Some(arguments) = arguments.as_object() else {
            return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
        };
        let result = match tool {
            OrchestrationControlTool::Doctor => {
                let (project_id, revision) = match parse_project_reference(arguments) {
                    Ok(reference) => reference,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                let loaded = match self.load_backend_settings() {
                    Ok(loaded) => loaded,
                    Err(response) => return response.with_id(id),
                };
                service
                    .doctor_openai(
                        &project_id,
                        revision,
                        &loaded.settings,
                        self.secret_store.as_ref(),
                    )
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Runs => {
                let (project_id, revision) = match parse_project_reference(arguments) {
                    Ok(reference) => reference,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .list_runs(&project_id, revision)
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Test => {
                let (project_id, revision, mode) = match parse_orchestration_test(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                let loaded = match self.load_backend_settings() {
                    Ok(loaded) => loaded,
                    Err(response) => return response.with_id(id),
                };
                service
                    .start_openai_test(
                        project_id,
                        revision,
                        mode,
                        true,
                        &loaded.settings,
                        std::sync::Arc::clone(&self.secret_store),
                    )
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Continue => {
                let reference = match parse_run_reference(arguments, true) {
                    Ok(reference) => reference,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                let loaded = match self.load_backend_settings() {
                    Ok(loaded) => loaded,
                    Err(response) => return response.with_id(id),
                };
                service
                    .continue_openai(
                        &reference,
                        true,
                        &loaded.settings,
                        std::sync::Arc::clone(&self.secret_store),
                    )
                    .and_then(serialize_orchestration)
            }
            OrchestrationControlTool::Action => {
                let (reference, action) = match parse_orchestration_action(arguments) {
                    Ok(request) => request,
                    Err(()) => {
                        return json_rpc_error(Some(id), -32602, "Invalid orchestration arguments");
                    }
                };
                service
                    .control(&reference, action)
                    .and_then(serialize_orchestration)
            }
        };
        match result {
            Ok(value) => tool_result(id, value),
            Err(error) => tool_error(id, error.reason_code(), "orchestration operation failed"),
        }
    }

    fn load_backend_settings(
        &self,
    ) -> Result<qiongli_config::LoadedGlobalSettings, DeferredToolError> {
        let store = self.backend_store.as_ref().ok_or(DeferredToolError {
            reason_code: "agent-backend-config-unavailable",
            message: "agent backend configuration is unavailable",
        })?;
        store.load().map_err(|error| DeferredToolError {
            reason_code: error.reason_code(),
            message: "agent backend configuration is unavailable",
        })
    }
}

struct DeferredToolError {
    reason_code: &'static str,
    message: &'static str,
}

impl DeferredToolError {
    fn with_id(self, id: Value) -> Value {
        tool_error(id, self.reason_code, self.message)
    }
}

#[derive(Clone, Copy)]
enum BackendControlTool {
    Status,
    Test,
    Run,
}

#[derive(Clone, Copy)]
enum OrchestrationControlTool {
    Doctor,
    Runs,
    Test,
    Continue,
    Action,
}

impl OrchestrationControlTool {
    fn resolve(name: &str) -> Option<Self> {
        match name {
            "qiongli_orchestration_doctor" => Some(Self::Doctor),
            "qiongli_orchestration_runs" => Some(Self::Runs),
            "qiongli_orchestration_test" => Some(Self::Test),
            "qiongli_orchestration_continue" => Some(Self::Continue),
            "qiongli_orchestration_action" => Some(Self::Action),
            _ => None,
        }
    }
}

impl BackendControlTool {
    fn resolve(name: &str) -> Option<Self> {
        match name {
            "qiongli_agent_backend_status" => Some(Self::Status),
            "qiongli_agent_backend_test" => Some(Self::Test),
            "qiongli_agent_run" => Some(Self::Run),
            _ => None,
        }
    }
}

fn backend_control_tools() -> impl Iterator<Item = Value> {
    [
        json!({
            "name": "qiongli_agent_backend_status",
            "description": "Inspect redacted direct-agent-backend readiness without making a network request.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_agent_backend_test",
            "description": "Explicitly send one minimal non-stored OpenAI Responses request and return only a redacted pass/fail result.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "confirmNetworkRequest": {"type": "boolean", "const": true}
                },
                "required": ["confirmNetworkRequest"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": true
            }
        }),
        json!({
            "name": "qiongli_agent_run",
            "description": "Explicitly run one project-scoped read-only Full query through the configured OpenAI backend and native ToolHost. The prompt and redacted project tool results are sent to OpenAI.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "prompt": {"type": "string", "minLength": 1, "maxLength": 16384},
                    "confirmNetworkRequest": {"type": "boolean", "const": true}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "prompt",
                    "confirmNetworkRequest"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": true
            }
        }),
    ]
    .into_iter()
}

fn orchestration_control_tools() -> impl Iterator<Item = Value> {
    let project_properties = json!({
        "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
        "expectedProjectRevision": {"type": "integer", "minimum": 1}
    });
    [
        json!({
            "name": "qiongli_orchestration_doctor",
            "description": "Inspect the embedded workflow contract, project binding, configured backend readiness, and interrupted-run count without making a network request.",
            "inputSchema": {
                "type": "object",
                "properties": project_properties.clone(),
                "required": ["projectId", "expectedProjectRevision"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_runs",
            "description": "List redacted revision-bound orchestration checkpoints and their available actions without returning prompts or model output.",
            "inputSchema": {
                "type": "object",
                "properties": project_properties,
                "required": ["projectId", "expectedProjectRevision"],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "qiongli_orchestration_test",
            "description": "Start one revision-bound workflow run and execute its next task through the configured OpenAI backend. Candidate role output is returned but never persisted.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "executionMode": {"type": "string", "enum": ["solo", "duo", "triad"]},
                    "confirmNetworkRequest": {"type": "boolean", "const": true}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "executionMode",
                    "confirmNetworkRequest"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": true
            }
        }),
        json!({
            "name": "qiongli_orchestration_continue",
            "description": "Execute the next task of an unchanged revision-bound run after explicit network confirmation and exact generation/document compare-and-swap validation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
                    "expectedGeneration": {"type": "integer", "minimum": 0},
                    "expectedDocumentSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                    "confirmNetworkRequest": {"type": "boolean", "const": true}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "runId",
                    "expectedGeneration",
                    "expectedDocumentSha256",
                    "confirmNetworkRequest"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": true
            }
        }),
        json!({
            "name": "qiongli_orchestration_action",
            "description": "Pause, explicitly recover, resume, or terminally cancel an unchanged orchestration checkpoint without making a network request.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "projectId": {"type": "string", "pattern": "^prj_[0-9a-f]{32}$"},
                    "expectedProjectRevision": {"type": "integer", "minimum": 1},
                    "runId": {"type": "string", "pattern": "^run_[0-9a-f]{32}$"},
                    "expectedGeneration": {"type": "integer", "minimum": 0},
                    "expectedDocumentSha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                    "action": {"type": "string", "enum": ["pause", "recover", "resume", "cancel"]}
                },
                "required": [
                    "projectId",
                    "expectedProjectRevision",
                    "runId",
                    "expectedGeneration",
                    "expectedDocumentSha256",
                    "action"
                ],
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
    ]
    .into_iter()
}

fn parse_project_reference(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(qiongli_project::ProjectId, u64), ()> {
    if arguments.len() != 2 {
        return Err(());
    }
    let project_id = arguments
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or(())?;
    let revision = arguments
        .get("expectedProjectRevision")
        .and_then(Value::as_u64)
        .filter(|revision| *revision > 0)
        .ok_or(())?;
    Ok((
        qiongli_project::ProjectId::parse(project_id.to_owned()).map_err(|_| ())?,
        revision,
    ))
}

fn parse_orchestration_test(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(qiongli_project::ProjectId, u64, OrchestrationExecutionMode), ()> {
    if arguments.len() != 4
        || arguments
            .get("confirmNetworkRequest")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return Err(());
    }
    let project_id = arguments
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or(())?;
    let revision = arguments
        .get("expectedProjectRevision")
        .and_then(Value::as_u64)
        .filter(|revision| *revision > 0)
        .ok_or(())?;
    let mode = match arguments.get("executionMode").and_then(Value::as_str) {
        Some("solo") => OrchestrationExecutionMode::Solo,
        Some("duo") => OrchestrationExecutionMode::Duo,
        Some("triad") => OrchestrationExecutionMode::Triad,
        _ => return Err(()),
    };
    Ok((
        qiongli_project::ProjectId::parse(project_id.to_owned()).map_err(|_| ())?,
        revision,
        mode,
    ))
}

fn parse_run_reference(
    arguments: &serde_json::Map<String, Value>,
    with_network_confirmation: bool,
) -> Result<OrchestrationRunReference, ()> {
    let expected_len = if with_network_confirmation { 6 } else { 5 };
    if arguments.len() != expected_len
        || (with_network_confirmation
            && arguments
                .get("confirmNetworkRequest")
                .and_then(Value::as_bool)
                != Some(true))
    {
        return Err(());
    }
    let project_id = arguments
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or(())?;
    let revision = arguments
        .get("expectedProjectRevision")
        .and_then(Value::as_u64)
        .filter(|revision| *revision > 0)
        .ok_or(())?;
    let run_id = arguments.get("runId").and_then(Value::as_str).ok_or(())?;
    let generation = arguments
        .get("expectedGeneration")
        .and_then(Value::as_u64)
        .ok_or(())?;
    let document_sha256 = arguments
        .get("expectedDocumentSha256")
        .and_then(Value::as_str)
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or(())?;
    Ok(OrchestrationRunReference {
        project_id: qiongli_project::ProjectId::parse(project_id.to_owned()).map_err(|_| ())?,
        expected_project_revision: revision,
        run_id: RunId::parse(run_id.to_owned()).map_err(|_| ())?,
        expected_generation: generation,
        expected_document_sha256: document_sha256.to_owned(),
    })
}

fn parse_orchestration_action(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(OrchestrationRunReference, OrchestrationControlAction), ()> {
    if arguments.len() != 6 {
        return Err(());
    }
    let mut reference_arguments = arguments.clone();
    let action = match reference_arguments
        .remove("action")
        .and_then(|value| value.as_str().map(str::to_owned))
        .as_deref()
    {
        Some("pause") => OrchestrationControlAction::Pause,
        Some("recover") => OrchestrationControlAction::Recover,
        Some("resume") => OrchestrationControlAction::Resume,
        Some("cancel") => OrchestrationControlAction::Cancel,
        _ => return Err(()),
    };
    Ok((parse_run_reference(&reference_arguments, false)?, action))
}

fn serialize_orchestration<T: serde::Serialize>(
    value: T,
) -> Result<Value, crate::orchestration_control::FullOrchestrationError> {
    serde_json::to_value(value).map_err(|_| {
        crate::orchestration_control::FullOrchestrationError::new(
            "orchestration-output-serialization-failed",
        )
    })
}

fn parse_agent_run_request(
    arguments: &serde_json::Map<String, Value>,
) -> Result<FullAgentRunRequest, ()> {
    if arguments.len() != 4
        || arguments
            .get("confirmNetworkRequest")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return Err(());
    }
    let project_id = arguments
        .get("projectId")
        .and_then(Value::as_str)
        .ok_or(())?;
    let expected_project_revision = arguments
        .get("expectedProjectRevision")
        .and_then(Value::as_u64)
        .filter(|revision| *revision > 0)
        .ok_or(())?;
    let prompt = arguments.get("prompt").and_then(Value::as_str).ok_or(())?;
    FullAgentRunRequest::new(
        qiongli_project::ProjectId::parse(project_id.to_owned()).map_err(|_| ())?,
        expected_project_revision,
        qiongli_ui::PrivateText::new(prompt.to_owned()),
        true,
    )
    .map_err(|_| ())
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn json_rpc_error(id: Option<Value>, code: i64, message: &'static str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}

fn tool_result(id: Value, structured_content: Value) -> Value {
    let text = serde_json::to_string_pretty(&structured_content)
        .unwrap_or_else(|_| "{\"status\":\"ok\"}".to_string());
    let response = json_rpc_result(
        id.clone(),
        json!({
            "content": [{"type": "text", "text": text}],
            "structuredContent": structured_content
        }),
    );
    if serde_json::to_vec(&response)
        .is_ok_and(|bytes| bytes.len() <= qiongli_runtime::protocol::MAX_MCP_MESSAGE_BYTES)
    {
        response
    } else {
        tool_error(
            id,
            "tool-output-too-large",
            "tool output exceeds the byte limit",
        )
    }
}

fn tool_error(id: Value, reason_code: &'static str, message: &'static str) -> Value {
    json_rpc_result(
        id,
        json!({
            "isError": true,
            "content": [{"type": "text", "text": message}],
            "structuredContent": {
                "status": "error",
                "error_kind": "tool_error",
                "reason_code": reason_code,
                "message": message
            }
        }),
    )
}
