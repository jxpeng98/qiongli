#![allow(clippy::disallowed_methods)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli_platform::{
    ClientActivationTarget, GrantMode, GrantSignatureV1, InstallerKind, IntegrationScope,
    LaunchGrantV1, NativeClientPluginGrantV1, NativeReleaseAuthority, NativeReleaseCandidateError,
    NativeReleaseCandidateVerificationContext, NativeReleaseSignatureV1, ReleaseChannel,
    SignatureAlgorithm, SignedLaunchGrantV1, SignedNativeReleaseCandidateV1,
    SignedNativeReleaseEnvelopeV1, approve_native_artifact_target,
    approve_native_portable_archive_target, build_native_release_candidate,
    build_native_release_envelope, compose_native_artifact, compose_native_portable_archive,
    current_target_native_artifact_identity, launch_grant_signing_bytes, native_artifact_id,
    native_portable_archive_file_name, native_release_candidate_signing_bytes,
    native_release_envelope_signing_bytes,
};
use serde_json::json;

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const NOW: u64 = 1_750_000_000;
const SOURCE_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const NOTES: &[u8] = b"# Qiongli 2.0.0-alpha.1\n\nLite local release candidate.\n";

struct Fixture {
    root: PathBuf,
    source_binary: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-native-release-candidate-tests");
        fs::create_dir_all(&test_base).expect("release candidate test base must exist");
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
        let source_binary = root.join(format!("qiongli-source{}", std::env::consts::EXE_SUFFIX));
        fs::write(
            &source_binary,
            b"qiongli-native-release-candidate-fixture-v1",
        )
        .expect("bounded candidate fixture binary must write");
        set_executable_mode(&source_binary);
        Self {
            root,
            source_binary,
        }
    }

    fn target(&self, parent_name: &str, leaf: &str) -> PathBuf {
        let parent = self.root.join(parent_name);
        create_private_directory(&parent);
        parent.join(leaf)
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

fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(N * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn sign_grant(grant: LaunchGrantV1, key: &SigningKey, key_id: &str) -> SignedLaunchGrantV1 {
    let signature = key.sign(&launch_grant_signing_bytes(&grant).unwrap());
    SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    }
}

fn plugin_grant(
    artifact: &qiongli_platform::ArtifactIdentityV1,
    target: ClientActivationTarget,
    binary_sha256: &str,
    pack_sha256: &str,
    key: &SigningKey,
) -> NativeClientPluginGrantV1 {
    let mut plugin_artifact = artifact.clone();
    plugin_artifact.installer_kind = InstallerKind::PluginBundle;
    NativeClientPluginGrantV1 {
        target,
        signed_launch_grant: sign_grant(
            LaunchGrantV1 {
                schema_version: 1,
                generation: 23,
                artifact: plugin_artifact,
                binary_sha256: binary_sha256.to_string(),
                resource_pack_sha256: pack_sha256.to_string(),
                allowed_modes: vec![GrantMode::LiteMcp],
                integration_scopes: vec![target.integration_scope()],
                not_before_unix: NOW - 60,
                expires_at_unix: NOW + 3_600,
            },
            key,
            "candidate-launch-test-key",
        ),
    }
}

fn authority_with_policy(
    release_key: &SigningKey,
    launch_key: &SigningKey,
    channel: &str,
    minimum_release_generation: u64,
    release_key_minimum_generation: u64,
    release_key_maximum_generation_exclusive: u64,
) -> NativeReleaseAuthority {
    let document = json!({
        "schema_version": 1,
        "channel": channel,
        "minimum_release_generation": minimum_release_generation,
        "minimum_launch_grant_generation": 23,
        "release_keys": [{
            "key_id": "candidate-release-test-key",
            "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
            "minimum_generation": release_key_minimum_generation,
            "maximum_generation_exclusive": release_key_maximum_generation_exclusive
        }],
        "launch_grant_keys": [{
            "key_id": "candidate-launch-test-key",
            "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
        }]
    });
    let bytes = serde_json_canonicalizer::to_vec(&document).unwrap();
    NativeReleaseAuthority::from_json(&bytes).expect("test authority must be canonical")
}

fn authority(release_key: &SigningKey, launch_key: &SigningKey) -> NativeReleaseAuthority {
    authority_with_policy(release_key, launch_key, "alpha", 29, 29, 30)
}

fn resign_candidate(
    mut candidate: SignedNativeReleaseCandidateV1,
    release_key: &SigningKey,
) -> SignedNativeReleaseCandidateV1 {
    let signature = release_key.sign(
        &native_release_candidate_signing_bytes(&candidate.candidate)
            .expect("candidate signing bytes must rebuild"),
    );
    candidate.signature.value_hex = encode_hex(&signature.to_bytes());
    candidate
}

fn corrupt_hex(value: &mut String) {
    let replacement = if value.starts_with('0') { "1" } else { "0" };
    value.replace_range(0..1, replacement);
}

#[test]
fn signed_candidate_verifies_both_target_capabilities_and_rejects_tampering() {
    let fixture = Fixture::new("complete-candidate");
    let content = qiongli::embedded_content().expect("embedded content must verify");
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .expect("current target artifact must resolve");
    let artifact_id = native_artifact_id(&artifact).expect("artifact ID must render");
    let artifact_path = fixture.target("artifact", &artifact_id);
    let artifact_target = approve_native_artifact_target(&artifact_path, &artifact)
        .expect("artifact target must approve");
    let assembled = compose_native_artifact(
        content.pack(),
        &artifact,
        &fixture.source_binary,
        &artifact_target,
    )
    .expect("native artifact must compose");
    let archive_name =
        native_portable_archive_file_name(&artifact).expect("archive name must render");
    let archive_path = fixture.target("archive", &archive_name);
    let archive_target = approve_native_portable_archive_target(&archive_path, &artifact)
        .expect("archive target must approve");
    let archive =
        compose_native_portable_archive(content.pack(), &artifact_target, &archive_target)
            .expect("portable archive must compose");

    let launch_key = SigningKey::from_bytes(&[101_u8; 32]);
    let release_key = SigningKey::from_bytes(&[102_u8; 32]);
    let portable_grant = sign_grant(
        LaunchGrantV1 {
            schema_version: 1,
            generation: 23,
            artifact: artifact.clone(),
            binary_sha256: assembled.manifest().binary_sha256.clone(),
            resource_pack_sha256: content.pack().pack_sha256().to_string(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        },
        &launch_key,
        "candidate-launch-test-key",
    );
    let envelope =
        build_native_release_envelope(29, &archive, &portable_grant, NOW - 30, NOW + 1_800)
            .expect("portable release envelope must build");
    let release_signature =
        release_key.sign(&native_release_envelope_signing_bytes(&envelope).unwrap());
    let signed_release = SignedNativeReleaseEnvelopeV1 {
        envelope,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "candidate-release-test-key".to_string(),
            value_hex: encode_hex(&release_signature.to_bytes()),
        },
    };
    let candidate = build_native_release_candidate(
        29,
        SOURCE_COMMIT,
        &signed_release,
        [
            plugin_grant(
                &artifact,
                ClientActivationTarget::Codex,
                &assembled.manifest().binary_sha256,
                content.pack().pack_sha256(),
                &launch_key,
            ),
            plugin_grant(
                &artifact,
                ClientActivationTarget::ClaudeCode,
                &assembled.manifest().binary_sha256,
                content.pack().pack_sha256(),
                &launch_key,
            ),
        ],
        NOTES,
        NOW,
        NOW + 1_200,
    )
    .expect("release candidate must build");
    let candidate_signature =
        release_key.sign(&native_release_candidate_signing_bytes(&candidate).unwrap());
    let signed_candidate = SignedNativeReleaseCandidateV1 {
        candidate,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "candidate-release-test-key".to_string(),
            value_hex: encode_hex(&candidate_signature.to_bytes()),
        },
    };
    let authority = authority(&release_key, &launch_key);

    let codex_context = NativeReleaseCandidateVerificationContext {
        now_unix: NOW + 1,
        expected_source_commit: SOURCE_COMMIT,
        expected_artifact: &artifact,
        requested_target: ClientActivationTarget::Codex,
    };
    let codex = signed_candidate
        .verify(
            &authority,
            &codex_context,
            content.pack(),
            &archive_target,
            NOTES,
        )
        .expect("Codex candidate must verify");
    assert_eq!(codex.target(), ClientActivationTarget::Codex);
    assert_eq!(
        codex.plugin_grant().authorized_scope(),
        IntegrationScope::CodexLocal
    );
    assert_eq!(
        codex.portable_release().archive().archive_sha256(),
        archive.archive_sha256()
    );
    let claude_context = NativeReleaseCandidateVerificationContext {
        requested_target: ClientActivationTarget::ClaudeCode,
        ..codex_context
    };
    let claude = signed_candidate
        .verify(
            &authority,
            &claude_context,
            content.pack(),
            &archive_target,
            NOTES,
        )
        .expect("Claude candidate must verify");
    assert_eq!(claude.target(), ClientActivationTarget::ClaudeCode);
    assert_eq!(
        claude.plugin_grant().authorized_scope(),
        IntegrationScope::ClaudeCodeLocal
    );
    assert_eq!(
        signed_candidate
            .verify(
                &authority,
                &codex_context,
                content.pack(),
                &archive_target,
                b"tampered notes",
            )
            .unwrap_err(),
        NativeReleaseCandidateError::ReleaseNotesInvalid
    );
    let wrong_source = NativeReleaseCandidateVerificationContext {
        expected_source_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ..codex_context
    };
    assert_eq!(
        signed_candidate
            .verify(
                &authority,
                &wrong_source,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateSourceMismatch
    );

    let mut bad_signature = signed_candidate.clone();
    corrupt_hex(&mut bad_signature.signature.value_hex);
    assert_eq!(
        bad_signature
            .verify(
                &authority,
                &codex_context,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::SignatureInvalid
    );

    let not_yet_valid = NativeReleaseCandidateVerificationContext {
        now_unix: NOW - 1,
        ..codex_context
    };
    assert_eq!(
        signed_candidate
            .verify(
                &authority,
                &not_yet_valid,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateNotYetValid
    );
    let expired = NativeReleaseCandidateVerificationContext {
        now_unix: NOW + 1_200,
        ..codex_context
    };
    assert_eq!(
        signed_candidate
            .verify(&authority, &expired, content.pack(), &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateExpired
    );

    let stale_authority = authority_with_policy(&release_key, &launch_key, "alpha", 30, 29, 31);
    assert_eq!(
        signed_candidate
            .verify(
                &stale_authority,
                &codex_context,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateReplayed
    );
    let beta_authority = authority_with_policy(&release_key, &launch_key, "beta", 29, 29, 30);
    assert_eq!(
        signed_candidate
            .verify(
                &beta_authority,
                &codex_context,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateChannelMismatch
    );

    let mut untrusted = signed_candidate.clone();
    untrusted.signature.key_id = "other-release-key".to_string();
    assert_eq!(
        untrusted
            .verify(
                &authority,
                &codex_context,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::ReleaseKeyUntrusted
    );

    let mut bad_portable = signed_candidate.clone();
    corrupt_hex(
        &mut bad_portable
            .candidate
            .signed_portable_release
            .signature
            .value_hex,
    );
    let bad_portable = resign_candidate(bad_portable, &release_key);
    assert_eq!(
        bad_portable
            .verify(
                &authority,
                &codex_context,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::PortableReleaseInvalid
    );

    let mut bad_plugin = signed_candidate.clone();
    corrupt_hex(
        &mut bad_plugin.candidate.client_plugins[0]
            .signed_launch_grant
            .signature
            .value_hex,
    );
    let bad_plugin = resign_candidate(bad_plugin, &release_key);
    assert_eq!(
        bad_plugin
            .verify(
                &authority,
                &codex_context,
                content.pack(),
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::PluginGrantInvalid
    );

    let debug = format!("{codex:?}");
    assert!(!debug.contains(fixture.root.to_string_lossy().as_ref()));
    assert!(!debug.contains(&signed_candidate.signature.value_hex));
}
