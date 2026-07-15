#![allow(clippy::disallowed_methods)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli_platform::{
    ClientActivationTarget, GrantMode, GrantSignatureV1, InstallerKind, IntegrationScope,
    LaunchGrantV1, NativeClientPluginGrantV1, NativeReleaseAuthority, NativeReleaseSignatureV1,
    ReleaseChannel, SignatureAlgorithm, SignedLaunchGrantV1, SignedNativeReleaseCandidateV1,
    SignedNativeReleaseEnvelopeV1, approve_native_artifact_target,
    approve_native_portable_archive_target, build_native_release_candidate,
    build_native_release_envelope, compose_native_artifact, compose_native_portable_archive,
    current_target_native_artifact_identity, extract_native_portable_archive,
    launch_grant_signing_bytes, native_artifact_binary_path, native_artifact_id,
    native_portable_archive_file_name, native_release_candidate_file_name,
    native_release_candidate_signing_bytes, native_release_envelope_signing_bytes,
    native_release_notes_file_name,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use serde_json::{Value, json};

const RELEASE_KEY_ID: &str = "alpha1-acceptance-release-key";
const LAUNCH_KEY_ID: &str = "alpha1-acceptance-launch-key";
const GENERATION: u64 = 1;
const RELEASE_VALIDITY_SECONDS: u64 = 3_600;
const CANDIDATE_VALIDITY_SECONDS: u64 = 1_800;
const NOTES: &[u8] = b"# Qiongli 2.0.0-alpha.1 acceptance candidate\n\n\
Test-signed, assembled-unpublished current-target Lite candidate. Supports the native CLI, UI \
startup preflight, embedded skills, Lite MCP, and receipt-backed Codex local and Claude Code \
local source registration. Client install, enablement, trust, and reload remain host actions. \
Full MCP, executing agents, ToolHost, full orchestration, Claude Desktop, Codex/ChatGPT \
Marketplace bypass, cloud execution, updater behavior, public publication, OS \
signing/notarization, SBOM, provenance, and cross-target packages are not claimed.\n";

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    create_private_directory(&arguments.output)?;
    let authority_root = create_child_directory(&arguments.output, "authority")?;
    let candidate_root = create_child_directory(&arguments.output, "candidate")?;
    let staging_root = create_child_directory(&arguments.output, "staging")?;
    let runtime_root = create_child_directory(&arguments.output, "runtime")?;
    let build_root = shared_build_root()?;

    let mut release_seed = random_seed()?;
    let mut launch_seed = random_seed()?;
    while launch_seed == release_seed {
        launch_seed = random_seed()?;
    }
    let release_key = SigningKey::from_bytes(&release_seed);
    let launch_key = SigningKey::from_bytes(&launch_seed);
    release_seed.fill(0);
    launch_seed.fill(0);

    let authority_bytes = authority_bytes(&release_key, &launch_key)?;
    let authority_path = authority_root.join("qiongli-native-release-authority.json");
    fs::write(&authority_path, &authority_bytes)
        .map_err(|_| "candidate-acceptance-authority-write-failed")?;
    NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "candidate-acceptance-authority-invalid")?;

    let product_binary = build_product(&build_root, &authority_path, &arguments.source_commit)?;
    let content =
        qiongli::embedded_content().map_err(|_| "candidate-acceptance-embedded-content-invalid")?;
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .map_err(|_| "candidate-acceptance-target-unsupported")?;
    let artifact_id =
        native_artifact_id(&artifact).map_err(|_| "candidate-acceptance-artifact-invalid")?;
    let artifact_target =
        approve_native_artifact_target(staging_root.join(&artifact_id), &artifact)
            .map_err(|_| "candidate-acceptance-artifact-target-invalid")?;
    let assembled =
        compose_native_artifact(content.pack(), &artifact, &product_binary, &artifact_target)
            .map_err(|_| "candidate-acceptance-artifact-compose-failed")?;

    let archive_name = native_portable_archive_file_name(&artifact)
        .map_err(|_| "candidate-acceptance-archive-name-invalid")?;
    let archive_path = candidate_root.join(&archive_name);
    let archive_target = approve_native_portable_archive_target(&archive_path, &artifact)
        .map_err(|_| "candidate-acceptance-archive-target-invalid")?;
    let archive =
        compose_native_portable_archive(content.pack(), &artifact_target, &archive_target)
            .map_err(|_| "candidate-acceptance-archive-compose-failed")?;

    let now_unix = now_unix()?;
    let portable_grant = sign_grant(
        LaunchGrantV1 {
            schema_version: 1,
            generation: GENERATION,
            artifact: artifact.clone(),
            binary_sha256: assembled.manifest().binary_sha256.clone(),
            resource_pack_sha256: content.pack().pack_sha256().to_string(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
            not_before_unix: now_unix.saturating_sub(60),
            expires_at_unix: now_unix.saturating_add(RELEASE_VALIDITY_SECONDS),
        },
        &launch_key,
    )?;
    let envelope = build_native_release_envelope(
        GENERATION,
        &archive,
        &portable_grant,
        now_unix.saturating_sub(30),
        now_unix.saturating_add(RELEASE_VALIDITY_SECONDS),
    )
    .map_err(|_| "candidate-acceptance-release-envelope-invalid")?;
    let release_signature = release_key.sign(
        &native_release_envelope_signing_bytes(&envelope)
            .map_err(|_| "candidate-acceptance-release-signing-input-invalid")?,
    );
    let signed_release = SignedNativeReleaseEnvelopeV1 {
        envelope,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: RELEASE_KEY_ID.to_string(),
            value_hex: encode_hex(&release_signature.to_bytes()),
        },
    };
    let candidate = build_native_release_candidate(
        GENERATION,
        &arguments.source_commit,
        &signed_release,
        [
            plugin_grant(
                &artifact,
                ClientActivationTarget::Codex,
                &assembled.manifest().binary_sha256,
                content.pack().pack_sha256(),
                &launch_key,
                now_unix,
            )?,
            plugin_grant(
                &artifact,
                ClientActivationTarget::ClaudeCode,
                &assembled.manifest().binary_sha256,
                content.pack().pack_sha256(),
                &launch_key,
                now_unix,
            )?,
        ],
        NOTES,
        now_unix,
        now_unix.saturating_add(CANDIDATE_VALIDITY_SECONDS),
    )
    .map_err(|_| "candidate-acceptance-candidate-invalid")?;
    let candidate_signature = release_key.sign(
        &native_release_candidate_signing_bytes(&candidate)
            .map_err(|_| "candidate-acceptance-candidate-signing-input-invalid")?,
    );
    let signed_candidate = SignedNativeReleaseCandidateV1 {
        candidate,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: RELEASE_KEY_ID.to_string(),
            value_hex: encode_hex(&candidate_signature.to_bytes()),
        },
    };
    let candidate_name = native_release_candidate_file_name(&artifact)
        .map_err(|_| "candidate-acceptance-candidate-name-invalid")?;
    let candidate_path = candidate_root.join(&candidate_name);
    fs::write(
        &candidate_path,
        signed_candidate
            .to_canonical_json()
            .map_err(|_| "candidate-acceptance-candidate-serialization-failed")?,
    )
    .map_err(|_| "candidate-acceptance-candidate-write-failed")?;
    let notes_name = native_release_notes_file_name(&artifact)
        .map_err(|_| "candidate-acceptance-notes-name-invalid")?;
    let notes_path = candidate_root.join(&notes_name);
    fs::write(&notes_path, NOTES).map_err(|_| "candidate-acceptance-notes-write-failed")?;
    assert_exact_candidate_files(
        &candidate_root,
        [&archive_name, &candidate_name, &notes_name],
    )?;
    drop(release_key);
    drop(launch_key);

    let runtime_target = approve_native_artifact_target(runtime_root.join(&artifact_id), &artifact)
        .map_err(|_| "candidate-acceptance-runtime-target-invalid")?;
    extract_native_portable_archive(content.pack(), &archive_target, &runtime_target)
        .map_err(|_| "candidate-acceptance-runtime-extract-failed")?;
    let runtime_binary = runtime_target.path().join(
        native_artifact_binary_path(&artifact)
            .map_err(|_| "candidate-acceptance-runtime-binary-invalid")?,
    );

    let checks = run_acceptance(
        &arguments.output,
        &runtime_binary,
        &candidate_path,
        &archive_path,
        &notes_path,
    )?;
    let evidence = json!({
        "schema_version": 1,
        "record_type": "qiongli-native-candidate-acceptance",
        "status": "passed",
        "publication_allowed": false,
        "signing": "ephemeral-test-keys-memory-only",
        "source_commit": arguments.source_commit,
        "artifact": artifact,
        "candidate_files": [archive_name, candidate_name, notes_name],
        "checks": checks,
        "external_gates": {
            "real_client": {
                "status": "not-run",
                "reason": "external-client-not-provided"
            },
            "displayed_window": {
                "status": "not-run",
                "reason": "interactive-display-not-provided"
            },
            "production_signing": {
                "status": "not-run",
                "reason": "maintainer-signing-boundary"
            }
        }
    });
    let evidence_bytes = serde_json::to_vec_pretty(&evidence)
        .map_err(|_| "candidate-acceptance-evidence-serialization-failed")?;
    fs::write(
        arguments.output.join("acceptance-evidence.json"),
        evidence_bytes,
    )
    .map_err(|_| "candidate-acceptance-evidence-write-failed")?;
    println!("candidate-acceptance-passed");
    Ok(())
}

struct Arguments {
    output: PathBuf,
    source_commit: String,
}

impl Arguments {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        let mut output = None;
        let mut source_commit = None;
        let mut index = 0;
        while index < values.len() {
            let option = values[index]
                .to_str()
                .ok_or("candidate-acceptance-usage-invalid")?;
            let value = values
                .get(index + 1)
                .ok_or("candidate-acceptance-usage-invalid")?;
            match option {
                "--output" if output.is_none() => output = Some(PathBuf::from(value)),
                "--source-commit" if source_commit.is_none() => {
                    source_commit = value.to_str().map(ToOwned::to_owned)
                }
                _ => return Err("candidate-acceptance-usage-invalid"),
            }
            index += 2;
        }
        let output = output.ok_or("candidate-acceptance-usage-invalid")?;
        let source_commit = source_commit.ok_or("candidate-acceptance-usage-invalid")?;
        if !output.is_absolute()
            || output.exists()
            || !valid_source_commit(&source_commit)
            || output.parent().is_none()
            || !outside_checkout(&output)
        {
            return Err("candidate-acceptance-usage-invalid");
        }
        Ok(Self {
            output,
            source_commit,
        })
    }
}

fn authority_bytes(
    release_key: &SigningKey,
    launch_key: &SigningKey,
) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(&json!({
        "schema_version": 1,
        "channel": "alpha",
        "minimum_release_generation": GENERATION,
        "minimum_launch_grant_generation": GENERATION,
        "release_keys": [{
            "key_id": RELEASE_KEY_ID,
            "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
            "minimum_generation": GENERATION,
            "maximum_generation_exclusive": GENERATION + 1
        }],
        "launch_grant_keys": [{
            "key_id": LAUNCH_KEY_ID,
            "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
        }]
    }))
    .map_err(|_| "candidate-acceptance-authority-serialization-failed")
}

fn build_product(
    build_root: &Path,
    authority_path: &Path,
    source_commit: &str,
) -> Result<PathBuf, &'static str> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(|root| root.join("Cargo.toml"))
        .ok_or("candidate-acceptance-manifest-unavailable")?;
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let status = Command::new(cargo)
        .args([
            OsStr::new("build"),
            OsStr::new("--manifest-path"),
            manifest.as_os_str(),
            OsStr::new("--package"),
            OsStr::new("qiongli"),
            OsStr::new("--bin"),
            OsStr::new("qiongli"),
            OsStr::new("--release"),
            OsStr::new("--locked"),
            OsStr::new("--target-dir"),
            build_root.as_os_str(),
        ])
        .env("QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE", authority_path)
        .env("QIONGLI_NATIVE_SOURCE_COMMIT", source_commit)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .status()
        .map_err(|_| "candidate-acceptance-product-build-failed")?;
    if !status.success() {
        return Err("candidate-acceptance-product-build-failed");
    }
    let binary = build_root
        .join("release")
        .join(format!("qiongli{}", env::consts::EXE_SUFFIX));
    if !binary.is_file() {
        return Err("candidate-acceptance-product-binary-missing");
    }
    Ok(binary)
}

fn shared_build_root() -> Result<PathBuf, &'static str> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(|root| root.join("target/qiongli-native-candidate-acceptance-build"))
        .ok_or("candidate-acceptance-build-root-unavailable")
}

fn outside_checkout(output: &Path) -> bool {
    let Some(output_parent) = output.parent() else {
        return false;
    };
    let Some(checkout_root) = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4) else {
        return false;
    };
    let Ok(output_parent) = fs::canonicalize(output_parent) else {
        return false;
    };
    let Ok(checkout_root) = fs::canonicalize(checkout_root) else {
        return false;
    };
    !output_parent.starts_with(checkout_root)
}

fn assert_exact_candidate_files(root: &Path, expected: [&str; 3]) -> Result<(), &'static str> {
    let mut actual = fs::read_dir(root)
        .map_err(|_| "candidate-acceptance-candidate-files-invalid")?
        .map(|entry| {
            let entry = entry.map_err(|_| "candidate-acceptance-candidate-files-invalid")?;
            if !entry
                .file_type()
                .map_err(|_| "candidate-acceptance-candidate-files-invalid")?
                .is_file()
            {
                return Err("candidate-acceptance-candidate-files-invalid");
            }
            entry
                .file_name()
                .into_string()
                .map_err(|_| "candidate-acceptance-candidate-files-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    actual.sort_unstable();
    let mut expected = expected.map(ToOwned::to_owned);
    expected.sort_unstable();
    if actual != expected {
        return Err("candidate-acceptance-candidate-files-invalid");
    }
    Ok(())
}

fn sign_grant(grant: LaunchGrantV1, key: &SigningKey) -> Result<SignedLaunchGrantV1, &'static str> {
    let signature = key.sign(
        &launch_grant_signing_bytes(&grant)
            .map_err(|_| "candidate-acceptance-grant-signing-input-invalid")?,
    );
    Ok(SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: LAUNCH_KEY_ID.to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    })
}

fn plugin_grant(
    portable_artifact: &qiongli_platform::ArtifactIdentityV1,
    target: ClientActivationTarget,
    binary_sha256: &str,
    pack_sha256: &str,
    key: &SigningKey,
    now_unix: u64,
) -> Result<NativeClientPluginGrantV1, &'static str> {
    let mut artifact = portable_artifact.clone();
    artifact.installer_kind = InstallerKind::PluginBundle;
    Ok(NativeClientPluginGrantV1 {
        target,
        signed_launch_grant: sign_grant(
            LaunchGrantV1 {
                schema_version: 1,
                generation: GENERATION,
                artifact,
                binary_sha256: binary_sha256.to_string(),
                resource_pack_sha256: pack_sha256.to_string(),
                allowed_modes: vec![GrantMode::LiteMcp],
                integration_scopes: vec![target.integration_scope()],
                not_before_unix: now_unix.saturating_sub(60),
                expires_at_unix: now_unix.saturating_add(RELEASE_VALIDITY_SECONDS),
            },
            key,
        )?,
    })
}

fn run_acceptance(
    root: &Path,
    binary: &Path,
    candidate: &Path,
    archive: &Path,
    notes: &Path,
) -> Result<Value, &'static str> {
    let product_home = create_child_directory(root, "product-home")?;
    let version = run_product(binary, root, &product_home, [OsStr::new("--version")])?;
    let version_text = String::from_utf8(version.stdout)
        .map_err(|_| "candidate-acceptance-version-output-invalid")?;
    if version_text.trim() != format!("qiongli {}", env!("CARGO_PKG_VERSION")) {
        return Err("candidate-acceptance-version-mismatch");
    }

    let content_list = run_product(
        binary,
        root,
        &product_home,
        [OsStr::new("content"), OsStr::new("list")],
    )?;
    let content_json = parse_output_json(&content_list)?;
    if content_json["command"] != "content-list" {
        return Err("candidate-acceptance-content-list-invalid");
    }
    let materialized = root.join("materialized-lite");
    let materialize = run_product(
        binary,
        root,
        &product_home,
        [
            OsStr::new("content"),
            OsStr::new("materialize"),
            OsStr::new("--profile"),
            OsStr::new("lite"),
            OsStr::new("--target"),
            materialized.as_os_str(),
        ],
    )?;
    let materialize_json = parse_output_json(&materialize)?;
    if materialize_json["command"] != "content-materialize"
        || !materialized.join("workflow/SKILL.md").is_file()
    {
        return Err("candidate-acceptance-content-materialize-invalid");
    }

    let ui = run_product(
        binary,
        root,
        &product_home,
        [OsStr::new("ui"), OsStr::new("--startup-check")],
    )?;
    let ui_json = parse_output_json(&ui)?;
    if ui_json["command"] != "ui-startup-check" || ui_json["service"] != "ready" {
        return Err("candidate-acceptance-ui-preflight-invalid");
    }
    run_mcp(binary, root, &product_home)?;

    for (target, directory) in [("codex", "codex-home"), ("claude", "claude-home")] {
        let home = create_child_directory(root, directory)?;
        let canary = home.join("unrelated-user-canary");
        fs::write(&canary, b"preserve").map_err(|_| "candidate-acceptance-canary-write-failed")?;
        let common = [
            OsStr::new("--candidate"),
            candidate.as_os_str(),
            OsStr::new("--archive"),
            archive.as_os_str(),
            OsStr::new("--release-notes"),
            notes.as_os_str(),
            OsStr::new("--target"),
            OsStr::new(target),
        ];
        let mut preview_args = vec![OsString::from("install"), OsString::from("candidate")];
        preview_args.push(OsString::from("preview"));
        preview_args.extend(common.iter().map(|value| (*value).to_os_string()));
        let preview = run_product(binary, root, &home, preview_args)?;
        let preview_json = parse_output_json(&preview)?;
        if preview_json["mutation"] != "none" || home.join(".qiongli").exists() {
            return Err("candidate-acceptance-preview-mutated-state");
        }
        let approval_digest = preview_json["approval_digest_sha256"]
            .as_str()
            .filter(|value| valid_sha256(value))
            .ok_or("candidate-acceptance-preview-digest-invalid")?;
        let install_id = preview_json["install_id"]
            .as_str()
            .ok_or("candidate-acceptance-preview-install-id-invalid")?
            .to_string();

        let mut apply_args = vec![
            OsString::from("install"),
            OsString::from("candidate"),
            OsString::from("apply"),
        ];
        apply_args.extend(common.iter().map(|value| (*value).to_os_string()));
        apply_args.extend([
            OsString::from("--expected-approval-digest"),
            OsString::from(approval_digest),
            OsString::from("--approve-filesystem-write"),
            OsString::from("--approve-client-config-change"),
            OsString::from("--approve-host-trust"),
        ]);
        if target == "codex" {
            let mut wrong_digest_args = apply_args.clone();
            let digest_index = wrong_digest_args
                .iter()
                .position(|value| value == "--expected-approval-digest")
                .and_then(|index| index.checked_add(1))
                .ok_or("candidate-acceptance-apply-arguments-invalid")?;
            wrong_digest_args[digest_index] = OsString::from(
                if approval_digest
                    == "0000000000000000000000000000000000000000000000000000000000000000"
                {
                    "1111111111111111111111111111111111111111111111111111111111111111"
                } else {
                    "0000000000000000000000000000000000000000000000000000000000000000"
                },
            );
            run_product_failure(binary, root, &home, wrong_digest_args, 1)?;
            let mut partial_approval_args = apply_args.clone();
            partial_approval_args.retain(|value| value != "--approve-host-trust");
            run_product_failure(binary, root, &home, partial_approval_args, 2)?;
            if home.join(".qiongli").exists() {
                return Err("candidate-acceptance-failed-approval-mutated-state");
            }

            let agents = create_child_directory(&home, ".agents")?;
            let plugins = create_child_directory(&agents, "plugins")?;
            let marketplace = plugins.join("marketplace.json");
            let conflict = serde_json::to_vec(&json!({
                "plugins": [{
                    "name": "qiongli",
                    "source": {"source": "local", "path": "./foreign-source"},
                    "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                    "category": "Education"
                }]
            }))
            .map_err(|_| "candidate-acceptance-conflict-serialization-failed")?;
            fs::write(&marketplace, &conflict)
                .map_err(|_| "candidate-acceptance-conflict-write-failed")?;
            run_product_failure(binary, root, &home, apply_args.clone(), 1)?;
            if home.join(".qiongli/plugins/codex/qiongli").exists()
                || payload_directory_present(&home.join(".qiongli/native/payloads"))?
                || fs::read(&marketplace)
                    .map_err(|_| "candidate-acceptance-conflict-read-failed")?
                    != conflict
            {
                return Err("candidate-acceptance-compensation-invalid");
            }
            fs::remove_file(marketplace)
                .map_err(|_| "candidate-acceptance-conflict-cleanup-failed")?;
        }
        let apply = run_product(binary, root, &home, apply_args)?;
        let apply_json = parse_output_json(&apply)?;
        if apply_json["install_id"] != install_id
            || apply_json["outstanding_host_action"] != "install-or-enable-plugin"
        {
            return Err("candidate-acceptance-apply-output-invalid");
        }
        let verify = run_product(
            binary,
            root,
            &home,
            [
                OsStr::new("install"),
                OsStr::new("candidate"),
                OsStr::new("verify"),
                OsStr::new("--target"),
                OsStr::new(target),
                OsStr::new("--install-id"),
                OsStr::new(&install_id),
            ],
        )?;
        if parse_output_json(&verify)?["state"] != "healthy" {
            return Err("candidate-acceptance-verify-output-invalid");
        }
        let remove = run_product(
            binary,
            root,
            &home,
            [
                OsStr::new("install"),
                OsStr::new("candidate"),
                OsStr::new("remove"),
                OsStr::new("--target"),
                OsStr::new(target),
                OsStr::new("--install-id"),
                OsStr::new(&install_id),
                OsStr::new("--approve-filesystem-write"),
                OsStr::new("--approve-client-config-change"),
            ],
        )?;
        if parse_output_json(&remove)?["payload_disposition"] != "removed"
            || fs::read(&canary).map_err(|_| "candidate-acceptance-canary-read-failed")?
                != b"preserve"
        {
            return Err("candidate-acceptance-remove-invalid");
        }
    }

    Ok(json!({
        "runtime_path": "empty",
        "checkout_boundary": "outside-checkout",
        "version": "passed",
        "embedded_skills": "passed",
        "ui_startup_preflight": "passed",
        "lite_mcp": "passed",
        "codex_local_lifecycle": "passed",
        "claude_code_local_lifecycle": "passed",
        "digest_and_partial_approval_rejection": "passed",
        "fresh_failure_compensation": "passed",
        "unrelated_state_preservation": "passed"
    }))
}

fn run_mcp(binary: &Path, root: &Path, home: &Path) -> Result<(), &'static str> {
    let mut command = product_command(binary, root, home);
    command
        .args([
            "mcp",
            "serve",
            "--transport",
            "stdio",
            "--profile",
            "marketplace-lite",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|_| "candidate-acceptance-mcp-start-failed")?;
    let requests = [
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-11-25", "capabilities": {}}
        }),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "qiongli_config_status", "arguments": {}}
        }),
    ];
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("candidate-acceptance-mcp-stdin-unavailable")?;
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request)
                .map_err(|_| "candidate-acceptance-mcp-request-invalid")?;
            stdin
                .write_all(b"\n")
                .map_err(|_| "candidate-acceptance-mcp-request-write-failed")?;
        }
    }
    drop(child.stdin.take());
    let output = child
        .wait_with_output()
        .map_err(|_| "candidate-acceptance-mcp-wait-failed")?;
    validate_public_output(&output, root)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err("candidate-acceptance-mcp-failed");
    }
    let stdout =
        String::from_utf8(output.stdout).map_err(|_| "candidate-acceptance-mcp-output-invalid")?;
    let responses = stdout
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "candidate-acceptance-mcp-output-invalid")?;
    if responses.len() != 3 {
        return Err("candidate-acceptance-mcp-output-invalid");
    }
    let tools = responses
        .iter()
        .find(|value| value["id"] == 2)
        .and_then(|value| value["result"]["tools"].as_array())
        .ok_or("candidate-acceptance-mcp-tools-invalid")?;
    let names = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    if names != LITE_PUBLIC_TOOL_NAMES {
        return Err("candidate-acceptance-mcp-tools-invalid");
    }
    let call = responses
        .iter()
        .find(|value| value["id"] == 3)
        .ok_or("candidate-acceptance-mcp-call-invalid")?;
    if call["result"]["structuredContent"]["config_path"] != "<managed-native-config>" {
        return Err("candidate-acceptance-mcp-call-invalid");
    }
    Ok(())
}

fn run_product<I, S>(
    binary: &Path,
    root: &Path,
    home: &Path,
    args: I,
) -> Result<Output, &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = product_command(binary, root, home)
        .args(args)
        .output()
        .map_err(|_| "candidate-acceptance-product-start-failed")?;
    validate_public_output(&output, root)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err("candidate-acceptance-product-command-failed");
    }
    Ok(output)
}

fn run_product_failure<I, S>(
    binary: &Path,
    root: &Path,
    home: &Path,
    args: I,
    expected_exit_code: i32,
) -> Result<(), &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = product_command(binary, root, home)
        .args(args)
        .output()
        .map_err(|_| "candidate-acceptance-product-start-failed")?;
    validate_public_output(&output, root)?;
    if output.status.code() != Some(expected_exit_code)
        || !output.stdout.is_empty()
        || output.stderr.is_empty()
    {
        return Err("candidate-acceptance-product-failure-contract-invalid");
    }
    Ok(())
}

fn product_command(binary: &Path, root: &Path, home: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("PATH", "")
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("QIONGLI_CONFIG_HOME", home.join(".qiongli/config"))
        .current_dir(root);
    for name in ["SYSTEMROOT", "WINDIR", "TEMP", "TMP", "TMPDIR"] {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

fn parse_output_json(output: &Output) -> Result<Value, &'static str> {
    serde_json::from_slice(&output.stdout)
        .map_err(|_| "candidate-acceptance-product-output-invalid")
}

fn validate_public_output(output: &Output, root: &Path) -> Result<(), &'static str> {
    let root = root.to_string_lossy();
    if String::from_utf8_lossy(&output.stdout).contains(root.as_ref())
        || String::from_utf8_lossy(&output.stderr).contains(root.as_ref())
    {
        return Err("candidate-acceptance-private-path-leaked");
    }
    Ok(())
}

fn create_child_directory(root: &Path, leaf: &str) -> Result<PathBuf, &'static str> {
    let path = root.join(leaf);
    create_private_directory(&path)?;
    Ok(path)
}

fn payload_directory_present(root: &Path) -> Result<bool, &'static str> {
    if !root.exists() {
        return Ok(false);
    }
    let entries =
        fs::read_dir(root).map_err(|_| "candidate-acceptance-payload-directory-read-failed")?;
    for entry in entries {
        let entry = entry.map_err(|_| "candidate-acceptance-payload-directory-read-failed")?;
        if entry
            .file_type()
            .map_err(|_| "candidate-acceptance-payload-directory-read-failed")?
            .is_dir()
            && !entry.file_name().to_string_lossy().starts_with('.')
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| "candidate-acceptance-directory-create-failed")
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| "candidate-acceptance-directory-create-failed")
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), &'static str> {
    Err("candidate-acceptance-platform-unsupported")
}

fn random_seed() -> Result<[u8; 32], &'static str> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|_| "candidate-acceptance-random-unavailable")?;
    Ok(seed)
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && valid_lower_hex(value)
}

fn valid_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "candidate-acceptance-clock-unavailable")
}
