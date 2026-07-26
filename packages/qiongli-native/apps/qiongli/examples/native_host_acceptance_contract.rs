use std::env;
use std::fs;
use std::path::Path;

use qiongli_execution::{HostAcceptanceFixtureV1, HostAcceptanceReceiptV1, HostFamilyV1};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest as _, Sha256};

const MAX_FIXTURE_BYTES: u64 = 64 * 1024;
const MAX_RECEIPT_BYTES: u64 = 256 * 1024;
const MAX_PACKAGE_JSON_BYTES: u64 = 2 * 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Serialize)]
struct FixtureReady<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    fixture_id: &'a str,
    fixture_sha256: String,
    expected_project_revision: u64,
    fact_count: usize,
    required_tool_count: usize,
    required_transition_count: usize,
    manual_host_session_required: bool,
}

#[derive(Serialize)]
struct ReceiptValidated<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    fixture_id: &'a str,
    fixture_sha256: &'a str,
    receipt_sha256: String,
    host_family: qiongli_execution::HostFamilyV1,
    host_version: &'a str,
    adapter_version: &'a str,
    observed_tool_count: usize,
    checkpoint_transition_count: usize,
    direct_model_request_count: u32,
    qiongli_model_cli_child_count: u32,
}

#[derive(Serialize)]
struct PackagedReceiptValidated<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    fixture_id: &'a str,
    fixture_sha256: &'a str,
    receipt_sha256: String,
    product_source_commit: &'a str,
    binary_sha256: &'a str,
    host_family: HostFamilyV1,
    host_version: &'a str,
    plugin_sha256: &'a str,
    product_bound: bool,
    prepared_fixture_bound: bool,
    plugin_registration_bound: bool,
    path_redacted: bool,
}

fn main() {
    if let Err(reason_code) = run() {
        eprintln!("{reason_code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, fixture_path] if command == "fixture" => {
            let fixture = read_fixture(Path::new(fixture_path))?;
            let output = FixtureReady {
                schema_version: 1,
                record_type: "qiongli-alpha2-host-acceptance-preflight",
                status: "fixture-ready-manual-host-required",
                publication_allowed: false,
                fixture_id: &fixture.fixture_id,
                fixture_sha256: fixture
                    .digest()
                    .map_err(|_| "host-acceptance-fixture-invalid")?,
                expected_project_revision: fixture.expected_project_revision,
                fact_count: fixture.facts.len(),
                required_tool_count: fixture.required_tool_ids.len(),
                required_transition_count: fixture.required_transitions.len(),
                manual_host_session_required: true,
            };
            print_canonical(&output)
        }
        [command, fixture_path, receipt_path] if command == "receipt" => {
            let fixture = read_fixture(Path::new(fixture_path))?;
            let receipt = read_receipt(Path::new(receipt_path))?;
            receipt
                .validate_against(&fixture)
                .map_err(|_| "host-acceptance-receipt-fixture-mismatch")?;
            let output = ReceiptValidated {
                schema_version: 1,
                record_type: "qiongli-alpha2-host-acceptance-validation",
                status: "receipt-valid",
                publication_allowed: false,
                fixture_id: &receipt.fixture_id,
                fixture_sha256: &receipt.fixture_sha256,
                receipt_sha256: receipt
                    .digest()
                    .map_err(|_| "host-acceptance-receipt-invalid")?,
                host_family: receipt.host_family,
                host_version: &receipt.host_version,
                adapter_version: &receipt.adapter_version,
                observed_tool_count: receipt.observed_tool_ids.len(),
                checkpoint_transition_count: receipt.checkpoint_transitions.len(),
                direct_model_request_count: receipt.verdict.direct_model_request_count,
                qiongli_model_cli_child_count: receipt.verdict.qiongli_model_cli_child_count,
            };
            print_canonical(&output)
        }
        [command, fixture_path, acceptance_root, receipt_path] if command == "packaged-receipt" => {
            validate_packaged_receipt(
                Path::new(fixture_path),
                Path::new(acceptance_root),
                Path::new(receipt_path),
            )
        }
        _ => Err("host-acceptance-usage-invalid"),
    }
}

fn validate_packaged_receipt(
    fixture_path: &Path,
    acceptance_root: &Path,
    receipt_path: &Path,
) -> Result<(), &'static str> {
    let metadata = fs::symlink_metadata(acceptance_root)
        .map_err(|_| "host-acceptance-package-root-invalid")?;
    if !acceptance_root.is_absolute()
        || metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || fs::canonicalize(acceptance_root).ok().as_deref() != Some(acceptance_root)
    {
        return Err("host-acceptance-package-root-invalid");
    }
    let fixture = read_fixture(fixture_path)?;
    let receipt = read_receipt(receipt_path)?;
    receipt
        .validate_against(&fixture)
        .map_err(|_| "host-acceptance-receipt-fixture-mismatch")?;

    let product_receipt_bytes = read_bounded(
        &acceptance_root.join("qiongli-packaged-product-acceptance.receipt.json"),
        MAX_PACKAGE_JSON_BYTES,
    )?;
    let product_receipt = read_canonical_value(&product_receipt_bytes)?;
    let preparation_receipt = read_canonical_value(&read_bounded(
        &acceptance_root.join("qiongli-packaged-host-fixture.receipt.json"),
        MAX_PACKAGE_JSON_BYTES,
    )?)?;
    let manifest = read_canonical_value(&read_bounded(
        &acceptance_root
            .join("extracted/Qiongli.app/Contents/Resources/.qiongli-desktop-package.json"),
        MAX_PACKAGE_JSON_BYTES,
    )?)?;
    let canonical_bytes = read_bounded(
        &acceptance_root.join("extracted/Qiongli.app/Contents/MacOS/qiongli-cli"),
        MAX_BINARY_BYTES,
    )?;
    let binary_sha256 = sha256(&canonical_bytes);
    let product_acceptance_receipt_sha256 = sha256(&product_receipt_bytes);
    let fixture_sha256 = fixture
        .digest()
        .map_err(|_| "host-acceptance-fixture-invalid")?;
    let registration_path = match receipt.host_family {
        HostFamilyV1::Codex => {
            "manual-home/.qiongli/plugins/codex/.qiongli-next-codex-registration.json"
        }
        HostFamilyV1::ClaudeCode => {
            "manual-home/.qiongli/v2/integrations/claude-code/.qiongli-next-claude-registration.json"
        }
        HostFamilyV1::ClaudeDesktop | HostFamilyV1::OtherLocal => {
            return Err("host-acceptance-package-host-unsupported");
        }
    };
    let registration = read_canonical_value(&read_bounded(
        &acceptance_root.join(registration_path),
        MAX_PACKAGE_JSON_BYTES,
    )?)?;
    let registered_plugin_sha256 =
        required_string(&registration, "/active/source_content_root_sha256")?;
    let registered_plugin_version = required_string(&registration, "/active/artifact/version")?;

    if product_receipt["schema_version"] != 2
        || product_receipt["record_type"] != "qiongli-packaged-product-acceptance"
        || product_receipt["status"] != "accepted-ad-hoc-nonpublishing"
        || product_receipt["publication_allowed"] != false
        || preparation_receipt["schema_version"] != 1
        || preparation_receipt["record_type"] != "qiongli-packaged-host-fixture-preparation"
        || preparation_receipt["status"] != "prepared-manual-host-required"
        || preparation_receipt["publication_allowed"] != false
        || preparation_receipt["manual_host_session_required"] != true
        || preparation_receipt["path_redacted"] != true
        || preparation_receipt["fixture_id"].as_str() != Some(fixture.fixture_id.as_str())
        || preparation_receipt["fixture_sha256"].as_str() != Some(fixture_sha256.as_str())
        || preparation_receipt["host_project_revision"].as_u64()
            != Some(fixture.expected_project_revision)
        || preparation_receipt["product_acceptance_receipt_sha256"].as_str()
            != Some(product_acceptance_receipt_sha256.as_str())
        || preparation_receipt["canonical_sha256"].as_str() != Some(binary_sha256.as_str())
        || product_receipt["canonical_sha256"].as_str() != Some(binary_sha256.as_str())
        || manifest["canonical_binary_sha256"].as_str() != Some(binary_sha256.as_str())
        || receipt.binary_sha256 != binary_sha256
        || receipt.product_source_commit
            != required_string(&product_receipt, "/product_source_commit")?
        || receipt.product_source_commit != required_string(&manifest, "/product_source_commit")?
        || receipt.product_version != required_string(&manifest, "/application/product_version")?
        || receipt.adapter_version != registered_plugin_version
        || receipt.plugin_sha256 != registered_plugin_sha256
    {
        return Err("host-acceptance-receipt-package-mismatch");
    }

    let output = PackagedReceiptValidated {
        schema_version: 1,
        record_type: "qiongli-packaged-host-acceptance-validation",
        status: "receipt-valid-package-bound",
        publication_allowed: false,
        fixture_id: &receipt.fixture_id,
        fixture_sha256: &receipt.fixture_sha256,
        receipt_sha256: receipt
            .digest()
            .map_err(|_| "host-acceptance-receipt-invalid")?,
        product_source_commit: &receipt.product_source_commit,
        binary_sha256: &receipt.binary_sha256,
        host_family: receipt.host_family,
        host_version: &receipt.host_version,
        plugin_sha256: &receipt.plugin_sha256,
        product_bound: true,
        prepared_fixture_bound: true,
        plugin_registration_bound: true,
        path_redacted: true,
    };
    print_canonical(&output)
}

fn read_fixture(path: &Path) -> Result<HostAcceptanceFixtureV1, &'static str> {
    let bytes = read_bounded(path, MAX_FIXTURE_BYTES)?;
    HostAcceptanceFixtureV1::from_canonical_json(trim_terminal_newline(&bytes))
        .map_err(|_| "host-acceptance-fixture-invalid")
}

fn read_receipt(path: &Path) -> Result<HostAcceptanceReceiptV1, &'static str> {
    let bytes = read_bounded(path, MAX_RECEIPT_BYTES)?;
    HostAcceptanceReceiptV1::from_canonical_json(trim_terminal_newline(&bytes))
        .map_err(|_| "host-acceptance-receipt-invalid")
}

fn read_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "host-acceptance-input-unavailable")?;
    if !metadata.file_type().is_file() || metadata.len() > maximum_bytes {
        return Err("host-acceptance-input-invalid");
    }
    fs::read(path).map_err(|_| "host-acceptance-input-unavailable")
}

fn read_canonical_value(bytes: &[u8]) -> Result<Value, &'static str> {
    let value = serde_json::from_slice::<Value>(bytes)
        .map_err(|_| "host-acceptance-package-json-invalid")?;
    if serde_json_canonicalizer::to_vec(&value)
        .map_err(|_| "host-acceptance-package-json-invalid")?
        != bytes
    {
        return Err("host-acceptance-package-json-noncanonical");
    }
    Ok(value)
}

fn required_string<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, &'static str> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or("host-acceptance-package-json-invalid")
}

fn trim_terminal_newline(bytes: &[u8]) -> &[u8] {
    bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
        .unwrap_or(bytes)
}

fn print_canonical(value: &impl Serialize) -> Result<(), &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| "host-acceptance-output-invalid")?;
    let rendered = String::from_utf8(bytes).map_err(|_| "host-acceptance-output-invalid")?;
    println!("{rendered}");
    Ok(())
}

fn sha256(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}
