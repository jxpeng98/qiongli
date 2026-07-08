use qiongli_lite_mcp::mcp::server::{McpRequest, McpServer};
use serde_json::json;

#[test]
fn initialize_and_tools_list_return_json_rpc_results() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");

    let initialize = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({"protocolVersion": "2025-11-25"})),
    });
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "qiongli-literature-provider"
    );

    let tools = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: Some(json!({})),
    });
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"qiongli_literature_status"));
}

