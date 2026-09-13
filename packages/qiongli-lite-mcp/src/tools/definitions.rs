use qiongli_runtime::LiteToolRegistry;
pub use qiongli_runtime::ToolDefinition;

const LITE_TOOLS_JSON: &[u8] = include_bytes!("../../../../content/mcp-contracts/lite-tools.json");

pub fn lite_tool_definitions() -> Vec<ToolDefinition> {
    LiteToolRegistry::from_json(LITE_TOOLS_JSON)
        .expect("bundled Lite MCP tool contract must satisfy the shared runtime contract")
        .into_tools()
}
