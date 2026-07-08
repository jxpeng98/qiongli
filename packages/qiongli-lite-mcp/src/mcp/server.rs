use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::provider_config::{normalize_key, save_provider_value, summary};
use crate::tools::definitions::lite_tool_definitions;

#[derive(Debug, Clone, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct McpError {
    code: i64,
    message: String,
}

#[derive(Debug, Clone)]
pub struct McpServer {
    name: String,
    version: String,
}

impl McpServer {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    pub fn handle(&self, request: McpRequest) -> Value {
        if request.jsonrpc != "2.0" {
            return self.error(request.id, -32600, "Invalid JSON-RPC version");
        }

        match request.method.as_str() {
            "initialize" => self.result(
                request.id,
                json!({
                    "protocolVersion": request
                        .params
                        .as_ref()
                        .and_then(|params| params.get("protocolVersion"))
                        .and_then(Value::as_str)
                        .unwrap_or("2025-11-25"),
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": self.name,
                        "version": self.version
                    }
                }),
            ),
            "ping" => self.result(request.id, json!({})),
            "tools/list" => self.result(request.id, json!({"tools": lite_tool_definitions()})),
            "tools/call" => self.handle_tool_call(request.id, request.params),
            _ => self.error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    fn handle_tool_call(&self, id: Option<Value>, params: Option<Value>) -> Value {
        let Some(params) = params else {
            return self.error(id, -32602, "Missing tool call params");
        };
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing tool name");
        };
        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

        match name {
            "qiongli_config_status" => match summary() {
                Ok(status) => self.tool_result(id, json!(status)),
                Err(error) => self.tool_error(id, error.to_string()),
            },
            "qiongli_save_provider_config" => self.save_provider_config(id, &arguments),
            _ => self.error(id, -32601, format!("Tool not found: {name}")),
        }
    }

    fn save_provider_config(&self, id: Option<Value>, arguments: &Value) -> Value {
        let Some(provider) = arguments.get("provider").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing provider");
        };
        let Some(field) = arguments.get("field").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing field");
        };
        let Some(value) = arguments.get("value").and_then(Value::as_str) else {
            return self.error(id, -32602, "Missing value");
        };

        let normalized_provider = normalize_key(provider);
        let normalized_field = normalize_key(field);
        match save_provider_value(provider, field, value) {
            Ok(_) => self.tool_result(
                id,
                json!({
                    "status": "ok",
                    "provider": normalized_provider,
                    "field": normalized_field,
                    "saved": true,
                    "warning": "Prefer qiongli_configure_provider for interactive API key setup."
                }),
            ),
            Err(error) => self.tool_error(id, error.to_string()),
        }
    }

    fn result(&self, id: Option<Value>, result: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "result": result})
    }

    fn tool_result(&self, id: Option<Value>, structured_content: Value) -> Value {
        let text = serde_json::to_string_pretty(&structured_content)
            .unwrap_or_else(|_| "{\"status\":\"ok\"}".to_string());
        self.result(
            id,
            json!({
                "content": [{"type": "text", "text": text}],
                "structuredContent": structured_content
            }),
        )
    }

    fn tool_error(&self, id: Option<Value>, message: String) -> Value {
        self.result(
            id,
            json!({
                "isError": true,
                "content": [{"type": "text", "text": message}],
                "structuredContent": {"status": "error"}
            }),
        )
    }

    fn error(&self, id: Option<Value>, code: i64, message: impl Into<String>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": McpError {
                code,
                message: message.into(),
            }
        })
    }
}
