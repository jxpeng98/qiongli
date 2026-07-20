use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{
    NativeCommunityAlphaAssetRole, NativeCommunityAlphaAssetV1, NativeCommunityAlphaCandidateSetV1,
    NativeDistributionClass, NativeDistributionReleaseSetV1, NativePublicationAuthorizationContext,
    NativePublicationAuthorizationV1, NativeReleaseAuthority, NativeReleaseSignatureV1,
    ReleaseChannel, SignatureAlgorithm, verify_native_distribution_release_set_authorization,
};

pub const NATIVE_COMMUNITY_ALPHA_INTEGRITY_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_SCHEMA_VERSION: u32 = 1;
pub const MAX_NATIVE_COMMUNITY_ALPHA_INTEGRITY_BYTES: usize = 512 * 1024;
pub const MAX_NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_BYTES: usize = 256 * 1024;

const SIGNING_DOMAIN: &[u8] = b"QIONGLI-NATIVE-COMMUNITY-ALPHA-INTEGRITY-V1\0";
const ED25519_SIGNATURE_BYTES: usize = 64;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const CANDIDATE_SET_FILE: &str = "qiongli-community-alpha-candidate-set.json";
const AUTHORITY_FILE: &str = "qiongli-native-release-authority.json";
const INTEGRITY_FILE: &str = "qiongli-community-alpha-integrity.json";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativeCommunityAlphaIntegrityRecordType {
    NativeCommunityAlphaIntegrity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaIntegrityManifestV1 {
    pub schema_version: u32,
    record_type: NativeCommunityAlphaIntegrityRecordType,
    pub generation: u64,
    pub distribution_class: NativeDistributionClass,
    pub release_set: NativeDistributionReleaseSetV1,
    pub candidate_set_file: String,
    pub candidate_set_file_sha256: String,
    pub authority_file: String,
    pub authority_sha256: String,
    pub checksums_file: String,
    pub checksums_sha256: String,
    pub sbom_file: String,
    pub sbom_sha256: String,
    pub provenance_file: String,
    pub provenance_sha256: String,
    pub release_notes_file: String,
    pub release_notes_sha256: String,
    pub assets: Vec<NativeCommunityAlphaAssetV1>,
    pub publication_allowed: bool,
}

impl NativeCommunityAlphaIntegrityManifestV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn from_candidate(
        candidate: &NativeCommunityAlphaCandidateSetV1,
        generation: u64,
        candidate_set_file_sha256: impl Into<String>,
        authority_sha256: impl Into<String>,
        checksums_sha256: impl Into<String>,
        sbom_sha256: impl Into<String>,
        provenance_sha256: impl Into<String>,
        release_notes_sha256: impl Into<String>,
    ) -> Result<Self, NativeCommunityAlphaIntegrityError> {
        candidate
            .validate()
            .map_err(|_| NativeCommunityAlphaIntegrityError::InvalidCandidate)?;
        let release_set = NativeDistributionReleaseSetV1::community_alpha(
            &candidate.content.source_commit,
            &candidate.candidate_set_sha256,
            &candidate.content.version,
            candidate
                .content
                .targets
                .iter()
                .map(|target| target.policy.clone())
                .collect(),
        )
        .map_err(|_| NativeCommunityAlphaIntegrityError::InvalidReleaseSet)?;
        let assets = candidate
            .content
            .targets
            .iter()
            .flat_map(|target| target.assets.iter().cloned())
            .collect();
        let version = &candidate.content.version;
        let manifest = Self {
            schema_version: NATIVE_COMMUNITY_ALPHA_INTEGRITY_SCHEMA_VERSION,
            record_type: NativeCommunityAlphaIntegrityRecordType::NativeCommunityAlphaIntegrity,
            generation,
            distribution_class: NativeDistributionClass::CommunityAlpha,
            release_set,
            candidate_set_file: CANDIDATE_SET_FILE.to_string(),
            candidate_set_file_sha256: candidate_set_file_sha256.into(),
            authority_file: AUTHORITY_FILE.to_string(),
            authority_sha256: authority_sha256.into(),
            checksums_file: format!("qiongli-{version}-community-alpha.SHA256SUMS"),
            checksums_sha256: checksums_sha256.into(),
            sbom_file: format!("qiongli-{version}-community-alpha.cdx.json"),
            sbom_sha256: sbom_sha256.into(),
            provenance_file: format!("qiongli-{version}-community-alpha.provenance.json"),
            provenance_sha256: provenance_sha256.into(),
            release_notes_file: format!("qiongli-{version}-community-alpha.release-notes.md"),
            release_notes_sha256: release_notes_sha256.into(),
            assets,
            publication_allowed: false,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), NativeCommunityAlphaIntegrityError> {
        self.release_set
            .validate()
            .map_err(|_| NativeCommunityAlphaIntegrityError::InvalidReleaseSet)?;
        let version = &self.release_set.version;
        if self.schema_version != NATIVE_COMMUNITY_ALPHA_INTEGRITY_SCHEMA_VERSION
            || self.record_type
                != NativeCommunityAlphaIntegrityRecordType::NativeCommunityAlphaIntegrity
            || self.generation == 0
            || self.generation > JCS_MAX_SAFE_INTEGER
            || self.distribution_class != NativeDistributionClass::CommunityAlpha
            || self.release_set.distribution_class != self.distribution_class
            || self.candidate_set_file != CANDIDATE_SET_FILE
            || self.authority_file != AUTHORITY_FILE
            || self.checksums_file != format!("qiongli-{version}-community-alpha.SHA256SUMS")
            || self.sbom_file != format!("qiongli-{version}-community-alpha.cdx.json")
            || self.provenance_file != format!("qiongli-{version}-community-alpha.provenance.json")
            || self.release_notes_file
                != format!("qiongli-{version}-community-alpha.release-notes.md")
            || !is_lower_hex(&self.candidate_set_file_sha256, 64)
            || !is_lower_hex(&self.authority_sha256, 64)
            || !is_lower_hex(&self.checksums_sha256, 64)
            || !is_lower_hex(&self.sbom_sha256, 64)
            || !is_lower_hex(&self.provenance_sha256, 64)
            || !is_lower_hex(&self.release_notes_sha256, 64)
            || self.publication_allowed
            || !valid_exact_asset_set(&self.assets, version)
        {
            return Err(NativeCommunityAlphaIntegrityError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedNativeCommunityAlphaIntegrityV1 {
    pub manifest: NativeCommunityAlphaIntegrityManifestV1,
    pub signature: NativeReleaseSignatureV1,
}

impl SignedNativeCommunityAlphaIntegrityV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, NativeCommunityAlphaIntegrityError> {
        if input.is_empty() || input.len() > MAX_NATIVE_COMMUNITY_ALPHA_INTEGRITY_BYTES {
            return Err(NativeCommunityAlphaIntegrityError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(input)
            .map_err(|_| NativeCommunityAlphaIntegrityError::InvalidJson)?;
        value.validate_structure()?;
        if value.to_canonical_json()?.as_slice() != input {
            return Err(NativeCommunityAlphaIntegrityError::NonCanonicalJson);
        }
        Ok(value)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeCommunityAlphaIntegrityError> {
        self.validate_structure()?;
        canonical_json(self)
    }

    pub fn verify(
        &self,
        authority: &NativeReleaseAuthority,
    ) -> Result<VerifiedNativeCommunityAlphaIntegrity, NativeCommunityAlphaIntegrityError> {
        self.validate_structure()?;
        if authority.channel() != ReleaseChannel::Alpha {
            return Err(NativeCommunityAlphaIntegrityError::AuthorityMismatch);
        }
        authority
            .validate_product_version(&self.manifest.release_set.version)
            .map_err(|_| NativeCommunityAlphaIntegrityError::AuthorityMismatch)?;
        let key = authority
            .release_keys()
            .iter()
            .find(|key| key.key_id() == self.signature.key_id)
            .ok_or(NativeCommunityAlphaIntegrityError::ReleaseKeyUntrusted)?;
        if !key.authorizes_generation(self.manifest.generation) {
            return Err(NativeCommunityAlphaIntegrityError::ReleaseKeyUnavailable);
        }
        let signature = decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(&self.signature.value_hex)
            .ok_or(NativeCommunityAlphaIntegrityError::InvalidSignature)?;
        let signing_bytes = native_community_alpha_integrity_signing_bytes(&self.manifest)?;
        if !key.verifies_signature(&signing_bytes, &signature) {
            return Err(NativeCommunityAlphaIntegrityError::InvalidSignature);
        }
        Ok(VerifiedNativeCommunityAlphaIntegrity {
            signed: self.clone(),
            signing_bytes_sha256: sha256_hex(&signing_bytes),
        })
    }

    fn validate_structure(&self) -> Result<(), NativeCommunityAlphaIntegrityError> {
        self.manifest.validate()?;
        if self.signature.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.signature.key_id)
            || !is_lower_hex(
                &self.signature.value_hex,
                ED25519_SIGNATURE_BYTES.saturating_mul(2),
            )
        {
            return Err(NativeCommunityAlphaIntegrityError::InvalidSignature);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct VerifiedNativeCommunityAlphaIntegrity {
    signed: SignedNativeCommunityAlphaIntegrityV1,
    signing_bytes_sha256: String,
}

impl VerifiedNativeCommunityAlphaIntegrity {
    #[must_use]
    pub const fn signed(&self) -> &SignedNativeCommunityAlphaIntegrityV1 {
        &self.signed
    }

    #[must_use]
    pub fn signing_bytes_sha256(&self) -> &str {
        &self.signing_bytes_sha256
    }
}

impl Debug for VerifiedNativeCommunityAlphaIntegrity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedNativeCommunityAlphaIntegrity")
            .field("version", &self.signed.manifest.release_set.version)
            .field(
                "source_commit",
                &self.signed.manifest.release_set.source_commit,
            )
            .field(
                "release_set_sha256",
                &self.signed.manifest.release_set.release_set_sha256,
            )
            .field("release_key_id", &self.signed.signature.key_id)
            .field("signing_bytes_sha256", &self.signing_bytes_sha256)
            .finish()
    }
}

pub fn native_community_alpha_integrity_signing_bytes(
    manifest: &NativeCommunityAlphaIntegrityManifestV1,
) -> Result<Vec<u8>, NativeCommunityAlphaIntegrityError> {
    manifest.validate()?;
    let payload = canonical_json(manifest)?;
    let mut bytes = Vec::with_capacity(SIGNING_DOMAIN.len().saturating_add(payload.len()));
    bytes.extend_from_slice(SIGNING_DOMAIN);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativeCommunityAlphaPublicationReceiptRecordType {
    NativeCommunityAlphaPublicationAuthorizationReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaPublicationReceiptV1 {
    pub schema_version: u32,
    record_type: NativeCommunityAlphaPublicationReceiptRecordType,
    pub distribution_class: NativeDistributionClass,
    pub version: String,
    pub source_commit: String,
    pub release_set_sha256: String,
    pub integrity_file: String,
    pub integrity_sha256: String,
    pub tag: String,
    pub prerelease: bool,
    pub authorization: NativePublicationAuthorizationV1,
    pub publication_allowed: bool,
    pub publication_performed: bool,
}

impl NativeCommunityAlphaPublicationReceiptV1 {
    pub fn authorize(
        integrity: &VerifiedNativeCommunityAlphaIntegrity,
        integrity_sha256: impl Into<String>,
        authorization: NativePublicationAuthorizationV1,
        context: &NativePublicationAuthorizationContext<'_>,
    ) -> Result<Self, NativeCommunityAlphaIntegrityError> {
        let release_set = integrity.signed().manifest.release_set.clone();
        verify_native_distribution_release_set_authorization(
            release_set.clone(),
            authorization.clone(),
            context,
        )
        .map_err(|_| NativeCommunityAlphaIntegrityError::AuthorizationMismatch)?;
        let receipt = Self {
            schema_version: NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_SCHEMA_VERSION,
            record_type: NativeCommunityAlphaPublicationReceiptRecordType::NativeCommunityAlphaPublicationAuthorizationReceipt,
            distribution_class: NativeDistributionClass::CommunityAlpha,
            version: release_set.version.clone(),
            source_commit: release_set.source_commit.clone(),
            release_set_sha256: release_set.release_set_sha256.clone(),
            integrity_file: INTEGRITY_FILE.to_string(),
            integrity_sha256: integrity_sha256.into(),
            tag: format!("v{}", release_set.version),
            prerelease: true,
            authorization,
            publication_allowed: true,
            publication_performed: false,
        };
        receipt.validate_shape()?;
        Ok(receipt)
    }

    pub fn from_json(input: &[u8]) -> Result<Self, NativeCommunityAlphaIntegrityError> {
        if input.is_empty() || input.len() > MAX_NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_BYTES {
            return Err(NativeCommunityAlphaIntegrityError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(input)
            .map_err(|_| NativeCommunityAlphaIntegrityError::InvalidJson)?;
        value.validate_shape()?;
        if value.to_canonical_json()?.as_slice() != input {
            return Err(NativeCommunityAlphaIntegrityError::NonCanonicalJson);
        }
        Ok(value)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeCommunityAlphaIntegrityError> {
        self.validate_shape()?;
        canonical_json(self)
    }

    fn validate_shape(&self) -> Result<(), NativeCommunityAlphaIntegrityError> {
        if self.schema_version != NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_SCHEMA_VERSION
            || self.record_type
                != NativeCommunityAlphaPublicationReceiptRecordType::NativeCommunityAlphaPublicationAuthorizationReceipt
            || self.distribution_class != NativeDistributionClass::CommunityAlpha
            || !valid_source_commit(&self.source_commit)
            || !is_lower_hex(&self.release_set_sha256, 64)
            || self.integrity_file != INTEGRITY_FILE
            || !is_lower_hex(&self.integrity_sha256, 64)
            || self.tag != format!("v{}", self.version)
            || !self.prerelease
            || !self.publication_allowed
            || self.publication_performed
            || self.authorization.distribution_class != self.distribution_class
            || self.authorization.source_commit != self.source_commit
            || self.authorization.release_set_sha256 != self.release_set_sha256
        {
            return Err(NativeCommunityAlphaIntegrityError::InvalidPublicationReceipt);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeCommunityAlphaIntegrityError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidCandidate,
    InvalidReleaseSet,
    InvalidManifest,
    InvalidSignature,
    AuthorityMismatch,
    ReleaseKeyUntrusted,
    ReleaseKeyUnavailable,
    AuthorizationMismatch,
    InvalidPublicationReceipt,
    CanonicalSerializationFailed,
}

impl NativeCommunityAlphaIntegrityError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "community-alpha-integrity-input-too-large",
            Self::InvalidJson => "community-alpha-integrity-json-invalid",
            Self::NonCanonicalJson => "community-alpha-integrity-json-noncanonical",
            Self::InvalidCandidate => "community-alpha-integrity-candidate-invalid",
            Self::InvalidReleaseSet => "community-alpha-integrity-release-set-invalid",
            Self::InvalidManifest => "community-alpha-integrity-manifest-invalid",
            Self::InvalidSignature => "community-alpha-integrity-signature-invalid",
            Self::AuthorityMismatch => "community-alpha-integrity-authority-mismatch",
            Self::ReleaseKeyUntrusted => "community-alpha-integrity-release-key-untrusted",
            Self::ReleaseKeyUnavailable => "community-alpha-integrity-release-key-unavailable",
            Self::AuthorizationMismatch => "community-alpha-publication-authorization-mismatch",
            Self::InvalidPublicationReceipt => "community-alpha-publication-receipt-invalid",
            Self::CanonicalSerializationFailed => {
                "community-alpha-integrity-canonical-serialization-failed"
            }
        }
    }
}

impl Display for NativeCommunityAlphaIntegrityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeCommunityAlphaIntegrityError {}

fn valid_exact_asset_set(assets: &[NativeCommunityAlphaAssetV1], version: &str) -> bool {
    let expected = [
        (
            NativeCommunityAlphaAssetRole::MacosApplicationZip,
            format!("Qiongli-{version}-macOS-arm64.zip"),
        ),
        (
            NativeCommunityAlphaAssetRole::MacosInstallerDmg,
            format!("Qiongli-{version}-macOS-arm64.dmg"),
        ),
        (
            NativeCommunityAlphaAssetRole::WindowsPortableZip,
            format!("Qiongli-{version}-Windows-x64.zip"),
        ),
        (
            NativeCommunityAlphaAssetRole::LinuxAppimage,
            format!("Qiongli-{version}-Linux-x64.AppImage"),
        ),
        (
            NativeCommunityAlphaAssetRole::LinuxPortableDirectoryZip,
            format!("Qiongli-{version}-Linux-x64.zip"),
        ),
    ];
    assets.len() == expected.len()
        && assets.iter().zip(expected).all(|(asset, (role, file))| {
            asset.role == role
                && asset.file == file
                && asset.size_bytes > 0
                && is_lower_hex(&asset.sha256, 64)
        })
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if !is_lower_hex(value, N.saturating_mul(2)) {
        return None;
    }
    let mut bytes = [0_u8; N];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_value(pair[0])? << 4) | hex_value(pair[1])?;
    }
    Some(bytes)
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, NativeCommunityAlphaIntegrityError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| NativeCommunityAlphaIntegrityError::CanonicalSerializationFailed)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    encode_hex(&hasher.finalize())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer as _, SigningKey};

    use super::*;
    use crate::{
        Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind,
        NativeCommunityAlphaEvidenceRole, NativeCommunityAlphaEvidenceV1,
        NativeCommunityAlphaTargetPromotionV1, NativeDistributionPolicyV1, OperatingSystem,
        ProductId,
    };

    const SOURCE: &str = "0123456789abcdef0123456789abcdef01234567";
    const RUN: &str = "https://github.com/jxpeng98/qiongli/actions/runs/29575237942";
    const DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

    fn policy(os: OperatingSystem, arch: Architecture) -> NativeDistributionPolicyV1 {
        NativeDistributionPolicyV1::for_artifact(
            NativeDistributionClass::CommunityAlpha,
            ArtifactIdentityV1 {
                product: ProductId::Qiongli,
                version: "2.0.0-alpha.1".to_string(),
                channel: ReleaseChannel::Alpha,
                profile: CapabilityProfile::Lite,
                os,
                arch,
                installer_kind: InstallerKind::NativeInstaller,
            },
        )
        .unwrap()
    }

    fn asset(role: NativeCommunityAlphaAssetRole, file: &str) -> NativeCommunityAlphaAssetV1 {
        NativeCommunityAlphaAssetV1::new(role, file, 42, DIGEST).unwrap()
    }

    fn evidence(
        role: NativeCommunityAlphaEvidenceRole,
        file: &str,
    ) -> NativeCommunityAlphaEvidenceV1 {
        NativeCommunityAlphaEvidenceV1::new(role, file, 42, DIGEST).unwrap()
    }

    fn target(os: OperatingSystem) -> NativeCommunityAlphaTargetPromotionV1 {
        let (arch, assets, evidence) = match os {
            OperatingSystem::Macos => (
                Architecture::Aarch64,
                vec![
                    asset(
                        NativeCommunityAlphaAssetRole::MacosApplicationZip,
                        "Qiongli-2.0.0-alpha.1-macOS-arm64.zip",
                    ),
                    asset(
                        NativeCommunityAlphaAssetRole::MacosInstallerDmg,
                        "Qiongli-2.0.0-alpha.1-macOS-arm64.dmg",
                    ),
                ],
                vec![
                    evidence(
                        NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
                        "qiongli-desktop-package.manifest.json",
                    ),
                    evidence(
                        NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
                        "qiongli-desktop-package.receipt.json",
                    ),
                    evidence(
                        NativeCommunityAlphaEvidenceRole::MacosSourceAcceptanceReceipt,
                        "qiongli-macos-alpha1-unsigned-acceptance.receipt.json",
                    ),
                    evidence(
                        NativeCommunityAlphaEvidenceRole::MacosSigningReceipt,
                        "qiongli-macos-alpha1-signing.receipt.json",
                    ),
                ],
            ),
            OperatingSystem::Windows => (
                Architecture::X86_64,
                vec![asset(
                    NativeCommunityAlphaAssetRole::WindowsPortableZip,
                    "Qiongli-2.0.0-alpha.1-Windows-x64.zip",
                )],
                vec![
                    evidence(
                        NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
                        "qiongli-desktop-package.manifest.json",
                    ),
                    evidence(
                        NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
                        "qiongli-desktop-package.receipt.json",
                    ),
                ],
            ),
            OperatingSystem::Linux => (
                Architecture::X86_64,
                vec![
                    asset(
                        NativeCommunityAlphaAssetRole::LinuxAppimage,
                        "Qiongli-2.0.0-alpha.1-Linux-x64.AppImage",
                    ),
                    asset(
                        NativeCommunityAlphaAssetRole::LinuxPortableDirectoryZip,
                        "Qiongli-2.0.0-alpha.1-Linux-x64.zip",
                    ),
                ],
                vec![
                    evidence(
                        NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
                        "qiongli-desktop-package.manifest.json",
                    ),
                    evidence(
                        NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
                        "qiongli-desktop-package.receipt.json",
                    ),
                    evidence(
                        NativeCommunityAlphaEvidenceRole::LinuxAppimageReceipt,
                        "qiongli-linux-appimage.receipt.json",
                    ),
                ],
            ),
        };
        NativeCommunityAlphaTargetPromotionV1::fresh_target_native(
            SOURCE,
            RUN,
            policy(os, arch),
            assets,
            evidence,
        )
        .unwrap()
    }

    fn candidate() -> NativeCommunityAlphaCandidateSetV1 {
        NativeCommunityAlphaCandidateSetV1::from_fresh_targets(vec![
            target(OperatingSystem::Macos),
            target(OperatingSystem::Windows),
            target(OperatingSystem::Linux),
        ])
        .unwrap()
    }

    fn authority(signing: &SigningKey) -> NativeReleaseAuthority {
        let launch = SigningKey::from_bytes(&[9_u8; 32]);
        let document = serde_json::json!({
            "channel": "alpha",
            "launch_grant_keys": [{
                "key_id": "community-alpha-launch-1",
                "public_key_hex": encode_hex(launch.verifying_key().as_bytes())
            }],
            "minimum_launch_grant_generation": 1,
            "minimum_release_generation": 1,
            "release_keys": [{
                "key_id": "community-alpha-release-1",
                "maximum_generation_exclusive": null,
                "minimum_generation": 1,
                "public_key_hex": encode_hex(signing.verifying_key().as_bytes())
            }],
            "schema_version": 1
        });
        NativeReleaseAuthority::from_json(&serde_json_canonicalizer::to_vec(&document).unwrap())
            .unwrap()
    }

    fn signed_integrity() -> (
        SignedNativeCommunityAlphaIntegrityV1,
        NativeReleaseAuthority,
    ) {
        let signing = SigningKey::from_bytes(&[7_u8; 32]);
        let authority = authority(&signing);
        let manifest = NativeCommunityAlphaIntegrityManifestV1::from_candidate(
            &candidate(),
            1,
            DIGEST,
            DIGEST,
            DIGEST,
            DIGEST,
            DIGEST,
            DIGEST,
        )
        .unwrap();
        let signature =
            signing.sign(&native_community_alpha_integrity_signing_bytes(&manifest).unwrap());
        (
            SignedNativeCommunityAlphaIntegrityV1 {
                manifest,
                signature: NativeReleaseSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "community-alpha-release-1".to_string(),
                    value_hex: encode_hex(&signature.to_bytes()),
                },
            },
            authority,
        )
    }

    #[test]
    fn signed_integrity_binds_the_exact_three_target_release_set() {
        let (signed, authority) = signed_integrity();
        let bytes = signed.to_canonical_json().unwrap();
        let parsed = SignedNativeCommunityAlphaIntegrityV1::from_json(&bytes).unwrap();
        let verified = parsed.verify(&authority).unwrap();
        assert_eq!(verified.signed().manifest.assets.len(), 5);
        assert!(!verified.signed().manifest.publication_allowed);
    }

    #[test]
    fn integrity_rejects_candidate_asset_drift() {
        let (mut signed, authority) = signed_integrity();
        signed.manifest.assets[0].sha256 = "0".repeat(64);
        assert_eq!(
            signed.verify(&authority).unwrap_err(),
            NativeCommunityAlphaIntegrityError::InvalidSignature
        );
    }

    #[test]
    fn authorization_receipt_consumes_only_matching_environment_context() {
        let (signed, authority) = signed_integrity();
        let verified = signed.verify(&authority).unwrap();
        let authorization = NativePublicationAuthorizationV1::exact_release_set(
            NativeDistributionClass::CommunityAlpha,
            SOURCE,
            &signed.manifest.release_set.release_set_sha256,
            "jxpeng98/qiongli",
            "community-alpha-publication",
            RUN,
            "jxpeng98",
            1_800_000_000,
        )
        .unwrap();
        let context = NativePublicationAuthorizationContext {
            expected_distribution_class: NativeDistributionClass::CommunityAlpha,
            expected_source_commit: SOURCE,
            expected_release_set_sha256: &signed.manifest.release_set.release_set_sha256,
            expected_repository: "jxpeng98/qiongli",
            expected_environment: "community-alpha-publication",
            expected_workflow_run_url: RUN,
            expected_actor: "jxpeng98",
            verified_at_unix: 1_800_000_001,
            max_authorization_age_seconds: 600,
        };
        let receipt = NativeCommunityAlphaPublicationReceiptV1::authorize(
            &verified,
            DIGEST,
            authorization,
            &context,
        )
        .unwrap();
        assert!(receipt.publication_allowed);
        assert!(!receipt.publication_performed);
        assert_eq!(receipt.tag, "v2.0.0-alpha.1");
    }
}
