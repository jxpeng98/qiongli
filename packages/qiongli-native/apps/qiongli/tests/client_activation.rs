#![allow(clippy::disallowed_methods)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli_content::{
    BuiltResourcePack, CompatibleProduct, LoadedResourcePack, ResourcePackBuildMetadata,
    build_resource_pack, collect_canonical_sources, load_resource_pack,
};
use qiongli_platform::{
    ApprovalRequirement, Architecture, ArtifactIdentityV1, CapabilityProfile,
    ClientActivationCoordinator, ClientActivationDisposition, ClientActivationEffect,
    ClientActivationError, ClientActivationLifecycleDisposition, ClientActivationState,
    ClientActivationTarget, ClientComponentState, ClientInventoryInput, GrantSignatureV1,
    GrantVerificationContext, InstallPlanMetadataV1, InstallerKind, LaunchGrantV1, OperatingSystem,
    ProductId, ReleaseChannel, SignatureAlgorithm, SignedLaunchGrantV1, TrustedPublicKey,
    VerifiedLaunchGrant, approve_claude_plugin_bundle_target, approve_codex_plugin_bundle_target,
    approve_install_plan, compose_claude_plugin_bundle, compose_codex_plugin_bundle,
    discover_client_activation, discover_client_inventory, launch_grant_signing_bytes,
    preview_client_activation,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const NOW: u64 = 1_783_987_200;
const GENERATION: u64 = 17;
const APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];
const PRIVATE_PATH_CANARY: &str = "client-activation-private-path-canary";
const TEST_BINARY_BYTES: &[u8] = b"qiongli native client activation fixture\n";
static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static PACK: OnceLock<BuiltResourcePack> = OnceLock::new();

struct Fixture {
    root: PathBuf,
    home: PathBuf,
    source_binary: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-client-activation-tests");
        fs::create_dir_all(&test_base).expect("activation test base must exist");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "{PRIVATE_PATH_CANARY}-{name}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        create_private_directory(&root);
        let home = root.join("home");
        create_private_directory(&home);
        let source_binary = root.join(format!("qiongli-source{}", std::env::consts::EXE_SUFFIX));
        fs::write(&source_binary, TEST_BINARY_BYTES).expect("fixture binary must write");
        set_executable_mode(&source_binary);
        Self {
            root,
            home,
            source_binary,
        }
    }

    fn source_target(&self, target: ClientActivationTarget) -> PathBuf {
        let qiongli = self.home.join(".qiongli");
        ensure_private_directory(&qiongli);
        let plugins = qiongli.join("plugins");
        ensure_private_directory(&plugins);
        match target {
            ClientActivationTarget::Codex => {
                let codex = plugins.join("codex");
                ensure_private_directory(&codex);
                codex.join("qiongli-next")
            }
            ClientActivationTarget::ClaudeCode => {
                let claude = plugins.join("claude-code");
                ensure_private_directory(&claude);
                let marketplace = claude.join("qiongli-local");
                ensure_private_directory(&marketplace);
                let marketplace_plugins = marketplace.join("plugins");
                ensure_private_directory(&marketplace_plugins);
                marketplace_plugins.join("qiongli-next")
            }
        }
    }

    fn marketplace_path(&self, target: ClientActivationTarget) -> PathBuf {
        match target {
            ClientActivationTarget::Codex => self.home.join(".agents/plugins/marketplace.json"),
            ClientActivationTarget::ClaudeCode => self
                .home
                .join(".qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json"),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct GrantFixture {
    verified: VerifiedLaunchGrant,
    trusted: TrustedPublicKey,
}

fn grant_fixture(binary: &Path, pack_sha256: &str, target: ClientActivationTarget) -> GrantFixture {
    let binary_sha256 = sha256_file(binary);
    let artifact = ArtifactIdentityV1 {
        product: ProductId::Qiongli,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        channel: ReleaseChannel::Alpha,
        profile: CapabilityProfile::Lite,
        os: OperatingSystem::current().expect("test OS must be supported"),
        arch: Architecture::current().expect("test architecture must be supported"),
        installer_kind: InstallerKind::PluginBundle,
    };
    let scope = target.integration_scope();
    let grant = LaunchGrantV1 {
        schema_version: 1,
        generation: GENERATION,
        artifact: artifact.clone(),
        binary_sha256: binary_sha256.clone(),
        resource_pack_sha256: pack_sha256.to_owned(),
        allowed_modes: target.allowed_grant_modes().to_vec(),
        integration_scopes: vec![scope],
        not_before_unix: NOW - 60,
        expires_at_unix: NOW + 3_600,
    };
    let signing_key = SigningKey::from_bytes(&[29_u8; 32]);
    let key_id = match target {
        ClientActivationTarget::Codex => "client-activation-codex-test-key",
        ClientActivationTarget::ClaudeCode => "client-activation-claude-test-key",
    };
    let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
    let signed = SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_owned(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    };
    let trusted = TrustedPublicKey::new(key_id, signing_key.verifying_key().to_bytes()).unwrap();
    let context = GrantVerificationContext {
        now_unix: NOW,
        minimum_generation: GENERATION,
        expected_artifact: &artifact,
        binary_sha256: &binary_sha256,
        resource_pack_sha256: pack_sha256,
        requested_mode: target.required_grant_mode(),
        requested_scope: scope,
    };
    let verified = signed
        .verify(std::slice::from_ref(&trusted), &context)
        .expect("activation test grant must verify");
    GrantFixture { verified, trusted }
}

fn compose_source(
    fixture: &Fixture,
    pack: &LoadedResourcePack<'_>,
    target: ClientActivationTarget,
    grant: &VerifiedLaunchGrant,
) {
    let path = fixture.source_target(target);
    match target {
        ClientActivationTarget::Codex => {
            let approved = approve_codex_plugin_bundle_target(&path).unwrap();
            compose_codex_plugin_bundle(pack, grant, &fixture.source_binary, &approved)
                .expect("Codex activation source must compose");
        }
        ClientActivationTarget::ClaudeCode => {
            let approved = approve_claude_plugin_bundle_target(&path).unwrap();
            compose_claude_plugin_bundle(pack, grant, &fixture.source_binary, &approved)
                .expect("Claude activation source must compose");
        }
    }
}

fn preview(
    handle: &qiongli_platform::ClientActivationHandle,
    grant: &GrantFixture,
    target: ClientActivationTarget,
) -> qiongli_platform::ClientActivationPreview {
    preview_client_activation(
        handle,
        InstallPlanMetadataV1 {
            plan_id: match target {
                ClientActivationTarget::Codex => "r3l-codex-activation-test",
                ClientActivationTarget::ClaudeCode => "r3l-claude-activation-test",
            }
            .to_owned(),
            created_at_unix: NOW,
            expires_at_unix: NOW + 600,
        },
        &grant.verified,
        std::slice::from_ref(&grant.trusted),
        GENERATION,
        NOW,
    )
    .expect("unified activation must preview")
}

fn remove_owned_marketplace_entry(path: &Path) {
    let mut document: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let plugins = document["plugins"]
        .as_array_mut()
        .expect("marketplace plugins must be an array");
    let before = plugins.len();
    plugins.retain(|entry| entry["name"] != "qiongli-next");
    assert_eq!(plugins.len() + 1, before);
    let mut bytes = serde_json::to_vec_pretty(&document).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}

fn exercise_activation(target: ClientActivationTarget, lifecycle: Lifecycle) {
    let fixture = Fixture::new(match target {
        ClientActivationTarget::Codex => "codex",
        ClientActivationTarget::ClaudeCode => "claude",
    });
    let pack = test_pack();
    let grant = grant_fixture(&fixture.source_binary, pack.pack_sha256(), target);
    compose_source(&fixture, pack, target, &grant.verified);
    let inventory = discover_client_inventory(ClientInventoryInput::new(&fixture.home));
    let client = &inventory.summary().clients[match target {
        ClientActivationTarget::Codex => 0,
        ClientActivationTarget::ClaudeCode => 1,
    }];
    assert_eq!(client.components.full_mcp, ClientComponentState::Ready);

    let handle = discover_client_activation(&fixture.home, None, target)
        .expect("activation target must discover");
    assert_eq!(handle.discovery().source, ClientActivationState::Ready);
    assert_eq!(
        handle.discovery().registration,
        ClientActivationState::Missing
    );
    let debug = format!("{handle:?}");
    assert!(!debug.contains(PRIVATE_PATH_CANARY));
    assert!(!debug.contains(fixture.home.to_string_lossy().as_ref()));

    let preview = preview(&handle, &grant, target);
    assert_eq!(preview.target(), target);
    assert_eq!(preview.effect(), ClientActivationEffect::Activate);
    assert_eq!(preview.plan().plan().approvals_required, APPROVALS);
    let approval = approve_install_plan(preview.plan(), &APPROVALS, NOW).unwrap();
    let independently_discovered = discover_client_activation(&fixture.home, None, target).unwrap();
    assert_eq!(
        ClientActivationCoordinator::new(independently_discovered)
            .apply(&preview, &approval, NOW + 1)
            .unwrap_err(),
        ClientActivationError::TargetMismatch
    );
    assert_eq!(
        discover_client_activation(&fixture.home, None, target)
            .unwrap()
            .discovery()
            .registration,
        ClientActivationState::Missing
    );
    let coordinator = ClientActivationCoordinator::new(handle);

    let applied = coordinator.apply(&preview, &approval, NOW + 1).unwrap();
    assert_eq!(applied.target, target);
    assert_eq!(applied.disposition, ClientActivationDisposition::Activated);
    assert_eq!(
        applied.plan_digest_sha256,
        preview.plan().plan().semantic_digest_sha256
    );
    assert_eq!(
        coordinator
            .apply(&preview, &approval, NOW + 2)
            .unwrap()
            .disposition,
        ClientActivationDisposition::AlreadyActive
    );
    assert_eq!(
        coordinator.verify().unwrap().plan_digest_sha256,
        applied.plan_digest_sha256
    );

    remove_owned_marketplace_entry(&fixture.marketplace_path(target));
    assert!(coordinator.verify().is_err());
    assert_eq!(
        coordinator
            .repair(&preview, &approval, NOW + 3)
            .unwrap()
            .disposition,
        ClientActivationDisposition::Repaired
    );
    assert!(coordinator.verify().is_ok());

    let first = match lifecycle {
        Lifecycle::Remove => coordinator.remove(NOW + 4).unwrap(),
        Lifecycle::Rollback => coordinator.rollback(NOW + 4).unwrap(),
    };
    let second = match lifecycle {
        Lifecycle::Remove => coordinator.remove(NOW + 5).unwrap(),
        Lifecycle::Rollback => coordinator.rollback(NOW + 5).unwrap(),
    };
    assert_eq!(
        first.disposition,
        match lifecycle {
            Lifecycle::Remove => ClientActivationLifecycleDisposition::Removed,
            Lifecycle::Rollback => ClientActivationLifecycleDisposition::RolledBack,
        }
    );
    assert_eq!(
        second.disposition,
        match lifecycle {
            Lifecycle::Remove => ClientActivationLifecycleDisposition::AlreadyRemoved,
            Lifecycle::Rollback => ClientActivationLifecycleDisposition::AlreadyRolledBack,
        }
    );
}

#[derive(Clone, Copy)]
enum Lifecycle {
    Remove,
    Rollback,
}

#[test]
fn codex_coordinator_applies_repairs_removes_and_replays() {
    exercise_activation(ClientActivationTarget::Codex, Lifecycle::Remove);
}

#[test]
fn claude_coordinator_applies_repairs_rolls_back_and_replays() {
    exercise_activation(ClientActivationTarget::ClaudeCode, Lifecycle::Rollback);
}

#[test]
fn coordinator_rejects_a_grant_for_the_other_target_without_path_disclosure() {
    let fixture = Fixture::new("wrong-target");
    let pack = test_pack();
    let codex_grant = grant_fixture(
        &fixture.source_binary,
        pack.pack_sha256(),
        ClientActivationTarget::Codex,
    );
    let claude =
        discover_client_activation(&fixture.home, None, ClientActivationTarget::ClaudeCode)
            .unwrap();
    let error = preview_client_activation(
        &claude,
        InstallPlanMetadataV1 {
            plan_id: "r3l-wrong-target-test".to_owned(),
            created_at_unix: NOW,
            expires_at_unix: NOW + 600,
        },
        &codex_grant.verified,
        std::slice::from_ref(&codex_grant.trusted),
        GENERATION,
        NOW,
    )
    .unwrap_err();
    assert_eq!(error, ClientActivationError::TargetMismatch);
    let rendered = format!("{error:?} {error}");
    assert!(!rendered.contains(PRIVATE_PATH_CANARY));
    assert!(!rendered.contains(fixture.home.to_string_lossy().as_ref()));
}

fn test_pack() -> &'static LoadedResourcePack<'static> {
    static LOADED: OnceLock<LoadedResourcePack<'static>> = OnceLock::new();
    let built = PACK.get_or_init(|| {
        const DIRECTORIES: [&str; 12] = [
            ".claude-plugin",
            ".codex-plugin",
            "distribution",
            "mcp-contracts",
            "roles",
            "schemas",
            "skills",
            "standards",
            "subjects",
            "templates",
            "venue-profiles",
            "workflow",
        ];
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qiongli-client-activation-pack-source");
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&source).unwrap();
        for directory in DIRECTORIES {
            fs::create_dir(source.join(directory)).unwrap();
            match directory {
                ".claude-plugin" => fs::write(
                    source.join(".claude-plugin/plugin.json"),
                    format!(
                        r#"{{"name":"qiongli","version":"{}","skills":"./skills/","mcpServers":"./.mcp.json"}}"#,
                        env!("CARGO_PKG_VERSION")
                    ),
                )
                .unwrap(),
                ".codex-plugin" => fs::write(
                    source.join(".codex-plugin/plugin.json"),
                    format!(
                        r#"{{"name":"qiongli","version":"{}","skills":"./"}}"#,
                        env!("CARGO_PKG_VERSION")
                    ),
                )
                .unwrap(),
                "workflow" => fs::write(
                    source.join("workflow/SKILL.md"),
                    b"---\nname: qiongli\ndescription: activation test\n---\n\n# Qiongli Academic Workflow\n",
                )
                .unwrap(),
                _ => fs::write(
                    source.join(directory).join("entry.txt"),
                    directory.as_bytes(),
                )
                .unwrap(),
            }
        }
        fs::write(source.join("skills-core.md"), b"core\n").unwrap();
        fs::write(source.join("skills-summary.md"), b"summary\n").unwrap();
        let resources = collect_canonical_sources(&source).unwrap();
        build_resource_pack(
            &ResourcePackBuildMetadata {
                pack_id: "qiongli-core".to_owned(),
                content_version: "1.19.0-beta.1".to_owned(),
                source_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
                compatible_product: CompatibleProduct {
                    minimum: "2.0.0-alpha.1".to_owned(),
                    maximum_exclusive: "3.0.0".to_owned(),
                },
            },
            &resources,
        )
        .inspect(|_| {
            let _ = fs::remove_dir_all(&source);
        })
        .unwrap()
    });
    LOADED.get_or_init(|| load_resource_pack(built.core_bytes(), built.pack_sha256()).unwrap())
}

fn sha256_file(path: &Path) -> String {
    encode_hex(&Sha256::digest(fs::read(path).unwrap()))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn ensure_private_directory(path: &Path) {
    if !path.exists() {
        create_private_directory(path);
    }
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path).unwrap();
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(path: &Path) {
    fs::create_dir(path).unwrap();
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}
