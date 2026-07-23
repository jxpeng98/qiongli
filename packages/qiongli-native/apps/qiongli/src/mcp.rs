use std::io::{BufRead, Write};

use qiongli_config::{GlobalSettingsStore, SecretStore};
use qiongli_content::EmbeddedContent;
use qiongli_execution::{BackendControlService, CancellationToken};
use qiongli_project::ProjectStateService;
use qiongli_runtime::mcp::LiteMcpServer;
use qiongli_runtime::protocol::{read_message, write_message};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{
    FullProjectService, FullProjectServiceErrorKind, FullProjectToolId, FullProjectToolRegistry,
    LiteToolRegistry, RuntimeError,
};
use serde_json::{Value, json};

use crate::agent_run::{FullAgentRunRequest, FullAgentRunService};
use crate::command::{CommandEnvironment, config_root, config_store};
use crate::credential_store::native_secret_store;

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
    FullMcpServer::new(
        lite_server(environment, content)?,
        FullProjectToolRegistry::from_embedded_content(content)?,
        projects,
        backend_store,
        secret_store,
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
}

impl FullMcpServer {
    #[must_use]
    pub fn new(
        lite: LiteMcpServer,
        registry: FullProjectToolRegistry,
        projects: Option<ProjectStateService>,
        backend_store: Option<GlobalSettingsStore>,
        secret_store: std::sync::Arc<dyn SecretStore>,
    ) -> Self {
        let project_state = projects.clone();
        Self {
            lite,
            registry,
            projects: projects.map(FullProjectService::new),
            project_state,
            backend_store,
            secret_store,
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
        }
        Some(response)
    }

    fn handle_tool_call(&self, request: Value) -> Option<Value> {
        let requested_name = request.pointer("/params/name").and_then(Value::as_str);
        let project_tool = requested_name.and_then(|name| self.registry.resolve(name));
        let backend_tool = requested_name.and_then(BackendControlTool::resolve);
        if project_tool.is_none() && backend_tool.is_none() {
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
        } else {
            Some(self.dispatch_backend(
                id,
                backend_tool.expect("validated backend tool remains present"),
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
}

#[derive(Clone, Copy)]
enum BackendControlTool {
    Status,
    Test,
    Run,
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
        prompt.to_owned(),
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
