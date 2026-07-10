use qiongli_lite_mcp::mcp::server::{has_tool_handler, McpRequest, McpServer};
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
    assert!(names.iter().all(|name| has_tool_handler(name)));
}

#[test]
fn preview_route_rejects_missing_request_and_declares_lite_safety() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    let missing = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_orchestrator_route",
            "arguments": {}
        })),
    });
    assert_eq!(missing["error"]["code"], -32602);

    let preview = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_orchestrator_route",
            "arguments": {"request": "run a full paper workflow"}
        })),
    });
    let payload = &preview["result"]["structuredContent"];
    assert_eq!(payload["mode"], "preview");
    assert_eq!(payload["runtime_profile"], "marketplace_lite");
    assert_eq!(payload["run_agents_allowed"], false);
    assert_eq!(payload["shell_execution_allowed"], false);
    assert_eq!(payload["project_writes_allowed"], false);
    assert_eq!(payload["upgrade"]["required_for_execution"], true);
}

#[test]
fn search_rejects_unsupported_or_out_of_range_arguments_before_network() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    for (arguments, expected_message) in [
        (
            json!({"query": "governance", "query_variants": ["platforms"]}),
            "Unsupported argument: query_variants",
        ),
        (
            json!({"query": "governance", "per_provider_limit": 201}),
            "per_provider_limit must be between 1 and 200",
        ),
        (
            json!({"query": "governance", "providers": []}),
            "providers must not be empty",
        ),
        (
            json!({"query": "governance", "search_mode": "title"}),
            "unsupported search_mode",
        ),
    ] {
        let response = server.handle(McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "qiongli_literature_search",
                "arguments": arguments
            })),
        });

        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert_eq!(response["error"]["message"], expected_message);
    }
}

#[test]
fn config_wizard_aliases_share_the_same_handler_policy() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    let call = |name: &str, id: i64| {
        server.handle(McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": name,
                "arguments": {"host": "0.0.0.0"}
            })),
        })
    };

    let canonical = call("qiongli_configure_provider", 1);
    let alias = call("qiongli_open_config_wizard", 2);

    assert_eq!(canonical["error"]["code"], -32602);
    assert_eq!(alias["error"]["code"], -32602);
    assert_eq!(canonical["error"]["message"], alias["error"]["message"]);
}

#[test]
fn repeated_wizard_calls_reuse_one_active_loopback_session() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    let call = |name: &str, id: i64| {
        server.handle(McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": name,
                "arguments": {"provider": "openalex"}
            })),
        })
    };

    let first = call("qiongli_configure_provider", 1);
    let second = call("qiongli_open_config_wizard", 2);
    let first_payload = &first["result"]["structuredContent"];
    let second_payload = &second["result"]["structuredContent"];

    assert_eq!(first_payload["status"], "ready");
    assert_eq!(second_payload["status"], "already_running");
    assert_eq!(first_payload["url"], second_payload["url"]);
    assert_eq!(first_payload["port"], second_payload["port"]);
}

#[test]
fn evidence_export_preserves_provider_status_and_search_plan() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    let response = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_export_evidence",
            "arguments": {
                "query": "governance",
                "provider_status": {"arxiv": "configured"},
                "search_plan": {"search_execution_mode": "provider_connected"},
                "diagnostics": {"status": "complete"},
                "results": [{"title": "Paper"}]
            }
        })),
    });
    let payload = &response["result"]["structuredContent"];

    assert_eq!(payload["provider_status"]["arxiv"], "configured");
    assert_eq!(
        payload["search_plan"]["search_execution_mode"],
        "provider_connected"
    );
    assert_eq!(payload["result_count"], 1);
}

#[test]
fn zotero_export_honors_requested_formats() {
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    let response = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_zotero_export_import_files",
            "arguments": {
                "records": [],
                "formats": ["references.ris"]
            }
        })),
    });

    let files = response["result"]["structuredContent"]["files"]
        .as_object()
        .unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("references.ris"));
}
