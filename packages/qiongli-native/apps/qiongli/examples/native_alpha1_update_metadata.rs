use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

use qiongli_platform::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, ClientActivationTarget,
    DesktopPackageManifestV1, GrantMode, GrantSignatureV1, GrantVerificationContext, InstallerKind,
    LaunchGrantV1, NativeClientPluginGrantV1, NativeReleaseAuthority, NativeReleaseSignatureV1,
    NativeUpdateError, NativeUpdateManifestV1, NativeUpdateStream, NativeUpdateVerificationContext,
    OperatingSystem, ProductId, ReleaseChannel, SignatureAlgorithm, SignedLaunchGrantV1,
    SignedNativeUpdateManifestV1, launch_grant_signing_bytes, native_update_manifest_signing_bytes,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

const VERSION: &str = "2.0.0-alpha.1";
const SIGNED_ARCHIVE_FILE: &str =
    "qiongli-desktop-2.0.0-alpha.1-macos-aarch64.signed-notarized.app.zip";
const DESKTOP_MANIFEST_FILE: &str = "qiongli-desktop-package.manifest.json";
const SIGNING_RECEIPT_FILE: &str =
    "qiongli-desktop-2.0.0-alpha.1-macos-aarch64.signing.receipt.json";
const UPDATE_METADATA_FILE: &str = "macos-aarch64.json";
const UPDATE_RECEIPT_FILE: &str = "qiongli-alpha1-update-metadata.receipt.json";
const RELEASE_ROOT: &str = "https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-alpha.1";
const ALLOWED_DOWNLOAD_HOSTS: &[&str] = &[
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];
const MAX_REQUEST_BYTES: u64 = 512 * 1024;
const MAX_SIDECAR_BYTES: u64 = 256 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    if env!("CARGO_PKG_VERSION") != VERSION {
        return Err("alpha1-update-metadata-product-version-mismatch");
    }
    match Command::parse(env::args_os().skip(1))? {
        Command::PrepareGrants(arguments) => prepare_grants(&arguments),
        Command::PrepareManifest(arguments) => prepare_manifest(&arguments),
        Command::Finalize(arguments) => finalize(&arguments),
    }
}

enum Command {
    PrepareGrants(PrepareGrantsArguments),
    PrepareManifest(PrepareManifestArguments),
    Finalize(FinalizeArguments),
}

impl Command {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let mut args = args.into_iter();
        let command = args
            .next()
            .and_then(|value| value.into_string().ok())
            .ok_or("alpha1-update-metadata-usage-invalid")?;
        let options = OptionMap::parse(args)?;
        match command.as_str() {
            "prepare-grants" => Ok(Self::PrepareGrants(PrepareGrantsArguments::parse(options)?)),
            "prepare-manifest" => Ok(Self::PrepareManifest(PrepareManifestArguments::parse(
                options,
            )?)),
            "finalize" => Ok(Self::Finalize(FinalizeArguments::parse(options)?)),
            _ => Err("alpha1-update-metadata-usage-invalid"),
        }
    }
}

struct PrepareGrantsArguments {
    signed_artifact_dir: PathBuf,
    authority: PathBuf,
    generation: u64,
    published_at_unix: u64,
    not_before_unix: u64,
    expires_at_unix: u64,
    release_key_id: String,
    launch_key_id: String,
    output: PathBuf,
}

impl PrepareGrantsArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            signed_artifact_dir: options.path("--signed-artifact-dir")?,
            authority: options.path("--authority")?,
            generation: options.number("--generation")?,
            published_at_unix: options.number("--published-at-unix")?,
            not_before_unix: options.number("--not-before-unix")?,
            expires_at_unix: options.number("--expires-at-unix")?,
            release_key_id: options.text("--release-key-id")?,
            launch_key_id: options.text("--launch-key-id")?,
            output: options.path("--output")?,
        };
        options.finish()?;
        Ok(arguments)
    }
}

struct PrepareManifestArguments {
    signed_artifact_dir: PathBuf,
    authority: PathBuf,
    grant_request: PathBuf,
    codex_signature_file: PathBuf,
    claude_signature_file: PathBuf,
    output: PathBuf,
}

impl PrepareManifestArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            signed_artifact_dir: options.path("--signed-artifact-dir")?,
            authority: options.path("--authority")?,
            grant_request: options.path("--grant-request")?,
            codex_signature_file: options.path("--codex-signature-file")?,
            claude_signature_file: options.path("--claude-signature-file")?,
            output: options.path("--output")?,
        };
        options.finish()?;
        Ok(arguments)
    }
}

struct FinalizeArguments {
    signed_artifact_dir: PathBuf,
    authority: PathBuf,
    manifest_request: PathBuf,
    release_signature_file: PathBuf,
    output_dir: PathBuf,
}

impl FinalizeArguments {
    fn parse(mut options: OptionMap) -> Result<Self, &'static str> {
        let arguments = Self {
            signed_artifact_dir: options.path("--signed-artifact-dir")?,
            authority: options.path("--authority")?,
            manifest_request: options.path("--manifest-request")?,
            release_signature_file: options.path("--release-signature-file")?,
            output_dir: options.path("--output-dir")?,
        };
        options.finish()?;
        Ok(arguments)
    }
}

struct OptionMap(BTreeMap<String, OsString>);

impl OptionMap {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let args = args.into_iter().collect::<Vec<_>>();
        if args.len() % 2 != 0 {
            return Err("alpha1-update-metadata-usage-invalid");
        }
        let mut options = BTreeMap::new();
        for pair in args.chunks_exact(2) {
            let name = pair[0]
                .to_str()
                .filter(|value| value.starts_with("--"))
                .ok_or("alpha1-update-metadata-usage-invalid")?
                .to_string();
            if options.insert(name, pair[1].clone()).is_some() {
                return Err("alpha1-update-metadata-usage-invalid");
            }
        }
        Ok(Self(options))
    }

    fn path(&mut self, name: &str) -> Result<PathBuf, &'static str> {
        self.0
            .remove(name)
            .map(PathBuf::from)
            .ok_or("alpha1-update-metadata-usage-invalid")
    }

    fn text(&mut self, name: &str) -> Result<String, &'static str> {
        self.0
            .remove(name)
            .and_then(|value| value.into_string().ok())
            .filter(|value| !value.is_empty())
            .ok_or("alpha1-update-metadata-usage-invalid")
    }

    fn number(&mut self, name: &str) -> Result<u64, &'static str> {
        self.text(name)?
            .parse()
            .map_err(|_| "alpha1-update-metadata-usage-invalid")
    }

    fn finish(self) -> Result<(), &'static str> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err("alpha1-update-metadata-usage-invalid")
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactSetV1 {
    artifact: ArtifactIdentityV1,
    source_commit: String,
    archive_file_name: String,
    archive_size_bytes: u64,
    archive_sha256: String,
    desktop_manifest_file_name: String,
    desktop_manifest_size_bytes: u64,
    desktop_manifest_sha256: String,
    signing_receipt_file_name: String,
    signing_receipt_size_bytes: u64,
    signing_receipt_sha256: String,
    resource_pack_sha256: String,
    signed_canonical_binary_sha256: String,
    macos_team_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct GrantSigningItemV1 {
    target: ClientActivationTarget,
    grant: LaunchGrantV1,
    signing_preimage_sha256: String,
    signing_preimage_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct GrantSigningRequestV1 {
    schema_version: u32,
    record_type: String,
    status: String,
    publication_allowed: bool,
    authority_sha256: String,
    release_key_id: String,
    launch_key_id: String,
    generation: u64,
    published_at_unix: u64,
    not_before_unix: u64,
    expires_at_unix: u64,
    artifact_set: ArtifactSetV1,
    grants: Vec<GrantSigningItemV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ManifestSigningRequestV1 {
    schema_version: u32,
    record_type: String,
    status: String,
    publication_allowed: bool,
    authority_sha256: String,
    grant_request_sha256: String,
    release_key_id: String,
    launch_key_id: String,
    manifest: NativeUpdateManifestV1,
    signing_preimage_sha256: String,
    signing_preimage_hex: String,
}

#[derive(Serialize)]
struct FinalizationReceiptV1<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    publication_allowed: bool,
    source_commit: &'a str,
    version: &'static str,
    stream: NativeUpdateStream,
    generation: u64,
    authority_sha256: &'a str,
    release_key_id: &'a str,
    launch_key_id: &'a str,
    signed_metadata_file: &'static str,
    signed_metadata_sha256: String,
    signed_payload_sha256: &'a str,
    desktop_manifest_sha256: &'a str,
    signing_receipt_sha256: &'a str,
    archive_sha256: &'a str,
    codex_launch_grant_verified: bool,
    claude_code_launch_grant_verified: bool,
    stable_stream_rejected_prerelease: bool,
    reason: &'static str,
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

struct ArtifactEvidence {
    summary: ArtifactSetV1,
    desktop_manifest_bytes: Vec<u8>,
    signing_receipt_bytes: Vec<u8>,
}

struct AuthorityEvidence {
    authority: NativeReleaseAuthority,
    sha256: String,
}

fn prepare_grants(arguments: &PrepareGrantsArguments) -> Result<(), &'static str> {
    validate_timestamps(
        arguments.published_at_unix,
        arguments.not_before_unix,
        arguments.expires_at_unix,
    )?;
    let evidence = load_artifact_evidence(&arguments.signed_artifact_dir)?;
    let authority = load_authority(
        &arguments.authority,
        arguments.generation,
        &arguments.release_key_id,
        &arguments.launch_key_id,
    )?;
    let request = build_grant_request(arguments, &evidence, &authority)?;
    write_new_private_file(&arguments.output, &canonical_json(&request)?)?;
    Ok(())
}

fn build_grant_request(
    arguments: &PrepareGrantsArguments,
    evidence: &ArtifactEvidence,
    authority: &AuthorityEvidence,
) -> Result<GrantSigningRequestV1, &'static str> {
    let mut plugin_artifact = evidence.summary.artifact.clone();
    plugin_artifact.installer_kind = InstallerKind::PluginBundle;
    let grants = [
        ClientActivationTarget::Codex,
        ClientActivationTarget::ClaudeCode,
    ]
    .into_iter()
    .map(|target| {
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: arguments.generation,
            artifact: plugin_artifact.clone(),
            binary_sha256: evidence.summary.signed_canonical_binary_sha256.clone(),
            resource_pack_sha256: evidence.summary.resource_pack_sha256.clone(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![target.integration_scope()],
            not_before_unix: arguments.not_before_unix,
            expires_at_unix: arguments.expires_at_unix,
        };
        let preimage = launch_grant_signing_bytes(&grant)
            .map_err(|_| "alpha1-update-grant-preimage-invalid")?;
        Ok(GrantSigningItemV1 {
            target,
            grant,
            signing_preimage_sha256: sha256_hex(&preimage),
            signing_preimage_hex: encode_hex(&preimage),
        })
    })
    .collect::<Result<Vec<_>, &'static str>>()?;
    Ok(GrantSigningRequestV1 {
        schema_version: 1,
        record_type: "qiongli-alpha1-launch-grant-signing-request".to_string(),
        status: "awaiting-external-launch-grant-signatures".to_string(),
        publication_allowed: false,
        authority_sha256: authority.sha256.clone(),
        release_key_id: arguments.release_key_id.clone(),
        launch_key_id: arguments.launch_key_id.clone(),
        generation: arguments.generation,
        published_at_unix: arguments.published_at_unix,
        not_before_unix: arguments.not_before_unix,
        expires_at_unix: arguments.expires_at_unix,
        artifact_set: evidence.summary.clone(),
        grants,
    })
}

fn prepare_manifest(arguments: &PrepareManifestArguments) -> Result<(), &'static str> {
    let grant_request_bytes = read_input_file(&arguments.grant_request, MAX_REQUEST_BYTES)?;
    let grant_request =
        parse_canonical_json::<GrantSigningRequestV1>(&grant_request_bytes, "grant-request")?;
    let evidence = load_artifact_evidence(&arguments.signed_artifact_dir)?;
    let authority = load_authority(
        &arguments.authority,
        grant_request.generation,
        &grant_request.release_key_id,
        &grant_request.launch_key_id,
    )?;
    validate_grant_request(&grant_request, &evidence, &authority)?;
    let signatures = [
        read_signature(&arguments.codex_signature_file)?,
        read_signature(&arguments.claude_signature_file)?,
    ];
    let client_plugins = grant_request
        .grants
        .iter()
        .zip(signatures)
        .map(|(item, signature)| {
            let signed = SignedLaunchGrantV1 {
                grant: item.grant.clone(),
                signature: GrantSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: grant_request.launch_key_id.clone(),
                    value_hex: signature,
                },
            };
            let context = GrantVerificationContext {
                now_unix: grant_request.not_before_unix,
                minimum_generation: authority.authority.minimum_launch_grant_generation(),
                expected_artifact: &item.grant.artifact,
                binary_sha256: &evidence.summary.signed_canonical_binary_sha256,
                resource_pack_sha256: &evidence.summary.resource_pack_sha256,
                requested_mode: GrantMode::LiteMcp,
                requested_scope: item.target.integration_scope(),
            };
            signed
                .verify(authority.authority.launch_grant_keys(), &context)
                .map_err(|_| "alpha1-update-launch-grant-signature-invalid")?;
            Ok(NativeClientPluginGrantV1 {
                target: item.target,
                signed_launch_grant: signed,
            })
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    let manifest = build_update_manifest(&grant_request, &evidence, client_plugins);
    let preimage = native_update_manifest_signing_bytes(&manifest)
        .map_err(|_| "alpha1-update-manifest-invalid")?;
    let request = ManifestSigningRequestV1 {
        schema_version: 1,
        record_type: "qiongli-alpha1-update-manifest-signing-request".to_string(),
        status: "awaiting-external-release-signature".to_string(),
        publication_allowed: false,
        authority_sha256: authority.sha256,
        grant_request_sha256: sha256_hex(&grant_request_bytes),
        release_key_id: grant_request.release_key_id,
        launch_key_id: grant_request.launch_key_id,
        manifest,
        signing_preimage_sha256: sha256_hex(&preimage),
        signing_preimage_hex: encode_hex(&preimage),
    };
    write_new_private_file(&arguments.output, &canonical_json(&request)?)?;
    Ok(())
}

fn build_update_manifest(
    request: &GrantSigningRequestV1,
    evidence: &ArtifactEvidence,
    client_plugins: Vec<NativeClientPluginGrantV1>,
) -> NativeUpdateManifestV1 {
    NativeUpdateManifestV1 {
        schema_version: 1,
        stream: NativeUpdateStream::Beta,
        generation: request.generation,
        artifact: evidence.summary.artifact.clone(),
        source_commit: evidence.summary.source_commit.clone(),
        minimum_updater_version: VERSION.to_string(),
        archive_file_name: evidence.summary.archive_file_name.clone(),
        archive_url: release_url(&evidence.summary.archive_file_name),
        archive_size_bytes: evidence.summary.archive_size_bytes,
        archive_sha256: evidence.summary.archive_sha256.clone(),
        desktop_manifest_file_name: evidence.summary.desktop_manifest_file_name.clone(),
        desktop_manifest_url: release_url(&evidence.summary.desktop_manifest_file_name),
        desktop_manifest_size_bytes: evidence.summary.desktop_manifest_size_bytes,
        desktop_manifest_sha256: evidence.summary.desktop_manifest_sha256.clone(),
        signing_receipt_file_name: evidence.summary.signing_receipt_file_name.clone(),
        signing_receipt_url: release_url(&evidence.summary.signing_receipt_file_name),
        signing_receipt_size_bytes: evidence.summary.signing_receipt_size_bytes,
        signing_receipt_sha256: evidence.summary.signing_receipt_sha256.clone(),
        resource_pack_sha256: evidence.summary.resource_pack_sha256.clone(),
        client_plugins,
        macos_team_id: evidence.summary.macos_team_id.clone(),
        published_at_unix: request.published_at_unix,
        not_before_unix: request.not_before_unix,
        expires_at_unix: request.expires_at_unix,
    }
}

fn finalize(arguments: &FinalizeArguments) -> Result<(), &'static str> {
    let request_bytes = read_input_file(&arguments.manifest_request, MAX_REQUEST_BYTES)?;
    let request =
        parse_canonical_json::<ManifestSigningRequestV1>(&request_bytes, "manifest-request")?;
    let evidence = load_artifact_evidence(&arguments.signed_artifact_dir)?;
    let authority = load_authority(
        &arguments.authority,
        request.manifest.generation,
        &request.release_key_id,
        &request.launch_key_id,
    )?;
    validate_manifest_request(&request, &evidence, &authority)?;
    let signed = SignedNativeUpdateManifestV1 {
        manifest: request.manifest.clone(),
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: request.release_key_id.clone(),
            value_hex: read_signature(&arguments.release_signature_file)?,
        },
    };
    let context = NativeUpdateVerificationContext {
        now_unix: request.manifest.not_before_unix,
        last_accepted_generation: request.manifest.generation.saturating_sub(1),
        current_version: VERSION,
        selected_stream: NativeUpdateStream::Beta,
        expected_macos_team_id: &request.manifest.macos_team_id,
        allowed_download_hosts: ALLOWED_DOWNLOAD_HOSTS,
        allow_current_version: true,
    };
    let verified = signed
        .verify(authority.authority.release_keys(), &context)
        .map_err(|_| "alpha1-update-release-signature-invalid")?;
    let verified_evidence = verified
        .verify_evidence(
            &evidence.desktop_manifest_bytes,
            &evidence.signing_receipt_bytes,
        )
        .map_err(|_| "alpha1-update-evidence-invalid")?;
    verified
        .verify_client_plugin_grant(
            &authority.authority,
            &verified_evidence,
            ClientActivationTarget::Codex,
        )
        .map_err(|_| "alpha1-update-codex-grant-invalid")?;
    verified
        .verify_client_plugin_grant(
            &authority.authority,
            &verified_evidence,
            ClientActivationTarget::ClaudeCode,
        )
        .map_err(|_| "alpha1-update-claude-grant-invalid")?;
    let mut stable_context = context;
    stable_context.selected_stream = NativeUpdateStream::Stable;
    if !matches!(
        signed.verify(authority.authority.release_keys(), &stable_context),
        Err(NativeUpdateError::StreamMismatch)
    ) {
        return Err("alpha1-update-stable-stream-policy-invalid");
    }

    let signed_bytes = signed
        .to_canonical_json()
        .map_err(|_| "alpha1-update-metadata-serialization-failed")?;
    create_new_private_directory(&arguments.output_dir)?;
    let result = write_final_outputs(
        &arguments.output_dir,
        &request,
        &evidence,
        &verified,
        &signed_bytes,
    );
    if result.is_err() {
        let _ = fs::remove_dir_all(&arguments.output_dir);
    }
    result
}

fn write_final_outputs(
    output_dir: &Path,
    request: &ManifestSigningRequestV1,
    evidence: &ArtifactEvidence,
    verified: &qiongli_platform::VerifiedNativeUpdateManifest,
    signed_bytes: &[u8],
) -> Result<(), &'static str> {
    write_new_private_file(&output_dir.join(UPDATE_METADATA_FILE), signed_bytes)?;
    let receipt = FinalizationReceiptV1 {
        schema_version: 1,
        record_type: "qiongli-alpha1-update-metadata-finalization",
        status: "signed-verified-nonpublishing",
        publication_allowed: false,
        source_commit: &request.manifest.source_commit,
        version: VERSION,
        stream: NativeUpdateStream::Beta,
        generation: request.manifest.generation,
        authority_sha256: &request.authority_sha256,
        release_key_id: &request.release_key_id,
        launch_key_id: &request.launch_key_id,
        signed_metadata_file: UPDATE_METADATA_FILE,
        signed_metadata_sha256: sha256_hex(signed_bytes),
        signed_payload_sha256: verified.signed_payload_sha256(),
        desktop_manifest_sha256: &evidence.summary.desktop_manifest_sha256,
        signing_receipt_sha256: &evidence.summary.signing_receipt_sha256,
        archive_sha256: &evidence.summary.archive_sha256,
        codex_launch_grant_verified: true,
        claude_code_launch_grant_verified: true,
        stable_stream_rejected_prerelease: true,
        reason: "external publication still requires the final exact-head release ledger and maintainer authorization",
    };
    let receipt_bytes = canonical_json(&receipt)?;
    write_new_private_file(&output_dir.join(UPDATE_RECEIPT_FILE), &receipt_bytes)?;
    let actual = fs::read_dir(output_dir)
        .map_err(|_| "alpha1-update-output-read-failed")?
        .map(|entry| {
            entry
                .map_err(|_| "alpha1-update-output-read-failed")?
                .file_name()
                .into_string()
                .map_err(|_| "alpha1-update-output-read-failed")
        })
        .collect::<Result<BTreeSet<_>, &'static str>>()?;
    let expected = [UPDATE_METADATA_FILE, UPDATE_RECEIPT_FILE]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err("alpha1-update-output-drift");
    }
    println!(
        "{}",
        String::from_utf8(receipt_bytes).map_err(|_| "alpha1-update-receipt-invalid")?
    );
    Ok(())
}

fn validate_grant_request(
    request: &GrantSigningRequestV1,
    evidence: &ArtifactEvidence,
    authority: &AuthorityEvidence,
) -> Result<(), &'static str> {
    validate_timestamps(
        request.published_at_unix,
        request.not_before_unix,
        request.expires_at_unix,
    )?;
    if request.schema_version != 1
        || request.record_type != "qiongli-alpha1-launch-grant-signing-request"
        || request.status != "awaiting-external-launch-grant-signatures"
        || request.publication_allowed
        || request.authority_sha256 != authority.sha256
        || request.artifact_set != evidence.summary
        || request.grants.len() != 2
    {
        return Err("alpha1-update-grant-request-invalid");
    }
    let expected_targets = [
        ClientActivationTarget::Codex,
        ClientActivationTarget::ClaudeCode,
    ];
    for (item, expected_target) in request.grants.iter().zip(expected_targets) {
        let preimage = launch_grant_signing_bytes(&item.grant)
            .map_err(|_| "alpha1-update-grant-request-invalid")?;
        if item.target != expected_target
            || item.grant.generation != request.generation
            || item.grant.artifact.installer_kind != InstallerKind::PluginBundle
            || item.grant.binary_sha256 != evidence.summary.signed_canonical_binary_sha256
            || item.grant.resource_pack_sha256 != evidence.summary.resource_pack_sha256
            || item.grant.allowed_modes.as_slice() != [GrantMode::LiteMcp]
            || item.grant.integration_scopes.as_slice() != [expected_target.integration_scope()]
            || item.grant.not_before_unix != request.not_before_unix
            || item.grant.expires_at_unix != request.expires_at_unix
            || item.signing_preimage_sha256 != sha256_hex(&preimage)
            || item.signing_preimage_hex != encode_hex(&preimage)
        {
            return Err("alpha1-update-grant-request-invalid");
        }
    }
    Ok(())
}

fn validate_manifest_request(
    request: &ManifestSigningRequestV1,
    evidence: &ArtifactEvidence,
    authority: &AuthorityEvidence,
) -> Result<(), &'static str> {
    let preimage = native_update_manifest_signing_bytes(&request.manifest)
        .map_err(|_| "alpha1-update-manifest-request-invalid")?;
    if request.schema_version != 1
        || request.record_type != "qiongli-alpha1-update-manifest-signing-request"
        || request.status != "awaiting-external-release-signature"
        || request.publication_allowed
        || request.authority_sha256 != authority.sha256
        || !is_lower_hex(&request.grant_request_sha256, 64)
        || request.signing_preimage_sha256 != sha256_hex(&preimage)
        || request.signing_preimage_hex != encode_hex(&preimage)
        || request.manifest.stream != NativeUpdateStream::Beta
        || request.manifest.artifact != evidence.summary.artifact
        || request.manifest.source_commit != evidence.summary.source_commit
        || request.manifest.archive_file_name != evidence.summary.archive_file_name
        || request.manifest.archive_size_bytes != evidence.summary.archive_size_bytes
        || request.manifest.archive_sha256 != evidence.summary.archive_sha256
        || request.manifest.desktop_manifest_file_name
            != evidence.summary.desktop_manifest_file_name
        || request.manifest.desktop_manifest_size_bytes
            != evidence.summary.desktop_manifest_size_bytes
        || request.manifest.desktop_manifest_sha256 != evidence.summary.desktop_manifest_sha256
        || request.manifest.signing_receipt_file_name != evidence.summary.signing_receipt_file_name
        || request.manifest.signing_receipt_size_bytes
            != evidence.summary.signing_receipt_size_bytes
        || request.manifest.signing_receipt_sha256 != evidence.summary.signing_receipt_sha256
        || request.manifest.resource_pack_sha256 != evidence.summary.resource_pack_sha256
        || request.manifest.macos_team_id != evidence.summary.macos_team_id
        || request.manifest.client_plugins.len() != 2
        || request
            .manifest
            .client_plugins
            .iter()
            .any(|plugin| plugin.signed_launch_grant.signature.key_id != request.launch_key_id)
    {
        return Err("alpha1-update-manifest-request-invalid");
    }
    Ok(())
}

fn load_authority(
    path: &Path,
    generation: u64,
    release_key_id: &str,
    launch_key_id: &str,
) -> Result<AuthorityEvidence, &'static str> {
    let bytes = read_input_file(path, MAX_SIDECAR_BYTES)?;
    let authority =
        NativeReleaseAuthority::from_json(&bytes).map_err(|_| "alpha1-update-authority-invalid")?;
    authority
        .validate_product_version(VERSION)
        .map_err(|_| "alpha1-update-authority-version-invalid")?;
    if authority.channel() != ReleaseChannel::Alpha
        || generation < authority.minimum_release_generation()
        || generation < authority.minimum_launch_grant_generation()
    {
        return Err("alpha1-update-authority-generation-invalid");
    }
    let release_key = authority
        .release_keys()
        .iter()
        .find(|key| key.key_id() == release_key_id)
        .ok_or("alpha1-update-release-key-unavailable")?;
    if generation < release_key.minimum_generation()
        || release_key
            .maximum_generation_exclusive()
            .is_some_and(|maximum| generation >= maximum)
    {
        return Err("alpha1-update-release-key-generation-invalid");
    }
    if !authority
        .launch_grant_keys()
        .iter()
        .any(|key| key.key_id() == launch_key_id)
    {
        return Err("alpha1-update-launch-key-unavailable");
    }
    Ok(AuthorityEvidence {
        authority,
        sha256: sha256_hex(&bytes),
    })
}

fn load_artifact_evidence(path: &Path) -> Result<ArtifactEvidence, &'static str> {
    validate_input_directory(path)?;
    let desktop_manifest_bytes =
        read_input_file(&path.join(DESKTOP_MANIFEST_FILE), MAX_SIDECAR_BYTES)?;
    let desktop_manifest = parse_canonical_json::<DesktopPackageManifestV1>(
        &desktop_manifest_bytes,
        "desktop-manifest",
    )?;
    let signing_receipt_bytes =
        read_input_file(&path.join(SIGNING_RECEIPT_FILE), MAX_SIDECAR_BYTES)?;
    let receipt = serde_json::from_slice::<MacosUpdateSigningReceiptV1>(&signing_receipt_bytes)
        .map_err(|_| "alpha1-update-signing-receipt-invalid")?;
    let (archive_size_bytes, archive_sha256) =
        sha256_file(&path.join(SIGNED_ARCHIVE_FILE), MAX_ARCHIVE_BYTES)?;
    let desktop_manifest_sha256 = sha256_hex(&desktop_manifest_bytes);
    if desktop_manifest.artifact.product != ProductId::Qiongli
        || desktop_manifest.artifact.version != VERSION
        || desktop_manifest.artifact.channel != ReleaseChannel::Alpha
        || desktop_manifest.artifact.profile != CapabilityProfile::Lite
        || desktop_manifest.artifact.os != OperatingSystem::Macos
        || desktop_manifest.artifact.arch != Architecture::Aarch64
        || desktop_manifest.artifact.installer_kind != InstallerKind::NativeInstaller
        || desktop_manifest.source_artifact.installer_kind != InstallerKind::PortableArchive
        || !is_lower_hex(&desktop_manifest.product_source_commit, 40)
            && !is_lower_hex(&desktop_manifest.product_source_commit, 64)
        || !is_lower_hex(&desktop_manifest.resource_pack_sha256, 64)
        || receipt.schema_version != 1
        || receipt.record_type != "qiongli-macos-update-signing"
        || receipt.status != "signed-notarized-candidate"
        || receipt.publication_allowed
        || receipt.source.product_source_commit != desktop_manifest.product_source_commit
        || receipt.source.unsigned_manifest_sha256 != desktop_manifest_sha256
        || receipt.final_artifact.status != "produced"
        || receipt.final_artifact.file != SIGNED_ARCHIVE_FILE
        || receipt.final_artifact.size_bytes != archive_size_bytes
        || receipt.final_artifact.sha256 != archive_sha256
        || !is_lower_hex(&receipt.final_artifact.launcher_sha256, 64)
        || !is_lower_hex(&receipt.final_artifact.canonical_binary_sha256, 64)
        || !is_lower_hex(&receipt.final_artifact.update_helper_sha256, 64)
        || receipt.signing.kind != "developer-id-application"
        || receipt.signing.verification != "passed"
        || !valid_team_id(&receipt.signing.team_identifier)
        || receipt.notarization.status != "accepted"
        || receipt.notarization.stapling != "passed"
        || receipt.notarization.gatekeeper_assessment != "passed"
    {
        return Err("alpha1-update-artifact-evidence-invalid");
    }
    Ok(ArtifactEvidence {
        summary: ArtifactSetV1 {
            artifact: desktop_manifest.artifact.clone(),
            source_commit: desktop_manifest.product_source_commit.clone(),
            archive_file_name: SIGNED_ARCHIVE_FILE.to_string(),
            archive_size_bytes,
            archive_sha256,
            desktop_manifest_file_name: DESKTOP_MANIFEST_FILE.to_string(),
            desktop_manifest_size_bytes: desktop_manifest_bytes.len() as u64,
            desktop_manifest_sha256,
            signing_receipt_file_name: SIGNING_RECEIPT_FILE.to_string(),
            signing_receipt_size_bytes: signing_receipt_bytes.len() as u64,
            signing_receipt_sha256: sha256_hex(&signing_receipt_bytes),
            resource_pack_sha256: desktop_manifest.resource_pack_sha256,
            signed_canonical_binary_sha256: receipt.final_artifact.canonical_binary_sha256,
            macos_team_id: receipt.signing.team_identifier,
        },
        desktop_manifest_bytes,
        signing_receipt_bytes,
    })
}

fn validate_timestamps(
    published_at_unix: u64,
    not_before_unix: u64,
    expires_at_unix: u64,
) -> Result<(), &'static str> {
    if published_at_unix == 0
        || not_before_unix == 0
        || expires_at_unix == 0
        || published_at_unix >= expires_at_unix
        || not_before_unix >= expires_at_unix
    {
        Err("alpha1-update-timestamps-invalid")
    } else {
        Ok(())
    }
}

fn release_url(file_name: &str) -> String {
    format!("{RELEASE_ROOT}/{file_name}")
}

fn parse_canonical_json<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    label: &'static str,
) -> Result<T, &'static str> {
    let value = serde_json::from_slice::<T>(bytes).map_err(|_| match label {
        "desktop-manifest" => "alpha1-update-desktop-manifest-invalid",
        "grant-request" => "alpha1-update-grant-request-invalid",
        "manifest-request" => "alpha1-update-manifest-request-invalid",
        _ => "alpha1-update-json-invalid",
    })?;
    if canonical_json(&value)?.as_slice() != bytes {
        return Err(match label {
            "desktop-manifest" => "alpha1-update-desktop-manifest-noncanonical",
            "grant-request" => "alpha1-update-grant-request-noncanonical",
            "manifest-request" => "alpha1-update-manifest-request-noncanonical",
            _ => "alpha1-update-json-noncanonical",
        });
    }
    Ok(value)
}

fn read_signature(path: &Path) -> Result<String, &'static str> {
    let bytes = read_input_file(path, 129)?;
    let signature = match bytes.as_slice() {
        [value @ .., b'\n'] if value.len() == 128 => value,
        value if value.len() == 128 => value,
        _ => return Err("alpha1-update-signature-file-invalid"),
    };
    let value =
        std::str::from_utf8(signature).map_err(|_| "alpha1-update-signature-file-invalid")?;
    if !is_lower_hex(value, 128) {
        return Err("alpha1-update-signature-file-invalid");
    }
    Ok(value.to_string())
}

fn validate_input_directory(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute() {
        return Err("alpha1-update-input-directory-invalid");
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "alpha1-update-input-directory-invalid")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("alpha1-update-input-directory-invalid");
    }
    Ok(())
}

fn read_input_file(path: &Path, limit: u64) -> Result<Vec<u8>, &'static str> {
    if !path.is_absolute() {
        return Err("alpha1-update-input-file-invalid");
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| "alpha1-update-input-file-invalid")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit {
        return Err("alpha1-update-input-file-invalid");
    }
    let file = File::open(path).map_err(|_| "alpha1-update-input-file-invalid")?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "alpha1-update-input-file-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("alpha1-update-input-file-invalid");
    }
    Ok(bytes)
}

fn sha256_file(path: &Path, limit: u64) -> Result<(u64, String), &'static str> {
    if !path.is_absolute() {
        return Err("alpha1-update-archive-invalid");
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| "alpha1-update-archive-invalid")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err("alpha1-update-archive-invalid");
    }
    let mut file = File::open(path).map_err(|_| "alpha1-update-archive-invalid")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut size = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "alpha1-update-archive-invalid")?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or("alpha1-update-archive-invalid")?;
        if size > limit {
            return Err("alpha1-update-archive-invalid");
        }
        hasher.update(&buffer[..read]);
    }
    if size != metadata.len() {
        return Err("alpha1-update-archive-invalid");
    }
    Ok((size, encode_hex(&hasher.finalize())))
}

fn create_new_private_directory(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute() || path.exists() {
        return Err("alpha1-update-output-directory-invalid");
    }
    let parent = path
        .parent()
        .ok_or("alpha1-update-output-directory-invalid")?;
    let metadata =
        fs::symlink_metadata(parent).map_err(|_| "alpha1-update-output-directory-invalid")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("alpha1-update-output-directory-invalid");
    }
    fs::create_dir(path).map_err(|_| "alpha1-update-output-directory-invalid")?;
    set_private_directory_permissions(path)?;
    Ok(())
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    if !path.is_absolute() {
        return Err("alpha1-update-output-file-invalid");
    }
    let parent = path.parent().ok_or("alpha1-update-output-file-invalid")?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| "alpha1-update-output-file-invalid")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("alpha1-update-output-file-invalid");
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "alpha1-update-output-file-invalid")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "alpha1-update-output-file-invalid")?;
    set_private_file_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| "alpha1-update-output-directory-invalid")
}

#[cfg(not(unix))]
fn set_private_directory_permissions(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| "alpha1-update-output-file-invalid")
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| "alpha1-update-canonical-serialization-failed")
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

fn is_lower_hex(value: &str, expected_length: usize) -> bool {
    value.len() == expected_length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn valid_team_id(value: &str) -> bool {
    value.len() == 10
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use ed25519_dalek::{Signer as _, SigningKey};
    use qiongli_content::LogicalMode;
    use qiongli_platform::{
        DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION, DesktopApplicationMetadataV1,
        DesktopPackageEntryV1, DesktopPackageKind, DesktopPackageRecordType, DesktopPackageStatus,
    };
    use serde_json::json;

    use super::*;

    const TEAM_ID: &str = "ABC123DEFG";
    const GENERATION: u64 = 9;
    const NOW: u64 = 2_000_000_000;
    static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn external_signing_workflow_verifies_without_publication_authority() {
        let root = test_root();
        fs::create_dir(&root).unwrap();
        let artifact_dir = root.join("artifact");
        fs::create_dir(&artifact_dir).unwrap();
        let release_key = SigningKey::from_bytes(&[31_u8; 32]);
        let launch_key = SigningKey::from_bytes(&[32_u8; 32]);
        let authority_path = root.join("authority.json");
        write_fixture(
            &authority_path,
            &canonical_json(&json!({
                "schema_version": 1,
                "channel": "alpha",
                "minimum_release_generation": GENERATION,
                "minimum_launch_grant_generation": GENERATION,
                "release_keys": [{
                    "key_id": "release-alpha1-test",
                    "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
                    "minimum_generation": GENERATION,
                    "maximum_generation_exclusive": GENERATION + 1
                }],
                "launch_grant_keys": [{
                    "key_id": "launch-alpha1-test",
                    "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
                }]
            }))
            .unwrap(),
        );
        write_artifact_fixture(&artifact_dir);

        let grant_request_path = root.join("grant-request.json");
        prepare_grants(&PrepareGrantsArguments {
            signed_artifact_dir: artifact_dir.clone(),
            authority: authority_path.clone(),
            generation: GENERATION,
            published_at_unix: NOW - 120,
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
            release_key_id: "release-alpha1-test".to_string(),
            launch_key_id: "launch-alpha1-test".to_string(),
            output: grant_request_path.clone(),
        })
        .unwrap();
        let grant_request_bytes = fs::read(&grant_request_path).unwrap();
        let grant_request =
            parse_canonical_json::<GrantSigningRequestV1>(&grant_request_bytes, "grant-request")
                .unwrap();
        let codex_signature =
            launch_key.sign(&launch_grant_signing_bytes(&grant_request.grants[0].grant).unwrap());
        let claude_signature =
            launch_key.sign(&launch_grant_signing_bytes(&grant_request.grants[1].grant).unwrap());
        let codex_signature_path = root.join("codex.sig");
        let claude_signature_path = root.join("claude.sig");
        write_fixture(
            &codex_signature_path,
            encode_hex(&codex_signature.to_bytes()).as_bytes(),
        );
        write_fixture(
            &claude_signature_path,
            encode_hex(&claude_signature.to_bytes()).as_bytes(),
        );

        let manifest_request_path = root.join("manifest-request.json");
        prepare_manifest(&PrepareManifestArguments {
            signed_artifact_dir: artifact_dir.clone(),
            authority: authority_path.clone(),
            grant_request: grant_request_path,
            codex_signature_file: codex_signature_path,
            claude_signature_file: claude_signature_path,
            output: manifest_request_path.clone(),
        })
        .unwrap();
        let manifest_request_bytes = fs::read(&manifest_request_path).unwrap();
        let manifest_request = parse_canonical_json::<ManifestSigningRequestV1>(
            &manifest_request_bytes,
            "manifest-request",
        )
        .unwrap();
        let release_signature = release_key
            .sign(&native_update_manifest_signing_bytes(&manifest_request.manifest).unwrap());
        let release_signature_path = root.join("release.sig");
        write_fixture(
            &release_signature_path,
            encode_hex(&release_signature.to_bytes()).as_bytes(),
        );

        let output_dir = root.join("final");
        finalize(&FinalizeArguments {
            signed_artifact_dir: artifact_dir,
            authority: authority_path,
            manifest_request: manifest_request_path,
            release_signature_file: release_signature_path,
            output_dir: output_dir.clone(),
        })
        .unwrap();
        let metadata = fs::read(output_dir.join(UPDATE_METADATA_FILE)).unwrap();
        SignedNativeUpdateManifestV1::from_json(&metadata).unwrap();
        let receipt = fs::read(output_dir.join(UPDATE_RECEIPT_FILE)).unwrap();
        let receipt: serde_json::Value = serde_json::from_slice(&receipt).unwrap();
        assert_eq!(receipt["publication_allowed"], false);
        assert_eq!(receipt["stable_stream_rejected_prerelease"], true);
        assert_eq!(receipt["status"], "signed-verified-nonpublishing");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn signature_input_rejects_whitespace_and_uppercase() {
        let root = test_root();
        fs::create_dir(&root).unwrap();
        let valid = root.join("valid.sig");
        write_fixture(&valid, format!("{}\n", "a".repeat(128)).as_bytes());
        assert_eq!(read_signature(&valid).unwrap(), "a".repeat(128));
        let invalid = root.join("invalid.sig");
        write_fixture(&invalid, format!(" {}", "A".repeat(128)).as_bytes());
        assert_eq!(
            read_signature(&invalid).unwrap_err(),
            "alpha1-update-signature-file-invalid"
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn write_artifact_fixture(root: &Path) {
        let archive = b"signed-notarized-archive-fixture";
        write_fixture(&root.join(SIGNED_ARCHIVE_FILE), archive);
        let artifact = ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: VERSION.to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Macos,
            arch: Architecture::Aarch64,
            installer_kind: InstallerKind::NativeInstaller,
        };
        let mut source_artifact = artifact.clone();
        source_artifact.installer_kind = InstallerKind::PortableArchive;
        let entries = [
            ("Qiongli.app/Contents/Info.plist", LogicalMode::Regular),
            (
                "Qiongli.app/Contents/MacOS/Qiongli",
                LogicalMode::Executable,
            ),
            (
                "Qiongli.app/Contents/MacOS/qiongli-cli",
                LogicalMode::Executable,
            ),
            (
                "Qiongli.app/Contents/MacOS/qiongli-update-helper",
                LogicalMode::Executable,
            ),
            (
                "Qiongli.app/Contents/Resources/LICENSE",
                LogicalMode::Regular,
            ),
            (
                "Qiongli.app/Contents/Resources/Qiongli.icns",
                LogicalMode::Regular,
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (path, mode))| DesktopPackageEntryV1 {
            path: path.to_string(),
            mode,
            size_bytes: index as u64 + 1,
            sha256: format!("{:064x}", index + 10),
        })
        .collect::<Vec<_>>();
        let manifest = DesktopPackageManifestV1 {
            schema_version: DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION,
            record_type: DesktopPackageRecordType::QiongliDesktopPackage,
            status: DesktopPackageStatus::AssembledUnpublished,
            package_kind: DesktopPackageKind::MacosApplicationZip,
            artifact,
            source_artifact,
            product_source_commit: "a".repeat(40),
            source_artifact_manifest_sha256: "1".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            canonical_binary_sha256: entries[2].sha256.clone(),
            launcher_sha256: entries[1].sha256.clone(),
            update_helper_sha256: entries[3].sha256.clone(),
            application: DesktopApplicationMetadataV1::new(
                "Qiongli",
                "Qiongli 2",
                "io.github.jxpeng98.qiongli",
                VERSION,
                "MIT",
            ),
            package_root: "Qiongli.app".to_string(),
            manifest_path: "Qiongli.app/Contents/Resources/.qiongli-desktop-package.json"
                .to_string(),
            entry_content_root_sha256: entry_content_root(&entries),
            entries,
        };
        let manifest_bytes = canonical_json(&manifest).unwrap();
        write_fixture(&root.join(DESKTOP_MANIFEST_FILE), &manifest_bytes);
        let receipt = json!({
            "schema_version": 1,
            "record_type": "qiongli-macos-update-signing",
            "status": "signed-notarized-candidate",
            "publication_allowed": false,
            "source": {
                "product_source_commit": "a".repeat(40),
                "unsigned_manifest_sha256": sha256_hex(&manifest_bytes)
            },
            "final_artifact": {
                "status": "produced",
                "file": SIGNED_ARCHIVE_FILE,
                "size_bytes": archive.len(),
                "sha256": sha256_hex(archive),
                "launcher_sha256": "6".repeat(64),
                "canonical_binary_sha256": "7".repeat(64),
                "update_helper_sha256": "8".repeat(64)
            },
            "signing": {
                "kind": "developer-id-application",
                "verification": "passed",
                "team_identifier": TEAM_ID
            },
            "notarization": {
                "status": "accepted",
                "stapling": "passed",
                "gatekeeper_assessment": "passed"
            }
        });
        write_fixture(
            &root.join(SIGNING_RECEIPT_FILE),
            &serde_json::to_vec(&receipt).unwrap(),
        );
    }

    fn entry_content_root(entries: &[DesktopPackageEntryV1]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"qiongli-desktop-package-content-root-v1\0");
        for entry in entries {
            hasher.update((entry.path.len() as u64).to_be_bytes());
            hasher.update(entry.path.as_bytes());
            hasher.update([match entry.mode {
                LogicalMode::Regular => 0,
                LogicalMode::Executable => 1,
            }]);
            hasher.update(entry.size_bytes.to_be_bytes());
            hasher.update(entry.sha256.as_bytes());
        }
        encode_hex(&hasher.finalize())
    }

    fn write_fixture(path: &Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
    }

    fn test_root() -> PathBuf {
        env::temp_dir().join(format!(
            "qiongli-alpha1-update-metadata-{}-{}",
            std::process::id(),
            NEXT_TEST.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
