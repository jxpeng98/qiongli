//! One local provider setup page for CLI and native MCP. Secrets never enter
//! command arguments, MCP responses, ordinary config or request diagnostics.
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use qiongli_config::{SecretStore, SecretStoreStatus};
use qiongli_content::EmbeddedContent;
use qiongli_runtime::mcp::{ProviderConfiguration, ProviderConfigurationSnapshot};
use qiongli_runtime::providers::{PROVIDER_ORDER, ProviderAccess, ProviderId};
use serde_json::{Value, json};
use url::form_urlencoded;
use zeroize::Zeroizing;

use crate::command::{CliOutput, CommandEnvironment, config_store};

pub(crate) const USAGE: &str = "Literature provider configuration\n\nUsage:\n  qiongli provider setup [openalex|semantic_scholar|crossref|pubmed|arxiv] [--no-open]\n  qiongli provider status\n  qiongli provider doctor\n  qiongli provider test <provider> --confirm-network-request\n\nSetup opens a local page for explicit confirmation. --no-open prints the link\nwithout opening a browser; keep this process running while using the page.\nThe link expires after ten minutes or one successful save. Empty key fields\nkeep the stored key. Remove key explicitly disables that provider.\nKeys use the existing OS credential store (currently macOS Keychain). Other\nplatforms report unavailable storage; public settings remain configurable.\nStatus/doctor are offline. Test sends one bounded query only to the selected\nprovider; it never returns papers or credentials. Models are managed by the Host.\n";
const TTL: Duration = Duration::from_secs(600);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_HEADER: usize = 8192;
const MAX_BODY: usize = 16 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub struct ProviderSetupOptions {
    pub(crate) provider: Option<ProviderId>,
    pub(crate) no_open: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ProviderCommand {
    Help,
    Setup(ProviderSetupOptions),
    Status,
    Test(ProviderId),
}

pub(crate) fn parse(args: &[OsString]) -> Result<ProviderCommand, &'static str> {
    let words = args
        .iter()
        .map(|arg| arg.to_str().ok_or("provider argument must be UTF-8"))
        .collect::<Result<Vec<_>, _>>()?;
    match words.as_slice() {
        [] | ["status" | "doctor"] => Ok(ProviderCommand::Status),
        ["--help"] => Ok(ProviderCommand::Help),
        ["test", provider, "--confirm-network-request"] => {
            Ok(ProviderCommand::Test(parse_provider(provider)?))
        }
        ["test", ..] => Err("test requires one provider and --confirm-network-request"),
        ["setup", options @ ..] => {
            let mut provider = None;
            let mut no_open = false;
            for option in options {
                if *option == "--no-open" && !no_open {
                    no_open = true;
                } else if provider.is_none() && !option.starts_with('-') {
                    provider = Some(parse_provider(option)?);
                } else {
                    return Err("unsupported or duplicate provider setup option");
                }
            }
            Ok(ProviderCommand::Setup(ProviderSetupOptions {
                provider,
                no_open,
            }))
        }
        _ => Err("unknown provider command"),
    }
}

fn parse_provider(name: &str) -> Result<ProviderId, &'static str> {
    ProviderId::parse(name).map_err(|_| "unsupported provider")
}

pub(crate) struct ProviderSetupChange {
    pub provider: ProviderId,
    pub expected_revision: u64,
    pub enabled: bool,
    pub key: Option<Zeroizing<String>>,
    pub remove_key: bool,
    pub email: Option<String>,
}

#[derive(Clone)]
struct SetupContext {
    environment: CommandEnvironment,
    content_sha256: String,
    secrets: Arc<dyn SecretStore>,
}

pub(crate) struct NativeProviderConfiguration {
    context: SetupContext,
    wizard: Mutex<Option<SetupWizard>>,
}

impl NativeProviderConfiguration {
    pub(crate) fn new(environment: &CommandEnvironment, content: &EmbeddedContent) -> Self {
        Self::with_store(
            environment,
            content,
            crate::credential_store::native_secret_store(),
        )
    }

    fn with_store(
        environment: &CommandEnvironment,
        content: &EmbeddedContent,
        secrets: Arc<dyn SecretStore>,
    ) -> Self {
        Self {
            context: SetupContext {
                environment: environment.clone(),
                content_sha256: content.pack().pack_sha256().to_owned(),
                secrets,
            },
            wizard: Mutex::new(None),
        }
    }
}

impl ProviderConfiguration for NativeProviderConfiguration {
    fn snapshot(&self) -> Result<ProviderConfigurationSnapshot, &'static str> {
        let loaded = config_store(&self.context.environment)
            .and_then(|store| store.load())
            .map_err(|error| error.reason_code())?;
        let metadata = ProviderAccess::from_global_settings_metadata(&loaded.settings);
        let secrets = Arc::clone(&self.context.secrets);
        Ok(ProviderConfigurationSnapshot {
            revision: loaded.revision,
            metadata,
            loader: Arc::new(move || {
                ProviderAccess::from_global_settings(&loaded.settings, secrets.as_ref())
            }),
        })
    }

    fn configure(&self, provider: Option<ProviderId>, port: u16) -> Result<Value, &'static str> {
        self.snapshot()?;
        let mut slot = self
            .wizard
            .lock()
            .map_err(|_| "provider-setup-unavailable")?;
        if let Some(wizard) = slot.as_ref().filter(|wizard| wizard.is_running()) {
            if wizard.provider != provider || (port != 0 && port != wizard.port) {
                return Err("provider-setup-already-running");
            }
            return Ok(wizard.result("already_running"));
        }
        let wizard = SetupWizard::start(self.context.clone(), provider, port, TTL)?;
        let result = wizard.result("ready");
        *slot = Some(wizard);
        Ok(result)
    }
}

pub(crate) fn status(environment: &CommandEnvironment) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let secrets = crate::credential_store::native_secret_store();
    let access = ProviderAccess::from_global_settings_metadata(&loaded.settings);
    CliOutput::success_text(json!({"command":"provider-status", "revision":loaded.revision,
        "providers":access.status(), "secure_store":if secrets.status() == SecretStoreStatus::Available { "available" } else { "unavailable" },
        "credential_access":"not-tested", "online_connection":"not-tested",
        "next_step":"qiongli provider setup"}).to_string()+"\n")
}

pub(crate) fn test(environment: &CommandEnvironment, provider: ProviderId) -> CliOutput {
    // Reuse the same bounded loader/search execution path as native MCP. The
    // parser has already required explicit approval of this online request.
    let content = match crate::embedded_content() {
        Ok(content) => content,
        Err(_) => return crate::failed_embedded_content_output(),
    };
    let server = match crate::mcp::full_server(environment, &content) {
        Ok(server) => server,
        Err(_) => return CliOutput::operation_failure("provider-runtime-unavailable"),
    };
    test_with_server(&server, provider)
}

fn test_with_server(server: &crate::mcp::FullMcpServer, provider: ProviderId) -> CliOutput {
    let reply = server.handle(json!({"jsonrpc":"2.0","id":1,"method":"tools/call", "params":{
        "name":"qiongli_literature_search", "arguments":{"query":"research", "providers":[provider.as_str()],"limit":1,"per_provider_limit":1,"total_limit":1}}}));
    let Some(reply) = reply else {
        return CliOutput::operation_failure("provider-test-unavailable");
    };
    test_result(provider, &reply)
}

fn test_result(provider: ProviderId, reply: &Value) -> CliOutput {
    let result = &reply["result"];
    if result["isError"] == true || reply.get("error").is_some() {
        let code = match result["structuredContent"]["reason_code"].as_str() {
            Some("provider-credentials-unavailable") => "provider-credentials-unavailable",
            Some("native-config-unavailable") => "native-config-unavailable",
            Some("provider-runtime-unavailable") => "provider-runtime-unavailable",
            _ => "provider-test-failed",
        };
        return CliOutput::operation_failure(code);
    }
    let content = &result["structuredContent"];
    // A successful request with zero matches still verifies connectivity.
    let success = content["diagnostics"]["providers"][provider.as_str()]["status"] == "ok";
    let reason = match content["diagnostics"]["providers"][provider.as_str()]["error_kind"].as_str()
    {
        Some("timeout") => "timeout",
        Some("http_error") => "http_error",
        Some("decode_error") => "decode_error",
        Some("cancelled") => "cancelled",
        Some("transport_error") => "transport_error",
        _ if success => "request-succeeded",
        _ => "provider-not-ready",
    };
    let output = json!({"command":"provider-test", "provider":provider.as_str(), "reason_code":reason,
        "status":if success { "connected" } else { "failed" }, "online_connection":if success { "verified" } else { "failed" },
        "note":"A successful bounded request verifies this connection only; no paper content is returned."});
    CliOutput::success_text(output.to_string() + "\n").with_exit_code(if success { 0 } else { 1 })
}

impl ProviderSetupOptions {
    pub fn run(&self, environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
        let backend = NativeProviderConfiguration::new(environment, content);
        let result = match backend.configure(self.provider, 0) {
            Ok(result) => result,
            Err(code) => return CliOutput::operation_failure(code),
        };
        let url = result["url"].as_str().unwrap_or_default();
        println!(
            "Configure literature providers: {url}\nKeep this process running. The link expires in ten minutes.\nConfirm changes on the page; no key is needed in this terminal."
        );
        if !self.no_open && open::that(url).is_err() {
            eprintln!("Browser could not open; use the displayed local link.");
        }
        loop {
            let Ok(slot) = backend.wizard.lock() else {
                return CliOutput::operation_failure("provider-setup-unavailable");
            };
            let Some(wizard) = slot.as_ref() else {
                return CliOutput::operation_failure("provider-setup-unavailable");
            };
            if !wizard.is_running() {
                return if wizard.saved.load(Ordering::Acquire) {
                    CliOutput::success_text(
                        "Provider configuration saved. Run `qiongli provider status`; use `provider test <provider> --confirm-network-request` for an online check.\n",
                    )
                } else {
                    CliOutput::operation_failure("provider-setup-expired")
                };
            }
            drop(slot);
            thread::sleep(Duration::from_millis(100));
        }
    }
}

struct SetupWizard {
    url: String,
    port: u16,
    provider: Option<ProviderId>,
    running: Arc<AtomicBool>,
    saved: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl SetupWizard {
    fn start(
        context: SetupContext,
        provider: Option<ProviderId>,
        port: u16,
        ttl: Duration,
    ) -> Result<Self, &'static str> {
        let listener =
            TcpListener::bind(("127.0.0.1", port)).map_err(|_| "provider-setup-bind-failed")?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "provider-setup-listener-failed")?;
        let port = listener
            .local_addr()
            .map_err(|_| "provider-setup-listener-failed")?
            .port();
        let mut random = [0u8; 24];
        getrandom::fill(&mut random).map_err(|_| "provider-setup-token-unavailable")?;
        let token = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let url = format!("http://127.0.0.1:{port}/?token={token}");
        let running = Arc::new(AtomicBool::new(true));
        let saved = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let state = (Arc::clone(&running), Arc::clone(&saved), Arc::clone(&stop));
        let worker = thread::Builder::new()
            .name("qiongli-provider-setup".to_owned())
            .spawn(move || {
                let deadline = Instant::now() + ttl;
                while Instant::now() < deadline && !state.2.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((mut stream, peer)) if peer.ip().is_loopback() => {
                            if handle_connection(
                                &mut stream,
                                &context,
                                provider,
                                port,
                                &token,
                                deadline,
                            ) {
                                state.1.store(true, Ordering::Release);
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(25))
                        }
                        Err(_) => break,
                    }
                }
                state.0.store(false, Ordering::Release);
            })
            .map_err(|_| "provider-setup-worker-failed")?;
        Ok(Self {
            url,
            port,
            provider,
            running,
            saved,
            stop,
            worker: Some(worker),
        })
    }
    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
    fn result(&self, status: &str) -> Value {
        let mut value = json!({"status":status,"url":self.url,"host":"127.0.0.1","port":self.port,"config_path":"<managed-native-config>"});
        if let Some(provider) = self.provider {
            value["provider"] = json!(provider.as_str());
        }
        value
    }
}

impl Drop for SetupWizard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct HttpRequest {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Zeroizing<Vec<u8>>,
}

fn read_request(stream: &mut TcpStream, expires: Instant) -> Result<HttpRequest, &'static str> {
    let deadline = expires.min(Instant::now() + REQUEST_TIMEOUT);
    stream
        .set_write_timeout(Some(REQUEST_TIMEOUT))
        .map_err(|_| "request-timeout")?;
    let mut bytes = Zeroizing::new(Vec::new());
    while !bytes.ends_with(b"\r\n\r\n") {
        if bytes.len() >= MAX_HEADER {
            return Err("request-too-large");
        }
        let mut byte = [0];
        read_before(stream, &mut byte, deadline)?;
        bytes.push(byte[0]);
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| "invalid-request")?;
    let mut lines = text.split("\r\n");
    let first = lines
        .next()
        .ok_or("invalid-request")?
        .split(' ')
        .collect::<Vec<_>>();
    if first.len() != 3 || first[2] != "HTTP/1.1" || !matches!(first[0], "GET" | "POST") {
        return Err("invalid-request");
    }
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or("invalid-request")?;
        let name = name.to_ascii_lowercase();
        if !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || headers.insert(name, value.trim().to_owned()).is_some()
        {
            return Err("invalid-request");
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err("invalid-request");
    }
    let length = match headers.get("content-length") {
        Some(length) => length.parse::<usize>().map_err(|_| "invalid-request")?,
        None if first[0] == "GET" => 0,
        None => return Err("invalid-request"),
    };
    if length > MAX_BODY || (first[0] == "GET" && length != 0) {
        return Err("request-too-large");
    }
    let mut body = Zeroizing::new(vec![0; length]);
    read_before(stream, &mut body, deadline)?;
    Ok(HttpRequest {
        method: first[0].to_owned(),
        target: first[1].to_owned(),
        headers,
        body,
    })
}

fn read_before(
    stream: &mut TcpStream,
    mut bytes: &mut [u8],
    deadline: Instant,
) -> Result<(), &'static str> {
    while !bytes.is_empty() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or("request-timeout")?;
        if remaining.is_zero() {
            return Err("request-timeout");
        }
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|_| "request-timeout")?;
        match stream.read(bytes) {
            Ok(0) => return Err("invalid-request"),
            Ok(count) => bytes = &mut bytes[count..],
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err("request-timeout"),
        }
    }
    Ok(())
}

fn fields(body: &[u8]) -> Result<BTreeMap<String, Zeroizing<String>>, &'static str> {
    std::str::from_utf8(body).map_err(|_| "invalid-form")?;
    for (index, byte) in body.iter().enumerate() {
        if *byte == b'%'
            && (body
                .get(index + 1)
                .is_none_or(|byte| !byte.is_ascii_hexdigit())
                || body
                    .get(index + 2)
                    .is_none_or(|byte| !byte.is_ascii_hexdigit()))
        {
            return Err("invalid-form");
        }
    }
    let mut fields = BTreeMap::new();
    for (key, value) in form_urlencoded::parse(body) {
        if key.contains('\u{fffd}') || value.contains('\u{fffd}') {
            return Err("invalid-form");
        }
        if fields
            .insert(key.into_owned(), Zeroizing::new(value.into_owned()))
            .is_some()
        {
            return Err("duplicate-field");
        }
    }
    Ok(fields)
}

fn handle_connection(
    stream: &mut TcpStream,
    context: &SetupContext,
    selected: Option<ProviderId>,
    port: u16,
    token: &str,
    deadline: Instant,
) -> bool {
    let outcome = (|| {
        let request = read_request(stream, deadline)?;
        let authority = format!("127.0.0.1:{port}");
        if request.headers.get("host") != Some(&authority) {
            return Err("invalid-host");
        }
        if request
            .headers
            .get("origin")
            .is_some_and(|origin| origin != &format!("http://{authority}"))
            || request
                .headers
                .get("sec-fetch-site")
                .is_some_and(|site| !matches!(site.as_str(), "same-origin" | "none"))
        {
            return Err("invalid-origin");
        }
        if Instant::now() >= deadline {
            return Err("provider-setup-expired");
        }
        let (path, query) = request.target.split_once('?').ok_or("invalid-token")?;
        let query = fields(query.as_bytes())?;
        if query.get("token").map(|value| value.as_str()) != Some(token) {
            return Err("invalid-token");
        }
        if query
            .keys()
            .any(|key| !matches!(key.as_str(), "token" | "provider"))
        {
            return Err("invalid-form");
        }
        if request.method == "GET" && path == "/" {
            let requested = query
                .get("provider")
                .map(|name| parse_provider(name))
                .transpose()?;
            if selected.is_some() && requested.is_some() && selected != requested {
                return Err("unsupported-provider");
            }
            let body = render_page(context, selected.or(requested), token)?;
            respond(stream, "200 OK", &body);
            return Ok(false);
        }
        if request.method != "POST" || path != "/save" || query.contains_key("provider") {
            return Err("invalid-request");
        }
        if request
            .headers
            .get("content-type")
            .is_none_or(|content_type| content_type != "application/x-www-form-urlencoded")
        {
            return Err("invalid-form");
        }
        let fields = fields(&request.body)?;
        let change = parse_change(&fields, selected)?;
        let content =
            crate::embedded_content().map_err(|_| "provider-setup-content-unavailable")?;
        if content.pack().pack_sha256() != context.content_sha256 {
            return Err("provider-setup-content-identity-mismatch");
        }
        let code = crate::desktop::apply_provider_setup(
            &context.environment,
            &content,
            Arc::clone(&context.secrets),
            change,
        )?;
        let note = if code.ends_with("cleanup-required") {
            "Saved, but credential/config cleanup needs attention. The previous credential may still exist in the OS credential manager. Keep this warning when requesting support."
        } else {
            "Saved. Close this page and run qiongli provider status. Use an explicitly approved provider test for an online check."
        };
        respond(
            stream,
            "200 OK",
            &format!(
                "<!doctype html><meta charset=utf-8><title>Qiongli</title><h1>Provider configuration saved</h1><p>{note}</p>"
            ),
        );
        Ok(true)
    })();
    match outcome {
        Ok(saved) => saved,
        Err(code) => {
            respond(
                stream,
                "400 Bad Request",
                &format!(
                    "<!doctype html><meta charset=utf-8><title>Qiongli</title><h1>Configuration was not saved</h1><p>{code}</p><p>Return to the setup link to reload the current settings and try again.</p>"
                ),
            );
            false
        }
    }
}

fn parse_change(
    fields: &BTreeMap<String, Zeroizing<String>>,
    selected: Option<ProviderId>,
) -> Result<ProviderSetupChange, &'static str> {
    if fields.keys().any(|key| {
        ![
            "provider",
            "revision",
            "enabled",
            "api_key",
            "remove_key",
            "email",
            "clear_email",
            "approve",
        ]
        .contains(&key.as_str())
    }) {
        return Err("unsupported-field");
    }
    let text = |key: &str| fields.get(key).map(|value| value.as_str()).unwrap_or("");
    if text("approve") != "yes" {
        return Err("provider-approval-required");
    }
    for key in ["enabled", "remove_key", "clear_email"] {
        if fields.contains_key(key) && text(key) != "yes" {
            return Err("invalid-form");
        }
    }
    let provider = parse_provider(text("provider"))?;
    if selected.is_some_and(|selected| selected != provider) {
        return Err("unsupported-provider");
    }
    let expected_revision = text("revision").parse().map_err(|_| "invalid-revision")?;
    let key = (!text("api_key").trim().is_empty())
        .then(|| Zeroizing::new(text("api_key").trim().to_owned()));
    if key
        .as_ref()
        .is_some_and(|key| key.len() > 4096 || key.chars().any(char::is_control))
    {
        return Err("provider-secret-invalid");
    }
    let email = if text("clear_email") == "yes" {
        if !text("email").trim().is_empty() {
            return Err("provider-email-change-conflict");
        }
        Some(String::new())
    } else {
        (!text("email").trim().is_empty()).then(|| text("email").trim().to_owned())
    };
    Ok(ProviderSetupChange {
        provider,
        expected_revision,
        enabled: text("enabled") == "yes",
        key,
        remove_key: text("remove_key") == "yes",
        email,
    })
}

fn render_page(
    context: &SetupContext,
    selected: Option<ProviderId>,
    token: &str,
) -> Result<String, &'static str> {
    let loaded = config_store(&context.environment)
        .and_then(|store| store.load())
        .map_err(|error| error.reason_code())?;
    let access = ProviderAccess::from_global_settings_metadata(&loaded.settings);
    let mut body = "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content=\"width=device-width\"><title>Qiongli provider setup</title><style>body{font:16px system-ui;max-width:660px;margin:40px auto;padding:0 20px;color:#202124}label{display:block;margin:20px 0}input[type=password],input[type=email]{display:block;width:95%;padding:10px;margin-top:8px}button{padding:10px 22px}li{margin:16px 0}</style><h1>Literature provider setup / 文献服务设置</h1>".to_owned();
    let Some(provider) = selected else {
        body.push_str(
            "<p>Select one provider to configure. Only that provider will change.</p><ul>",
        );
        for provider in PROVIDER_ORDER {
            body.push_str(&format!(
                "<li><a href=\"/?token={token}&amp;provider={}\">{}</a></li>",
                provider.as_str(),
                provider.as_str()
            ));
        }
        body.push_str("</ul></html>");
        return Ok(body);
    };
    let enabled = access.is_enabled(provider);
    body.push_str(&format!("<h2>{}</h2><p>Only this service changes. Saved settings do not prove an online connection.</p><form method=post action=\"/save?token={token}\"><input type=hidden name=provider value=\"{}\"><input type=hidden name=revision value=\"{}\"><label><input type=checkbox name=enabled value=yes {}> Enable this service / 启用</label>",provider.as_str(),provider.as_str(),loaded.revision,if enabled { "checked" } else { "" }));
    if matches!(
        provider,
        ProviderId::OpenAlex | ProviderId::SemanticScholar | ProviderId::PubMed
    ) {
        if context.secrets.status() == SecretStoreStatus::Available {
            let configured = access
                .is_field_configured(provider, qiongli_runtime::providers::ProviderField::ApiKey);
            body.push_str(&format!("<p>Key: {}. It is stored in the OS credential store.</p><label>API key <input type=password name=api_key maxlength=4096 autocomplete=new-password placeholder=\"Leave empty to keep the existing key\"></label><label><input type=checkbox name=remove_key value=yes> Remove stored key and disable this service / 删除密钥并停用</label>",if configured { "configured" } else { "missing" }));
        } else {
            body.push_str("<p>Secure credential storage is unavailable on this platform. Saving a new key requires macOS Keychain in this version. Existing keys will be retained; no plaintext fallback is used.</p>");
        }
    }
    if matches!(provider, ProviderId::OpenAlex | ProviderId::Crossref) {
        body.push_str("<label>Contact email <input type=email name=email maxlength=254 placeholder=\"Leave empty to keep the existing email\"></label><label><input type=checkbox name=clear_email value=yes> Remove stored email / 删除联系邮箱</label>");
    }
    let help = match provider {
        ProviderId::OpenAlex => "https://openalex.org/settings/api",
        ProviderId::SemanticScholar => "https://www.semanticscholar.org/product/api",
        ProviderId::PubMed => "https://www.ncbi.nlm.nih.gov/account/settings/",
        ProviderId::Crossref => {
            "https://www.crossref.org/documentation/retrieve-metadata/rest-api/"
        }
        ProviderId::Arxiv => "https://info.arxiv.org/help/api/index.html",
    };
    body.push_str(&format!("<p><a href=\"{help}\" target=_blank rel=\"noreferrer noopener\">Provider account / API information</a></p><label><input type=checkbox name=approve value=yes required> I approve updating this service's settings and any selected credential change. / 确认保存上述修改</label><button type=submit>Save / 保存</button></form><p>This link expires in ten minutes or after saving. Never paste keys into your AI conversation.</p></html>"));
    Ok(body)
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'\r\n\r\n",
        body.len()
    );
    let _ = stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.write_all(body.as_bytes()));
}

#[cfg(test)]
mod tests;
