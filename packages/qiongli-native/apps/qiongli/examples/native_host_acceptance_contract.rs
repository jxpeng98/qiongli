use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::Path;

use qiongli_execution::{
    FULL_MCP_HOST_PROTOCOL_VERSION, HOST_ACCEPTANCE_RECORD_TYPE, HOST_ACCEPTANCE_SCHEMA_VERSION,
    HostAcceptanceCheckpointTransitionV1, HostAcceptanceFixtureV1, HostAcceptanceProfileScopeV1,
    HostAcceptanceReceiptV1, HostAcceptanceStatusV1, HostAcceptanceVerdictV1, HostFamilyV1,
    HostReviewResultV1, ToolId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

const MAX_FIXTURE_BYTES: u64 = 64 * 1024;
const MAX_RECEIPT_BYTES: u64 = 256 * 1024;
const MAX_PACKAGE_JSON_BYTES: u64 = 2 * 1024 * 1024;
const MAX_BINARY_BYTES: u64 = 512 * 1024 * 1024;
const MAX_EVIDENCE_RESULT_DIGESTS: usize = 32;

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
    host_profile_scope: HostAcceptanceProfileScopeV1,
    host_version: &'a str,
    plugin_sha256: &'a str,
    product_bound: bool,
    prepared_fixture_bound: bool,
    plugin_registration_bound: bool,
    isolated_installation_bound: bool,
    system_registration_bound: bool,
    path_redacted: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HostAcceptanceObservationV1 {
    schema_version: u32,
    record_type: String,
    host_family: HostFamilyV1,
    host_profile_scope: HostAcceptanceProfileScopeV1,
    host_version: String,
    observed_tool_ids: Vec<ToolId>,
    evidence_result_sha256s: Vec<String>,
    accepted_candidate_sha256: String,
    review_result: HostReviewResultV1,
    checkpoint_transitions: Vec<HostAcceptanceCheckpointTransitionV1>,
    verdict: HostAcceptanceVerdictV1,
}

struct PackagedHostBindingV1 {
    fixture_sha256: String,
    product_version: String,
    product_source_commit: String,
    binary_sha256: String,
    plugin_version: String,
    plugin_sha256: String,
}

#[derive(Serialize)]
struct ReceiptComposed<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    fixture_id: &'a str,
    fixture_sha256: &'a str,
    receipt_sha256: String,
    host_family: HostFamilyV1,
    host_profile_scope: HostAcceptanceProfileScopeV1,
    host_version: &'a str,
    observed_tool_count: usize,
    checkpoint_transition_count: usize,
    rejection_observation_count: u64,
    product_bound: bool,
    plugin_registration_bound: bool,
    isolated_installation_bound: bool,
    system_registration_bound: bool,
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
        [
            command,
            fixture_path,
            acceptance_root,
            receipt_path,
            system_registration_path,
        ] if command == "packaged-receipt" => validate_packaged_receipt(
            Path::new(fixture_path),
            Path::new(acceptance_root),
            Path::new(receipt_path),
            Path::new(system_registration_path),
        ),
        [
            command,
            fixture_path,
            acceptance_root,
            observation_path,
            system_registration_path,
        ] if command == "compose-packaged-receipt" => compose_packaged_receipt(
            Path::new(fixture_path),
            Path::new(acceptance_root),
            Path::new(observation_path),
            Path::new(system_registration_path),
        ),
        _ => Err("host-acceptance-usage-invalid"),
    }
}

fn validate_packaged_receipt(
    fixture_path: &Path,
    acceptance_root: &Path,
    receipt_path: &Path,
    system_registration_path: &Path,
) -> Result<(), &'static str> {
    let fixture = read_fixture(fixture_path)?;
    let receipt = read_receipt(receipt_path)?;
    receipt
        .validate_against(&fixture)
        .map_err(|_| "host-acceptance-receipt-fixture-mismatch")?;
    let binding = read_packaged_binding(&fixture, acceptance_root, receipt.host_family)?;
    let host_profile_scope = receipt
        .host_profile_scope
        .ok_or("host-acceptance-system-profile-required")?;
    if host_profile_scope != HostAcceptanceProfileScopeV1::SystemExisting {
        return Err("host-acceptance-system-profile-required");
    }
    validate_system_registration(
        system_registration_path,
        acceptance_root,
        receipt.host_family,
        &binding,
    )?;
    if receipt.fixture_sha256 != binding.fixture_sha256
        || receipt.binary_sha256 != binding.binary_sha256
        || receipt.product_source_commit != binding.product_source_commit
        || receipt.product_version != binding.product_version
        || receipt.adapter_version != binding.plugin_version
        || receipt.plugin_sha256 != binding.plugin_sha256
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
        host_profile_scope,
        host_version: &receipt.host_version,
        plugin_sha256: &receipt.plugin_sha256,
        product_bound: true,
        prepared_fixture_bound: true,
        plugin_registration_bound: true,
        isolated_installation_bound: true,
        system_registration_bound: true,
        path_redacted: true,
    };
    print_canonical(&output)
}

fn compose_packaged_receipt(
    fixture_path: &Path,
    acceptance_root: &Path,
    observation_path: &Path,
    system_registration_path: &Path,
) -> Result<(), &'static str> {
    let fixture = read_fixture(fixture_path)?;
    let observation_bytes = read_bounded(observation_path, MAX_RECEIPT_BYTES)?;
    let observation_input = trim_terminal_newline(&observation_bytes);
    let observation = serde_json::from_slice::<HostAcceptanceObservationV1>(observation_input)
        .map_err(|_| "host-acceptance-observation-invalid")?;
    if serde_json_canonicalizer::to_vec(&observation)
        .map_err(|_| "host-acceptance-observation-invalid")?
        != observation_input
        || observation.schema_version != 1
        || observation.record_type != "qiongli-host-acceptance-observation"
    {
        return Err("host-acceptance-observation-invalid");
    }
    if observation.evidence_result_sha256s.len()
        < usize::from(fixture.candidate_contract.minimum_evidence_audit_count)
        || observation.evidence_result_sha256s.len() > MAX_EVIDENCE_RESULT_DIGESTS
        || !strictly_sorted(&observation.evidence_result_sha256s)
        || observation
            .evidence_result_sha256s
            .iter()
            .any(|digest| !valid_sha256(digest))
    {
        return Err("host-acceptance-observation-invalid");
    }
    let evidence_audit_count = u16::try_from(observation.evidence_result_sha256s.len())
        .map_err(|_| "host-acceptance-observation-invalid")?;
    let evidence_audit_sha256 = sha256(
        &serde_json_canonicalizer::to_vec(&observation.evidence_result_sha256s)
            .map_err(|_| "host-acceptance-observation-invalid")?,
    );
    let known_fact_count =
        u16::try_from(fixture.facts.len()).map_err(|_| "host-acceptance-fixture-invalid")?;
    let known_fact_set_sha256 = fixture
        .fact_set_digest()
        .map_err(|_| "host-acceptance-fixture-invalid")?;
    let binding = read_packaged_binding(&fixture, acceptance_root, observation.host_family)?;
    if observation.host_profile_scope != HostAcceptanceProfileScopeV1::SystemExisting {
        return Err("host-acceptance-system-profile-required");
    }
    validate_system_registration(
        system_registration_path,
        acceptance_root,
        observation.host_family,
        &binding,
    )?;
    let receipt = HostAcceptanceReceiptV1 {
        schema_version: HOST_ACCEPTANCE_SCHEMA_VERSION,
        record_type: HOST_ACCEPTANCE_RECORD_TYPE.to_owned(),
        status: HostAcceptanceStatusV1::Accepted,
        publication_allowed: false,
        fixture_id: fixture.fixture_id.clone(),
        fixture_sha256: binding.fixture_sha256,
        product_version: binding.product_version,
        product_source_commit: binding.product_source_commit,
        binary_sha256: binding.binary_sha256,
        host_family: observation.host_family,
        host_version: observation.host_version,
        adapter_version: binding.plugin_version,
        plugin_sha256: binding.plugin_sha256,
        host_profile_scope: Some(observation.host_profile_scope),
        full_mcp_protocol: FULL_MCP_HOST_PROTOCOL_VERSION.to_owned(),
        observed_tool_ids: observation.observed_tool_ids,
        evidence_audit_count,
        evidence_audit_sha256,
        known_fact_count,
        known_fact_set_sha256,
        accepted_candidate_sha256: observation.accepted_candidate_sha256,
        review_result: observation.review_result,
        checkpoint_transitions: observation.checkpoint_transitions,
        verdict: observation.verdict,
    };
    receipt
        .validate_against(&fixture)
        .map_err(|_| "host-acceptance-observation-fixture-mismatch")?;
    let receipt_bytes = receipt
        .to_canonical_json()
        .map_err(|_| "host-acceptance-receipt-invalid")?;
    let receipt_name = match receipt.host_family {
        HostFamilyV1::Codex => "qiongli-c5-codex-host-acceptance.receipt.json",
        HostFamilyV1::ClaudeCode => "qiongli-c5-claude-code-host-acceptance.receipt.json",
        HostFamilyV1::ClaudeDesktop | HostFamilyV1::OtherLocal => {
            return Err("host-acceptance-package-host-unsupported");
        }
    };
    let receipt_path = acceptance_root.join(receipt_name);
    if receipt_path.exists() {
        let existing = read_receipt(&receipt_path)?;
        if existing != receipt {
            return Err("host-acceptance-receipt-existing-drift");
        }
    } else {
        write_private_new(&receipt_path, &receipt_bytes)?;
    }
    let rejection_observation_count =
        u64::from(receipt.verdict.stale_project_revision_rejection_count)
            + u64::from(receipt.verdict.checkpoint_digest_rejection_count)
            + u64::from(receipt.verdict.undeclared_evidence_rejection_count)
            + u64::from(receipt.verdict.unknown_field_rejection_count);
    let output = ReceiptComposed {
        schema_version: 1,
        record_type: "qiongli-packaged-host-acceptance-composition",
        status: "receipt-composed-package-bound",
        publication_allowed: false,
        fixture_id: &receipt.fixture_id,
        fixture_sha256: &receipt.fixture_sha256,
        receipt_sha256: receipt
            .digest()
            .map_err(|_| "host-acceptance-receipt-invalid")?,
        host_family: receipt.host_family,
        host_profile_scope: observation.host_profile_scope,
        host_version: &receipt.host_version,
        observed_tool_count: receipt.observed_tool_ids.len(),
        checkpoint_transition_count: receipt.checkpoint_transitions.len(),
        rejection_observation_count,
        product_bound: true,
        plugin_registration_bound: true,
        isolated_installation_bound: true,
        system_registration_bound: true,
        path_redacted: true,
    };
    print_canonical(&output)
}

fn read_packaged_binding(
    fixture: &HostAcceptanceFixtureV1,
    acceptance_root: &Path,
    host_family: HostFamilyV1,
) -> Result<PackagedHostBindingV1, &'static str> {
    let metadata = fs::symlink_metadata(acceptance_root)
        .map_err(|_| "host-acceptance-package-root-invalid")?;
    if !acceptance_root.is_absolute()
        || metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || fs::canonicalize(acceptance_root).ok().as_deref() != Some(acceptance_root)
    {
        return Err("host-acceptance-package-root-invalid");
    }
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
    let registration_path = match host_family {
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
    let plugin_sha256 = required_string(&registration, "/active/source_content_root_sha256")?;
    let plugin_version = required_string(&registration, "/active/artifact/version")?;
    let product_source_commit = required_string(&product_receipt, "/product_source_commit")?;
    let manifest_source_commit = required_string(&manifest, "/product_source_commit")?;
    let product_version = required_string(&manifest, "/application/product_version")?;

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
        || product_source_commit != manifest_source_commit
        || plugin_version != product_version
    {
        return Err("host-acceptance-receipt-package-mismatch");
    }
    Ok(PackagedHostBindingV1 {
        fixture_sha256,
        product_version: product_version.to_owned(),
        product_source_commit: product_source_commit.to_owned(),
        binary_sha256,
        plugin_version: plugin_version.to_owned(),
        plugin_sha256: plugin_sha256.to_owned(),
    })
}

fn validate_system_registration(
    registration_path: &Path,
    acceptance_root: &Path,
    host_family: HostFamilyV1,
    binding: &PackagedHostBindingV1,
) -> Result<(), &'static str> {
    let (expected_suffix, expected_file_name, expected_target_family) = match host_family {
        HostFamilyV1::Codex => (
            Path::new(".qiongli/plugins/codex/.qiongli-next-codex-registration.json"),
            OsStr::new(".qiongli-next-codex-registration.json"),
            "codex-local",
        ),
        HostFamilyV1::ClaudeCode => (
            Path::new(
                ".qiongli/v2/integrations/claude-code/.qiongli-next-claude-registration.json",
            ),
            OsStr::new(".qiongli-next-claude-registration.json"),
            "claude-code-local",
        ),
        HostFamilyV1::ClaudeDesktop | HostFamilyV1::OtherLocal => {
            return Err("host-acceptance-package-host-unsupported");
        }
    };
    let metadata = fs::symlink_metadata(registration_path)
        .map_err(|_| "host-acceptance-system-registration-unavailable")?;
    if !registration_path.is_absolute()
        || metadata.file_type().is_symlink()
        || !metadata.is_file()
        || registration_path.file_name() != Some(expected_file_name)
        || !registration_path.ends_with(expected_suffix)
        || registration_path.starts_with(acceptance_root)
        || fs::canonicalize(registration_path).ok().as_deref() != Some(registration_path)
    {
        return Err("host-acceptance-system-registration-invalid");
    }
    let registration =
        read_canonical_value(&read_bounded(registration_path, MAX_PACKAGE_JSON_BYTES)?)?;
    let active_install_id = required_string(&registration, "/active/install_id")?;
    if registration["schema_version"] != 1
        || registration["active"]["schema_version"] != 1
        || registration["install_id"].as_str() != Some(active_install_id)
        || registration["active"]["artifact"]["product"] != "qiongli"
        || registration["active"]["target"]["family"] != expected_target_family
        || registration["active"]["artifact"]["version"].as_str()
            != Some(binding.plugin_version.as_str())
        || registration["active"]["source_content_root_sha256"].as_str()
            != Some(binding.plugin_sha256.as_str())
    {
        return Err("host-acceptance-system-registration-mismatch");
    }
    Ok(())
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

#[cfg(unix)]
fn write_private_new(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "host-acceptance-output-unavailable")?;
    output
        .write_all(bytes)
        .and_then(|()| output.sync_all())
        .map_err(|_| "host-acceptance-output-unavailable")
}

#[cfg(not(unix))]
fn write_private_new(_path: &Path, _bytes: &[u8]) -> Result<(), &'static str> {
    Err("host-acceptance-output-unavailable")
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
