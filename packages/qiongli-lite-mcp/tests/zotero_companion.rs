use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use qiongli_lite_mcp::zotero::companion::{CompanionClient, CompanionError};

#[test]
fn accepts_only_http_loopback_urls_without_credentials() {
    assert!(CompanionClient::new("http://127.0.0.1:23119").is_ok());
    assert!(CompanionClient::new("https://localhost:23119").is_ok());
    assert!(CompanionClient::new("http://[::1]:23119").is_ok());
    assert!(matches!(
        CompanionClient::new("http://example.com:23119"),
        Err(CompanionError::NonLoopback)
    ));
    assert!(matches!(
        CompanionClient::new("ftp://127.0.0.1:23119"),
        Err(CompanionError::InvalidScheme)
    ));
    assert!(matches!(
        CompanionClient::new("http://user:password@127.0.0.1:23119"),
        Err(CompanionError::CredentialsNotAllowed)
    ));
}

#[test]
fn disabled_status_does_not_probe_and_keeps_import_fallback() {
    let client =
        CompanionClient::with_timeout("http://127.0.0.1:9", Duration::from_millis(50)).unwrap();
    let status = client.probe(false);

    assert_eq!(status.status, "disabled");
    assert!(!status.connector.available);
    assert!(!status.companion.available);
    assert!(status.fallback_import_files.available);
}

#[test]
fn reports_fallback_only_when_connector_is_unreachable() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let client =
        CompanionClient::with_timeout(&format!("http://{address}"), Duration::from_millis(100))
            .unwrap();

    let status = client.probe(true);

    assert_eq!(status.status, "fallback_only");
    assert_eq!(status.error_code.as_deref(), Some("zotero_not_running"));
    assert!(!status.connector.available);
    assert!(status.fallback_import_files.available);
}

#[test]
fn reports_companion_missing_after_connector_only_probe() {
    let server = FixtureServer::start(vec![
        Response::ok("Zotero is running"),
        Response::not_found(),
    ]);
    let client = CompanionClient::new(&server.base_url).unwrap();

    let status = client.probe(true);

    assert_eq!(status.status, "companion_missing");
    assert_eq!(status.error_code.as_deref(), Some("companion_missing"));
    assert!(status.connector.available);
    assert_eq!(status.connector.status, Some(200));
    assert!(!status.companion.available);
    assert_eq!(status.companion.status, Some(404));
    assert_eq!(server.finish(), vec!["/connector/ping", "/qiongli/ping"]);
}

#[test]
fn reports_filtered_companion_versions_when_both_probes_succeed() {
    let server = FixtureServer::start(vec![
        Response::ok("Zotero is running"),
        Response::json(r#"{"version":"1.2.3","endpoint_version":"1"}"#),
    ]);
    let client = CompanionClient::new(&server.base_url).unwrap();

    let status = client.probe(true);

    assert_eq!(status.status, "ok");
    assert_eq!(status.error_code, None);
    assert!(status.connector.available);
    assert!(status.companion.available);
    assert_eq!(status.companion.version.as_deref(), Some("1.2.3"));
    assert_eq!(status.companion.endpoint_version.as_deref(), Some("1"));
    assert!(status.fallback_import_files.available);
    assert_eq!(server.finish(), vec!["/connector/ping", "/qiongli/ping"]);
}

#[test]
fn refuses_redirects_instead_of_following_non_loopback_location() {
    let server = FixtureServer::start(vec![Response::redirect("http://example.com/steal")]);
    let client = CompanionClient::new(&server.base_url).unwrap();

    let status = client.probe(true);

    assert_eq!(status.status, "fallback_only");
    assert_eq!(status.connector.status, Some(302));
    assert_eq!(server.finish(), vec!["/connector/ping"]);
}

#[test]
fn malformed_companion_json_is_reported_as_companion_missing() {
    let server = FixtureServer::start(vec![
        Response::ok("Zotero is running"),
        Response::json("not-json"),
    ]);
    let client = CompanionClient::new(&server.base_url).unwrap();

    let status = client.probe(true);

    assert_eq!(status.status, "companion_missing");
    assert!(status.connector.available);
    assert!(!status.companion.available);
    assert_eq!(status.companion.status, Some(200));
    assert_eq!(server.finish(), vec!["/connector/ping", "/qiongli/ping"]);
}

#[test]
fn oversized_companion_response_is_rejected() {
    let oversized = format!(r#"{{"version":"{}"}}"#, "x".repeat(33 * 1024));
    let server = FixtureServer::start(vec![
        Response::ok("Zotero is running"),
        Response::json(&oversized),
    ]);
    let client = CompanionClient::new(&server.base_url).unwrap();

    let status = client.probe(true);

    assert_eq!(status.status, "companion_missing");
    assert!(!status.companion.available);
    assert_eq!(status.companion.status, Some(200));
    assert_eq!(server.finish(), vec!["/connector/ping", "/qiongli/ping"]);
}

#[test]
fn connector_timeout_is_bounded_and_keeps_import_fallback() {
    let server = FixtureServer::start(vec![
        Response::ok("Zotero is running").with_delay(Duration::from_millis(200))
    ]);
    let client =
        CompanionClient::with_timeout(&server.base_url, Duration::from_millis(50)).unwrap();

    let started = Instant::now();
    let status = client.probe(true);

    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(status.status, "fallback_only");
    assert!(!status.connector.available);
    assert!(status.fallback_import_files.available);
    assert_eq!(server.finish(), vec!["/connector/ping"]);
}

struct FixtureServer {
    base_url: String,
    paths: Arc<Mutex<Vec<String>>>,
    worker: thread::JoinHandle<()>,
}

impl FixtureServer {
    fn start(responses: Vec<Response>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let paths = Arc::new(Mutex::new(Vec::new()));
        let worker_paths = Arc::clone(&paths);
        let worker = thread::spawn(move || {
            for response in responses {
                let deadline = Instant::now() + Duration::from_secs(2);
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "timed out waiting for probe");
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("fixture accept failed: {error}"),
                    }
                };
                // Accepted sockets inherit the listener's non-blocking mode on
                // Windows, so restore blocking reads before applying a timeout.
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut request = Vec::new();
                let mut chunk = [0_u8; 1024];
                loop {
                    let count = stream.read(&mut chunk).unwrap();
                    if count == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..count]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                let request = String::from_utf8(request).unwrap();
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap()
                    .to_string();
                worker_paths.lock().unwrap().push(path);
                thread::sleep(response.delay);
                let _ = stream.write_all(response.render().as_bytes());
                let _ = stream.flush();
            }
        });
        Self {
            base_url: format!("http://{address}"),
            paths,
            worker,
        }
    }

    fn finish(self) -> Vec<String> {
        self.worker.join().unwrap();
        Arc::try_unwrap(self.paths).unwrap().into_inner().unwrap()
    }
}

struct Response {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: String,
    location: Option<String>,
    delay: Duration,
}

impl Response {
    fn ok(body: &str) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type: "text/plain",
            body: body.to_string(),
            location: None,
            delay: Duration::ZERO,
        }
    }

    fn json(body: &str) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type: "application/json",
            body: body.to_string(),
            location: None,
            delay: Duration::ZERO,
        }
    }

    fn not_found() -> Self {
        Self {
            status: 404,
            reason: "Not Found",
            content_type: "text/plain",
            body: String::new(),
            location: None,
            delay: Duration::ZERO,
        }
    }

    fn redirect(location: &str) -> Self {
        Self {
            status: 302,
            reason: "Found",
            content_type: "text/plain",
            body: String::new(),
            location: Some(location.to_string()),
            delay: Duration::ZERO,
        }
    }

    fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    fn render(&self) -> String {
        let location = self
            .location
            .as_ref()
            .map(|value| format!("Location: {value}\r\n"))
            .unwrap_or_default();
        format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status,
            self.reason,
            self.content_type,
            location,
            self.body.len(),
            self.body
        )
    }
}
