use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;
use std::time::Duration;

use qiongli_lite_mcp::config::provider_config::ResolvedProviderConfig;
use qiongli_lite_mcp::providers::arxiv::search_arxiv;
use qiongli_lite_mcp::providers::crossref::search_crossref;
use qiongli_lite_mcp::providers::openalex::search_openalex;
use qiongli_lite_mcp::providers::pubmed::search_pubmed;
use qiongli_lite_mcp::providers::runtime::{
    ProviderEndpoints, ProviderRuntime, ProviderRuntimeError,
};
use qiongli_lite_mcp::providers::search::SearchInput;
use qiongli_lite_mcp::providers::semantic_scholar::search_semantic_scholar;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;

const OPENALEX_RESPONSE: &str =
    include_str!("../../../content/mcp-contracts/fixtures/openalex-search-response.json");
const SEMANTIC_SCHOLAR_RESPONSE: &str =
    include_str!("../../../content/mcp-contracts/fixtures/semantic-scholar-search-response.json");
const CROSSREF_RESPONSE: &str =
    include_str!("../../../content/mcp-contracts/fixtures/crossref-search-response.json");
const PUBMED_SUMMARY_RESPONSE: &str =
    include_str!("../../../content/mcp-contracts/fixtures/pubmed-summary-response.json");
const ARXIV_RESPONSE: &str =
    include_str!("../../../content/mcp-contracts/fixtures/arxiv-search-response.xml");

#[test]
fn openalex_request_includes_activation_key_optional_email_and_limit() {
    let server = FakeServer::start(vec![FakeResponse::json(200, OPENALEX_RESPONSE)]);
    let config = ResolvedProviderConfig::from_values(&[
        ("openalex", "api_key", "openalex-secret"),
        ("openalex", "email", "person@example.com"),
    ])
    .unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let results = search_openalex(&runtime, &search_input(3)).unwrap();
    let request = server.finish().remove(0);
    let url = request.url();

    assert_eq!(url.path(), "/works");
    assert_eq!(query_value(&url, "search").as_deref(), Some("AI feedback"));
    assert_eq!(query_value(&url, "per-page").as_deref(), Some("3"));
    assert_eq!(
        query_value(&url, "api_key").as_deref(),
        Some("openalex-secret")
    );
    assert_eq!(
        query_value(&url, "mailto").as_deref(),
        Some("person@example.com")
    );
    assert_eq!(results[0].provider, "openalex");
}

#[test]
fn semantic_scholar_request_uses_api_key_header_without_query_leak() {
    let server = FakeServer::start(vec![FakeResponse::json(200, SEMANTIC_SCHOLAR_RESPONSE)]);
    let config =
        ResolvedProviderConfig::from_values(&[("semantic-scholar", "api-key", "semantic-secret")])
            .unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let results = search_semantic_scholar(&runtime, &search_input(4)).unwrap();
    let request = server.finish().remove(0);
    let url = request.url();

    assert_eq!(url.path(), "/paper/search");
    assert_eq!(query_value(&url, "limit").as_deref(), Some("4"));
    assert_eq!(request.header("x-api-key"), Some("semantic-secret"));
    assert!(!request.target.contains("semantic-secret"));
    assert_eq!(results[0].provider, "semantic_scholar");
}

#[test]
fn crossref_request_includes_polite_pool_email() {
    let server = FakeServer::start(vec![FakeResponse::json(200, CROSSREF_RESPONSE)]);
    let config =
        ResolvedProviderConfig::from_values(&[("crossref", "email", "crossref@example.com")])
            .unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let results = search_crossref(&runtime, &search_input(5)).unwrap();
    let request = server.finish().remove(0);
    let url = request.url();

    assert_eq!(url.path(), "/works");
    assert_eq!(query_value(&url, "rows").as_deref(), Some("5"));
    assert_eq!(
        query_value(&url, "mailto").as_deref(),
        Some("crossref@example.com")
    );
    assert_eq!(results[0].provider, "crossref");
}

#[test]
fn pubmed_search_calls_esearch_then_esummary_with_api_key() {
    let server = FakeServer::start(vec![
        FakeResponse::json(200, r#"{"esearchresult":{"count":"1","idlist":["1"]}}"#),
        FakeResponse::json(200, PUBMED_SUMMARY_RESPONSE),
    ]);
    let config =
        ResolvedProviderConfig::from_values(&[("pubmed", "api_key", "pubmed-secret")]).unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let results = search_pubmed(&runtime, &search_input(6)).unwrap();
    let requests = server.finish();
    let search_url = requests[0].url();
    let summary_url = requests[1].url();

    assert_eq!(search_url.path(), "/esearch.fcgi");
    assert_eq!(
        query_value(&search_url, "term").as_deref(),
        Some("AI feedback")
    );
    assert_eq!(query_value(&search_url, "retmax").as_deref(), Some("6"));
    assert_eq!(
        query_value(&search_url, "api_key").as_deref(),
        Some("pubmed-secret")
    );
    assert_eq!(summary_url.path(), "/esummary.fcgi");
    assert_eq!(query_value(&summary_url, "id").as_deref(), Some("1"));
    assert_eq!(
        query_value(&summary_url, "api_key").as_deref(),
        Some("pubmed-secret")
    );
    assert_eq!(results[0].provider, "pubmed");
}

#[test]
fn arxiv_request_uses_atom_endpoint_and_bounded_limit() {
    let server = FakeServer::start(vec![FakeResponse::xml(200, ARXIV_RESPONSE)]);
    let config = ResolvedProviderConfig::from_values(&[]).unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let results = search_arxiv(&runtime, &search_input(999)).unwrap();
    let request = server.finish().remove(0);
    let url = request.url();

    assert_eq!(url.path(), "/api/query");
    assert_eq!(
        query_value(&url, "search_query").as_deref(),
        Some("all:AI feedback")
    );
    assert_eq!(query_value(&url, "max_results").as_deref(), Some("200"));
    assert_eq!(request.header("accept"), Some("application/atom+xml"));
    assert_eq!(results[0].provider, "arxiv");
}

#[test]
fn provider_errors_are_stable_and_do_not_render_credential_urls() {
    let server = FakeServer::start(vec![FakeResponse::json(401, r#"{"error":"denied"}"#)]);
    let config = ResolvedProviderConfig::from_values(&[(
        "openalex",
        "api_key",
        "do-not-render-this-secret",
    )])
    .unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let error = search_openalex(&runtime, &search_input(1)).unwrap_err();
    let rendered = format!("{error:?} {error}");
    let requests = server.finish();

    assert_eq!(error, ProviderRuntimeError::HttpStatus { status: 401 });
    assert_eq!(error.code(), "http_error");
    assert_eq!(error.http_status(), Some(401));
    assert!(!rendered.contains("do-not-render-this-secret"));
    assert_eq!(requests.len(), 1);
}

#[test]
fn malformed_provider_payload_returns_redacted_decode_error() {
    let server = FakeServer::start(vec![FakeResponse::json(200, "not-json")]);
    let config =
        ResolvedProviderConfig::from_values(&[("openalex", "api_key", "another-secret")]).unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let error = search_openalex(&runtime, &search_input(1)).unwrap_err();
    let requests = server.finish();

    assert_eq!(error, ProviderRuntimeError::Decode);
    assert_eq!(error.to_string(), "provider response could not be decoded");
    assert!(!error.to_string().contains("another-secret"));
    assert_eq!(requests.len(), 1);
}

#[test]
fn provider_query_preserves_unicode_and_percent_encoding() {
    let server = FakeServer::start(vec![FakeResponse::json(200, OPENALEX_RESPONSE)]);
    let config = ResolvedProviderConfig::from_values(&[("openalex", "api_key", "key")]).unwrap();
    let runtime = test_runtime(&server.base_url, config);
    let mut input = search_input(2);
    input.query = "治理 学术".to_string();

    search_openalex(&runtime, &input).unwrap();
    let request = server.finish().remove(0);
    let url = request.url();

    assert_eq!(query_value(&url, "search").as_deref(), Some("治理 学术"));
    assert!(request.target.contains('%'));
}

#[test]
fn provider_redirect_is_reported_without_following_location() {
    let server = FakeServer::start(vec![FakeResponse::redirect(
        "http://example.com/credential-sink",
    )]);
    let config =
        ResolvedProviderConfig::from_values(&[("openalex", "api_key", "redirect-secret")]).unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let error = search_openalex(&runtime, &search_input(1)).unwrap_err();
    let requests = server.finish();

    assert_eq!(error, ProviderRuntimeError::HttpStatus { status: 302 });
    assert_eq!(requests.len(), 1);
    assert!(!format!("{error:?} {error}").contains("redirect-secret"));
}

#[test]
fn provider_timeout_maps_to_stable_sanitized_error() {
    let server = FakeServer::start(vec![
        FakeResponse::json(200, OPENALEX_RESPONSE).with_delay(Duration::from_millis(200))
    ]);
    let config =
        ResolvedProviderConfig::from_values(&[("openalex", "api_key", "timeout-secret")]).unwrap();
    let endpoints = ProviderEndpoints::from_urls(
        &server.base_url,
        &server.base_url,
        &server.base_url,
        &server.base_url,
        &server.base_url,
    )
    .unwrap();
    let client = Client::builder()
        .connect_timeout(Duration::from_millis(50))
        .timeout(Duration::from_millis(50))
        .redirect(Policy::none())
        .build()
        .unwrap();
    let runtime = ProviderRuntime::with_client(client, endpoints, config);

    let error = search_openalex(&runtime, &search_input(1)).unwrap_err();
    let requests = server.finish();

    assert_eq!(error, ProviderRuntimeError::Timeout);
    assert_eq!(requests.len(), 1);
    assert!(!format!("{error:?} {error}").contains("timeout-secret"));
}

#[test]
fn oversized_provider_response_is_rejected_before_decode() {
    let oversized = format!(
        r#"{{"results":[],"padding":"{}"}}"#,
        "x".repeat(4 * 1024 * 1024)
    );
    let server = FakeServer::start(vec![FakeResponse::json(200, oversized)]);
    let config = ResolvedProviderConfig::from_values(&[("openalex", "api_key", "key")]).unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let error = search_openalex(&runtime, &search_input(1)).unwrap_err();
    server.finish();

    assert_eq!(error, ProviderRuntimeError::Decode);
}

#[test]
fn malformed_arxiv_xml_returns_decode_error() {
    let server = FakeServer::start(vec![FakeResponse::xml(
        200,
        "<feed><entry><title>bad &undefined;</title></entry></feed>",
    )]);
    let runtime = test_runtime(
        &server.base_url,
        ResolvedProviderConfig::from_values(&[]).unwrap(),
    );

    let error = search_arxiv(&runtime, &search_input(1)).unwrap_err();
    server.finish();

    assert_eq!(error, ProviderRuntimeError::Decode);
}

#[test]
fn pubmed_empty_id_list_skips_summary_request() {
    let server = FakeServer::start(vec![FakeResponse::json(
        200,
        r#"{"esearchresult":{"count":"0","idlist":[]}}"#,
    )]);
    let config =
        ResolvedProviderConfig::from_values(&[("pubmed", "api_key", "pubmed-secret")]).unwrap();
    let runtime = test_runtime(&server.base_url, config);

    let records = search_pubmed(&runtime, &search_input(200)).unwrap();
    let requests = server.finish();

    assert!(records.is_empty());
    assert_eq!(requests.len(), 1);
    assert_eq!(
        query_value(&requests[0].url(), "retmax").as_deref(),
        Some("200")
    );
}

fn search_input(limit: usize) -> SearchInput {
    SearchInput {
        query: "AI feedback".to_string(),
        search_mode: Some("topic".to_string()),
        limit: Some(limit),
        per_provider_limit: None,
        total_limit: None,
    }
}

fn test_runtime(base_url: &str, config: ResolvedProviderConfig) -> ProviderRuntime {
    let endpoints =
        ProviderEndpoints::from_urls(base_url, base_url, base_url, base_url, base_url).unwrap();
    ProviderRuntime::with_endpoints(endpoints, config).unwrap()
}

fn query_value(url: &url::Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
}

struct FakeResponse {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
    location: Option<String>,
    delay: Duration,
}

impl FakeResponse {
    fn json(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: body.into(),
            location: None,
            delay: Duration::ZERO,
        }
    }

    fn xml(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            content_type: "application/atom+xml",
            body: body.into(),
            location: None,
            delay: Duration::ZERO,
        }
    }

    fn redirect(location: &str) -> Self {
        Self {
            status: 302,
            content_type: "text/plain",
            body: Vec::new(),
            location: Some(location.to_string()),
            delay: Duration::ZERO,
        }
    }

    fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

struct CapturedRequest {
    target: String,
    headers: Vec<(String, String)>,
}

impl CapturedRequest {
    fn url(&self) -> url::Url {
        url::Url::parse(&format!("http://localhost{}", self.target)).unwrap()
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

struct FakeServer {
    base_url: String,
    requests: Receiver<CapturedRequest>,
    handle: JoinHandle<()>,
}

impl FakeServer {
    fn start(responses: Vec<FakeResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (sender, requests) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_request(&mut stream);
                sender.send(request).unwrap();
                write_response(&mut stream, response);
            }
        });
        Self {
            base_url: format!("http://{address}/"),
            requests,
            handle,
        }
    }

    fn finish(self) -> Vec<CapturedRequest> {
        self.handle.join().unwrap();
        self.requests.try_iter().collect()
    }
}

fn read_request(stream: &mut TcpStream) -> CapturedRequest {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        let count = stream.read(&mut chunk).unwrap();
        assert!(count > 0, "client closed before sending HTTP headers");
        bytes.extend_from_slice(&chunk[..count]);
        assert!(
            bytes.len() < 64 * 1024,
            "request headers exceeded test limit"
        );
    }
    let request = String::from_utf8(bytes).unwrap();
    let mut lines = request.split("\r\n");
    let request_line = lines.next().unwrap();
    let target = request_line.split_whitespace().nth(1).unwrap().to_string();
    let headers = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        .collect();
    CapturedRequest { target, headers }
}

fn write_response(stream: &mut TcpStream, response: FakeResponse) {
    std::thread::sleep(response.delay);
    let reason = match response.status {
        200 => "OK",
        302 => "Found",
        _ => "Error",
    };
    let location = response
        .location
        .as_deref()
        .map(|value| format!("Location: {value}\r\n"))
        .unwrap_or_default();
    let headers = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        reason,
        response.content_type,
        location,
        response.body.len()
    );
    let _ = stream.write_all(headers.as_bytes());
    let _ = stream.write_all(&response.body);
    let _ = stream.flush();
}
