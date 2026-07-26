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
use qiongli::FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES;
use qiongli_config::{
    ConfigRoot, GLOBAL_SETTINGS_FILE, GlobalSettingsStore, SecretRef, SecretStoreStatus,
    SecretValue, resolve_config_root,
};
use qiongli_content::{approve_materialization_target, verify_materialization};
use qiongli_platform::{
    ClientActivationCoordinator, ClientActivationTarget, NativeReleaseAuthority,
    PackagedProductInstallDisposition, PackagedProductInstallEffect,
    PackagedProductVerificationInput, apply_packaged_product_install, discover_client_activation,
    preview_packaged_product_install, remove_packaged_product_install, verify_packaged_product,
    verify_packaged_product_install,
};
use qiongli_project::{
    AcademicGraphConfidence, AcademicGraphEdgeStatus, AcademicGraphEdgeV1,
    AcademicGraphIdentityScope, AcademicGraphLayer, AcademicGraphNodeType, AcademicGraphNodeV1,
    AcademicGraphRelation, AcademicInferenceStrength, ApprovedCaptureIntake, CaptureArea,
    CaptureDelivery, CaptureDeliveryAcknowledgementRequestV1, CaptureDeliveryDestinationV1,
    CaptureDeliveryEnvelopeV1, CapturePolicy, CaptureSource, ContradictionV1, DecisionCandidateV1,
    DecisionRelation, EvidenceLocatorKind, EvidenceReferenceV1, PortfolioQueryV1, ProjectBindingV1,
    ProjectId, ProjectStage, ProjectStateService, ResearchCaptureDraftV1, ResearchCaptureV1,
    SemanticChangeV1, SemanticTimelineQueryV1,
};
use qiongli_runtime::mcp::MCP_PROTOCOL_VERSION;
use qiongli_runtime::{FULL_PROJECT_PUBLIC_TOOL_NAMES, LITE_PUBLIC_TOOL_NAMES};
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
    let values = env::args_os().skip(1).collect::<Vec<_>>();
    if values.first().and_then(|value| value.to_str()) == Some("--legacy-only") {
        return run_legacy_only(&values[1..]);
    }
    let arguments = Arguments::parse(values)?;
    create_private_directory(&arguments.output)?;
    let authority_root = create_private_child(&arguments.output, "authority")?;
    let components_root = create_private_child(&arguments.output, "components")?;
    let preliminary_root = arguments.output.join("preliminary-package");
    let final_root = arguments.output.join("product-package");
    let signed_root = arguments.output.join("signed-product");
    let request_root = create_private_child(&arguments.output, "product-control")?;
    let extracted_root = create_private_child(&arguments.output, "extracted")?;
    let home = create_private_child(&arguments.output, "automated-home")?;
    let manual_home = create_private_child(&arguments.output, "manual-home")?;
    create_private_tree(&manual_home.join(".codex"))?;
    create_private_tree(&manual_home.join(".claude"))?;

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

    let signed_archive = signed_root.join(format!(
        "Qiongli-{}-macOS-arm64.zip",
        env!("CARGO_PKG_VERSION")
    ));
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
    let continuity = exercise_project_state_lifecycle(&packaged_canonical, &home)?;
    progress("project-state");
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
    let migration_home = create_private_child(&arguments.output, "legacy-migration-home")?;
    exercise_legacy_migration_lifecycle(&packaged_canonical, &migration_home)?;
    progress("legacy-migration");

    let signing_receipt = read_json(&signed_root.join("qiongli-macos-signing.receipt.json"))?;
    if signing_receipt["signing"]["canonical_signature_preserved"] != true {
        return Err("packaged-product-acceptance-signing-receipt-invalid");
    }
    let receipt = AcceptanceReceiptV2 {
        schema_version: 2,
        record_type: "qiongli-packaged-product-acceptance",
        status: "accepted-ad-hoc-nonpublishing",
        publication_allowed: false,
        product_source_commit: &arguments.source_commit,
        canonical_sha256: &canonical_sha256,
        product_control_sha256: sha256_file(&control)?,
        signed_archive_sha256: sha256_file(&signed_archive)?,
        continuity,
        checks: AcceptanceChecksV1 {
            embedded_authority: true,
            canonical_signature_preserved: true,
            product_control_verified: true,
            inventory_discovered: true,
            skills_materialize_verify_refresh: true,
            lite_mcp_self_test: true,
            project_three_project_restart: true,
            project_app_cli_library_full_mcp_parity: true,
            continuity_delivery_restart_replay: true,
            continuity_assignment_resolution: true,
            continuity_archive_restore_rebuild: true,
            continuity_catalog_query_timeline: true,
            continuity_path_redacted: true,
            provider_keychain_save_replace_restart_remove: true,
            codex_install_verify_remove: true,
            claude_install_verify_remove: true,
            registration_repair: true,
            packaged_restart_verification: true,
            legacy_migration_fixture_isolated: true,
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

fn run_legacy_only(values: &[OsString]) -> Result<(), &'static str> {
    if values.len() != 4 {
        return Err("packaged-product-acceptance-legacy-only-usage-invalid");
    }
    let mut canonical = None;
    let mut home = None;
    for pair in values.chunks_exact(2) {
        match pair[0].to_str() {
            Some("--canonical") if canonical.is_none() => {
                canonical = Some(PathBuf::from(&pair[1]));
            }
            Some("--home") if home.is_none() => {
                home = Some(PathBuf::from(&pair[1]));
            }
            _ => return Err("packaged-product-acceptance-legacy-only-usage-invalid"),
        }
    }
    let canonical = canonical.ok_or("packaged-product-acceptance-legacy-only-usage-invalid")?;
    let home = home.ok_or("packaged-product-acceptance-legacy-only-usage-invalid")?;
    let canonical_metadata = fs::symlink_metadata(&canonical)
        .map_err(|_| "packaged-product-acceptance-legacy-only-usage-invalid")?;
    if !canonical.is_absolute()
        || canonical_metadata.file_type().is_symlink()
        || !canonical_metadata.is_file()
        || canonical_metadata.len() == 0
        || !home.is_absolute()
        || home.exists()
        || home.parent().is_none_or(|parent| !parent.is_dir())
    {
        return Err("packaged-product-acceptance-legacy-only-usage-invalid");
    }
    create_private_directory(&home)?;
    exercise_legacy_migration_lifecycle(&canonical, &home)?;
    println!("{{\"schema_version\":1,\"status\":\"legacy-migration-accepted\"}}");
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
        OsStr::new("--features"),
        OsStr::new("custom-protocol"),
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

fn exercise_project_state_lifecycle(
    canonical: &Path,
    home: &Path,
) -> Result<ContinuityEvidenceV1, &'static str> {
    let projects_root = home.join("r4a-projects");
    create_private_tree(&projects_root)?;
    let mut fixtures = Vec::new();
    for (index, name) in ["Evidence Atlas", "Method Notes", "Draft Synthesis"]
        .into_iter()
        .enumerate()
    {
        let project_root = projects_root.join(format!("paper-{}", index + 1));
        let preview_arguments = vec![
            OsString::from("project"),
            OsString::from("create"),
            OsString::from("preview"),
            OsString::from("--root"),
            project_root.as_os_str().to_owned(),
            OsString::from("--name"),
            OsString::from(name),
            OsString::from("--kind"),
            OsString::from("article"),
            OsString::from("--stage"),
            OsString::from("writing"),
        ];
        let preview = isolated_command_args(canonical, home, &preview_arguments)?;
        reject_project_path_output(&preview, &project_root)?;
        let preview = parse_command_json(
            &preview,
            "packaged-product-acceptance-project-preview-invalid",
        )?;
        let project_id = preview
            .pointer("/preview/projectId")
            .and_then(Value::as_str)
            .ok_or("packaged-product-acceptance-project-preview-invalid")?;
        let plan_digest = preview
            .pointer("/preview/planDigest")
            .and_then(Value::as_str)
            .ok_or("packaged-product-acceptance-project-preview-invalid")?;
        let apply_arguments = vec![
            OsString::from("project"),
            OsString::from("create"),
            OsString::from("apply"),
            OsString::from("--root"),
            project_root.as_os_str().to_owned(),
            OsString::from("--name"),
            OsString::from(name),
            OsString::from("--kind"),
            OsString::from("article"),
            OsString::from("--stage"),
            OsString::from("writing"),
            OsString::from("--project-id"),
            OsString::from(project_id),
            OsString::from("--expected-plan-digest"),
            OsString::from(plan_digest),
            OsString::from("--approve-filesystem-write"),
        ];
        let applied = isolated_command_args(canonical, home, &apply_arguments)?;
        reject_project_path_output(&applied, &project_root)?;
        if parse_command_json(
            &applied,
            "packaged-product-acceptance-project-apply-invalid",
        )?["command"]
            != "project-create-apply"
        {
            return Err("packaged-product-acceptance-project-apply-invalid");
        }
        fixtures.push(AcceptanceProject {
            project_id: ProjectId::parse(project_id.to_string())
                .map_err(|_| "packaged-product-acceptance-project-apply-invalid")?,
            root: project_root,
            display_name: name,
        });
    }

    write_continuity_semantic_fixtures(&fixtures)?;
    for project in &fixtures {
        apply_project_lifecycle(canonical, home, "refresh", &project.project_id)?;
    }
    progress("project-fixture");

    let config_root = resolve_config_root(None, home)
        .map_err(|_| "packaged-product-acceptance-project-config-invalid")?;
    let continuity = exercise_capture_continuity(canonical, home, &config_root, &fixtures)?;
    apply_project_lifecycle(canonical, home, "archive", &fixtures[2].project_id)?;
    apply_project_lifecycle(canonical, home, "restore", &fixtures[2].project_id)?;

    let first_rebuild = apply_portfolio_mutation(canonical, home, "rebuild")?;
    let first_portfolio = first_rebuild
        .pointer("/reconciliation/snapshot/portfolio")
        .cloned()
        .ok_or("packaged-product-acceptance-project-portfolio-invalid")?;
    verify_continuity_portfolio(&first_portfolio)?;
    let project_artifact_digest = continuity_project_artifact_digest(&fixtures)?;

    let deletion = apply_portfolio_mutation(canonical, home, "delete-derived-state")?;
    if deletion["command"] != "project-portfolio-delete-derived-state-apply"
        || deletion
            .pointer("/deletion/removedContributionCount")
            .and_then(Value::as_u64)
            != Some(3)
    {
        return Err("packaged-product-acceptance-project-portfolio-delete-invalid");
    }
    let doctor = isolated_command(canonical, home, ["project", "portfolio", "doctor"])?;
    let doctor = parse_command_json(
        &doctor,
        "packaged-product-acceptance-project-portfolio-doctor-invalid",
    )?;
    if doctor.pointer("/doctor/status").and_then(Value::as_str) != Some("missing") {
        return Err("packaged-product-acceptance-project-portfolio-doctor-invalid");
    }
    let second_rebuild = apply_portfolio_mutation(canonical, home, "rebuild")?;
    let second_portfolio = second_rebuild
        .pointer("/reconciliation/snapshot/portfolio")
        .cloned()
        .ok_or("packaged-product-acceptance-project-portfolio-invalid")?;
    if second_portfolio != first_portfolio
        || continuity_project_artifact_digest(&fixtures)? != project_artifact_digest
    {
        return Err("packaged-product-acceptance-project-portfolio-rebuild-drift");
    }

    let catalog_id = second_rebuild
        .pointer("/reconciliation/snapshot/catalog/catalogId")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-project-portfolio-invalid")?;
    let query = PortfolioQueryV1::new(catalog_id)
        .and_then(|query| query.to_canonical_json())
        .map_err(|_| "packaged-product-acceptance-project-query-invalid")?;
    let query = String::from_utf8(query)
        .map_err(|_| "packaged-product-acceptance-project-query-invalid")?;
    let query_output = isolated_command_args(
        canonical,
        home,
        &[
            OsString::from("project"),
            OsString::from("portfolio"),
            OsString::from("query"),
            OsString::from("--request-json"),
            OsString::from(query),
        ],
    )?;
    let query_output = parse_command_json(
        &query_output,
        "packaged-product-acceptance-project-query-invalid",
    )?;
    let matched_project_count = query_output
        .pointer("/result/matchedProjectCount")
        .and_then(Value::as_u64)
        .ok_or("packaged-product-acceptance-project-query-invalid")?;
    let matched_lineage_count = query_output
        .pointer("/result/matchedLineageCount")
        .and_then(Value::as_u64)
        .ok_or("packaged-product-acceptance-project-query-invalid")?;
    if matched_project_count != 3 || matched_lineage_count < 3 {
        return Err("packaged-product-acceptance-project-query-invalid");
    }

    let timeline = SemanticTimelineQueryV1::new(catalog_id)
        .and_then(|query| query.to_canonical_json())
        .map_err(|_| "packaged-product-acceptance-project-timeline-invalid")?;
    let timeline = String::from_utf8(timeline)
        .map_err(|_| "packaged-product-acceptance-project-timeline-invalid")?;
    let timeline_output = isolated_command_args(
        canonical,
        home,
        &[
            OsString::from("project"),
            OsString::from("portfolio"),
            OsString::from("timeline"),
            OsString::from("--request-json"),
            OsString::from(timeline),
        ],
    )?;
    let timeline_output = parse_command_json(
        &timeline_output,
        "packaged-product-acceptance-project-timeline-invalid",
    )?;
    let timeline_event_count = timeline_output
        .pointer("/result/matchedEventCount")
        .and_then(Value::as_u64)
        .ok_or("packaged-product-acceptance-project-timeline-invalid")?;
    if timeline_event_count < 3 {
        return Err("packaged-product-acceptance-project-timeline-invalid");
    }

    let cli = isolated_command(canonical, home, ["project", "list"])?;
    let cli = parse_command_json(&cli, "packaged-product-acceptance-project-list-invalid")?;
    let library = cli
        .get("library")
        .ok_or("packaged-product-acceptance-project-list-invalid")?;
    if library
        .get("projects")
        .and_then(Value::as_array)
        .is_none_or(|projects| {
            projects.len() != 3
                || projects.iter().any(|project| {
                    project.get("health") != Some(&Value::String("ready".to_string()))
                        || project.get("lifecycle") != Some(&Value::String("active".to_string()))
                })
        })
    {
        return Err("packaged-product-acceptance-project-list-invalid");
    }

    let app = isolated_command(canonical, home, ["app", "snapshot"])?;
    let app = parse_command_json(&app, "packaged-product-acceptance-project-app-invalid")?;
    if app.get("researchLibrary") != Some(library) {
        return Err("packaged-product-acceptance-project-app-drift");
    }

    let full = run_full_project_mcp(canonical, home)?;
    if full.library != *library || full.portfolio != second_portfolio {
        return Err("packaged-product-acceptance-project-mcp-drift");
    }
    Ok(ContinuityEvidenceV1 {
        project_count: 3,
        shared_source_identity_count: 1,
        shared_concept_identity_count: 1,
        shared_method_identity_count: 1,
        reviewed_lineage_count: 1,
        delivery_record_count: continuity.delivery_record_count,
        retry_count: 1,
        acknowledgement_replay_count: 1,
        duplicate_suppression_count: 1,
        assignment_count: continuity.assignment_count,
        resolution_count: continuity.resolution_count,
        resolution_item_count: 5,
        archive_count: 1,
        restore_count: 1,
        derived_deletion_count: 1,
        full_rebuild_count: 2,
        matched_query_project_count: matched_project_count,
        matched_query_lineage_count: matched_lineage_count,
        timeline_event_count,
        app_cli_library_parity: true,
        full_mcp_library_portfolio_parity: true,
        canonical_project_artifacts_unchanged_by_derived_rebuild: true,
        path_redacted: true,
    })
}

#[derive(Clone)]
struct AcceptanceProject {
    project_id: ProjectId,
    root: PathBuf,
    display_name: &'static str,
}

struct CaptureContinuityCounts {
    delivery_record_count: u64,
    assignment_count: u64,
    resolution_count: u64,
}

fn write_continuity_semantic_fixtures(projects: &[AcceptanceProject]) -> Result<(), &'static str> {
    if projects.len() != 3 {
        return Err("packaged-product-acceptance-project-fixture-invalid");
    }
    for (index, project) in projects.iter().enumerate() {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        if index < 2 {
            nodes.push(
                AcademicGraphNodeV1::new(
                    &project.project_id,
                    AcademicGraphNodeType::Paper,
                    AcademicGraphIdentityScope::Global,
                    "doi:10.5555/qiongli-c5-shared",
                    "Shared continuity source",
                    vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
                    "graph/semantic_links.jsonl",
                    "line:1",
                )
                .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?,
            );
            nodes.push(
                AcademicGraphNodeV1::new(
                    &project.project_id,
                    AcademicGraphNodeType::Concept,
                    AcademicGraphIdentityScope::Global,
                    "concept:qiongli-c5-continuity",
                    "Cross-surface continuity",
                    vec![
                        AcademicGraphLayer::IdeaDecision,
                        AcademicGraphLayer::Combined,
                    ],
                    "graph/semantic_links.jsonl",
                    "line:2",
                )
                .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?,
            );
        }
        if index > 0 {
            nodes.push(
                AcademicGraphNodeV1::new(
                    &project.project_id,
                    AcademicGraphNodeType::Method,
                    AcademicGraphIdentityScope::Global,
                    "method:qiongli-c5-restart-protocol",
                    "Restart-safe acceptance protocol",
                    vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
                    "graph/semantic_links.jsonl",
                    "line:3",
                )
                .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?,
            );
        }
        if index == 1 {
            let local = AcademicGraphNodeV1::new(
                &project.project_id,
                AcademicGraphNodeType::Project,
                AcademicGraphIdentityScope::Project,
                project.project_id.as_str(),
                project.display_name,
                vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
                "context/project_manifest.json",
                "#/project_id",
            )
            .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?;
            let parent = AcademicGraphNodeV1::new(
                &project.project_id,
                AcademicGraphNodeType::Project,
                AcademicGraphIdentityScope::Global,
                projects[0].project_id.as_str(),
                "Reviewed parent project",
                vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
                "graph/semantic_links.jsonl",
                "line:4",
            )
            .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?;
            edges.push(
                AcademicGraphEdgeV1::new(
                    &project.project_id,
                    &local.node_id,
                    AcademicGraphRelation::ForkedFrom,
                    &parent.node_id,
                    vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
                    "A reviewed lineage record connects the two projects.",
                    "graph/semantic_links.jsonl",
                    "line:5",
                    "Lineage does not imply identical academic conclusions.",
                    AcademicInferenceStrength::DirectEvidence,
                    AcademicGraphConfidence::High,
                    AcademicGraphEdgeStatus::Reviewed,
                    None,
                )
                .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?,
            );
            nodes.push(parent);
        }
        let mut bytes = Vec::new();
        for node in &nodes {
            append_canonical_json_line(
                &mut bytes,
                &json!({
                    "schema_version": 1,
                    "document_kind": "qiongli-academic-graph-node",
                    "project_id": project.project_id,
                    "node_id": node.node_id,
                    "node_type": node.node_type,
                    "identity_scope": node.identity_scope,
                    "canonical_id": node.canonical_id,
                    "label": node.label,
                    "layers": node.layers,
                    "artifact_path": node.artifact_path,
                    "source_anchor": node.source_anchor,
                }),
            )?;
        }
        for edge in &edges {
            append_canonical_json_line(
                &mut bytes,
                &json!({
                    "schema_version": 1,
                    "document_kind": "qiongli-academic-semantic-link",
                    "project_id": project.project_id,
                    "edge_id": edge.edge_id,
                    "source_node_id": edge.source_node_id,
                    "relation": edge.relation,
                    "target_node_id": edge.target_node_id,
                    "layers": edge.layers,
                    "rationale": edge.rationale,
                    "artifact_path": edge.artifact_path,
                    "source_anchor": edge.source_anchor,
                    "evidence_limit": edge.evidence_limit,
                    "inference_strength": edge.inference_strength,
                    "confidence": edge.confidence,
                    "status": edge.status,
                    "created_from_capture": edge.created_from_capture,
                }),
            )?;
        }
        write_private_tree_file(&project.root.join("graph/semantic_links.jsonl"), &bytes)?;
    }
    Ok(())
}

fn append_canonical_json_line<T: Serialize>(
    output: &mut Vec<u8>,
    value: &T,
) -> Result<(), &'static str> {
    output.extend(
        serde_json_canonicalizer::to_vec(value)
            .map_err(|_| "packaged-product-acceptance-project-fixture-invalid")?,
    );
    output.push(b'\n');
    Ok(())
}

fn exercise_capture_continuity(
    canonical: &Path,
    home: &Path,
    config_root: &ConfigRoot,
    projects: &[AcceptanceProject],
) -> Result<CaptureContinuityCounts, &'static str> {
    let service = ProjectStateService::new(config_root.clone());
    let first_revision = project_revision(&service, &projects[0].project_id)?;
    let offline_capture = continuity_capture(
        &projects[0].project_id,
        first_revision,
        1_800_020_001,
        false,
    )?;
    let offline_envelope = CaptureDeliveryEnvelopeV1::new(
        offline_capture.clone(),
        Some(
            CaptureDeliveryDestinationV1::new(projects[0].project_id.clone(), first_revision)
                .map_err(|_| "packaged-product-acceptance-delivery-invalid")?,
        ),
        1_800_020_010,
    )
    .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    let queued = service
        .enqueue_capture_delivery(offline_envelope.clone())
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    let delivering = service
        .begin_capture_delivery(
            &offline_envelope.envelope_id,
            queued.generation,
            &queued.record_sha256,
            1_800_020_011,
        )
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;

    let inspected = isolated_command_args(
        canonical,
        home,
        &delivery_inspect_arguments(&offline_envelope.envelope_id),
    )?;
    if parse_command_json(&inspected, "packaged-product-acceptance-delivery-invalid")?
        .pointer("/delivery/state")
        .and_then(Value::as_str)
        != Some("delivering")
    {
        return Err("packaged-product-acceptance-delivery-invalid");
    }
    let retried = isolated_command_args(
        canonical,
        home,
        &delivery_retry_arguments(
            &offline_envelope.envelope_id,
            delivering.generation,
            &delivering.record_sha256,
        ),
    )?;
    let retried = parse_command_json(&retried, "packaged-product-acceptance-delivery-invalid")?;
    if retried.pointer("/delivery/state").and_then(Value::as_str) != Some("retry-required") {
        return Err("packaged-product-acceptance-delivery-invalid");
    }
    let retry_generation = retried
        .pointer("/delivery/generation")
        .and_then(Value::as_u64)
        .ok_or("packaged-product-acceptance-delivery-invalid")?;
    let retry_digest = retried
        .pointer("/delivery/recordSha256")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-delivery-invalid")?;
    let restarted = ProjectStateService::new(config_root.clone());
    let delivering_again = restarted
        .begin_capture_delivery(
            &offline_envelope.envelope_id,
            retry_generation,
            retry_digest,
            1_800_020_013,
        )
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    let intake = restarted
        .preview_capture(offline_capture)
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    restarted
        .apply_capture(
            &intake,
            &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
            1_800_020_014,
        )
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    let delivered = restarted
        .record_capture_delivery(
            &offline_envelope.envelope_id,
            delivering_again.generation,
            &delivering_again.record_sha256,
            1_800_020_015,
        )
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    let acknowledgement_request = CaptureDeliveryAcknowledgementRequestV1 {
        envelope_id: offline_envelope.envelope_id.clone(),
        destination_project_id: projects[0].project_id.clone(),
        accepted_capture_id: offline_envelope.capture_id.clone(),
        expected_project_revision: first_revision,
        resulting_project_revision: first_revision,
        acknowledged_at_unix: 1_800_020_016,
    };
    let acknowledged = restarted
        .acknowledge_capture_delivery(
            &acknowledgement_request,
            delivered.generation,
            &delivered.record_sha256,
        )
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    let replayed = ProjectStateService::new(config_root.clone())
        .acknowledge_capture_delivery(
            &acknowledgement_request,
            delivered.generation,
            &delivered.record_sha256,
        )
        .map_err(|_| "packaged-product-acceptance-delivery-invalid")?;
    if acknowledged != replayed {
        return Err("packaged-product-acceptance-delivery-replay-drift");
    }
    let reopened = isolated_command_args(
        canonical,
        home,
        &delivery_inspect_arguments(&offline_envelope.envelope_id),
    )?;
    if parse_command_json(&reopened, "packaged-product-acceptance-delivery-invalid")?
        .pointer("/delivery/state")
        .and_then(Value::as_str)
        != Some("acknowledged")
    {
        return Err("packaged-product-acceptance-delivery-invalid");
    }
    progress("continuity-delivery");

    let duplicate_envelope =
        CaptureDeliveryEnvelopeV1::new(offline_envelope.capture.clone(), None, 1_800_020_020)
            .map_err(|_| "packaged-product-acceptance-assignment-invalid")?;
    restarted
        .enqueue_capture_delivery(duplicate_envelope.clone())
        .map_err(|_| "packaged-product-acceptance-assignment-invalid")?;
    let duplicate_preview = isolated_command_args(
        canonical,
        home,
        &assignment_arguments(
            "preview",
            duplicate_envelope.envelope_id.as_str(),
            &projects[0].project_id,
            1_800_020_021,
            None,
        ),
    )?;
    if parse_command_json(
        &duplicate_preview,
        "packaged-product-acceptance-assignment-invalid",
    )?
    .pointer("/preview/outcome")
    .and_then(Value::as_str)
        != Some("duplicate")
    {
        return Err("packaged-product-acceptance-assignment-invalid");
    }
    progress("continuity-duplicate");

    let second_revision = project_revision(&restarted, &projects[1].project_id)?;
    let divergent_capture =
        continuity_capture(&projects[0].project_id, first_revision, 1_800_020_030, true)?;
    let divergent_envelope = CaptureDeliveryEnvelopeV1::new(divergent_capture, None, 1_800_020_031)
        .map_err(|_| "packaged-product-acceptance-assignment-invalid")?;
    restarted
        .enqueue_capture_delivery(divergent_envelope.clone())
        .map_err(|_| "packaged-product-acceptance-assignment-invalid")?;
    let assignment_preview = isolated_command_args(
        canonical,
        home,
        &assignment_arguments(
            "preview",
            divergent_envelope.envelope_id.as_str(),
            &projects[1].project_id,
            1_800_020_032,
            None,
        ),
    )?;
    let assignment_preview = parse_command_json(
        &assignment_preview,
        "packaged-product-acceptance-assignment-invalid",
    )?;
    if assignment_preview
        .pointer("/preview/bindingEffect")
        .and_then(Value::as_str)
        != Some("rebound")
        || assignment_preview
            .pointer("/preview/expectedProjectRevision")
            .and_then(Value::as_u64)
            != Some(second_revision)
    {
        return Err("packaged-product-acceptance-assignment-invalid");
    }
    progress("continuity-assignment-preview");
    let assignment_digest = assignment_preview
        .pointer("/preview/planDigest")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-assignment-invalid")?;
    let assignment = isolated_command_args(
        canonical,
        home,
        &assignment_arguments(
            "apply",
            divergent_envelope.envelope_id.as_str(),
            &projects[1].project_id,
            1_800_020_032,
            Some(assignment_digest),
        ),
    )?;
    let assignment = parse_command_json(
        &assignment,
        "packaged-product-acceptance-assignment-invalid",
    )?;
    let assignment_receipt_id = assignment
        .pointer("/commit/receiptId")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-assignment-invalid")?;
    progress("continuity-assignment");
    let resolution_preview = isolated_command_args(
        canonical,
        home,
        &resolution_preview_arguments(assignment_receipt_id, &[]),
    )?;
    let resolution_preview = parse_command_json(
        &resolution_preview,
        "packaged-product-acceptance-resolution-invalid",
    )?;
    progress("continuity-resolution-preview");
    let selections = resolution_selections(&resolution_preview)?;
    let selected_preview = isolated_command_args(
        canonical,
        home,
        &resolution_preview_arguments(assignment_receipt_id, &selections),
    )?;
    let selected_preview = parse_command_json(
        &selected_preview,
        "packaged-product-acceptance-resolution-invalid",
    )?;
    progress("continuity-resolution-selection");
    let plan_digest = selected_preview
        .pointer("/preview/planDigest")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-resolution-invalid")?;
    let selection_digest = selected_preview
        .pointer("/selectionSet/selectionDigest")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-resolution-invalid")?;
    let resolution_arguments = resolution_apply_arguments(
        assignment_receipt_id,
        &selections,
        plan_digest,
        selection_digest,
    );
    let resolution = isolated_command_args(canonical, home, &resolution_arguments)?;
    let resolution = parse_command_json(
        &resolution,
        "packaged-product-acceptance-resolution-invalid",
    )?;
    if resolution
        .pointer("/commit/childState")
        .and_then(Value::as_str)
        != Some("acknowledged")
        || resolution
            .pointer("/commit/fromProjectRevision")
            .and_then(Value::as_u64)
            != Some(second_revision)
    {
        return Err("packaged-product-acceptance-resolution-invalid");
    }
    progress("continuity-resolution");
    let exact_replay = isolated_command_args(canonical, home, &resolution_arguments)?;
    if parse_command_json(
        &exact_replay,
        "packaged-product-acceptance-resolution-invalid",
    )?
    .pointer("/commit/exactReplay")
    .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("packaged-product-acceptance-resolution-replay-invalid");
    }

    let delivery_list =
        isolated_command(canonical, home, ["project", "capture", "delivery", "list"])?;
    let delivery_record_count = parse_command_json(
        &delivery_list,
        "packaged-product-acceptance-delivery-invalid",
    )?
    .get("deliveries")
    .and_then(Value::as_array)
    .map(|values| values.len() as u64)
    .ok_or("packaged-product-acceptance-delivery-invalid")?;
    let assignment_list = isolated_command(
        canonical,
        home,
        ["project", "capture", "assignment", "list"],
    )?;
    let assignment_count = parse_command_json(
        &assignment_list,
        "packaged-product-acceptance-assignment-invalid",
    )?
    .get("assignments")
    .and_then(Value::as_array)
    .map(|values| values.len() as u64)
    .ok_or("packaged-product-acceptance-assignment-invalid")?;
    let resolution_list = isolated_command_args(
        canonical,
        home,
        &[
            OsString::from("project"),
            OsString::from("capture"),
            OsString::from("resolution"),
            OsString::from("list"),
            OsString::from("--project-id"),
            OsString::from(projects[1].project_id.as_str()),
        ],
    )?;
    let resolution_count = parse_command_json(
        &resolution_list,
        "packaged-product-acceptance-resolution-invalid",
    )?
    .get("resolutions")
    .and_then(Value::as_array)
    .map(|values| values.len() as u64)
    .ok_or("packaged-product-acceptance-resolution-invalid")?;
    if delivery_record_count < 4 || assignment_count != 1 || resolution_count != 1 {
        return Err("packaged-product-acceptance-continuity-count-invalid");
    }
    Ok(CaptureContinuityCounts {
        delivery_record_count,
        assignment_count,
        resolution_count,
    })
}

fn project_revision(
    service: &ProjectStateService,
    project_id: &ProjectId,
) -> Result<u64, &'static str> {
    service
        .snapshot()
        .map_err(|_| "packaged-product-acceptance-project-list-invalid")?
        .projects
        .iter()
        .find(|project| &project.project_id == project_id)
        .map(|project| project.semantic_revision)
        .ok_or("packaged-product-acceptance-project-list-invalid")
}

fn continuity_capture(
    project_id: &ProjectId,
    base_revision: u64,
    captured_at_unix: u64,
    divergent: bool,
) -> Result<ResearchCaptureV1, &'static str> {
    ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            project_id.clone(),
            base_revision,
            ProjectStage::Writing,
            "Qualify one bounded packaged continuity capture",
            CapturePolicy::ReviewRequired,
        )
        .map_err(|_| "packaged-product-acceptance-capture-invalid")?,
        source: if divergent {
            CaptureSource::Codex
        } else {
            CaptureSource::PortableFile
        },
        delivery: if divergent {
            CaptureDelivery::Connected
        } else {
            CaptureDelivery::Portable
        },
        captured_at_unix,
        summary: if divergent {
            "Review a divergent unbound capture after restart."
        } else {
            "Replay one offline capture after restart."
        }
        .to_string(),
        changes: vec![SemanticChangeV1 {
            area: CaptureArea::Thesis,
            summary: "Preserve deterministic continuity across packaged process restarts."
                .to_string(),
        }],
        decisions: vec![DecisionCandidateV1 {
            relation: DecisionRelation::Refinement,
            statement: "Use explicit revision-bound continuity decisions.".to_string(),
            rationale: "The packaged acceptance requires durable lineage.".to_string(),
            target: Some("decision:packaged-continuity".to_string()),
        }],
        evidence: vec![EvidenceReferenceV1 {
            locator_kind: EvidenceLocatorKind::Doi,
            locator: "10.5555/qiongli-c5-shared".to_string(),
            relevance: "Supports the deterministic continuity fixture.".to_string(),
            limitation: divergent.then(|| "Requires explicit item review.".to_string()),
        }],
        contradictions: vec![ContradictionV1 {
            statement: "Implicit overwrite is unsafe.".to_string(),
            conflicts_with: "Unreviewed stale project state.".to_string(),
            consequence: "Select an explicit resolution for every item.".to_string(),
        }],
        next_actions: vec!["Inspect the durable resolution receipt.".to_string()],
    }
    .into_capture()
    .map_err(|_| "packaged-product-acceptance-capture-invalid")
}

fn delivery_inspect_arguments(envelope_id: &qiongli_project::DeliveryEnvelopeId) -> Vec<OsString> {
    vec![
        "project".into(),
        "capture".into(),
        "delivery".into(),
        "inspect".into(),
        "--envelope-id".into(),
        envelope_id.as_str().into(),
    ]
}

fn delivery_retry_arguments(
    envelope_id: &qiongli_project::DeliveryEnvelopeId,
    generation: u64,
    record_sha256: &str,
) -> Vec<OsString> {
    vec![
        "project".into(),
        "capture".into(),
        "delivery".into(),
        "retry".into(),
        "--envelope-id".into(),
        envelope_id.as_str().into(),
        "--expected-generation".into(),
        generation.to_string().into(),
        "--expected-record-sha256".into(),
        record_sha256.into(),
        "--retried-at-unix".into(),
        "1800020012".into(),
        "--cause".into(),
        "process-interrupted".into(),
    ]
}

fn assignment_arguments(
    mode: &str,
    envelope_id: &str,
    project_id: &ProjectId,
    decided_at_unix: u64,
    expected_plan_digest: Option<&str>,
) -> Vec<OsString> {
    let mut arguments = vec![
        "project".into(),
        "capture".into(),
        "assignment".into(),
        mode.into(),
        "--source-envelope-id".into(),
        envelope_id.into(),
        "--target-project-id".into(),
        project_id.as_str().into(),
        "--decision".into(),
        "assign".into(),
        "--decided-at-unix".into(),
        decided_at_unix.to_string().into(),
    ];
    if let Some(digest) = expected_plan_digest {
        arguments.extend([
            "--expected-plan-digest".into(),
            digest.into(),
            "--approve-assignment-write".into(),
        ]);
    }
    arguments
}

fn resolution_preview_arguments(
    assignment_receipt_id: &str,
    selections: &[String],
) -> Vec<OsString> {
    let mut arguments = vec![
        "project".into(),
        "capture".into(),
        "resolution".into(),
        "preview".into(),
        "--assignment-receipt-id".into(),
        assignment_receipt_id.into(),
        "--reviewed-at-unix".into(),
        "1800020040".into(),
    ];
    for selection in selections {
        arguments.extend(["--select".into(), selection.into()]);
    }
    arguments
}

fn resolution_apply_arguments(
    assignment_receipt_id: &str,
    selections: &[String],
    plan_digest: &str,
    selection_digest: &str,
) -> Vec<OsString> {
    let mut arguments = vec![
        "project".into(),
        "capture".into(),
        "resolution".into(),
        "apply".into(),
        "--assignment-receipt-id".into(),
        assignment_receipt_id.into(),
        "--reviewed-at-unix".into(),
        "1800020040".into(),
        "--resolved-at-unix".into(),
        "1800020041".into(),
    ];
    for selection in selections {
        arguments.extend(["--select".into(), selection.into()]);
    }
    arguments.extend([
        "--expected-plan-digest".into(),
        plan_digest.into(),
        "--expected-selection-digest".into(),
        selection_digest.into(),
        "--approve-academic-review".into(),
        "--approve-filesystem-write".into(),
    ]);
    arguments
}

fn resolution_selections(preview: &Value) -> Result<Vec<String>, &'static str> {
    let expected_kinds = [
        "semantic-change",
        "decision",
        "evidence",
        "contradiction",
        "next-action",
    ];
    let items = preview
        .pointer("/preview/items")
        .and_then(Value::as_array)
        .filter(|items| items.len() == expected_kinds.len())
        .ok_or("packaged-product-acceptance-resolution-invalid")?;
    items
        .iter()
        .zip(expected_kinds)
        .map(|(item, expected_kind)| {
            if item.pointer("/item/kind").and_then(Value::as_str) != Some(expected_kind) {
                return Err("packaged-product-acceptance-resolution-invalid");
            }
            let item_id = item
                .pointer("/item/itemId")
                .and_then(Value::as_str)
                .ok_or("packaged-product-acceptance-resolution-invalid")?;
            let allowed = item
                .pointer("/item/allowedDispositions")
                .and_then(Value::as_array)
                .ok_or("packaged-product-acceptance-resolution-invalid")?;
            let preferences = match expected_kind {
                "semantic-change" | "next-action" => [
                    "accept-current",
                    "accept-capture",
                    "retain-both",
                    "reject-capture",
                ],
                "decision" => [
                    "retain-both",
                    "accept-capture",
                    "accept-current",
                    "reject-capture",
                ],
                "evidence" => [
                    "accept-capture",
                    "retain-both",
                    "accept-current",
                    "reject-capture",
                ],
                "contradiction" => [
                    "reject-capture",
                    "retain-both",
                    "accept-current",
                    "accept-capture",
                ],
                _ => return Err("packaged-product-acceptance-resolution-invalid"),
            };
            let disposition = preferences
                .into_iter()
                .find(|candidate| {
                    allowed
                        .iter()
                        .any(|value| value.as_str() == Some(*candidate))
                })
                .ok_or("packaged-product-acceptance-resolution-invalid")?;
            Ok(format!("{item_id}={disposition}"))
        })
        .collect()
}

fn apply_project_lifecycle(
    canonical: &Path,
    home: &Path,
    operation: &str,
    project_id: &ProjectId,
) -> Result<(), &'static str> {
    let preview_arguments = [
        OsString::from("project"),
        OsString::from(operation),
        OsString::from("preview"),
        OsString::from("--project-id"),
        OsString::from(project_id.as_str()),
    ];
    let preview = isolated_command_args(canonical, home, &preview_arguments)?;
    let preview = parse_command_json(
        &preview,
        "packaged-product-acceptance-project-lifecycle-invalid",
    )?;
    let digest = preview
        .pointer("/preview/planDigest")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-project-lifecycle-invalid")?;
    let apply_arguments = [
        OsString::from("project"),
        OsString::from(operation),
        OsString::from("apply"),
        OsString::from("--project-id"),
        OsString::from(project_id.as_str()),
        OsString::from("--expected-plan-digest"),
        OsString::from(digest),
        OsString::from("--approve-filesystem-write"),
    ];
    let applied = isolated_command_args(canonical, home, &apply_arguments)?;
    let applied = parse_command_json(
        &applied,
        "packaged-product-acceptance-project-lifecycle-invalid",
    )?;
    let expected_command = format!("project-{operation}-apply");
    if applied.get("command").and_then(Value::as_str) != Some(&expected_command) {
        return Err("packaged-product-acceptance-project-lifecycle-invalid");
    }
    Ok(())
}

fn apply_portfolio_mutation(
    canonical: &Path,
    home: &Path,
    operation: &str,
) -> Result<Value, &'static str> {
    let preview = isolated_command_args(
        canonical,
        home,
        &[
            OsString::from("project"),
            OsString::from("portfolio"),
            OsString::from(operation),
            OsString::from("preview"),
        ],
    )?;
    let preview = parse_command_json(
        &preview,
        "packaged-product-acceptance-project-portfolio-invalid",
    )?;
    let digest = preview
        .pointer("/preview/planDigest")
        .and_then(Value::as_str)
        .ok_or("packaged-product-acceptance-project-portfolio-invalid")?;
    let applied = isolated_command_args(
        canonical,
        home,
        &[
            OsString::from("project"),
            OsString::from("portfolio"),
            OsString::from(operation),
            OsString::from("apply"),
            OsString::from("--expected-plan-digest"),
            OsString::from(digest),
            OsString::from("--approve-derived-state-write"),
        ],
    )?;
    parse_command_json(
        &applied,
        "packaged-product-acceptance-project-portfolio-invalid",
    )
}

fn verify_continuity_portfolio(portfolio: &Value) -> Result<(), &'static str> {
    let nodes = portfolio
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or("packaged-product-acceptance-project-portfolio-invalid")?;
    let edges = portfolio
        .get("edges")
        .and_then(Value::as_array)
        .ok_or("packaged-product-acceptance-project-portfolio-invalid")?;
    let shared = [
        ("paper", "doi:10.5555/qiongli-c5-shared", 2),
        ("concept", "concept:qiongli-c5-continuity", 2),
        ("method", "method:qiongli-c5-restart-protocol", 2),
    ];
    for (node_type, canonical_id, project_count) in shared {
        let observed = nodes.iter().find(|node| {
            node.get("nodeType").and_then(Value::as_str) == Some(node_type)
                && node.get("canonicalId").and_then(Value::as_str) == Some(canonical_id)
        });
        if observed
            .and_then(|node| node.get("projectIds"))
            .and_then(Value::as_array)
            .is_none_or(|project_ids| project_ids.len() != project_count)
        {
            return Err("packaged-product-acceptance-project-portfolio-invalid");
        }
    }
    let relation_count = |relation: &str| {
        edges
            .iter()
            .filter(|edge| edge.get("relation").and_then(Value::as_str) == Some(relation))
            .count()
    };
    if relation_count("shares-source") != 2
        || relation_count("shares-concept") != 2
        || relation_count("uses-method") != 2
        || edges
            .iter()
            .filter(|edge| {
                edge.get("relation").and_then(Value::as_str) == Some("forked-from")
                    && edge.get("status").and_then(Value::as_str) == Some("reviewed")
            })
            .count()
            != 1
    {
        return Err("packaged-product-acceptance-project-portfolio-invalid");
    }
    Ok(())
}

fn continuity_project_artifact_digest(
    projects: &[AcceptanceProject],
) -> Result<String, &'static str> {
    let mut identity = Vec::new();
    for project in projects {
        for relative in [
            "context/project_manifest.json",
            "graph/semantic_links.jsonl",
        ] {
            identity.extend(read_bounded(&project.root.join(relative), MAX_JSON_BYTES)?);
        }
    }
    Ok(sha256_hex(&identity))
}

struct FullMcpViews {
    library: Value,
    portfolio: Value,
}

fn run_full_project_mcp(canonical: &Path, home: &Path) -> Result<FullMcpViews, &'static str> {
    let requests = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "qiongli_project_list", "arguments": {}}
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {"name": "qiongli_project_graph_portfolio", "arguments": {}}
        }),
    ];
    let mut input = Vec::new();
    for request in requests {
        serde_json::to_writer(&mut input, &request)
            .map_err(|_| "packaged-product-acceptance-project-mcp-input-invalid")?;
        input.push(b'\n');
    }
    let mut command = Command::new(canonical);
    command
        .args(["mcp", "serve", "--profile", "full", "--transport", "stdio"])
        .env_clear()
        .env("HOME", home)
        .env("PATH", "")
        .current_dir(home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|_| "packaged-product-acceptance-project-mcp-start-failed")?;
    child
        .stdin
        .take()
        .ok_or("packaged-product-acceptance-project-mcp-start-failed")?
        .write_all(&input)
        .map_err(|_| "packaged-product-acceptance-project-mcp-write-failed")?;
    let output = child
        .wait_with_output()
        .map_err(|_| "packaged-product-acceptance-project-mcp-wait-failed")?;
    if !output.status.success()
        || !output.stderr.is_empty()
        || output.stdout.len() > MAX_COMMAND_OUTPUT_BYTES
        || home.to_str().is_some_and(|private| {
            output
                .stdout
                .windows(private.len())
                .any(|part| part == private.as_bytes())
        })
    {
        return Err("packaged-product-acceptance-project-mcp-command-failed");
    }
    let responses = output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_slice::<Value>(line)
                .map_err(|_| "packaged-product-acceptance-project-mcp-output-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let names = responses
        .get(1)
        .and_then(|response| response.pointer("/result/tools"))
        .and_then(Value::as_array)
        .ok_or("packaged-product-acceptance-project-mcp-output-invalid")?
        .iter()
        .map(|tool| tool.get("name").and_then(Value::as_str))
        .collect::<Option<Vec<_>>>()
        .ok_or("packaged-product-acceptance-project-mcp-output-invalid")?;
    let expected = LITE_PUBLIC_TOOL_NAMES
        .into_iter()
        .chain(FULL_PROJECT_PUBLIC_TOOL_NAMES)
        .chain(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES)
        .collect::<Vec<_>>();
    if names != expected {
        return Err("packaged-product-acceptance-project-mcp-tools-drift");
    }
    if responses.len() != 4
        || responses[2..]
            .iter()
            .any(|response| response.get("result").is_none() || response.get("error").is_some())
    {
        return Err("packaged-product-acceptance-project-mcp-output-invalid");
    }
    let library = responses
        .get(2)
        .and_then(|response| response.pointer("/result/structuredContent"))
        .cloned()
        .ok_or("packaged-product-acceptance-project-mcp-output-invalid")?;
    let portfolio = responses
        .get(3)
        .and_then(|response| response.pointer("/result/structuredContent"))
        .cloned()
        .ok_or("packaged-product-acceptance-project-mcp-output-invalid")?;
    Ok(FullMcpViews { library, portfolio })
}

fn parse_command_json(output: &Output, error: &'static str) -> Result<Value, &'static str> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(error);
    }
    serde_json::from_slice(&output.stdout).map_err(|_| error)
}

fn reject_project_path_output(output: &Output, project_root: &Path) -> Result<(), &'static str> {
    let private = project_root
        .to_str()
        .ok_or("packaged-product-acceptance-project-path-invalid")?;
    let bytes = output
        .stdout
        .iter()
        .chain(&output.stderr)
        .copied()
        .collect::<Vec<_>>();
    if bytes
        .windows(private.len())
        .any(|part| part == private.as_bytes())
    {
        return Err("packaged-product-acceptance-project-path-leak");
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
    let codex_legacy = home.join(".agents/plugins/qiongli/legacy-canary");
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

fn exercise_legacy_migration_lifecycle(canonical: &Path, home: &Path) -> Result<(), &'static str> {
    create_private_tree(&home.join(".codex"))?;
    create_private_tree(&home.join(".claude"))?;
    for (relative, platform) in [
        (".agents/plugins/qiongli", "codex"),
        (
            ".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli",
            "claude",
        ),
    ] {
        let plugin = home.join(relative);
        write_private_tree_file(
            &plugin.join(".qiongli-managed.json"),
            &serde_json::to_vec(&json!({
                "managed_by": "qiongli-cli",
                "plugin": "qiongli",
                "surface": "plugin",
                "platform": platform,
                "version": "1.19.0-beta.1"
            }))
            .map_err(|_| "packaged-product-acceptance-legacy-fixture-invalid")?,
        )?;
        write_private_tree_file(
            &plugin.join("skills/qiongli-workflow/fixture.txt"),
            b"recognized-qiongli-1x-plugin",
        )?;
    }
    for relative in [
        ".codex/skills/qiongli-workflow/SKILL.md",
        ".claude/skills/qiongli-workflow/SKILL.md",
    ] {
        write_private_tree_file(
            &home.join(relative),
            b"---\nname: qiongli\ndescription: \"Qiongli version: v1.19.0-beta.1\"\n---\n",
        )?;
    }
    write_private_tree_file(
        &home.join(".agents/plugins/marketplace.json"),
        &serde_json::to_vec(&json!({
            "name": "personal",
            "preserve": {"user": true},
            "plugins": [{
                "name": "qiongli",
                "source": {"source": "local", "path": "./plugins/qiongli"},
                "metadata": {"managedBy": "qiongli-cli", "surface": "plugin"}
            }]
        }))
        .map_err(|_| "packaged-product-acceptance-legacy-fixture-invalid")?,
    )?;
    write_private_tree_file(
        &home.join(".qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json"),
        &serde_json::to_vec(&json!({
            "name": "qiongli-local",
            "preserve": {"user": true},
            "plugins": [{
                "name": "qiongli",
                "version": "1.19.0-beta.1",
                "source": "./plugins/qiongli"
            }]
        }))
        .map_err(|_| "packaged-product-acceptance-legacy-fixture-invalid")?,
    )?;
    write_private_tree_file(
        &home.join(".codex/config.toml"),
        concat!(
            "model = \"host-owned\"\n\n",
            "# BEGIN QIONGLI MANAGED MCP\n",
            "[mcp_servers.qiongli]\n",
            "command = \"qiongli\"\n",
            "args = [\"mcp\", \"serve\", \"--transport\", \"stdio\"]\n",
            "# END QIONGLI MANAGED MCP\n"
        )
        .as_bytes(),
    )?;
    write_private_tree_file(
        &home.join(".claude.json"),
        &serde_json::to_vec(&json!({
            "theme": "dark",
            "mcpServers": {
                "qiongli": {
                    "command": "qiongli",
                    "args": ["mcp", "serve", "--transport", "stdio"],
                    "type": "stdio"
                }
            }
        }))
        .map_err(|_| "packaged-product-acceptance-legacy-fixture-invalid")?,
    )?;
    write_private_tree_file(
        &home.join(".config/qiongli/providers.json"),
        br#"{
  "version": 1,
  "providers": {
    "crossref": {"email": "migration-fixture@example.org"},
    "arxiv": {"enabled": false}
  }
}"#,
    )?;
    progress("legacy-fixture");

    let inspect = isolated_command(canonical, home, ["migrate-1x", "inspect"])?;
    let inspect = parse_command_json(
        &inspect,
        "packaged-product-acceptance-legacy-inspect-invalid",
    )?;
    if inspect["command"] != "inspect"
        || inspect["inventory"]["detected_item_count"] != 9
        || inspect["inventory"]["eligible_item_count"] != 9
        || inspect["inventory"]["review_item_count"] != 0
    {
        return Err("packaged-product-acceptance-legacy-inspect-invalid");
    }
    progress("legacy-inspect");

    let preview = isolated_command(canonical, home, ["migrate-1x", "preview"])?;
    let preview = parse_command_json(
        &preview,
        "packaged-product-acceptance-legacy-preview-invalid",
    )?;
    let migration_id = preview["plan"]["plan_id"]
        .as_str()
        .ok_or("packaged-product-acceptance-legacy-preview-invalid")?;
    let plan_sha256 = preview["plan"]["plan_sha256"]
        .as_str()
        .filter(|value| valid_lower_hex(value, 64))
        .ok_or("packaged-product-acceptance-legacy-preview-invalid")?;
    progress("legacy-preview");
    let apply = [
        OsString::from("migrate-1x"),
        OsString::from("apply"),
        OsString::from("--migration-id"),
        OsString::from(migration_id),
        OsString::from("--expected-plan-digest"),
        OsString::from(plan_sha256),
        OsString::from("--approve-filesystem-write"),
        OsString::from("--approve-client-config-change"),
    ];
    let apply = isolated_command_args(canonical, home, &apply)?;
    let apply = parse_command_json(&apply, "packaged-product-acceptance-legacy-apply-invalid")?;
    if apply["state"] != "awaiting-client-activation" {
        return Err("packaged-product-acceptance-legacy-apply-invalid");
    }
    progress("legacy-apply");
    let migrated_settings: Value = serde_json::from_slice(
        &fs::read(home.join(".config/qiongli/v2/settings.json"))
            .map_err(|_| "packaged-product-acceptance-legacy-provider-migration-invalid")?,
    )
    .map_err(|_| "packaged-product-acceptance-legacy-provider-migration-invalid")?;
    if migrated_settings["providers"]["crossref"]["enabled"] != true
        || migrated_settings["providers"]["crossref"]["email"] != "migration-fixture@example.org"
        || migrated_settings["providers"]["arxiv"]["enabled"] != false
    {
        return Err("packaged-product-acceptance-legacy-provider-migration-invalid");
    }

    let confirm = [
        OsString::from("migrate-1x"),
        OsString::from("continue"),
        OsString::from("--migration-id"),
        OsString::from(migration_id),
        OsString::from("--confirm-host-activation"),
    ];
    let confirm = isolated_command_args(canonical, home, &confirm)?;
    let confirm = parse_command_json(
        &confirm,
        "packaged-product-acceptance-legacy-confirm-invalid",
    )?;
    if confirm["state"] != "cleanup-ready" {
        return Err("packaged-product-acceptance-legacy-confirm-invalid");
    }
    progress("legacy-activation");

    let cleanup = [
        OsString::from("migrate-1x"),
        OsString::from("continue"),
        OsString::from("--migration-id"),
        OsString::from(migration_id),
        OsString::from("--approve-cleanup"),
    ];
    let cleanup = isolated_command_args(canonical, home, &cleanup)?;
    let cleanup = parse_command_json(
        &cleanup,
        "packaged-product-acceptance-legacy-cleanup-invalid",
    )?;
    if cleanup["state"] != "complete" {
        return Err("packaged-product-acceptance-legacy-cleanup-invalid");
    }
    progress("legacy-cleanup");

    let inspect = isolated_command(canonical, home, ["migrate-1x", "inspect"])?;
    let inspect = parse_command_json(
        &inspect,
        "packaged-product-acceptance-legacy-cleanup-invalid",
    )?;
    if inspect["inventory"]["detected_item_count"] != 0
        || inspect["inventory"]["eligible_item_count"] != 0
        || inspect["inventory"]["review_item_count"] != 0
    {
        return Err("packaged-product-acceptance-legacy-cleanup-invalid");
    }
    progress("legacy-cleanup-inspect");
    let finalize = [
        OsString::from("migrate-1x"),
        OsString::from("continue"),
        OsString::from("--migration-id"),
        OsString::from(migration_id),
        OsString::from("--finalize"),
    ];
    let finalize = isolated_command_args(canonical, home, &finalize)?;
    let finalize = parse_command_json(
        &finalize,
        "packaged-product-acceptance-legacy-finalize-invalid",
    )?;
    if finalize["state"] != "complete"
        || home
            .join(format!(
                ".qiongli/v2/migrations/1x-to-2x/{migration_id}/cleanup-journal.json"
            ))
            .exists()
    {
        return Err("packaged-product-acceptance-legacy-finalize-invalid");
    }
    progress("legacy-finalize");
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
        if env::var_os("QIONGLI_ACCEPTANCE_DIAGNOSTICS").is_some() {
            eprintln!("acceptance diagnostic: {error}; status={}", output.status);
            eprintln!(
                "acceptance diagnostic stdout:\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
            eprintln!(
                "acceptance diagnostic stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
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
struct AcceptanceReceiptV2<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    product_source_commit: &'a str,
    canonical_sha256: &'a str,
    product_control_sha256: String,
    signed_archive_sha256: String,
    continuity: ContinuityEvidenceV1,
    checks: AcceptanceChecksV1,
}

#[derive(Serialize)]
struct ContinuityEvidenceV1 {
    project_count: u64,
    shared_source_identity_count: u64,
    shared_concept_identity_count: u64,
    shared_method_identity_count: u64,
    reviewed_lineage_count: u64,
    delivery_record_count: u64,
    retry_count: u64,
    acknowledgement_replay_count: u64,
    duplicate_suppression_count: u64,
    assignment_count: u64,
    resolution_count: u64,
    resolution_item_count: u64,
    archive_count: u64,
    restore_count: u64,
    derived_deletion_count: u64,
    full_rebuild_count: u64,
    matched_query_project_count: u64,
    matched_query_lineage_count: u64,
    timeline_event_count: u64,
    app_cli_library_parity: bool,
    full_mcp_library_portfolio_parity: bool,
    canonical_project_artifacts_unchanged_by_derived_rebuild: bool,
    path_redacted: bool,
}

#[derive(Serialize)]
struct AcceptanceChecksV1 {
    embedded_authority: bool,
    canonical_signature_preserved: bool,
    product_control_verified: bool,
    inventory_discovered: bool,
    skills_materialize_verify_refresh: bool,
    lite_mcp_self_test: bool,
    project_three_project_restart: bool,
    project_app_cli_library_full_mcp_parity: bool,
    continuity_delivery_restart_replay: bool,
    continuity_assignment_resolution: bool,
    continuity_archive_restore_rebuild: bool,
    continuity_catalog_query_timeline: bool,
    continuity_path_redacted: bool,
    provider_keychain_save_replace_restart_remove: bool,
    codex_install_verify_remove: bool,
    claude_install_verify_remove: bool,
    registration_repair: bool,
    packaged_restart_verification: bool,
    legacy_migration_fixture_isolated: bool,
    empty_path_startup: bool,
}
