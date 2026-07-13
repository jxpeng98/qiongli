use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use qiongli_lite_mcp::mcp::server::{McpRequest, McpServer};
use serde_json::{json, Value};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn literature_status_returns_rich_redacted_provider_state() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    let secret = "openalex-literature-status-secret";
    std::fs::write(
        temp.path.join("providers.json"),
        format!(
            r#"{{"version":1,"providers":{{"openalex":{{"enabled":true,"api_key":"{secret}"}}}}}}"#
        ),
    )
    .unwrap();
    let server = McpServer::new("qiongli-literature-provider", "test");

    let response = call(
        &server,
        "qiongli_literature_status",
        json!({"cwd": "/ignored/by/lite"}),
    );
    let payload = &response["result"]["structuredContent"];
    let rendered = serde_json::to_string(&response).unwrap();

    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["capability_mode"], "provider_connected");
    assert_eq!(payload["active_providers"], json!(["openalex", "arxiv"]));
    assert_eq!(payload["providers"]["openalex"], "configured");
    assert_eq!(
        payload["provider_capabilities"]["openalex"]["status"],
        "implemented"
    );
    let redacted_fields = payload["redacted_config"]["providers"]["openalex"]["fields"]
        .as_object()
        .unwrap();
    assert!(!redacted_fields.contains_key("api_key"));
    assert_eq!(redacted_fields["email"], "missing");
    assert_eq!(
        payload["next_action"]["args"]["provider"],
        "semantic_scholar"
    );
    assert!(!rendered.contains(secret));
    assert!(!rendered.contains(temp.path.to_string_lossy().as_ref()));
}

#[test]
fn literature_status_rejects_unknown_mistyped_and_blank_context() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    let server = McpServer::new("qiongli-literature-provider", "test");

    for arguments in [
        json!({"cwd": 7}),
        json!({"cwd": "   "}),
        json!({"unknown": true}),
    ] {
        let response = call(&server, "qiongli_literature_status", arguments);
        assert_eq!(response["error"]["code"], -32602, "{response}");
    }
}

#[test]
fn search_plan_normalizes_legacy_aliases_into_complete_hybrid_plan() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    let semantic_secret = "semantic-plan-secret";
    let openalex_secret = "openalex-plan-secret";
    std::fs::write(
        temp.path.join("providers.json"),
        format!(
            r#"{{"version":1,"providers":{{"semantic_scholar":{{"enabled":true,"api_key":"{semantic_secret}"}},"openalex":{{"enabled":true,"api_key":"{openalex_secret}"}}}}}}"#
        ),
    )
    .unwrap();
    let server = McpServer::new("qiongli-literature-provider", "test");

    let response = call(
        &server,
        "qiongli_search_plan",
        json!({
            "cwd": "/ignored/by/lite",
            "query": "AI feedback in education",
            "platform": "Claude-Code__",
            "nativeSearchAvailable": true,
            "nativeSearchTools": ["CLAUDE--web  search_"],
            "queryVariants": ["algorithmic feedback in classrooms"],
            "includeWorkingPapers": true,
            "fromYear": "2020",
            "toYear": 2026,
            "searchMode": "review",
            "venueFilter": "learning analytics",
            "documentTypes": ["journal-article"]
        }),
    );
    let payload = &response["result"]["structuredContent"];
    let rendered = serde_json::to_string(&response).unwrap();

    assert_eq!(payload["artifact_type"], "qiongli_hybrid_search_plan");
    assert_eq!(payload["search_mode"], "review");
    assert_eq!(payload["platform"], "claude_code");
    assert_eq!(payload["search_execution_mode"], "hybrid_search");
    assert_eq!(payload["native_search_tools"], json!(["claude_web_search"]));
    assert_eq!(payload["provider_queries"].as_array().unwrap().len(), 6);
    assert_eq!(
        payload["provider_queries"][0]["provider"],
        "semantic_scholar"
    );
    assert_eq!(payload["provider_queries"][2]["provider"], "openalex");
    assert_eq!(payload["provider_queries"][4]["provider"], "arxiv");
    assert_eq!(payload["provider_queries"][0]["filters"]["from_year"], 2020);
    assert_eq!(payload["provider_queries"][0]["filters"]["fromYear"], 2020);
    assert_eq!(payload["provider_queries"][0]["filters"]["to_year"], 2026);
    assert_eq!(payload["provider_queries"][0]["filters"]["toYear"], 2026);
    assert_eq!(
        payload["native_search_queries"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        payload["native_fulltext_queries"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        payload["native_fulltext_candidate_schema"]["status_values"],
        json!(["candidate_only"])
    );
    assert_eq!(
        payload["execution_sequence"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["action"],
        "merge/dedupe/search_log"
    );
    assert!(!rendered.contains(semantic_secret));
    assert!(!rendered.contains(openalex_secret));
    assert!(!rendered.contains(temp.path.to_string_lossy().as_ref()));
}

#[test]
fn search_plan_rejects_invalid_alias_year_list_and_unknown_inputs() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    let server = McpServer::new("qiongli-literature-provider", "test");
    let too_many_variants: Vec<String> = (0..17).map(|index| format!("variant {index}")).collect();

    let cases = [
        json!({"query": "   "}),
        json!({"query": 7}),
        json!({
            "query": "governance",
            "native_search_available": true,
            "nativeSearchAvailable": true
        }),
        json!({"query": "governance", "from_year": "20x0"}),
        json!({"query": "governance", "from_year": 2026, "to_year": 2020}),
        json!({"query": "governance", "query_variants": too_many_variants}),
        json!({"query": "governance", "document_types": ["article", "article"]}),
        json!({"query": "governance", "unknown": true}),
    ];

    for arguments in cases {
        let response = call(&server, "qiongli_search_plan", arguments);
        assert_eq!(response["error"]["code"], -32602, "{response}");
    }
}

#[test]
fn search_plan_reports_native_only_and_strategy_only_for_disabled_providers() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    std::fs::write(
        temp.path.join("providers.json"),
        r#"{"version":1,"providers":{"arxiv":{"enabled":false}}}"#,
    )
    .unwrap();
    let server = McpServer::new("qiongli-literature-provider", "test");

    let native = call(
        &server,
        "qiongli_search_plan",
        json!({"query": "governance", "native_search_available": true}),
    );
    let strategy = call(
        &server,
        "qiongli_search_plan",
        json!({"query": "governance", "native_search_available": false}),
    );

    assert_eq!(
        native["result"]["structuredContent"]["search_execution_mode"],
        "native_only"
    );
    assert_eq!(
        strategy["result"]["structuredContent"]["search_execution_mode"],
        "strategy_only"
    );
}

#[test]
fn malformed_provider_config_is_a_redacted_tool_error_for_status_and_plan() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    let malformed = "{not-json provider-secret-canary";
    let config_path = temp.path.join("providers.json");
    std::fs::write(&config_path, malformed).unwrap();
    let original = std::fs::read(&config_path).unwrap();
    let server = McpServer::new("qiongli-literature-provider", "test");

    for (tool, arguments, expected_message) in [
        (
            "qiongli_config_status",
            json!({}),
            "provider configuration is unavailable",
        ),
        (
            "qiongli_save_provider_config",
            json!({"provider": "crossref", "field": "email", "value": "write-secret-canary"}),
            "provider configuration could not be saved",
        ),
        (
            "qiongli_literature_status",
            json!({}),
            "provider configuration is unavailable",
        ),
        (
            "qiongli_search_plan",
            json!({"query": "governance"}),
            "provider configuration is unavailable",
        ),
        (
            "qiongli_literature_search",
            json!({"query": "governance"}),
            "provider configuration is unavailable",
        ),
    ] {
        let response = call(&server, tool, arguments);
        let rendered = serde_json::to_string(&response).unwrap();
        assert_eq!(response["result"]["isError"], true, "{response}");
        assert_eq!(
            response["result"]["structuredContent"]["error_kind"],
            "tool_error"
        );
        assert_eq!(
            response["result"]["structuredContent"]["message"],
            expected_message
        );
        assert!(!rendered.contains("provider-secret-canary"));
        assert!(!rendered.contains("write-secret-canary"));
        assert!(!rendered.contains(temp.path.to_string_lossy().as_ref()));
        assert_eq!(std::fs::read(&config_path).unwrap(), original);
    }
}

#[test]
fn config_wizard_path_failure_is_a_fixed_redacted_tool_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = TempDir::new();
    let _env = IsolatedProviderEnv::new(&temp.path);
    let path_canary = "relative-config-path-canary";
    std::env::set_var("QIONGLI_CONFIG_HOME", path_canary);
    let server = McpServer::new("qiongli-literature-provider", "test");

    let response = call(&server, "qiongli_configure_provider", json!({}));
    let rendered = serde_json::to_string(&response).unwrap();

    assert_eq!(response["result"]["isError"], true, "{response}");
    assert_eq!(
        response["result"]["structuredContent"]["error_kind"],
        "tool_error"
    );
    assert_eq!(
        response["result"]["structuredContent"]["message"],
        "provider configuration wizard could not start"
    );
    assert!(!rendered.contains(path_canary));
}

fn call(server: &McpServer, name: &str, arguments: Value) -> Value {
    server.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({"name": name, "arguments": arguments})),
    })
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "qiongli-lite-literature-plan-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

struct IsolatedProviderEnv {
    previous: Vec<(String, Option<OsString>)>,
}

impl IsolatedProviderEnv {
    fn new(config_home: &std::path::Path) -> Self {
        let names = [
            "QIONGLI_CONFIG_HOME",
            "QIONGLI_OPENALEX_API_KEY",
            "OPENALEX_API_KEY",
            "QIONGLI_MCPB_OPENALEX_API_KEY",
            "QIONGLI_OPENALEX_EMAIL",
            "OPENALEX_EMAIL",
            "QIONGLI_MCPB_OPENALEX_EMAIL",
            "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
            "SEMANTIC_SCHOLAR_API_KEY",
            "S2_API_KEY",
            "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
            "QIONGLI_CROSSREF_EMAIL",
            "CROSSREF_EMAIL",
            "QIONGLI_MCPB_CROSSREF_EMAIL",
            "QIONGLI_NCBI_API_KEY",
            "NCBI_API_KEY",
            "PUBMED_API_KEY",
            "QIONGLI_MCPB_PUBMED_API_KEY",
        ];
        let previous = names
            .iter()
            .map(|name| ((*name).to_string(), std::env::var_os(name)))
            .collect();
        for name in names {
            std::env::remove_var(name);
        }
        std::env::set_var("QIONGLI_CONFIG_HOME", config_home);
        Self { previous }
    }
}

impl Drop for IsolatedProviderEnv {
    fn drop(&mut self) {
        for (name, value) in &self.previous {
            if let Some(value) = value {
                std::env::set_var(name, value);
            } else {
                std::env::remove_var(name);
            }
        }
    }
}
