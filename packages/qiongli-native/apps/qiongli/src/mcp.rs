use std::io::{BufRead, Write};

use qiongli_content::EmbeddedContent;
use qiongli_project::ProjectStateService;
use qiongli_runtime::mcp::LiteMcpServer;
use qiongli_runtime::protocol::{read_message, write_message};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{
    FullProjectService, FullProjectServiceErrorKind, FullProjectToolId, FullProjectToolRegistry,
    LiteToolRegistry, RuntimeError,
};
use serde_json::{Value, json};

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
    FullMcpServer::new(
        lite_server(environment, content)?,
        FullProjectToolRegistry::from_embedded_content(content)?,
        projects,
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
}

impl FullMcpServer {
    #[must_use]
    pub fn new(
        lite: LiteMcpServer,
        registry: FullProjectToolRegistry,
        projects: Option<ProjectStateService>,
    ) -> Self {
        Self {
            lite,
            registry,
            projects: projects.map(FullProjectService::new),
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
        }
        Some(response)
    }

    fn handle_tool_call(&self, request: Value) -> Option<Value> {
        let name = request
            .pointer("/params/name")
            .and_then(Value::as_str)
            .and_then(|name| self.registry.resolve(name));
        let Some(tool) = name else {
            return self.lite.handle(request);
        };
        let validation = self.lite.handle(request.clone())?;
        if validation.pointer("/error/code").and_then(Value::as_i64) != Some(-32601) {
            return Some(validation);
        }
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let arguments = request
            .pointer("/params/arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        Some(self.dispatch_project(id, tool, &arguments))
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
