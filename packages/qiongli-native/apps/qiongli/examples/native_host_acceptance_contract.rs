use std::env;
use std::fs;
use std::path::Path;

use qiongli_execution::{HostAcceptanceFixtureV1, HostAcceptanceReceiptV1};
use serde::Serialize;

const MAX_FIXTURE_BYTES: u64 = 64 * 1024;
const MAX_RECEIPT_BYTES: u64 = 256 * 1024;

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
        _ => Err("host-acceptance-usage-invalid"),
    }
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
