use serde::{Deserialize, Serialize};

use crate::grant::{
    GrantMode, GrantVerificationContext, IntegrationScope, SignedLaunchGrantV1, TrustedPublicKey,
    VerifiedLaunchGrant, is_lower_hex, is_sorted_unique, launch_grant_signing_bytes, sha256_hex,
    valid_identifier,
};
use crate::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, INSTALL_PLAN_SCHEMA_VERSION,
    OperatingSystem, PlatformError, ProductId,
};

const MAX_INSTALL_PLAN_BYTES: usize = 1024 * 1024;
const MAX_ALLOWED_ROOTS: usize = 16;
const MAX_OPERATIONS: usize = 128;
const MAX_APPROVALS: usize = 16;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_RELATIVE_PATH_BYTES: usize = 512;
const MAX_ADAPTER_VERSION: u32 = 1;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const LITE_MCP_ARGUMENTS: [&str; 6] = ["mcp", "serve", "--profile", "lite", "--transport", "stdio"];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalTargetFamily {
    CodexLocal,
    ClaudeCodeLocal,
}

impl LocalTargetFamily {
    const fn integration_scope(self) -> IntegrationScope {
        match self {
            Self::CodexLocal => IntegrationScope::CodexLocal,
            Self::ClaudeCodeLocal => IntegrationScope::ClaudeCodeLocal,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalSurface {
    CliLocal,
    DesktopLocal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallScope {
    User,
    Repository,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetDescriptorV1 {
    pub family: LocalTargetFamily,
    pub surface: LocalSurface,
    pub scope: InstallScope,
    pub profile: CapabilityProfile,
    pub os: OperatingSystem,
    pub arch: Architecture,
    pub adapter_version: u32,
}

impl TargetDescriptorV1 {
    fn validate(&self, artifact: &ArtifactIdentityV1) -> Result<(), PlatformError> {
        if self.profile != CapabilityProfile::Lite
            || self.os != artifact.os
            || self.arch != artifact.arch
            || self.adapter_version == 0
            || self.adapter_version > MAX_ADAPTER_VERSION
        {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolicRoot {
    QiongliManagedData,
    CodexPersonalMarketplace,
    CodexRepositoryMarketplace,
    CodexConfig,
    ClaudeSkillsDirectory,
    ClaudeMarketplaceSource,
    ClaudeCodeConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllowedRootV1 {
    pub id: String,
    pub root: SymbolicRoot,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnershipMarkerV1 {
    pub schema_version: u32,
    pub product: ProductId,
    pub install_id: String,
    pub artifact_digest_sha256: String,
}

impl OwnershipMarkerV1 {
    fn validate_shape(&self) -> Result<(), PlatformError> {
        if self.schema_version != 1
            || !valid_identifier(&self.install_id, MAX_IDENTIFIER_BYTES)
            || !is_lower_hex(&self.artifact_digest_sha256, 64)
        {
            return Err(PlatformError::InvalidInstallPlan);
        }
        Ok(())
    }

    fn validate(&self, artifact_digest: &str) -> Result<(), PlatformError> {
        self.validate_shape()?;
        if self.artifact_digest_sha256 != artifact_digest {
            return Err(PlatformError::InvalidInstallPlan);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, tag = "state", rename_all = "kebab-case")]
pub enum PlanStateV1 {
    Missing,
    Managed {
        ownership: OwnershipMarkerV1,
        content_sha256: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "kebab-case")]
pub enum InstallActionV1 {
    MaterializeResources {
        root_id: String,
        entry_key: String,
        relative_path: String,
        content_root_sha256: String,
        ownership: OwnershipMarkerV1,
    },
    InstallNativePayload {
        root_id: String,
        entry_key: String,
        relative_path: String,
        release_envelope_sha256: String,
        archive_sha256: String,
        manifest_sha256: String,
        pack_sha256: String,
        artifact_content_root_sha256: String,
        binary_sha256: String,
        ownership: OwnershipMarkerV1,
    },
    RegisterPluginSource {
        root_id: String,
        entry_key: String,
        source_id: String,
        source_digest_sha256: String,
        ownership: OwnershipMarkerV1,
    },
    RegisterLiteMcp {
        root_id: String,
        entry_key: String,
        executable_relative_path: String,
        arguments: Vec<String>,
        binary_sha256: String,
        ownership: OwnershipMarkerV1,
    },
    RemoveManagedEntry {
        root_id: String,
        entry_key: String,
        expected_ownership: OwnershipMarkerV1,
        expected_sha256: String,
    },
}

impl InstallActionV1 {
    fn root_id(&self) -> &str {
        match self {
            Self::MaterializeResources { root_id, .. }
            | Self::InstallNativePayload { root_id, .. }
            | Self::RegisterPluginSource { root_id, .. }
            | Self::RegisterLiteMcp { root_id, .. }
            | Self::RemoveManagedEntry { root_id, .. } => root_id,
        }
    }

    fn entry_key(&self) -> &str {
        match self {
            Self::MaterializeResources { entry_key, .. }
            | Self::InstallNativePayload { entry_key, .. }
            | Self::RegisterPluginSource { entry_key, .. }
            | Self::RegisterLiteMcp { entry_key, .. }
            | Self::RemoveManagedEntry { entry_key, .. } => entry_key,
        }
    }

    fn ownership(&self) -> &OwnershipMarkerV1 {
        match self {
            Self::MaterializeResources { ownership, .. }
            | Self::InstallNativePayload { ownership, .. }
            | Self::RegisterPluginSource { ownership, .. }
            | Self::RegisterLiteMcp { ownership, .. } => ownership,
            Self::RemoveManagedEntry {
                expected_ownership, ..
            } => expected_ownership,
        }
    }

    const fn is_remove(&self) -> bool {
        matches!(self, Self::RemoveManagedEntry { .. })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallOperationV1 {
    pub operation_id: String,
    pub action: InstallActionV1,
    pub precondition: PlanStateV1,
    pub observed_state_sha256: String,
    pub postcondition: PlanStateV1,
    pub inverse: InstallActionV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalRequirement {
    FilesystemWrite,
    ClientConfigChange,
    HostTrust,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostAction {
    InstallOrEnablePlugin,
    RestartClient,
    ApproveMcp,
}

#[derive(Clone, Debug)]
pub struct InstallPlanMetadataV1 {
    pub plan_id: String,
    pub created_at_unix: u64,
    pub expires_at_unix: u64,
}

#[derive(Clone, Debug)]
pub struct InstallPlanDraftV1 {
    pub target: TargetDescriptorV1,
    pub allowed_roots: Vec<AllowedRootV1>,
    pub operations: Vec<InstallOperationV1>,
    pub approvals_required: Vec<ApprovalRequirement>,
    pub outstanding_host_action: Option<HostAction>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallPlanV1 {
    pub schema_version: u32,
    pub plan_id: String,
    pub created_at_unix: u64,
    pub expires_at_unix: u64,
    pub artifact: ArtifactIdentityV1,
    pub signed_launch_grant: SignedLaunchGrantV1,
    pub target: TargetDescriptorV1,
    pub allowed_roots: Vec<AllowedRootV1>,
    pub operations: Vec<InstallOperationV1>,
    pub approvals_required: Vec<ApprovalRequirement>,
    pub outstanding_host_action: Option<HostAction>,
    pub semantic_digest_sha256: String,
}

impl InstallPlanV1 {
    pub fn build(
        metadata: InstallPlanMetadataV1,
        verified_grant: &VerifiedLaunchGrant,
        draft: InstallPlanDraftV1,
    ) -> Result<Self, PlatformError> {
        if verified_grant.authorized_mode() != GrantMode::LiteMcp
            || verified_grant.authorized_scope() != draft.target.family.integration_scope()
            || metadata.created_at_unix < verified_grant.verified_at_unix()
        {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        let mut plan = Self {
            schema_version: INSTALL_PLAN_SCHEMA_VERSION,
            plan_id: metadata.plan_id,
            created_at_unix: metadata.created_at_unix,
            expires_at_unix: metadata.expires_at_unix,
            artifact: verified_grant.grant().artifact.clone(),
            signed_launch_grant: verified_grant.signed_grant().clone(),
            target: draft.target,
            allowed_roots: draft.allowed_roots,
            operations: draft.operations,
            approvals_required: draft.approvals_required,
            outstanding_host_action: draft.outstanding_host_action,
            semantic_digest_sha256: String::new(),
        };
        plan.validate_structure(Some(verified_grant.signed_payload_sha256()))?;
        plan.semantic_digest_sha256 = plan.compute_semantic_digest()?;
        plan.validate_structure(Some(verified_grant.signed_payload_sha256()))?;
        Ok(plan)
    }

    pub fn from_json(input: &[u8]) -> Result<Self, PlatformError> {
        if input.len() > MAX_INSTALL_PLAN_BYTES {
            return Err(PlatformError::InstallPlanTooLarge);
        }
        let plan = serde_json::from_slice::<Self>(input)
            .map_err(|_| PlatformError::InvalidInstallPlanJson)?;
        plan.validate_structure(None)?;
        plan.verify_semantic_digest()?;
        Ok(plan)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, PlatformError> {
        self.validate_structure(None)?;
        self.verify_semantic_digest()?;
        serde_json_canonicalizer::to_vec(self)
            .map_err(|_| PlatformError::CanonicalSerializationFailed)
    }

    pub fn verify(
        &self,
        trusted_keys: &[TrustedPublicKey],
        context: &GrantVerificationContext<'_>,
    ) -> Result<VerifiedInstallPlan, PlatformError> {
        self.validate_structure(None)?;
        self.verify_semantic_digest()?;
        if context.now_unix < self.created_at_unix {
            return Err(PlatformError::InstallPlanNotYetValid);
        }
        if context.now_unix >= self.expires_at_unix {
            return Err(PlatformError::InstallPlanExpired);
        }
        if context.expected_artifact != &self.artifact
            || context.requested_scope != self.target.family.integration_scope()
        {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        let verified_grant = self.signed_launch_grant.verify(trusted_keys, context)?;
        if verified_grant.signed_payload_sha256() != self.artifact_digest()? {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        self.validate_structure(Some(verified_grant.signed_payload_sha256()))?;
        Ok(VerifiedInstallPlan {
            plan: self.clone(),
            grant: verified_grant,
        })
    }

    /// Verifies a newly built plan against an already verified launch capability.
    ///
    /// This path is used after a release candidate has authenticated the exact
    /// signed grant. It never re-opens a public-key or raw-signature boundary.
    pub(crate) fn verify_with_grant_capability(
        &self,
        verified_grant: &VerifiedLaunchGrant,
        now_unix: u64,
    ) -> Result<VerifiedInstallPlan, PlatformError> {
        self.validate_structure(None)?;
        self.verify_semantic_digest()?;
        if now_unix < self.created_at_unix || now_unix < verified_grant.verified_at_unix() {
            return Err(PlatformError::InstallPlanNotYetValid);
        }
        if now_unix >= self.expires_at_unix {
            return Err(PlatformError::InstallPlanExpired);
        }
        if &self.signed_launch_grant != verified_grant.signed_grant()
            || self.artifact != verified_grant.grant().artifact
            || self.target.family.integration_scope() != verified_grant.authorized_scope()
            || verified_grant.signed_payload_sha256() != self.artifact_digest()?
        {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        self.validate_structure(Some(verified_grant.signed_payload_sha256()))?;
        Ok(VerifiedInstallPlan {
            plan: self.clone(),
            grant: verified_grant.clone(),
        })
    }

    fn validate_structure(
        &self,
        expected_artifact_digest: Option<&str>,
    ) -> Result<(), PlatformError> {
        if self.schema_version != INSTALL_PLAN_SCHEMA_VERSION {
            return Err(PlatformError::InvalidInstallPlanSchema);
        }
        if !valid_identifier(&self.plan_id, MAX_IDENTIFIER_BYTES)
            || self.created_at_unix > JCS_MAX_SAFE_INTEGER
            || self.expires_at_unix > JCS_MAX_SAFE_INTEGER
            || self.created_at_unix >= self.expires_at_unix
        {
            return Err(PlatformError::InvalidInstallPlan);
        }
        self.signed_launch_grant.validate_structure()?;
        if self.artifact != self.signed_launch_grant.grant.artifact
            || self.created_at_unix < self.signed_launch_grant.grant.not_before_unix
            || self.expires_at_unix > self.signed_launch_grant.grant.expires_at_unix
        {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        self.target.validate(&self.artifact)?;
        if self
            .signed_launch_grant
            .grant
            .integration_scopes
            .binary_search(&self.target.family.integration_scope())
            .is_err()
        {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        let artifact_digest = self.artifact_digest()?;
        if expected_artifact_digest.is_some_and(|expected| expected != artifact_digest) {
            return Err(PlatformError::InstallPlanTargetMismatch);
        }
        self.validate_roots()?;
        self.validate_operations(&artifact_digest)?;
        if self.approvals_required.len() > MAX_APPROVALS
            || !is_sorted_unique(&self.approvals_required)
            || self
                .approvals_required
                .binary_search(&ApprovalRequirement::FilesystemWrite)
                .is_err()
            || (self.operations.iter().any(|operation| {
                matches!(
                    operation.action,
                    InstallActionV1::RegisterPluginSource { .. }
                        | InstallActionV1::RegisterLiteMcp { .. }
                )
            }) && self
                .approvals_required
                .binary_search(&ApprovalRequirement::ClientConfigChange)
                .is_err())
            || (self.outstanding_host_action.is_some()
                && self
                    .approvals_required
                    .binary_search(&ApprovalRequirement::HostTrust)
                    .is_err())
        {
            return Err(PlatformError::InvalidInstallPlan);
        }
        if !self.semantic_digest_sha256.is_empty()
            && !is_lower_hex(&self.semantic_digest_sha256, 64)
        {
            return Err(PlatformError::InvalidInstallPlan);
        }
        Ok(())
    }

    fn validate_roots(&self) -> Result<(), PlatformError> {
        if self.allowed_roots.is_empty() || self.allowed_roots.len() > MAX_ALLOWED_ROOTS {
            return Err(PlatformError::InvalidInstallPlan);
        }
        for (index, root) in self.allowed_roots.iter().enumerate() {
            if !valid_identifier(&root.id, MAX_IDENTIFIER_BYTES)
                || (index > 0 && self.allowed_roots[index - 1].id >= root.id)
                || !root_allowed_for_target(root.root, &self.target)
            {
                return Err(PlatformError::InvalidInstallPlan);
            }
        }
        let mut symbolic = self
            .allowed_roots
            .iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        symbolic.sort_unstable();
        if !is_sorted_unique(&symbolic) {
            return Err(PlatformError::InvalidInstallPlan);
        }
        Ok(())
    }

    fn validate_operations(&self, artifact_digest: &str) -> Result<(), PlatformError> {
        if self.operations.is_empty() || self.operations.len() > MAX_OPERATIONS {
            return Err(PlatformError::InvalidInstallPlan);
        }
        for (index, operation) in self.operations.iter().enumerate() {
            if !valid_identifier(&operation.operation_id, MAX_IDENTIFIER_BYTES)
                || (index > 0 && self.operations[index - 1].operation_id >= operation.operation_id)
                || !is_lower_hex(&operation.observed_state_sha256, 64)
            {
                return Err(PlatformError::InvalidInstallPlan);
            }
            if self.operations[..index].iter().any(|prior| {
                prior.action.root_id() == operation.action.root_id()
                    && prior.action.entry_key() == operation.action.entry_key()
            }) {
                return Err(PlatformError::InvalidInstallPlan);
            }
            self.validate_action(&operation.action, artifact_digest)?;
            self.validate_action(&operation.inverse, artifact_digest)?;
            validate_state(&operation.precondition, artifact_digest)?;
            validate_state(&operation.postcondition, artifact_digest)?;
            validate_inverse(operation)?;
        }
        Ok(())
    }

    fn validate_action(
        &self,
        action: &InstallActionV1,
        artifact_digest: &str,
    ) -> Result<(), PlatformError> {
        let root = self
            .allowed_roots
            .iter()
            .find(|root| root.id == action.root_id())
            .ok_or(PlatformError::InvalidInstallPlan)?;
        if !valid_entry_key(action.entry_key()) {
            return Err(PlatformError::InvalidInstallPlan);
        }
        action.ownership().validate(artifact_digest)?;
        match action {
            InstallActionV1::MaterializeResources {
                relative_path,
                content_root_sha256,
                ..
            } => {
                if root.root != SymbolicRoot::QiongliManagedData
                    || !valid_relative_path(relative_path)
                    || !is_lower_hex(content_root_sha256, 64)
                {
                    return Err(PlatformError::InvalidInstallPlan);
                }
            }
            InstallActionV1::InstallNativePayload {
                relative_path,
                release_envelope_sha256,
                archive_sha256,
                manifest_sha256,
                pack_sha256,
                artifact_content_root_sha256,
                binary_sha256,
                ..
            } => {
                if root.root != SymbolicRoot::QiongliManagedData
                    || !valid_relative_path(relative_path)
                    || relative_path.contains('/')
                    || !is_lower_hex(release_envelope_sha256, 64)
                    || !is_lower_hex(archive_sha256, 64)
                    || !is_lower_hex(manifest_sha256, 64)
                    || !is_lower_hex(pack_sha256, 64)
                    || !is_lower_hex(artifact_content_root_sha256, 64)
                    || !is_lower_hex(binary_sha256, 64)
                    || pack_sha256 != &self.signed_launch_grant.grant.resource_pack_sha256
                    || binary_sha256 != &self.signed_launch_grant.grant.binary_sha256
                {
                    return Err(PlatformError::InvalidInstallPlan);
                }
            }
            InstallActionV1::RegisterPluginSource {
                source_id,
                source_digest_sha256,
                ..
            } => {
                if !is_plugin_root(root.root)
                    || !valid_identifier(source_id, MAX_IDENTIFIER_BYTES)
                    || !is_lower_hex(source_digest_sha256, 64)
                {
                    return Err(PlatformError::InvalidInstallPlan);
                }
            }
            InstallActionV1::RegisterLiteMcp {
                executable_relative_path,
                arguments,
                binary_sha256,
                ..
            } => {
                if !is_client_config_root(root.root)
                    || !valid_relative_path(executable_relative_path)
                    || arguments
                        .iter()
                        .map(String::as_str)
                        .ne(LITE_MCP_ARGUMENTS.iter().copied())
                    || !is_lower_hex(binary_sha256, 64)
                    || binary_sha256 != &self.signed_launch_grant.grant.binary_sha256
                {
                    return Err(PlatformError::InvalidInstallPlan);
                }
            }
            InstallActionV1::RemoveManagedEntry {
                expected_sha256, ..
            } => {
                if !is_lower_hex(expected_sha256, 64) {
                    return Err(PlatformError::InvalidInstallPlan);
                }
            }
        }
        Ok(())
    }

    fn artifact_digest(&self) -> Result<String, PlatformError> {
        Ok(sha256_hex(&launch_grant_signing_bytes(
            &self.signed_launch_grant.grant,
        )?))
    }

    fn verify_semantic_digest(&self) -> Result<(), PlatformError> {
        if self.semantic_digest_sha256 != self.compute_semantic_digest()? {
            return Err(PlatformError::InstallPlanDigestMismatch);
        }
        Ok(())
    }

    fn compute_semantic_digest(&self) -> Result<String, PlatformError> {
        let semantics = SemanticPlanV1 {
            schema_version: self.schema_version,
            artifact: &self.artifact,
            signed_launch_grant: &self.signed_launch_grant,
            target: &self.target,
            allowed_roots: &self.allowed_roots,
            operations: &self.operations,
            approvals_required: &self.approvals_required,
            outstanding_host_action: self.outstanding_host_action,
        };
        let canonical = serde_json_canonicalizer::to_vec(&semantics)
            .map_err(|_| PlatformError::CanonicalSerializationFailed)?;
        Ok(sha256_hex(&canonical))
    }
}

#[derive(Serialize)]
struct SemanticPlanV1<'a> {
    schema_version: u32,
    artifact: &'a ArtifactIdentityV1,
    signed_launch_grant: &'a SignedLaunchGrantV1,
    target: &'a TargetDescriptorV1,
    allowed_roots: &'a [AllowedRootV1],
    operations: &'a [InstallOperationV1],
    approvals_required: &'a [ApprovalRequirement],
    outstanding_host_action: Option<HostAction>,
}

#[derive(Clone, Debug)]
pub struct VerifiedInstallPlan {
    plan: InstallPlanV1,
    grant: VerifiedLaunchGrant,
}

impl VerifiedInstallPlan {
    #[must_use]
    pub fn plan(&self) -> &InstallPlanV1 {
        &self.plan
    }

    #[must_use]
    pub fn grant(&self) -> &VerifiedLaunchGrant {
        &self.grant
    }
}

/// Returns the canonical digest used by a planner to bind an observed
/// missing or managed state into an install operation.
pub fn observed_plan_state_sha256(state: &PlanStateV1) -> Result<String, PlatformError> {
    match state {
        PlanStateV1::Missing => {}
        PlanStateV1::Managed {
            ownership,
            content_sha256,
        } => {
            ownership.validate_shape()?;
            if !is_lower_hex(content_sha256, 64) {
                return Err(PlatformError::InvalidInstallPlan);
            }
        }
    }
    let canonical = serde_json_canonicalizer::to_vec(state)
        .map_err(|_| PlatformError::CanonicalSerializationFailed)?;
    Ok(sha256_hex(&canonical))
}

fn validate_state(state: &PlanStateV1, artifact_digest: &str) -> Result<(), PlatformError> {
    match state {
        PlanStateV1::Missing => Ok(()),
        PlanStateV1::Managed {
            ownership,
            content_sha256,
        } => {
            ownership.validate(artifact_digest)?;
            if is_lower_hex(content_sha256, 64) {
                Ok(())
            } else {
                Err(PlatformError::InvalidInstallPlan)
            }
        }
    }
}

fn validate_inverse(operation: &InstallOperationV1) -> Result<(), PlatformError> {
    if operation.action.root_id() != operation.inverse.root_id()
        || operation.action.entry_key() != operation.inverse.entry_key()
        || operation.action.ownership() != operation.inverse.ownership()
        || operation.action.is_remove() == operation.inverse.is_remove()
    {
        return Err(PlatformError::InvalidInstallPlan);
    }
    match (
        &operation.action,
        &operation.precondition,
        &operation.postcondition,
    ) {
        (
            InstallActionV1::RemoveManagedEntry {
                expected_sha256, ..
            },
            PlanStateV1::Managed {
                ownership,
                content_sha256,
            },
            PlanStateV1::Missing,
        ) if ownership == operation.action.ownership() && expected_sha256 == content_sha256 => {}
        (
            _,
            precondition,
            PlanStateV1::Managed {
                ownership,
                content_sha256,
            },
        ) if !operation.action.is_remove()
            && ownership == operation.action.ownership()
            && precondition_matches_owner(precondition, operation.action.ownership())
            && matches!(
                &operation.inverse,
                InstallActionV1::RemoveManagedEntry {
                    expected_sha256,
                    ..
                } if expected_sha256 == content_sha256
            ) => {}
        _ => return Err(PlatformError::InvalidInstallPlan),
    }
    Ok(())
}

fn precondition_matches_owner(precondition: &PlanStateV1, expected: &OwnershipMarkerV1) -> bool {
    match precondition {
        PlanStateV1::Missing => true,
        PlanStateV1::Managed { ownership, .. } => ownership == expected,
    }
}

const fn root_allowed_for_target(root: SymbolicRoot, target: &TargetDescriptorV1) -> bool {
    match root {
        SymbolicRoot::QiongliManagedData => true,
        SymbolicRoot::CodexPersonalMarketplace | SymbolicRoot::CodexConfig => {
            matches!(target.family, LocalTargetFamily::CodexLocal)
                && matches!(target.scope, InstallScope::User)
        }
        SymbolicRoot::CodexRepositoryMarketplace => {
            matches!(target.family, LocalTargetFamily::CodexLocal)
                && matches!(target.scope, InstallScope::Repository)
        }
        SymbolicRoot::ClaudeSkillsDirectory | SymbolicRoot::ClaudeCodeConfig => {
            matches!(target.family, LocalTargetFamily::ClaudeCodeLocal)
                && matches!(target.scope, InstallScope::User)
        }
        SymbolicRoot::ClaudeMarketplaceSource => {
            matches!(target.family, LocalTargetFamily::ClaudeCodeLocal)
                && matches!(target.scope, InstallScope::User)
        }
    }
}

const fn is_plugin_root(root: SymbolicRoot) -> bool {
    matches!(
        root,
        SymbolicRoot::CodexPersonalMarketplace
            | SymbolicRoot::CodexRepositoryMarketplace
            | SymbolicRoot::ClaudeSkillsDirectory
            | SymbolicRoot::ClaudeMarketplaceSource
    )
}

const fn is_client_config_root(root: SymbolicRoot) -> bool {
    matches!(
        root,
        SymbolicRoot::CodexConfig | SymbolicRoot::ClaudeCodeConfig
    )
}

fn valid_entry_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn valid_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_RELATIVE_PATH_BYTES
        && !value.starts_with('/')
        && !value.contains('\\')
        && value.split('/').all(valid_portable_component)
}

fn valid_portable_component(component: &str) -> bool {
    if component.is_empty()
        || matches!(component, "." | "..")
        || component.ends_with('.')
        || !component
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return false;
    }
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _)| stem)
        .to_ascii_lowercase();
    !matches!(
        stem.as_str(),
        "con"
            | "prn"
            | "aux"
            | "nul"
            | "com1"
            | "com2"
            | "com3"
            | "com4"
            | "com5"
            | "com6"
            | "com7"
            | "com8"
            | "com9"
            | "lpt1"
            | "lpt2"
            | "lpt3"
            | "lpt4"
            | "lpt5"
            | "lpt6"
            | "lpt7"
            | "lpt8"
            | "lpt9"
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use ed25519_dalek::{Signer, SigningKey};
    use sha2::Digest;

    use super::*;
    use crate::{
        GrantMode, GrantSignatureV1, InstallerKind, LaunchGrantV1, ReleaseChannel,
        SignatureAlgorithm, launch_grant_signing_bytes,
    };

    const BINARY_DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const PACK_DIGEST: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const STATE_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    const RESULT_DIGEST: &str = "4444444444444444444444444444444444444444444444444444444444444444";
    static NEXT_KEY_ID: AtomicU64 = AtomicU64::new(0);

    fn artifact() -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.1".to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
            installer_kind: InstallerKind::PortableArchive,
        }
    }

    fn signed_and_key() -> (SignedLaunchGrantV1, TrustedPublicKey) {
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: 4,
            artifact: artifact(),
            binary_sha256: BINARY_DIGEST.to_string(),
            resource_pack_sha256: PACK_DIGEST.to_string(),
            allowed_modes: vec![GrantMode::Cli, GrantMode::LiteMcp],
            integration_scopes: vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
            not_before_unix: 1_700_000_000,
            expires_at_unix: 1_800_000_000,
        };
        let key_pair = temporary_test_signing_key();
        let signature = key_pair.sign(&launch_grant_signing_bytes(&grant).unwrap());
        let public_key = key_pair.verifying_key().to_bytes();
        (
            SignedLaunchGrantV1 {
                grant,
                signature: GrantSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "plan-test-key".to_string(),
                    value_hex: encode_hex(&signature.to_bytes()),
                },
            },
            TrustedPublicKey::new("plan-test-key", public_key).unwrap(),
        )
    }

    fn verification_context(artifact: &ArtifactIdentityV1) -> GrantVerificationContext<'_> {
        GrantVerificationContext {
            now_unix: 1_750_000_000,
            minimum_generation: 4,
            expected_artifact: artifact,
            binary_sha256: BINARY_DIGEST,
            resource_pack_sha256: PACK_DIGEST,
            requested_mode: GrantMode::LiteMcp,
            requested_scope: IntegrationScope::CodexLocal,
        }
    }

    fn verified() -> (VerifiedLaunchGrant, TrustedPublicKey) {
        let (signed, key) = signed_and_key();
        let expected = artifact();
        let verified = signed
            .verify(std::slice::from_ref(&key), &verification_context(&expected))
            .unwrap();
        (verified, key)
    }

    fn ownership(verified: &VerifiedLaunchGrant) -> OwnershipMarkerV1 {
        OwnershipMarkerV1 {
            schema_version: 1,
            product: ProductId::Qiongli,
            install_id: "qiongli-lite-user".to_string(),
            artifact_digest_sha256: verified.signed_payload_sha256().to_string(),
        }
    }

    fn draft(verified: &VerifiedLaunchGrant) -> InstallPlanDraftV1 {
        let ownership = ownership(verified);
        InstallPlanDraftV1 {
            target: TargetDescriptorV1 {
                family: LocalTargetFamily::CodexLocal,
                surface: LocalSurface::CliLocal,
                scope: InstallScope::User,
                profile: CapabilityProfile::Lite,
                os: OperatingSystem::Linux,
                arch: Architecture::X86_64,
                adapter_version: 1,
            },
            allowed_roots: vec![
                AllowedRootV1 {
                    id: "codex-config".to_string(),
                    root: SymbolicRoot::CodexConfig,
                },
                AllowedRootV1 {
                    id: "qiongli-data".to_string(),
                    root: SymbolicRoot::QiongliManagedData,
                },
            ],
            operations: vec![InstallOperationV1 {
                operation_id: "register-lite-mcp".to_string(),
                action: InstallActionV1::RegisterLiteMcp {
                    root_id: "codex-config".to_string(),
                    entry_key: "qiongli-lite".to_string(),
                    executable_relative_path: "bin/qiongli".to_string(),
                    arguments: LITE_MCP_ARGUMENTS.iter().map(ToString::to_string).collect(),
                    binary_sha256: BINARY_DIGEST.to_string(),
                    ownership: ownership.clone(),
                },
                precondition: PlanStateV1::Missing,
                observed_state_sha256: STATE_DIGEST.to_string(),
                postcondition: PlanStateV1::Managed {
                    ownership: ownership.clone(),
                    content_sha256: RESULT_DIGEST.to_string(),
                },
                inverse: InstallActionV1::RemoveManagedEntry {
                    root_id: "codex-config".to_string(),
                    entry_key: "qiongli-lite".to_string(),
                    expected_ownership: ownership,
                    expected_sha256: RESULT_DIGEST.to_string(),
                },
            }],
            approvals_required: vec![
                ApprovalRequirement::FilesystemWrite,
                ApprovalRequirement::ClientConfigChange,
                ApprovalRequirement::HostTrust,
            ],
            outstanding_host_action: Some(HostAction::ApproveMcp),
        }
    }

    fn metadata(plan_id: &str, created: u64) -> InstallPlanMetadataV1 {
        InstallPlanMetadataV1 {
            plan_id: plan_id.to_string(),
            created_at_unix: created,
            expires_at_unix: created + 600,
        }
    }

    #[test]
    fn equivalent_semantics_have_one_digest_and_verify() {
        let (verified, key) = verified();
        let first = InstallPlanV1::build(
            metadata("preview-one", 1_750_000_000),
            &verified,
            draft(&verified),
        )
        .unwrap();
        let second = InstallPlanV1::build(
            metadata("preview-two", 1_750_000_100),
            &verified,
            draft(&verified),
        )
        .unwrap();
        assert_eq!(first.semantic_digest_sha256, second.semantic_digest_sha256);

        let expected = artifact();
        let parsed = InstallPlanV1::from_json(&first.to_canonical_json().unwrap()).unwrap();
        let checked = parsed
            .verify(&[key], &verification_context(&expected))
            .unwrap();
        assert_eq!(checked.plan().plan_id, "preview-one");
        assert_eq!(checked.grant().grant().generation, 4);
    }

    #[test]
    fn semantic_changes_change_digest() {
        let (verified, _) = verified();
        let first = InstallPlanV1::build(
            metadata("preview-one", 1_750_000_000),
            &verified,
            draft(&verified),
        )
        .unwrap();
        let mut changed = draft(&verified);
        changed.outstanding_host_action = Some(HostAction::RestartClient);
        let second =
            InstallPlanV1::build(metadata("preview-two", 1_750_000_000), &verified, changed)
                .unwrap();
        assert_ne!(first.semantic_digest_sha256, second.semantic_digest_sha256);
    }

    #[test]
    fn roots_paths_arguments_and_inverse_fail_closed() {
        let (verified, _) = verified();
        let mut invalid = draft(&verified);
        if let InstallActionV1::RegisterLiteMcp {
            executable_relative_path,
            ..
        } = &mut invalid.operations[0].action
        {
            *executable_relative_path = "../private-canary".to_string();
        }
        assert_eq!(
            InstallPlanV1::build(metadata("bad-path", 1_750_000_000), &verified, invalid)
                .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );

        let mut invalid = draft(&verified);
        if let InstallActionV1::RegisterLiteMcp {
            executable_relative_path,
            ..
        } = &mut invalid.operations[0].action
        {
            *executable_relative_path = "CON/qiongli".to_string();
        }
        assert_eq!(
            InstallPlanV1::build(metadata("reserved-path", 1_750_000_000), &verified, invalid,)
                .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );

        let mut invalid = draft(&verified);
        invalid.allowed_roots[0].root = SymbolicRoot::ClaudeCodeConfig;
        assert_eq!(
            InstallPlanV1::build(metadata("bad-root", 1_750_000_000), &verified, invalid)
                .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );

        let mut invalid = draft(&verified);
        invalid.operations[0].inverse = invalid.operations[0].action.clone();
        assert_eq!(
            InstallPlanV1::build(metadata("bad-inverse", 1_750_000_000), &verified, invalid)
                .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );

        let mut invalid = draft(&verified);
        let mut foreign_owner = ownership(&verified);
        foreign_owner.install_id = "another-install".to_string();
        invalid.operations[0].precondition = PlanStateV1::Managed {
            ownership: foreign_owner,
            content_sha256: STATE_DIGEST.to_string(),
        };
        assert_eq!(
            InstallPlanV1::build(metadata("foreign-owner", 1_750_000_000), &verified, invalid,)
                .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );

        let mut invalid = draft(&verified);
        invalid.approvals_required.clear();
        assert_eq!(
            InstallPlanV1::build(
                metadata("missing-approval", 1_750_000_000),
                &verified,
                invalid,
            )
            .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );

        let mut invalid = draft(&verified);
        let mut duplicate = invalid.operations[0].clone();
        duplicate.operation_id = "register-lite-mcp-again".to_string();
        invalid.operations.push(duplicate);
        assert_eq!(
            InstallPlanV1::build(
                metadata("duplicate-target", 1_750_000_000),
                &verified,
                invalid,
            )
            .unwrap_err(),
            PlatformError::InvalidInstallPlan
        );
    }

    #[test]
    fn verified_capability_cannot_be_reused_for_another_mode_or_scope() {
        let (signed, key) = signed_and_key();
        let expected = artifact();

        let mut context = verification_context(&expected);
        context.requested_scope = IntegrationScope::ClaudeCodeLocal;
        let claude_token = signed.verify(std::slice::from_ref(&key), &context).unwrap();
        assert_eq!(
            InstallPlanV1::build(
                metadata("wrong-scope", 1_750_000_000),
                &claude_token,
                draft(&claude_token),
            )
            .unwrap_err(),
            PlatformError::InstallPlanTargetMismatch
        );

        context = verification_context(&expected);
        context.requested_mode = GrantMode::Cli;
        let cli_token = signed.verify(&[key], &context).unwrap();
        assert_eq!(
            InstallPlanV1::build(
                metadata("wrong-mode", 1_750_000_000),
                &cli_token,
                draft(&cli_token),
            )
            .unwrap_err(),
            PlatformError::InstallPlanTargetMismatch
        );
    }

    #[test]
    fn plan_json_is_strict_bounded_digest_checked_and_expiring() {
        let (verified, key) = verified();
        let plan = InstallPlanV1::build(
            metadata("strict-plan", 1_750_000_000),
            &verified,
            draft(&verified),
        )
        .unwrap();
        let mut value = serde_json::to_value(&plan).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("private-canary".to_string(), serde_json::json!(true));
        assert_eq!(
            InstallPlanV1::from_json(&serde_json::to_vec(&value).unwrap()).unwrap_err(),
            PlatformError::InvalidInstallPlanJson
        );
        assert_eq!(
            InstallPlanV1::from_json(&vec![b' '; MAX_INSTALL_PLAN_BYTES + 1]).unwrap_err(),
            PlatformError::InstallPlanTooLarge
        );

        let mut tampered = plan.clone();
        tampered.outstanding_host_action = Some(HostAction::RestartClient);
        assert_eq!(
            tampered.to_canonical_json().unwrap_err(),
            PlatformError::InstallPlanDigestMismatch
        );

        let expected = artifact();
        let mut context = verification_context(&expected);
        context.now_unix = plan.created_at_unix - 1;
        assert_eq!(
            plan.verify(std::slice::from_ref(&key), &context)
                .unwrap_err(),
            PlatformError::InstallPlanNotYetValid
        );
        context.now_unix = plan.expires_at_unix;
        assert_eq!(
            plan.verify(&[key], &context).unwrap_err(),
            PlatformError::InstallPlanExpired
        );
    }

    fn encode_hex(input: &[u8]) -> String {
        let mut output = String::with_capacity(input.len() * 2);
        for byte in input {
            use std::fmt::Write;
            write!(&mut output, "{byte:02x}").unwrap();
        }
        output
    }

    fn temporary_test_signing_key() -> SigningKey {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let sequence = NEXT_KEY_ID.fetch_add(1, Ordering::Relaxed);
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"qiongli-r3a-ephemeral-plan-test-key");
        hasher.update(nonce.to_le_bytes());
        hasher.update(std::process::id().to_le_bytes());
        hasher.update(sequence.to_le_bytes());
        SigningKey::from_bytes(&hasher.finalize().into())
    }
}
