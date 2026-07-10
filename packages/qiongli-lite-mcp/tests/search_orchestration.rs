use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use qiongli_lite_mcp::config::provider_config::ResolvedProviderConfig;
use qiongli_lite_mcp::mcp::server::{McpRequest, McpServer};
use qiongli_lite_mcp::providers::runtime::{ProviderEndpoints, ProviderRuntime};
use qiongli_lite_mcp::providers::search::{
    deduplicate_results, default_limit_from_value, execute_search, limit_for, LiteratureResult,
    SearchInput,
};
use serde_json::json;

#[test]
fn orchestrates_in_provider_order_deduplicates_and_applies_total_limit() {
    let openalex = json!({
        "results": [
            {
                "display_name": "A Test Paper",
                "publication_year": 2025,
                "doi": "https://doi.org/10.1234/example",
                "primary_location": {"source": {"display_name": "Journal of Tests"}}
            },
            {
                "display_name": "B Test Paper",
                "publication_year": 2024,
                "doi": "10.1234/b",
                "primary_location": {"source": {"display_name": "Journal B"}}
            }
        ]
    })
    .to_string();
    let semantic_scholar = json!({
        "data": [
            {
                "title": "A Test Paper",
                "year": 2025,
                "venue": "Journal of Tests",
                "externalIds": {"DOI": "10.1234/example"}
            },
            {
                "title": "C Test Paper",
                "year": 2023,
                "venue": "Journal C",
                "externalIds": {"DOI": "10.1234/c"}
            }
        ]
    })
    .to_string();
    let server = MockServer::start(vec![
        Reply::ok("/openalex/works", openalex).delayed(Duration::from_millis(80)),
        Reply::ok("/s2/paper/search", semantic_scholar),
    ]);
    let runtime = runtime_for(
        &server.base_url,
        &[
            ("openalex", "api_key", "oa-canary"),
            ("semantic_scholar", "api_key", "s2-canary"),
        ],
    );
    let input = search_input(Some(2));
    let selected = vec!["openalex".to_string(), "semantic_scholar".to_string()];

    let output = execute_search(&runtime, &input, Some(&selected));
    let requests = server.finish();

    assert_eq!(output.status, "ok");
    assert_eq!(output.diagnostics.status, "complete");
    assert_eq!(output.results.len(), 2);
    assert_eq!(output.results[0].title, "A Test Paper");
    assert_eq!(
        output.results[0].providers,
        vec!["openalex", "semantic_scholar"]
    );
    assert_eq!(output.results[1].title, "B Test Paper");
    assert!(requests
        .iter()
        .any(|request| request.contains("per-page=25")));
    assert!(requests.iter().any(|request| request.contains("limit=25")));
}

#[test]
fn preserves_successful_results_when_one_provider_fails_without_leaking_keys() {
    let server = MockServer::start(vec![
        Reply::ok(
            "/openalex/works",
            include_str!("../../../content/mcp-contracts/fixtures/openalex-search-response.json"),
        ),
        Reply::status("/s2/paper/search", 503, "provider unavailable"),
    ]);
    let runtime = runtime_for(
        &server.base_url,
        &[
            ("openalex", "api_key", "oa-canary"),
            ("semantic_scholar", "api_key", "s2-canary"),
        ],
    );
    let selected = vec!["openalex".to_string(), "semantic_scholar".to_string()];

    let output = execute_search(&runtime, &search_input(None), Some(&selected));
    server.finish();
    let rendered = serde_json::to_string(&output).unwrap();

    assert_eq!(output.status, "warning");
    assert_eq!(output.diagnostics.status, "partial");
    assert_eq!(output.results.len(), 1);
    assert_eq!(
        output.diagnostics.providers["semantic_scholar"]
            .error_kind
            .as_deref(),
        Some("http_error")
    );
    assert!(!rendered.contains("oa-canary"));
    assert!(!rendered.contains("s2-canary"));
}

#[test]
fn reports_error_when_every_attempted_provider_fails() {
    let server = MockServer::start(vec![
        Reply::status("/openalex/works", 500, "failed"),
        Reply::status("/s2/paper/search", 500, "failed"),
    ]);
    let runtime = runtime_for(
        &server.base_url,
        &[
            ("openalex", "api_key", "oa-canary"),
            ("semantic_scholar", "api_key", "s2-canary"),
        ],
    );
    let selected = vec!["openalex".to_string(), "semantic_scholar".to_string()];

    let output = execute_search(&runtime, &search_input(None), Some(&selected));
    server.finish();

    assert_eq!(output.status, "error");
    assert_eq!(output.diagnostics.status, "failed");
    assert!(output.results.is_empty());
}

#[test]
fn reports_warning_when_successful_provider_returns_no_results() {
    let server = MockServer::start(vec![Reply::ok("/openalex/works", r#"{"results":[]}"#)]);
    let runtime = runtime_for(&server.base_url, &[("openalex", "api_key", "oa-canary")]);
    let selected = vec!["openalex".to_string()];

    let output = execute_search(&runtime, &search_input(None), Some(&selected));
    server.finish();

    assert_eq!(output.status, "warning");
    assert_eq!(output.diagnostics.status, "complete");
    assert!(output.results.is_empty());
    assert_eq!(
        output.diagnostics.warnings,
        vec!["configured providers returned no results"]
    );
}

#[test]
fn selected_unconfigured_provider_is_not_called() {
    let runtime = runtime_for("http://127.0.0.1:9", &[]);
    let selected = vec!["openalex".to_string()];

    let output = execute_search(&runtime, &search_input(None), Some(&selected));

    assert_eq!(output.status, "warning");
    assert_eq!(output.diagnostics.status, "not_run");
    assert_eq!(
        output.diagnostics.status_reason.as_deref(),
        Some("no_active_providers")
    );
    assert!(output.diagnostics.providers.is_empty());
    assert_eq!(
        output.diagnostics.warnings,
        vec!["no active literature providers; no network search was performed"]
    );
}

#[test]
fn mcp_plan_is_strategy_only_when_selected_provider_is_unconfigured() {
    let runtime = runtime_for("http://127.0.0.1:9", &[]);
    let mcp = McpServer::with_provider_runtime("qiongli-literature-provider", "test", runtime);

    let response = mcp.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_search",
            "arguments": {"query": "governance", "providers": ["openalex"]}
        })),
    });
    let payload = &response["result"]["structuredContent"];

    assert_eq!(payload["status"], "warning");
    assert_eq!(payload["diagnostics"]["status"], "not_run");
    assert_eq!(
        payload["diagnostics"]["status_reason"],
        "no_active_providers"
    );
    assert_eq!(
        payload["search_plan"]["search_execution_mode"],
        "strategy_only"
    );
}

#[test]
fn title_and_year_fallback_deduplicates_without_erasing_unicode() {
    let results = deduplicate_results(vec![
        result("治理 平台：证据", None, Some(2025), "openalex"),
        result("治理平台 证据", None, Some(2025), "semantic_scholar"),
        result("治理平台 证据", None, Some(2024), "crossref"),
    ]);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].providers, vec!["openalex", "semantic_scholar"]);
    assert_eq!(results[1].year, Some(2024));
}

#[test]
fn limits_use_review_default_and_clamp_explicit_values() {
    let review = SearchInput {
        query: "governance".to_string(),
        search_mode: Some("systematic_review".to_string()),
        limit: None,
        per_provider_limit: None,
        total_limit: None,
    };
    let explicit = SearchInput {
        per_provider_limit: Some(999),
        ..review.clone()
    };

    assert_eq!(limit_for(&review), 50);
    assert_eq!(limit_for(&explicit), 200);
    assert_eq!(default_limit_from_value(Some("37")), 37);
    assert_eq!(default_limit_from_value(Some("0")), 25);
    assert_eq!(default_limit_from_value(Some("201")), 25);
    assert_eq!(default_limit_from_value(Some("invalid")), 25);
    assert_eq!(default_limit_from_value(Some("  ")), 25);
}

#[test]
fn per_provider_limit_is_enforced_even_if_provider_overreturns() {
    let response = json!({
        "results": [
            {"display_name": "Paper 1", "publication_year": 2025},
            {"display_name": "Paper 2", "publication_year": 2024},
            {"display_name": "Paper 3", "publication_year": 2023}
        ]
    })
    .to_string();
    let server = MockServer::start(vec![Reply::ok("/openalex/works", response)]);
    let runtime = runtime_for(&server.base_url, &[("openalex", "api_key", "oa-canary")]);
    let input = SearchInput {
        per_provider_limit: Some(2),
        ..search_input(None)
    };
    let selected = vec!["openalex".to_string()];

    let output = execute_search(&runtime, &input, Some(&selected));
    server.finish();

    assert_eq!(output.results.len(), 2);
    assert_eq!(output.diagnostics.providers["openalex"].count, 2);
}

#[test]
fn mcp_handler_executes_injected_provider_search() {
    let server = MockServer::start(vec![Reply::ok(
        "/openalex/works",
        include_str!("../../../content/mcp-contracts/fixtures/openalex-search-response.json"),
    )]);
    let runtime = runtime_for(&server.base_url, &[("openalex", "api_key", "oa-canary")]);
    let mcp = McpServer::with_provider_runtime("qiongli-literature-provider", "test", runtime);

    let response = mcp.handle(McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "qiongli_literature_search",
            "arguments": {
                "query": "platform governance",
                "providers": ["openalex"],
                "total_limit": 10
            }
        })),
    });
    server.finish();
    let payload = &response["result"]["structuredContent"];

    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["diagnostics"]["status"], "complete");
    assert_eq!(payload["results"].as_array().unwrap().len(), 1);
    assert_ne!(payload["diagnostics"]["status"], "not_run");
    assert_eq!(
        payload["search_plan"]["search_execution_mode"],
        "provider_connected"
    );
}

fn runtime_for(base_url: &str, values: &[(&str, &str, &str)]) -> ProviderRuntime {
    let endpoints = ProviderEndpoints::from_urls(
        &format!("{base_url}/openalex/"),
        &format!("{base_url}/s2/"),
        &format!("{base_url}/crossref/"),
        &format!("{base_url}/pubmed/"),
        &format!("{base_url}/arxiv/"),
    )
    .unwrap();
    let config = ResolvedProviderConfig::from_values(values).unwrap();
    ProviderRuntime::with_endpoints(endpoints, config).unwrap()
}

fn search_input(total_limit: Option<usize>) -> SearchInput {
    SearchInput {
        query: "platform governance".to_string(),
        search_mode: Some("topic".to_string()),
        limit: None,
        per_provider_limit: None,
        total_limit,
    }
}

fn result(title: &str, doi: Option<&str>, year: Option<i64>, provider: &str) -> LiteratureResult {
    LiteratureResult {
        title: title.to_string(),
        doi: doi.map(ToString::to_string),
        year,
        venue: None,
        provider: provider.to_string(),
        providers: vec![provider.to_string()],
    }
}

struct Reply {
    path: String,
    status: u16,
    body: String,
    delay: Duration,
}

impl Reply {
    fn ok(path: &str, body: impl Into<String>) -> Self {
        Self::status(path, 200, body)
    }

    fn status(path: &str, status: u16, body: impl Into<String>) -> Self {
        Self {
            path: path.to_string(),
            status,
            body: body.into(),
            delay: Duration::ZERO,
        }
    }

    fn delayed(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

struct MockServer {
    base_url: String,
    requests: Arc<Mutex<Vec<String>>>,
    pending: Arc<Mutex<Vec<Reply>>>,
    worker: thread::JoinHandle<()>,
}

impl MockServer {
    fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let pending = Arc::new(Mutex::new(replies));
        let worker_requests = Arc::clone(&requests);
        let worker_pending = Arc::clone(&pending);
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                if worker_pending.lock().unwrap().is_empty() {
                    break;
                }
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(_) => break,
                };
                stream.set_nonblocking(false).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut first_line = String::new();
                reader.read_line(&mut first_line).unwrap();
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if header == "\r\n" || header == "\n" || header.is_empty() {
                        break;
                    }
                }
                worker_requests.lock().unwrap().push(first_line.clone());
                let mut replies = worker_pending.lock().unwrap();
                let position = replies
                    .iter()
                    .position(|reply| first_line.contains(&reply.path));
                let reply = position
                    .map(|position| replies.remove(position))
                    .unwrap_or_else(|| Reply::status("unexpected", 500, "unexpected request"));
                drop(replies);
                thread::sleep(reply.delay);
                let reason = if reply.status == 200 { "OK" } else { "Error" };
                write!(
                    stream,
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.status,
                    reason,
                    reply.body.len(),
                    reply.body
                )
                .unwrap();
                stream.flush().unwrap();
            }
        });
        Self {
            base_url: format!("http://{address}"),
            requests,
            pending,
            worker,
        }
    }

    fn finish(self) -> Vec<String> {
        self.worker.join().unwrap();
        assert!(
            self.pending.lock().unwrap().is_empty(),
            "not every expected provider request was observed"
        );
        Arc::try_unwrap(self.requests)
            .unwrap()
            .into_inner()
            .unwrap()
    }
}
