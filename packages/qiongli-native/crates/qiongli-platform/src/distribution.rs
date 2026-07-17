use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, OperatingSystem, ProductId,
    ReleaseChannel,
};

pub const NATIVE_DISTRIBUTION_POLICY_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_DISTRIBUTION_RELEASE_SET_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_PUBLICATION_AUTHORIZATION_SCHEMA_VERSION: u32 = 1;
pub const MAX_NATIVE_DISTRIBUTION_POLICY_BYTES: usize = 64 * 1024;
pub const MAX_NATIVE_DISTRIBUTION_RELEASE_SET_BYTES: usize = 256 * 1024;
pub const MAX_NATIVE_PUBLICATION_AUTHORIZATION_BYTES: usize = 64 * 1024;

const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_AUTHORIZATION_AGE_SECONDS: u64 = 86_400;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeDistributionClass {
    CommunityAlpha,
    Production,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativePlatformTrust {
    MacosAdHocNotNotarized,
    MacosDeveloperIdNotarized,
    WindowsUnsignedPortable,
    WindowsAuthenticodeTimestamped,
    LinuxAppimageQiongliMetadata,
    LinuxAppimageSignedMetadata,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeDistributionReleaseLabel {
    CommunityAlphaNotPlatformTrusted,
    ProductionPlatformTrusted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeDistributionWarning {
    CommunityAlphaNotPlatformTrusted,
    MacosOpenAnywayRequired,
    WindowsUnsignedMayBeBlocked,
    LinuxAppimageFacilitiesRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativePublicationAuthorizationRequirement {
    ExplicitExactReleaseSet,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativeDistributionPolicyRecordType {
    NativeDistributionPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeDistributionPolicyV1 {
    pub schema_version: u32,
    record_type: NativeDistributionPolicyRecordType,
    pub distribution_class: NativeDistributionClass,
    pub artifact: ArtifactIdentityV1,
    pub platform_trust: NativePlatformTrust,
    pub release_notes_label: NativeDistributionReleaseLabel,
    pub warnings: Vec<NativeDistributionWarning>,
    pub qiongli_release_envelope_required: bool,
    pub qiongli_update_metadata_signature_required: bool,
    pub sha256_inventory_required: bool,
    pub sbom_required: bool,
    pub provenance_required: bool,
    pub target_native_startup_required: bool,
    pub raw_ci_artifact_publishable: bool,
    pub stable_eligible: bool,
    pub publication_authorization: NativePublicationAuthorizationRequirement,
}

impl NativeDistributionPolicyV1 {
    pub fn for_artifact(
        distribution_class: NativeDistributionClass,
        artifact: ArtifactIdentityV1,
    ) -> Result<Self, NativeDistributionError> {
        let expected = expected_policy_fields(distribution_class, &artifact)?;
        let policy = Self {
            schema_version: NATIVE_DISTRIBUTION_POLICY_SCHEMA_VERSION,
            record_type: NativeDistributionPolicyRecordType::NativeDistributionPolicy,
            distribution_class,
            artifact,
            platform_trust: expected.platform_trust,
            release_notes_label: expected.release_notes_label,
            warnings: expected.warnings,
            qiongli_release_envelope_required: true,
            qiongli_update_metadata_signature_required: true,
            sha256_inventory_required: true,
            sbom_required: true,
            provenance_required: true,
            target_native_startup_required: true,
            raw_ci_artifact_publishable: false,
            stable_eligible: expected.stable_eligible,
            publication_authorization:
                NativePublicationAuthorizationRequirement::ExplicitExactReleaseSet,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> Result<(), NativeDistributionError> {
        if self.schema_version != NATIVE_DISTRIBUTION_POLICY_SCHEMA_VERSION
            || self.record_type != NativeDistributionPolicyRecordType::NativeDistributionPolicy
        {
            return Err(NativeDistributionError::UnsupportedSchema);
        }
        let expected = expected_policy_fields(self.distribution_class, &self.artifact)?;
        if self.platform_trust != expected.platform_trust
            || self.release_notes_label != expected.release_notes_label
            || self.warnings != expected.warnings
            || !self.qiongli_release_envelope_required
            || !self.qiongli_update_metadata_signature_required
            || !self.sha256_inventory_required
            || !self.sbom_required
            || !self.provenance_required
            || !self.target_native_startup_required
            || self.raw_ci_artifact_publishable
            || self.stable_eligible != expected.stable_eligible
            || self.publication_authorization
                != NativePublicationAuthorizationRequirement::ExplicitExactReleaseSet
        {
            return Err(NativeDistributionError::InvalidPolicy);
        }
        Ok(())
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeDistributionError> {
        self.validate()?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|_| NativeDistributionError::CanonicalSerializationFailed)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, NativeDistributionError> {
        if bytes.is_empty() || bytes.len() > MAX_NATIVE_DISTRIBUTION_POLICY_BYTES {
            return Err(NativeDistributionError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(bytes)
            .map_err(|_| NativeDistributionError::InvalidJson)?;
        value.validate()?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(NativeDistributionError::NonCanonicalJson);
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativeDistributionReleaseSetRecordType {
    NativeDistributionReleaseSet,
}

/// Closed policy record for one exact distribution release set.
///
/// The digest identifies the externally assembled public asset set; R3P-C is
/// responsible for computing and signing that asset inventory. This record
/// closes the distribution class and exact target matrix before a publication
/// authorization can be consumed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeDistributionReleaseSetV1 {
    pub schema_version: u32,
    record_type: NativeDistributionReleaseSetRecordType,
    pub distribution_class: NativeDistributionClass,
    pub source_commit: String,
    pub release_set_sha256: String,
    pub version: String,
    pub policies: Vec<NativeDistributionPolicyV1>,
}

impl NativeDistributionReleaseSetV1 {
    pub fn community_alpha(
        source_commit: impl Into<String>,
        release_set_sha256: impl Into<String>,
        version: impl Into<String>,
        policies: Vec<NativeDistributionPolicyV1>,
    ) -> Result<Self, NativeDistributionError> {
        let release_set = Self {
            schema_version: NATIVE_DISTRIBUTION_RELEASE_SET_SCHEMA_VERSION,
            record_type: NativeDistributionReleaseSetRecordType::NativeDistributionReleaseSet,
            distribution_class: NativeDistributionClass::CommunityAlpha,
            source_commit: source_commit.into(),
            release_set_sha256: release_set_sha256.into(),
            version: version.into(),
            policies,
        };
        release_set.validate()?;
        Ok(release_set)
    }

    pub fn validate(&self) -> Result<(), NativeDistributionError> {
        if self.schema_version != NATIVE_DISTRIBUTION_RELEASE_SET_SCHEMA_VERSION
            || self.record_type
                != NativeDistributionReleaseSetRecordType::NativeDistributionReleaseSet
            || self.distribution_class != NativeDistributionClass::CommunityAlpha
            || !valid_source_commit(&self.source_commit)
            || !is_lower_hex(&self.release_set_sha256, 64)
            || self.version.is_empty()
            || self.version.len() > 64
            || !self.version.is_ascii()
        {
            return Err(NativeDistributionError::InvalidReleaseSet);
        }
        let expected_targets = [
            (OperatingSystem::Macos, Architecture::Aarch64),
            (OperatingSystem::Windows, Architecture::X86_64),
            (OperatingSystem::Linux, Architecture::X86_64),
        ];
        if self.policies.len() != expected_targets.len() {
            return Err(NativeDistributionError::InvalidReleaseSet);
        }
        for (policy, (expected_os, expected_arch)) in self.policies.iter().zip(expected_targets) {
            policy
                .validate()
                .map_err(|_| NativeDistributionError::InvalidReleaseSet)?;
            if policy.distribution_class != self.distribution_class
                || policy.artifact.version != self.version
                || policy.artifact.os != expected_os
                || policy.artifact.arch != expected_arch
            {
                return Err(NativeDistributionError::InvalidReleaseSet);
            }
        }
        Ok(())
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeDistributionError> {
        self.validate()?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|_| NativeDistributionError::CanonicalSerializationFailed)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, NativeDistributionError> {
        if bytes.is_empty() || bytes.len() > MAX_NATIVE_DISTRIBUTION_RELEASE_SET_BYTES {
            return Err(NativeDistributionError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(bytes)
            .map_err(|_| NativeDistributionError::InvalidJson)?;
        value.validate()?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(NativeDistributionError::NonCanonicalJson);
        }
        Ok(value)
    }
}

struct ExpectedPolicyFields {
    platform_trust: NativePlatformTrust,
    release_notes_label: NativeDistributionReleaseLabel,
    warnings: Vec<NativeDistributionWarning>,
    stable_eligible: bool,
}

fn expected_policy_fields(
    distribution_class: NativeDistributionClass,
    artifact: &ArtifactIdentityV1,
) -> Result<ExpectedPolicyFields, NativeDistributionError> {
    artifact
        .validate()
        .map_err(|_| NativeDistributionError::InvalidPolicy)?;
    if artifact.product != ProductId::Qiongli
        || artifact.installer_kind != InstallerKind::NativeInstaller
        || artifact.profile == CapabilityProfile::SkillOnly
    {
        return Err(NativeDistributionError::InvalidPolicy);
    }
    match distribution_class {
        NativeDistributionClass::CommunityAlpha => community_alpha_fields(artifact),
        NativeDistributionClass::Production => production_fields(artifact),
    }
}

fn community_alpha_fields(
    artifact: &ArtifactIdentityV1,
) -> Result<ExpectedPolicyFields, NativeDistributionError> {
    if artifact.channel != ReleaseChannel::Alpha || artifact.profile != CapabilityProfile::Lite {
        return Err(NativeDistributionError::DistributionClassUnavailable);
    }
    let (platform_trust, platform_warning) = match (artifact.os, artifact.arch) {
        (OperatingSystem::Macos, Architecture::Aarch64) => (
            NativePlatformTrust::MacosAdHocNotNotarized,
            NativeDistributionWarning::MacosOpenAnywayRequired,
        ),
        (OperatingSystem::Windows, Architecture::X86_64) => (
            NativePlatformTrust::WindowsUnsignedPortable,
            NativeDistributionWarning::WindowsUnsignedMayBeBlocked,
        ),
        (OperatingSystem::Linux, Architecture::X86_64) => (
            NativePlatformTrust::LinuxAppimageQiongliMetadata,
            NativeDistributionWarning::LinuxAppimageFacilitiesRequired,
        ),
        _ => return Err(NativeDistributionError::TargetUnavailable),
    };
    Ok(ExpectedPolicyFields {
        platform_trust,
        release_notes_label: NativeDistributionReleaseLabel::CommunityAlphaNotPlatformTrusted,
        warnings: vec![
            NativeDistributionWarning::CommunityAlphaNotPlatformTrusted,
            platform_warning,
        ],
        stable_eligible: false,
    })
}

fn production_fields(
    artifact: &ArtifactIdentityV1,
) -> Result<ExpectedPolicyFields, NativeDistributionError> {
    if artifact.channel == ReleaseChannel::Alpha {
        return Err(NativeDistributionError::DistributionClassUnavailable);
    }
    let platform_trust = match (artifact.os, artifact.arch) {
        (OperatingSystem::Macos, Architecture::Aarch64) => {
            NativePlatformTrust::MacosDeveloperIdNotarized
        }
        (OperatingSystem::Windows, Architecture::X86_64) => {
            NativePlatformTrust::WindowsAuthenticodeTimestamped
        }
        (OperatingSystem::Linux, Architecture::X86_64) => {
            NativePlatformTrust::LinuxAppimageSignedMetadata
        }
        _ => return Err(NativeDistributionError::TargetUnavailable),
    };
    Ok(ExpectedPolicyFields {
        platform_trust,
        release_notes_label: NativeDistributionReleaseLabel::ProductionPlatformTrusted,
        warnings: Vec::new(),
        stable_eligible: artifact.channel == ReleaseChannel::Stable,
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativePublicationAuthorizationAuthority {
    GithubProtectedEnvironment,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativePublicationAuthorizationDecision {
    AuthorizeExactReleaseSet,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NativePublicationAuthorizationRecordType {
    NativePublicationAuthorization,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativePublicationAuthorizationV1 {
    pub schema_version: u32,
    record_type: NativePublicationAuthorizationRecordType,
    pub authority: NativePublicationAuthorizationAuthority,
    pub decision: NativePublicationAuthorizationDecision,
    pub distribution_class: NativeDistributionClass,
    pub source_commit: String,
    pub release_set_sha256: String,
    pub repository: String,
    pub environment: String,
    pub workflow_run_url: String,
    pub authorized_by: String,
    pub authorized_at_unix: u64,
}

impl NativePublicationAuthorizationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn exact_release_set(
        distribution_class: NativeDistributionClass,
        source_commit: impl Into<String>,
        release_set_sha256: impl Into<String>,
        repository: impl Into<String>,
        environment: impl Into<String>,
        workflow_run_url: impl Into<String>,
        authorized_by: impl Into<String>,
        authorized_at_unix: u64,
    ) -> Result<Self, NativeDistributionError> {
        let authorization = Self {
            schema_version: NATIVE_PUBLICATION_AUTHORIZATION_SCHEMA_VERSION,
            record_type: NativePublicationAuthorizationRecordType::NativePublicationAuthorization,
            authority: NativePublicationAuthorizationAuthority::GithubProtectedEnvironment,
            decision: NativePublicationAuthorizationDecision::AuthorizeExactReleaseSet,
            distribution_class,
            source_commit: source_commit.into(),
            release_set_sha256: release_set_sha256.into(),
            repository: repository.into(),
            environment: environment.into(),
            workflow_run_url: workflow_run_url.into(),
            authorized_by: authorized_by.into(),
            authorized_at_unix,
        };
        authorization.validate_shape()?;
        Ok(authorization)
    }

    fn validate_shape(&self) -> Result<(), NativeDistributionError> {
        if self.schema_version != NATIVE_PUBLICATION_AUTHORIZATION_SCHEMA_VERSION
            || self.record_type
                != NativePublicationAuthorizationRecordType::NativePublicationAuthorization
        {
            return Err(NativeDistributionError::UnsupportedSchema);
        }
        if self.authority != NativePublicationAuthorizationAuthority::GithubProtectedEnvironment
            || self.decision != NativePublicationAuthorizationDecision::AuthorizeExactReleaseSet
            || !valid_source_commit(&self.source_commit)
            || !is_lower_hex(&self.release_set_sha256, 64)
            || !valid_repository(&self.repository)
            || !valid_label(&self.environment, 64)
            || !valid_label(&self.authorized_by, 64)
            || !valid_workflow_run_url(&self.workflow_run_url, &self.repository)
            || self.authorized_at_unix == 0
            || self.authorized_at_unix > JCS_MAX_SAFE_INTEGER
        {
            return Err(NativeDistributionError::InvalidAuthorization);
        }
        Ok(())
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeDistributionError> {
        self.validate_shape()?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|_| NativeDistributionError::CanonicalSerializationFailed)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, NativeDistributionError> {
        if bytes.is_empty() || bytes.len() > MAX_NATIVE_PUBLICATION_AUTHORIZATION_BYTES {
            return Err(NativeDistributionError::InputTooLarge);
        }
        let value = serde_json::from_slice::<Self>(bytes)
            .map_err(|_| NativeDistributionError::InvalidJson)?;
        value.validate_shape()?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(NativeDistributionError::NonCanonicalJson);
        }
        Ok(value)
    }
}

impl Debug for NativePublicationAuthorizationV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativePublicationAuthorizationV1")
            .field("distribution_class", &self.distribution_class)
            .field("source_commit", &self.source_commit)
            .field("release_set_sha256", &self.release_set_sha256)
            .field("repository", &self.repository)
            .field("environment", &self.environment)
            .field("workflow_run_url", &self.workflow_run_url)
            .field("authorized_by", &self.authorized_by)
            .field("authorized_at_unix", &self.authorized_at_unix)
            .finish_non_exhaustive()
    }
}

pub struct NativePublicationAuthorizationContext<'a> {
    pub expected_distribution_class: NativeDistributionClass,
    pub expected_source_commit: &'a str,
    pub expected_release_set_sha256: &'a str,
    pub expected_repository: &'a str,
    pub expected_environment: &'a str,
    pub expected_workflow_run_url: &'a str,
    pub expected_actor: &'a str,
    pub verified_at_unix: u64,
    pub max_authorization_age_seconds: u64,
}

pub struct VerifiedNativePublicationAuthorization {
    authorization: NativePublicationAuthorizationV1,
    verified_at_unix: u64,
}

impl VerifiedNativePublicationAuthorization {
    #[must_use]
    pub const fn authorization(&self) -> &NativePublicationAuthorizationV1 {
        &self.authorization
    }

    #[must_use]
    pub const fn verified_at_unix(&self) -> u64 {
        self.verified_at_unix
    }
}

impl Debug for VerifiedNativePublicationAuthorization {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedNativePublicationAuthorization")
            .field("distribution_class", &self.authorization.distribution_class)
            .field("source_commit", &self.authorization.source_commit)
            .field("release_set_sha256", &self.authorization.release_set_sha256)
            .field("authorized_by", &self.authorization.authorized_by)
            .field("verified_at_unix", &self.verified_at_unix)
            .finish()
    }
}

pub struct VerifiedNativeDistributionReleaseSet {
    release_set: NativeDistributionReleaseSetV1,
    authorization: VerifiedNativePublicationAuthorization,
}

impl VerifiedNativeDistributionReleaseSet {
    #[must_use]
    pub const fn release_set(&self) -> &NativeDistributionReleaseSetV1 {
        &self.release_set
    }

    #[must_use]
    pub const fn authorization(&self) -> &VerifiedNativePublicationAuthorization {
        &self.authorization
    }
}

impl Debug for VerifiedNativeDistributionReleaseSet {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedNativeDistributionReleaseSet")
            .field("distribution_class", &self.release_set.distribution_class)
            .field("source_commit", &self.release_set.source_commit)
            .field("release_set_sha256", &self.release_set.release_set_sha256)
            .field("version", &self.release_set.version)
            .field("policy_count", &self.release_set.policies.len())
            .field("authorization", &self.authorization)
            .finish()
    }
}

pub fn verify_native_publication_authorization(
    authorization: NativePublicationAuthorizationV1,
    context: &NativePublicationAuthorizationContext<'_>,
) -> Result<VerifiedNativePublicationAuthorization, NativeDistributionError> {
    authorization.validate_shape()?;
    if !valid_authorization_context(context) {
        return Err(NativeDistributionError::InvalidAuthorizationContext);
    }
    if authorization.distribution_class != context.expected_distribution_class
        || authorization.source_commit != context.expected_source_commit
        || authorization.release_set_sha256 != context.expected_release_set_sha256
        || authorization.repository != context.expected_repository
        || authorization.environment != context.expected_environment
        || authorization.workflow_run_url != context.expected_workflow_run_url
        || authorization.authorized_by != context.expected_actor
        || authorization.authorized_at_unix > context.verified_at_unix
        || context
            .verified_at_unix
            .saturating_sub(authorization.authorized_at_unix)
            > context.max_authorization_age_seconds
    {
        return Err(NativeDistributionError::AuthorizationMismatch);
    }
    Ok(VerifiedNativePublicationAuthorization {
        authorization,
        verified_at_unix: context.verified_at_unix,
    })
}

pub fn verify_native_distribution_release_set_authorization(
    release_set: NativeDistributionReleaseSetV1,
    authorization: NativePublicationAuthorizationV1,
    context: &NativePublicationAuthorizationContext<'_>,
) -> Result<VerifiedNativeDistributionReleaseSet, NativeDistributionError> {
    release_set.validate()?;
    if context.expected_distribution_class != release_set.distribution_class
        || context.expected_source_commit != release_set.source_commit
        || context.expected_release_set_sha256 != release_set.release_set_sha256
    {
        return Err(NativeDistributionError::AuthorizationMismatch);
    }
    let authorization = verify_native_publication_authorization(authorization, context)?;
    Ok(VerifiedNativeDistributionReleaseSet {
        release_set,
        authorization,
    })
}

fn valid_authorization_context(context: &NativePublicationAuthorizationContext<'_>) -> bool {
    valid_source_commit(context.expected_source_commit)
        && is_lower_hex(context.expected_release_set_sha256, 64)
        && valid_repository(context.expected_repository)
        && valid_label(context.expected_environment, 64)
        && valid_workflow_run_url(
            context.expected_workflow_run_url,
            context.expected_repository,
        )
        && valid_label(context.expected_actor, 64)
        && context.verified_at_unix > 0
        && context.verified_at_unix <= JCS_MAX_SAFE_INTEGER
        && context.max_authorization_age_seconds > 0
        && context.max_authorization_age_seconds <= MAX_AUTHORIZATION_AGE_SECONDS
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn valid_repository(value: &str) -> bool {
    let mut components = value.split('/');
    let owner = components.next().unwrap_or_default();
    let repository = components.next().unwrap_or_default();
    components.next().is_none() && valid_label(owner, 64) && valid_label(repository, 100)
}

fn valid_label(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_workflow_run_url(value: &str, repository: &str) -> bool {
    value
        .strip_prefix("https://github.com/")
        .and_then(|suffix| suffix.strip_prefix(repository))
        .and_then(|suffix| suffix.strip_prefix("/actions/runs/"))
        .is_some_and(|run_id| {
            !run_id.is_empty()
                && run_id.len() <= 20
                && run_id.bytes().all(|byte| byte.is_ascii_digit())
                && !run_id.starts_with('0')
        })
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeDistributionError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    UnsupportedSchema,
    InvalidPolicy,
    DistributionClassUnavailable,
    TargetUnavailable,
    InvalidReleaseSet,
    InvalidAuthorization,
    InvalidAuthorizationContext,
    AuthorizationMismatch,
    CanonicalSerializationFailed,
}

impl NativeDistributionError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "native-distribution-input-too-large",
            Self::InvalidJson => "native-distribution-json-invalid",
            Self::NonCanonicalJson => "native-distribution-json-noncanonical",
            Self::UnsupportedSchema => "native-distribution-schema-unsupported",
            Self::InvalidPolicy => "native-distribution-policy-invalid",
            Self::DistributionClassUnavailable => "native-distribution-class-unavailable",
            Self::TargetUnavailable => "native-distribution-target-unavailable",
            Self::InvalidReleaseSet => "native-distribution-release-set-invalid",
            Self::InvalidAuthorization => "native-publication-authorization-invalid",
            Self::InvalidAuthorizationContext => "native-publication-authorization-context-invalid",
            Self::AuthorizationMismatch => "native-publication-authorization-mismatch",
            Self::CanonicalSerializationFailed => {
                "native-distribution-canonical-serialization-failed"
            }
        }
    }
}

impl Display for NativeDistributionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeDistributionError {}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const RELEASE_SET_SHA256: &str =
        "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    const NOW: u64 = 1_800_000_000;

    fn artifact(
        version: &str,
        channel: ReleaseChannel,
        os: OperatingSystem,
        arch: Architecture,
    ) -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: version.to_string(),
            channel,
            profile: CapabilityProfile::Lite,
            os,
            arch,
            installer_kind: InstallerKind::NativeInstaller,
        }
    }

    fn community_alpha_release_set() -> NativeDistributionReleaseSetV1 {
        let policies = [
            (OperatingSystem::Macos, Architecture::Aarch64),
            (OperatingSystem::Windows, Architecture::X86_64),
            (OperatingSystem::Linux, Architecture::X86_64),
        ]
        .into_iter()
        .map(|(os, arch)| {
            NativeDistributionPolicyV1::for_artifact(
                NativeDistributionClass::CommunityAlpha,
                artifact("2.0.0-alpha.1", ReleaseChannel::Alpha, os, arch),
            )
            .unwrap()
        })
        .collect();
        NativeDistributionReleaseSetV1::community_alpha(
            SOURCE_COMMIT,
            RELEASE_SET_SHA256,
            "2.0.0-alpha.1",
            policies,
        )
        .unwrap()
    }

    fn authorization() -> NativePublicationAuthorizationV1 {
        NativePublicationAuthorizationV1::exact_release_set(
            NativeDistributionClass::CommunityAlpha,
            SOURCE_COMMIT,
            RELEASE_SET_SHA256,
            "jxpeng98/qiongli",
            "community-alpha",
            "https://github.com/jxpeng98/qiongli/actions/runs/29575237942",
            "jxpeng98",
            NOW - 60,
        )
        .unwrap()
    }

    fn authorization_context() -> NativePublicationAuthorizationContext<'static> {
        NativePublicationAuthorizationContext {
            expected_distribution_class: NativeDistributionClass::CommunityAlpha,
            expected_source_commit: SOURCE_COMMIT,
            expected_release_set_sha256: RELEASE_SET_SHA256,
            expected_repository: "jxpeng98/qiongli",
            expected_environment: "community-alpha",
            expected_workflow_run_url: "https://github.com/jxpeng98/qiongli/actions/runs/29575237942",
            expected_actor: "jxpeng98",
            verified_at_unix: NOW,
            max_authorization_age_seconds: 600,
        }
    }

    #[test]
    fn community_alpha_policy_closes_the_three_free_targets() {
        let cases = [
            (
                OperatingSystem::Macos,
                Architecture::Aarch64,
                NativePlatformTrust::MacosAdHocNotNotarized,
                NativeDistributionWarning::MacosOpenAnywayRequired,
            ),
            (
                OperatingSystem::Windows,
                Architecture::X86_64,
                NativePlatformTrust::WindowsUnsignedPortable,
                NativeDistributionWarning::WindowsUnsignedMayBeBlocked,
            ),
            (
                OperatingSystem::Linux,
                Architecture::X86_64,
                NativePlatformTrust::LinuxAppimageQiongliMetadata,
                NativeDistributionWarning::LinuxAppimageFacilitiesRequired,
            ),
        ];
        for (os, arch, expected_trust, expected_warning) in cases {
            let policy = NativeDistributionPolicyV1::for_artifact(
                NativeDistributionClass::CommunityAlpha,
                artifact("2.0.0-alpha.1", ReleaseChannel::Alpha, os, arch),
            )
            .unwrap();
            assert_eq!(policy.platform_trust, expected_trust);
            assert_eq!(
                policy.release_notes_label,
                NativeDistributionReleaseLabel::CommunityAlphaNotPlatformTrusted
            );
            assert_eq!(
                policy.warnings,
                vec![
                    NativeDistributionWarning::CommunityAlphaNotPlatformTrusted,
                    expected_warning
                ]
            );
            assert!(!policy.raw_ci_artifact_publishable);
            assert!(!policy.stable_eligible);
            let canonical = policy.to_canonical_json().unwrap();
            assert_eq!(
                NativeDistributionPolicyV1::from_json(&canonical).unwrap(),
                policy
            );
        }
    }

    #[test]
    fn community_alpha_rejects_wrong_channel_profile_kind_and_target() {
        let beta = artifact(
            "2.0.0-beta.1",
            ReleaseChannel::Beta,
            OperatingSystem::Linux,
            Architecture::X86_64,
        );
        assert_eq!(
            NativeDistributionPolicyV1::for_artifact(NativeDistributionClass::CommunityAlpha, beta),
            Err(NativeDistributionError::DistributionClassUnavailable)
        );

        let mut skill_only = artifact(
            "2.0.0-alpha.1",
            ReleaseChannel::Alpha,
            OperatingSystem::Linux,
            Architecture::X86_64,
        );
        skill_only.profile = CapabilityProfile::SkillOnly;
        assert_eq!(
            NativeDistributionPolicyV1::for_artifact(
                NativeDistributionClass::CommunityAlpha,
                skill_only
            ),
            Err(NativeDistributionError::InvalidPolicy)
        );

        let mut portable = artifact(
            "2.0.0-alpha.1",
            ReleaseChannel::Alpha,
            OperatingSystem::Windows,
            Architecture::X86_64,
        );
        portable.installer_kind = InstallerKind::PortableArchive;
        assert_eq!(
            NativeDistributionPolicyV1::for_artifact(
                NativeDistributionClass::CommunityAlpha,
                portable
            ),
            Err(NativeDistributionError::InvalidPolicy)
        );

        let intel_macos = artifact(
            "2.0.0-alpha.1",
            ReleaseChannel::Alpha,
            OperatingSystem::Macos,
            Architecture::X86_64,
        );
        assert_eq!(
            NativeDistributionPolicyV1::for_artifact(
                NativeDistributionClass::CommunityAlpha,
                intel_macos
            ),
            Err(NativeDistributionError::TargetUnavailable)
        );
    }

    #[test]
    fn production_policy_cannot_silently_relabel_community_alpha() {
        let alpha = artifact(
            "2.0.0-alpha.1",
            ReleaseChannel::Alpha,
            OperatingSystem::Macos,
            Architecture::Aarch64,
        );
        assert_eq!(
            NativeDistributionPolicyV1::for_artifact(NativeDistributionClass::Production, alpha),
            Err(NativeDistributionError::DistributionClassUnavailable)
        );

        let beta = NativeDistributionPolicyV1::for_artifact(
            NativeDistributionClass::Production,
            artifact(
                "2.0.0-beta.1",
                ReleaseChannel::Beta,
                OperatingSystem::Windows,
                Architecture::X86_64,
            ),
        )
        .unwrap();
        assert_eq!(
            beta.platform_trust,
            NativePlatformTrust::WindowsAuthenticodeTimestamped
        );
        assert!(!beta.stable_eligible);

        let stable = NativeDistributionPolicyV1::for_artifact(
            NativeDistributionClass::Production,
            artifact(
                "2.0.0",
                ReleaseChannel::Stable,
                OperatingSystem::Linux,
                Architecture::X86_64,
            ),
        )
        .unwrap();
        assert!(stable.stable_eligible);
    }

    #[test]
    fn policy_tampering_fails_closed() {
        let policy = NativeDistributionPolicyV1::for_artifact(
            NativeDistributionClass::CommunityAlpha,
            artifact(
                "2.0.0-alpha.1",
                ReleaseChannel::Alpha,
                OperatingSystem::Windows,
                Architecture::X86_64,
            ),
        )
        .unwrap();
        let mut value = serde_json::to_value(&policy).unwrap();
        value["raw_ci_artifact_publishable"] = Value::Bool(true);
        let tampered = serde_json::from_value::<NativeDistributionPolicyV1>(value).unwrap();
        assert_eq!(
            tampered.validate(),
            Err(NativeDistributionError::InvalidPolicy)
        );

        let mut noncanonical = policy.to_canonical_json().unwrap();
        noncanonical.push(b'\n');
        assert_eq!(
            NativeDistributionPolicyV1::from_json(&noncanonical),
            Err(NativeDistributionError::NonCanonicalJson)
        );
        assert_eq!(
            NativeDistributionPolicyV1::from_json(&vec![b' '; 65 * 1024]),
            Err(NativeDistributionError::InputTooLarge)
        );
    }

    #[test]
    fn community_alpha_release_set_requires_the_exact_target_matrix() {
        let release_set = community_alpha_release_set();
        let canonical = release_set.to_canonical_json().unwrap();
        assert_eq!(
            NativeDistributionReleaseSetV1::from_json(&canonical).unwrap(),
            release_set
        );

        let mut wrong_order = release_set.clone();
        wrong_order.policies.swap(0, 1);
        assert_eq!(
            wrong_order.validate(),
            Err(NativeDistributionError::InvalidReleaseSet)
        );

        let mut missing_target = release_set.clone();
        missing_target.policies.pop();
        assert_eq!(
            missing_target.validate(),
            Err(NativeDistributionError::InvalidReleaseSet)
        );

        let mut wrong_version = release_set;
        wrong_version.version = "2.0.0-alpha.2".to_string();
        assert_eq!(
            wrong_version.validate(),
            Err(NativeDistributionError::InvalidReleaseSet)
        );
    }

    #[test]
    fn release_set_json_is_strict_bounded_and_canonical() {
        let canonical = community_alpha_release_set().to_canonical_json().unwrap();
        let mut unknown = serde_json::from_slice::<Value>(&canonical).unwrap();
        unknown["publication_allowed"] = Value::Bool(true);
        assert_eq!(
            NativeDistributionReleaseSetV1::from_json(&serde_json::to_vec(&unknown).unwrap()),
            Err(NativeDistributionError::InvalidJson)
        );

        let mut noncanonical = canonical;
        noncanonical.push(b'\n');
        assert_eq!(
            NativeDistributionReleaseSetV1::from_json(&noncanonical),
            Err(NativeDistributionError::NonCanonicalJson)
        );
        assert_eq!(
            NativeDistributionReleaseSetV1::from_json(&vec![
                b' ';
                MAX_NATIVE_DISTRIBUTION_RELEASE_SET_BYTES
                    + 1
            ]),
            Err(NativeDistributionError::InputTooLarge)
        );
    }

    #[test]
    fn release_set_authorization_is_consumed_only_for_its_exact_set() {
        let verified = verify_native_distribution_release_set_authorization(
            community_alpha_release_set(),
            authorization(),
            &authorization_context(),
        )
        .unwrap();
        assert_eq!(
            verified.release_set().release_set_sha256,
            RELEASE_SET_SHA256
        );
        assert_eq!(verified.authorization().verified_at_unix(), NOW);

        let mut different_set = community_alpha_release_set();
        different_set.release_set_sha256 =
            "1111111111111111111111111111111111111111111111111111111111111111".to_string();
        assert_eq!(
            verify_native_distribution_release_set_authorization(
                different_set,
                authorization(),
                &authorization_context()
            )
            .unwrap_err(),
            NativeDistributionError::AuthorizationMismatch
        );
    }

    #[test]
    fn protected_environment_authorization_is_exact_and_time_bounded() {
        let verified =
            verify_native_publication_authorization(authorization(), &authorization_context())
                .unwrap();
        assert_eq!(verified.verified_at_unix(), NOW);
        assert_eq!(
            verified.authorization().release_set_sha256,
            RELEASE_SET_SHA256
        );

        let canonical = verified.authorization().to_canonical_json().unwrap();
        assert_eq!(
            NativePublicationAuthorizationV1::from_json(&canonical).unwrap(),
            *verified.authorization()
        );
    }

    #[test]
    fn authorization_mismatch_future_and_stale_records_fail_closed() {
        let mut context = authorization_context();
        context.expected_release_set_sha256 =
            "1111111111111111111111111111111111111111111111111111111111111111";
        assert_eq!(
            verify_native_publication_authorization(authorization(), &context).unwrap_err(),
            NativeDistributionError::AuthorizationMismatch
        );

        let future = NativePublicationAuthorizationV1::exact_release_set(
            NativeDistributionClass::CommunityAlpha,
            SOURCE_COMMIT,
            RELEASE_SET_SHA256,
            "jxpeng98/qiongli",
            "community-alpha",
            "https://github.com/jxpeng98/qiongli/actions/runs/29575237942",
            "jxpeng98",
            NOW + 1,
        )
        .unwrap();
        assert_eq!(
            verify_native_publication_authorization(future, &authorization_context()).unwrap_err(),
            NativeDistributionError::AuthorizationMismatch
        );

        let stale = NativePublicationAuthorizationV1::exact_release_set(
            NativeDistributionClass::CommunityAlpha,
            SOURCE_COMMIT,
            RELEASE_SET_SHA256,
            "jxpeng98/qiongli",
            "community-alpha",
            "https://github.com/jxpeng98/qiongli/actions/runs/29575237942",
            "jxpeng98",
            NOW - 601,
        )
        .unwrap();
        assert_eq!(
            verify_native_publication_authorization(stale, &authorization_context()).unwrap_err(),
            NativeDistributionError::AuthorizationMismatch
        );
    }

    #[test]
    fn authorization_json_is_strict_and_unknown_fields_are_rejected() {
        let authorization = authorization();
        let canonical = authorization.to_canonical_json().unwrap();
        let mut unknown = serde_json::from_slice::<Value>(&canonical).unwrap();
        unknown["publication_allowed"] = Value::Bool(true);
        assert_eq!(
            NativePublicationAuthorizationV1::from_json(&serde_json::to_vec(&unknown).unwrap()),
            Err(NativeDistributionError::InvalidJson)
        );
        let mut noncanonical = canonical;
        noncanonical.push(b'\n');
        assert_eq!(
            NativePublicationAuthorizationV1::from_json(&noncanonical),
            Err(NativeDistributionError::NonCanonicalJson)
        );
    }

    #[test]
    fn error_output_is_fixed_and_path_free() {
        for error in [
            NativeDistributionError::InvalidPolicy,
            NativeDistributionError::TargetUnavailable,
            NativeDistributionError::InvalidReleaseSet,
            NativeDistributionError::InvalidAuthorization,
            NativeDistributionError::AuthorizationMismatch,
        ] {
            assert_eq!(error.to_string(), error.reason_code());
            assert!(!error.to_string().contains('/'));
            assert!(!error.to_string().contains('\\'));
        }
    }
}
