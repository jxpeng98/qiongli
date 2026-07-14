use std::fmt::{self, Display, Formatter};
use std::path::Path;

use qiongli_content::LoadedResourcePack;

use crate::{
    AllowedRootV1, ApprovalRequirement, ApprovedManagedRoot, CapabilityProfile, ClaudeAdapterError,
    ClaudeRegistrationCommit, ClaudeRegistrationDisposition, ClaudeRegistrationExecutor,
    ClientActivationTarget, CodexAdapterError, CodexRegistrationCommit,
    CodexRegistrationDisposition, CodexRegistrationExecutor, HostAction, InstallDisposition,
    InstallPlanMetadataV1, InstallScope, LocalSurface, LocalTargetFamily,
    ManagedNativePayloadExecutor, NativeCandidatePluginSourceCommit,
    NativeCandidatePluginSourceDisposition, NativeCandidatePluginSourceError,
    NativeCandidatePluginSourceTarget, NativePayloadInstallCommit, PlatformError, SymbolicRoot,
    TargetDescriptorV1, TransactionError, VerifiedNativeReleaseCandidate, approve_install_plan,
    discover_claude_user, discover_codex_user, materialize_native_candidate_plugin_source,
    native_artifact_binary_path, native_artifact_id, prepare_native_candidate_plugin_source_target,
    preview_claude_registration, preview_codex_registration, preview_native_payload_install,
    remove_native_candidate_plugin_source,
};

const PLAN_TTL_SECONDS: u64 = 600;
const PAYLOAD_APPROVALS: [ApprovalRequirement; 1] = [ApprovalRequirement::FilesystemWrite];
const REGISTRATION_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeCandidateRegistrationCommit {
    Codex(CodexRegistrationCommit),
    ClaudeCode(ClaudeRegistrationCommit),
}

impl NativeCandidateRegistrationCommit {
    fn was_fresh(&self) -> bool {
        match self {
            Self::Codex(commit) => commit.disposition == CodexRegistrationDisposition::Registered,
            Self::ClaudeCode(commit) => {
                commit.disposition == ClaudeRegistrationDisposition::Registered
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCandidateLocalInstallCommit {
    pub target: ClientActivationTarget,
    pub payload: NativePayloadInstallCommit,
    pub source: NativeCandidatePluginSourceCommit,
    pub registration: NativeCandidateRegistrationCommit,
    pub outstanding_host_action: HostAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeCandidateLocalInstallError {
    Platform(PlatformError),
    Transaction(TransactionError),
    Source(NativeCandidatePluginSourceError),
    Codex(CodexAdapterError),
    ClaudeCode(ClaudeAdapterError),
    InstalledBinaryInvalid,
    RecoveryRequired,
    CompensationFailed,
}

impl NativeCandidateLocalInstallError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Platform(error) => error.reason_code(),
            Self::Transaction(error) => error.reason_code(),
            Self::Source(error) => error.reason_code(),
            Self::Codex(error) => error.reason_code(),
            Self::ClaudeCode(error) => error.reason_code(),
            Self::InstalledBinaryInvalid => "native-candidate-installed-binary-invalid",
            Self::RecoveryRequired => "native-candidate-install-recovery-required",
            Self::CompensationFailed => "native-candidate-install-compensation-failed",
        }
    }
}

impl Display for NativeCandidateLocalInstallError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeCandidateLocalInstallError {}

/// Applies one verified candidate to its fixed local-client integration.
///
/// The home and managed root are trusted product boundaries. The source path,
/// installed binary path, plans, and inverse operations are derived internally.
pub fn apply_native_release_candidate_local(
    pack: &LoadedResourcePack<'_>,
    candidate: &VerifiedNativeReleaseCandidate,
    home: impl AsRef<Path>,
    managed_root: ApprovedManagedRoot,
    now_unix: u64,
) -> Result<NativeCandidateLocalInstallCommit, NativeCandidateLocalInstallError> {
    let home = home.as_ref();
    let root = AllowedRootV1 {
        id: managed_root.root_id().to_string(),
        root: SymbolicRoot::QiongliManagedData,
    };
    let payload_plan = preview_native_payload_install(
        plan_metadata(candidate, "payload", now_unix)?,
        candidate.portable_release(),
        target_descriptor(candidate),
        root,
    )
    .map_err(NativeCandidateLocalInstallError::Platform)?
    .verify_with_grant_capability(candidate.portable_release().launch_grant(), now_unix)
    .map_err(NativeCandidateLocalInstallError::Platform)?;
    let payload_approval = approve_install_plan(&payload_plan, &PAYLOAD_APPROVALS, now_unix)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let payload_executor = ManagedNativePayloadExecutor::new(managed_root.clone());
    let payload = payload_executor
        .apply(
            &payload_plan,
            &payload_approval,
            pack,
            candidate.portable_release(),
            now_unix,
        )
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let payload_fresh = payload.disposition == InstallDisposition::Applied;
    if payload_executor
        .verify(&payload.receipt.install_id, pack)
        .is_err()
    {
        compensate_payload(
            &payload_executor,
            &payload.receipt.install_id,
            pack,
            payload_fresh,
            now_unix,
        )?;
        return Err(NativeCandidateLocalInstallError::RecoveryRequired);
    }

    let installed_binary = managed_root
        .path()
        .join(
            native_artifact_id(&candidate.candidate().artifact)
                .map_err(|_| NativeCandidateLocalInstallError::InstalledBinaryInvalid)?,
        )
        .join(
            native_artifact_binary_path(&candidate.candidate().artifact)
                .map_err(|_| NativeCandidateLocalInstallError::InstalledBinaryInvalid)?,
        );
    let source_target =
        match prepare_native_candidate_plugin_source_target(home, candidate.target()) {
            Ok(target) => target,
            Err(error) => {
                compensate_payload(
                    &payload_executor,
                    &payload.receipt.install_id,
                    pack,
                    payload_fresh,
                    now_unix,
                )?;
                return Err(NativeCandidateLocalInstallError::Source(error));
            }
        };
    let source = match materialize_native_candidate_plugin_source(
        pack,
        candidate,
        &installed_binary,
        &source_target,
    ) {
        Ok(commit) => commit,
        Err(error) if source_commit_may_be_ambiguous(error) => {
            return Err(NativeCandidateLocalInstallError::RecoveryRequired);
        }
        Err(error) => {
            compensate_payload(
                &payload_executor,
                &payload.receipt.install_id,
                pack,
                payload_fresh,
                now_unix,
            )?;
            return Err(NativeCandidateLocalInstallError::Source(error));
        }
    };
    let source_fresh = source.disposition == NativeCandidatePluginSourceDisposition::Materialized;
    let compensation = CandidateCompensation {
        source_target: &source_target,
        source_fresh,
        payload_executor: &payload_executor,
        install_id: &payload.receipt.install_id,
        pack,
        payload_fresh,
        now_unix,
    };

    let registration = match apply_registration(candidate, home, now_unix) {
        Ok(commit) => commit,
        Err(error) if registration_error_requires_recovery(error) => {
            return Err(NativeCandidateLocalInstallError::RecoveryRequired);
        }
        Err(error) => {
            compensate_source_and_payload(&compensation)?;
            return Err(error);
        }
    };

    if verify_registration(&registration, home).is_err() {
        compensate_registration_source_and_payload(&registration, home, &compensation)?;
        return Err(NativeCandidateLocalInstallError::RecoveryRequired);
    }

    Ok(NativeCandidateLocalInstallCommit {
        target: candidate.target(),
        payload,
        source,
        registration,
        outstanding_host_action: HostAction::InstallOrEnablePlugin,
    })
}

fn apply_registration(
    candidate: &VerifiedNativeReleaseCandidate,
    home: &Path,
    now_unix: u64,
) -> Result<NativeCandidateRegistrationCommit, NativeCandidateLocalInstallError> {
    match candidate.target() {
        ClientActivationTarget::Codex => {
            let discovered =
                discover_codex_user(home).map_err(NativeCandidateLocalInstallError::Codex)?;
            let plan = preview_codex_registration(
                &discovered,
                plan_metadata(candidate, "codex", now_unix)?,
                candidate.plugin_grant(),
            )
            .map_err(NativeCandidateLocalInstallError::Codex)?
            .plan
            .verify_with_grant_capability(candidate.plugin_grant(), now_unix)
            .map_err(NativeCandidateLocalInstallError::Platform)?;
            let approval = approve_install_plan(&plan, &REGISTRATION_APPROVALS, now_unix)
                .map_err(NativeCandidateLocalInstallError::Transaction)?;
            CodexRegistrationExecutor::new(discovered)
                .apply(&plan, &approval, now_unix)
                .map(NativeCandidateRegistrationCommit::Codex)
                .map_err(NativeCandidateLocalInstallError::Codex)
        }
        ClientActivationTarget::ClaudeCode => {
            let discovered =
                discover_claude_user(home).map_err(NativeCandidateLocalInstallError::ClaudeCode)?;
            let plan = preview_claude_registration(
                &discovered,
                plan_metadata(candidate, "claude-code", now_unix)?,
                candidate.plugin_grant(),
            )
            .map_err(NativeCandidateLocalInstallError::ClaudeCode)?
            .plan
            .verify_with_grant_capability(candidate.plugin_grant(), now_unix)
            .map_err(NativeCandidateLocalInstallError::Platform)?;
            let approval = approve_install_plan(&plan, &REGISTRATION_APPROVALS, now_unix)
                .map_err(NativeCandidateLocalInstallError::Transaction)?;
            ClaudeRegistrationExecutor::new(discovered)
                .apply(&plan, &approval, now_unix)
                .map(NativeCandidateRegistrationCommit::ClaudeCode)
                .map_err(NativeCandidateLocalInstallError::ClaudeCode)
        }
    }
}

fn verify_registration(
    registration: &NativeCandidateRegistrationCommit,
    home: &Path,
) -> Result<(), NativeCandidateLocalInstallError> {
    match registration {
        NativeCandidateRegistrationCommit::Codex(_) => {
            let target =
                discover_codex_user(home).map_err(NativeCandidateLocalInstallError::Codex)?;
            CodexRegistrationExecutor::new(target)
                .verify()
                .map(|_| ())
                .map_err(NativeCandidateLocalInstallError::Codex)
        }
        NativeCandidateRegistrationCommit::ClaudeCode(_) => {
            let target =
                discover_claude_user(home).map_err(NativeCandidateLocalInstallError::ClaudeCode)?;
            ClaudeRegistrationExecutor::new(target)
                .verify()
                .map(|_| ())
                .map_err(NativeCandidateLocalInstallError::ClaudeCode)
        }
    }
}

struct CandidateCompensation<'a, 'pack> {
    source_target: &'a NativeCandidatePluginSourceTarget,
    source_fresh: bool,
    payload_executor: &'a ManagedNativePayloadExecutor,
    install_id: &'a str,
    pack: &'a LoadedResourcePack<'pack>,
    payload_fresh: bool,
    now_unix: u64,
}

fn compensate_registration_source_and_payload(
    registration: &NativeCandidateRegistrationCommit,
    home: &Path,
    compensation: &CandidateCompensation<'_, '_>,
) -> Result<(), NativeCandidateLocalInstallError> {
    if registration.was_fresh() {
        let rollback_failed = match registration {
            NativeCandidateRegistrationCommit::Codex(_) => discover_codex_user(home)
                .and_then(|target| {
                    CodexRegistrationExecutor::new(target).rollback(compensation.now_unix)
                })
                .is_err(),
            NativeCandidateRegistrationCommit::ClaudeCode(_) => discover_claude_user(home)
                .and_then(|target| {
                    ClaudeRegistrationExecutor::new(target).rollback(compensation.now_unix)
                })
                .is_err(),
        };
        if rollback_failed {
            return Err(NativeCandidateLocalInstallError::CompensationFailed);
        }
    }
    compensate_source_and_payload(compensation)
}

fn compensate_source_and_payload(
    compensation: &CandidateCompensation<'_, '_>,
) -> Result<(), NativeCandidateLocalInstallError> {
    if compensation.source_fresh
        && remove_native_candidate_plugin_source(compensation.source_target).is_err()
    {
        return Err(NativeCandidateLocalInstallError::CompensationFailed);
    }
    compensate_payload(
        compensation.payload_executor,
        compensation.install_id,
        compensation.pack,
        compensation.payload_fresh,
        compensation.now_unix,
    )
}

fn compensate_payload(
    executor: &ManagedNativePayloadExecutor,
    install_id: &str,
    pack: &LoadedResourcePack<'_>,
    fresh: bool,
    now_unix: u64,
) -> Result<(), NativeCandidateLocalInstallError> {
    if fresh && executor.rollback(install_id, pack, now_unix).is_err() {
        return Err(NativeCandidateLocalInstallError::CompensationFailed);
    }
    Ok(())
}

fn target_descriptor(candidate: &VerifiedNativeReleaseCandidate) -> TargetDescriptorV1 {
    let artifact = &candidate.candidate().artifact;
    let (family, surface) = match candidate.target() {
        ClientActivationTarget::Codex => {
            (LocalTargetFamily::CodexLocal, LocalSurface::DesktopLocal)
        }
        ClientActivationTarget::ClaudeCode => {
            (LocalTargetFamily::ClaudeCodeLocal, LocalSurface::CliLocal)
        }
    };
    TargetDescriptorV1 {
        family,
        surface,
        scope: InstallScope::User,
        profile: CapabilityProfile::Lite,
        os: artifact.os,
        arch: artifact.arch,
        adapter_version: 1,
    }
}

fn plan_metadata(
    candidate: &VerifiedNativeReleaseCandidate,
    phase: &str,
    now_unix: u64,
) -> Result<InstallPlanMetadataV1, NativeCandidateLocalInstallError> {
    let prefix = candidate
        .signed_payload_sha256()
        .get(..24)
        .ok_or(NativeCandidateLocalInstallError::InstalledBinaryInvalid)?;
    let target = match candidate.target() {
        ClientActivationTarget::Codex => "codex",
        ClientActivationTarget::ClaudeCode => "claude-code",
    };
    let expires_at_unix = now_unix
        .saturating_add(PLAN_TTL_SECONDS)
        .min(candidate.candidate().expires_at_unix);
    if expires_at_unix <= now_unix {
        return Err(NativeCandidateLocalInstallError::Platform(
            PlatformError::InstallPlanExpired,
        ));
    }
    Ok(InstallPlanMetadataV1 {
        plan_id: format!("candidate-{target}-{phase}-{prefix}"),
        created_at_unix: now_unix,
        expires_at_unix,
    })
}

const fn source_commit_may_be_ambiguous(error: NativeCandidatePluginSourceError) -> bool {
    matches!(
        error,
        NativeCandidatePluginSourceError::CodexBundle(
            crate::CodexPluginBundleError::CommitFailed(_)
                | crate::CodexPluginBundleError::CommittedPersistenceFailed(_)
                | crate::CodexPluginBundleError::CommittedVerificationFailed
        ) | NativeCandidatePluginSourceError::ClaudeBundle(
            crate::ClaudePluginBundleError::CommitFailed(_)
                | crate::ClaudePluginBundleError::CommittedPersistenceFailed(_)
                | crate::ClaudePluginBundleError::CommittedVerificationFailed
        )
    )
}

const fn registration_error_requires_recovery(error: NativeCandidateLocalInstallError) -> bool {
    matches!(
        error,
        NativeCandidateLocalInstallError::Codex(
            CodexAdapterError::RecoveryRequired | CodexAdapterError::RollbackFailed
        ) | NativeCandidateLocalInstallError::ClaudeCode(
            ClaudeAdapterError::RecoveryRequired | ClaudeAdapterError::RollbackFailed
        )
    )
}
