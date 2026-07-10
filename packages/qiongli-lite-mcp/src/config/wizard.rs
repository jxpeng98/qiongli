use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use thiserror::Error;
use url::form_urlencoded;

use crate::config::provider_config::{
    normalize_key, provider_config_path, save_provider_value_at, ConfigError,
};

pub const DEFAULT_WIZARD_TTL: Duration = Duration::from_secs(10 * 60);
pub const DEFAULT_MAX_BODY_BYTES: usize = 16 * 1024;

const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_ALLOWED_BODY_BYTES: usize = 64 * 1024;
const MAX_FIELD_BYTES: usize = 4 * 1024;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(2);
const ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(10);
const SAVE_REDIRECT_GRACE: Duration = Duration::from_secs(1);
const TOKEN_BYTES: usize = 24;

#[derive(Debug, Clone)]
pub struct ConfigWizardOptions {
    pub host: String,
    pub port: u16,
    pub provider: Option<String>,
    pub ttl: Duration,
    pub max_body_bytes: usize,
}

impl Default for ConfigWizardOptions {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 0,
            provider: None,
            ttl: DEFAULT_WIZARD_TTL,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        }
    }
}

#[derive(Debug, Error)]
pub enum WizardError {
    #[error("wizard host must be 127.0.0.1 or localhost")]
    NonLoopbackHost,
    #[error("unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("wizard TTL must be greater than zero")]
    InvalidTtl,
    #[error("wizard request body limit must be between 1 and {MAX_ALLOWED_BODY_BYTES} bytes")]
    InvalidBodyLimit,
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("failed to bind local configuration wizard: {0}")]
    Bind(io::Error),
    #[error("failed to configure local configuration wizard: {0}")]
    Listener(io::Error),
    #[error("failed to initialize configuration wizard token: {0}")]
    Random(io::Error),
    #[error("failed to start local configuration wizard: {0}")]
    Spawn(io::Error),
}

#[derive(Debug)]
pub struct ConfigWizard {
    host: String,
    port: u16,
    url: String,
    config_path: PathBuf,
    provider: Option<String>,
    running: Arc<AtomicBool>,
    completed: Arc<AtomicBool>,
    stop_requested: Arc<AtomicBool>,
    _worker: JoinHandle<()>,
}

impl ConfigWizard {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::Acquire)
    }

    pub fn stop(&self) {
        self.stop_requested.store(true, Ordering::Release);
    }

    pub fn wait_until_stopped(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while self.is_running() && Instant::now() < deadline {
            thread::sleep(ACCEPT_POLL_INTERVAL);
        }
        !self.is_running()
    }
}

impl Drop for ConfigWizard {
    fn drop(&mut self) {
        self.stop_requested.store(true, Ordering::Release);
    }
}

pub fn start_config_wizard(options: ConfigWizardOptions) -> Result<ConfigWizard, WizardError> {
    let host = normalize_host(&options.host)?;
    let provider = normalize_provider(options.provider.as_deref())?;
    if options.ttl.is_zero() {
        return Err(WizardError::InvalidTtl);
    }
    if !(1..=MAX_ALLOWED_BODY_BYTES).contains(&options.max_body_bytes) {
        return Err(WizardError::InvalidBodyLimit);
    }
    let config_path = provider_config_path()?;

    let token = secure_token().map_err(WizardError::Random)?;
    let listener = TcpListener::bind((host.as_str(), options.port)).map_err(WizardError::Bind)?;
    listener
        .set_nonblocking(true)
        .map_err(WizardError::Listener)?;
    let address = listener.local_addr().map_err(WizardError::Listener)?;
    let port = address.port();
    let url = format!("http://{host}:{port}/?token={token}");
    let running = Arc::new(AtomicBool::new(true));
    let completed = Arc::new(AtomicBool::new(false));
    let stop_requested = Arc::new(AtomicBool::new(false));

    let worker_state = WizardWorkerState {
        token,
        selected_provider: provider.clone(),
        config_path: config_path.clone(),
        ttl: options.ttl,
        max_body_bytes: options.max_body_bytes,
        running: Arc::clone(&running),
        completed: Arc::clone(&completed),
        stop_requested: Arc::clone(&stop_requested),
    };
    let worker = thread::Builder::new()
        .name("qiongli-config-wizard".to_string())
        .spawn(move || serve_wizard(listener, worker_state))
        .map_err(WizardError::Spawn)?;

    Ok(ConfigWizard {
        host,
        port,
        url,
        config_path,
        provider,
        running,
        completed,
        stop_requested,
        _worker: worker,
    })
}

struct WizardWorkerState {
    token: String,
    selected_provider: Option<String>,
    config_path: PathBuf,
    ttl: Duration,
    max_body_bytes: usize,
    running: Arc<AtomicBool>,
    completed: Arc<AtomicBool>,
    stop_requested: Arc<AtomicBool>,
}

fn serve_wizard(listener: TcpListener, state: WizardWorkerState) {
    let expires_at = Instant::now() + state.ttl;
    let mut stop_at = expires_at;
    let mut saved = false;

    while !state.stop_requested.load(Ordering::Acquire) && Instant::now() < stop_at {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                match handle_connection(
                    &mut stream,
                    &state.token,
                    state.selected_provider.as_deref(),
                    &state.config_path,
                    state.max_body_bytes,
                    saved,
                ) {
                    ConnectionOutcome::Saved => {
                        saved = true;
                        state.completed.store(true, Ordering::Release);
                        stop_at = stop_at.min(Instant::now() + SAVE_REDIRECT_GRACE);
                    }
                    ConnectionOutcome::SavedPage => break,
                    ConnectionOutcome::Continue => {}
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(ACCEPT_POLL_INTERVAL);
            }
            Err(_) => break,
        }
    }

    state.running.store(false, Ordering::Release);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionOutcome {
    Continue,
    Saved,
    SavedPage,
}

fn handle_connection(
    stream: &mut TcpStream,
    token: &str,
    selected_provider: Option<&str>,
    config_path: &std::path::Path,
    max_body_bytes: usize,
    already_saved: bool,
) -> ConnectionOutcome {
    let _ = stream.set_read_timeout(Some(CONNECTION_TIMEOUT));
    let _ = stream.set_write_timeout(Some(CONNECTION_TIMEOUT));

    let request = match read_request(stream, max_body_bytes) {
        Ok(request) => request,
        Err(RequestError::TooLarge) => {
            let _ =
                write_text_response(stream, 413, "Payload Too Large", "Request too large.", &[]);
            return ConnectionOutcome::Continue;
        }
        Err(RequestError::BadRequest | RequestError::Io) => {
            let _ = write_text_response(stream, 400, "Bad Request", "Invalid request.", &[]);
            return ConnectionOutcome::Continue;
        }
    };

    let (path, query) = split_target(&request.target);
    if query_token(query).as_deref() != Some(token) {
        let _ = write_text_response(stream, 403, "Forbidden", "Forbidden.", &[]);
        return ConnectionOutcome::Continue;
    }

    match (request.method.as_str(), path) {
        ("GET", "/") if already_saved => {
            let _ = write_html_response(stream, 200, "OK", &render_saved_page());
            ConnectionOutcome::SavedPage
        }
        ("GET", "/") => {
            let _ = write_html_response(stream, 200, "OK", &render_form(token, selected_provider));
            ConnectionOutcome::Continue
        }
        ("GET", "/saved") if already_saved => {
            let _ = write_html_response(stream, 200, "OK", &render_saved_page());
            ConnectionOutcome::SavedPage
        }
        ("POST", "/save") if already_saved => {
            let _ = write_text_response(stream, 410, "Gone", "Setup already completed.", &[]);
            ConnectionOutcome::Continue
        }
        ("POST", "/save") => match save_form(&request.body, selected_provider, config_path) {
            Ok(()) => {
                let location = format!("/saved?token={token}");
                let _ = write_text_response(
                    stream,
                    303,
                    "See Other",
                    "Configuration saved.",
                    &[("Location", location.as_str())],
                );
                ConnectionOutcome::Saved
            }
            Err(SaveFormError::InvalidInput) => {
                let _ = write_text_response(
                    stream,
                    400,
                    "Bad Request",
                    "No supported configuration values were provided.",
                    &[],
                );
                ConnectionOutcome::Continue
            }
            Err(SaveFormError::WriteFailed) => {
                let _ = write_text_response(
                    stream,
                    500,
                    "Internal Server Error",
                    "Unable to save configuration.",
                    &[],
                );
                ConnectionOutcome::Continue
            }
        },
        _ => {
            let _ = write_text_response(stream, 404, "Not Found", "Not found.", &[]);
            ConnectionOutcome::Continue
        }
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    target: String,
    body: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
enum RequestError {
    TooLarge,
    BadRequest,
    Io,
}

fn read_request(
    stream: &mut TcpStream,
    max_body_bytes: usize,
) -> Result<HttpRequest, RequestError> {
    let mut buffer = Vec::with_capacity(1024);
    let header_end = loop {
        if let Some(position) = find_bytes(&buffer, b"\r\n\r\n") {
            break position + 4;
        }
        if buffer.len() >= MAX_HEADER_BYTES {
            return Err(RequestError::TooLarge);
        }
        let mut chunk = [0_u8; 1024];
        let count = stream.read(&mut chunk).map_err(|_| RequestError::Io)?;
        if count == 0 {
            return Err(RequestError::BadRequest);
        }
        buffer.extend_from_slice(&chunk[..count]);
    };

    let (method, target, content_length) = {
        let header =
            std::str::from_utf8(&buffer[..header_end]).map_err(|_| RequestError::BadRequest)?;
        let mut lines = header.split("\r\n");
        let request_line = lines.next().ok_or(RequestError::BadRequest)?;
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts
            .next()
            .filter(|value| *value == "GET" || *value == "POST")
            .ok_or(RequestError::BadRequest)?;
        let target = request_parts
            .next()
            .filter(|value| value.starts_with('/'))
            .ok_or(RequestError::BadRequest)?;
        let version = request_parts.next().ok_or(RequestError::BadRequest)?;
        if !version.starts_with("HTTP/1.") || request_parts.next().is_some() {
            return Err(RequestError::BadRequest);
        }

        let mut content_length = 0_usize;
        for line in lines.filter(|line| !line.is_empty()) {
            let Some((name, value)) = line.split_once(':') else {
                return Err(RequestError::BadRequest);
            };
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| RequestError::BadRequest)?;
            }
        }
        (method.to_string(), target.to_string(), content_length)
    };
    if content_length > max_body_bytes {
        return Err(RequestError::TooLarge);
    }

    let expected_length = header_end + content_length;
    while buffer.len() < expected_length {
        let remaining = expected_length - buffer.len();
        let mut chunk = [0_u8; 1024];
        let read_length = remaining.min(chunk.len());
        let count = stream
            .read(&mut chunk[..read_length])
            .map_err(|_| RequestError::Io)?;
        if count == 0 {
            return Err(RequestError::BadRequest);
        }
        buffer.extend_from_slice(&chunk[..count]);
    }

    Ok(HttpRequest {
        method,
        target,
        body: buffer[header_end..expected_length].to_vec(),
    })
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn split_target(target: &str) -> (&str, &str) {
    target
        .split_once('?')
        .map_or((target, ""), |(path, query)| (path, query))
}

fn query_token(query: &str) -> Option<String> {
    form_urlencoded::parse(query.as_bytes()).find_map(|(key, value)| {
        if key == "token" {
            Some(value.into_owned())
        } else {
            None
        }
    })
}

#[derive(Debug, Clone, Copy)]
enum SaveFormError {
    InvalidInput,
    WriteFailed,
}

fn save_form(
    body: &[u8],
    selected_provider: Option<&str>,
    config_path: &std::path::Path,
) -> Result<(), SaveFormError> {
    let mut values = Vec::new();
    for (raw_key, raw_value) in form_urlencoded::parse(body) {
        let Some((provider, field)) = raw_key.split_once('.') else {
            return Err(SaveFormError::InvalidInput);
        };
        let provider =
            normalize_provider(Some(provider)).map_err(|_| SaveFormError::InvalidInput)?;
        let Some(provider) = provider else {
            return Err(SaveFormError::InvalidInput);
        };
        let field = normalize_key(field);
        if !is_wizard_field(&provider, &field)
            || selected_provider.is_some_and(|selected| selected != provider)
        {
            return Err(SaveFormError::InvalidInput);
        }
        let value = raw_value.trim();
        if value.is_empty() {
            continue;
        }
        if value.len() > MAX_FIELD_BYTES {
            return Err(SaveFormError::InvalidInput);
        }
        values.push((provider, field, value.to_string()));
    }
    if values.is_empty() {
        return Err(SaveFormError::InvalidInput);
    }

    for (provider, field, value) in values {
        save_provider_value_at(config_path, &provider, &field, &value)
            .map_err(|_| SaveFormError::WriteFailed)?;
    }
    Ok(())
}

fn normalize_host(raw: &str) -> Result<String, WizardError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "127.0.0.1" | "localhost" => Ok("127.0.0.1".to_string()),
        _ => Err(WizardError::NonLoopbackHost),
    }
}

fn normalize_provider(raw: Option<&str>) -> Result<Option<String>, WizardError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let normalized = normalize_key(raw);
    let normalized = match normalized.as_str() {
        "s2" | "semanticscholar" => "semantic_scholar".to_string(),
        _ => normalized,
    };
    match normalized.as_str() {
        "openalex" | "semantic_scholar" | "crossref" | "pubmed" => Ok(Some(normalized)),
        _ => Err(WizardError::UnsupportedProvider(raw.to_string())),
    }
}

fn is_wizard_field(provider: &str, field: &str) -> bool {
    matches!(
        (provider, field),
        ("openalex", "api_key")
            | ("openalex", "email")
            | ("semantic_scholar", "api_key")
            | ("crossref", "email")
            | ("pubmed", "api_key")
    )
}

fn render_form(token: &str, selected_provider: Option<&str>) -> String {
    let mut inputs = String::new();
    for (provider, field, label) in wizard_fields() {
        if selected_provider.is_some_and(|selected| selected != provider) {
            continue;
        }
        let input_type = if field == "email" {
            "email"
        } else {
            "password"
        };
        inputs.push_str(&format!(
            "<label>{label}<input type=\"{input_type}\" name=\"{provider}.{field}\" autocomplete=\"new-password\"></label>"
        ));
    }

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><meta name=\"referrer\" content=\"no-referrer\"><title>Qiongli Provider Setup</title></head><body><main><h1>Qiongli Provider Setup</h1><p>Credentials are stored only in your local Qiongli configuration.</p><form method=\"post\" action=\"/save?token={token}\">{inputs}<button type=\"submit\">Save locally</button></form></main></body></html>"
    )
}

fn render_saved_page() -> String {
    "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"referrer\" content=\"no-referrer\"><title>Qiongli Provider Setup</title></head><body><main><h1>Configuration saved</h1><p>You can close this page.</p></main></body></html>".to_string()
}

fn wizard_fields() -> [(&'static str, &'static str, &'static str); 5] {
    [
        ("openalex", "api_key", "OpenAlex API key"),
        ("openalex", "email", "OpenAlex contact email (optional)"),
        ("semantic_scholar", "api_key", "Semantic Scholar API key"),
        ("crossref", "email", "Crossref contact email"),
        ("pubmed", "api_key", "NCBI API key"),
    ]
}

fn write_html_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    body: &str,
) -> io::Result<()> {
    write_response(
        stream,
        status,
        reason,
        "text/html; charset=utf-8",
        body,
        &[],
    )
}

fn write_text_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> io::Result<()> {
    write_response(
        stream,
        status,
        reason,
        "text/plain; charset=utf-8",
        body,
        headers,
    )
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n",
        body.len()
    );
    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    response.push_str(body);
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn secure_token() -> io::Result<String> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    fill_random(&mut bytes)?;
    let mut token = String::with_capacity(TOKEN_BYTES * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut token, "{byte:02x}").expect("writing to String cannot fail");
    }
    Ok(token)
}

#[cfg(unix)]
fn fill_random(bytes: &mut [u8]) -> io::Result<()> {
    File::open("/dev/urandom")?.read_exact(bytes)
}

#[cfg(windows)]
fn fill_random(bytes: &mut [u8]) -> io::Result<()> {
    use std::ffi::c_void;

    #[link(name = "advapi32")]
    extern "system" {
        #[link_name = "SystemFunction036"]
        fn rtl_gen_random(buffer: *mut c_void, length: u32) -> u8;
    }

    let length = u32::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "random request too large"))?;
    // SAFETY: `bytes` is valid for `length` writable bytes for the duration of the call.
    let success = unsafe { rtl_gen_random(bytes.as_mut_ptr().cast(), length) };
    if success == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(any(unix, windows)))]
fn fill_random(_bytes: &mut [u8]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "secure system random source unavailable",
    ))
}
