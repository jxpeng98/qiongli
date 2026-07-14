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
            "Unsupported argument",
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

    let oversized = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_search",
            "arguments": {"query": "x".repeat(4097)}
        })),
    });
    assert_eq!(oversized["error"]["code"], -32602);
    assert_eq!(
        oversized["error"]["message"],
        "search query exceeds the byte limit"
    );
}

#[test]
fn validation_errors_do_not_echo_attacker_controlled_names_or_enum_values() {
    const CANARY: &str = "attacker-controlled-secret-canary";
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");

    let mut arguments = json!({"query": "governance"});
    arguments
        .as_object_mut()
        .unwrap()
        .insert(CANARY.to_string(), json!(true));
    let unknown_argument = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_search",
            "arguments": arguments
        })),
    });
    assert_eq!(unknown_argument["error"]["code"], -32602);
    assert_eq!(unknown_argument["error"]["message"], "Unsupported argument");

    let unsupported_format = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_zotero_export_import_files",
            "arguments": {"records": [], "formats": [CANARY]}
        })),
    });
    assert_eq!(unsupported_format["error"]["code"], -32602);
    assert_eq!(unsupported_format["error"]["message"], "Unsupported format");

    let unsupported_provider = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_search",
            "arguments": {"query": "governance", "providers": [CANARY]}
        })),
    });
    assert_eq!(unsupported_provider["error"]["code"], -32602);
    assert_eq!(
        unsupported_provider["error"]["message"],
        "unsupported provider"
    );

    let unsupported_wizard_provider = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_configure_provider",
            "arguments": {"provider": CANARY}
        })),
    });
    assert_eq!(unsupported_wizard_provider["error"]["code"], -32602);
    assert_eq!(
        unsupported_wizard_provider["error"]["message"],
        "unsupported provider"
    );

    let unsupported_config_field = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(5)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_save_provider_config",
            "arguments": {"provider": "openalex", "field": CANARY, "value": "value"}
        })),
    });
    assert_eq!(unsupported_config_field["error"]["code"], -32602);
    assert_eq!(
        unsupported_config_field["error"]["message"],
        "unsupported provider field"
    );

    for response in [
        unknown_argument,
        unsupported_format,
        unsupported_provider,
        unsupported_wizard_provider,
        unsupported_config_field,
    ] {
        let rendered = serde_json::to_string(&response).unwrap();
        assert!(
            !rendered.contains(CANARY),
            "response leaked canary: {rendered}"
        );
    }
}

#[test]
fn unknown_methods_and_tool_names_return_static_no_echo_errors() {
    const CANARY: &str = "attacker-controlled-dispatch-canary";
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");

    let unknown_method = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: CANARY.to_string(),
        params: Some(json!({})),
    });
    assert_eq!(unknown_method["error"]["code"], -32601);
    assert_eq!(unknown_method["error"]["message"], "Method not found");

    let unknown_tool = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({"name": CANARY, "arguments": {}})),
    });
    assert_eq!(unknown_tool["error"]["code"], -32601);
    assert_eq!(unknown_tool["error"]["message"], "Tool not found");

    for response in [unknown_method, unknown_tool] {
        assert!(!serde_json::to_string(&response).unwrap().contains(CANARY));
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
fn evidence_export_recursively_redacts_credentials_but_preserves_benign_keys() {
    const CANARY: &str = "nested-output-secret-canary";
    let server = McpServer::new("qiongli-literature-provider", "0.1.0");
    let response = server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_export_evidence",
            "arguments": {
                "query": "governance",
                "diagnostics": {
                    "secret": CANARY,
                    "password": CANARY,
                    "passwd": CANARY,
                    "credential": CANARY,
                    "accessToken": CANARY,
                    "nested": {
                        "client_secret": CANARY,
                        "authorization": CANARY,
                        "auth": CANARY,
                        "bearer": CANARY,
                        "private_key": CANARY,
                        "serviceAccessKey": CANARY,
                        "token_budget": 4096,
                        "public_key": "kept"
                    }
                },
                "results": [{
                    "title": "Paper",
                    "metadata": {
                        "credentials": CANARY,
                        "token": CANARY,
                        "refresh_token": CANARY,
                        "sessionToken": CANARY,
                        "service_api_key": CANARY,
                        "serviceApiKey": CANARY,
                        "servicePrivateKey": CANARY,
                        "serviceClientSecret": CANARY,
                        "cookie": CANARY,
                        "token_budget": 2048,
                        "public_key": "also-kept"
                    }
                }]
            }
        })),
    });

    let rendered = serde_json::to_string(&response).unwrap();
    assert!(
        !rendered.contains(CANARY),
        "response leaked nested credential output: {rendered}"
    );

    let payload = &response["result"]["structuredContent"];
    assert_eq!(
        payload["artifact_type"],
        "qiongli_literature_evidence_snapshot"
    );
    assert_eq!(payload["result_count"], 1);
    assert_eq!(payload["diagnostics"]["nested"]["token_budget"], 4096);
    assert_eq!(payload["diagnostics"]["nested"]["public_key"], "kept");
    assert_eq!(payload["results"][0]["metadata"]["token_budget"], 2048);
    assert_eq!(payload["results"][0]["metadata"]["public_key"], "also-kept");
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
