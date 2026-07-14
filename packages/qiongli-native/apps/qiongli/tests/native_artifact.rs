#![allow(clippy::disallowed_methods)]

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_platform::{
    NATIVE_ARTIFACT_MANIFEST_FILE, NativeArtifactError, NativeArtifactStatus,
    approve_native_artifact_target, compose_native_artifact,
    current_target_native_artifact_identity, native_artifact_id, verify_native_artifact,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use serde_json::{Value, json};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const PRIVATE_PATH_CANARY: &str = "native-artifact-private-path-canary";

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
        let test_base = native_root.join("target/qiongli-native-artifact-tests");
        fs::create_dir_all(&test_base).expect("native artifact test base must exist");
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

    fn artifact_target(&self, parent_name: &str, artifact_id: &str) -> PathBuf {
        let parent = self.root.join(parent_name);
        create_private_directory(&parent);
        parent.join(artifact_id)
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

#[cfg(not(unix))]
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
fn assembled_artifact_is_deterministic_tamper_evident_and_runtime_independent() {
    let fixture = Fixture::new("complete-artifact");
    let content = qiongli::embedded_content().expect("embedded content must verify");
    let artifact = current_target_native_artifact_identity(
        env!("CARGO_PKG_VERSION"),
        qiongli_platform::ReleaseChannel::Alpha,
    )
    .expect("current target identity must resolve");
    let artifact_id = native_artifact_id(&artifact).expect("artifact ID must render");
    let first_path = fixture.artifact_target("first", &artifact_id);
    let second_path = fixture.artifact_target("second", &artifact_id);
    let first = approve_native_artifact_target(&first_path, &artifact)
        .expect("first artifact target must approve");
    let second = approve_native_artifact_target(&second_path, &artifact)
        .expect("second artifact target must approve");
    let target_debug = format!("{first:?}");
    assert!(target_debug.contains("<approved-native-artifact>"));
    assert!(!target_debug.contains(&first_path.to_string_lossy().into_owned()));

    let first_composed =
        compose_native_artifact(content.pack(), &artifact, &fixture.source_binary, &first)
            .expect("first artifact must compose");
    let second_composed =
        compose_native_artifact(content.pack(), &artifact, &fixture.source_binary, &second)
            .expect("second artifact must compose");
    assert_eq!(
        first_composed,
        verify_native_artifact(content.pack(), &first).unwrap()
    );
    assert_eq!(
        second_composed,
        verify_native_artifact(content.pack(), &second).unwrap()
    );
    assert_eq!(first_composed, second_composed);
    assert_eq!(
        fs::read(first_path.join(NATIVE_ARTIFACT_MANIFEST_FILE)).unwrap(),
        fs::read(second_path.join(NATIVE_ARTIFACT_MANIFEST_FILE)).unwrap()
    );
    assert_eq!(
        first_composed.manifest().status,
        NativeArtifactStatus::AssembledUnpublished
    );
    assert_eq!(first_composed.manifest().artifact, artifact);
    assert_eq!(
        first_composed.manifest().content.pack_sha256,
        content.pack().pack_sha256()
    );
    assert_eq!(
        first_composed.manifest().content.content_root_sha256,
        content.pack().manifest().content_root_sha256
    );

    let artifact_binary = first_path.join(&first_composed.manifest().binary_path);
    let version = fixture
        .command(&artifact_binary)
        .arg("--version")
        .output()
        .expect("artifact CLI must start without PATH");
    assert!(version.status.success(), "{}", public_output(&version));
    assert_eq!(
        version.stdout,
        format!("qiongli {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(version.stderr.is_empty());

    let listed = fixture
        .command(&artifact_binary)
        .args(["content", "list"])
        .output()
        .expect("artifact content command must start without PATH");
    assert!(listed.status.success(), "{}", public_output(&listed));
    assert!(listed.stderr.is_empty());
    let listed_json: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(
        listed_json["pack_id"],
        first_composed.manifest().content.pack_id
    );
    assert_eq!(
        listed_json["pack_sha256"],
        first_composed.manifest().content.pack_sha256
    );
    assert_eq!(
        listed_json["content_root_sha256"],
        first_composed.manifest().content.content_root_sha256
    );

    let mut child = fixture
        .command(&artifact_binary)
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
        .expect("artifact MCP must start without PATH");
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
        let stdin = child.stdin.as_mut().expect("artifact MCP stdin must exist");
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    let mcp = child
        .wait_with_output()
        .expect("artifact MCP must exit on EOF");
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

    fs::write(first_path.join("unexpected"), b"drift").unwrap();
    assert_eq!(
        verify_native_artifact(content.pack(), &first),
        Err(NativeArtifactError::ArtifactDrift)
    );
    fs::remove_file(first_path.join("unexpected")).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let manifest_path = first_path.join(NATIVE_ARTIFACT_MANIFEST_FILE);
        fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            verify_native_artifact(content.pack(), &first),
            Err(NativeArtifactError::ArtifactDrift)
        );
        fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    let external_link = fixture.root.join("artifact-binary-hard-link");
    fs::hard_link(&artifact_binary, &external_link).unwrap();
    assert_eq!(
        verify_native_artifact(content.pack(), &first),
        Err(NativeArtifactError::ArtifactDrift)
    );
    fs::remove_file(external_link).unwrap();

    let manifest_path = first_path.join(NATIVE_ARTIFACT_MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(&manifest_path)
        .unwrap()
        .write_all(b"\n")
        .unwrap();
    assert_eq!(
        verify_native_artifact(content.pack(), &first),
        Err(NativeArtifactError::ManifestInvalid)
    );
    fs::write(&manifest_path, manifest_bytes).unwrap();

    let mut rewritten = first_composed.manifest().clone();
    rewritten.content.pack_sha256 = "0".repeat(64);
    fs::write(
        &manifest_path,
        serde_json_canonicalizer::to_vec(&rewritten).unwrap(),
    )
    .unwrap();
    assert_eq!(
        verify_native_artifact(content.pack(), &first),
        Err(NativeArtifactError::ArtifactDrift)
    );
    fs::write(
        &manifest_path,
        serde_json_canonicalizer::to_vec(first_composed.manifest()).unwrap(),
    )
    .unwrap();

    fs::OpenOptions::new()
        .append(true)
        .open(&artifact_binary)
        .unwrap()
        .write_all(b"tamper")
        .unwrap();
    assert_eq!(
        verify_native_artifact(content.pack(), &first),
        Err(NativeArtifactError::ArtifactDrift)
    );
}

#[test]
fn composition_conflicts_and_unsafe_sources_fail_closed() {
    let fixture = Fixture::new("artifact-conflicts");
    let content = qiongli::embedded_content().expect("embedded content must verify");
    let artifact = current_target_native_artifact_identity(
        env!("CARGO_PKG_VERSION"),
        qiongli_platform::ReleaseChannel::Alpha,
    )
    .unwrap();
    let artifact_id = native_artifact_id(&artifact).unwrap();

    let wrong_parent = fixture.root.join("wrong-parent");
    create_private_directory(&wrong_parent);
    assert_eq!(
        approve_native_artifact_target(wrong_parent.join("qiongli-current"), &artifact)
            .unwrap_err(),
        NativeArtifactError::InvalidTarget
    );

    let target_path = fixture.artifact_target("existing", &artifact_id);
    let target = approve_native_artifact_target(&target_path, &artifact).unwrap();
    create_private_directory(&target_path);
    fs::write(target_path.join("user-canary"), b"preserve").unwrap();
    assert_eq!(
        compose_native_artifact(content.pack(), &artifact, &fixture.source_binary, &target,),
        Err(NativeArtifactError::TargetExists)
    );
    assert_eq!(
        fs::read(target_path.join("user-canary")).unwrap(),
        b"preserve"
    );
    fs::remove_dir_all(&target_path).unwrap();

    let lock_path = target_path
        .parent()
        .unwrap()
        .join(".qiongli.qiongli-native-artifact.lock");
    fs::write(&lock_path, b"held").unwrap();
    assert_eq!(
        compose_native_artifact(content.pack(), &artifact, &fixture.source_binary, &target,),
        Err(NativeArtifactError::TargetBusy)
    );
    fs::remove_file(lock_path).unwrap();

    let oversized = fixture.root.join("oversized-qiongli");
    let oversized_file = fs::File::create(&oversized).unwrap();
    oversized_file.set_len(128 * 1024 * 1024 + 1).unwrap();
    drop(oversized_file);
    set_executable_mode(&oversized);
    assert_eq!(
        compose_native_artifact(content.pack(), &artifact, &oversized, &target),
        Err(NativeArtifactError::SourceBinaryTooLarge)
    );

    let hard_link = fixture.root.join("source-hard-link");
    fs::hard_link(&fixture.source_binary, &hard_link).unwrap();
    assert_eq!(
        compose_native_artifact(content.pack(), &artifact, &fixture.source_binary, &target,),
        Err(NativeArtifactError::SourceBinaryInvalid)
    );
    fs::remove_file(hard_link).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let source_link = fixture.root.join("linked-source");
        symlink(&fixture.source_binary, &source_link).unwrap();
        assert_eq!(
            compose_native_artifact(content.pack(), &artifact, &source_link, &target),
            Err(NativeArtifactError::SourceBinaryInvalid)
        );

        fs::set_permissions(&fixture.source_binary, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            compose_native_artifact(content.pack(), &artifact, &fixture.source_binary, &target,),
            Err(NativeArtifactError::SourceBinaryInvalid)
        );
        fs::set_permissions(&fixture.source_binary, fs::Permissions::from_mode(0o700)).unwrap();
    }

    assert!(!target_path.exists());
}
