#![allow(clippy::disallowed_methods)]

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_platform::{
    NativePortableArchiveError, ReleaseChannel, approve_native_artifact_target,
    approve_native_portable_archive_target, compose_native_artifact,
    compose_native_portable_archive, current_target_native_artifact_identity,
    extract_native_portable_archive, native_artifact_id, native_portable_archive_file_name,
    verify_native_artifact, verify_native_portable_archive,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use serde_json::{Value, json};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const PRIVATE_PATH_CANARY: &str = "native-portable-archive-private-path-canary";

struct Fixture {
    root: PathBuf,
    home: PathBuf,
    config_root: PathBuf,
    source_binary: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-native-portable-archive-tests");
        fs::create_dir_all(&test_base).expect("portable archive test base must exist");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "{name}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        create_private_directory(&root);
        let home = root.join("home");
        create_private_directory(&home);
        let config_root = root.join(PRIVATE_PATH_CANARY);
        let source_binary = root.join(format!("source-qiongli{}", std::env::consts::EXE_SUFFIX));
        fs::copy(env!("CARGO_BIN_EXE_qiongli"), &source_binary)
            .expect("canonical native binary must copy");
        set_executable_mode(&source_binary);
        Self {
            root,
            home,
            config_root,
            source_binary,
        }
    }

    fn target(&self, parent_name: &str, leaf: &str) -> PathBuf {
        let parent = self.root.join(parent_name);
        create_private_directory(&parent);
        parent.join(leaf)
    }

    fn command(&self, executable: &Path) -> Command {
        let mut command = Command::new(executable);
        command
            .current_dir(&self.root)
            .env("PATH", "")
            .env("QIONGLI_CONFIG_HOME", &self.config_root)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home);
        command
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .expect("private fixture directory must be created");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture directory must remain private");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path)
        .expect("owner-only Windows fixture directory must be created");
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(path: &Path) {
    fs::create_dir(path).expect("private fixture directory must be created");
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture binary must be executable");
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}

fn public_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn rpc(id: u64, method: &str, params: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
}

#[test]
fn portable_archive_is_deterministic_safe_and_runtime_independent() {
    let fixture = Fixture::new("portable-archive");
    let content = qiongli::embedded_content().expect("embedded content must verify");
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .expect("current target identity must resolve");
    let artifact_id = native_artifact_id(&artifact).expect("artifact ID must render");
    let archive_file_name =
        native_portable_archive_file_name(&artifact).expect("archive filename must render");

    let source_path = fixture.target("source-parent", &artifact_id);
    let source_target = approve_native_artifact_target(&source_path, &artifact)
        .expect("source artifact target must approve");
    let source = compose_native_artifact(
        content.pack(),
        &artifact,
        &fixture.source_binary,
        &source_target,
    )
    .expect("source artifact must compose");

    let first_path = fixture.target("first-archive-parent", &archive_file_name);
    let first_target = approve_native_portable_archive_target(&first_path, &artifact)
        .expect("first archive target must approve");
    let target_debug = format!("{first_target:?}");
    assert!(target_debug.contains("<approved-native-portable-archive>"));
    assert!(!target_debug.contains(&first_path.to_string_lossy().into_owned()));
    let first = compose_native_portable_archive(content.pack(), &source_target, &first_target)
        .expect("first archive must compose");
    assert_eq!(first.artifact(), &artifact);
    assert_eq!(first.file_name(), archive_file_name);
    assert_eq!(first.manifest_sha256(), source.manifest_sha256());

    let second_path = fixture.target("second-archive-parent", &archive_file_name);
    let second_target = approve_native_portable_archive_target(&second_path, &artifact)
        .expect("second archive target must approve");
    let second = compose_native_portable_archive(content.pack(), &source_target, &second_target)
        .expect("second archive must compose");
    let original_bytes = fs::read(&first_path).unwrap();
    assert_eq!(first, second);
    assert_eq!(original_bytes, fs::read(&second_path).unwrap());
    assert_eq!(first.size_bytes(), original_bytes.len() as u64);

    let extracted_path = fixture.target("extracted-parent", &artifact_id);
    let extracted_target = approve_native_artifact_target(&extracted_path, &artifact)
        .expect("extracted artifact target must approve");
    let extracted =
        extract_native_portable_archive(content.pack(), &first_target, &extracted_target)
            .expect("archive must extract through the R3G commit path");
    assert_eq!(extracted, source);
    assert_eq!(
        extracted,
        verify_native_artifact(content.pack(), &extracted_target).unwrap()
    );

    let extracted_binary = extracted_path.join(&extracted.manifest().binary_path);
    let version = fixture
        .command(&extracted_binary)
        .arg("--version")
        .output()
        .expect("extracted CLI must start without PATH");
    assert!(version.status.success(), "{}", public_output(&version));
    assert_eq!(
        version.stdout,
        format!("qiongli {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(version.stderr.is_empty());

    let listed = fixture
        .command(&extracted_binary)
        .args(["content", "list"])
        .output()
        .expect("extracted content command must start without PATH");
    assert!(listed.status.success(), "{}", public_output(&listed));
    assert!(listed.stderr.is_empty());
    let listed_json: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed_json["pack_id"], extracted.manifest().content.pack_id);
    assert_eq!(
        listed_json["pack_sha256"],
        extracted.manifest().content.pack_sha256
    );
    assert_eq!(
        listed_json["content_root_sha256"],
        extracted.manifest().content.content_root_sha256
    );

    let mut child = fixture
        .command(&extracted_binary)
        .args([
            "mcp",
            "serve",
            "--profile",
            "marketplace-lite",
            "--transport",
            "stdio",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("extracted MCP must start without PATH");
    let requests = [
        rpc(
            1,
            "initialize",
            json!({"protocolVersion": "2025-11-25", "capabilities": {}}),
        ),
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
        rpc(2, "tools/list", json!({})),
        rpc(
            3,
            "tools/call",
            json!({"name": "qiongli_config_status", "arguments": {}}),
        ),
    ];
    {
        let stdin = child
            .stdin
            .as_mut()
            .expect("extracted MCP stdin must exist");
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    let mcp = child
        .wait_with_output()
        .expect("extracted MCP must exit on EOF");
    assert!(mcp.status.success(), "{}", public_output(&mcp));
    assert!(mcp.stderr.is_empty(), "{}", public_output(&mcp));
    let rendered = String::from_utf8(mcp.stdout).expect("MCP output must be UTF-8");
    assert!(!rendered.contains(PRIVATE_PATH_CANARY));
    let responses = rendered
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 3);
    let tools = responses
        .iter()
        .find(|response| response["id"] == 2)
        .unwrap()["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(tools, LITE_PUBLIC_TOOL_NAMES);
    assert_eq!(
        responses
            .iter()
            .find(|response| response["id"] == 3)
            .unwrap()["result"]["structuredContent"]["config_path"],
        "<managed-native-config>"
    );

    assert_eq!(
        compose_native_portable_archive(content.pack(), &source_target, &first_target),
        Err(NativePortableArchiveError::TargetExists)
    );
    assert_eq!(fs::read(&first_path).unwrap(), original_bytes);
    assert_eq!(
        extract_native_portable_archive(content.pack(), &first_target, &extracted_target),
        Err(NativePortableArchiveError::DestinationExists)
    );
    assert_eq!(
        verify_native_artifact(content.pack(), &extracted_target).unwrap(),
        source
    );

    let hard_link = fixture.root.join("archive-hard-link.zip");
    fs::hard_link(&first_path, &hard_link).unwrap();
    assert_eq!(
        verify_native_portable_archive(content.pack(), &first_target),
        Err(NativePortableArchiveError::ArchiveDrift)
    );
    fs::remove_file(hard_link).unwrap();

    let locked_path = fixture.target("locked-archive-parent", &archive_file_name);
    let locked_target = approve_native_portable_archive_target(&locked_path, &artifact).unwrap();
    let lock_path = locked_path
        .parent()
        .unwrap()
        .join(".qiongli.qiongli-native-portable-archive.lock");
    fs::write(&lock_path, b"held").unwrap();
    assert_eq!(
        compose_native_portable_archive(content.pack(), &source_target, &locked_target),
        Err(NativePortableArchiveError::TargetBusy)
    );
    fs::remove_file(lock_path).unwrap();

    assert_eq!(
        approve_native_portable_archive_target(
            fixture.target("wrong-name-parent", "qiongli-current.zip"),
            &artifact,
        )
        .unwrap_err(),
        NativePortableArchiveError::InvalidTarget
    );

    let source_binary = source_path.join(&source.manifest().binary_path);
    fs::OpenOptions::new()
        .append(true)
        .open(source_binary)
        .unwrap()
        .write_all(b"drift")
        .unwrap();
    let drift_target_path = fixture.target("drift-archive-parent", &archive_file_name);
    let drift_target =
        approve_native_portable_archive_target(&drift_target_path, &artifact).unwrap();
    assert_eq!(
        compose_native_portable_archive(content.pack(), &source_target, &drift_target),
        Err(NativePortableArchiveError::SourceArtifactInvalid)
    );
    assert!(!drift_target_path.exists());
}
