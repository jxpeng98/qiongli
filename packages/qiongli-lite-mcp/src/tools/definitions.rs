use serde::{Deserialize, Serialize};

const LITE_TOOLS_JSON: &str =
    include_str!("../../../../content/mcp-contracts/lite-tools.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct ToolContract {
    tools: Vec<ToolDefinition>,
}

pub fn lite_tool_definitions() -> Vec<ToolDefinition> {
    serde_json::from_str::<ToolContract>(LITE_TOOLS_JSON)
        .expect("bundled Lite MCP tool contract must be valid JSON")
        .tools
}
