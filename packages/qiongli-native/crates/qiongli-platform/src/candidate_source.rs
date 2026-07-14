use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use qiongli_content::LoadedResourcePack;

use crate::claude::prepare_claude_plugin_source_target;
use crate::codex::prepare_codex_plugin_source_target;
use crate::{
    ArtifactIdentityV1, ClaudeAdapterError, ClaudePluginBundleError, ClaudePluginBundleTarget,
    ClientActivationTarget, CodexAdapterError, CodexPluginBundleError, CodexPluginBundleTarget,
    VerifiedNativeReleaseCandidate, compose_claude_plugin_bundle, compose_codex_plugin_bundle,
    remove_claude_plugin_bundle, remove_codex_plugin_bundle, verify_claude_plugin_bundle,
    verify_codex_plugin_bundle,
};

#[derive(Clone, Debug)]
pub struct NativeCandidatePluginSourceTarget {
    inner: NativeCandidatePluginSourceTargetKind,
}

#[derive(Clone, Debug)]
enum NativeCandidatePluginSourceTargetKind {
    Codex(CodexPluginBundleTarget),
    ClaudeCode(ClaudePluginBundleTarget),
}

impl NativeCandidatePluginSourceTarget {
    #[must_use]
    pub const fn target(&self) -> ClientActivationTarget {
        match &self.inner {
            NativeCandidatePluginSourceTargetKind::Codex(_) => ClientActivationTarget::Codex,
            NativeCandidatePluginSourceTargetKind::ClaudeCode(_) => {
                ClientActivationTarget::ClaudeCode
            }
        }
    }

    fn path(&self) -> &Path {
        match &self.inner {
            NativeCandidatePluginSourceTargetKind::Codex(target) => target.path(),
            NativeCandidatePluginSourceTargetKind::ClaudeCode(target) => target.path(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeCandidatePluginSourceDisposition {
    Materialized,
    AlreadyHealthy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCandidatePluginSourceVerification {
    pub target: ClientActivationTarget,
    pub artifact: ArtifactIdentityV1,
    pub signed_grant_payload_sha256: String,
    pub receipt_sha256: String,
    pub package_content_root_sha256: String,
    pub binary_sha256: String,
    pub resource_pack_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCandidatePluginSourceCommit {
    pub disposition: NativeCandidatePluginSourceDisposition,
    pub verification: NativeCandidatePluginSourceVerification,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeCandidatePluginSourceError {
    TargetMismatch,
    SourceIdentityMismatch,
    CodexAdapter(CodexAdapterError),
    ClaudeAdapter(ClaudeAdapterError),
    CodexBundle(CodexPluginBundleError),
    ClaudeBundle(ClaudePluginBundleError),
}

impl NativeCandidatePluginSourceError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::TargetMismatch => "native-candidate-plugin-target-mismatch",
            Self::SourceIdentityMismatch => "native-candidate-plugin-source-mismatch",
            Self::CodexAdapter(error) => error.reason_code(),
            Self::ClaudeAdapter(error) => error.reason_code(),
            Self::CodexBundle(error) => error.reason_code(),
            Self::ClaudeBundle(error) => error.reason_code(),
        }
    }
}

impl Display for NativeCandidatePluginSourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeCandidatePluginSourceError {}

/// Creates only the fixed Qiongli-owned parent chain for one local plugin source.
///
/// The supplied home boundary must already exist and pass the target adapter's
/// ownership and traversal checks. No caller-selected source path is accepted.
pub fn prepare_native_candidate_plugin_source_target(
    home: impl AsRef<Path>,
    target: ClientActivationTarget,
) -> Result<NativeCandidatePluginSourceTarget, NativeCandidatePluginSourceError> {
    match target {
        ClientActivationTarget::Codex => prepare_codex_plugin_source_target(home)
            .map(|inner| NativeCandidatePluginSourceTarget {
                inner: NativeCandidatePluginSourceTargetKind::Codex(inner),
            })
            .map_err(NativeCandidatePluginSourceError::CodexAdapter),
        ClientActivationTarget::ClaudeCode => prepare_claude_plugin_source_target(home)
            .map(|inner| NativeCandidatePluginSourceTarget {
                inner: NativeCandidatePluginSourceTargetKind::ClaudeCode(inner),
            })
            .map_err(NativeCandidatePluginSourceError::ClaudeAdapter),
    }
}

/// Materializes or replays one exact target source from a verified candidate.
pub fn materialize_native_candidate_plugin_source(
    pack: &LoadedResourcePack<'_>,
    candidate: &VerifiedNativeReleaseCandidate,
    source_binary: impl AsRef<Path>,
    target: &NativeCandidatePluginSourceTarget,
) -> Result<NativeCandidatePluginSourceCommit, NativeCandidatePluginSourceError> {
    validate_candidate_target(candidate, target)?;
    let existed = path_exists(target)?;
    let verification = if existed {
        verify_native_candidate_plugin_source(target)?
    } else {
        match &target.inner {
            NativeCandidatePluginSourceTargetKind::Codex(target) => {
                let bundle = compose_codex_plugin_bundle(
                    pack,
                    candidate.plugin_grant(),
                    source_binary,
                    target,
                )
                .map_err(NativeCandidatePluginSourceError::CodexBundle)?;
                codex_verification(&bundle)
            }
            NativeCandidatePluginSourceTargetKind::ClaudeCode(target) => {
                let bundle = compose_claude_plugin_bundle(
                    pack,
                    candidate.plugin_grant(),
                    source_binary,
                    target,
                )
                .map_err(NativeCandidatePluginSourceError::ClaudeBundle)?;
                claude_verification(&bundle)
            }
        }
    };
    validate_candidate_source(candidate, &verification)?;
    Ok(NativeCandidatePluginSourceCommit {
        disposition: if existed {
            NativeCandidatePluginSourceDisposition::AlreadyHealthy
        } else {
            NativeCandidatePluginSourceDisposition::Materialized
        },
        verification,
    })
}

/// Verifies one complete receipt-backed source without requiring unexpired release inputs.
pub fn verify_native_candidate_plugin_source(
    target: &NativeCandidatePluginSourceTarget,
) -> Result<NativeCandidatePluginSourceVerification, NativeCandidatePluginSourceError> {
    match &target.inner {
        NativeCandidatePluginSourceTargetKind::Codex(target) => verify_codex_plugin_bundle(target)
            .map(|bundle| codex_verification(&bundle))
            .map_err(NativeCandidatePluginSourceError::CodexBundle),
        NativeCandidatePluginSourceTargetKind::ClaudeCode(target) => {
            verify_claude_plugin_bundle(target)
                .map(|bundle| claude_verification(&bundle))
                .map_err(NativeCandidatePluginSourceError::ClaudeBundle)
        }
    }
}

/// Removes only an exact receipt-verified source and is independent of candidate expiry.
pub fn remove_native_candidate_plugin_source(
    target: &NativeCandidatePluginSourceTarget,
) -> Result<NativeCandidatePluginSourceVerification, NativeCandidatePluginSourceError> {
    match &target.inner {
        NativeCandidatePluginSourceTargetKind::Codex(target) => remove_codex_plugin_bundle(target)
            .map(|bundle| codex_verification(&bundle))
            .map_err(NativeCandidatePluginSourceError::CodexBundle),
        NativeCandidatePluginSourceTargetKind::ClaudeCode(target) => {
            remove_claude_plugin_bundle(target)
                .map(|bundle| claude_verification(&bundle))
                .map_err(NativeCandidatePluginSourceError::ClaudeBundle)
        }
    }
}

fn validate_candidate_target(
    candidate: &VerifiedNativeReleaseCandidate,
    target: &NativeCandidatePluginSourceTarget,
) -> Result<(), NativeCandidatePluginSourceError> {
    if candidate.target() != target.target() {
        return Err(NativeCandidatePluginSourceError::TargetMismatch);
    }
    Ok(())
}

fn validate_candidate_source(
    candidate: &VerifiedNativeReleaseCandidate,
    verification: &NativeCandidatePluginSourceVerification,
) -> Result<(), NativeCandidatePluginSourceError> {
    let grant = candidate.plugin_grant();
    if verification.target != candidate.target()
        || verification.artifact != grant.grant().artifact
        || verification.signed_grant_payload_sha256 != grant.signed_payload_sha256()
        || verification.binary_sha256 != grant.grant().binary_sha256
        || verification.resource_pack_sha256 != grant.grant().resource_pack_sha256
    {
        return Err(NativeCandidatePluginSourceError::SourceIdentityMismatch);
    }
    Ok(())
}

fn codex_verification(
    bundle: &crate::VerifiedCodexPluginBundle,
) -> NativeCandidatePluginSourceVerification {
    let receipt = bundle.receipt();
    NativeCandidatePluginSourceVerification {
        target: ClientActivationTarget::Codex,
        artifact: receipt.artifact.clone(),
        signed_grant_payload_sha256: receipt.signed_grant_payload_sha256.clone(),
        receipt_sha256: bundle.receipt_sha256().to_string(),
        package_content_root_sha256: receipt.package_content_root_sha256.clone(),
        binary_sha256: receipt.binary_sha256.clone(),
        resource_pack_sha256: receipt.resource_pack_sha256.clone(),
    }
}

fn claude_verification(
    bundle: &crate::VerifiedClaudePluginBundle,
) -> NativeCandidatePluginSourceVerification {
    let receipt = bundle.receipt();
    NativeCandidatePluginSourceVerification {
        target: ClientActivationTarget::ClaudeCode,
        artifact: receipt.artifact.clone(),
        signed_grant_payload_sha256: receipt.signed_grant_payload_sha256.clone(),
        receipt_sha256: bundle.receipt_sha256().to_string(),
        package_content_root_sha256: receipt.package_content_root_sha256.clone(),
        binary_sha256: receipt.binary_sha256.clone(),
        resource_pack_sha256: receipt.resource_pack_sha256.clone(),
    }
}

fn path_exists(
    target: &NativeCandidatePluginSourceTarget,
) -> Result<bool, NativeCandidatePluginSourceError> {
    match fs::symlink_metadata(target.path()) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => match &target.inner {
            NativeCandidatePluginSourceTargetKind::Codex(_) => {
                Err(NativeCandidatePluginSourceError::CodexBundle(
                    CodexPluginBundleError::PersistenceFailed(error.kind()),
                ))
            }
            NativeCandidatePluginSourceTargetKind::ClaudeCode(_) => {
                Err(NativeCandidatePluginSourceError::ClaudeBundle(
                    ClaudePluginBundleError::PersistenceFailed(error.kind()),
                ))
            }
        },
    }
}
