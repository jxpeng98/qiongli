use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
            _ => self.error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    fn result(&self, id: Option<Value>, result: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "result": result})
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
