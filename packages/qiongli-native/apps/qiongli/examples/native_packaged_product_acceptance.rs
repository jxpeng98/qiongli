#![allow(clippy::disallowed_methods)]

use std::env;
use std::ffi::{OsStr, OsString};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer as _, SigningKey};
use qiongli_config::{
    GLOBAL_SETTINGS_FILE, GlobalSettingsStore, SecretRef, SecretStoreStatus, SecretValue,
    resolve_config_root,
};
use qiongli_content::{approve_materialization_target, verify_materialization};
use qiongli_platform::{
    ClientActivationCoordinator, ClientActivationTarget, NativeReleaseAuthority,
    PackagedProductInstallDisposition, PackagedProductInstallEffect,
    PackagedProductVerificationInput, apply_packaged_product_install, discover_client_activation,
    preview_packaged_product_install, remove_packaged_product_install, verify_packaged_product,
    verify_packaged_product_install,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use qiongli_runtime::mcp::MCP_PROTOCOL_VERSION;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use zeroize::Zeroizing;

const RELEASE_KEY_ID: &str = "packaged-acceptance-release-key";
const LAUNCH_KEY_ID: &str = "packaged-acceptance-launch-key";
const GENERATION: u64 = 1;
const MAX_JSON_BYTES: u64 = 2 * 1024 * 1024;
const MAX_COMMAND_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MANIFEST_FILE: &str = "qiongli-desktop-package.manifest.json";
const RECEIPT_FILE: &str = "qiongli-desktop-package.receipt.json";
const CONTROL_FILE: &str = ".qiongli-product-control.json";
const INTERNAL_MANIFEST_FILE: &str = ".qiongli-desktop-package.json";
const ACCEPTANCE_RECEIPT_FILE: &str = "qiongli-packaged-product-acceptance.receipt.json";

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    if env::consts::OS != "macos" {
        return Err("packaged-product-acceptance-macos-required");
    }
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    create_private_directory(&arguments.output)?;
    let authority_root = create_private_child(&arguments.output, "authority")?;
    let components_root = create_private_child(&arguments.output, "components")?;
    let preliminary_root = arguments.output.join("preliminary-package");
    let final_root = arguments.output.join("product-package");
    let signed_root = arguments.output.join("signed-product");
    let request_root = create_private_child(&arguments.output, "product-control")?;
    let extracted_root = create_private_child(&arguments.output, "extracted")?;
    let home = create_private_child(&arguments.output, "isolated-home")?;

    let release_seed = random_seed()?;
    let launch_seed = random_seed()?;
    if release_seed.as_ref() == launch_seed.as_ref() {
        return Err("packaged-product-acceptance-random-failed");
    }
    let release_key = SigningKey::from_bytes(&release_seed);
    let launch_key = SigningKey::from_bytes(&launch_seed);
    let authority_bytes = authority_bytes(&release_key, &launch_key)?;
    let authority_path = authority_root.join("qiongli-native-release-authority.json");
    write_new_private(&authority_path, &authority_bytes)?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "packaged-product-acceptance-authority-invalid")?;

    let tools = build_tools(&authority_path, &arguments.source_commit)?;
    let canonical = components_root.join("qiongli-cli");
    let launcher = components_root.join("Qiongli");
    let update_helper = components_root.join("qiongli-update-helper");
    stage_executable(&tools.canonical, &canonical)?;
    stage_executable(&tools.launcher, &launcher)?;
    stage_executable(&tools.update_helper, &update_helper)?;
    ad_hoc_sign_canonical(&canonical)?;
    let canonical_sha256 = sha256_file(&canonical)?;

    run_desktop_composer(
        &tools.desktop_composer,
        &canonical,
        &launcher,
        &update_helper,
        &preliminary_root,
        &arguments.source_commit,
        None,
    )?;
    let now_unix = now_unix()?;
    let not_before_unix = now_unix.saturating_sub(60).to_string();
    let expires_at_unix = now_unix.saturating_add(3_600).to_string();
    let request = request_root.join("product-control-signing-request.json");
    run_command(
        Command::new(&tools.product_control).args([
            OsStr::new("prepare"),
            OsStr::new("--desktop-manifest"),
            preliminary_root.join(MANIFEST_FILE).as_os_str(),
            OsStr::new("--canonical"),
            canonical.as_os_str(),
            OsStr::new("--authority"),
            authority_path.as_os_str(),
            OsStr::new("--generation"),
            OsStr::new("1"),
            OsStr::new("--not-before-unix"),
            OsStr::new(&not_before_unix),
            OsStr::new("--expires-at-unix"),
            OsStr::new(&expires_at_unix),
            OsStr::new("--output"),
            request.as_os_str(),
        ]),
        "packaged-product-acceptance-control-prepare-failed",
    )?;

    let request_value = read_json(&request)?;
    let signatures = sign_requested_grants(&request_value, &launch_key)?;
    let codex_signature = request_root.join("codex.sig");
    let claude_signature = request_root.join("claude.sig");
    write_new_private(&codex_signature, signatures[0].as_bytes())?;
    write_new_private(&claude_signature, signatures[1].as_bytes())?;
    let control = request_root.join(CONTROL_FILE);
    let finalized_manifest = request_root.join(MANIFEST_FILE);
    run_command(
        Command::new(&tools.product_control).args([
            OsStr::new("finalize"),
            OsStr::new("--request"),
            request.as_os_str(),
            OsStr::new("--desktop-manifest"),
            preliminary_root.join(MANIFEST_FILE).as_os_str(),
            OsStr::new("--authority"),
            authority_path.as_os_str(),
            OsStr::new("--launch-key-id"),
            OsStr::new(LAUNCH_KEY_ID),
            OsStr::new("--codex-signature"),
            codex_signature.as_os_str(),
            OsStr::new("--claude-signature"),
            claude_signature.as_os_str(),
            OsStr::new("--control-output"),
            control.as_os_str(),
            OsStr::new("--manifest-output"),
            finalized_manifest.as_os_str(),
        ]),
        "packaged-product-acceptance-control-finalize-failed",
    )?;

    run_desktop_composer(
        &tools.desktop_composer,
        &canonical,
        &launcher,
        &update_helper,
        &final_root,
        &arguments.source_commit,
        Some(&control),
    )?;
    if read_bounded(&final_root.join(MANIFEST_FILE), MAX_JSON_BYTES)?
        != read_bounded(&finalized_manifest, MAX_JSON_BYTES)?
    {
        return Err("packaged-product-acceptance-final-manifest-drift");
    }
    let package_receipt = read_json(&final_root.join(RECEIPT_FILE))?;
    let package_sha256 = package_receipt["package_sha256"]
        .as_str()
        .filter(|value| valid_lower_hex(value, 64))
        .ok_or("packaged-product-acceptance-package-receipt-invalid")?;
    run_command(
        Command::new(&arguments.signing_script).args([
            OsStr::new("--artifact-dir"),
            final_root.as_os_str(),
            OsStr::new("--expected-source-commit"),
            OsStr::new(&arguments.source_commit),
            OsStr::new("--expected-package-sha256"),
            OsStr::new(package_sha256),
            OsStr::new("--output-dir"),
            signed_root.as_os_str(),
            OsStr::new("--test-only-ad-hoc"),
            OsStr::new("--preserve-signed-canonical"),
        ]),
        "packaged-product-acceptance-app-signing-failed",
    )?;

    let signed_archive =
        signed_root.join("qiongli-desktop-2.0.0-alpha.1-macos-aarch64.ad-hoc-test.app.zip");
    run_command(
        Command::new("/usr/bin/ditto").args([
            OsStr::new("-x"),
            OsStr::new("-k"),
            signed_archive.as_os_str(),
            extracted_root.as_os_str(),
        ]),
        "packaged-product-acceptance-extraction-failed",
    )?;
    let app = extracted_root.join("Qiongli.app");
    let packaged_canonical = app.join("Contents/MacOS/qiongli-cli");
    let packaged_launcher = app.join("Contents/MacOS/Qiongli");
    let resources = app.join("Contents/Resources");
    if sha256_file(&packaged_canonical)? != canonical_sha256 {
        return Err("packaged-product-acceptance-canonical-drift");
    }
    verify_packaged_entrypoints(&packaged_canonical, &packaged_launcher, &home)?;
    progress("entrypoints");
    exercise_skills_lifecycle(&packaged_canonical, &home)?;
    progress("skills");
    exercise_lite_mcp_self_test(&packaged_canonical, &home)?;
    progress("lite-mcp");
    exercise_provider_secret_lifecycle(&home)?;
    progress("provider-keychain");
    exercise_product_lifecycle(
        &packaged_canonical,
        &resources.join(INTERNAL_MANIFEST_FILE),
        &resources.join(CONTROL_FILE),
        &authority,
        &arguments.source_commit,
        &home,
        now_unix,
    )?;
    progress("client-lifecycle");

    let signing_receipt =
        read_json(&signed_root.join("qiongli-macos-alpha1-signing.receipt.json"))?;
    if signing_receipt["signing"]["canonical_signature_preserved"] != true {
        return Err("packaged-product-acceptance-signing-receipt-invalid");
    }
    let receipt = AcceptanceReceiptV1 {
        schema_version: 1,
        record_type: "qiongli-packaged-product-acceptance",
        status: "accepted-ad-hoc-nonpublishing",
        publication_allowed: false,
        product_source_commit: &arguments.source_commit,
        canonical_sha256: &canonical_sha256,
        product_control_sha256: sha256_file(&control)?,
        signed_archive_sha256: sha256_file(&signed_archive)?,
        checks: AcceptanceChecksV1 {
            embedded_authority: true,
            canonical_signature_preserved: true,
            product_control_verified: true,
            inventory_discovered: true,
            skills_materialize_verify_refresh: true,
            lite_mcp_self_test: true,
            provider_keychain_save_replace_restart_remove: true,
            codex_install_verify_remove: true,
            claude_install_verify_remove: true,
            registration_repair: true,
            packaged_restart_verification: true,
            legacy_content_preserved: true,
            empty_path_startup: true,
        },
    };
    let receipt_bytes = serde_json_canonicalizer::to_vec(&receipt)
        .map_err(|_| "packaged-product-acceptance-receipt-invalid")?;
    let receipt_path = arguments.output.join(ACCEPTANCE_RECEIPT_FILE);
    write_new_private(&receipt_path, &receipt_bytes)?;
    println!(
        "{}",
        String::from_utf8(receipt_bytes)
            .map_err(|_| "packaged-product-acceptance-receipt-invalid")?
    );
    Ok(())
}

struct Arguments {
    output: PathBuf,
    source_commit: String,
    signing_script: PathBuf,
}

impl Arguments {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.len() != 6 {
            return Err("packaged-product-acceptance-usage-invalid");
        }
        let mut output = None;
        let mut source_commit = None;
        let mut signing_script = None;
        for pair in values.chunks_exact(2) {
            match pair[0].to_str() {
                Some("--output") if output.is_none() => output = Some(PathBuf::from(&pair[1])),
                Some("--source-commit") if source_commit.is_none() => {
                    source_commit = pair[1].to_str().map(ToOwned::to_owned)
                }
                Some("--signing-script") if signing_script.is_none() => {
                    signing_script = Some(PathBuf::from(&pair[1]))
                }
                _ => return Err("packaged-product-acceptance-usage-invalid"),
            }
        }
        let output = output.ok_or("packaged-product-acceptance-usage-invalid")?;
        let source_commit = source_commit.ok_or("packaged-product-acceptance-usage-invalid")?;
        let signing_script = signing_script.ok_or("packaged-product-acceptance-usage-invalid")?;
        if !output.is_absolute()
            || output.exists()
            || output.parent().is_none_or(|parent| !parent.is_dir())
            || !valid_source_commit(&source_commit)
            || !signing_script.is_absolute()
            || !signing_script.is_file()
        {
            return Err("packaged-product-acceptance-usage-invalid");
        }
        Ok(Self {
            output,
            source_commit,
            signing_script,
        })
    }
}

struct BuiltTools {
    canonical: PathBuf,
    launcher: PathBuf,
    update_helper: PathBuf,
    desktop_composer: PathBuf,
    product_control: PathBuf,
}

fn build_tools(authority: &Path, source_commit: &str) -> Result<BuiltTools, &'static str> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("packaged-product-acceptance-workspace-invalid")?;
    let target = workspace.join("target/qiongli-packaged-product-acceptance-build");
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut command = Command::new(cargo);
    command.args([
        OsStr::new("build"),
        OsStr::new("--manifest-path"),
        workspace.join("Cargo.toml").as_os_str(),
        OsStr::new("--package"),
        OsStr::new("qiongli"),
        OsStr::new("--release"),
        OsStr::new("--locked"),
        OsStr::new("--target-dir"),
        target.as_os_str(),
        OsStr::new("--bins"),
        OsStr::new("--example"),
        OsStr::new("native_desktop_package"),
        OsStr::new("--example"),
        OsStr::new("native_product_control"),
    ]);
    command
        .env("QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE", authority)
        .env("QIONGLI_NATIVE_SOURCE_COMMIT", source_commit)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER");
    run_command(
        &mut command,
        "packaged-product-acceptance-product-build-failed",
    )?;
    let release = target.join("release");
    let examples = release.join("examples");
    let tools = BuiltTools {
        canonical: release.join("qiongli"),
        launcher: release.join("qiongli-desktop"),
        update_helper: release.join("qiongli-update-helper"),
        desktop_composer: examples.join("native_desktop_package"),
        product_control: examples.join("native_product_control"),
    };
    for path in [
        &tools.canonical,
        &tools.launcher,
        &tools.update_helper,
        &tools.desktop_composer,
        &tools.product_control,
    ] {
        if !path.is_file() {
            return Err("packaged-product-acceptance-built-tool-missing");
        }
    }
    Ok(tools)
}

fn run_desktop_composer(
    composer: &Path,
    canonical: &Path,
    launcher: &Path,
    update_helper: &Path,
    output: &Path,
    source_commit: &str,
    product_control: Option<&Path>,
) -> Result<(), &'static str> {
    let mut command = Command::new(composer);
    command.args([
        OsStr::new("--canonical"),
        canonical.as_os_str(),
        OsStr::new("--launcher"),
        launcher.as_os_str(),
        OsStr::new("--update-helper"),
        update_helper.as_os_str(),
        OsStr::new("--output"),
        output.as_os_str(),
        OsStr::new("--source-commit"),
        OsStr::new(source_commit),
    ]);
    if let Some(control) = product_control {
        command.args([OsStr::new("--product-control"), control.as_os_str()]);
    }
    run_command(
        &mut command,
        "packaged-product-acceptance-package-compose-failed",
    )?;
    Ok(())
}

fn sign_requested_grants(
    request: &Value,
    launch_key: &SigningKey,
) -> Result<[String; 2], &'static str> {
    if request["publication_allowed"] != false
        || request["status"] != "awaiting-external-launch-grant-signatures"
    {
        return Err("packaged-product-acceptance-signing-request-invalid");
    }
    let grants = request["grants"]
        .as_array()
        .filter(|grants| grants.len() == 2)
        .ok_or("packaged-product-acceptance-signing-request-invalid")?;
    let expected_targets = ["codex", "claude-code"];
    let mut signatures = Vec::new();
    for (grant, target) in grants.iter().zip(expected_targets) {
        if grant["target"] != target {
            return Err("packaged-product-acceptance-signing-request-invalid");
        }
        let preimage_hex = grant["signing_preimage_hex"]
            .as_str()
            .ok_or("packaged-product-acceptance-signing-request-invalid")?;
        let preimage = decode_hex(preimage_hex)?;
        let digest = sha256_hex(&preimage);
        if grant["signing_preimage_sha256"] != digest {
            return Err("packaged-product-acceptance-signing-request-invalid");
        }
        signatures.push(encode_hex(&launch_key.sign(&preimage).to_bytes()));
    }
    signatures
        .try_into()
        .map_err(|_| "packaged-product-acceptance-signing-request-invalid")
}

fn exercise_skills_lifecycle(canonical: &Path, home: &Path) -> Result<(), &'static str> {
    let managed_root = home.join(".qiongli");
    create_private_tree(&managed_root)?;
    let target = managed_root.join("skills");
    let approved_target = approve_materialization_target(&target)
        .map_err(|_| "packaged-product-acceptance-skills-target-invalid")?;
    let arguments = vec![
        OsString::from("content"),
        OsString::from("materialize"),
        OsString::from("--profile"),
        OsString::from("skill-only"),
        OsString::from("--target"),
        target.as_os_str().to_owned(),
    ];
    let first = isolated_command_args(canonical, home, &arguments)?;
    let first: Value = serde_json::from_slice(&first.stdout)
        .map_err(|_| "packaged-product-acceptance-skills-output-invalid")?;
    if first["command"] != "content-materialize"
        || first["profile"] != "skill-only"
        || first["entry_count"].as_u64().is_none_or(|count| count == 0)
    {
        return Err("packaged-product-acceptance-skills-output-invalid");
    }
    verify_materialization(&approved_target)
        .map_err(|_| "packaged-product-acceptance-skills-verification-failed")?;

    let canary = managed_root.join("content-refresh-canary");
    write_new_private(&canary, b"preserve-outside-receipt-owned-skills")?;
    isolated_command_args(canonical, home, &arguments)?;
    verify_materialization(&approved_target)
        .map_err(|_| "packaged-product-acceptance-skills-refresh-failed")?;
    if fs::read(&canary).ok().as_deref() != Some(b"preserve-outside-receipt-owned-skills") {
        return Err("packaged-product-acceptance-skills-refresh-drift");
    }
    Ok(())
}

fn exercise_lite_mcp_self_test(canonical: &Path, home: &Path) -> Result<(), &'static str> {
    let requests = [
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "qiongli_task_plan",
                "arguments": {
                    "task_id": "packaged-product-acceptance",
                    "paper_type": "review",
                    "topic": "offline packaged self-test"
                }
            }
        }),
    ];
    let mut input = Vec::new();
    for request in requests {
        serde_json::to_writer(&mut input, &request)
            .map_err(|_| "packaged-product-acceptance-mcp-input-invalid")?;
        input.push(b'\n');
    }

    let mut command = Command::new(canonical);
    command
        .args(["mcp", "serve", "--profile", "lite", "--transport", "stdio"])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "")
        .current_dir(home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|_| "packaged-product-acceptance-mcp-start-failed")?;
    child
        .stdin
        .take()
        .ok_or("packaged-product-acceptance-mcp-start-failed")?
        .write_all(&input)
        .map_err(|_| "packaged-product-acceptance-mcp-write-failed")?;
    let output = child
        .wait_with_output()
        .map_err(|_| "packaged-product-acceptance-mcp-wait-failed")?;
    if !output.status.success()
        || output.stdout.len().saturating_add(output.stderr.len()) > MAX_COMMAND_OUTPUT_BYTES
    {
        return Err("packaged-product-acceptance-mcp-command-failed");
    }
    let responses = output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_slice::<Value>(line)
                .map_err(|_| "packaged-product-acceptance-mcp-output-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if responses.len() != 3
        || responses[0]
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str)
            != Some(MCP_PROTOCOL_VERSION)
        || responses[2].get("result").is_none()
        || responses[2].get("error").is_some()
    {
        return Err("packaged-product-acceptance-mcp-output-invalid");
    }
    let names = responses[1]
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("packaged-product-acceptance-mcp-output-invalid")?
        .iter()
        .map(|tool| tool.get("name").and_then(Value::as_str))
        .collect::<Option<Vec<_>>>()
        .ok_or("packaged-product-acceptance-mcp-output-invalid")?;
    if names.as_slice() != LITE_PUBLIC_TOOL_NAMES {
        return Err("packaged-product-acceptance-mcp-tools-drift");
    }
    Ok(())
}

fn exercise_provider_secret_lifecycle(home: &Path) -> Result<(), &'static str> {
    let secret_store = qiongli::native_secret_store();
    if secret_store.status() != SecretStoreStatus::Available {
        return Err("packaged-product-acceptance-secret-store-unavailable");
    }
    let mut identifier = [0_u8; 16];
    getrandom::fill(&mut identifier).map_err(|_| "packaged-product-acceptance-random-failed")?;
    let secret_ref = SecretRef::parse(&format!("qsr1_{}", encode_hex(&identifier)))
        .map_err(|_| "packaged-product-acceptance-secret-ref-invalid")?;
    let mut first_bytes = Zeroizing::new(vec![0_u8; 32]);
    let mut replacement_bytes = Zeroizing::new(vec![0_u8; 32]);
    getrandom::fill(first_bytes.as_mut_slice())
        .map_err(|_| "packaged-product-acceptance-random-failed")?;
    getrandom::fill(replacement_bytes.as_mut_slice())
        .map_err(|_| "packaged-product-acceptance-random-failed")?;
    if first_bytes.as_slice() == replacement_bytes.as_slice() {
        return Err("packaged-product-acceptance-random-failed");
    }
    let first = SecretValue::new(first_bytes.as_slice().to_vec())
        .map_err(|_| "packaged-product-acceptance-secret-invalid")?;
    let replacement = SecretValue::new(replacement_bytes.as_slice().to_vec())
        .map_err(|_| "packaged-product-acceptance-secret-invalid")?;
    secret_store
        .store(&secret_ref, &first)
        .map_err(|_| "packaged-product-acceptance-secret-save-failed")?;

    let root = resolve_config_root(None, home)
        .map_err(|_| "packaged-product-acceptance-config-root-invalid")?;
    let settings_path = root.state_root().join(GLOBAL_SETTINGS_FILE);
    let store = GlobalSettingsStore::new(root);
    let result = (|| {
        let loaded = store
            .load()
            .map_err(|_| "packaged-product-acceptance-config-load-failed")?;
        let mut settings = loaded.settings;
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.api_key_ref = Some(secret_ref.clone());
        store
            .replace(loaded.revision, settings)
            .map_err(|_| "packaged-product-acceptance-config-save-failed")?;
        let settings_bytes = read_bounded(&settings_path, MAX_JSON_BYTES)?;
        if contains_bytes(&settings_bytes, first_bytes.as_slice()) {
            return Err("packaged-product-acceptance-secret-persisted");
        }

        let restarted = GlobalSettingsStore::new(
            resolve_config_root(None, home)
                .map_err(|_| "packaged-product-acceptance-config-root-invalid")?,
        );
        let loaded = restarted
            .load()
            .map_err(|_| "packaged-product-acceptance-config-restart-failed")?;
        if loaded.settings.providers.openalex.api_key_ref.as_ref() != Some(&secret_ref)
            || secret_store
                .resolve(&secret_ref)
                .map_err(|_| "packaged-product-acceptance-secret-restart-failed")?
                .as_bytes()
                != first_bytes.as_slice()
        {
            return Err("packaged-product-acceptance-secret-restart-failed");
        }

        secret_store
            .store(&secret_ref, &replacement)
            .map_err(|_| "packaged-product-acceptance-secret-replace-failed")?;
        if secret_store
            .resolve(&secret_ref)
            .map_err(|_| "packaged-product-acceptance-secret-replace-failed")?
            .as_bytes()
            != replacement_bytes.as_slice()
            || contains_bytes(&settings_bytes, replacement_bytes.as_slice())
        {
            return Err("packaged-product-acceptance-secret-replace-failed");
        }

        let mut settings = loaded.settings;
        settings.providers.openalex.api_key_ref = None;
        restarted
            .replace(loaded.revision, settings)
            .map_err(|_| "packaged-product-acceptance-config-remove-failed")?;
        secret_store
            .remove(&secret_ref)
            .map_err(|_| "packaged-product-acceptance-secret-remove-failed")?;
        if GlobalSettingsStore::new(
            resolve_config_root(None, home)
                .map_err(|_| "packaged-product-acceptance-config-root-invalid")?,
        )
        .load()
        .map_err(|_| "packaged-product-acceptance-config-restart-failed")?
        .settings
        .providers
        .openalex
        .api_key_ref
        .is_some()
        {
            return Err("packaged-product-acceptance-secret-ref-remove-failed");
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = secret_store.remove(&secret_ref);
    }
    result
}

fn exercise_product_lifecycle(
    canonical: &Path,
    manifest: &Path,
    control: &Path,
    authority: &NativeReleaseAuthority,
    source_commit: &str,
    home: &Path,
    now_unix: u64,
) -> Result<(), &'static str> {
    let content =
        qiongli::embedded_content().map_err(|_| "packaged-product-acceptance-content-invalid")?;
    let product = verify_packaged_product(&PackagedProductVerificationInput {
        current_executable: canonical,
        desktop_manifest_path: manifest,
        control_path: control,
        release_authority: authority,
        pack: content.pack(),
        product_version: env!("CARGO_PKG_VERSION"),
        product_source_commit: source_commit,
        home,
        now_unix,
    })
    .map_err(|_| "packaged-product-acceptance-product-verification-failed")?;
    create_private_tree(&home.join(".codex"))?;
    create_private_tree(&home.join(".claude"))?;
    let codex_legacy = home.join(".qiongli/plugins/codex/qiongli/legacy-canary");
    let claude_legacy =
        home.join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli/legacy-canary");
    write_private_tree_file(&codex_legacy, b"codex-legacy-preserved")?;
    write_private_tree_file(&claude_legacy, b"claude-legacy-preserved")?;

    for (index, target) in [
        ClientActivationTarget::Codex,
        ClientActivationTarget::ClaudeCode,
    ]
    .into_iter()
    .enumerate()
    {
        progress(match target {
            ClientActivationTarget::Codex => "codex-start",
            ClientActivationTarget::ClaudeCode => "claude-start",
        });
        let preview = preview_packaged_product_install(&product, target)
            .map_err(|_| "packaged-product-acceptance-preview-failed")?;
        if preview.effect != PackagedProductInstallEffect::Install || !preview.can_apply {
            return Err("packaged-product-acceptance-preview-invalid");
        }
        let committed = apply_packaged_product_install(
            content.pack(),
            &product,
            &preview,
            now_unix.saturating_add(index as u64 + 1),
        )
        .map_err(|_| "packaged-product-acceptance-apply-failed")?;
        if committed.disposition != PackagedProductInstallDisposition::Installed {
            return Err("packaged-product-acceptance-commit-invalid");
        }
        verify_packaged_product_install(&product, target)
            .map_err(|_| "packaged-product-acceptance-installed-verification-failed")?;
        progress(match target {
            ClientActivationTarget::Codex => "codex-installed",
            ClientActivationTarget::ClaudeCode => "claude-installed",
        });
        if target == ClientActivationTarget::Codex {
            let handle = discover_client_activation(home, None, target)
                .map_err(|_| "packaged-product-acceptance-repair-setup-failed")?;
            ClientActivationCoordinator::new(handle)
                .remove(now_unix.saturating_add(5))
                .map_err(|_| "packaged-product-acceptance-repair-setup-failed")?;
            let repair = preview_packaged_product_install(&product, target)
                .map_err(|_| "packaged-product-acceptance-repair-preview-failed")?;
            if repair.effect != PackagedProductInstallEffect::Repair || !repair.can_apply {
                return Err("packaged-product-acceptance-repair-preview-invalid");
            }
            apply_packaged_product_install(
                content.pack(),
                &product,
                &repair,
                now_unix.saturating_add(6),
            )
            .map_err(|_| "packaged-product-acceptance-repair-apply-failed")?;
            progress("codex-repaired");
        }
        let restarted = verify_packaged_product(&PackagedProductVerificationInput {
            current_executable: canonical,
            desktop_manifest_path: manifest,
            control_path: control,
            release_authority: authority,
            pack: content.pack(),
            product_version: env!("CARGO_PKG_VERSION"),
            product_source_commit: source_commit,
            home,
            now_unix,
        })
        .map_err(|_| "packaged-product-acceptance-product-restart-failed")?;
        verify_packaged_product_install(&restarted, target)
            .map_err(|_| "packaged-product-acceptance-restart-verification-failed")?;
        progress(match target {
            ClientActivationTarget::Codex => "codex-restarted",
            ClientActivationTarget::ClaudeCode => "claude-restarted",
        });
        let current = preview_packaged_product_install(&product, target)
            .map_err(|_| "packaged-product-acceptance-current-preview-failed")?;
        if current.effect != PackagedProductInstallEffect::AlreadyCurrent || !current.can_apply {
            return Err("packaged-product-acceptance-current-state-invalid");
        }
        remove_packaged_product_install(
            &product,
            target,
            now_unix.saturating_add(index as u64 + 10),
        )
        .map_err(|_| "packaged-product-acceptance-remove-failed")?;
        let absent = preview_packaged_product_install(&product, target)
            .map_err(|_| "packaged-product-acceptance-removed-preview-failed")?;
        if absent.effect != PackagedProductInstallEffect::Install {
            return Err("packaged-product-acceptance-removed-state-invalid");
        }
        progress(match target {
            ClientActivationTarget::Codex => "codex-removed",
            ClientActivationTarget::ClaudeCode => "claude-removed",
        });
    }
    if fs::read(&codex_legacy).ok().as_deref() != Some(b"codex-legacy-preserved")
        || fs::read(&claude_legacy).ok().as_deref() != Some(b"claude-legacy-preserved")
    {
        return Err("packaged-product-acceptance-legacy-content-drift");
    }
    Ok(())
}

fn verify_packaged_entrypoints(
    canonical: &Path,
    launcher: &Path,
    home: &Path,
) -> Result<(), &'static str> {
    let status = isolated_command(canonical, home, ["install", "status"])?;
    let status: Value = serde_json::from_slice(&status.stdout)
        .map_err(|_| "packaged-product-acceptance-install-status-invalid")?;
    if status["release_authority"] != "embedded" || status["source_commit"] != "embedded" {
        return Err("packaged-product-acceptance-embedded-product-invalid");
    }
    let inventory = isolated_command(canonical, home, ["install", "inventory"])?;
    let inventory_text = std::str::from_utf8(&inventory.stdout)
        .map_err(|_| "packaged-product-acceptance-inventory-invalid")?;
    let private_home = home
        .to_str()
        .ok_or("packaged-product-acceptance-inventory-invalid")?;
    if inventory_text.contains(private_home) {
        return Err("packaged-product-acceptance-inventory-path-leak");
    }
    let inventory: Value = serde_json::from_str(inventory_text)
        .map_err(|_| "packaged-product-acceptance-inventory-invalid")?;
    if inventory["command"] != "install-inventory"
        || inventory
            .pointer("/inventory/clients")
            .and_then(Value::as_array)
            .is_none_or(|clients| clients.len() != 2)
    {
        return Err("packaged-product-acceptance-inventory-invalid");
    }
    let startup = isolated_command(launcher, home, ["--startup-check"])?;
    let startup: Value = serde_json::from_slice(&startup.stdout)
        .map_err(|_| "packaged-product-acceptance-startup-invalid")?;
    if startup["command"] != "ui-startup-check" || startup["service"] != "ready" {
        return Err("packaged-product-acceptance-startup-invalid");
    }
    Ok(())
}

fn isolated_command<const N: usize>(
    executable: &Path,
    home: &Path,
    arguments: [&str; N],
) -> Result<Output, &'static str> {
    let mut command = Command::new(executable);
    command
        .args(arguments)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "")
        .current_dir(home);
    run_command(
        &mut command,
        "packaged-product-acceptance-entrypoint-failed",
    )
}

fn isolated_command_args(
    executable: &Path,
    home: &Path,
    arguments: &[OsString],
) -> Result<Output, &'static str> {
    let mut command = Command::new(executable);
    command
        .args(arguments)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "")
        .current_dir(home);
    run_command(
        &mut command,
        "packaged-product-acceptance-entrypoint-failed",
    )
}

fn ad_hoc_sign_canonical(canonical: &Path) -> Result<(), &'static str> {
    run_command(
        Command::new("/usr/bin/codesign").args([
            OsStr::new("--force"),
            OsStr::new("--options"),
            OsStr::new("runtime"),
            OsStr::new("--timestamp=none"),
            OsStr::new("--sign"),
            OsStr::new("-"),
            canonical.as_os_str(),
        ]),
        "packaged-product-acceptance-canonical-signing-failed",
    )?;
    run_command(
        Command::new("/usr/bin/codesign").args([
            OsStr::new("--verify"),
            OsStr::new("--strict"),
            canonical.as_os_str(),
        ]),
        "packaged-product-acceptance-canonical-signing-failed",
    )?;
    Ok(())
}

fn run_command(command: &mut Command, error: &'static str) -> Result<Output, &'static str> {
    let output = command.output().map_err(|_| error)?;
    if output.stdout.len().saturating_add(output.stderr.len()) > MAX_COMMAND_OUTPUT_BYTES
        || !output.status.success()
    {
        return Err(error);
    }
    Ok(output)
}

fn progress(stage: &'static str) {
    eprintln!("packaged-product-acceptance: {stage} passed");
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
    .map_err(|_| "packaged-product-acceptance-authority-invalid")
}

fn random_seed() -> Result<Zeroizing<[u8; 32]>, &'static str> {
    let mut seed = Zeroizing::new([0_u8; 32]);
    getrandom::fill(seed.as_mut()).map_err(|_| "packaged-product-acceptance-random-failed")?;
    Ok(seed)
}

fn stage_executable(source: &Path, destination: &Path) -> Result<(), &'static str> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|_| "packaged-product-acceptance-component-invalid")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
        return Err("packaged-product-acceptance-component-invalid");
    }
    fs::copy(source, destination)
        .map_err(|_| "packaged-product-acceptance-component-stage-failed")?;
    set_executable(destination)
}

fn read_json(path: &Path) -> Result<Value, &'static str> {
    serde_json::from_slice(&read_bounded(path, MAX_JSON_BYTES)?)
        .map_err(|_| "packaged-product-acceptance-json-invalid")
}

fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "packaged-product-acceptance-input-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err("packaged-product-acceptance-input-invalid");
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "packaged-product-acceptance-input-invalid")?
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "packaged-product-acceptance-input-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("packaged-product-acceptance-input-invalid");
    }
    Ok(bytes)
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(unix)]
fn write_new_private(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "packaged-product-acceptance-output-invalid")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "packaged-product-acceptance-output-invalid")
}

#[cfg(not(unix))]
fn write_new_private(_path: &Path, _bytes: &[u8]) -> Result<(), &'static str> {
    Err("packaged-product-acceptance-macos-required")
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt as _;

    let mut builder = fs::DirBuilder::new();
    builder
        .mode(0o700)
        .create(path)
        .map_err(|_| "packaged-product-acceptance-directory-invalid")
}

#[cfg(not(unix))]
fn create_private_directory(_path: &Path) -> Result<(), &'static str> {
    Err("packaged-product-acceptance-macos-required")
}

fn create_private_child(root: &Path, leaf: &str) -> Result<PathBuf, &'static str> {
    let path = root.join(leaf);
    create_private_directory(&path)?;
    Ok(path)
}

#[cfg(unix)]
fn create_private_tree(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};

    let mut builder = fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder
        .create(path)
        .map_err(|_| "packaged-product-acceptance-directory-invalid")?;
    let mut current = Some(path);
    while let Some(directory) = current {
        if directory.exists() {
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
                .map_err(|_| "packaged-product-acceptance-directory-invalid")?;
        }
        current = directory.parent().filter(|parent| parent.starts_with(path));
    }
    Ok(())
}

#[cfg(not(unix))]
fn create_private_tree(_path: &Path) -> Result<(), &'static str> {
    Err("packaged-product-acceptance-macos-required")
}

fn write_private_tree_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let parent = path
        .parent()
        .ok_or("packaged-product-acceptance-directory-invalid")?;
    create_private_tree(parent)?;
    write_new_private(path, bytes)
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt as _;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| "packaged-product-acceptance-component-stage-failed")
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), &'static str> {
    Err("packaged-product-acceptance-macos-required")
}

fn decode_hex(value: &str) -> Result<Vec<u8>, &'static str> {
    if value.is_empty() || !value.len().is_multiple_of(2) || !valid_lower_hex(value, value.len()) {
        return Err("packaged-product-acceptance-signing-request-invalid");
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("packaged-product-acceptance-signing-request-invalid"),
    }
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && valid_lower_hex(value, value.len())
}

fn sha256_file(path: &Path) -> Result<String, &'static str> {
    Ok(sha256_hex(&read_bounded(path, 512 * 1024 * 1024)?))
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
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

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "packaged-product-acceptance-clock-invalid")
}

#[derive(Serialize)]
struct AcceptanceReceiptV1<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    product_source_commit: &'a str,
    canonical_sha256: &'a str,
    product_control_sha256: String,
    signed_archive_sha256: String,
    checks: AcceptanceChecksV1,
}

#[derive(Serialize)]
struct AcceptanceChecksV1 {
    embedded_authority: bool,
    canonical_signature_preserved: bool,
    product_control_verified: bool,
    inventory_discovered: bool,
    skills_materialize_verify_refresh: bool,
    lite_mcp_self_test: bool,
    provider_keychain_save_replace_restart_remove: bool,
    codex_install_verify_remove: bool,
    claude_install_verify_remove: bool,
    registration_repair: bool,
    packaged_restart_verification: bool,
    legacy_content_preserved: bool,
    empty_path_startup: bool,
}
