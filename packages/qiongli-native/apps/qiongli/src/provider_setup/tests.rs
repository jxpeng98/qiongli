use super::*;
use qiongli_config::{GlobalSettingsStore, SecretRef, SecretStoreError, SecretValue};
use std::fs;
use std::sync::atomic::AtomicUsize;

struct Fixture {
    root: std::path::PathBuf,
    env: CommandEnvironment,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "provider-setup-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let env = CommandEnvironment::with_paths(
            Some(root.join("config").into_os_string()),
            Some(root.clone()),
            None,
        );
        Self { root, env }
    }
    fn store(&self) -> GlobalSettingsStore {
        config_store(&self.env).unwrap()
    }
    fn context(&self, secrets: Arc<dyn SecretStore>) -> SetupContext {
        SetupContext {
            environment: self.env.clone(),
            content_sha256: crate::embedded_content()
                .unwrap()
                .pack()
                .pack_sha256()
                .to_owned(),
            secrets,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
#[derive(Default)]
struct Secrets {
    values: Mutex<BTreeMap<String, Vec<u8>>>,
    reads: AtomicUsize,
    unavailable: bool,
    bad_readback: bool,
    fail_write: bool,
    skip_write: bool,
    fail_remove: Option<String>,
    race: Option<GlobalSettingsStore>,
}
impl SecretStore for Secrets {
    fn status(&self) -> SecretStoreStatus {
        if self.unavailable {
            SecretStoreStatus::Unavailable
        } else {
            SecretStoreStatus::Available
        }
    }
    fn resolve(&self, r: &SecretRef) -> Result<SecretValue, SecretStoreError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let values = self.values.lock().unwrap();
        let bytes = values
            .get(r.storage_key())
            .ok_or(SecretStoreError::NotFound)?;
        SecretValue::new(if self.bad_readback {
            b"WRONG".to_vec()
        } else {
            bytes.clone()
        })
        .map_err(|_| SecretStoreError::PersistenceFailed)
    }
    fn store(&self, r: &SecretRef, v: &SecretValue) -> Result<(), SecretStoreError> {
        if !self.skip_write {
            self.values
                .lock()
                .unwrap()
                .insert(r.storage_key().into(), v.as_bytes().to_vec());
        }
        if let Some(store) = &self.race {
            let mut loaded = store.load().unwrap();
            loaded.settings.providers.arxiv.enabled = false;
            store.replace(loaded.revision, loaded.settings).unwrap();
        }
        if self.fail_write {
            Err(SecretStoreError::PersistenceFailed)
        } else {
            Ok(())
        }
    }
    fn remove(&self, r: &SecretRef) -> Result<(), SecretStoreError> {
        if self.fail_remove.as_deref() == Some(r.storage_key()) {
            return Err(SecretStoreError::PersistenceFailed);
        }
        self.values
            .lock()
            .unwrap()
            .remove(r.storage_key())
            .map(|_| ())
            .ok_or(SecretStoreError::NotFound)
    }
}
fn request(w: &SetupWizard, method: &str, target: &str, headers: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", w.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let message = format!(
        "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n{headers}Content-Length: {}\r\n\r\n{body}",
        w.port,
        body.len()
    );
    stream.write_all(message.as_bytes()).unwrap();
    let mut response = String::new();
    if let Err(error) = stream.read_to_string(&mut response) {
        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionReset);
        assert!(!response.is_empty(), "reject response absent");
    }
    response
}
fn target(w: &SetupWizard, path: &str) -> String {
    format!("{path}?{}", w.url.split_once('?').unwrap().1)
}
fn change(provider: ProviderId, revision: u64, key: Option<&str>) -> ProviderSetupChange {
    ProviderSetupChange {
        provider,
        expected_revision: revision,
        enabled: true,
        key: key.map(|k| Zeroizing::new(k.into())),
        remove_key: false,
        email: None,
    }
}
#[test]
fn parser_requires_explicit_network_approval_and_rejects_duplicate_options() {
    let parse_words = |words: &[&str]| parse(&words.iter().map(OsString::from).collect::<Vec<_>>());
    for words in [
        vec!["test", "openalex"],
        vec!["test", "all", "--confirm-network-request"],
        vec!["setup", "--no-open", "--no-open"],
        vec!["setup", "arxiv", "crossref"],
        vec!["setup", "--api-key", "CANARY"],
    ] {
        assert!(parse_words(&words).is_err());
    }
    assert!(matches!(
        parse_words(&["doctor"]),
        Ok(ProviderCommand::Status)
    ));
    assert!(matches!(
        parse_words(&["test", "pubmed", "--confirm-network-request"]),
        Ok(ProviderCommand::Test(ProviderId::PubMed))
    ));
    assert!(matches!(
        parse_words(&["setup", "crossref", "--no-open"]),
        Ok(ProviderCommand::Setup(ProviderSetupOptions {
            provider: Some(ProviderId::Crossref),
            no_open: true
        }))
    ));
}
#[test]
fn result_uses_selected_provider_diagnostic_and_never_returns_papers_or_secrets() {
    let reply = json!({"result":{"structuredContent":{"status":"warning","results":[{"secret":"PAPER_CANARY"}],"diagnostics":{"providers":{"arxiv":{"status":"ok"}}}}}});
    let result = test_result(ProviderId::Arxiv, &reply);
    assert_eq!(result.exit_code(), 0);
    assert!(!result.stdout().contains("PAPER_CANARY"));
    assert_eq!(test_result(ProviderId::Crossref, &reply).exit_code(), 1);
    assert_eq!(
        test_result(ProviderId::Arxiv, &json!({"result":{"isError":true}})).exit_code(),
        1
    );
}
#[test]
fn actual_http_get_is_read_only_and_bound_to_token_host_origin_and_size() {
    let f = Fixture::new();
    let secrets = Arc::new(Secrets::default());
    let w = SetupWizard::start(
        f.context(secrets.clone()),
        Some(ProviderId::OpenAlex),
        0,
        Duration::from_secs(60),
    )
    .unwrap();
    let before = f.store().load().unwrap();
    let page = request(&w, "GET", &target(&w, "/"), "", "");
    assert!(page.starts_with("HTTP/1.1 200"));
    assert!(page.contains("Cache-Control: no-store"));
    assert_eq!(secrets.reads.load(Ordering::SeqCst), 0);
    for (url, headers, body, reason) in [
        (
            "/?token=wrong".to_owned(),
            "".to_owned(),
            "".to_owned(),
            "invalid-token",
        ),
        (
            target(&w, "/"),
            "Origin: https://evil.invalid\r\n".into(),
            "".into(),
            "invalid-origin",
        ),
        (
            target(&w, "/"),
            "Sec-Fetch-Site: cross-site\r\n".into(),
            "".into(),
            "invalid-origin",
        ),
        (
            target(&w, "/"),
            "Host: evil.invalid\r\n".into(),
            "".into(),
            "invalid-request",
        ),
        (
            target(&w, "/"),
            "Transfer-Encoding: chunked\r\n".into(),
            "".into(),
            "invalid-request",
        ),
        (target(&w, "/"), "".into(), "x".into(), "request-too-large"),
    ] {
        let response = request(&w, "GET", &url, &headers, &body);
        assert!(response.contains(reason), "{response}");
    }
    for (raw, reason) in [
        (
            format!(
                "GET {} HTTP/1.1\r\nHost: evil.invalid\r\n\r\n",
                target(&w, "/")
            ),
            "invalid-host",
        ),
        (
            format!(
                "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\n\r\n",
                target(&w, "/save"),
                w.port,
                MAX_BODY + 1
            ),
            "request-too-large",
        ),
    ] {
        let mut stream = TcpStream::connect(("127.0.0.1", w.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream.write_all(raw.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.contains(reason), "{response}");
    }
    assert_eq!(f.store().load().unwrap().revision, before.revision);
    assert!(w.is_running());
}
#[test]
fn http_post_requires_approval_rejects_duplicates_and_stale_revision_then_saves_once() {
    let f = Fixture::new();
    let secrets = Arc::new(Secrets::default());
    let w = SetupWizard::start(
        f.context(secrets.clone()),
        Some(ProviderId::Arxiv),
        0,
        Duration::from_secs(60),
    )
    .unwrap();
    let base = format!(
        "provider=arxiv&revision={}&enabled=yes",
        f.store().load().unwrap().revision
    );
    for (body, reason) in [
        (base.clone(), "provider-approval-required"),
        (format!("{base}&approve=yes&approve=yes"), "duplicate-field"),
        (
            "provider=arxiv&revision=999&approve=yes".into(),
            "global-settings-revision-conflict",
        ),
        (format!("{base}&approve=yes&api_key=%ZZ"), "invalid-form"),
    ] {
        assert!(
            request(
                &w,
                "POST",
                &target(&w, "/save"),
                "Content-Type: application/x-www-form-urlencoded\r\n",
                &body
            )
            .contains(reason)
        );
    }
    let response = request(
        &w,
        "POST",
        &target(&w, "/save"),
        "Content-Type: application/x-www-form-urlencoded\r\n",
        &format!("{base}&approve=yes"),
    );
    assert!(response.contains("configuration saved"));
    let deadline = Instant::now() + Duration::from_secs(2);
    while w.is_running() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(w.saved.load(Ordering::Acquire));
    assert!(!w.is_running());
    assert_eq!(secrets.reads.load(Ordering::SeqCst), 0);
}
#[test]
fn request_deadline_bounds_slow_headers_and_wizard_expiration() {
    let f = Fixture::new();
    let w = SetupWizard::start(
        f.context(Arc::new(Secrets::default())),
        None,
        0,
        Duration::from_millis(250),
    )
    .unwrap();
    let start = Instant::now();
    let mut s = TcpStream::connect(("127.0.0.1", w.port)).unwrap();
    s.write_all(b"GET /").unwrap();
    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut reply = String::new();
    s.read_to_string(&mut reply).unwrap();
    assert!(reply.contains("request-timeout"));
    drop(w);
    assert!(start.elapsed() < Duration::from_secs(2));
}
#[test]
fn rotation_preserves_other_providers_and_rolls_back_store_readback_and_cas_failures() {
    for failure in [
        "write",
        "write_absent",
        "readback",
        "readback_absent",
        "cas",
    ] {
        let f = Fixture::new();
        let old = SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap();
        let mut loaded = f.store().load().unwrap();
        loaded.settings.providers.openalex.api_key_ref = Some(old.clone());
        f.store().replace(loaded.revision, loaded.settings).unwrap();
        let revision = f.store().load().unwrap().revision;
        let secrets = Arc::new(Secrets {
            fail_write: failure.starts_with("write"),
            skip_write: failure.ends_with("absent"),
            bad_readback: failure == "readback",
            race: (failure == "cas").then(|| f.store()),
            ..Default::default()
        });
        secrets
            .values
            .lock()
            .unwrap()
            .insert(old.storage_key().into(), b"OLD_CANARY".to_vec());
        let result = crate::desktop::apply_provider_setup(
            &f.env,
            &crate::embedded_content().unwrap(),
            secrets.clone(),
            change(ProviderId::OpenAlex, revision, Some("NEW_CANARY")),
        );
        assert_eq!(
            result,
            Err(if failure.starts_with("write") {
                "secure-store-write-failed"
            } else if failure.starts_with("readback") {
                "secure-store-read-back-failed"
            } else {
                "revision-conflict"
            })
        );
        if failure == "cas" {
            assert!(!f.store().load().unwrap().settings.providers.arxiv.enabled);
        }
        assert_eq!(
            f.store()
                .load()
                .unwrap()
                .settings
                .providers
                .openalex
                .api_key_ref,
            Some(old.clone())
        );
        assert_eq!(secrets.values.lock().unwrap().len(), 1);
        assert_eq!(
            secrets
                .values
                .lock()
                .unwrap()
                .get(old.storage_key())
                .unwrap(),
            b"OLD_CANARY"
        );
    }
}
#[test]
fn rotation_old_cleanup_failure_is_reported_and_empty_key_keeps_existing_without_reading() {
    let f = Fixture::new();
    let old = SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap();
    let mut loaded = f.store().load().unwrap();
    loaded.settings.providers.pubmed.api_key_ref = Some(old.clone());
    f.store().replace(loaded.revision, loaded.settings).unwrap();
    let secrets = Arc::new(Secrets {
        fail_remove: Some(old.storage_key().into()),
        ..Default::default()
    });
    secrets
        .values
        .lock()
        .unwrap()
        .insert(old.storage_key().into(), b"OLD_CANARY".to_vec());
    let revision = f.store().load().unwrap().revision;
    assert!(
        crate::desktop::apply_provider_setup(
            &f.env,
            &crate::embedded_content().unwrap(),
            secrets.clone(),
            change(ProviderId::PubMed, revision, None)
        )
        .is_ok()
    );
    assert_eq!(secrets.reads.load(Ordering::SeqCst), 0);
    let before = f.store().load().unwrap();
    assert_eq!(before.settings.providers.pubmed.api_key_ref, Some(old));
    let other = before.settings.providers.openalex.clone();
    let result = crate::desktop::apply_provider_setup(
        &f.env,
        &crate::embedded_content().unwrap(),
        secrets.clone(),
        change(ProviderId::PubMed, before.revision, Some("NEW_CANARY")),
    )
    .unwrap();
    assert_eq!(result, "provider-secret-updated-cleanup-required");
    let after = f.store().load().unwrap();
    assert_ne!(
        after.settings.providers.pubmed.api_key_ref,
        before.settings.providers.pubmed.api_key_ref
    );
    assert_eq!(after.settings.providers.openalex, other);
    assert_eq!(secrets.values.lock().unwrap().len(), 2);
    let bytes = fs::read_to_string(f.root.join("config/v2/settings.json")).unwrap();
    assert!(!bytes.contains("NEW_CANARY") && !bytes.contains("OLD_CANARY"));
}
#[test]
fn explicit_removal_preserves_old_key_on_stale_revision_and_reports_committed_cleanup() {
    for cleanup_fails in [false, true] {
        let f = Fixture::new();
        let old = SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap();
        let mut loaded = f.store().load().unwrap();
        loaded.settings.providers.pubmed.api_key_ref = Some(old.clone());
        loaded.settings.providers.pubmed.enabled = true;
        f.store().replace(loaded.revision, loaded.settings).unwrap();
        let secrets = Arc::new(Secrets {
            fail_remove: cleanup_fails.then(|| old.storage_key().into()),
            ..Default::default()
        });
        secrets
            .values
            .lock()
            .unwrap()
            .insert(old.storage_key().into(), b"OLD_CANARY".to_vec());
        let before = f.store().load().unwrap();
        let mut removal = change(ProviderId::PubMed, before.revision - 1, None);
        removal.remove_key = true;
        assert!(
            crate::desktop::apply_provider_setup(
                &f.env,
                &crate::embedded_content().unwrap(),
                secrets.clone(),
                removal
            )
            .is_err()
        );
        assert_eq!(
            f.store()
                .load()
                .unwrap()
                .settings
                .providers
                .pubmed
                .api_key_ref,
            Some(old.clone())
        );
        assert!(
            secrets
                .values
                .lock()
                .unwrap()
                .contains_key(old.storage_key())
        );
        let mut removal = change(ProviderId::PubMed, before.revision, None);
        removal.remove_key = true;
        let result = crate::desktop::apply_provider_setup(
            &f.env,
            &crate::embedded_content().unwrap(),
            secrets.clone(),
            removal,
        )
        .unwrap();
        assert_eq!(result.contains("cleanup-required"), cleanup_fails);
        let after = f.store().load().unwrap();
        assert!(!after.settings.providers.pubmed.enabled);
        assert_eq!(after.settings.providers.pubmed.api_key_ref, None);
        assert_eq!(
            after.settings.providers.openalex,
            before.settings.providers.openalex
        );
        assert_eq!(
            secrets
                .values
                .lock()
                .unwrap()
                .contains_key(old.storage_key()),
            cleanup_fails
        );
        assert_eq!(secrets.reads.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn unavailable_secret_store_refuses_keys_but_allows_public_configuration() {
    let f = Fixture::new();
    let secrets = Arc::new(Secrets {
        unavailable: true,
        ..Default::default()
    });
    let revision = f.store().load().unwrap().revision;
    assert_eq!(
        crate::desktop::apply_provider_setup(
            &f.env,
            &crate::embedded_content().unwrap(),
            secrets.clone(),
            change(ProviderId::OpenAlex, revision, Some("PRIVATE_CANARY"))
        ),
        Err("provider-secret-store-unavailable")
    );
    let mut public = change(ProviderId::Crossref, revision, None);
    public.email = Some("fixture@example.test".into());
    assert!(
        crate::desktop::apply_provider_setup(
            &f.env,
            &crate::embedded_content().unwrap(),
            secrets.clone(),
            public
        )
        .is_ok()
    );
    assert_eq!(secrets.reads.load(Ordering::SeqCst), 0);
    let page = render_page(&f.context(secrets), Some(ProviderId::PubMed), "TOKEN").unwrap();
    assert!(page.contains("unavailable"));
    assert!(!page.contains("name=api_key"));
}
#[test]
fn same_mcp_session_refreshes_after_page_save_and_external_revision_without_secret_reads() {
    use qiongli_runtime::{LiteToolRegistry, mcp::LiteMcpServer};
    let f = Fixture::new();
    let content = crate::embedded_content().unwrap();
    let secrets = Arc::new(Secrets::default());
    let backend = Arc::new(NativeProviderConfiguration::with_store(
        &f.env,
        &content,
        secrets.clone(),
    ));
    let snapshot = backend.snapshot().unwrap();
    let server = LiteMcpServer::production_deferred(
        "test",
        "0",
        LiteToolRegistry::from_embedded_content(&content).unwrap(),
        snapshot.metadata,
        snapshot.loader,
    )
    .with_provider_configuration(backend.clone());
    let call = |name: &str, args: Value| {
        server.handle(json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args}})).unwrap()
    };
    let inventory = server
        .handle(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))
        .unwrap();
    let tools = inventory["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 15);
    for name in ["qiongli_save_provider_config", "qiongli_configure_provider"] {
        let description = tools.iter().find(|tool| tool["name"] == name).unwrap()["description"]
            .as_str()
            .unwrap();
        assert!(
            description.contains("local")
                && description.contains("never")
                && description.contains("chat")
        );
    }
    let original = call("qiongli_config_status", json!({}));
    let configured = call(
        "qiongli_configure_provider",
        json!({"host":"127.0.0.1","port":0}),
    );
    assert!(
        configured["result"]["structuredContent"]["url"].is_string(),
        "{configured}"
    );
    let slot = backend.wizard.lock().unwrap();
    let w = slot.as_ref().unwrap();
    let revision = f.store().load().unwrap().revision;
    let response = request(
        w,
        "POST",
        &target(w, "/save"),
        "Content-Type: application/x-www-form-urlencoded\r\n",
        &format!("provider=arxiv&revision={revision}&approve=yes"),
    );
    assert!(response.contains("configuration saved"));
    drop(slot);
    let updated = call("qiongli_config_status", json!({}));
    assert_ne!(updated, original);
    assert_eq!(
        updated["result"]["structuredContent"]["redacted_config"]["providers"]["arxiv"]["enabled"],
        false
    );
    let mut loaded = f.store().load().unwrap();
    loaded.settings.providers.arxiv.enabled = true;
    f.store().replace(loaded.revision, loaded.settings).unwrap();
    let external = call("qiongli_config_status", json!({}));
    assert_eq!(
        external["result"]["structuredContent"]["redacted_config"]["providers"]["arxiv"]["enabled"],
        true
    );
    assert_eq!(secrets.reads.load(Ordering::SeqCst), 0);
    assert!(!external.to_string().contains(f.root.to_str().unwrap()));
    assert_eq!(
        call(
            "qiongli_configure_provider",
            json!({"provider":"arxiv","host":"evil.invalid"})
        )["error"]["code"],
        -32602
    );
}
#[test]
fn approved_provider_test_sends_only_one_selected_bounded_query_and_accepts_empty_results() {
    use qiongli_runtime::providers::{
        ProviderAvailability, ProviderEndpoints, ProviderField, ProviderRuntime,
    };
    use qiongli_runtime::{FullProjectToolRegistry, LiteToolRegistry, mcp::LiteMcpServer};
    let selected = TcpListener::bind("127.0.0.1:0").unwrap();
    selected.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}/", selected.local_addr().unwrap());
    let other = TcpListener::bind("127.0.0.1:0").unwrap();
    other.set_nonblocking(true).unwrap();
    let other_endpoint = format!("http://{}/", other.local_addr().unwrap());
    let worker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match selected.accept() {
                Ok((s, _)) => break s,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(e) => panic!("local provider request absent: {e}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut raw = Vec::new();
        while !raw.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            raw.push(byte[0]);
            assert!(raw.len() < 8192);
        }
        let request = String::from_utf8(raw).unwrap();
        assert!(request.starts_with("GET /works?"));
        assert!(request.contains("rows=1") && request.contains("query=research"));
        let body = r#"{"status":"ok","message":{"items":[]}}"#;
        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        assert!(selected.accept().is_err());
        request
    });
    let mut access = ProviderAccess::builder();
    for p in PROVIDER_ORDER {
        access.set_availability(p, ProviderAvailability::Ready);
    }
    access.set_value(
        ProviderId::Crossref,
        ProviderField::Email,
        "fixture@example.test",
    );
    let endpoints = ProviderEndpoints::from_urls(
        &other_endpoint,
        &other_endpoint,
        &endpoint,
        &other_endpoint,
        &other_endpoint,
    )
    .unwrap();
    let content = crate::embedded_content().unwrap();
    let lite = LiteMcpServer::with_provider_runtime(
        "test",
        "0",
        LiteToolRegistry::from_embedded_content(&content).unwrap(),
        ProviderRuntime::with_endpoints(endpoints, access.build()).unwrap(),
    );
    let full = crate::mcp::FullMcpServer::new(
        lite,
        FullProjectToolRegistry::from_embedded_content(&content).unwrap(),
        None,
        None,
    );
    let result = test_with_server(&full, ProviderId::Crossref);
    assert_eq!(result.exit_code(), 0, "{}", result.stderr());
    assert!(result.stdout().contains("connected"));
    assert!(!result.stdout().contains("fixture@example.test"));
    worker.join().unwrap();
    assert!(matches!(other.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock));
}
