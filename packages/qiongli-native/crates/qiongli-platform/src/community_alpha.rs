use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{Architecture, NativeDistributionClass, NativeDistributionPolicyV1, OperatingSystem};

pub const NATIVE_COMMUNITY_ALPHA_PROMOTION_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_SCHEMA_VERSION: u32 = 1;
pub const MAX_NATIVE_COMMUNITY_ALPHA_PROMOTION_BYTES: usize = 256 * 1024;
pub const MAX_NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_BYTES: usize = 512 * 1024;

const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_ASSET_BYTES: u64 = 1024 * 1024 * 1024;
const BUILD_RUN_URL_PREFIX: &str = "https://github.com/jxpeng98/qiongli/actions/runs/";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeCommunityAlphaAssetRole {
    MacosApplicationZip,
    MacosInstallerDmg,
    WindowsPortableZip,
    LinuxAppimage,
    LinuxPortableDirectoryZip,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaAssetV1 {
    pub role: NativeCommunityAlphaAssetRole,
    pub file: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeCommunityAlphaEvidenceRole {
    DesktopPackageManifest,
    DesktopPackageReceipt,
    MacosSourceAcceptanceReceipt,
    MacosSigningReceipt,
    LinuxAppimageReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaEvidenceV1 {
    pub role: NativeCommunityAlphaEvidenceRole,
    pub file: String,
    pub size_bytes: u64,
    pub sha256: String,
}

impl NativeCommunityAlphaEvidenceV1 {
    pub fn new(
        role: NativeCommunityAlphaEvidenceRole,
        file: impl Into<String>,
        size_bytes: u64,
        sha256: impl Into<String>,
    ) -> Result<Self, NativeCommunityAlphaPromotionError> {
        let evidence = Self {
            role,
            file: file.into(),
            size_bytes,
            sha256: sha256.into(),
        };
        evidence.validate_shape()?;
        Ok(evidence)
    }

    fn validate_shape(&self) -> Result<(), NativeCommunityAlphaPromotionError> {
        if !valid_file_name(&self.file)
            || self.size_bytes == 0
            || self.size_bytes > MAX_NATIVE_COMMUNITY_ALPHA_PROMOTION_BYTES as u64
            || !is_lower_hex(&self.sha256, 64)
        {
            return Err(NativeCommunityAlphaPromotionError::InvalidEvidence);
        }
        Ok(())
    }
}

impl NativeCommunityAlphaAssetV1 {
    pub fn new(
        role: NativeCommunityAlphaAssetRole,
        file: impl Into<String>,
        size_bytes: u64,
        sha256: impl Into<String>,
    ) -> Result<Self, NativeCommunityAlphaPromotionError> {
        let asset = Self {
            role,
            file: file.into(),
            size_bytes,
            sha256: sha256.into(),
        };
        asset.validate_shape()?;
        Ok(asset)
    }

    fn validate_shape(&self) -> Result<(), NativeCommunityAlphaPromotionError> {
        if !valid_file_name(&self.file)
            || self.size_bytes == 0
            || self.size_bytes > MAX_ASSET_BYTES
            || self.size_bytes > JCS_MAX_SAFE_INTEGER
            || !is_lower_hex(&self.sha256, 64)
        {
            return Err(NativeCommunityAlphaPromotionError::InvalidAsset);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativeCommunityAlphaPromotionRecordType {
    NativeCommunityAlphaTargetPromotion,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeCommunityAlphaBuildProvenance {
    FreshExactSourceTargetNativeBuild,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaTargetPromotionV1 {
    pub schema_version: u32,
    record_type: NativeCommunityAlphaPromotionRecordType,
    pub distribution_class: NativeDistributionClass,
    pub source_commit: String,
    pub build_run_url: String,
    pub provenance: NativeCommunityAlphaBuildProvenance,
    pub raw_ci_artifact_reused: bool,
    pub publication_allowed: bool,
    pub policy: NativeDistributionPolicyV1,
    pub assets: Vec<NativeCommunityAlphaAssetV1>,
    pub evidence: Vec<NativeCommunityAlphaEvidenceV1>,
}

impl NativeCommunityAlphaTargetPromotionV1 {
    pub fn fresh_target_native(
        source_commit: impl Into<String>,
        build_run_url: impl Into<String>,
        policy: NativeDistributionPolicyV1,
        assets: Vec<NativeCommunityAlphaAssetV1>,
        evidence: Vec<NativeCommunityAlphaEvidenceV1>,
    ) -> Result<Self, NativeCommunityAlphaPromotionError> {
        let promotion = Self {
            schema_version: NATIVE_COMMUNITY_ALPHA_PROMOTION_SCHEMA_VERSION,
            record_type:
                NativeCommunityAlphaPromotionRecordType::NativeCommunityAlphaTargetPromotion,
            distribution_class: NativeDistributionClass::CommunityAlpha,
            source_commit: source_commit.into(),
            build_run_url: build_run_url.into(),
            provenance: NativeCommunityAlphaBuildProvenance::FreshExactSourceTargetNativeBuild,
            raw_ci_artifact_reused: false,
            publication_allowed: false,
            policy,
            assets,
            evidence,
        };
        promotion.validate()?;
        Ok(promotion)
    }

    pub fn validate(&self) -> Result<(), NativeCommunityAlphaPromotionError> {
        if self.schema_version != NATIVE_COMMUNITY_ALPHA_PROMOTION_SCHEMA_VERSION
            || self.record_type
                != NativeCommunityAlphaPromotionRecordType::NativeCommunityAlphaTargetPromotion
            || self.distribution_class != NativeDistributionClass::CommunityAlpha
            || !valid_source_commit(&self.source_commit)
            || !valid_build_run_url(&self.build_run_url)
            || self.provenance
                != NativeCommunityAlphaBuildProvenance::FreshExactSourceTargetNativeBuild
            || self.raw_ci_artifact_reused
            || self.publication_allowed
        {
            return Err(NativeCommunityAlphaPromotionError::InvalidPromotion);
        }
        self.policy
            .validate()
            .map_err(|_| NativeCommunityAlphaPromotionError::InvalidPolicy)?;
        if self.policy.distribution_class != self.distribution_class {
            return Err(NativeCommunityAlphaPromotionError::InvalidPolicy);
        }
        let expected_roles = expected_roles(&self.policy)?;
        if self.assets.len() != expected_roles.len() {
            return Err(NativeCommunityAlphaPromotionError::InvalidAssetSet);
        }
        for (asset, expected_role) in self.assets.iter().zip(expected_roles) {
            asset.validate_shape()?;
            if asset.role != expected_role
                || asset.file != expected_asset_file_name(expected_role, &self.policy)
            {
                return Err(NativeCommunityAlphaPromotionError::InvalidAssetSet);
            }
        }
        let expected_evidence_roles = expected_evidence_roles(&self.policy)?;
        if self.evidence.len() != expected_evidence_roles.len() {
            return Err(NativeCommunityAlphaPromotionError::InvalidEvidenceSet);
        }
        for (evidence, expected_role) in self.evidence.iter().zip(expected_evidence_roles) {
            evidence.validate_shape()?;
            if evidence.role != expected_role
                || evidence.file != expected_evidence_file_name(expected_role)
            {
                return Err(NativeCommunityAlphaPromotionError::InvalidEvidenceSet);
            }
        }
        Ok(())
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeCommunityAlphaPromotionError> {
        self.validate()?;
        canonical_json(self)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, NativeCommunityAlphaPromotionError> {
        if bytes.is_empty() || bytes.len() > MAX_NATIVE_COMMUNITY_ALPHA_PROMOTION_BYTES {
            return Err(NativeCommunityAlphaPromotionError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(bytes)
            .map_err(|_| NativeCommunityAlphaPromotionError::InvalidJson)?;
        value.validate()?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(NativeCommunityAlphaPromotionError::NonCanonicalJson);
        }
        Ok(value)
    }
}

fn expected_roles(
    policy: &NativeDistributionPolicyV1,
) -> Result<Vec<NativeCommunityAlphaAssetRole>, NativeCommunityAlphaPromotionError> {
    match (policy.artifact.os, policy.artifact.arch) {
        (OperatingSystem::Macos, Architecture::Aarch64) => Ok(vec![
            NativeCommunityAlphaAssetRole::MacosApplicationZip,
            NativeCommunityAlphaAssetRole::MacosInstallerDmg,
        ]),
        (OperatingSystem::Windows, Architecture::X86_64) => {
            Ok(vec![NativeCommunityAlphaAssetRole::WindowsPortableZip])
        }
        (OperatingSystem::Linux, Architecture::X86_64) => Ok(vec![
            NativeCommunityAlphaAssetRole::LinuxAppimage,
            NativeCommunityAlphaAssetRole::LinuxPortableDirectoryZip,
        ]),
        _ => Err(NativeCommunityAlphaPromotionError::InvalidPolicy),
    }
}

fn expected_evidence_roles(
    policy: &NativeDistributionPolicyV1,
) -> Result<Vec<NativeCommunityAlphaEvidenceRole>, NativeCommunityAlphaPromotionError> {
    match (policy.artifact.os, policy.artifact.arch) {
        (OperatingSystem::Macos, Architecture::Aarch64) => Ok(vec![
            NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
            NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
            NativeCommunityAlphaEvidenceRole::MacosSourceAcceptanceReceipt,
            NativeCommunityAlphaEvidenceRole::MacosSigningReceipt,
        ]),
        (OperatingSystem::Windows, Architecture::X86_64) => Ok(vec![
            NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
            NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
        ]),
        (OperatingSystem::Linux, Architecture::X86_64) => Ok(vec![
            NativeCommunityAlphaEvidenceRole::DesktopPackageManifest,
            NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt,
            NativeCommunityAlphaEvidenceRole::LinuxAppimageReceipt,
        ]),
        _ => Err(NativeCommunityAlphaPromotionError::InvalidPolicy),
    }
}

const fn expected_evidence_file_name(role: NativeCommunityAlphaEvidenceRole) -> &'static str {
    match role {
        NativeCommunityAlphaEvidenceRole::DesktopPackageManifest => {
            "qiongli-desktop-package.manifest.json"
        }
        NativeCommunityAlphaEvidenceRole::DesktopPackageReceipt => {
            "qiongli-desktop-package.receipt.json"
        }
        NativeCommunityAlphaEvidenceRole::MacosSourceAcceptanceReceipt => {
            "qiongli-macos-alpha1-unsigned-acceptance.receipt.json"
        }
        NativeCommunityAlphaEvidenceRole::MacosSigningReceipt => {
            "qiongli-macos-alpha1-signing.receipt.json"
        }
        NativeCommunityAlphaEvidenceRole::LinuxAppimageReceipt => {
            "qiongli-linux-appimage.receipt.json"
        }
    }
}

fn expected_asset_file_name(
    role: NativeCommunityAlphaAssetRole,
    policy: &NativeDistributionPolicyV1,
) -> String {
    let version = &policy.artifact.version;
    match role {
        NativeCommunityAlphaAssetRole::MacosApplicationZip => {
            format!("Qiongli-{version}-macOS-arm64.zip")
        }
        NativeCommunityAlphaAssetRole::MacosInstallerDmg => {
            format!("Qiongli-{version}-macOS-arm64.dmg")
        }
        NativeCommunityAlphaAssetRole::WindowsPortableZip => {
            format!("Qiongli-{version}-Windows-x64.zip")
        }
        NativeCommunityAlphaAssetRole::LinuxAppimage => {
            format!("Qiongli-{version}-Linux-x64.AppImage")
        }
        NativeCommunityAlphaAssetRole::LinuxPortableDirectoryZip => {
            format!("Qiongli-{version}-Linux-x64.zip")
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativeCommunityAlphaCandidateSetRecordType {
    NativeCommunityAlphaCandidateSet,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeCommunityAlphaCandidateStatus {
    FreshThreeTargetNonpublishingCandidate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaCandidateSetContentV1 {
    pub distribution_class: NativeDistributionClass,
    pub status: NativeCommunityAlphaCandidateStatus,
    pub publication_allowed: bool,
    pub source_commit: String,
    pub version: String,
    pub build_run_url: String,
    pub targets: Vec<NativeCommunityAlphaTargetPromotionV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommunityAlphaCandidateSetV1 {
    pub schema_version: u32,
    record_type: NativeCommunityAlphaCandidateSetRecordType,
    pub content: NativeCommunityAlphaCandidateSetContentV1,
    pub candidate_set_sha256: String,
}

impl NativeCommunityAlphaCandidateSetV1 {
    pub fn from_fresh_targets(
        targets: Vec<NativeCommunityAlphaTargetPromotionV1>,
    ) -> Result<Self, NativeCommunityAlphaPromotionError> {
        let first = targets
            .first()
            .ok_or(NativeCommunityAlphaPromotionError::InvalidCandidateSet)?;
        let content = NativeCommunityAlphaCandidateSetContentV1 {
            distribution_class: NativeDistributionClass::CommunityAlpha,
            status: NativeCommunityAlphaCandidateStatus::FreshThreeTargetNonpublishingCandidate,
            publication_allowed: false,
            source_commit: first.source_commit.clone(),
            version: first.policy.artifact.version.clone(),
            build_run_url: first.build_run_url.clone(),
            targets,
        };
        let candidate_set_sha256 = sha256_hex(&canonical_json(&content)?);
        let candidate = Self {
            schema_version: NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_SCHEMA_VERSION,
            record_type:
                NativeCommunityAlphaCandidateSetRecordType::NativeCommunityAlphaCandidateSet,
            content,
            candidate_set_sha256,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    pub fn validate(&self) -> Result<(), NativeCommunityAlphaPromotionError> {
        if self.schema_version != NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_SCHEMA_VERSION
            || self.record_type
                != NativeCommunityAlphaCandidateSetRecordType::NativeCommunityAlphaCandidateSet
            || self.content.distribution_class != NativeDistributionClass::CommunityAlpha
            || self.content.status
                != NativeCommunityAlphaCandidateStatus::FreshThreeTargetNonpublishingCandidate
            || self.content.publication_allowed
            || !valid_source_commit(&self.content.source_commit)
            || !valid_build_run_url(&self.content.build_run_url)
            || self.content.version.is_empty()
            || self.content.version.len() > 64
            || !self.content.version.is_ascii()
            || !is_lower_hex(&self.candidate_set_sha256, 64)
            || self.content.targets.len() != 3
        {
            return Err(NativeCommunityAlphaPromotionError::InvalidCandidateSet);
        }
        let expected_targets = [
            (OperatingSystem::Macos, Architecture::Aarch64),
            (OperatingSystem::Windows, Architecture::X86_64),
            (OperatingSystem::Linux, Architecture::X86_64),
        ];
        for (target, (expected_os, expected_arch)) in
            self.content.targets.iter().zip(expected_targets)
        {
            target.validate()?;
            if target.distribution_class != self.content.distribution_class
                || target.source_commit != self.content.source_commit
                || target.build_run_url != self.content.build_run_url
                || target.policy.artifact.version != self.content.version
                || target.policy.artifact.os != expected_os
                || target.policy.artifact.arch != expected_arch
            {
                return Err(NativeCommunityAlphaPromotionError::InvalidCandidateSet);
            }
        }
        let expected_digest = sha256_hex(&canonical_json(&self.content)?);
        if self.candidate_set_sha256 != expected_digest {
            return Err(NativeCommunityAlphaPromotionError::CandidateSetDigestMismatch);
        }
        Ok(())
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeCommunityAlphaPromotionError> {
        self.validate()?;
        canonical_json(self)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, NativeCommunityAlphaPromotionError> {
        if bytes.is_empty() || bytes.len() > MAX_NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_BYTES {
            return Err(NativeCommunityAlphaPromotionError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(bytes)
            .map_err(|_| NativeCommunityAlphaPromotionError::InvalidJson)?;
        value.validate()?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(NativeCommunityAlphaPromotionError::NonCanonicalJson);
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeCommunityAlphaPromotionError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidAsset,
    InvalidAssetSet,
    InvalidEvidence,
    InvalidEvidenceSet,
    InvalidPolicy,
    InvalidPromotion,
    InvalidCandidateSet,
    CandidateSetDigestMismatch,
    CanonicalSerializationFailed,
}

impl NativeCommunityAlphaPromotionError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "community-alpha-promotion-input-too-large",
            Self::InvalidJson => "community-alpha-promotion-json-invalid",
            Self::NonCanonicalJson => "community-alpha-promotion-json-noncanonical",
            Self::InvalidAsset => "community-alpha-promotion-asset-invalid",
            Self::InvalidAssetSet => "community-alpha-promotion-asset-set-invalid",
            Self::InvalidEvidence => "community-alpha-promotion-evidence-invalid",
            Self::InvalidEvidenceSet => "community-alpha-promotion-evidence-set-invalid",
            Self::InvalidPolicy => "community-alpha-promotion-policy-invalid",
            Self::InvalidPromotion => "community-alpha-promotion-record-invalid",
            Self::InvalidCandidateSet => "community-alpha-candidate-set-invalid",
            Self::CandidateSetDigestMismatch => "community-alpha-candidate-set-digest-mismatch",
            Self::CanonicalSerializationFailed => {
                "community-alpha-promotion-canonical-serialization-failed"
            }
        }
    }
}

impl Display for NativeCommunityAlphaPromotionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeCommunityAlphaPromotionError {}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, NativeCommunityAlphaPromotionError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| NativeCommunityAlphaPromotionError::CanonicalSerializationFailed)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    encode_hex(&hasher.finalize())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(HEX[(byte >> 4) as usize] as char);
        value.push(HEX[(byte & 0x0f) as usize] as char);
    }
    value
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn valid_build_run_url(value: &str) -> bool {
    value
        .strip_prefix(BUILD_RUN_URL_PREFIX)
        .is_some_and(|run_id| {
            !run_id.is_empty()
                && run_id.len() <= 20
                && run_id.bytes().all(|byte| byte.is_ascii_digit())
                && !run_id.starts_with('0')
        })
}

fn valid_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value.is_ascii()
        && value != "."
        && value != ".."
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::{
        ArtifactIdentityV1, CapabilityProfile, InstallerKind, NativeDistributionPolicyV1,
        ProductId, ReleaseChannel,
    };

    const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const RUN_URL: &str = "https://github.com/jxpeng98/qiongli/actions/runs/29575237942";
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

    fn promotion(os: OperatingSystem) -> NativeCommunityAlphaTargetPromotionV1 {
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
            SOURCE_COMMIT,
            RUN_URL,
            policy(os, arch),
            assets,
            evidence,
        )
        .unwrap()
    }

    fn candidate() -> NativeCommunityAlphaCandidateSetV1 {
        NativeCommunityAlphaCandidateSetV1::from_fresh_targets(vec![
            promotion(OperatingSystem::Macos),
            promotion(OperatingSystem::Windows),
            promotion(OperatingSystem::Linux),
        ])
        .unwrap()
    }

    #[test]
    fn each_target_requires_its_exact_public_assets() {
        for os in [
            OperatingSystem::Macos,
            OperatingSystem::Windows,
            OperatingSystem::Linux,
        ] {
            assert!(promotion(os).validate().is_ok());
        }
        let mut wrong_name = promotion(OperatingSystem::Windows);
        wrong_name.assets[0].file = "renamed.zip".to_string();
        assert_eq!(
            wrong_name.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidAssetSet)
        );
        let mut missing_cli = promotion(OperatingSystem::Linux);
        missing_cli.assets.pop();
        assert_eq!(
            missing_cli.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidAssetSet)
        );
        let mut missing_receipt = promotion(OperatingSystem::Macos);
        missing_receipt.evidence.pop();
        assert_eq!(
            missing_receipt.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidEvidenceSet)
        );
    }

    #[test]
    fn raw_ci_or_publishable_promotion_fails_closed() {
        let mut raw_ci = promotion(OperatingSystem::Macos);
        raw_ci.raw_ci_artifact_reused = true;
        assert_eq!(
            raw_ci.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidPromotion)
        );
        let mut publishing = promotion(OperatingSystem::Macos);
        publishing.publication_allowed = true;
        assert_eq!(
            publishing.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidPromotion)
        );
    }

    #[test]
    fn promotion_json_is_strict_bounded_and_canonical() {
        let promotion = promotion(OperatingSystem::Windows);
        let canonical = promotion.to_canonical_json().unwrap();
        assert_eq!(
            NativeCommunityAlphaTargetPromotionV1::from_json(&canonical).unwrap(),
            promotion
        );
        let mut unknown = serde_json::from_slice::<Value>(&canonical).unwrap();
        unknown["existing_ci_run_id"] = Value::String("123".to_string());
        assert_eq!(
            NativeCommunityAlphaTargetPromotionV1::from_json(
                &serde_json::to_vec(&unknown).unwrap()
            ),
            Err(NativeCommunityAlphaPromotionError::InvalidJson)
        );
    }

    #[test]
    fn candidate_set_closes_source_run_version_and_target_order() {
        let candidate = candidate();
        assert!(candidate.validate().is_ok());
        let canonical = candidate.to_canonical_json().unwrap();
        assert_eq!(
            NativeCommunityAlphaCandidateSetV1::from_json(&canonical).unwrap(),
            candidate
        );

        let mut wrong_order = candidate.clone();
        wrong_order.content.targets.swap(0, 1);
        assert_eq!(
            wrong_order.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidCandidateSet)
        );
        let mut wrong_source = candidate;
        wrong_source.content.targets[2].source_commit = "1".repeat(40);
        assert_eq!(
            wrong_source.validate(),
            Err(NativeCommunityAlphaPromotionError::InvalidCandidateSet)
        );
    }

    #[test]
    fn candidate_set_digest_detects_content_tampering() {
        let mut tampered = candidate();
        tampered.content.targets[1].assets[0].size_bytes += 1;
        assert_eq!(
            tampered.validate(),
            Err(NativeCommunityAlphaPromotionError::CandidateSetDigestMismatch)
        );
        let mut digest = candidate();
        digest.candidate_set_sha256 = "1".repeat(64);
        assert_eq!(
            digest.validate(),
            Err(NativeCommunityAlphaPromotionError::CandidateSetDigestMismatch)
        );
    }

    #[test]
    fn errors_are_fixed_and_path_free() {
        for error in [
            NativeCommunityAlphaPromotionError::InvalidAsset,
            NativeCommunityAlphaPromotionError::InvalidPromotion,
            NativeCommunityAlphaPromotionError::InvalidCandidateSet,
        ] {
            assert_eq!(error.to_string(), error.reason_code());
            assert!(!error.to_string().contains('/'));
            assert!(!error.to_string().contains('\\'));
        }
    }
}
