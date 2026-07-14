#![allow(clippy::disallowed_methods)]

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli_platform::{
    AllowedRootV1, ApprovalRequirement, CapabilityProfile, GrantMode, GrantSignatureV1,
    GrantVerificationContext, InstallDisposition, InstallPlanMetadataV1, InstallScope,
    IntegrationScope, LaunchGrantV1, LocalSurface, LocalTargetFamily, ManagedNativePayloadExecutor,
    NativeArtifactManifestV1, NativePortableArchiveError, ReleaseChannel, SignatureAlgorithm,
    SignedLaunchGrantV1, SymbolicRoot, TargetDescriptorV1, TrustedPublicKey, approve_install_plan,
    approve_managed_root, approve_native_artifact_target, approve_native_portable_archive_target,
    compose_native_artifact, compose_native_portable_archive,
    current_target_native_artifact_identity, extract_native_portable_archive,
    launch_grant_signing_bytes, native_artifact_id, native_payload_install_id,
    native_portable_archive_file_name, preview_native_payload_install, verify_native_artifact,
    verify_native_portable_archive,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use serde_json::{Value, json};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const PRIVATE_PATH_CANARY: &str = "native-portable-archive-private-path-canary";
const NOW: u64 = 1_750_000_000;

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

fn assert_runtime(fixture: &Fixture, executable: &Path, expected: &NativeArtifactManifestV1) {
    let version = fixture
        .command(executable)
        .arg("--version")
        .output()
        .expect("isolated CLI must start without PATH");
    assert!(version.status.success(), "{}", public_output(&version));
    assert_eq!(
        version.stdout,
        format!("qiongli {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(version.stderr.is_empty());

    let listed = fixture
        .command(executable)
        .args(["content", "list"])
        .output()
        .expect("isolated content command must start without PATH");
    assert!(listed.status.success(), "{}", public_output(&listed));
    assert!(listed.stderr.is_empty());
    let listed_json: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed_json["pack_id"], expected.content.pack_id);
    assert_eq!(listed_json["pack_sha256"], expected.content.pack_sha256);
    assert_eq!(
        listed_json["content_root_sha256"],
        expected.content.content_root_sha256
    );

    let mut child = fixture
        .command(executable)
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
        .expect("isolated MCP must start without PATH");
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
        let stdin = child.stdin.as_mut().expect("isolated MCP stdin must exist");
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    let mcp = child
        .wait_with_output()
        .expect("isolated MCP must exit on EOF");
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
}

fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(N * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
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

    let grant = LaunchGrantV1 {
        schema_version: 1,
        generation: 13,
        artifact: artifact.clone(),
        binary_sha256: first.payload().manifest().binary_sha256.clone(),
        resource_pack_sha256: first.payload().manifest().content.pack_sha256.clone(),
        allowed_modes: vec![GrantMode::LiteMcp],
        integration_scopes: vec![IntegrationScope::CodexLocal],
        not_before_unix: NOW - 60,
        expires_at_unix: NOW + 3_600,
    };
    let signing_key = SigningKey::from_bytes(&[73_u8; 32]);
    let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
    let signed = SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "installed-runtime-test-key".to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    };
    let trusted = TrustedPublicKey::new(
        "installed-runtime-test-key",
        signing_key.verifying_key().to_bytes(),
    )
    .unwrap();
    let context = GrantVerificationContext {
        now_unix: NOW,
        minimum_generation: 13,
        expected_artifact: &artifact,
        binary_sha256: &first.payload().manifest().binary_sha256,
        resource_pack_sha256: &first.payload().manifest().content.pack_sha256,
        requested_mode: GrantMode::LiteMcp,
        requested_scope: IntegrationScope::CodexLocal,
    };
    let verified_grant = signed
        .verify(std::slice::from_ref(&trusted), &context)
        .expect("test-signed installed-runtime grant must verify");
    let managed_root = fixture.root.join("installed-managed");
    create_private_directory(&managed_root);
    let root = AllowedRootV1 {
        id: "qiongli-data".to_string(),
        root: SymbolicRoot::QiongliManagedData,
    };
    let approved_root =
        approve_managed_root(&root, &managed_root).expect("installed root must approve");
    let plan = preview_native_payload_install(
        InstallPlanMetadataV1 {
            plan_id: "installed-runtime-plan".to_string(),
            created_at_unix: NOW,
            expires_at_unix: NOW + 600,
        },
        &verified_grant,
        &first,
        TargetDescriptorV1 {
            family: LocalTargetFamily::CodexLocal,
            surface: LocalSurface::CliLocal,
            scope: InstallScope::User,
            profile: CapabilityProfile::Lite,
            os: artifact.os,
            arch: artifact.arch,
            adapter_version: 1,
        },
        root,
    )
    .expect("native payload install plan must preview");
    let verified_plan = plan
        .verify(std::slice::from_ref(&trusted), &context)
        .expect("native payload install plan must verify");
    let approval =
        approve_install_plan(&verified_plan, &[ApprovalRequirement::FilesystemWrite], NOW)
            .expect("native payload install plan must approve");
    let install_id = native_payload_install_id(&first);
    let executor = ManagedNativePayloadExecutor::new(approved_root);
    let applied = executor
        .apply(
            &verified_plan,
            &approval,
            content.pack(),
            &first_target,
            NOW + 1,
        )
        .expect("verified archive must install");
    assert_eq!(applied.disposition, InstallDisposition::Applied);
    assert_eq!(
        executor
            .verify(&install_id, content.pack())
            .unwrap()
            .receipt,
        applied.receipt
    );
    let state_bytes = fs::read(managed_root.join(format!(".qiongli-{install_id}.json"))).unwrap();
    assert!(
        !String::from_utf8_lossy(&state_bytes).contains(fixture.root.to_string_lossy().as_ref())
    );
    let installed_path = managed_root.join(&artifact_id);
    let installed_target = approve_native_artifact_target(&installed_path, &artifact)
        .expect("installed artifact target must approve");
    assert_eq!(
        verify_native_artifact(content.pack(), &installed_target).unwrap(),
        source
    );
    assert_eq!(
        extract_native_portable_archive(content.pack(), &first_target, &installed_target),
        Err(NativePortableArchiveError::DestinationExists)
    );
    let installed_binary = installed_path.join(&first.payload().manifest().binary_path);
    assert_runtime(&fixture, &installed_binary, first.payload().manifest());

    assert_eq!(
        compose_native_portable_archive(content.pack(), &source_target, &first_target),
        Err(NativePortableArchiveError::TargetExists)
    );
    assert_eq!(fs::read(&first_path).unwrap(), original_bytes);

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
