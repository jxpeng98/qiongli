use std::fmt::{self, Display, Formatter};

use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::grant::{decode_fixed_hex, is_lower_hex, sha256_hex, valid_identifier};
use crate::native_release::validate_release_keys;
use crate::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, NativeReleaseSignatureV1,
    OperatingSystem, ProductId, ReleaseChannel, SignatureAlgorithm, TrustedReleasePublicKey,
};

pub const NATIVE_UPDATE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const MAX_NATIVE_UPDATE_MANIFEST_BYTES: usize = 128 * 1024;

const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_KEY_ID_BYTES: usize = 64;
const MAX_ALLOWED_DOWNLOAD_HOSTS: usize = 8;
const MAX_DOWNLOAD_HOST_BYTES: usize = 253;
const MAX_URL_BYTES: usize = 2_048;
const ED25519_SIGNATURE_BYTES: usize = 64;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const SIGNING_DOMAIN: &[u8] = b"QIONGLI-NATIVE-UPDATE-MANIFEST-V1\0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeUpdateStream {
    Stable,
    Beta,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeUpdateManifestV1 {
    pub schema_version: u32,
    pub stream: NativeUpdateStream,
    pub generation: u64,
    pub artifact: ArtifactIdentityV1,
    pub source_commit: String,
    pub minimum_updater_version: String,
    pub archive_file_name: String,
    pub archive_url: String,
    pub archive_size_bytes: u64,
    pub archive_sha256: String,
    pub desktop_manifest_sha256: String,
    pub signing_receipt_sha256: String,
    pub resource_pack_sha256: String,
    pub macos_team_id: String,
    pub published_at_unix: u64,
    pub not_before_unix: u64,
    pub expires_at_unix: u64,
}

impl NativeUpdateManifestV1 {
    fn validate_structure(&self) -> Result<(), NativeUpdateError> {
        if self.schema_version != NATIVE_UPDATE_MANIFEST_SCHEMA_VERSION {
            return Err(NativeUpdateError::UnsupportedSchema);
        }
        self.artifact
            .validate_lite()
            .map_err(|_| NativeUpdateError::InvalidManifest)?;
        let expected_file_name = signed_macos_archive_file_name(&self.artifact.version);
        let url = parse_archive_url(&self.archive_url)?;
        if self.generation == 0
            || self.generation > JCS_MAX_SAFE_INTEGER
            || !valid_source_commit(&self.source_commit)
            || parse_product_version(&self.minimum_updater_version).is_none()
            || self.archive_file_name != expected_file_name
            || url.path_segments().and_then(Iterator::last) != Some(self.archive_file_name.as_str())
            || self.archive_size_bytes == 0
            || self.archive_size_bytes > MAX_ARCHIVE_BYTES
            || !is_lower_hex(&self.archive_sha256, 64)
            || !is_lower_hex(&self.desktop_manifest_sha256, 64)
            || !is_lower_hex(&self.signing_receipt_sha256, 64)
            || !is_lower_hex(&self.resource_pack_sha256, 64)
            || !valid_team_id(&self.macos_team_id)
            || !valid_timestamp(self.published_at_unix)
            || !valid_timestamp(self.not_before_unix)
            || !valid_timestamp(self.expires_at_unix)
            || self.not_before_unix >= self.expires_at_unix
            || self.published_at_unix >= self.expires_at_unix
        {
            return Err(NativeUpdateError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedNativeUpdateManifestV1 {
    pub manifest: NativeUpdateManifestV1,
    pub signature: NativeReleaseSignatureV1,
}

impl SignedNativeUpdateManifestV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, NativeUpdateError> {
        if input.len() > MAX_NATIVE_UPDATE_MANIFEST_BYTES {
            return Err(NativeUpdateError::InputTooLarge);
        }
        let signed =
            serde_json::from_slice::<Self>(input).map_err(|_| NativeUpdateError::InvalidJson)?;
        signed.validate_structure()?;
        if signed.to_canonical_json()? != input {
            return Err(NativeUpdateError::NonCanonicalJson);
        }
        Ok(signed)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeUpdateError> {
        self.validate_structure()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| NativeUpdateError::CanonicalSerializationFailed)?;
        if bytes.len() > MAX_NATIVE_UPDATE_MANIFEST_BYTES {
            return Err(NativeUpdateError::InputTooLarge);
        }
        Ok(bytes)
    }

    pub fn verify(
        &self,
        release_keys: &[TrustedReleasePublicKey],
        context: &NativeUpdateVerificationContext<'_>,
    ) -> Result<VerifiedNativeUpdateManifest, NativeUpdateError> {
        self.validate_structure()?;
        validate_context(context)?;
        validate_release_keys(release_keys).map_err(|_| NativeUpdateError::InvalidReleaseKeySet)?;
        let key = release_keys
            .iter()
            .find(|key| key.key_id() == self.signature.key_id)
            .ok_or(NativeUpdateError::ReleaseKeyUntrusted)?;
        if !key.authorizes_generation(self.manifest.generation) {
            return Err(NativeUpdateError::ReleaseKeyGenerationUnavailable);
        }
        let signature_bytes =
            decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(&self.signature.value_hex)
                .ok_or(NativeUpdateError::InvalidManifest)?;
        let signing_bytes = native_update_manifest_signing_bytes(&self.manifest)?;
        if !key.verifies_signature(&signing_bytes, &signature_bytes) {
            return Err(NativeUpdateError::SignatureInvalid);
        }

        if context.now_unix < self.manifest.not_before_unix {
            return Err(NativeUpdateError::NotYetValid);
        }
        if context.now_unix >= self.manifest.expires_at_unix {
            return Err(NativeUpdateError::Expired);
        }
        if self.manifest.generation <= context.last_accepted_generation {
            return Err(NativeUpdateError::GenerationReplayed);
        }
        if self.manifest.stream != context.selected_stream {
            return Err(NativeUpdateError::StreamMismatch);
        }
        if !stream_accepts_channel(self.manifest.stream, self.manifest.artifact.channel) {
            return Err(NativeUpdateError::ChannelUnavailable);
        }
        if self.manifest.artifact.product != ProductId::Qiongli
            || self.manifest.artifact.profile != CapabilityProfile::Lite
            || self.manifest.artifact.os != OperatingSystem::Macos
            || self.manifest.artifact.arch != Architecture::Aarch64
            || self.manifest.artifact.installer_kind != InstallerKind::NativeInstaller
        {
            return Err(NativeUpdateError::TargetMismatch);
        }

        let current = parse_product_version(context.current_version)
            .ok_or(NativeUpdateError::CurrentVersionInvalid)?;
        if current.major < 2 {
            return Err(NativeUpdateError::LegacyCurrentVersion);
        }
        let target = parse_product_version(&self.manifest.artifact.version)
            .ok_or(NativeUpdateError::InvalidManifest)?;
        if target.major < 2 {
            return Err(NativeUpdateError::LegacyTargetVersion);
        }
        if target <= current {
            return Err(NativeUpdateError::VersionNotNewer);
        }
        let minimum_updater = parse_product_version(&self.manifest.minimum_updater_version)
            .ok_or(NativeUpdateError::InvalidManifest)?;
        if minimum_updater.major < 2 {
            return Err(NativeUpdateError::LegacyMinimumUpdaterVersion);
        }
        if current < minimum_updater {
            return Err(NativeUpdateError::UpdaterIncompatible);
        }
        if self.manifest.macos_team_id != context.expected_macos_team_id {
            return Err(NativeUpdateError::TeamIdMismatch);
        }
        let archive_url = parse_archive_url(&self.manifest.archive_url)?;
        let host = archive_url
            .host_str()
            .ok_or(NativeUpdateError::InvalidManifest)?;
        if !context
            .allowed_download_hosts
            .iter()
            .any(|allowed| host.eq_ignore_ascii_case(allowed))
        {
            return Err(NativeUpdateError::DownloadHostUntrusted);
        }

        Ok(VerifiedNativeUpdateManifest {
            signed: self.clone(),
            signed_payload_sha256: sha256_hex(&signing_bytes),
            release_key_id: key.key_id().to_string(),
            verified_at_unix: context.now_unix,
        })
    }

    fn validate_structure(&self) -> Result<(), NativeUpdateError> {
        self.manifest.validate_structure()?;
        if self.signature.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.signature.key_id, MAX_KEY_ID_BYTES)
            || !is_lower_hex(
                &self.signature.value_hex,
                ED25519_SIGNATURE_BYTES.saturating_mul(2),
            )
        {
            return Err(NativeUpdateError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NativeUpdateVerificationContext<'a> {
    pub now_unix: u64,
    pub last_accepted_generation: u64,
    pub current_version: &'a str,
    pub selected_stream: NativeUpdateStream,
    pub expected_macos_team_id: &'a str,
    pub allowed_download_hosts: &'a [&'a str],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedNativeUpdateManifest {
    signed: SignedNativeUpdateManifestV1,
    signed_payload_sha256: String,
    release_key_id: String,
    verified_at_unix: u64,
}

impl VerifiedNativeUpdateManifest {
    #[must_use]
    pub const fn manifest(&self) -> &NativeUpdateManifestV1 {
        &self.signed.manifest
    }

    #[must_use]
    pub const fn signed_manifest(&self) -> &SignedNativeUpdateManifestV1 {
        &self.signed
    }

    #[must_use]
    pub fn signed_payload_sha256(&self) -> &str {
        &self.signed_payload_sha256
    }

    #[must_use]
    pub fn release_key_id(&self) -> &str {
        &self.release_key_id
    }

    #[must_use]
    pub const fn verified_at_unix(&self) -> u64 {
        self.verified_at_unix
    }
}

pub fn native_update_manifest_signing_bytes(
    manifest: &NativeUpdateManifestV1,
) -> Result<Vec<u8>, NativeUpdateError> {
    manifest.validate_structure()?;
    let canonical = serde_json_canonicalizer::to_vec(manifest)
        .map_err(|_| NativeUpdateError::CanonicalSerializationFailed)?;
    let mut output = Vec::with_capacity(SIGNING_DOMAIN.len().saturating_add(canonical.len()));
    output.extend_from_slice(SIGNING_DOMAIN);
    output.extend_from_slice(&canonical);
    Ok(output)
}

fn validate_context(
    context: &NativeUpdateVerificationContext<'_>,
) -> Result<(), NativeUpdateError> {
    if !valid_timestamp(context.now_unix)
        || context.last_accepted_generation > JCS_MAX_SAFE_INTEGER
        || !valid_team_id(context.expected_macos_team_id)
        || context.allowed_download_hosts.is_empty()
        || context.allowed_download_hosts.len() > MAX_ALLOWED_DOWNLOAD_HOSTS
        || context
            .allowed_download_hosts
            .iter()
            .any(|host| !valid_download_host(host))
        || context
            .allowed_download_hosts
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(NativeUpdateError::InvalidVerificationContext);
    }
    Ok(())
}

fn parse_archive_url(value: &str) -> Result<Url, NativeUpdateError> {
    if value.is_empty() || value.len() > MAX_URL_BYTES || !value.is_ascii() {
        return Err(NativeUpdateError::InvalidManifest);
    }
    let url = Url::parse(value).map_err(|_| NativeUpdateError::InvalidManifest)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(NativeUpdateError::InvalidManifest);
    }
    Ok(url)
}

fn parse_product_version(value: &str) -> Option<Version> {
    if value.is_empty() || value.len() > 64 || !value.is_ascii() {
        return None;
    }
    let version = Version::parse(value).ok()?;
    if !version.build.is_empty() {
        return None;
    }
    let prerelease = version.pre.as_str();
    if prerelease.is_empty()
        || valid_numbered_prerelease(prerelease, "alpha")
        || valid_numbered_prerelease(prerelease, "beta")
    {
        Some(version)
    } else {
        None
    }
}

fn valid_numbered_prerelease(value: &str, channel: &str) -> bool {
    let Some(sequence) = value
        .strip_prefix(channel)
        .and_then(|value| value.strip_prefix('.'))
    else {
        return false;
    };
    !sequence.is_empty()
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.parse::<u64>().is_ok_and(|number| number > 0)
        && !sequence.starts_with('0')
}

const fn stream_accepts_channel(stream: NativeUpdateStream, channel: ReleaseChannel) -> bool {
    match stream {
        NativeUpdateStream::Stable => matches!(channel, ReleaseChannel::Stable),
        NativeUpdateStream::Beta => true,
    }
}

fn signed_macos_archive_file_name(version: &str) -> String {
    format!("qiongli-desktop-{version}-macos-aarch64.signed-notarized.app.zip")
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn valid_team_id(value: &str) -> bool {
    value.len() == 10
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_download_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_DOWNLOAD_HOST_BYTES
        && value.is_ascii()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
}

const fn valid_timestamp(value: u64) -> bool {
    value > 0 && value <= JCS_MAX_SAFE_INTEGER
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeUpdateError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    UnsupportedSchema,
    InvalidManifest,
    InvalidVerificationContext,
    InvalidReleaseKeySet,
    ReleaseKeyUntrusted,
    ReleaseKeyGenerationUnavailable,
    SignatureInvalid,
    NotYetValid,
    Expired,
    GenerationReplayed,
    StreamMismatch,
    ChannelUnavailable,
    TargetMismatch,
    CurrentVersionInvalid,
    LegacyCurrentVersion,
    LegacyTargetVersion,
    LegacyMinimumUpdaterVersion,
    VersionNotNewer,
    UpdaterIncompatible,
    TeamIdMismatch,
    DownloadHostUntrusted,
    CanonicalSerializationFailed,
}

impl NativeUpdateError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "native-update-manifest-too-large",
            Self::InvalidJson => "native-update-manifest-json-invalid",
            Self::NonCanonicalJson => "native-update-manifest-json-noncanonical",
            Self::UnsupportedSchema => "native-update-manifest-schema-unsupported",
            Self::InvalidManifest => "native-update-manifest-invalid",
            Self::InvalidVerificationContext => "native-update-verification-context-invalid",
            Self::InvalidReleaseKeySet => "native-update-release-key-set-invalid",
            Self::ReleaseKeyUntrusted => "native-update-release-key-untrusted",
            Self::ReleaseKeyGenerationUnavailable => {
                "native-update-release-key-generation-unavailable"
            }
            Self::SignatureInvalid => "native-update-signature-invalid",
            Self::NotYetValid => "native-update-not-yet-valid",
            Self::Expired => "native-update-expired",
            Self::GenerationReplayed => "native-update-generation-stale",
            Self::StreamMismatch => "native-update-stream-mismatch",
            Self::ChannelUnavailable => "native-update-channel-unavailable",
            Self::TargetMismatch => "native-update-target-mismatch",
            Self::CurrentVersionInvalid => "native-update-current-version-invalid",
            Self::LegacyCurrentVersion => "native-update-current-version-legacy",
            Self::LegacyTargetVersion => "native-update-target-version-legacy",
            Self::LegacyMinimumUpdaterVersion => "native-update-minimum-version-legacy",
            Self::VersionNotNewer => "native-update-version-not-newer",
            Self::UpdaterIncompatible => "native-update-updater-incompatible",
            Self::TeamIdMismatch => "native-update-team-id-mismatch",
            Self::DownloadHostUntrusted => "native-update-download-host-untrusted",
            Self::CanonicalSerializationFailed => "native-update-canonicalization-failed",
        }
    }
}

impl Display for NativeUpdateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeUpdateError {}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::Value;

    use super::*;

    const NOW: u64 = 1_750_000_000;
    const TEAM_ID: &str = "ABC123DEFG";
    const HOSTS: &[&str] = &["github.com", "objects.githubusercontent.com"];

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    fn artifact(version: &str, channel: ReleaseChannel) -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: version.to_string(),
            channel,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Macos,
            arch: Architecture::Aarch64,
            installer_kind: InstallerKind::NativeInstaller,
        }
    }

    fn manifest(
        version: &str,
        channel: ReleaseChannel,
        stream: NativeUpdateStream,
    ) -> NativeUpdateManifestV1 {
        let file_name = signed_macos_archive_file_name(version);
        NativeUpdateManifestV1 {
            schema_version: NATIVE_UPDATE_MANIFEST_SCHEMA_VERSION,
            stream,
            generation: 9,
            artifact: artifact(version, channel),
            source_commit: "a".repeat(40),
            minimum_updater_version: "2.0.0-alpha.1".to_string(),
            archive_url: format!(
                "https://github.com/jxpeng98/qiongli/releases/download/v{version}/{file_name}"
            ),
            archive_file_name: file_name,
            archive_size_bytes: 42_000_000,
            archive_sha256: "1".repeat(64),
            desktop_manifest_sha256: "2".repeat(64),
            signing_receipt_sha256: "3".repeat(64),
            resource_pack_sha256: "4".repeat(64),
            macos_team_id: TEAM_ID.to_string(),
            published_at_unix: NOW - 120,
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        }
    }

    fn sign(manifest: NativeUpdateManifestV1) -> SignedNativeUpdateManifestV1 {
        let signing_key = SigningKey::from_bytes(&[71_u8; 32]);
        let signature = signing_key.sign(&native_update_manifest_signing_bytes(&manifest).unwrap());
        SignedNativeUpdateManifestV1 {
            manifest,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "release-test-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        }
    }

    fn trusted_key() -> TrustedReleasePublicKey {
        let signing_key = SigningKey::from_bytes(&[71_u8; 32]);
        TrustedReleasePublicKey::new(
            "release-test-key",
            signing_key.verifying_key().to_bytes(),
            1,
            None,
        )
        .unwrap()
    }

    fn context(
        current_version: &str,
        selected_stream: NativeUpdateStream,
    ) -> NativeUpdateVerificationContext<'_> {
        NativeUpdateVerificationContext {
            now_unix: NOW,
            last_accepted_generation: 8,
            current_version,
            selected_stream,
            expected_macos_team_id: TEAM_ID,
            allowed_download_hosts: HOSTS,
        }
    }

    #[test]
    fn signed_manifest_is_bounded_canonical_and_verifiable() {
        let signed = sign(manifest(
            "2.0.0-alpha.2",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Beta,
        ));
        let canonical = signed.to_canonical_json().unwrap();
        assert_eq!(
            SignedNativeUpdateManifestV1::from_json(&canonical).unwrap(),
            signed
        );
        let verified = signed
            .verify(
                &[trusted_key()],
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta),
            )
            .unwrap();
        assert_eq!(verified.manifest().artifact.version, "2.0.0-alpha.2");
        assert_eq!(verified.release_key_id(), "release-test-key");
        assert_eq!(verified.verified_at_unix(), NOW);
        assert_eq!(verified.signed_payload_sha256().len(), 64);
        assert!(
            native_update_manifest_signing_bytes(&signed.manifest)
                .unwrap()
                .starts_with(SIGNING_DOMAIN)
        );

        let mut noncanonical = canonical.clone();
        noncanonical.push(b'\n');
        assert_eq!(
            SignedNativeUpdateManifestV1::from_json(&noncanonical),
            Err(NativeUpdateError::NonCanonicalJson)
        );
        let mut unknown: Value = serde_json::from_slice(&canonical).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_string(), Value::Bool(true));
        assert_eq!(
            SignedNativeUpdateManifestV1::from_json(&serde_json::to_vec(&unknown).unwrap()),
            Err(NativeUpdateError::InvalidJson)
        );
        assert_eq!(
            SignedNativeUpdateManifestV1::from_json(&vec![b' '; 129 * 1024]),
            Err(NativeUpdateError::InputTooLarge)
        );
    }

    #[test]
    fn stable_and_beta_streams_enforce_the_two_stream_policy() {
        let stable_alpha = sign(manifest(
            "2.0.0-alpha.2",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Stable,
        ));
        assert_eq!(
            stable_alpha.verify(
                &[trusted_key()],
                &context("2.0.0-alpha.1", NativeUpdateStream::Stable)
            ),
            Err(NativeUpdateError::ChannelUnavailable)
        );

        for (version, channel) in [
            ("2.0.0-alpha.2", ReleaseChannel::Alpha),
            ("2.0.0-beta.1", ReleaseChannel::Beta),
            ("2.0.0", ReleaseChannel::Stable),
        ] {
            assert!(
                sign(manifest(version, channel, NativeUpdateStream::Beta))
                    .verify(
                        &[trusted_key()],
                        &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
                    )
                    .is_ok()
            );
        }

        let stable = sign(manifest(
            "2.0.0",
            ReleaseChannel::Stable,
            NativeUpdateStream::Stable,
        ));
        assert!(
            stable
                .verify(
                    &[trusted_key()],
                    &context("2.0.0-beta.1", NativeUpdateStream::Stable)
                )
                .is_ok()
        );
    }

    #[test]
    fn updater_rejects_legacy_equal_downgrade_and_incompatible_versions() {
        let update = sign(manifest(
            "2.0.0-alpha.2",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Beta,
        ));
        assert_eq!(
            update.verify(
                &[trusted_key()],
                &context("1.19.0-beta.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::LegacyCurrentVersion)
        );
        assert_eq!(
            update.verify(
                &[trusted_key()],
                &context("2.0.0-alpha.2", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::VersionNotNewer)
        );
        assert_eq!(
            update.verify(
                &[trusted_key()],
                &context("2.0.0-beta.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::VersionNotNewer)
        );

        let legacy_target = sign(manifest(
            "1.20.0",
            ReleaseChannel::Stable,
            NativeUpdateStream::Beta,
        ));
        assert_eq!(
            legacy_target.verify(
                &[trusted_key()],
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::LegacyTargetVersion)
        );

        let mut incompatible = manifest(
            "2.0.0-alpha.3",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Beta,
        );
        incompatible.minimum_updater_version = "2.0.0-alpha.2".to_string();
        assert_eq!(
            sign(incompatible).verify(
                &[trusted_key()],
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::UpdaterIncompatible)
        );
    }

    #[test]
    fn signature_generation_time_stream_team_and_host_fail_closed() {
        let update = sign(manifest(
            "2.0.0-alpha.2",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Beta,
        ));
        let key = trusted_key();

        let mut bad_signature = update.clone();
        bad_signature.signature.value_hex = "0".repeat(128);
        assert_eq!(
            bad_signature.verify(
                std::slice::from_ref(&key),
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::SignatureInvalid)
        );

        let future_key = TrustedReleasePublicKey::new(
            "release-test-key",
            SigningKey::from_bytes(&[71_u8; 32])
                .verifying_key()
                .to_bytes(),
            10,
            None,
        )
        .unwrap();
        assert_eq!(
            update.verify(
                &[future_key],
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::ReleaseKeyGenerationUnavailable)
        );

        let mut replay = context("2.0.0-alpha.1", NativeUpdateStream::Beta);
        replay.last_accepted_generation = 9;
        assert_eq!(
            update.verify(std::slice::from_ref(&key), &replay),
            Err(NativeUpdateError::GenerationReplayed)
        );

        let mut expired = context("2.0.0-alpha.1", NativeUpdateStream::Beta);
        expired.now_unix = NOW + 3_600;
        assert_eq!(
            update.verify(std::slice::from_ref(&key), &expired),
            Err(NativeUpdateError::Expired)
        );

        let mut not_yet_valid = context("2.0.0-alpha.1", NativeUpdateStream::Beta);
        not_yet_valid.now_unix = NOW - 61;
        assert_eq!(
            update.verify(std::slice::from_ref(&key), &not_yet_valid),
            Err(NativeUpdateError::NotYetValid)
        );

        assert_eq!(
            update.verify(
                std::slice::from_ref(&key),
                &context("2.0.0-alpha.1", NativeUpdateStream::Stable)
            ),
            Err(NativeUpdateError::StreamMismatch)
        );

        let mut team = context("2.0.0-alpha.1", NativeUpdateStream::Beta);
        team.expected_macos_team_id = "ZZZ999YYYY";
        assert_eq!(
            update.verify(std::slice::from_ref(&key), &team),
            Err(NativeUpdateError::TeamIdMismatch)
        );

        let untrusted_hosts = ["example.com"];
        let mut host = context("2.0.0-alpha.1", NativeUpdateStream::Beta);
        host.allowed_download_hosts = &untrusted_hosts;
        assert_eq!(
            update.verify(std::slice::from_ref(&key), &host),
            Err(NativeUpdateError::DownloadHostUntrusted)
        );
    }

    #[test]
    fn malformed_source_url_target_and_context_are_rejected() {
        for mutate in [
            |value: &mut NativeUpdateManifestV1| {
                value.archive_url = "http://github.com/file".to_string()
            },
            |value: &mut NativeUpdateManifestV1| value.source_commit = "not-a-commit".to_string(),
            |value: &mut NativeUpdateManifestV1| value.macos_team_id = "team".to_string(),
        ] {
            let mut value = manifest(
                "2.0.0-alpha.2",
                ReleaseChannel::Alpha,
                NativeUpdateStream::Beta,
            );
            mutate(&mut value);
            assert_eq!(
                native_update_manifest_signing_bytes(&value),
                Err(NativeUpdateError::InvalidManifest)
            );
        }

        let mut wrong_target = manifest(
            "2.0.0-alpha.2",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Beta,
        );
        wrong_target.artifact.arch = Architecture::X86_64;
        assert_eq!(
            sign(wrong_target).verify(
                &[trusted_key()],
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::TargetMismatch)
        );

        let mut legacy_minimum = manifest(
            "2.0.0-alpha.2",
            ReleaseChannel::Alpha,
            NativeUpdateStream::Beta,
        );
        legacy_minimum.minimum_updater_version = "1.19.0".to_string();
        assert_eq!(
            sign(legacy_minimum).verify(
                &[trusted_key()],
                &context("2.0.0-alpha.1", NativeUpdateStream::Beta)
            ),
            Err(NativeUpdateError::LegacyMinimumUpdaterVersion)
        );

        let unsorted_hosts = ["objects.githubusercontent.com", "github.com"];
        let mut invalid_context = context("2.0.0-alpha.1", NativeUpdateStream::Beta);
        invalid_context.allowed_download_hosts = &unsorted_hosts;
        assert_eq!(
            sign(manifest(
                "2.0.0-alpha.2",
                ReleaseChannel::Alpha,
                NativeUpdateStream::Beta,
            ))
            .verify(&[trusted_key()], &invalid_context),
            Err(NativeUpdateError::InvalidVerificationContext)
        );
    }

    #[test]
    fn errors_are_fixed_reason_codes_without_paths() {
        for error in [
            NativeUpdateError::LegacyCurrentVersion,
            NativeUpdateError::DownloadHostUntrusted,
            NativeUpdateError::SignatureInvalid,
        ] {
            assert_eq!(error.to_string(), error.reason_code());
            assert!(!error.to_string().contains('/'));
            assert!(!error.to_string().contains('\\'));
        }
    }
}
