use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::{
    ApprovalRequirement, ApprovedInstallPlan, CapabilityProfile, ClaudeAdapterError,
    ClaudeMarketplaceState, ClaudeRegistrationDisposition, ClaudeRegistrationExecutor,
    ClaudeRegistrationLifecycleDisposition, ClaudeRegistrationState, ClaudeSkillsPluginState,
    ClaudeSourceState, ClaudeUserTarget, CodexAdapterError, CodexMarketplaceState,
    CodexRegistrationDisposition, CodexRegistrationExecutor, CodexRegistrationLifecycleDisposition,
    CodexRegistrationState, CodexSourceState, CodexUserTarget, GrantMode, GrantVerificationContext,
    HostAction, InstallPlanMetadataV1, InstallerKind, IntegrationScope, LocalTargetFamily,
    TrustedPublicKey, VerifiedInstallPlan, VerifiedLaunchGrant, discover_claude_user_with_config,
    discover_codex_user, preview_claude_registration, preview_codex_registration,
};

pub const CLIENT_ACTIVATION_SCHEMA_VERSION: u32 = 1;

const EXACT_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];
static NEXT_TARGET_BINDING: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientActivationTarget {
    Codex,
    ClaudeCode,
}

impl ClientActivationTarget {
    #[must_use]
    pub const fn integration_scope(self) -> IntegrationScope {
        match self {
            Self::Codex => IntegrationScope::CodexLocal,
            Self::ClaudeCode => IntegrationScope::ClaudeCodeLocal,
        }
    }

    const fn family(self) -> LocalTargetFamily {
        match self {
            Self::Codex => LocalTargetFamily::CodexLocal,
            Self::ClaudeCode => LocalTargetFamily::ClaudeCodeLocal,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientActivationState {
    Missing,
    Ready,
    Conflict,
    Drifted,
    RecoveryRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientActivationDiscoveryV1 {
    pub schema_version: u32,
    pub target: ClientActivationTarget,
    pub source: ClientActivationState,
    pub marketplace: ClientActivationState,
    pub direct_package: Option<ClientActivationState>,
    pub registration: ClientActivationState,
}

#[derive(Clone)]
pub struct ClientActivationHandle {
    target: ClientActivationTarget,
    target_binding: u64,
    discovery: ClientActivationDiscoveryV1,
    inner: ClientActivationHandleInner,
}

#[derive(Clone)]
enum ClientActivationHandleInner {
    Codex(CodexUserTarget),
    ClaudeCode(ClaudeUserTarget),
}

impl ClientActivationHandle {
    #[must_use]
    pub const fn target(&self) -> ClientActivationTarget {
        self.target
    }

    #[must_use]
    pub const fn discovery(&self) -> &ClientActivationDiscoveryV1 {
        &self.discovery
    }
}

impl Debug for ClientActivationHandle {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientActivationHandle")
            .field("target", &self.target)
            .field("discovery", &self.discovery)
            .finish()
    }
}

pub fn discover_client_activation(
    home: &Path,
    claude_config_root: Option<&Path>,
    target: ClientActivationTarget,
) -> Result<ClientActivationHandle, ClientActivationError> {
    let target_binding = NEXT_TARGET_BINDING
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| ClientActivationError::BindingUnavailable)?;
    match target {
        ClientActivationTarget::Codex => {
            let target_handle = discover_codex_user(home).map_err(ClientActivationError::Codex)?;
            let summary = target_handle.summary();
            Ok(ClientActivationHandle {
                target,
                target_binding,
                discovery: ClientActivationDiscoveryV1 {
                    schema_version: CLIENT_ACTIVATION_SCHEMA_VERSION,
                    target,
                    source: codex_source_state(summary.source),
                    marketplace: codex_marketplace_state(summary.marketplace),
                    direct_package: None,
                    registration: codex_registration_state(summary.registration),
                },
                inner: ClientActivationHandleInner::Codex(target_handle),
            })
        }
        ClientActivationTarget::ClaudeCode => {
            let default_config_root = home.join(".claude");
            let config_root = claude_config_root.unwrap_or(&default_config_root);
            let target_handle = discover_claude_user_with_config(home, config_root)
                .map_err(ClientActivationError::Claude)?;
            let summary = target_handle.summary();
            Ok(ClientActivationHandle {
                target,
                target_binding,
                discovery: ClientActivationDiscoveryV1 {
                    schema_version: CLIENT_ACTIVATION_SCHEMA_VERSION,
                    target,
                    source: claude_source_state(summary.source),
                    marketplace: claude_marketplace_state(summary.marketplace),
                    direct_package: Some(claude_skills_state(summary.skills_plugin)),
                    registration: claude_registration_state(summary.registration),
                },
                inner: ClientActivationHandleInner::ClaudeCode(target_handle),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientActivationEffect {
    Activate,
    AlreadyActive,
}

#[derive(Clone)]
pub struct ClientActivationPreview {
    target: ClientActivationTarget,
    target_binding: u64,
    effect: ClientActivationEffect,
    plan: VerifiedInstallPlan,
}

impl Debug for ClientActivationPreview {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientActivationPreview")
            .field("target", &self.target)
            .field("effect", &self.effect)
            .field(
                "plan_digest_sha256",
                &self.plan.plan().semantic_digest_sha256,
            )
            .finish()
    }
}

impl ClientActivationPreview {
    #[must_use]
    pub const fn target(&self) -> ClientActivationTarget {
        self.target
    }

    #[must_use]
    pub const fn effect(&self) -> ClientActivationEffect {
        self.effect
    }

    #[must_use]
    pub const fn plan(&self) -> &VerifiedInstallPlan {
        &self.plan
    }
}

#[allow(clippy::too_many_arguments)]
pub fn preview_client_activation(
    target: &ClientActivationHandle,
    metadata: InstallPlanMetadataV1,
    grant: &VerifiedLaunchGrant,
    trusted_keys: &[TrustedPublicKey],
    minimum_generation: u64,
    now_unix: u64,
) -> Result<ClientActivationPreview, ClientActivationError> {
    let artifact = &grant.grant().artifact;
    if artifact.profile != CapabilityProfile::Lite
        || artifact.installer_kind != InstallerKind::PluginBundle
        || grant.authorized_mode() != GrantMode::LiteMcp
        || grant.authorized_scope() != target.target.integration_scope()
    {
        return Err(ClientActivationError::TargetMismatch);
    }

    let (plan, effect) = match &target.inner {
        ClientActivationHandleInner::Codex(target) => {
            let preview = preview_codex_registration(target, metadata, grant)
                .map_err(ClientActivationError::Codex)?;
            let effect = match preview.effect {
                crate::CodexRegistrationEffect::Register => ClientActivationEffect::Activate,
                crate::CodexRegistrationEffect::AlreadyRegistered => {
                    ClientActivationEffect::AlreadyActive
                }
            };
            (preview.plan, effect)
        }
        ClientActivationHandleInner::ClaudeCode(target) => {
            let preview = preview_claude_registration(target, metadata, grant)
                .map_err(ClientActivationError::Claude)?;
            let effect = match preview.effect {
                crate::ClaudeRegistrationEffect::Register => ClientActivationEffect::Activate,
                crate::ClaudeRegistrationEffect::AlreadyRegistered => {
                    ClientActivationEffect::AlreadyActive
                }
            };
            (preview.plan, effect)
        }
    };

    let context = GrantVerificationContext {
        now_unix,
        minimum_generation,
        expected_artifact: artifact,
        binary_sha256: &grant.grant().binary_sha256,
        resource_pack_sha256: &grant.grant().resource_pack_sha256,
        requested_mode: GrantMode::LiteMcp,
        requested_scope: target.target.integration_scope(),
    };
    let plan = plan
        .verify(trusted_keys, &context)
        .map_err(|_| ClientActivationError::PlanInvalid)?;
    if plan.plan().target.family != target.target.family()
        || plan.plan().approvals_required != EXACT_APPROVALS
        || plan.plan().outstanding_host_action != Some(HostAction::InstallOrEnablePlugin)
    {
        return Err(ClientActivationError::PlanInvalid);
    }
    Ok(ClientActivationPreview {
        target: target.target,
        target_binding: target.target_binding,
        effect,
        plan,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientActivationDisposition {
    Activated,
    AlreadyActive,
    Repaired,
    AlreadyHealthy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientActivationCommit {
    pub target: ClientActivationTarget,
    pub disposition: ClientActivationDisposition,
    pub transaction_id: String,
    pub plan_digest_sha256: String,
    pub cleanup_required: bool,
    pub outstanding_host_action: HostAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientActivationVerification {
    pub target: ClientActivationTarget,
    pub transaction_id: String,
    pub plan_digest_sha256: String,
    pub outstanding_host_action: HostAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientActivationLifecycleDisposition {
    Removed,
    AlreadyRemoved,
    RolledBack,
    AlreadyRolledBack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientActivationLifecycleCommit {
    pub target: ClientActivationTarget,
    pub disposition: ClientActivationLifecycleDisposition,
    pub transaction_id: String,
    pub prior_transaction_id: String,
    pub cleanup_required: bool,
}

#[derive(Clone)]
pub struct ClientActivationCoordinator {
    target: ClientActivationTarget,
    target_binding: u64,
    inner: ClientActivationCoordinatorInner,
}

impl Debug for ClientActivationCoordinator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientActivationCoordinator")
            .field("target", &self.target)
            .finish()
    }
}

#[derive(Clone, Debug)]
enum ClientActivationCoordinatorInner {
    Codex(CodexRegistrationExecutor),
    ClaudeCode(ClaudeRegistrationExecutor),
}

impl ClientActivationCoordinator {
    #[must_use]
    pub fn new(target: ClientActivationHandle) -> Self {
        let target_kind = target.target;
        let target_binding = target.target_binding;
        let inner = match target.inner {
            ClientActivationHandleInner::Codex(target) => {
                ClientActivationCoordinatorInner::Codex(CodexRegistrationExecutor::new(target))
            }
            ClientActivationHandleInner::ClaudeCode(target) => {
                ClientActivationCoordinatorInner::ClaudeCode(ClaudeRegistrationExecutor::new(
                    target,
                ))
            }
        };
        Self {
            target: target_kind,
            target_binding,
            inner,
        }
    }

    pub fn apply(
        &self,
        preview: &ClientActivationPreview,
        approval: &ApprovedInstallPlan,
        now_unix: u64,
    ) -> Result<ClientActivationCommit, ClientActivationError> {
        self.apply_or_repair(preview, approval, now_unix, false)
    }

    pub fn repair(
        &self,
        preview: &ClientActivationPreview,
        approval: &ApprovedInstallPlan,
        now_unix: u64,
    ) -> Result<ClientActivationCommit, ClientActivationError> {
        self.apply_or_repair(preview, approval, now_unix, true)
    }

    pub fn verify(&self) -> Result<ClientActivationVerification, ClientActivationError> {
        match &self.inner {
            ClientActivationCoordinatorInner::Codex(executor) => {
                let verification = executor.verify().map_err(ClientActivationError::Codex)?;
                Ok(ClientActivationVerification {
                    target: self.target,
                    transaction_id: verification.receipt.transaction_id,
                    plan_digest_sha256: verification.receipt.semantic_digest_sha256,
                    outstanding_host_action: verification.receipt.outstanding_host_action,
                })
            }
            ClientActivationCoordinatorInner::ClaudeCode(executor) => {
                let verification = executor.verify().map_err(ClientActivationError::Claude)?;
                Ok(ClientActivationVerification {
                    target: self.target,
                    transaction_id: verification.receipt.transaction_id,
                    plan_digest_sha256: verification.receipt.semantic_digest_sha256,
                    outstanding_host_action: verification.receipt.outstanding_host_action,
                })
            }
        }
    }

    pub fn remove(
        &self,
        now_unix: u64,
    ) -> Result<ClientActivationLifecycleCommit, ClientActivationError> {
        match &self.inner {
            ClientActivationCoordinatorInner::Codex(executor) => executor
                .remove(now_unix)
                .map(codex_lifecycle_commit)
                .map_err(ClientActivationError::Codex),
            ClientActivationCoordinatorInner::ClaudeCode(executor) => executor
                .remove(now_unix)
                .map(claude_lifecycle_commit)
                .map_err(ClientActivationError::Claude),
        }
    }

    pub fn rollback(
        &self,
        now_unix: u64,
    ) -> Result<ClientActivationLifecycleCommit, ClientActivationError> {
        match &self.inner {
            ClientActivationCoordinatorInner::Codex(executor) => executor
                .rollback(now_unix)
                .map(codex_lifecycle_commit)
                .map_err(ClientActivationError::Codex),
            ClientActivationCoordinatorInner::ClaudeCode(executor) => executor
                .rollback(now_unix)
                .map(claude_lifecycle_commit)
                .map_err(ClientActivationError::Claude),
        }
    }

    fn apply_or_repair(
        &self,
        preview: &ClientActivationPreview,
        approval: &ApprovedInstallPlan,
        now_unix: u64,
        repair: bool,
    ) -> Result<ClientActivationCommit, ClientActivationError> {
        if preview.target != self.target || preview.target_binding != self.target_binding {
            return Err(ClientActivationError::TargetMismatch);
        }
        let commit = match &self.inner {
            ClientActivationCoordinatorInner::Codex(executor) => {
                let commit = if repair {
                    executor.repair(&preview.plan, approval, now_unix)
                } else {
                    executor.apply(&preview.plan, approval, now_unix)
                }
                .map_err(ClientActivationError::Codex)?;
                codex_commit(commit)
            }
            ClientActivationCoordinatorInner::ClaudeCode(executor) => {
                let commit = if repair {
                    executor.repair(&preview.plan, approval, now_unix)
                } else {
                    executor.apply(&preview.plan, approval, now_unix)
                }
                .map_err(ClientActivationError::Claude)?;
                claude_commit(commit)
            }
        };

        if self.verify().is_err() {
            if matches!(
                commit.disposition,
                ClientActivationDisposition::Activated | ClientActivationDisposition::Repaired
            ) {
                return if self.rollback(now_unix).is_ok() {
                    Err(ClientActivationError::PostApplyVerificationFailed)
                } else {
                    Err(ClientActivationError::RollbackFailed)
                };
            }
            return Err(ClientActivationError::PostApplyVerificationFailed);
        }
        Ok(commit)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientActivationError {
    TargetMismatch,
    BindingUnavailable,
    PlanInvalid,
    PostApplyVerificationFailed,
    RollbackFailed,
    Codex(CodexAdapterError),
    Claude(ClaudeAdapterError),
}

impl ClientActivationError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::TargetMismatch => "client-activation-target-mismatch",
            Self::BindingUnavailable => "client-activation-binding-unavailable",
            Self::PlanInvalid => "client-activation-plan-invalid",
            Self::PostApplyVerificationFailed => "client-activation-verification-failed",
            Self::RollbackFailed => "client-activation-rollback-failed",
            Self::Codex(error) => error.reason_code(),
            Self::Claude(error) => error.reason_code(),
        }
    }
}

impl Display for ClientActivationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for ClientActivationError {}

fn codex_commit(commit: crate::CodexRegistrationCommit) -> ClientActivationCommit {
    ClientActivationCommit {
        target: ClientActivationTarget::Codex,
        disposition: match commit.disposition {
            CodexRegistrationDisposition::Registered => ClientActivationDisposition::Activated,
            CodexRegistrationDisposition::AlreadyRegistered => {
                ClientActivationDisposition::AlreadyActive
            }
            CodexRegistrationDisposition::Repaired => ClientActivationDisposition::Repaired,
            CodexRegistrationDisposition::AlreadyHealthy => {
                ClientActivationDisposition::AlreadyHealthy
            }
        },
        transaction_id: commit.receipt.transaction_id,
        plan_digest_sha256: commit.receipt.semantic_digest_sha256,
        cleanup_required: commit.cleanup_required,
        outstanding_host_action: commit.receipt.outstanding_host_action,
    }
}

fn claude_commit(commit: crate::ClaudeRegistrationCommit) -> ClientActivationCommit {
    ClientActivationCommit {
        target: ClientActivationTarget::ClaudeCode,
        disposition: match commit.disposition {
            ClaudeRegistrationDisposition::Registered => ClientActivationDisposition::Activated,
            ClaudeRegistrationDisposition::AlreadyRegistered => {
                ClientActivationDisposition::AlreadyActive
            }
            ClaudeRegistrationDisposition::Repaired => ClientActivationDisposition::Repaired,
            ClaudeRegistrationDisposition::AlreadyHealthy => {
                ClientActivationDisposition::AlreadyHealthy
            }
        },
        transaction_id: commit.receipt.transaction_id,
        plan_digest_sha256: commit.receipt.semantic_digest_sha256,
        cleanup_required: commit.cleanup_required,
        outstanding_host_action: commit.receipt.outstanding_host_action,
    }
}

fn codex_lifecycle_commit(
    commit: crate::CodexRegistrationLifecycleCommit,
) -> ClientActivationLifecycleCommit {
    ClientActivationLifecycleCommit {
        target: ClientActivationTarget::Codex,
        disposition: match commit.disposition {
            CodexRegistrationLifecycleDisposition::Removed => {
                ClientActivationLifecycleDisposition::Removed
            }
            CodexRegistrationLifecycleDisposition::AlreadyRemoved => {
                ClientActivationLifecycleDisposition::AlreadyRemoved
            }
            CodexRegistrationLifecycleDisposition::RolledBack => {
                ClientActivationLifecycleDisposition::RolledBack
            }
            CodexRegistrationLifecycleDisposition::AlreadyRolledBack => {
                ClientActivationLifecycleDisposition::AlreadyRolledBack
            }
        },
        transaction_id: commit.receipt.transaction_id,
        prior_transaction_id: commit.receipt.prior_transaction_id,
        cleanup_required: commit.cleanup_required,
    }
}

fn claude_lifecycle_commit(
    commit: crate::ClaudeRegistrationLifecycleCommit,
) -> ClientActivationLifecycleCommit {
    ClientActivationLifecycleCommit {
        target: ClientActivationTarget::ClaudeCode,
        disposition: match commit.disposition {
            ClaudeRegistrationLifecycleDisposition::Removed => {
                ClientActivationLifecycleDisposition::Removed
            }
            ClaudeRegistrationLifecycleDisposition::AlreadyRemoved => {
                ClientActivationLifecycleDisposition::AlreadyRemoved
            }
            ClaudeRegistrationLifecycleDisposition::RolledBack => {
                ClientActivationLifecycleDisposition::RolledBack
            }
            ClaudeRegistrationLifecycleDisposition::AlreadyRolledBack => {
                ClientActivationLifecycleDisposition::AlreadyRolledBack
            }
        },
        transaction_id: commit.receipt.transaction_id,
        prior_transaction_id: commit.receipt.prior_transaction_id,
        cleanup_required: commit.cleanup_required,
    }
}

const fn codex_source_state(state: CodexSourceState) -> ClientActivationState {
    match state {
        CodexSourceState::Missing => ClientActivationState::Missing,
        CodexSourceState::Ready => ClientActivationState::Ready,
    }
}

const fn codex_marketplace_state(state: CodexMarketplaceState) -> ClientActivationState {
    match state {
        CodexMarketplaceState::Missing => ClientActivationState::Missing,
        CodexMarketplaceState::Ready => ClientActivationState::Ready,
    }
}

const fn codex_registration_state(state: CodexRegistrationState) -> ClientActivationState {
    match state {
        CodexRegistrationState::Absent => ClientActivationState::Missing,
        CodexRegistrationState::Registered => ClientActivationState::Ready,
        CodexRegistrationState::Conflict => ClientActivationState::Conflict,
        CodexRegistrationState::Drifted => ClientActivationState::Drifted,
        CodexRegistrationState::RecoveryRequired => ClientActivationState::RecoveryRequired,
    }
}

const fn claude_source_state(state: ClaudeSourceState) -> ClientActivationState {
    match state {
        ClaudeSourceState::Missing => ClientActivationState::Missing,
        ClaudeSourceState::Ready => ClientActivationState::Ready,
    }
}

const fn claude_marketplace_state(state: ClaudeMarketplaceState) -> ClientActivationState {
    match state {
        ClaudeMarketplaceState::Missing => ClientActivationState::Missing,
        ClaudeMarketplaceState::Ready => ClientActivationState::Ready,
    }
}

const fn claude_skills_state(state: ClaudeSkillsPluginState) -> ClientActivationState {
    match state {
        ClaudeSkillsPluginState::Missing => ClientActivationState::Missing,
        ClaudeSkillsPluginState::Ready => ClientActivationState::Ready,
        ClaudeSkillsPluginState::Conflict => ClientActivationState::Conflict,
    }
}

const fn claude_registration_state(state: ClaudeRegistrationState) -> ClientActivationState {
    match state {
        ClaudeRegistrationState::Absent => ClientActivationState::Missing,
        ClaudeRegistrationState::Registered => ClientActivationState::Ready,
        ClaudeRegistrationState::Conflict => ClientActivationState::Conflict,
        ClaudeRegistrationState::Drifted => ClientActivationState::Drifted,
        ClaudeRegistrationState::RecoveryRequired => ClientActivationState::RecoveryRequired,
    }
}
