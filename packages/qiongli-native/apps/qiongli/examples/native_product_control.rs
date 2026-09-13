#![allow(clippy::disallowed_methods)]

use std::env;
use std::ffi::OsString;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

use qiongli_platform::{
    CapabilityProfile, ClientActivationTarget, GrantSignatureV1, GrantVerificationContext,
    InstallerKind, LaunchGrantV1, NativeClientPluginGrantV1, NativeReleaseAuthority,
    PackagedProductActivationExpectation, PackagedProductControlV1, PackagedProductDesiredStateV1,
    PackagedProductPluginIdentity, PackagedProductRecordType, PackagedProductSkillsScope,
    SignatureAlgorithm, SignedLaunchGrantV1, attach_product_control_to_desktop_manifest,
    launch_grant_signing_bytes, parse_desktop_package_manifest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_INPUT_BYTES: u64 = 256 * 1024;
const MAX_BINARY_BYTES: u64 = 128 * 1024 * 1024;
const MAX_SIGNATURE_BYTES: u64 = 512;

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    match arguments {
        Arguments::Prepare(arguments) => prepare(&arguments),
        Arguments::Finalize(arguments) => finalize(&arguments),
    }
}

fn prepare(arguments: &PrepareArguments) -> Result<(), &'static str> {
    let authority_bytes = read_bounded(&arguments.authority, MAX_INPUT_BYTES)?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "product-control-authority-invalid")?;
    let manifest_bytes = read_bounded(&arguments.desktop_manifest, MAX_INPUT_BYTES)?;
    let manifest = parse_desktop_package_manifest(&manifest_bytes)
        .map_err(|_| "product-control-desktop-manifest-invalid")?;
    authority
        .validate_product_version(&manifest.artifact.version)
        .map_err(|_| "product-control-version-invalid")?;
    if manifest.artifact.profile != CapabilityProfile::Lite
        || manifest.artifact.installer_kind != InstallerKind::NativeInstaller
        || manifest.artifact.channel != authority.channel()
        || arguments.generation < authority.minimum_launch_grant_generation()
        || arguments.not_before_unix >= arguments.expires_at_unix
    {
        return Err("product-control-identity-invalid");
    }
    let binary = read_bounded(&arguments.canonical, MAX_BINARY_BYTES)?;
    let binary_sha256 = sha256_hex(&binary);
    let mut plugin_artifact = manifest.artifact.clone();
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
            binary_sha256: binary_sha256.clone(),
            resource_pack_sha256: manifest.resource_pack_sha256.clone(),
            allowed_modes: target.allowed_grant_modes().to_vec(),
            integration_scopes: vec![target.integration_scope()],
            not_before_unix: arguments.not_before_unix,
            expires_at_unix: arguments.expires_at_unix,
        };
        let preimage =
            launch_grant_signing_bytes(&grant).map_err(|_| "product-control-grant-invalid")?;
        Ok(ProductControlGrantSigningRequestV1 {
            target,
            grant,
            signing_preimage_sha256: sha256_hex(&preimage),
            signing_preimage_hex: encode_hex(&preimage),
        })
    })
    .collect::<Result<Vec<_>, &'static str>>()?;
    let request = ProductControlSigningRequestV1 {
        schema_version: 1,
        record_type: "qiongli-product-control-signing-request".to_string(),
        status: "awaiting-external-launch-grant-signatures".to_string(),
        publication_allowed: false,
        authority_sha256: sha256_hex(&authority_bytes),
        artifact: manifest.artifact,
        product_source_commit: manifest.product_source_commit,
        canonical_binary_sha256: binary_sha256,
        resource_pack_sha256: manifest.resource_pack_sha256,
        generation: arguments.generation,
        not_before_unix: arguments.not_before_unix,
        expires_at_unix: arguments.expires_at_unix,
        grants,
    };
    write_new_private(&arguments.output, &canonical_json(&request)?)
}

fn finalize(arguments: &FinalizeArguments) -> Result<(), &'static str> {
    let authority_bytes = read_bounded(&arguments.authority, MAX_INPUT_BYTES)?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "product-control-authority-invalid")?;
    let request_bytes = read_bounded(&arguments.request, MAX_INPUT_BYTES)?;
    let request: ProductControlSigningRequestV1 = parse_canonical(&request_bytes)?;
    validate_request(&request, &authority, &authority_bytes)?;
    let manifest_bytes = read_bounded(&arguments.desktop_manifest, MAX_INPUT_BYTES)?;
    let manifest = parse_desktop_package_manifest(&manifest_bytes)
        .map_err(|_| "product-control-desktop-manifest-invalid")?;
    if request.artifact != manifest.artifact
        || request.product_source_commit != manifest.product_source_commit
        || request.resource_pack_sha256 != manifest.resource_pack_sha256
    {
        return Err("product-control-identity-invalid");
    }
    let signatures = [
        read_signature(&arguments.codex_signature)?,
        read_signature(&arguments.claude_signature)?,
    ];
    let plugins = request
        .grants
        .iter()
        .zip(signatures)
        .map(|(item, signature)| {
            let signed = SignedLaunchGrantV1 {
                grant: item.grant.clone(),
                signature: GrantSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: arguments.launch_key_id.clone(),
                    value_hex: signature,
                },
            };
            let context = GrantVerificationContext {
                now_unix: request.not_before_unix,
                minimum_generation: authority.minimum_launch_grant_generation(),
                expected_artifact: &item.grant.artifact,
                binary_sha256: &request.canonical_binary_sha256,
                resource_pack_sha256: &request.resource_pack_sha256,
                requested_mode: item.target.required_grant_mode(),
                requested_scope: item.target.integration_scope(),
            };
            signed
                .verify(authority.launch_grant_keys(), &context)
                .map_err(|_| "product-control-signature-invalid")?;
            Ok(NativeClientPluginGrantV1 {
                target: item.target,
                signed_launch_grant: signed,
            })
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    let control = PackagedProductControlV1 {
        schema_version: qiongli_platform::PACKAGED_PRODUCT_CONTROL_SCHEMA_VERSION,
        record_type: PackagedProductRecordType::QiongliPackagedProductControl,
        artifact: request.artifact,
        product_source_commit: request.product_source_commit,
        canonical_binary_sha256: request.canonical_binary_sha256,
        resource_pack_sha256: request.resource_pack_sha256,
        desired_state: PackagedProductDesiredStateV1 {
            profile: CapabilityProfile::Lite,
            target_clients: vec![
                ClientActivationTarget::Codex,
                ClientActivationTarget::ClaudeCode,
            ],
            skills_scope: PackagedProductSkillsScope::MarketplaceLite,
            plugin_identity: PackagedProductPluginIdentity::QiongliNext,
            lite_mcp: true,
            full_mcp_targets: vec![
                ClientActivationTarget::Codex,
                ClientActivationTarget::ClaudeCode,
            ],
            activation: PackagedProductActivationExpectation::RegisterThenClientEnablement,
        },
        client_plugins: plugins,
    };
    let control_bytes = control
        .to_canonical_json()
        .map_err(|_| "product-control-output-invalid")?;
    let updated_manifest =
        attach_product_control_to_desktop_manifest(&manifest_bytes, &control_bytes)
            .map_err(|_| "product-control-desktop-manifest-update-failed")?;
    write_new_private(&arguments.control_output, &control_bytes)?;
    if let Err(error) = write_new_private(&arguments.manifest_output, &updated_manifest) {
        let _ = fs::remove_file(&arguments.control_output);
        return Err(error);
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductControlGrantSigningRequestV1 {
    target: ClientActivationTarget,
    grant: LaunchGrantV1,
    signing_preimage_sha256: String,
    signing_preimage_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductControlSigningRequestV1 {
    schema_version: u32,
    record_type: String,
    status: String,
    publication_allowed: bool,
    authority_sha256: String,
    artifact: qiongli_platform::ArtifactIdentityV1,
    product_source_commit: String,
    canonical_binary_sha256: String,
    resource_pack_sha256: String,
    generation: u64,
    not_before_unix: u64,
    expires_at_unix: u64,
    grants: Vec<ProductControlGrantSigningRequestV1>,
}

fn validate_request(
    request: &ProductControlSigningRequestV1,
    authority: &NativeReleaseAuthority,
    authority_bytes: &[u8],
) -> Result<(), &'static str> {
    if request.schema_version != 1
        || request.record_type != "qiongli-product-control-signing-request"
        || request.status != "awaiting-external-launch-grant-signatures"
        || request.publication_allowed
        || request.authority_sha256 != sha256_hex(authority_bytes)
        || request.artifact.channel != authority.channel()
        || request.generation < authority.minimum_launch_grant_generation()
        || request.not_before_unix >= request.expires_at_unix
        || request.grants.len() != 2
        || request.grants[0].target != ClientActivationTarget::Codex
        || request.grants[1].target != ClientActivationTarget::ClaudeCode
    {
        return Err("product-control-request-invalid");
    }
    for item in &request.grants {
        let preimage = launch_grant_signing_bytes(&item.grant)
            .map_err(|_| "product-control-request-invalid")?;
        if item.grant.generation != request.generation
            || item.grant.binary_sha256 != request.canonical_binary_sha256
            || item.grant.resource_pack_sha256 != request.resource_pack_sha256
            || item.grant.integration_scopes.as_slice() != [item.target.integration_scope()]
            || item.signing_preimage_sha256 != sha256_hex(&preimage)
            || item.signing_preimage_hex != encode_hex(&preimage)
        {
            return Err("product-control-request-invalid");
        }
    }
    Ok(())
}

enum Arguments {
    Prepare(PrepareArguments),
    Finalize(FinalizeArguments),
}

struct PrepareArguments {
    desktop_manifest: PathBuf,
    canonical: PathBuf,
    authority: PathBuf,
    generation: u64,
    not_before_unix: u64,
    expires_at_unix: u64,
    output: PathBuf,
}

struct FinalizeArguments {
    request: PathBuf,
    desktop_manifest: PathBuf,
    authority: PathBuf,
    launch_key_id: String,
    codex_signature: PathBuf,
    claude_signature: PathBuf,
    control_output: PathBuf,
    manifest_output: PathBuf,
}

impl Arguments {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        let command = values
            .first()
            .and_then(|value| value.to_str())
            .ok_or("product-control-usage-invalid")?;
        let options = parse_options(&values[1..])?;
        match command {
            "prepare" => Ok(Self::Prepare(PrepareArguments {
                desktop_manifest: required_path(&options, "--desktop-manifest")?,
                canonical: required_path(&options, "--canonical")?,
                authority: required_path(&options, "--authority")?,
                generation: required_u64(&options, "--generation")?,
                not_before_unix: required_u64(&options, "--not-before-unix")?,
                expires_at_unix: required_u64(&options, "--expires-at-unix")?,
                output: required_path(&options, "--output")?,
            })),
            "finalize" => Ok(Self::Finalize(FinalizeArguments {
                request: required_path(&options, "--request")?,
                desktop_manifest: required_path(&options, "--desktop-manifest")?,
                authority: required_path(&options, "--authority")?,
                launch_key_id: required_text(&options, "--launch-key-id")?,
                codex_signature: required_path(&options, "--codex-signature")?,
                claude_signature: required_path(&options, "--claude-signature")?,
                control_output: required_path(&options, "--control-output")?,
                manifest_output: required_path(&options, "--manifest-output")?,
            })),
            _ => Err("product-control-usage-invalid"),
        }
    }
}

fn parse_options(values: &[OsString]) -> Result<Vec<(String, OsString)>, &'static str> {
    if values.is_empty() || !values.len().is_multiple_of(2) {
        return Err("product-control-usage-invalid");
    }
    let mut output = Vec::new();
    for pair in values.chunks_exact(2) {
        let key = pair[0]
            .to_str()
            .filter(|value| value.starts_with("--"))
            .ok_or("product-control-usage-invalid")?
            .to_string();
        if output.iter().any(|(existing, _)| existing == &key) {
            return Err("product-control-usage-invalid");
        }
        output.push((key, pair[1].clone()));
    }
    Ok(output)
}

fn required_path(options: &[(String, OsString)], key: &str) -> Result<PathBuf, &'static str> {
    let path = PathBuf::from(required_value(options, key)?);
    if !path.is_absolute() {
        return Err("product-control-usage-invalid");
    }
    Ok(path)
}

fn required_text(options: &[(String, OsString)], key: &str) -> Result<String, &'static str> {
    required_value(options, key)?
        .to_str()
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 64
                && value.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_')
                })
        })
        .map(ToOwned::to_owned)
        .ok_or("product-control-usage-invalid")
}

fn required_u64(options: &[(String, OsString)], key: &str) -> Result<u64, &'static str> {
    required_value(options, key)?
        .to_str()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .ok_or("product-control-usage-invalid")
}

fn required_value<'a>(
    options: &'a [(String, OsString)],
    key: &str,
) -> Result<&'a OsString, &'static str> {
    options
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value)
        .ok_or("product-control-usage-invalid")
}

fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "product-control-input-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err("product-control-input-invalid");
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "product-control-input-invalid")?
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "product-control-input-invalid")?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > maximum {
        return Err("product-control-input-invalid");
    }
    Ok(bytes)
}

fn read_signature(path: &Path) -> Result<String, &'static str> {
    let bytes = read_bounded(path, MAX_SIGNATURE_BYTES)?;
    let value = std::str::from_utf8(&bytes)
        .map_err(|_| "product-control-signature-invalid")?
        .trim_end_matches(['\r', '\n']);
    if value.len() != 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("product-control-signature-invalid");
    }
    Ok(value.to_string())
}

fn write_new_private(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    if !path.is_absolute() || path.parent().is_none() || path.exists() {
        return Err("product-control-output-invalid");
    }
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .map_err(|_| "product-control-output-invalid")?
    };
    #[cfg(windows)]
    let mut file = qiongli_windows_security::create_owner_only_new_file(path)
        .map_err(|_| "product-control-output-invalid")?;
    #[cfg(not(any(unix, windows)))]
    return Err("product-control-platform-unsupported");
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "product-control-output-invalid")
}

fn parse_canonical<T: for<'de> Deserialize<'de> + Serialize>(
    bytes: &[u8],
) -> Result<T, &'static str> {
    let value = serde_json::from_slice(bytes).map_err(|_| "product-control-request-invalid")?;
    if canonical_json(&value)? != bytes {
        return Err("product-control-request-invalid");
    }
    Ok(value)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| "product-control-json-invalid")
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
