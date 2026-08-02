#![allow(clippy::disallowed_methods)]

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli::FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES;
use qiongli_content::ProfileId;
use qiongli_platform::{
    ApprovalRequirement, Architecture, ArtifactIdentityV1, CapabilityProfile,
    ClaudePluginBundleError, ClaudeRegistrationExecutor, GrantMode, GrantSignatureV1,
    GrantVerificationContext, InstallPlanMetadataV1, InstallerKind, IntegrationScope,
    LaunchGrantV1, OperatingSystem, ProductId, ReleaseChannel, SignatureAlgorithm,
    SignedLaunchGrantV1, TrustedPublicKey, VerifiedLaunchGrant,
    approve_claude_plugin_bundle_target, approve_install_plan, compose_claude_plugin_bundle,
    discover_claude_user, launch_grant_signing_bytes, preview_claude_registration,
    remove_claude_plugin_bundle, verify_claude_plugin_bundle,
};
use qiongli_runtime::{FULL_PROJECT_PUBLIC_TOOL_NAMES, LITE_PUBLIC_TOOL_NAMES};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const NOW: u64 = 1_783_987_200;
const APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];
static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

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
        let test_base = native_root.join("target/qiongli-claude-plugin-tests");
        fs::create_dir_all(&test_base).expect("Claude plugin test base must exist");
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
        let config_root = root.join("config");
        let source_binary = root.join(format!("source-qiongli{}", std::env::consts::EXE_SUFFIX));
        fs::copy(env!("CARGO_BIN_EXE_qiongli"), &source_binary)
            .expect("native app binary must copy");
        set_executable_mode(&source_binary);
        Self {
            root,
            home,
            config_root,
            source_binary,
        }
    }

    fn standalone_target(&self) -> PathBuf {
        let parent = self.root.join("bundle");
        create_private_directory(&parent);
        parent.join("qiongli-next")
    }

    fn claude_source_target(&self) -> PathBuf {
        let qiongli = self.home.join(".qiongli");
        create_private_directory(&qiongli);
        let plugins = qiongli.join("plugins");
        create_private_directory(&plugins);
        let claude = plugins.join("claude-code");
        create_private_directory(&claude);
        let marketplace = claude.join("qiongli-local");
        create_private_directory(&marketplace);
        let marketplace_plugins = marketplace.join("plugins");
        create_private_directory(&marketplace_plugins);
        marketplace_plugins.join("qiongli-next")
    }

    fn claude_config_root(&self) -> PathBuf {
        let path = self.root.join("claude-config");
        create_private_directory(&path);
        path
    }

    fn direct_skills_target(&self, claude_config_root: &Path) -> PathBuf {
        let skills = claude_config_root.join("skills");
        create_private_directory(&skills);
        skills.join("qiongli-next")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct GrantFixture {
    artifact: ArtifactIdentityV1,
    binary_sha256: String,
    verified: VerifiedLaunchGrant,
    trusted: TrustedPublicKey,
}

fn grant_fixture(binary: &Path, pack_sha256: &str) -> GrantFixture {
    let binary_sha256 = sha256_file(binary);
    let artifact = ArtifactIdentityV1 {
        product: ProductId::Qiongli,
        version: env!("CARGO_PKG_VERSION").to_string(),
        channel: ReleaseChannel::Alpha,
        profile: CapabilityProfile::Lite,
        os: OperatingSystem::current().expect("test OS must be supported"),
        arch: Architecture::current().expect("test architecture must be supported"),
        installer_kind: InstallerKind::PluginBundle,
    };
    let grant = LaunchGrantV1 {
        schema_version: 1,
        generation: 11,
        artifact: artifact.clone(),
        binary_sha256: binary_sha256.clone(),
        resource_pack_sha256: pack_sha256.to_string(),
        allowed_modes: vec![GrantMode::LiteMcp, GrantMode::FullMcp],
        integration_scopes: vec![IntegrationScope::ClaudeCodeLocal],
        not_before_unix: NOW - 60,
        expires_at_unix: NOW + 3_600,
    };
    let signing_key = SigningKey::from_bytes(&[19_u8; 32]);
    let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
    let signed = SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "claude-plugin-test-key".to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    };
    let trusted = TrustedPublicKey::new(
        "claude-plugin-test-key",
        signing_key.verifying_key().to_bytes(),
    )
    .unwrap();
    let context = GrantVerificationContext {
        now_unix: NOW,
        minimum_generation: 11,
        expected_artifact: &artifact,
        binary_sha256: &binary_sha256,
        resource_pack_sha256: pack_sha256,
        requested_mode: GrantMode::FullMcp,
        requested_scope: IntegrationScope::ClaudeCodeLocal,
    };
    let verified = signed
        .verify(std::slice::from_ref(&trusted), &context)
        .expect("test launch grant must verify");
    GrantFixture {
        artifact,
        binary_sha256,
        verified,
        trusted,
    }
}

#[test]
fn complete_bundle_is_deterministic_tamper_evident_and_runtime_independent() {
    let fixture = Fixture::new("complete-bundle");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());
    let target_path = fixture.standalone_target();
    let target =
        approve_claude_plugin_bundle_target(&target_path).expect("bundle target must approve");
    let composed = compose_claude_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &target,
    )
    .expect("complete Claude bundle must compose");
    let verified = verify_claude_plugin_bundle(&target).expect("bundle must verify");
    assert_eq!(composed, verified);
    assert_eq!(verified.receipt().artifact, grant.artifact);
    assert_eq!(verified.receipt().binary_sha256, grant.binary_sha256);
    assert_eq!(verified.receipt().profile, ProfileId::MarketplaceLite);
    assert_eq!(verified.receipt().mcp_profile, ProfileId::Full);
    assert!(verified.receipt().entries.len() > 400);

    let manifest: Value =
        serde_json::from_slice(&fs::read(target_path.join(".claude-plugin/plugin.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["name"], "qiongli-next");
    assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["skills"], "./skills/");
    assert_eq!(manifest["mcpServers"], "./.mcp.json");

    let mcp_bytes = fs::read(target_path.join(".mcp.json")).unwrap();
    let mcp: Value = serde_json::from_slice(&mcp_bytes).unwrap();
    let command = mcp["mcpServers"]["qiongli-next"]["command"]
        .as_str()
        .unwrap();
    assert_eq!(
        command,
        format!("${{CLAUDE_PLUGIN_ROOT}}/{}", verified.receipt().binary_path)
    );
    assert_eq!(
        mcp["mcpServers"]["qiongli-next"]["args"],
        json!(["mcp", "serve", "--profile", "full", "--transport", "stdio"])
    );
    let lower_mcp = String::from_utf8(mcp_bytes).unwrap().to_ascii_lowercase();
    for forbidden in ["python", "node", "cargo", "npm", "rustup"] {
        assert!(!lower_mcp.contains(forbidden));
    }

    let output = run_packaged_mcp(
        &target_path,
        verified.receipt().binary_path.as_str(),
        &fixture,
    );
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty(), "{}", public_output(&output));
    let responses = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "qiongli");
    let tool_names = responses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    let expected_tool_names = LITE_PUBLIC_TOOL_NAMES
        .into_iter()
        .chain(FULL_PROJECT_PUBLIC_TOOL_NAMES)
        .chain(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES)
        .collect::<Vec<_>>();
    assert_eq!(tool_names, expected_tool_names);

    let skill = fs::read_to_string(target_path.join("skills/qiongli-workflow/SKILL.md")).unwrap();
    for required in [
        "## Claude Code Native Host Adapter",
        "qiongli_orchestration_doctor",
        "qiongli_orchestration_start",
        "qiongli_orchestration_read",
        "qiongli_orchestration_submit",
        "qiongli_orchestration_next",
        "structuredContent.qiongliOrchestration.evidence",
        "Claude Code may omit MCP `_meta`",
        "single-agent",
        "native-subagents",
        "knownFactDigests",
        "evidenceGaps",
        "reviewResult",
        "explicit artifact apply approval",
    ] {
        assert!(
            skill.contains(required),
            "missing Claude Code host guidance: {required}"
        );
    }

    assert_eq!(
        compose_claude_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        ClaudePluginBundleError::TargetExists
    );
    fs::write(target_path.join("unexpected.txt"), b"drift").unwrap();
    assert_eq!(
        verify_claude_plugin_bundle(&target).unwrap_err(),
        ClaudePluginBundleError::BundleDrift
    );
    fs::remove_file(target_path.join("unexpected.txt")).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let linked = target_path.join("linked-entry");
        symlink(&fixture.source_binary, &linked).unwrap();
        assert_eq!(
            verify_claude_plugin_bundle(&target).unwrap_err(),
            ClaudePluginBundleError::BundleDrift
        );
        fs::remove_file(linked).unwrap();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let manifest_path = target_path.join(".claude-plugin/plugin.json");
        fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            verify_claude_plugin_bundle(&target).unwrap_err(),
            ClaudePluginBundleError::BundleDrift
        );
        fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    let packaged_binary = target_path.join(&verified.receipt().binary_path);
    let outside_hard_link = fixture.root.join("managed-binary-hard-link");
    fs::hard_link(&packaged_binary, &outside_hard_link).unwrap();
    assert_eq!(
        verify_claude_plugin_bundle(&target).unwrap_err(),
        ClaudePluginBundleError::BundleDrift
    );
    fs::remove_file(outside_hard_link).unwrap();

    let receipt_path = target_path.join(".qiongli-claude-plugin-bundle.json");
    let receipt_bytes = fs::read(&receipt_path).unwrap();
    fs::write(&receipt_path, b"{}").unwrap();
    assert_eq!(
        verify_claude_plugin_bundle(&target).unwrap_err(),
        ClaudePluginBundleError::ReceiptInvalid
    );
    fs::write(&receipt_path, receipt_bytes).unwrap();

    let skill_path = target_path.join("skills/qiongli-workflow/SKILL.md");
    let skill_bytes = fs::read(&skill_path).unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(&skill_path)
        .unwrap()
        .write_all(b"content-tamper")
        .unwrap();
    assert_eq!(
        verify_claude_plugin_bundle(&target).unwrap_err(),
        ClaudePluginBundleError::BundleDrift
    );
    fs::write(&skill_path, skill_bytes).unwrap();

    fs::OpenOptions::new()
        .append(true)
        .open(&packaged_binary)
        .unwrap()
        .write_all(b"tamper")
        .unwrap();
    assert_eq!(
        verify_claude_plugin_bundle(&target).unwrap_err(),
        ClaudePluginBundleError::BundleDrift
    );
}

#[test]
fn composition_conflicts_fail_closed_without_overwriting_existing_data() {
    let fixture = Fixture::new("composition-conflicts");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());

    let invalid_parent = fixture.root.join("invalid-bundle");
    create_private_directory(&invalid_parent);
    let invalid_target = approve_claude_plugin_bundle_target(invalid_parent.join("not-qiongli"))
        .expect("portable verification target must approve");
    assert_eq!(
        compose_claude_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &invalid_target,
        )
        .unwrap_err(),
        ClaudePluginBundleError::InvalidTarget
    );

    let target_path = fixture.standalone_target();
    let target = approve_claude_plugin_bundle_target(&target_path).unwrap();
    create_private_directory(&target_path);
    fs::write(target_path.join("user-canary"), b"preserve").unwrap();
    assert_eq!(
        compose_claude_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        ClaudePluginBundleError::TargetExists
    );
    assert_eq!(
        fs::read(target_path.join("user-canary")).unwrap(),
        b"preserve"
    );
    fs::remove_dir_all(&target_path).unwrap();

    let lock_path = target_path
        .parent()
        .unwrap()
        .join(".qiongli.qiongli-claude-bundle.lock");
    fs::write(&lock_path, b"held").unwrap();
    assert_eq!(
        compose_claude_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        ClaudePluginBundleError::TargetBusy
    );
    fs::remove_file(lock_path).unwrap();

    let mismatched_pack = "0000000000000000000000000000000000000000000000000000000000000000";
    let pack_mismatch_grant = grant_fixture(&fixture.source_binary, mismatched_pack);
    assert_eq!(
        compose_claude_plugin_bundle(
            content.pack(),
            &pack_mismatch_grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        ClaudePluginBundleError::ResourcePackMismatch
    );

    let oversized_binary = fixture.root.join("oversized-qiongli");
    let oversized = fs::File::create(&oversized_binary).unwrap();
    oversized.set_len(128 * 1024 * 1024 + 1).unwrap();
    drop(oversized);
    set_executable_mode(&oversized_binary);
    assert_eq!(
        compose_claude_plugin_bundle(content.pack(), &grant.verified, &oversized_binary, &target,)
            .unwrap_err(),
        ClaudePluginBundleError::SourceBinaryTooLarge
    );

    fs::OpenOptions::new()
        .append(true)
        .open(&fixture.source_binary)
        .unwrap()
        .write_all(b"changed-after-signing")
        .unwrap();
    assert_eq!(
        compose_claude_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        ClaudePluginBundleError::BinaryDigestMismatch
    );
    assert!(!target_path.exists());
}

#[test]
fn exact_removal_deletes_only_a_verified_bundle_and_preserves_drift() {
    let fixture = Fixture::new("exact-removal");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());
    let target_path = fixture.standalone_target();
    let target = approve_claude_plugin_bundle_target(&target_path).unwrap();
    let composed = compose_claude_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &target,
    )
    .unwrap();
    let canary = target_path.parent().unwrap().join("user-canary");
    fs::write(&canary, b"preserve").unwrap();

    let removed = remove_claude_plugin_bundle(&target).unwrap();
    assert_eq!(removed, composed);
    assert!(!target_path.exists());
    assert_eq!(fs::read(&canary).unwrap(), b"preserve");
    assert!(
        fs::read_dir(target_path.parent().unwrap())
            .unwrap()
            .flatten()
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .contains("qiongli-claude-remove"))
    );

    compose_claude_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &target,
    )
    .unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(target_path.join("skills/qiongli-workflow/SKILL.md"))
        .unwrap()
        .write_all(b"drift")
        .unwrap();
    assert_eq!(
        remove_claude_plugin_bundle(&target).unwrap_err(),
        ClaudePluginBundleError::BundleDrift
    );
    assert!(target_path.exists());
    assert_eq!(fs::read(canary).unwrap(), b"preserve");
}

#[test]
#[ignore = "requires the Claude Code CLI"]
fn real_claude_clean_client_discovers_and_installs_both_local_forms() {
    let fixture = Fixture::new("real-claude-client");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());

    let claude_config_root = fixture.claude_config_root();
    let direct_path = fixture.direct_skills_target(&claude_config_root);
    let direct_target = approve_claude_plugin_bundle_target(&direct_path)
        .expect("direct Claude skills target must approve");
    let direct_bundle = compose_claude_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &direct_target,
    )
    .expect("direct Claude skills bundle must compose");

    let claude = std::env::var_os("QIONGLI_CLAUDE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("claude"));
    let version = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .arg("--version")
        .output()
        .expect("Claude version command must start");
    assert!(version.status.success(), "{}", public_output(&version));

    let direct_validation = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "validate", "--strict"])
        .arg(&direct_path)
        .output()
        .expect("Claude strict plugin validation must start");
    assert!(
        direct_validation.status.success(),
        "{}",
        public_output(&direct_validation)
    );

    let direct_list = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "list", "--json"])
        .output()
        .expect("Claude direct plugin list must start");
    assert!(
        direct_list.status.success(),
        "{}",
        public_output(&direct_list)
    );
    let direct_list_text = String::from_utf8_lossy(&direct_list.stdout);
    assert!(
        direct_list_text.contains("qiongli-next@skills-dir"),
        "{}",
        public_output(&direct_list)
    );

    let direct_mcp = run_packaged_mcp(
        &direct_path,
        direct_bundle.receipt().binary_path.as_str(),
        &fixture,
    );
    assert!(
        direct_mcp.status.success(),
        "{}",
        public_output(&direct_mcp)
    );
    assert!(
        direct_mcp.stderr.is_empty(),
        "{}",
        public_output(&direct_mcp)
    );

    let source_path = fixture.claude_source_target();
    let bundle_target = approve_claude_plugin_bundle_target(&source_path)
        .expect("Claude marketplace source target must approve");
    let marketplace_bundle = compose_claude_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &bundle_target,
    )
    .expect("Claude marketplace source bundle must compose");
    let marketplace_root = source_path
        .parent()
        .and_then(Path::parent)
        .expect("marketplace source must have a root");
    let discovered = discover_claude_user(&fixture.home).expect("Claude target must discover");
    let executor = ClaudeRegistrationExecutor::new(discovered.clone());
    let preview = preview_claude_registration(
        &discovered,
        InstallPlanMetadataV1 {
            plan_id: "r3e-real-claude-client".to_string(),
            created_at_unix: NOW,
            expires_at_unix: NOW + 600,
        },
        &grant.verified,
    )
    .expect("Claude marketplace registration must preview");
    let context = GrantVerificationContext {
        now_unix: NOW,
        minimum_generation: 11,
        expected_artifact: &grant.artifact,
        binary_sha256: &grant.binary_sha256,
        resource_pack_sha256: content.pack().pack_sha256(),
        requested_mode: GrantMode::FullMcp,
        requested_scope: IntegrationScope::ClaudeCodeLocal,
    };
    let verified_plan = preview
        .plan
        .verify(std::slice::from_ref(&grant.trusted), &context)
        .expect("Claude marketplace registration plan must verify");
    let approval = approve_install_plan(&verified_plan, &APPROVALS, NOW)
        .expect("Claude marketplace registration must approve");
    let registration = executor
        .apply(&verified_plan, &approval, NOW + 1)
        .expect("Claude local marketplace catalog must register");
    assert_eq!(executor.verify().unwrap().receipt, registration.receipt);

    let marketplace_validation = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "validate", "--strict"])
        .arg(marketplace_root)
        .output()
        .expect("Claude marketplace validation must start");
    assert!(
        marketplace_validation.status.success(),
        "{}",
        public_output(&marketplace_validation)
    );

    let add_marketplace = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "marketplace", "add"])
        .arg(marketplace_root)
        .args(["--scope", "user"])
        .output()
        .expect("Claude marketplace add must start");
    assert!(
        add_marketplace.status.success(),
        "{}",
        public_output(&add_marketplace)
    );

    let install = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args([
            "plugin",
            "install",
            "qiongli-next@qiongli-local",
            "--scope",
            "user",
        ])
        .output()
        .expect("Claude marketplace plugin install must start");
    assert!(install.status.success(), "{}", public_output(&install));

    let listed = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "list", "--json"])
        .output()
        .expect("Claude plugin list must start");
    assert!(listed.status.success(), "{}", public_output(&listed));
    assert!(String::from_utf8_lossy(&listed.stdout).contains("qiongli-next@qiongli-local"));

    let cached_root = find_cached_bundle(&claude_config_root.join("plugins/cache"))
        .expect("Claude must cache the Qiongli plugin");
    let cached_target =
        approve_claude_plugin_bundle_target(&cached_root).expect("cache must approve");
    let cached = verify_claude_plugin_bundle(&cached_target).expect("cached bundle must verify");
    assert_eq!(cached.receipt_sha256(), marketplace_bundle.receipt_sha256());
    let mcp = run_packaged_mcp(
        &cached_root,
        cached.receipt().binary_path.as_str(),
        &fixture,
    );
    assert!(mcp.status.success(), "{}", public_output(&mcp));
    assert!(mcp.stderr.is_empty(), "{}", public_output(&mcp));
    let responses = String::from_utf8(mcp.stdout).unwrap();
    assert!(responses.contains("\"serverInfo\""));
    assert!(responses.contains("\"qiongli_task_plan\""));

    let uninstall = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args([
            "plugin",
            "uninstall",
            "qiongli-next@qiongli-local",
            "--scope",
            "user",
        ])
        .output()
        .expect("Claude plugin uninstall must start");
    assert!(uninstall.status.success(), "{}", public_output(&uninstall));
    let remove_marketplace = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "marketplace", "remove", "qiongli-local"])
        .output()
        .expect("Claude marketplace remove must start");
    assert!(
        remove_marketplace.status.success(),
        "{}",
        public_output(&remove_marketplace)
    );
    let after = isolated_claude_command(&claude, &fixture, &claude_config_root)
        .args(["plugin", "list", "--json"])
        .output()
        .expect("Claude final plugin list must start");
    assert!(after.status.success(), "{}", public_output(&after));
    assert!(!String::from_utf8_lossy(&after.stdout).contains("qiongli-next@qiongli-local"));
    executor
        .remove(NOW + 2)
        .expect("Qiongli marketplace catalog entry must remove");

    let version_text = String::from_utf8_lossy(&version.stdout).trim().to_string();
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schema_version": 1,
            "evidence": "isolated-claude-native-plugin",
            "claude_cli": version_text,
            "direct_bundle_receipt_sha256": direct_bundle.receipt_sha256(),
            "marketplace_bundle_receipt_sha256": marketplace_bundle.receipt_sha256(),
            "strict_plugin_validation": true,
            "skills_directory_discovered": true,
            "local_marketplace_added": true,
            "marketplace_catalog_receipted": true,
            "marketplace_install_succeeded": true,
            "marketplace_remove_succeeded": true,
            "client_cache_verified": true,
            "cached_mcp_empty_path_succeeded": true,
            "full_tool_count": LITE_PUBLIC_TOOL_NAMES.len()
                + FULL_PROJECT_PUBLIC_TOOL_NAMES.len()
                + FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES.len()
        }))
        .unwrap()
    );
}

fn run_packaged_mcp(root: &Path, binary_path: &str, fixture: &Fixture) -> Output {
    let executable = root.join(binary_path);
    let mut child = Command::new(executable)
        .current_dir(root)
        .env("PATH", "")
        .env("QIONGLI_CONFIG_HOME", &fixture.config_root)
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home)
        .args(["mcp", "serve", "--profile", "full", "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("packaged MCP executable must start without PATH");
    let requests = [
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-11-25", "capabilities": {}}
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    ];
    {
        let stdin = child.stdin.as_mut().expect("MCP stdin must be piped");
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("packaged MCP must exit")
}

fn isolated_claude_command(claude: &Path, fixture: &Fixture, config_root: &Path) -> Command {
    let mut command = Command::new(claude);
    command
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home)
        .env("CLAUDE_CONFIG_DIR", config_root)
        .env("NO_COLOR", "1");
    command
}

fn find_cached_bundle(root: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).ok()?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if path.join(".qiongli-claude-plugin-bundle.json").is_file() {
                return Some(path);
            }
            if let Some(found) = find_cached_bundle(&path) {
                return Some(found);
            }
        }
    }
    None
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).expect("test binary must read");
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn public_output(output: &Output) -> String {
    format!(
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path).expect("private directory must create");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path)
        .expect("owner-only Windows directory must create");
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("test binary must be executable");
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}
