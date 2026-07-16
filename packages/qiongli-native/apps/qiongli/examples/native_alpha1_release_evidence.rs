use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};

use qiongli_platform::{
    ClientActivationTarget, NativeReleaseAuthority, NativeReleaseCandidateVerificationContext,
    NativeUpdateError, NativeUpdateStream, NativeUpdateVerificationContext,
    SignedNativeReleaseCandidateV1, SignedNativeUpdateManifestV1,
    approve_native_portable_archive_target, native_portable_archive_file_name,
    native_release_candidate_file_name, native_release_notes_file_name,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

const VERSION: &str = "2.0.0-alpha.1";
const TARGET: &str = "macos-aarch64";
const CHECKSUMS_FILE: &str = "qiongli-2.0.0-alpha.1-macos-aarch64.SHA256SUMS";
const SBOM_FILE: &str = "qiongli-2.0.0-alpha.1-macos-aarch64.cdx.json";
const PROVENANCE_FILE: &str = "qiongli-2.0.0-alpha.1-macos-aarch64.provenance.json";
const SUPPLY_CHAIN_RECEIPT_FILE: &str = "qiongli-alpha1-supply-chain.receipt.json";
const PUBLICATION_LEDGER_FILE: &str = "qiongli-alpha1-publication-ledger.json";
const PUBLICATION_LEDGER_SHA256_FILE: &str = "qiongli-alpha1-publication-ledger.sha256";
const UNSIGNED_ARCHIVE_FILE: &str = "qiongli-desktop-2.0.0-alpha.1-macos-aarch64.app.zip";
const SIGNED_ARCHIVE_FILE: &str =
    "qiongli-desktop-2.0.0-alpha.1-macos-aarch64.signed-notarized.app.zip";
const DESKTOP_MANIFEST_FILE: &str = "qiongli-desktop-package.manifest.json";
const DESKTOP_RECEIPT_FILE: &str = "qiongli-desktop-package.receipt.json";
const SIGNING_RECEIPT_FILE: &str =
    "qiongli-desktop-2.0.0-alpha.1-macos-aarch64.signing.receipt.json";
const SIGNING_BOUNDARY_RECEIPT_FILE: &str = "qiongli-macos-alpha1-signing.receipt.json";
const UNSIGNED_ACCEPTANCE_RECEIPT_FILE: &str =
    "qiongli-macos-alpha1-unsigned-acceptance.receipt.json";
const UPDATE_METADATA_FILE: &str = "macos-aarch64.json";
const UPDATE_RECEIPT_FILE: &str = "qiongli-alpha1-update-metadata.receipt.json";
const AUTHORITY_FILE: &str = "qiongli-native-release-authority.json";
const PORTABLE_ARCHIVE_FILE: &str =
    "qiongli-2.0.0-alpha.1-alpha-lite-macos-aarch64-portable-archive.zip";
const CANDIDATE_FILE: &str =
    "qiongli-2.0.0-alpha.1-alpha-lite-macos-aarch64-portable-archive.candidate.json";
const RELEASE_NOTES_FILE: &str =
    "qiongli-2.0.0-alpha.1-alpha-lite-macos-aarch64-portable-archive.release-notes.md";
const GITHUB_RUN_PREFIX: &str = "https://github.com/jxpeng98/qiongli/actions/runs/";
const ALLOWED_DOWNLOAD_HOSTS: &[&str] = &[
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];
const MAX_JSON_BYTES: u64 = 2 * 1024 * 1024;
const MAX_LOCK_BYTES: u64 = 8 * 1024 * 1024;
const MAX_EVIDENCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ASSET_BYTES: u64 = 512 * 1024 * 1024;

const REQUIRED_GATES: [(&str, &str); 7] = [
    ("finder-clean-machine", "finder-clean-machine.receipt.json"),
    ("packaged-ui", "packaged-ui.receipt.json"),
    ("real-clients", "real-clients.receipt.json"),
    ("production-update", "production-update.receipt.json"),
    ("rollback", "rollback.receipt.json"),
    ("accessibility", "accessibility.receipt.json"),
    ("exact-head-ci", "exact-head-ci.receipt.json"),
];

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    if env!("CARGO_PKG_VERSION") != VERSION {
        return Err("alpha1-release-evidence-product-version-mismatch");
    }
    match Command::parse(env::args_os().skip(1))? {
        Command::PreparePreflight(arguments) => prepare_preflight(&arguments),
        Command::PrepareProduction(arguments) => prepare_production(&arguments),
        Command::Finalize(arguments) => finalize_publication_ledger(&arguments),
    }
}

enum Command {
    PreparePreflight(PreparePreflightArguments),
    PrepareProduction(PrepareProductionArguments),
    Finalize(FinalizeArguments),
}

impl Command {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let mut values = values.into_iter();
        let command = values
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or("alpha1-release-evidence-usage-invalid")?;
        let options = OptionMap::parse(values)?;
        match command.as_str() {
            "prepare-preflight" => Ok(Self::PreparePreflight(PreparePreflightArguments::parse(
                options,
            )?)),
            "prepare-production" => Ok(Self::PrepareProduction(PrepareProductionArguments::parse(
                options,
            )?)),
            "finalize" => Ok(Self::Finalize(FinalizeArguments::parse(options)?)),
            _ => Err("alpha1-release-evidence-usage-invalid"),
        }
    }
}

struct CommonPrepareArguments {
    source_commit: String,
    cargo_lock: PathBuf,
    build_run_url: String,
    build_started_at: String,
    build_finished_at: String,
    output_dir: PathBuf,
}

impl CommonPrepareArguments {
    fn parse(options: &mut OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            source_commit: options.text("--source-commit")?,
            cargo_lock: options.path("--cargo-lock")?,
            build_run_url: options.text("--build-run-url")?,
            build_started_at: options.text("--build-started-at")?,
            build_finished_at: options.text("--build-finished-at")?,
            output_dir: options.path("--output-dir")?,
        };
        validate_common_prepare_arguments(&arguments)?;
        Ok(arguments)
    }
}

struct PreparePreflightArguments {
    common: CommonPrepareArguments,
    artifact_dir: PathBuf,
}

impl PreparePreflightArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            common: CommonPrepareArguments::parse(&mut options)?,
            artifact_dir: options.path("--artifact-dir")?,
        };
        options.finish()?;
        validate_input_directory(&arguments.artifact_dir)?;
        Ok(arguments)
    }
}

struct PrepareProductionArguments {
    common: CommonPrepareArguments,
    signed_artifact_dir: PathBuf,
    update_metadata_dir: PathBuf,
    candidate_dir: PathBuf,
    authority: PathBuf,
}

impl PrepareProductionArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            common: CommonPrepareArguments::parse(&mut options)?,
            signed_artifact_dir: options.path("--signed-artifact-dir")?,
            update_metadata_dir: options.path("--update-metadata-dir")?,
            candidate_dir: options.path("--candidate-dir")?,
            authority: options.path("--authority")?,
        };
        options.finish()?;
        validate_input_directory(&arguments.signed_artifact_dir)?;
        validate_input_directory(&arguments.update_metadata_dir)?;
        validate_input_directory(&arguments.candidate_dir)?;
        validate_input_file(&arguments.authority, MAX_JSON_BYTES)?;
        if arguments
            .authority
            .file_name()
            .and_then(|name| name.to_str())
            != Some(AUTHORITY_FILE)
        {
            return Err("alpha1-release-evidence-authority-name-invalid");
        }
        Ok(arguments)
    }
}

struct FinalizeArguments {
    source_commit: String,
    supply_chain_dir: PathBuf,
    gate_evidence_dir: PathBuf,
    output_dir: PathBuf,
}

impl FinalizeArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            source_commit: options.text("--source-commit")?,
            supply_chain_dir: options.path("--supply-chain-dir")?,
            gate_evidence_dir: options.path("--gate-evidence-dir")?,
            output_dir: options.path("--output-dir")?,
        };
        options.finish()?;
        if !valid_source_commit(&arguments.source_commit) {
            return Err("alpha1-release-evidence-source-commit-invalid");
        }
        validate_input_directory(&arguments.supply_chain_dir)?;
        validate_input_directory(&arguments.gate_evidence_dir)?;
        validate_output_path(&arguments.output_dir)?;
        Ok(arguments)
    }
}

struct OptionMap(BTreeMap<String, OsString>);

impl OptionMap {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.len() % 2 != 0 {
            return Err("alpha1-release-evidence-usage-invalid");
        }
        let mut options = BTreeMap::new();
        for pair in values.chunks_exact(2) {
            let name = pair[0]
                .to_str()
                .filter(|value| value.starts_with("--"))
                .ok_or("alpha1-release-evidence-usage-invalid")?
                .to_string();
            if options.insert(name, pair[1].clone()).is_some() {
                return Err("alpha1-release-evidence-usage-invalid");
            }
        }
        Ok(Self(options))
    }

    fn path(&mut self, name: &str) -> Result<PathBuf, &'static str> {
        self.0
            .remove(name)
            .map(PathBuf::from)
            .ok_or("alpha1-release-evidence-usage-invalid")
    }

    fn text(&mut self, name: &str) -> Result<String, &'static str> {
        self.0
            .remove(name)
            .and_then(|value| value.into_string().ok())
            .filter(|value| !value.is_empty())
            .ok_or("alpha1-release-evidence-usage-invalid")
    }

    fn finish(self) -> Result<(), &'static str> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err("alpha1-release-evidence-usage-invalid")
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum EvidenceClass {
    Preflight,
    Production,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct AssetRecord {
    role: String,
    file: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct SupplyChainReceiptV1 {
    schema_version: u32,
    record_type: String,
    status: String,
    publication_allowed: bool,
    evidence_class: EvidenceClass,
    source_commit: String,
    version: String,
    target: String,
    build_run_url: String,
    build_started_at: String,
    build_finished_at: String,
    cargo_lock_sha256: String,
    dependency_count: usize,
    assets: Vec<AssetRecord>,
    release_set_sha256: String,
    checksums_file: String,
    checksums_sha256: String,
    sbom_file: String,
    sbom_sha256: String,
    provenance_file: String,
    provenance_sha256: String,
    open_gates: Vec<String>,
    reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct GateAttachmentV1 {
    file: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PublicationGateReceiptV1 {
    schema_version: u32,
    record_type: String,
    gate_id: String,
    status: String,
    publication_allowed: bool,
    source_commit: String,
    release_set_sha256: String,
    observed_at: String,
    actor: String,
    environment: String,
    checks: BTreeMap<String, bool>,
    attachments: Vec<GateAttachmentV1>,
}

#[derive(Clone, Debug, Serialize)]
struct LedgerGateV1 {
    gate_id: String,
    receipt_file: String,
    receipt_sha256: String,
    observed_at: String,
    actor: String,
    environment: String,
    checks: BTreeMap<String, bool>,
    attachments: Vec<GateAttachmentV1>,
}

#[derive(Serialize)]
struct PublicationLedgerV1<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    source_commit: &'a str,
    version: &'static str,
    target: &'static str,
    release_set_sha256: &'a str,
    supply_chain_receipt_file: &'static str,
    supply_chain_receipt_sha256: String,
    checksums_sha256: &'a str,
    sbom_sha256: &'a str,
    provenance_sha256: &'a str,
    gates: Vec<LedgerGateV1>,
    remaining_authorization: &'static str,
    reason: &'static str,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DesktopPackageReceiptV1 {
    schema_version: u32,
    status: String,
    product_source_commit: String,
    package_file: String,
    package_size_bytes: u64,
    package_sha256: String,
    package_manifest_file: String,
    package_manifest_sha256: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MacosUpdateSigningReceiptV1 {
    schema_version: u32,
    record_type: String,
    status: String,
    publication_allowed: bool,
    source: ReceiptSourceV1,
    final_artifact: ReceiptArtifactV1,
    signing: ReceiptSigningV1,
    notarization: ReceiptNotarizationV1,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptSourceV1 {
    product_source_commit: String,
    unsigned_manifest_sha256: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptArtifactV1 {
    status: String,
    file: String,
    size_bytes: u64,
    sha256: String,
    launcher_sha256: String,
    canonical_binary_sha256: String,
    update_helper_sha256: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptSigningV1 {
    kind: String,
    verification: String,
    team_identifier: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptNotarizationV1 {
    status: String,
    stapling: String,
    gatekeeper_assessment: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateMetadataReceiptV1 {
    schema_version: u32,
    record_type: String,
    status: String,
    publication_allowed: bool,
    source_commit: String,
    version: String,
    stream: NativeUpdateStream,
    generation: u64,
    authority_sha256: String,
    release_key_id: String,
    launch_key_id: String,
    signed_metadata_file: String,
    signed_metadata_sha256: String,
    signed_payload_sha256: String,
    desktop_manifest_sha256: String,
    signing_receipt_sha256: String,
    archive_sha256: String,
    codex_launch_grant_verified: bool,
    claude_code_launch_grant_verified: bool,
    stable_stream_rejected_prerelease: bool,
    reason: String,
}

#[derive(Clone, Debug)]
struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    dependencies: Vec<String>,
}

#[derive(Default)]
struct LockedPackageBuilder {
    name: Option<String>,
    version: Option<String>,
    source: Option<String>,
    checksum: Option<String>,
    dependencies: Vec<String>,
}

struct ProductionUpdateAssets<'a> {
    signed_archive: &'a AssetRecord,
    desktop_manifest: &'a AssetRecord,
    signing_receipt: &'a AssetRecord,
    update_metadata: &'a AssetRecord,
}

impl LockedPackageBuilder {
    fn finish(self) -> Result<LockedPackage, &'static str> {
        Ok(LockedPackage {
            name: self.name.ok_or("alpha1-release-evidence-lock-invalid")?,
            version: self.version.ok_or("alpha1-release-evidence-lock-invalid")?,
            source: self.source,
            checksum: self.checksum,
            dependencies: self.dependencies,
        })
    }
}

fn prepare_preflight(arguments: &PreparePreflightArguments) -> Result<(), &'static str> {
    assert_exact_files(
        &arguments.artifact_dir,
        [
            UNSIGNED_ARCHIVE_FILE,
            DESKTOP_MANIFEST_FILE,
            DESKTOP_RECEIPT_FILE,
        ],
    )?;
    let archive = asset_record(
        "desktop/unsigned-archive",
        &arguments.artifact_dir.join(UNSIGNED_ARCHIVE_FILE),
        format!("desktop/{UNSIGNED_ARCHIVE_FILE}"),
        MAX_ASSET_BYTES,
    )?;
    let manifest = asset_record(
        "desktop/manifest",
        &arguments.artifact_dir.join(DESKTOP_MANIFEST_FILE),
        format!("desktop/{DESKTOP_MANIFEST_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let receipt = asset_record(
        "desktop/package-receipt",
        &arguments.artifact_dir.join(DESKTOP_RECEIPT_FILE),
        format!("desktop/{DESKTOP_RECEIPT_FILE}"),
        MAX_JSON_BYTES,
    )?;
    validate_desktop_package_receipt(
        &arguments.artifact_dir.join(DESKTOP_RECEIPT_FILE),
        &arguments.common.source_commit,
        &archive,
        &manifest,
    )?;
    write_supply_chain_outputs(
        EvidenceClass::Preflight,
        &arguments.common,
        vec![archive, manifest, receipt],
    )
}

fn prepare_production(arguments: &PrepareProductionArguments) -> Result<(), &'static str> {
    assert_exact_files(
        &arguments.signed_artifact_dir,
        [
            SIGNED_ARCHIVE_FILE,
            DESKTOP_MANIFEST_FILE,
            SIGNING_RECEIPT_FILE,
            SIGNING_BOUNDARY_RECEIPT_FILE,
            UNSIGNED_ACCEPTANCE_RECEIPT_FILE,
        ],
    )?;
    assert_exact_files(
        &arguments.update_metadata_dir,
        [UPDATE_METADATA_FILE, UPDATE_RECEIPT_FILE],
    )?;

    let authority_bytes = read_input_file(&arguments.authority, MAX_JSON_BYTES)?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "alpha1-release-evidence-authority-invalid")?;
    authority
        .validate_product_version(VERSION)
        .map_err(|_| "alpha1-release-evidence-authority-version-invalid")?;

    let signed_archive = asset_record(
        "desktop/signed-notarized-archive",
        &arguments.signed_artifact_dir.join(SIGNED_ARCHIVE_FILE),
        format!("desktop/{SIGNED_ARCHIVE_FILE}"),
        MAX_ASSET_BYTES,
    )?;
    let desktop_manifest = asset_record(
        "desktop/manifest",
        &arguments.signed_artifact_dir.join(DESKTOP_MANIFEST_FILE),
        format!("desktop/{DESKTOP_MANIFEST_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let signing_receipt = asset_record(
        "desktop/signing-receipt",
        &arguments.signed_artifact_dir.join(SIGNING_RECEIPT_FILE),
        format!("desktop/{SIGNING_RECEIPT_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let signing_boundary_receipt = asset_record(
        "desktop/signing-boundary-receipt",
        &arguments
            .signed_artifact_dir
            .join(SIGNING_BOUNDARY_RECEIPT_FILE),
        format!("desktop/{SIGNING_BOUNDARY_RECEIPT_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let unsigned_acceptance_receipt = asset_record(
        "desktop/unsigned-acceptance-receipt",
        &arguments
            .signed_artifact_dir
            .join(UNSIGNED_ACCEPTANCE_RECEIPT_FILE),
        format!("desktop/{UNSIGNED_ACCEPTANCE_RECEIPT_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let signing = validate_production_signing_receipt(
        &arguments.signed_artifact_dir.join(SIGNING_RECEIPT_FILE),
        &arguments.common.source_commit,
        &signed_archive,
        &desktop_manifest,
    )?;
    validate_signing_boundary_receipt(
        &arguments
            .signed_artifact_dir
            .join(SIGNING_BOUNDARY_RECEIPT_FILE),
        &arguments.common.source_commit,
    )?;
    validate_unsigned_acceptance_receipt(
        &arguments
            .signed_artifact_dir
            .join(UNSIGNED_ACCEPTANCE_RECEIPT_FILE),
        &arguments.common.source_commit,
    )?;

    let update_metadata = asset_record(
        "update/signed-beta-metadata",
        &arguments.update_metadata_dir.join(UPDATE_METADATA_FILE),
        format!("update/{UPDATE_METADATA_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let update_receipt = asset_record(
        "update/finalization-receipt",
        &arguments.update_metadata_dir.join(UPDATE_RECEIPT_FILE),
        format!("update/{UPDATE_RECEIPT_FILE}"),
        MAX_JSON_BYTES,
    )?;
    validate_production_update(
        arguments,
        &authority,
        &authority_bytes,
        &signing,
        &ProductionUpdateAssets {
            signed_archive: &signed_archive,
            desktop_manifest: &desktop_manifest,
            signing_receipt: &signing_receipt,
            update_metadata: &update_metadata,
        },
    )?;

    let candidate_assets = validate_production_candidate(arguments, &authority, &authority_bytes)?;
    let authority_asset = asset_record(
        "authority/public-release-authority",
        &arguments.authority,
        format!("authority/{AUTHORITY_FILE}"),
        MAX_JSON_BYTES,
    )?;
    let mut assets = vec![
        signed_archive,
        desktop_manifest,
        signing_receipt,
        signing_boundary_receipt,
        unsigned_acceptance_receipt,
        update_metadata,
        update_receipt,
        authority_asset,
    ];
    assets.extend(candidate_assets);
    write_supply_chain_outputs(EvidenceClass::Production, &arguments.common, assets)
}

fn validate_production_candidate(
    arguments: &PrepareProductionArguments,
    authority: &NativeReleaseAuthority,
    authority_bytes: &[u8],
) -> Result<Vec<AssetRecord>, &'static str> {
    let entries = directory_file_names(&arguments.candidate_dir)?;
    if entries.len() != 3 {
        return Err("alpha1-release-evidence-candidate-set-invalid");
    }
    let candidate_file = entries
        .iter()
        .find(|name| name.ends_with(".candidate.json"))
        .ok_or("alpha1-release-evidence-candidate-set-invalid")?;
    let candidate_bytes = read_input_file(
        &arguments.candidate_dir.join(candidate_file),
        MAX_JSON_BYTES,
    )?;
    let candidate = SignedNativeReleaseCandidateV1::from_json(&candidate_bytes)
        .map_err(|_| "alpha1-release-evidence-candidate-invalid")?;
    if candidate.candidate.source_commit != arguments.common.source_commit {
        return Err("alpha1-release-evidence-candidate-source-mismatch");
    }
    let expected_candidate = native_release_candidate_file_name(&candidate.candidate.artifact)
        .map_err(|_| "alpha1-release-evidence-candidate-invalid")?;
    let archive_file = native_portable_archive_file_name(&candidate.candidate.artifact)
        .map_err(|_| "alpha1-release-evidence-candidate-invalid")?;
    let notes_file = native_release_notes_file_name(&candidate.candidate.artifact)
        .map_err(|_| "alpha1-release-evidence-candidate-invalid")?;
    let expected = [
        archive_file.as_str(),
        expected_candidate.as_str(),
        notes_file.as_str(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    if entries != expected || candidate_file != &expected_candidate {
        return Err("alpha1-release-evidence-candidate-set-invalid");
    }
    let archive_path = arguments.candidate_dir.join(&archive_file);
    let archive_target =
        approve_native_portable_archive_target(&archive_path, &candidate.candidate.artifact)
            .map_err(|_| "alpha1-release-evidence-candidate-archive-invalid")?;
    let notes = read_input_file(&arguments.candidate_dir.join(&notes_file), MAX_JSON_BYTES)?;
    let content =
        qiongli::embedded_content().map_err(|_| "alpha1-release-evidence-content-invalid")?;
    for requested_target in [
        ClientActivationTarget::Codex,
        ClientActivationTarget::ClaudeCode,
    ] {
        let context = NativeReleaseCandidateVerificationContext {
            now_unix: candidate.candidate.not_before_unix,
            expected_artifact: &candidate.candidate.artifact,
            expected_source_commit: &arguments.common.source_commit,
            requested_target,
        };
        candidate
            .verify(authority, &context, content.pack(), &archive_target, &notes)
            .map_err(|_| "alpha1-release-evidence-candidate-verification-failed")?;
    }
    if sha256_hex(authority_bytes)
        != sha256_hex(
            &read_input_file(&arguments.authority, MAX_JSON_BYTES)
                .map_err(|_| "alpha1-release-evidence-authority-invalid")?,
        )
    {
        return Err("alpha1-release-evidence-authority-drift");
    }
    Ok(vec![
        asset_record(
            "candidate/portable-archive",
            &archive_path,
            format!("candidate/{archive_file}"),
            MAX_ASSET_BYTES,
        )?,
        asset_record(
            "candidate/signed-candidate",
            &arguments.candidate_dir.join(&expected_candidate),
            format!("candidate/{expected_candidate}"),
            MAX_JSON_BYTES,
        )?,
        asset_record(
            "candidate/release-notes",
            &arguments.candidate_dir.join(&notes_file),
            format!("candidate/{notes_file}"),
            MAX_JSON_BYTES,
        )?,
    ])
}

fn validate_production_update(
    arguments: &PrepareProductionArguments,
    authority: &NativeReleaseAuthority,
    authority_bytes: &[u8],
    signing: &MacosUpdateSigningReceiptV1,
    assets: &ProductionUpdateAssets<'_>,
) -> Result<(), &'static str> {
    let metadata_bytes = read_input_file(
        &arguments.update_metadata_dir.join(UPDATE_METADATA_FILE),
        MAX_JSON_BYTES,
    )?;
    let signed = SignedNativeUpdateManifestV1::from_json(&metadata_bytes)
        .map_err(|_| "alpha1-release-evidence-update-metadata-invalid")?;
    if signed.manifest.source_commit != arguments.common.source_commit {
        return Err("alpha1-release-evidence-update-source-mismatch");
    }
    let context = NativeUpdateVerificationContext {
        now_unix: signed.manifest.not_before_unix,
        last_accepted_generation: signed.manifest.generation.saturating_sub(1),
        current_version: VERSION,
        selected_stream: NativeUpdateStream::Beta,
        expected_macos_team_id: &signing.signing.team_identifier,
        allowed_download_hosts: ALLOWED_DOWNLOAD_HOSTS,
        allow_current_version: true,
    };
    let verified = signed
        .verify(authority.release_keys(), &context)
        .map_err(|_| "alpha1-release-evidence-update-signature-invalid")?;
    let manifest_bytes = read_input_file(
        &arguments.signed_artifact_dir.join(DESKTOP_MANIFEST_FILE),
        MAX_JSON_BYTES,
    )?;
    let signing_receipt_bytes = read_input_file(
        &arguments.signed_artifact_dir.join(SIGNING_RECEIPT_FILE),
        MAX_JSON_BYTES,
    )?;
    let evidence = verified
        .verify_evidence(&manifest_bytes, &signing_receipt_bytes)
        .map_err(|_| "alpha1-release-evidence-update-evidence-invalid")?;
    for target in [
        ClientActivationTarget::Codex,
        ClientActivationTarget::ClaudeCode,
    ] {
        verified
            .verify_client_plugin_grant(authority, &evidence, target)
            .map_err(|_| "alpha1-release-evidence-update-grant-invalid")?;
    }
    let mut stable_context = context;
    stable_context.selected_stream = NativeUpdateStream::Stable;
    if !matches!(
        signed.verify(authority.release_keys(), &stable_context),
        Err(NativeUpdateError::StreamMismatch)
    ) {
        return Err("alpha1-release-evidence-update-stream-policy-invalid");
    }
    let receipt = read_json::<UpdateMetadataReceiptV1>(
        &arguments.update_metadata_dir.join(UPDATE_RECEIPT_FILE),
        MAX_JSON_BYTES,
    )?;
    if receipt.schema_version != 1
        || receipt.record_type != "qiongli-alpha1-update-metadata-finalization"
        || receipt.status != "signed-verified-nonpublishing"
        || receipt.publication_allowed
        || receipt.source_commit != arguments.common.source_commit
        || receipt.version != VERSION
        || receipt.stream != NativeUpdateStream::Beta
        || receipt.generation != signed.manifest.generation
        || receipt.authority_sha256 != sha256_hex(authority_bytes)
        || receipt.release_key_id != signed.signature.key_id
        || receipt.launch_key_id.is_empty()
        || receipt.signed_metadata_file != UPDATE_METADATA_FILE
        || receipt.signed_metadata_sha256 != assets.update_metadata.sha256
        || receipt.signed_payload_sha256 != verified.signed_payload_sha256()
        || receipt.desktop_manifest_sha256 != assets.desktop_manifest.sha256
        || receipt.signing_receipt_sha256 != assets.signing_receipt.sha256
        || receipt.archive_sha256 != assets.signed_archive.sha256
        || !receipt.codex_launch_grant_verified
        || !receipt.claude_code_launch_grant_verified
        || !receipt.stable_stream_rejected_prerelease
        || signed
            .manifest
            .client_plugins
            .iter()
            .any(|plugin| plugin.signed_launch_grant.signature.key_id != receipt.launch_key_id)
        || receipt.reason
            != "external publication still requires the final exact-head release ledger and maintainer authorization"
    {
        return Err("alpha1-release-evidence-update-receipt-invalid");
    }
    Ok(())
}

fn validate_desktop_package_receipt(
    path: &Path,
    source_commit: &str,
    archive: &AssetRecord,
    manifest: &AssetRecord,
) -> Result<(), &'static str> {
    let receipt = read_json::<DesktopPackageReceiptV1>(path, MAX_JSON_BYTES)?;
    if receipt.schema_version != 1
        || receipt.status != "assembled-unpublished"
        || receipt.product_source_commit != source_commit
        || receipt.package_file != UNSIGNED_ARCHIVE_FILE
        || receipt.package_size_bytes != archive.size_bytes
        || receipt.package_sha256 != archive.sha256
        || receipt.package_manifest_file != DESKTOP_MANIFEST_FILE
        || receipt.package_manifest_sha256 != manifest.sha256
    {
        return Err("alpha1-release-evidence-package-receipt-invalid");
    }
    Ok(())
}

fn validate_production_signing_receipt(
    path: &Path,
    source_commit: &str,
    archive: &AssetRecord,
    manifest: &AssetRecord,
) -> Result<MacosUpdateSigningReceiptV1, &'static str> {
    let receipt = read_json::<MacosUpdateSigningReceiptV1>(path, MAX_JSON_BYTES)?;
    if receipt.schema_version != 1
        || receipt.record_type != "qiongli-macos-update-signing"
        || receipt.status != "signed-notarized-candidate"
        || receipt.publication_allowed
        || receipt.source.product_source_commit != source_commit
        || receipt.source.unsigned_manifest_sha256 != manifest.sha256
        || receipt.final_artifact.status != "produced"
        || receipt.final_artifact.file != SIGNED_ARCHIVE_FILE
        || receipt.final_artifact.size_bytes != archive.size_bytes
        || receipt.final_artifact.sha256 != archive.sha256
        || !is_lower_hex(&receipt.final_artifact.launcher_sha256, 64)
        || !is_lower_hex(&receipt.final_artifact.canonical_binary_sha256, 64)
        || !is_lower_hex(&receipt.final_artifact.update_helper_sha256, 64)
        || receipt.signing.kind != "developer-id-application"
        || receipt.signing.verification != "passed"
        || receipt.signing.team_identifier.is_empty()
        || receipt.notarization.status != "accepted"
        || receipt.notarization.stapling != "passed"
        || receipt.notarization.gatekeeper_assessment != "passed"
    {
        return Err("alpha1-release-evidence-signing-receipt-invalid");
    }
    Ok(receipt)
}

fn validate_signing_boundary_receipt(path: &Path, source_commit: &str) -> Result<(), &'static str> {
    let value = read_json::<Value>(path, MAX_JSON_BYTES)?;
    if value.get("schema_version").and_then(Value::as_u64) != Some(1)
        || value.get("record_type").and_then(Value::as_str)
            != Some("qiongli-macos-alpha1-signing-boundary")
        || value.get("status").and_then(Value::as_str)
            != Some("signed-notarized-nonpublishing-candidate")
        || value.get("publication_allowed").and_then(Value::as_bool) != Some(false)
        || value
            .pointer("/source/product_source_commit")
            .and_then(Value::as_str)
            != Some(source_commit)
        || value
            .pointer("/open_gates/publication")
            .and_then(Value::as_str)
            != Some("blocked")
    {
        return Err("alpha1-release-evidence-signing-boundary-receipt-invalid");
    }
    Ok(())
}

fn validate_unsigned_acceptance_receipt(
    path: &Path,
    source_commit: &str,
) -> Result<(), &'static str> {
    let value = read_json::<Value>(path, MAX_JSON_BYTES)?;
    if value.get("schema_version").and_then(Value::as_u64) != Some(1)
        || value.get("record_type").and_then(Value::as_str)
            != Some("qiongli-macos-alpha1-acceptance")
        || value.get("status").and_then(Value::as_str)
            != Some("accepted-nonpublishing-automated-evidence")
        || value.get("publication_allowed").and_then(Value::as_bool) != Some(false)
        || value
            .pointer("/artifact/product_source_commit")
            .and_then(Value::as_str)
            != Some(source_commit)
    {
        return Err("alpha1-release-evidence-unsigned-acceptance-receipt-invalid");
    }
    Ok(())
}

fn write_supply_chain_outputs(
    evidence_class: EvidenceClass,
    arguments: &CommonPrepareArguments,
    mut assets: Vec<AssetRecord>,
) -> Result<(), &'static str> {
    assets.sort_by(|left, right| left.file.cmp(&right.file));
    if assets.is_empty()
        || assets
            .iter()
            .any(|asset| !safe_logical_path(&asset.file) || !is_lower_hex(&asset.sha256, 64))
    {
        return Err("alpha1-release-evidence-assets-invalid");
    }
    let lock_bytes = read_input_file(&arguments.cargo_lock, MAX_LOCK_BYTES)?;
    let packages = parse_cargo_lock(&lock_bytes)?;
    let checksums = checksum_document(&assets);
    let sbom = cyclonedx_sbom(&packages, arguments, evidence_class)?;
    let provenance = provenance_statement(&packages, arguments, evidence_class, &assets)?;
    let release_set_sha256 = sha256_hex(&canonical_json(&assets)?);

    create_new_private_directory(&arguments.output_dir)?;
    let result = (|| {
        write_new_private_file(&arguments.output_dir.join(CHECKSUMS_FILE), &checksums)?;
        write_new_private_file(&arguments.output_dir.join(SBOM_FILE), &sbom)?;
        write_new_private_file(&arguments.output_dir.join(PROVENANCE_FILE), &provenance)?;
        let open_gates = expected_open_gates(evidence_class);
        let receipt = SupplyChainReceiptV1 {
            schema_version: 1,
            record_type: "qiongli-alpha1-supply-chain-evidence".to_string(),
            status: match evidence_class {
                EvidenceClass::Preflight => "nonpublishing-preflight".to_string(),
                EvidenceClass::Production => "production-assets-verified-nonpublishing".to_string(),
            },
            publication_allowed: false,
            evidence_class,
            source_commit: arguments.source_commit.clone(),
            version: VERSION.to_string(),
            target: TARGET.to_string(),
            build_run_url: arguments.build_run_url.clone(),
            build_started_at: arguments.build_started_at.clone(),
            build_finished_at: arguments.build_finished_at.clone(),
            cargo_lock_sha256: sha256_hex(&lock_bytes),
            dependency_count: packages.len(),
            assets,
            release_set_sha256,
            checksums_file: CHECKSUMS_FILE.to_string(),
            checksums_sha256: sha256_hex(&checksums),
            sbom_file: SBOM_FILE.to_string(),
            sbom_sha256: sha256_hex(&sbom),
            provenance_file: PROVENANCE_FILE.to_string(),
            provenance_sha256: sha256_hex(&provenance),
            open_gates,
            reason: match evidence_class {
                EvidenceClass::Preflight => {
                    "preflight supply-chain evidence cannot authorize publication".to_string()
                }
                EvidenceClass::Production => {
                    "production assets still require bound acceptance gates and explicit maintainer authorization"
                        .to_string()
                }
            },
        };
        write_new_private_file(
            &arguments.output_dir.join(SUPPLY_CHAIN_RECEIPT_FILE),
            &canonical_json(&receipt)?,
        )?;
        assert_exact_files(
            &arguments.output_dir,
            [
                CHECKSUMS_FILE,
                SBOM_FILE,
                PROVENANCE_FILE,
                SUPPLY_CHAIN_RECEIPT_FILE,
            ],
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&arguments.output_dir);
    }
    result
}

fn finalize_publication_ledger(arguments: &FinalizeArguments) -> Result<(), &'static str> {
    let supply_chain =
        validate_supply_chain(&arguments.supply_chain_dir, &arguments.source_commit)?;
    if supply_chain.evidence_class != EvidenceClass::Production {
        return Err("alpha1-release-evidence-production-supply-chain-required");
    }
    let gates = validate_gate_evidence(
        &arguments.gate_evidence_dir,
        &arguments.source_commit,
        &supply_chain.release_set_sha256,
    )?;
    let receipt_bytes = read_input_file(
        &arguments.supply_chain_dir.join(SUPPLY_CHAIN_RECEIPT_FILE),
        MAX_JSON_BYTES,
    )?;
    let ledger = PublicationLedgerV1 {
        schema_version: 1,
        record_type: "qiongli-alpha1-publication-ledger",
        status: "publication-evidence-complete-maintainer-authorization-required",
        publication_allowed: false,
        source_commit: &arguments.source_commit,
        version: VERSION,
        target: TARGET,
        release_set_sha256: &supply_chain.release_set_sha256,
        supply_chain_receipt_file: SUPPLY_CHAIN_RECEIPT_FILE,
        supply_chain_receipt_sha256: sha256_hex(&receipt_bytes),
        checksums_sha256: &supply_chain.checksums_sha256,
        sbom_sha256: &supply_chain.sbom_sha256,
        provenance_sha256: &supply_chain.provenance_sha256,
        gates,
        remaining_authorization: "explicit-maintainer-publication-authorization",
        reason: "the ledger proves evidence completeness but does not create a tag, upload assets, update an endpoint, or authorize publication",
    };
    let ledger_bytes = canonical_json(&ledger)?;
    let ledger_checksum = format!("{}  {PUBLICATION_LEDGER_FILE}\n", sha256_hex(&ledger_bytes));
    create_new_private_directory(&arguments.output_dir)?;
    let result = (|| {
        write_new_private_file(
            &arguments.output_dir.join(PUBLICATION_LEDGER_FILE),
            &ledger_bytes,
        )?;
        write_new_private_file(
            &arguments.output_dir.join(PUBLICATION_LEDGER_SHA256_FILE),
            ledger_checksum.as_bytes(),
        )?;
        assert_exact_files(
            &arguments.output_dir,
            [PUBLICATION_LEDGER_FILE, PUBLICATION_LEDGER_SHA256_FILE],
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&arguments.output_dir);
    }
    result?;
    println!(
        "{}",
        String::from_utf8(ledger_bytes)
            .map_err(|_| "alpha1-release-evidence-ledger-encoding-invalid")?
    );
    Ok(())
}

fn validate_supply_chain(
    directory: &Path,
    source_commit: &str,
) -> Result<SupplyChainReceiptV1, &'static str> {
    assert_exact_files(
        directory,
        [
            CHECKSUMS_FILE,
            SBOM_FILE,
            PROVENANCE_FILE,
            SUPPLY_CHAIN_RECEIPT_FILE,
        ],
    )?;
    let receipt_bytes =
        read_input_file(&directory.join(SUPPLY_CHAIN_RECEIPT_FILE), MAX_JSON_BYTES)?;
    let receipt = parse_canonical_json::<SupplyChainReceiptV1>(&receipt_bytes)?;
    if receipt.schema_version != 1
        || receipt.record_type != "qiongli-alpha1-supply-chain-evidence"
        || receipt.publication_allowed
        || receipt.source_commit != source_commit
        || receipt.version != VERSION
        || receipt.target != TARGET
        || !valid_build_run_url(&receipt.build_run_url)
        || !valid_rfc3339_utc(&receipt.build_started_at)
        || !valid_rfc3339_utc(&receipt.build_finished_at)
        || receipt.build_started_at > receipt.build_finished_at
        || !is_lower_hex(&receipt.cargo_lock_sha256, 64)
        || receipt.dependency_count == 0
        || receipt.assets.is_empty()
        || !valid_asset_inventory(receipt.evidence_class, &receipt.assets)
        || receipt.release_set_sha256 != sha256_hex(&canonical_json(&receipt.assets)?)
        || receipt.checksums_file != CHECKSUMS_FILE
        || receipt.sbom_file != SBOM_FILE
        || receipt.provenance_file != PROVENANCE_FILE
    {
        return Err("alpha1-release-evidence-supply-chain-receipt-invalid");
    }
    let expected_status = match receipt.evidence_class {
        EvidenceClass::Preflight => "nonpublishing-preflight",
        EvidenceClass::Production => "production-assets-verified-nonpublishing",
    };
    let expected_open_gates = expected_open_gates(receipt.evidence_class);
    let expected_reason = match receipt.evidence_class {
        EvidenceClass::Preflight => "preflight supply-chain evidence cannot authorize publication",
        EvidenceClass::Production => {
            "production assets still require bound acceptance gates and explicit maintainer authorization"
        }
    };
    if receipt.status != expected_status
        || receipt.open_gates != expected_open_gates
        || receipt.reason != expected_reason
    {
        return Err("alpha1-release-evidence-supply-chain-receipt-invalid");
    }
    let checksums = read_input_file(&directory.join(CHECKSUMS_FILE), MAX_JSON_BYTES)?;
    let sbom = read_input_file(&directory.join(SBOM_FILE), MAX_JSON_BYTES)?;
    let provenance = read_input_file(&directory.join(PROVENANCE_FILE), MAX_JSON_BYTES)?;
    if receipt.checksums_sha256 != sha256_hex(&checksums)
        || receipt.sbom_sha256 != sha256_hex(&sbom)
        || receipt.provenance_sha256 != sha256_hex(&provenance)
        || checksums != checksum_document(&receipt.assets)
    {
        return Err("alpha1-release-evidence-supply-chain-digest-mismatch");
    }
    validate_sbom(&sbom, source_commit, receipt.dependency_count)?;
    validate_provenance(&provenance, source_commit, &receipt.assets)?;
    Ok(receipt)
}

fn validate_gate_evidence(
    directory: &Path,
    source_commit: &str,
    release_set_sha256: &str,
) -> Result<Vec<LedgerGateV1>, &'static str> {
    let actual = directory_file_names(directory)?;
    let mut expected_files = REQUIRED_GATES
        .iter()
        .map(|(_, file)| (*file).to_string())
        .collect::<BTreeSet<_>>();
    let mut gates = Vec::new();
    for (gate_id, receipt_file) in REQUIRED_GATES {
        let receipt_path = directory.join(receipt_file);
        let receipt_bytes = read_input_file(&receipt_path, MAX_JSON_BYTES)?;
        let receipt = serde_json::from_slice::<PublicationGateReceiptV1>(&receipt_bytes)
            .map_err(|_| "alpha1-release-evidence-gate-receipt-invalid")?;
        if receipt.schema_version != 1
            || receipt.record_type != "qiongli-alpha1-publication-gate"
            || receipt.gate_id != gate_id
            || receipt.status != "passed"
            || receipt.publication_allowed
            || receipt.source_commit != source_commit
            || receipt.release_set_sha256 != release_set_sha256
            || !valid_rfc3339_utc(&receipt.observed_at)
            || !valid_label(&receipt.actor)
            || !valid_label(&receipt.environment)
            || receipt.checks.is_empty()
            || receipt
                .checks
                .keys()
                .any(|check| !valid_gate_check_name(check))
            || receipt.checks.values().any(|passed| !passed)
            || receipt.attachments.is_empty()
        {
            return Err("alpha1-release-evidence-gate-receipt-invalid");
        }
        let mut attachment_names = BTreeSet::new();
        for attachment in &receipt.attachments {
            if !safe_file_name(&attachment.file)
                || !attachment_names.insert(attachment.file.clone())
                || expected_files.contains(&attachment.file)
            {
                return Err("alpha1-release-evidence-gate-attachment-invalid");
            }
            let actual_attachment = asset_record(
                "gate/attachment",
                &directory.join(&attachment.file),
                attachment.file.clone(),
                MAX_EVIDENCE_BYTES,
            )?;
            if actual_attachment.size_bytes != attachment.size_bytes
                || actual_attachment.sha256 != attachment.sha256
            {
                return Err("alpha1-release-evidence-gate-attachment-mismatch");
            }
            expected_files.insert(attachment.file.clone());
        }
        gates.push(LedgerGateV1 {
            gate_id: gate_id.to_string(),
            receipt_file: receipt_file.to_string(),
            receipt_sha256: sha256_hex(&receipt_bytes),
            observed_at: receipt.observed_at,
            actor: receipt.actor,
            environment: receipt.environment,
            checks: receipt.checks,
            attachments: receipt.attachments,
        });
    }
    if actual != expected_files {
        return Err("alpha1-release-evidence-gate-evidence-drift");
    }
    Ok(gates)
}

fn checksum_document(assets: &[AssetRecord]) -> Vec<u8> {
    assets
        .iter()
        .map(|asset| format!("{}  {}\n", asset.sha256, asset.file))
        .collect::<String>()
        .into_bytes()
}

fn cyclonedx_sbom(
    packages: &[LockedPackage],
    arguments: &CommonPrepareArguments,
    evidence_class: EvidenceClass,
) -> Result<Vec<u8>, &'static str> {
    let references = package_references(packages);
    let components = packages
        .iter()
        .enumerate()
        .filter(|(_, package)| package.name != "qiongli")
        .map(|(index, package)| {
            let mut component = json!({
                "type": "library",
                "bom-ref": references[index],
                "name": package.name,
                "version": package.version,
                "purl": cargo_purl(package),
                "properties": [{
                    "name": "qiongli:cargo-source",
                    "value": package.source.as_deref().unwrap_or("workspace")
                }]
            });
            if let Some(checksum) = &package.checksum {
                component["hashes"] = json!([{
                    "alg": "SHA-256",
                    "content": checksum
                }]);
            }
            component
        })
        .collect::<Vec<_>>();
    let dependencies = dependency_graph(packages, &references);
    let serial = deterministic_uuid(
        format!(
            "{}:{:?}:{}",
            arguments.source_commit, evidence_class, arguments.build_run_url
        )
        .as_bytes(),
    );
    canonical_json(&json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "serialNumber": format!("urn:uuid:{serial}"),
        "version": 1,
        "metadata": {
            "timestamp": arguments.build_finished_at,
            "tools": {
                "components": [{
                    "type": "application",
                    "name": "qiongli-alpha1-release-evidence",
                    "version": VERSION
                }]
            },
            "component": {
                "type": "application",
                "bom-ref": format!("pkg:cargo/qiongli@{VERSION}"),
                "name": "qiongli",
                "version": VERSION,
                "purl": format!("pkg:cargo/qiongli@{VERSION}"),
                "properties": [
                    {"name": "qiongli:source-commit", "value": arguments.source_commit},
                    {"name": "qiongli:evidence-class", "value": evidence_class_name(evidence_class)},
                    {"name": "qiongli:target", "value": TARGET}
                ]
            }
        },
        "components": components,
        "dependencies": dependencies
    }))
}

fn provenance_statement(
    packages: &[LockedPackage],
    arguments: &CommonPrepareArguments,
    evidence_class: EvidenceClass,
    assets: &[AssetRecord],
) -> Result<Vec<u8>, &'static str> {
    let subjects = assets
        .iter()
        .map(|asset| {
            json!({
                "name": asset.file,
                "digest": {"sha256": asset.sha256}
            })
        })
        .collect::<Vec<_>>();
    let mut resolved = vec![json!({
        "uri": format!("git+https://github.com/jxpeng98/qiongli@{}", arguments.source_commit),
        "digest": {"gitCommit": arguments.source_commit}
    })];
    resolved.extend(packages.iter().filter_map(|package| {
        package.checksum.as_ref().map(|checksum| {
            json!({
                "uri": cargo_purl(package),
                "digest": {"sha256": checksum}
            })
        })
    }));
    canonical_json(&json!({
        "_type": "https://in-toto.io/Statement/v1",
        "subject": subjects,
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {
            "buildDefinition": {
                "buildType": "https://github.com/jxpeng98/qiongli/blob/2.x/.github/workflows/native-ci.yml",
                "externalParameters": {
                    "sourceCommit": arguments.source_commit,
                    "version": VERSION,
                    "target": TARGET,
                    "evidenceClass": evidence_class_name(evidence_class),
                    "cargoLocked": true
                },
                "internalParameters": {},
                "resolvedDependencies": resolved
            },
            "runDetails": {
                "builder": {
                    "id": "https://github.com/jxpeng98/qiongli/actions/workflows/native-ci.yml"
                },
                "metadata": {
                    "invocationId": arguments.build_run_url,
                    "startedOn": arguments.build_started_at,
                    "finishedOn": arguments.build_finished_at
                }
            }
        }
    }))
}

fn validate_sbom(
    bytes: &[u8],
    source_commit: &str,
    dependency_count: usize,
) -> Result<(), &'static str> {
    let value = parse_canonical_json::<Value>(bytes)?;
    if value.get("bomFormat").and_then(Value::as_str) != Some("CycloneDX")
        || value.get("specVersion").and_then(Value::as_str) != Some("1.6")
        || value
            .pointer("/metadata/component/version")
            .and_then(Value::as_str)
            != Some(VERSION)
        || value
            .pointer("/metadata/component/properties")
            .and_then(Value::as_array)
            .is_none_or(|properties| {
                !properties.iter().any(|property| {
                    property.get("name").and_then(Value::as_str) == Some("qiongli:source-commit")
                        && property.get("value").and_then(Value::as_str) == Some(source_commit)
                })
            })
        || value
            .get("components")
            .and_then(Value::as_array)
            .is_none_or(|components| components.len().saturating_add(1) < dependency_count)
    {
        return Err("alpha1-release-evidence-sbom-invalid");
    }
    Ok(())
}

fn validate_provenance(
    bytes: &[u8],
    source_commit: &str,
    assets: &[AssetRecord],
) -> Result<(), &'static str> {
    let value = parse_canonical_json::<Value>(bytes)?;
    let subjects = value
        .get("subject")
        .and_then(Value::as_array)
        .ok_or("alpha1-release-evidence-provenance-invalid")?;
    let actual = subjects
        .iter()
        .map(|subject| {
            let name = subject
                .get("name")
                .and_then(Value::as_str)
                .ok_or("alpha1-release-evidence-provenance-invalid")?;
            let digest = subject
                .pointer("/digest/sha256")
                .and_then(Value::as_str)
                .ok_or("alpha1-release-evidence-provenance-invalid")?;
            Ok((name.to_string(), digest.to_string()))
        })
        .collect::<Result<BTreeMap<_, _>, &'static str>>()?;
    let expected = assets
        .iter()
        .map(|asset| (asset.file.clone(), asset.sha256.clone()))
        .collect::<BTreeMap<_, _>>();
    if value.get("_type").and_then(Value::as_str) != Some("https://in-toto.io/Statement/v1")
        || value.get("predicateType").and_then(Value::as_str)
            != Some("https://slsa.dev/provenance/v1")
        || value
            .pointer("/predicate/buildDefinition/externalParameters/sourceCommit")
            .and_then(Value::as_str)
            != Some(source_commit)
        || actual != expected
    {
        return Err("alpha1-release-evidence-provenance-invalid");
    }
    Ok(())
}

fn parse_cargo_lock(bytes: &[u8]) -> Result<Vec<LockedPackage>, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "alpha1-release-evidence-lock-invalid")?;
    if !text.starts_with("# This file is automatically @generated by Cargo.\n")
        || !text.lines().any(|line| line.trim() == "version = 4")
    {
        return Err("alpha1-release-evidence-lock-invalid");
    }
    let mut packages = Vec::new();
    let mut current = None::<LockedPackageBuilder>;
    let mut in_dependencies = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if let Some(builder) = current.take() {
                packages.push(builder.finish()?);
            }
            current = Some(LockedPackageBuilder::default());
            in_dependencies = false;
            continue;
        }
        let Some(builder) = current.as_mut() else {
            continue;
        };
        if in_dependencies {
            if line == "]" {
                in_dependencies = false;
            } else if !line.is_empty() {
                let value = line
                    .strip_suffix(',')
                    .ok_or("alpha1-release-evidence-lock-invalid")?;
                builder.dependencies.push(parse_quoted(value)?);
            }
            continue;
        }
        if line == "dependencies = [" {
            in_dependencies = true;
        } else if let Some(value) = line.strip_prefix("name = ") {
            builder.name = Some(parse_quoted(value)?);
        } else if let Some(value) = line.strip_prefix("version = ") {
            builder.version = Some(parse_quoted(value)?);
        } else if let Some(value) = line.strip_prefix("source = ") {
            builder.source = Some(parse_quoted(value)?);
        } else if let Some(value) = line.strip_prefix("checksum = ") {
            let checksum = parse_quoted(value)?;
            if !is_lower_hex(&checksum, 64) {
                return Err("alpha1-release-evidence-lock-invalid");
            }
            builder.checksum = Some(checksum);
        }
    }
    if in_dependencies {
        return Err("alpha1-release-evidence-lock-invalid");
    }
    if let Some(builder) = current {
        packages.push(builder.finish()?);
    }
    if packages.is_empty()
        || !packages
            .iter()
            .any(|package| package.name == "qiongli" && package.version == VERSION)
    {
        return Err("alpha1-release-evidence-lock-invalid");
    }
    packages.sort_by(|left, right| {
        (&left.name, &left.version, &left.source).cmp(&(&right.name, &right.version, &right.source))
    });
    Ok(packages)
}

fn parse_quoted(value: &str) -> Result<String, &'static str> {
    serde_json::from_str::<String>(value).map_err(|_| "alpha1-release-evidence-lock-invalid")
}

fn package_references(packages: &[LockedPackage]) -> Vec<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    packages
        .iter()
        .map(|package| {
            let base = cargo_purl(package);
            let count = counts.entry(base.clone()).or_default();
            *count += 1;
            if *count == 1 {
                base
            } else {
                format!("{base}#{}", *count)
            }
        })
        .collect()
}

fn dependency_graph(packages: &[LockedPackage], references: &[String]) -> Vec<Value> {
    let mut by_name = BTreeMap::<&str, Vec<usize>>::new();
    let mut by_name_version = BTreeMap::<(&str, &str), Vec<usize>>::new();
    for (index, package) in packages.iter().enumerate() {
        by_name.entry(&package.name).or_default().push(index);
        by_name_version
            .entry((&package.name, &package.version))
            .or_default()
            .push(index);
    }
    packages
        .iter()
        .enumerate()
        .map(|(index, package)| {
            let depends_on = package
                .dependencies
                .iter()
                .filter_map(|dependency| {
                    let mut parts = dependency.split_whitespace();
                    let name = parts.next()?;
                    let possible_version = parts
                        .next()
                        .filter(|value| value.as_bytes().first().is_some_and(u8::is_ascii_digit));
                    let candidates = possible_version
                        .and_then(|version| by_name_version.get(&(name, version)))
                        .or_else(|| by_name.get(name));
                    candidates
                        .filter(|matches| matches.len() == 1)
                        .map(|matches| references[matches[0]].clone())
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            json!({
                "ref": references[index],
                "dependsOn": depends_on
            })
        })
        .collect()
}

fn cargo_purl(package: &LockedPackage) -> String {
    let mut value = format!(
        "pkg:cargo/{}@{}",
        percent_encode(&package.name),
        percent_encode(&package.version)
    );
    if let Some(source) = &package.source {
        value.push_str("?repository_url=");
        value.push_str(&percent_encode(source));
    }
    value
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn deterministic_uuid(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn evidence_class_name(value: EvidenceClass) -> &'static str {
    match value {
        EvidenceClass::Preflight => "preflight",
        EvidenceClass::Production => "production",
    }
}

fn expected_open_gates(value: EvidenceClass) -> Vec<String> {
    let gates = REQUIRED_GATES.iter().map(|(gate, _)| (*gate).to_string());
    match value {
        EvidenceClass::Preflight => gates
            .chain([
                "production-signing".to_string(),
                "production-candidate".to_string(),
                "production-update-metadata".to_string(),
                "maintainer-publication-authorization".to_string(),
            ])
            .collect(),
        EvidenceClass::Production => gates
            .chain(["maintainer-publication-authorization".to_string()])
            .collect(),
    }
}

fn valid_asset_inventory(value: EvidenceClass, assets: &[AssetRecord]) -> bool {
    let actual = assets
        .iter()
        .map(|asset| (asset.role.as_str(), asset.file.as_str()))
        .collect::<BTreeSet<_>>();
    if actual.len() != assets.len()
        || assets.iter().any(|asset| {
            !valid_label(&asset.role)
                || !safe_logical_path(&asset.file)
                || asset.size_bytes == 0
                || !is_lower_hex(&asset.sha256, 64)
        })
        || !assets.windows(2).all(|pair| pair[0].file < pair[1].file)
    {
        return false;
    }
    let expected = match value {
        EvidenceClass::Preflight => [
            (
                "desktop/manifest",
                format!("desktop/{DESKTOP_MANIFEST_FILE}"),
            ),
            (
                "desktop/package-receipt",
                format!("desktop/{DESKTOP_RECEIPT_FILE}"),
            ),
            (
                "desktop/unsigned-archive",
                format!("desktop/{UNSIGNED_ARCHIVE_FILE}"),
            ),
        ]
        .into_iter()
        .map(|(role, file)| (role.to_string(), file))
        .collect::<BTreeSet<_>>(),
        EvidenceClass::Production => [
            (
                "authority/public-release-authority",
                format!("authority/{AUTHORITY_FILE}"),
            ),
            (
                "candidate/portable-archive",
                format!("candidate/{PORTABLE_ARCHIVE_FILE}"),
            ),
            (
                "candidate/release-notes",
                format!("candidate/{RELEASE_NOTES_FILE}"),
            ),
            (
                "candidate/signed-candidate",
                format!("candidate/{CANDIDATE_FILE}"),
            ),
            (
                "desktop/manifest",
                format!("desktop/{DESKTOP_MANIFEST_FILE}"),
            ),
            (
                "desktop/signed-notarized-archive",
                format!("desktop/{SIGNED_ARCHIVE_FILE}"),
            ),
            (
                "desktop/signing-boundary-receipt",
                format!("desktop/{SIGNING_BOUNDARY_RECEIPT_FILE}"),
            ),
            (
                "desktop/signing-receipt",
                format!("desktop/{SIGNING_RECEIPT_FILE}"),
            ),
            (
                "desktop/unsigned-acceptance-receipt",
                format!("desktop/{UNSIGNED_ACCEPTANCE_RECEIPT_FILE}"),
            ),
            (
                "update/finalization-receipt",
                format!("update/{UPDATE_RECEIPT_FILE}"),
            ),
            (
                "update/signed-beta-metadata",
                format!("update/{UPDATE_METADATA_FILE}"),
            ),
        ]
        .into_iter()
        .map(|(role, file)| (role.to_string(), file))
        .collect::<BTreeSet<_>>(),
    };
    actual
        == expected
            .iter()
            .map(|(role, file)| (role.as_str(), file.as_str()))
            .collect()
}

fn validate_common_prepare_arguments(
    arguments: &CommonPrepareArguments,
) -> Result<(), &'static str> {
    if !valid_source_commit(&arguments.source_commit)
        || !valid_build_run_url(&arguments.build_run_url)
        || !valid_rfc3339_utc(&arguments.build_started_at)
        || !valid_rfc3339_utc(&arguments.build_finished_at)
        || arguments.build_started_at > arguments.build_finished_at
    {
        return Err("alpha1-release-evidence-build-identity-invalid");
    }
    validate_input_file(&arguments.cargo_lock, MAX_LOCK_BYTES)?;
    validate_output_path(&arguments.output_dir)
}

fn valid_build_run_url(value: &str) -> bool {
    value.strip_prefix(GITHUB_RUN_PREFIX).is_some_and(|run_id| {
        !run_id.is_empty() && run_id.len() <= 24 && run_id.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn valid_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }
    let number =
        |start: usize, end: usize| value[start..end].parse::<u32>().ok().unwrap_or_default();
    let month = number(5, 7);
    let day = number(8, 10);
    let hour = number(11, 13);
    let minute = number(14, 16);
    let second = number(17, 19);
    (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && hour <= 23
        && minute <= 59
        && second <= 60
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
}

fn valid_gate_check_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn safe_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value != "."
        && value != ".."
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn safe_logical_path(value: &str) -> bool {
    let mut parts = value.split('/');
    let first = parts.next();
    let second = parts.next();
    parts.next().is_none()
        && first.is_some_and(safe_file_name)
        && second.is_some_and(safe_file_name)
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_input_directory(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path) {
        return Err("alpha1-release-evidence-path-invalid");
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "alpha1-release-evidence-path-invalid")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("alpha1-release-evidence-path-invalid");
    }
    Ok(())
}

fn validate_input_file(path: &Path, max_bytes: u64) -> Result<(), &'static str> {
    if !valid_absolute_path(path) {
        return Err("alpha1-release-evidence-path-invalid");
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "alpha1-release-evidence-path-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > max_bytes
    {
        return Err("alpha1-release-evidence-path-invalid");
    }
    Ok(())
}

fn validate_output_path(path: &Path) -> Result<(), &'static str> {
    if !valid_absolute_path(path)
        || path.exists()
        || path.parent().is_none()
        || path.parent().is_some_and(|parent| !parent.is_dir())
        || path.parent().is_some_and(|parent| {
            fs::symlink_metadata(parent)
                .map(|metadata| metadata.file_type().is_symlink() || !metadata.is_dir())
                .unwrap_or(true)
        })
        || !outside_checkout(path)
    {
        return Err("alpha1-release-evidence-output-invalid");
    }
    Ok(())
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
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

fn directory_file_names(path: &Path) -> Result<BTreeSet<String>, &'static str> {
    fs::read_dir(path)
        .map_err(|_| "alpha1-release-evidence-directory-read-failed")?
        .map(|entry| {
            let entry = entry.map_err(|_| "alpha1-release-evidence-directory-read-failed")?;
            let metadata = entry
                .file_type()
                .map_err(|_| "alpha1-release-evidence-directory-read-failed")?;
            if !metadata.is_file() || metadata.is_symlink() {
                return Err("alpha1-release-evidence-directory-entry-invalid");
            }
            entry
                .file_name()
                .into_string()
                .map_err(|_| "alpha1-release-evidence-directory-entry-invalid")
        })
        .collect()
}

fn assert_exact_files<const N: usize>(
    path: &Path,
    expected: [&str; N],
) -> Result<(), &'static str> {
    let expected = expected
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if directory_file_names(path)? != expected {
        return Err("alpha1-release-evidence-directory-drift");
    }
    Ok(())
}

fn asset_record(
    role: &str,
    path: &Path,
    logical_file: String,
    max_bytes: u64,
) -> Result<AssetRecord, &'static str> {
    validate_input_file(path, max_bytes)?;
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "alpha1-release-evidence-asset-invalid")?;
    Ok(AssetRecord {
        role: role.to_string(),
        file: logical_file,
        size_bytes: metadata.len(),
        sha256: sha256_file(path, max_bytes)?,
    })
}

fn sha256_file(path: &Path, max_bytes: u64) -> Result<String, &'static str> {
    let mut file = File::open(path).map_err(|_| "alpha1-release-evidence-file-read-failed")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut total = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "alpha1-release-evidence-file-read-failed")?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(
                u64::try_from(read).map_err(|_| "alpha1-release-evidence-file-read-failed")?,
            )
            .ok_or("alpha1-release-evidence-file-read-failed")?;
        if total > max_bytes {
            return Err("alpha1-release-evidence-file-too-large");
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn read_input_file(path: &Path, max_bytes: u64) -> Result<Vec<u8>, &'static str> {
    validate_input_file(path, max_bytes)?;
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "alpha1-release-evidence-file-read-failed")?
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "alpha1-release-evidence-file-read-failed")?;
    if bytes.is_empty()
        || u64::try_from(bytes.len()).map_err(|_| "alpha1-release-evidence-file-too-large")?
            > max_bytes
    {
        return Err("alpha1-release-evidence-file-too-large");
    }
    Ok(bytes)
}

fn read_json<T: DeserializeOwned>(path: &Path, max_bytes: u64) -> Result<T, &'static str> {
    serde_json::from_slice(&read_input_file(path, max_bytes)?)
        .map_err(|_| "alpha1-release-evidence-json-invalid")
}

fn parse_canonical_json<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T, &'static str> {
    let value =
        serde_json::from_slice::<T>(bytes).map_err(|_| "alpha1-release-evidence-json-invalid")?;
    if canonical_json(&value)? != bytes {
        return Err("alpha1-release-evidence-json-noncanonical");
    }
    Ok(value)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| "alpha1-release-evidence-json-serialization-failed")
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(unix)]
fn create_new_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| "alpha1-release-evidence-output-create-failed")
}

#[cfg(not(unix))]
fn create_new_private_directory(path: &Path) -> Result<(), &'static str> {
    fs::create_dir(path).map_err(|_| "alpha1-release-evidence-output-create-failed")
}

#[cfg(unix)]
fn create_new_private_file(path: &Path) -> Result<File, &'static str> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "alpha1-release-evidence-output-create-failed")
}

#[cfg(not(unix))]
fn create_new_private_file(path: &Path) -> Result<File, &'static str> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| "alpha1-release-evidence-output-create-failed")
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = create_new_private_file(path)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "alpha1-release-evidence-output-write-failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        env::temp_dir().join(format!(
            "qiongli-alpha1-release-evidence-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn sample_lock() -> Vec<u8> {
        format!(
            r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "dep"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

[[package]]
name = "qiongli"
version = "{VERSION}"
dependencies = [
 "dep",
]
"#
        )
        .into_bytes()
    }

    #[test]
    fn cargo_lock_parser_is_offline_and_deterministic() {
        let packages = parse_cargo_lock(&sample_lock()).expect("lock should parse");
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "dep");
        assert_eq!(packages[1].dependencies, ["dep"]);
        let references = package_references(&packages);
        let first = dependency_graph(&packages, &references);
        let second = dependency_graph(&packages, &references);
        assert_eq!(first, second);
    }

    #[test]
    fn preflight_prepare_emits_bound_standard_evidence() {
        let root = temp_directory("prepare");
        let artifact = root.join("artifact");
        let output = root.join("output");
        let lock = root.join("Cargo.lock");
        fs::create_dir_all(&artifact).expect("artifact directory");
        let source_commit = "a".repeat(40);
        let archive_bytes = b"unsigned-desktop-archive";
        let manifest_bytes = br#"{"manifest":"fixture"}"#;
        fs::write(artifact.join(UNSIGNED_ARCHIVE_FILE), archive_bytes).expect("archive");
        fs::write(artifact.join(DESKTOP_MANIFEST_FILE), manifest_bytes).expect("manifest");
        let receipt = DesktopPackageReceiptV1 {
            schema_version: 1,
            status: "assembled-unpublished".to_string(),
            product_source_commit: source_commit.clone(),
            package_file: UNSIGNED_ARCHIVE_FILE.to_string(),
            package_size_bytes: archive_bytes.len() as u64,
            package_sha256: sha256_hex(archive_bytes),
            package_manifest_file: DESKTOP_MANIFEST_FILE.to_string(),
            package_manifest_sha256: sha256_hex(manifest_bytes),
        };
        fs::write(
            artifact.join(DESKTOP_RECEIPT_FILE),
            canonical_json(&receipt).expect("receipt"),
        )
        .expect("receipt");
        fs::write(&lock, sample_lock()).expect("lock");
        let arguments = PreparePreflightArguments {
            common: CommonPrepareArguments {
                source_commit: source_commit.clone(),
                cargo_lock: lock,
                build_run_url: format!("{GITHUB_RUN_PREFIX}123"),
                build_started_at: "2026-07-16T10:00:00Z".to_string(),
                build_finished_at: "2026-07-16T10:01:00Z".to_string(),
                output_dir: output.clone(),
            },
            artifact_dir: artifact,
        };
        prepare_preflight(&arguments).expect("preflight");
        let evidence = validate_supply_chain(&output, &source_commit).expect("evidence");
        assert_eq!(evidence.evidence_class, EvidenceClass::Preflight);
        assert_eq!(evidence.assets.len(), 3);
        assert_eq!(evidence.dependency_count, 2);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn supply_chain_validation_rejects_preflight_for_finalization() {
        let root = temp_directory("preflight");
        let supply = root.join("supply");
        let gates = root.join("gates");
        let output = root.join("ledger");
        fs::create_dir_all(&gates).expect("gate directory");
        let source_commit = "a".repeat(40);
        let mut assets = vec![
            AssetRecord {
                role: "desktop/unsigned-archive".to_string(),
                file: format!("desktop/{UNSIGNED_ARCHIVE_FILE}"),
                size_bytes: 1,
                sha256: "b".repeat(64),
            },
            AssetRecord {
                role: "desktop/manifest".to_string(),
                file: format!("desktop/{DESKTOP_MANIFEST_FILE}"),
                size_bytes: 1,
                sha256: "c".repeat(64),
            },
            AssetRecord {
                role: "desktop/package-receipt".to_string(),
                file: format!("desktop/{DESKTOP_RECEIPT_FILE}"),
                size_bytes: 1,
                sha256: "d".repeat(64),
            },
        ];
        assets.sort_by(|left, right| left.file.cmp(&right.file));
        fs::create_dir_all(&supply).expect("supply directory");
        let checksums = checksum_document(&assets);
        let sbom = canonical_json(&json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.6",
            "metadata": {
                "component": {
                    "version": VERSION,
                    "properties": [{
                        "name": "qiongli:source-commit",
                        "value": source_commit
                    }]
                }
            },
            "components": []
        }))
        .expect("sbom");
        let provenance_subjects = assets
            .iter()
            .map(|asset| {
                json!({
                    "name": asset.file,
                    "digest": {"sha256": asset.sha256}
                })
            })
            .collect::<Vec<_>>();
        let provenance = canonical_json(&json!({
            "_type": "https://in-toto.io/Statement/v1",
            "subject": provenance_subjects,
            "predicateType": "https://slsa.dev/provenance/v1",
            "predicate": {
                "buildDefinition": {
                    "externalParameters": {"sourceCommit": source_commit}
                }
            }
        }))
        .expect("provenance");
        fs::write(supply.join(CHECKSUMS_FILE), &checksums).expect("checksums");
        fs::write(supply.join(SBOM_FILE), &sbom).expect("sbom");
        fs::write(supply.join(PROVENANCE_FILE), &provenance).expect("provenance");
        let receipt = SupplyChainReceiptV1 {
            schema_version: 1,
            record_type: "qiongli-alpha1-supply-chain-evidence".to_string(),
            status: "nonpublishing-preflight".to_string(),
            publication_allowed: false,
            evidence_class: EvidenceClass::Preflight,
            source_commit: source_commit.clone(),
            version: VERSION.to_string(),
            target: TARGET.to_string(),
            build_run_url: format!("{GITHUB_RUN_PREFIX}123"),
            build_started_at: "2026-07-16T10:00:00Z".to_string(),
            build_finished_at: "2026-07-16T10:01:00Z".to_string(),
            cargo_lock_sha256: "e".repeat(64),
            dependency_count: 1,
            assets: assets.clone(),
            release_set_sha256: sha256_hex(&canonical_json(&assets).expect("assets")),
            checksums_file: CHECKSUMS_FILE.to_string(),
            checksums_sha256: sha256_hex(&checksums),
            sbom_file: SBOM_FILE.to_string(),
            sbom_sha256: sha256_hex(&sbom),
            provenance_file: PROVENANCE_FILE.to_string(),
            provenance_sha256: sha256_hex(&provenance),
            open_gates: expected_open_gates(EvidenceClass::Preflight),
            reason: "preflight supply-chain evidence cannot authorize publication".to_string(),
        };
        fs::write(
            supply.join(SUPPLY_CHAIN_RECEIPT_FILE),
            canonical_json(&receipt).expect("receipt"),
        )
        .expect("receipt");
        let arguments = FinalizeArguments {
            source_commit,
            supply_chain_dir: supply,
            gate_evidence_dir: gates,
            output_dir: output,
        };
        assert_eq!(
            finalize_publication_ledger(&arguments),
            Err("alpha1-release-evidence-production-supply-chain-required")
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn finalizer_requires_and_binds_all_production_gates() {
        let root = temp_directory("finalize");
        let supply = root.join("supply");
        let gates = root.join("gates");
        let output = root.join("ledger");
        fs::create_dir_all(&supply).expect("supply directory");
        fs::create_dir_all(&gates).expect("gate directory");
        let source_commit = "a".repeat(40);
        let inventory = [
            (
                "authority/public-release-authority",
                format!("authority/{AUTHORITY_FILE}"),
            ),
            (
                "candidate/portable-archive",
                format!("candidate/{PORTABLE_ARCHIVE_FILE}"),
            ),
            (
                "candidate/release-notes",
                format!("candidate/{RELEASE_NOTES_FILE}"),
            ),
            (
                "candidate/signed-candidate",
                format!("candidate/{CANDIDATE_FILE}"),
            ),
            (
                "desktop/manifest",
                format!("desktop/{DESKTOP_MANIFEST_FILE}"),
            ),
            (
                "desktop/signed-notarized-archive",
                format!("desktop/{SIGNED_ARCHIVE_FILE}"),
            ),
            (
                "desktop/signing-boundary-receipt",
                format!("desktop/{SIGNING_BOUNDARY_RECEIPT_FILE}"),
            ),
            (
                "desktop/signing-receipt",
                format!("desktop/{SIGNING_RECEIPT_FILE}"),
            ),
            (
                "desktop/unsigned-acceptance-receipt",
                format!("desktop/{UNSIGNED_ACCEPTANCE_RECEIPT_FILE}"),
            ),
            (
                "update/finalization-receipt",
                format!("update/{UPDATE_RECEIPT_FILE}"),
            ),
            (
                "update/signed-beta-metadata",
                format!("update/{UPDATE_METADATA_FILE}"),
            ),
        ];
        let mut assets = inventory
            .into_iter()
            .enumerate()
            .map(|(index, (role, file))| AssetRecord {
                role: role.to_string(),
                file,
                size_bytes: u64::try_from(index + 1).expect("size"),
                sha256: format!("{:064x}", index + 1),
            })
            .collect::<Vec<_>>();
        assets.sort_by(|left, right| left.file.cmp(&right.file));
        let checksums = checksum_document(&assets);
        let sbom = canonical_json(&json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.6",
            "metadata": {
                "component": {
                    "version": VERSION,
                    "properties": [{
                        "name": "qiongli:source-commit",
                        "value": source_commit
                    }]
                }
            },
            "components": []
        }))
        .expect("sbom");
        let subjects = assets
            .iter()
            .map(|asset| {
                json!({
                    "name": asset.file,
                    "digest": {"sha256": asset.sha256}
                })
            })
            .collect::<Vec<_>>();
        let provenance = canonical_json(&json!({
            "_type": "https://in-toto.io/Statement/v1",
            "subject": subjects,
            "predicateType": "https://slsa.dev/provenance/v1",
            "predicate": {
                "buildDefinition": {
                    "externalParameters": {"sourceCommit": source_commit}
                }
            }
        }))
        .expect("provenance");
        fs::write(supply.join(CHECKSUMS_FILE), &checksums).expect("checksums");
        fs::write(supply.join(SBOM_FILE), &sbom).expect("sbom");
        fs::write(supply.join(PROVENANCE_FILE), &provenance).expect("provenance");
        let release_set_sha256 = sha256_hex(&canonical_json(&assets).expect("assets"));
        let receipt = SupplyChainReceiptV1 {
            schema_version: 1,
            record_type: "qiongli-alpha1-supply-chain-evidence".to_string(),
            status: "production-assets-verified-nonpublishing".to_string(),
            publication_allowed: false,
            evidence_class: EvidenceClass::Production,
            source_commit: source_commit.clone(),
            version: VERSION.to_string(),
            target: TARGET.to_string(),
            build_run_url: format!("{GITHUB_RUN_PREFIX}123"),
            build_started_at: "2026-07-16T10:00:00Z".to_string(),
            build_finished_at: "2026-07-16T10:01:00Z".to_string(),
            cargo_lock_sha256: "e".repeat(64),
            dependency_count: 1,
            assets,
            release_set_sha256: release_set_sha256.clone(),
            checksums_file: CHECKSUMS_FILE.to_string(),
            checksums_sha256: sha256_hex(&checksums),
            sbom_file: SBOM_FILE.to_string(),
            sbom_sha256: sha256_hex(&sbom),
            provenance_file: PROVENANCE_FILE.to_string(),
            provenance_sha256: sha256_hex(&provenance),
            open_gates: expected_open_gates(EvidenceClass::Production),
            reason: "production assets still require bound acceptance gates and explicit maintainer authorization".to_string(),
        };
        fs::write(
            supply.join(SUPPLY_CHAIN_RECEIPT_FILE),
            canonical_json(&receipt).expect("receipt"),
        )
        .expect("receipt");
        for (gate_id, receipt_file) in REQUIRED_GATES {
            let attachment_file = format!("{gate_id}.log");
            let attachment_bytes = gate_id.as_bytes();
            fs::write(gates.join(&attachment_file), attachment_bytes).expect("attachment");
            let gate = PublicationGateReceiptV1 {
                schema_version: 1,
                record_type: "qiongli-alpha1-publication-gate".to_string(),
                gate_id: gate_id.to_string(),
                status: "passed".to_string(),
                publication_allowed: false,
                source_commit: source_commit.clone(),
                release_set_sha256: release_set_sha256.clone(),
                observed_at: "2026-07-16T12:00:00Z".to_string(),
                actor: "test-maintainer".to_string(),
                environment: "isolated-test-fixture".to_string(),
                checks: BTreeMap::from([("fixture_check".to_string(), true)]),
                attachments: vec![GateAttachmentV1 {
                    file: attachment_file,
                    size_bytes: attachment_bytes.len() as u64,
                    sha256: sha256_hex(attachment_bytes),
                }],
            };
            fs::write(
                gates.join(receipt_file),
                serde_json::to_vec_pretty(&gate).expect("gate"),
            )
            .expect("gate");
        }
        let arguments = FinalizeArguments {
            source_commit,
            supply_chain_dir: supply,
            gate_evidence_dir: gates,
            output_dir: output.clone(),
        };
        finalize_publication_ledger(&arguments).expect("finalize");
        assert_exact_files(
            &output,
            [PUBLICATION_LEDGER_FILE, PUBLICATION_LEDGER_SHA256_FILE],
        )
        .expect("ledger files");
        let ledger = parse_canonical_json::<Value>(
            &fs::read(output.join(PUBLICATION_LEDGER_FILE)).expect("ledger"),
        )
        .expect("ledger");
        assert_eq!(
            ledger.get("publication_allowed").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            ledger.get("gates").and_then(Value::as_array).map(Vec::len),
            Some(REQUIRED_GATES.len())
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn timestamp_and_path_validation_are_strict() {
        assert!(valid_rfc3339_utc("2026-07-16T10:20:30Z"));
        assert!(!valid_rfc3339_utc("2026-7-16T10:20:30Z"));
        assert!(!valid_rfc3339_utc("2026-07-16T25:20:30Z"));
        assert!(safe_logical_path("desktop/archive.zip"));
        assert!(!safe_logical_path("../archive.zip"));
        assert!(!safe_logical_path("desktop/nested/archive.zip"));
    }
}
