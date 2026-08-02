use std::io::{BufReader, Cursor};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use qiongli_runtime::mcp::{LiteMcpServer, MCP_PROTOCOL_VERSION};
use qiongli_runtime::protocol::{Framing, read_message};
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{LITE_PUBLIC_TOOL_NAMES, LiteToolRegistry};
use serde_json::{Value, json};

const CONTRACT: &[u8] = include_bytes!("../../../../../content/mcp-contracts/lite-tools.json");
const SECRET_CANARY: &str = "native-mcp-secret-canary";

fn server() -> LiteMcpServer {
    LiteMcpServer::production(
        "qiongli-test",
        "2.0.0-test",
        LiteToolRegistry::from_json(CONTRACT).unwrap(),
        ProviderAccess::default(),
    )
}

fn request(id: u64, method: &str, params: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
}

fn call(server: &LiteMcpServer, id: u64, name: &str, arguments: Value) -> Value {
    server
        .handle(request(
            id,
            "tools/call",
            json!({"name": name, "arguments": arguments}),
        ))
        .unwrap()
}

#[test]
fn initialize_list_ping_and_notifications_use_bounded_static_protocol_results() {
    let server = server();
    let initialized = server
        .handle(request(
            1,
            "initialize",
            json!({"protocolVersion": "peer-controlled-version"}),
        ))
        .unwrap();
    assert_eq!(
        initialized["result"]["protocolVersion"],
        MCP_PROTOCOL_VERSION
    );
    assert_eq!(initialized["result"]["serverInfo"]["name"], "qiongli-test");

    let listed = server.handle(request(2, "tools/list", json!({}))).unwrap();
    let names = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, LITE_PUBLIC_TOOL_NAMES);

    let ping = server.handle(request(3, "ping", json!({}))).unwrap();
    assert_eq!(ping["result"], json!({}));
    assert!(
        server
            .handle(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            }))
            .is_none()
    );
    assert!(
        server
            .handle(json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": {
                    "name": "qiongli_literature_search",
                    "arguments": {"query": SECRET_CANARY}
                }
            }))
            .is_none()
    );
}

#[test]
fn deferred_provider_credentials_are_not_loaded_by_protocol_or_status_calls() {
    let loads = Arc::new(AtomicUsize::new(0));
    let loader_loads = Arc::clone(&loads);
    let server = LiteMcpServer::production_deferred(
        "qiongli-test",
        "2.0.0-test",
        LiteToolRegistry::from_json(CONTRACT).unwrap(),
        ProviderAccess::default(),
        Arc::new(move || {
            loader_loads.fetch_add(1, Ordering::SeqCst);
            ProviderAccess::default()
        }),
    );

    assert!(server.handle(request(1, "initialize", json!({}))).unwrap()["result"].is_object());
    assert!(
        server.handle(request(2, "tools/list", json!({}))).unwrap()["result"]["tools"].is_array()
    );
    for (id, name, arguments) in [
        (3, "qiongli_config_status", json!({})),
        (4, "qiongli_literature_status", json!({})),
        (5, "qiongli_search_plan", json!({"query": "governance"})),
    ] {
        let response = call(&server, id, name, arguments);
        assert!(response["error"].is_null());
        assert!(response["result"]["structuredContent"].is_object());
    }
    assert_eq!(loads.load(Ordering::SeqCst), 0);
}

#[test]
fn deferred_provider_credential_load_is_bounded_and_cached() {
    let loads = Arc::new(AtomicUsize::new(0));
    let finished = Arc::new(AtomicBool::new(false));
    let loader_loads = Arc::clone(&loads);
    let loader_finished = Arc::clone(&finished);
    let server = LiteMcpServer::production_deferred_with_timeout(
        "qiongli-test",
        "2.0.0-test",
        LiteToolRegistry::from_json(CONTRACT).unwrap(),
        ProviderAccess::default(),
        Arc::new(move || {
            loader_loads.fetch_add(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(80));
            loader_finished.store(true, Ordering::SeqCst);
            ProviderAccess::default()
        }),
        Duration::from_millis(10),
    );

    let timed_out = call(
        &server,
        1,
        "qiongli_literature_search",
        json!({"query": "governance"}),
    );
    assert!(!finished.load(Ordering::SeqCst));
    assert_eq!(timed_out["result"]["isError"], true);
    assert_eq!(
        timed_out["result"]["structuredContent"]["reason_code"],
        "provider-credentials-unavailable"
    );

    thread::sleep(Duration::from_millis(100));
    assert!(finished.load(Ordering::SeqCst));
    let cached = call(
        &server,
        2,
        "qiongli_literature_search",
        json!({"query": "governance"}),
    );
    assert_eq!(cached["result"]["structuredContent"]["status"], "warning");
    assert_eq!(loads.load(Ordering::SeqCst), 1);
}

#[test]
fn every_frozen_lite_public_name_has_a_safe_native_response() {
    let server = server();

    let config = call(&server, 1, "qiongli_config_status", json!({}));
    assert_eq!(
        config["result"]["structuredContent"]["config_path"],
        "<managed-native-config>"
    );
    assert_eq!(
        config["result"]["structuredContent"]["capability_mode"],
        "strategy_only"
    );

    let save = call(
        &server,
        2,
        "qiongli_save_provider_config",
        json!({
            "provider": "semantic_scholar",
            "field": "api_key",
            "value": SECRET_CANARY
        }),
    );
    assert_eq!(save["result"]["isError"], true);
    assert_eq!(
        save["result"]["structuredContent"]["reason_code"],
        "capability-unavailable"
    );
    assert!(
        !serde_json::to_string(&save)
            .unwrap()
            .contains(SECRET_CANARY)
    );

    for (id, name) in [
        (3, "qiongli_configure_provider"),
        (4, "qiongli_open_config_wizard"),
    ] {
        let configured = call(
            &server,
            id,
            name,
            json!({"provider": "crossref", "host": "127.0.0.1", "port": 0}),
        );
        assert_eq!(configured["result"]["isError"], true);
        assert_eq!(
            configured["result"]["structuredContent"]["reason_code"],
            "capability-unavailable"
        );
    }

    let literature = call(&server, 5, "qiongli_literature_status", json!({}));
    assert_eq!(
        literature["result"]["structuredContent"]["active_providers"],
        json!([])
    );

    let plan = call(
        &server,
        6,
        "qiongli_search_plan",
        json!({
            "query": "governance",
            "from_year": 2020,
            "toYear": "2026"
        }),
    );
    let plan = &plan["result"]["structuredContent"];
    assert_eq!(plan["search_execution_mode"], "strategy_only");
    assert_eq!(
        plan["provider_queries"],
        json!([]),
        "disabled providers must not trigger network routes"
    );

    let search = call(
        &server,
        7,
        "qiongli_literature_search",
        json!({"query": "governance"}),
    );
    assert_eq!(search["result"]["structuredContent"]["status"], "warning");
    assert_eq!(
        search["result"]["structuredContent"]["diagnostics"]["status_reason"],
        "no_active_providers"
    );

    let evidence = call(
        &server,
        8,
        "qiongli_literature_export_evidence",
        json!({
            "query": "governance",
            "results": [{"title": "record", "api_key": SECRET_CANARY}],
            "diagnostics": {}
        }),
    );
    let rendered = serde_json::to_string(&evidence).unwrap();
    assert_eq!(
        evidence["result"]["structuredContent"]["artifact_type"],
        "qiongli_literature_evidence_snapshot"
    );
    assert!(!rendered.contains(SECRET_CANARY));
    assert!(evidence["result"]["structuredContent"]["results"][0]["api_key"].is_null());

    let zotero = call(&server, 9, "qiongli_zotero_status", json!({}));
    assert_eq!(zotero["result"]["structuredContent"]["status"], "disabled");
    assert_eq!(
        zotero["result"]["structuredContent"]["fallback_import_files"]["available"],
        true
    );

    let export = call(
        &server,
        10,
        "qiongli_zotero_export_import_files",
        json!({"records": [], "formats": []}),
    );
    assert_eq!(export["result"]["structuredContent"]["status"], "ok");

    let route = call(
        &server,
        11,
        "qiongli_orchestrator_route",
        json!({"request": "plan a review", "platform": "codex"}),
    );
    assert_eq!(route["result"]["structuredContent"]["preview_only"], true);
    assert_eq!(
        route["result"]["structuredContent"]["run_agents_allowed"],
        false
    );

    let task = call(
        &server,
        12,
        "qiongli_task_plan",
        json!({"task_id": " B1 ", "paper_type": " review ", "topic": " AI "}),
    );
    assert_eq!(task["result"]["structuredContent"]["task_id"], "B1");
    assert_eq!(task["result"]["structuredContent"]["preview_only"], true);
}

#[test]
fn config_failure_keeps_handshake_available_and_dependent_calls_redacted() {
    let server = LiteMcpServer::config_unavailable(
        "qiongli-test",
        "2.0.0-test",
        LiteToolRegistry::from_json(CONTRACT).unwrap(),
    );
    assert!(server.handle(request(1, "initialize", json!({}))).unwrap()["result"].is_object());
    assert_eq!(
        server.handle(request(2, "tools/list", json!({}))).unwrap()["result"]["tools"]
            .as_array()
            .unwrap()
            .len(),
        12
    );
    for (id, name, arguments) in [
        (3, "qiongli_config_status", json!({})),
        (4, "qiongli_literature_status", json!({})),
        (5, "qiongli_search_plan", json!({"query": SECRET_CANARY})),
        (
            6,
            "qiongli_literature_search",
            json!({"query": SECRET_CANARY}),
        ),
    ] {
        let response = call(&server, id, name, arguments);
        let rendered = serde_json::to_string(&response).unwrap();
        assert_eq!(response["result"]["isError"], true);
        assert_eq!(
            response["result"]["structuredContent"]["reason_code"],
            "native-config-unavailable"
        );
        assert!(!rendered.contains(SECRET_CANARY));
    }
}

#[test]
fn invalid_calls_fail_before_side_effects_without_echoing_peer_values() {
    let server = server();
    let mut unknown_argument = json!({"query": "topic"});
    unknown_argument
        .as_object_mut()
        .unwrap()
        .insert(SECRET_CANARY.to_string(), json!(true));
    for request in [
        request(
            1,
            "tools/call",
            json!({
                "name": "qiongli_configure_provider",
                "arguments": {"host": SECRET_CANARY, "port": 0}
            }),
        ),
        request(
            2,
            "tools/call",
            json!({
                "name": "qiongli_literature_search",
                "arguments": {"query": "topic", "providers": [SECRET_CANARY]}
            }),
        ),
        request(
            3,
            "tools/call",
            json!({
                "name": "qiongli_search_plan",
                "arguments": unknown_argument
            }),
        ),
    ] {
        let response = server.handle(request).unwrap();
        let rendered = serde_json::to_string(&response).unwrap();
        assert_eq!(response["error"]["code"], -32602);
        assert!(!rendered.contains(SECRET_CANARY));
    }
}

#[test]
fn serve_recovers_after_malformed_json_suppresses_notifications_and_preserves_framing() {
    let server = server();
    let input = concat!(
        "{not-json}\n",
        "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"ping\",\"params\":{}}\n"
    );
    let mut reader = BufReader::new(Cursor::new(input.as_bytes()));
    let mut output = Vec::new();
    server.serve(&mut reader, &mut output).unwrap();
    let lines = String::from_utf8(output).unwrap();
    let responses = lines
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["error"]["code"], -32700);
    assert_eq!(responses[1]["id"], 7);

    let payload = serde_json::to_string(&request(8, "tools/list", json!({}))).unwrap();
    let framed = format!("Content-Length: {}\r\n\r\n{payload}", payload.len());
    let mut reader = BufReader::new(Cursor::new(framed.into_bytes()));
    let mut output = Vec::new();
    server.serve(&mut reader, &mut output).unwrap();
    let mut response_reader = BufReader::new(Cursor::new(output));
    let response = read_message(&mut response_reader).unwrap().unwrap();
    assert_eq!(response.framing, Framing::ContentLength);
    assert_eq!(
        serde_json::from_str::<Value>(&response.payload).unwrap()["id"],
        8
    );
}
