use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "qiongli-dsh-package-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// An npm package with a valid content pack whose version differs from this CLI.
pub(crate) fn package(root: &Path, version: &str) -> Value {
    let content = crate::embedded_content().unwrap();
    let original = content.pack();
    let mut manifest = original.manifest().clone();
    manifest.content_version = version.into();
    let manifest_bytes = serde_json_canonicalizer::to_vec(&manifest).unwrap();
    let payload_start = 20 + original.manifest_bytes().len();
    let payload = &original.core_bytes()[payload_start..];
    let mut bytes = b"QLPACK\0\0".to_vec();
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&(manifest_bytes.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&manifest_bytes);
    bytes.extend_from_slice(payload);
    let digest = format!("{:x}", Sha256::digest(&bytes));
    load_resource_pack(&bytes, &digest).unwrap();
    let dsh = root.join("dsh");
    fs::create_dir_all(&dsh).unwrap();
    let mut manifests = serde_json::Map::new();
    for entry in &manifest.entries {
        let start = entry.payload_offset as usize;
        let data = &payload[start..start + entry.size_bytes as usize];
        let relative = match entry.path.as_str() {
            ".codex-plugin/plugin.json" | ".claude-plugin/plugin.json" => {
                manifests.insert(
                    entry.path.clone(),
                    Value::String(String::from_utf8(data.to_vec()).unwrap()),
                );
                continue;
            }
            "workflow/no-qiongli/SKILL.md" => "skills/no-qiongli/SKILL.md".to_owned(),
            path => format!(
                "skills/qiongli-workflow/{}",
                path.strip_prefix("workflow/").unwrap_or(path)
            ),
        };
        let path = dsh.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
    }
    for (name, data) in [
        ("index.mjs", "export default {};"),
        ("cordis.patch.yml", "name: qiongli\n"),
        ("skills.json", "[]"),
    ] {
        fs::write(dsh.join(name), data).unwrap();
    }
    fs::write(
        root.join("package.json"),
        serde_json::to_vec(&metadata(version)).unwrap(),
    )
    .unwrap();
    let receipt = json!({"schema_version":1,"platform":"deepseek-npm","source":{
        "schema_version":1,"version":version,"source_commit":crate::embedded_source_commit(),
        "content_source_commit":manifest.source_commit,"content_root_sha256":manifest.content_root_sha256,
        "pack_sha256":digest,"pack_manifest_json":String::from_utf8(manifest_bytes).unwrap(),
        "entries":manifest.entries.iter().map(|e|json!({"path":e.path,"size_bytes":e.size_bytes,"sha256":e.sha256})).collect::<Vec<_>>()
    },"source_manifest_bytes":manifests});
    fs::write(
        dsh.join(".qiongli-marketplace.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    receipt
}
fn metadata(version: &str) -> Value {
    json!({"name":"qiongli","version":version,"main":"dsh/index.mjs","dsh":{"bundle":{"patch":"./dsh/cordis.patch.yml"}}})
}

fn response(status: &str, headers: &str, body: &[u8]) -> Result<String, &'static str> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/qiongli/latest", listener.local_addr().unwrap());
    let wire = [
        format!("HTTP/1.1 {status}\r\nConnection: close\r\n{headers}\r\n").into_bytes(),
        body.to_vec(),
    ]
    .concat();
    let server = std::thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(error) => panic!("local HTTP fixture accept failed: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = [0; 4096];
        let length = stream.read(&mut request).unwrap();
        assert!(
            std::str::from_utf8(&request[..length])
                .unwrap()
                .starts_with("GET /qiongli/latest ")
        );
        let _ = stream.write_all(&wire);
    });
    let result = resolve_latest(&endpoint);
    server.join().unwrap();
    result
}

#[test]
fn latest_lookup_freezes_owned_stable_version_without_cli_version_binding() {
    let body = serde_json::to_vec(&metadata("2.99.0")).unwrap();
    let first = response(
        "200 OK",
        &format!("Content-Length: {}\r\n", body.len()),
        &body,
    )
    .unwrap();
    let next = serde_json::to_vec(&metadata("2.99.1")).unwrap();
    assert_eq!(response("200 OK", "", &next).unwrap(), "2.99.1");
    assert_eq!(first, "2.99.0");
    assert_ne!(first, env!("CARGO_PKG_VERSION"));
}

#[test]
fn latest_lookup_rejects_status_redirect_encoding_size_and_metadata_drift() {
    let valid = serde_json::to_vec(&metadata("2.99.0")).unwrap();
    for (status, headers, body) in [
        ("404 Not Found", "", valid.clone()),
        (
            "302 Found",
            "Location: http://127.0.0.1:1/untrusted\r\n",
            valid.clone(),
        ),
        ("200 OK", "Content-Encoding: gzip\r\n", valid.clone()),
        ("200 OK", "Content-Length: 1048577\r\n", vec![]),
        ("200 OK", "", vec![b' '; 1048577]),
        ("200 OK", "", b"not-json".to_vec()),
    ] {
        assert_eq!(response(status, headers, &body), Err(LOOKUP));
    }
    for field in ["name", "main", "version"] {
        let mut changed = metadata("2.99.0");
        changed[field] = json!("wrong");
        assert_eq!(
            response("200 OK", "", &serde_json::to_vec(&changed).unwrap()),
            Err(LOOKUP)
        );
    }
    let mut changed = metadata("2.99.0");
    changed["dsh"]["bundle"]["patch"] = json!("../escape");
    assert_eq!(
        response("200 OK", "", &serde_json::to_vec(&changed).unwrap()),
        Err(LOOKUP)
    );
    for value in ["v2.99.0", "2.99", "2.99.0-rc.1", "2.99.0+build", "2.99.0\n"] {
        assert_eq!(stable_version(value), Err(LOOKUP));
    }
}

#[test]
fn newer_content_pack_verifies_and_wrong_approved_version_is_rejected() {
    let fixture = Fixture::new();
    let receipt = package(&fixture.0, "2.99.0");
    assert_ne!(
        receipt["source"]["pack_sha256"],
        crate::embedded_content().unwrap().pack().pack_sha256()
    );
    assert_eq!(
        verify_content(&fixture.0.join("dsh"), "2.99.0", &receipt),
        Ok(())
    );
    let mut reordered = receipt.clone();
    reordered["source"]["entries"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_eq!(
        verify_content(&fixture.0.join("dsh"), "2.99.0", &reordered),
        Ok(())
    );
    assert_eq!(
        verify_content(&fixture.0.join("dsh"), env!("CARGO_PKG_VERSION"), &receipt),
        Err(INVALID)
    );
}

#[test]
fn content_verification_rejects_receipt_hash_path_source_and_missing_entry() {
    let fixture = Fixture::new();
    let receipt = package(&fixture.0, "2.99.0");
    let root = fixture.0.join("dsh");
    for key in [
        "version",
        "content_source_commit",
        "content_root_sha256",
        "pack_sha256",
    ] {
        let mut changed = receipt.clone();
        changed["source"][key] = json!("wrong");
        assert_eq!(
            verify_content(&root, "2.99.0", &changed),
            Err(INVALID),
            "{key}"
        );
    }
    let mut duplicate = receipt.clone();
    duplicate["source"]["entries"][1] = duplicate["source"]["entries"][0].clone();
    assert_eq!(verify_content(&root, "2.99.0", &duplicate), Err(INVALID));
    for key in ["path", "sha256", "size_bytes"] {
        let mut changed = receipt.clone();
        changed["source"]["entries"][0][key] = json!("../escape");
        assert_eq!(
            verify_content(&root, "2.99.0", &changed),
            Err(INVALID),
            "{key}"
        );
    }
    let mut changed = receipt.clone();
    changed["schema_version"] = json!(2);
    assert_eq!(verify_content(&root, "2.99.0", &changed), Err(INVALID));
    let mut changed = receipt.clone();
    changed["source_manifest_bytes"][".codex-plugin/plugin.json"] = json!("{}");
    assert_eq!(verify_content(&root, "2.99.0", &changed), Err(INVALID));
    for file in ["index.mjs", "cordis.patch.yml", "skills.json"] {
        let path = root.join(file);
        let saved = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(verify_content(&root, "2.99.0", &receipt), Err(INVALID));
        fs::write(path, saved).unwrap();
    }
    let path = root.join("skills/qiongli-workflow/SKILL.md");
    let mut tampered = fs::read(&path).unwrap();
    tampered[0] ^= 1;
    fs::write(path, tampered).unwrap();
    assert_eq!(verify_content(&root, "2.99.0", &receipt), Err(INVALID));
}

#[test]
fn installed_resource_paths_reject_escape_and_oversized_files() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("file"), b"abc").unwrap();
    for path in ["../file", "/file", "file\\escape", "file\n"] {
        assert_eq!(regular_bytes(&fixture.0, path, 3), Err(INVALID));
    }
    assert_eq!(regular_bytes(&fixture.0, "file", 2), Err(INVALID));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(fixture.0.join("file"), fixture.0.join("link")).unwrap();
        assert_eq!(regular_bytes(&fixture.0, "link", 3), Err(INVALID));
    }
}

#[test]
#[ignore = "requires an explicitly mounted retained qualified npm package"]
fn retained_qualified_npm_bundle_matches_real_projection() {
    let root = std::path::PathBuf::from(
        std::env::var_os("QIONGLI_TEST_DSH_PACKAGE")
            .expect("private artifact fixture path required"),
    );
    let metadata: Value =
        serde_json::from_slice(&fs::read(root.join("package.json")).unwrap()).unwrap();
    let receipt: Value =
        serde_json::from_slice(&fs::read(root.join("dsh/.qiongli-marketplace.json")).unwrap())
            .unwrap();
    assert_eq!(
        verify_content(
            &root.join("dsh"),
            metadata["version"].as_str().unwrap(),
            &receipt
        ),
        Ok(())
    );
}

#[cfg(unix)]
#[test]
#[ignore = "requires an explicitly driven private PTY; never invokes a real Host"]
fn terminal_handoff_preserves_interactive_io_and_finishes_progress_first() {
    use crate::install_output::InstallWriter;
    use std::os::unix::fs::PermissionsExt;
    struct Writer(std::io::Stdout);
    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.flush()
        }
    }
    impl InstallWriter for Writer {
        fn finish_progress(&mut self) -> Result<(), &'static str> {
            writeln!(self, "FIXTURE_PROGRESS_FINISHED").map_err(|_| "installation-output-failed")
        }
        fn waiting(&mut self, _: Duration, _: Duration) -> Result<(), &'static str> {
            panic!("Qiongli must not overlay the handed-off terminal")
        }
    }
    let fixture = Fixture::new();
    let executable = fixture.0.join("dsh");
    fs::write(&executable, "#!/bin/sh\n[ -t 0 ] && [ -t 1 ] || exit 21\nprintf 'FIXTURE_DSH_PROMPT: '\nIFS= read -r answer || exit 22\n[ \"$answer\" = approved ] || exit 23\nprintf 'FIXTURE_DSH_INPUT_OK\\n'\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let environment =
        crate::command::CommandEnvironment::with_paths(None, Some(fixture.0.clone()), None);
    let mut writer = Writer(std::io::stdout());
    super::super::installation_command::run_deepseek_terminal(
        &environment,
        &executable,
        &[],
        &mut writer,
    )
    .unwrap();
}
