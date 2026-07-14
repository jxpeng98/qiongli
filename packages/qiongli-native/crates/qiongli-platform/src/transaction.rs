use std::fmt::{self, Debug, Display, Formatter};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
#[cfg(not(any(unix, windows)))]
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    LoadedResourcePack, MaterializationReceiptV1, MaterializationTarget, ProfileId,
    approve_materialization_target, materialize_profile, verify_materialization,
};
use same_file::Handle;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    AllowedRootV1, ApprovalRequirement, ArtifactIdentityV1, CapabilityProfile, HostAction,
    InstallActionV1, InstallOperationV1, InstallScope, OwnershipMarkerV1, PlanStateV1, ProductId,
    SymbolicRoot, TargetDescriptorV1, VerifiedInstallPlan, observed_plan_state_sha256,
};

pub const INSTALL_RECEIPT_SCHEMA_VERSION: u32 = 1;
pub const MANAGED_INSTALL_STATE_SCHEMA_VERSION: u32 = 1;
pub const INSTALL_JOURNAL_SCHEMA_VERSION: u32 = 1;

const MAX_STATE_BYTES: u64 = 1024 * 1024;
const MAX_IDENTIFIER_BYTES: usize = 128;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const STATE_FILE_PREFIX: &str = ".qiongli-install-";
const STATE_FILE_SUFFIX: &str = ".json";
const JOURNAL_FILE_NAME: &str = ".qiongli-transaction.json";
const QUARANTINE_MARKER: &str = ".qiongli-quarantine-";
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionError {
    UnsupportedPlatform,
    InvalidApproval,
    UnsafeManagedRoot,
    UnsupportedPlan,
    PlanExpired,
    PayloadMismatch,
    ObservedStateMismatch,
    DestinationConflict,
    ReceiptMissing,
    InvalidReceipt,
    ManagedStateDrift,
    RecoveryRequired,
    MaterializationFailed,
    PersistenceFailed(io::ErrorKind),
    RollbackConflict,
}

impl TransactionError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "transaction-platform-unsupported",
            Self::InvalidApproval => "install-approval-invalid",
            Self::UnsafeManagedRoot => "managed-root-unsafe",
            Self::UnsupportedPlan => "install-plan-not-executable",
            Self::PlanExpired => "install-plan-expired",
            Self::PayloadMismatch => "install-payload-mismatch",
            Self::ObservedStateMismatch => "install-observed-state-mismatch",
            Self::DestinationConflict => "install-destination-conflict",
            Self::ReceiptMissing => "install-receipt-missing",
            Self::InvalidReceipt => "install-receipt-invalid",
            Self::ManagedStateDrift => "managed-install-drift",
            Self::RecoveryRequired => "install-recovery-required",
            Self::MaterializationFailed => "resource-materialization-failed",
            Self::PersistenceFailed(_) => "install-persistence-failed",
            Self::RollbackConflict => "install-rollback-conflict",
        }
    }
}

impl Display for TransactionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())?;
        if let Self::PersistenceFailed(kind) = self {
            write!(formatter, " ({kind:?})")?;
        }
        Ok(())
    }
}

impl std::error::Error for TransactionError {}

#[derive(Clone)]
pub struct ApprovedInstallPlan {
    semantic_digest_sha256: String,
    expires_at_unix: u64,
    approvals: Vec<ApprovalRequirement>,
}

impl Debug for ApprovedInstallPlan {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApprovedInstallPlan")
            .field("semantic_digest_sha256", &"<bound-plan-digest>")
            .field("expires_at_unix", &self.expires_at_unix)
            .field("approvals", &self.approvals)
            .finish()
    }
}

/// Creates an approval token at a trusted local CLI or UI confirmation
/// boundary. Callers must not invoke this function directly from an MCP or
/// model-generated request.
pub fn approve_install_plan(
    plan: &VerifiedInstallPlan,
    approvals: &[ApprovalRequirement],
    now_unix: u64,
) -> Result<ApprovedInstallPlan, TransactionError> {
    let plan = plan.plan();
    if now_unix < plan.created_at_unix || now_unix >= plan.expires_at_unix {
        return Err(TransactionError::PlanExpired);
    }
    if approvals != plan.approvals_required {
        return Err(TransactionError::InvalidApproval);
    }
    Ok(ApprovedInstallPlan {
        semantic_digest_sha256: plan.semantic_digest_sha256.clone(),
        expires_at_unix: plan.expires_at_unix,
        approvals: approvals.to_vec(),
    })
}

#[derive(Clone)]
pub struct ApprovedManagedRoot {
    root_id: String,
    path: PathBuf,
    identity: Arc<Handle>,
}

impl ApprovedManagedRoot {
    #[must_use]
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn validate(&self) -> Result<(), TransactionError> {
        validate_managed_root(&self.path)?;
        let current =
            Handle::from_path(&self.path).map_err(|_| TransactionError::UnsafeManagedRoot)?;
        if current != *self.identity {
            return Err(TransactionError::UnsafeManagedRoot);
        }
        Ok(())
    }
}

impl Debug for ApprovedManagedRoot {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApprovedManagedRoot")
            .field("root_id", &self.root_id)
            .field("path", &"<approved-managed-root>")
            .finish()
    }
}

/// Approves one caller-selected private managed root. The root must already
/// exist and be owned by the current user.
pub fn approve_managed_root(
    root: &AllowedRootV1,
    path: impl AsRef<Path>,
) -> Result<ApprovedManagedRoot, TransactionError> {
    if root.root != SymbolicRoot::QiongliManagedData
        || !valid_identifier(&root.id)
        || !path.as_ref().is_absolute()
        || has_lexical_traversal(path.as_ref())
    {
        return Err(TransactionError::UnsafeManagedRoot);
    }
    validate_managed_root(path.as_ref())?;
    let identity =
        Handle::from_path(path.as_ref()).map_err(|_| TransactionError::UnsafeManagedRoot)?;
    Ok(ApprovedManagedRoot {
        root_id: root.id.clone(),
        path: path.as_ref().to_path_buf(),
        identity: Arc::new(identity),
    })
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedOperationReceiptV1 {
    pub operation_id: String,
    pub root_id: String,
    pub entry_key: String,
    pub relative_path: String,
    pub ownership: OwnershipMarkerV1,
    pub pack_sha256: String,
    pub content_root_sha256: String,
    pub materialization_receipt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallReceiptV1 {
    pub schema_version: u32,
    pub transaction_id: String,
    pub plan_id: String,
    pub semantic_digest_sha256: String,
    pub install_id: String,
    pub artifact: ArtifactIdentityV1,
    pub target: TargetDescriptorV1,
    pub operation: ManagedOperationReceiptV1,
    pub applied_at_unix: u64,
    pub replaces_transaction_id: Option<String>,
    pub outstanding_host_action: Option<HostAction>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallLifecycleKind {
    Removed,
    RolledBack,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallLifecycleReceiptV1 {
    pub schema_version: u32,
    pub transaction_id: String,
    pub install_id: String,
    pub prior_transaction_id: String,
    pub kind: InstallLifecycleKind,
    pub completed_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedInstallStateV1 {
    pub schema_version: u32,
    pub generation: u64,
    pub install_id: String,
    pub active: Option<InstallReceiptV1>,
    pub last_lifecycle: Option<InstallLifecycleReceiptV1>,
}

impl ManagedInstallStateV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, TransactionError> {
        if input.len() as u64 > MAX_STATE_BYTES {
            return Err(TransactionError::InvalidReceipt);
        }
        let state: Self =
            serde_json::from_slice(input).map_err(|_| TransactionError::InvalidReceipt)?;
        state.validate()?;
        let canonical = canonical_json(&state)?;
        if canonical != input {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(state)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, TransactionError> {
        self.validate()?;
        let bytes = canonical_json(self)?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(bytes)
    }

    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != MANAGED_INSTALL_STATE_SCHEMA_VERSION
            || self.generation == 0
            || self.generation > JCS_MAX_SAFE_INTEGER
            || !valid_identifier(&self.install_id)
            || (self.active.is_none() && self.last_lifecycle.is_none())
        {
            return Err(TransactionError::InvalidReceipt);
        }
        if let Some(active) = &self.active {
            active.validate()?;
            if active.install_id != self.install_id {
                return Err(TransactionError::InvalidReceipt);
            }
        }
        if let Some(lifecycle) = &self.last_lifecycle {
            lifecycle.validate()?;
            if lifecycle.install_id != self.install_id {
                return Err(TransactionError::InvalidReceipt);
            }
        }
        Ok(())
    }
}

impl InstallReceiptV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != INSTALL_RECEIPT_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.plan_id)
            || !valid_lower_hex(&self.semantic_digest_sha256, 64)
            || !valid_identifier(&self.install_id)
            || self.applied_at_unix > JCS_MAX_SAFE_INTEGER
            || self.outstanding_host_action.is_some()
            || self.target.profile != CapabilityProfile::Lite
            || self.target.scope != InstallScope::User
            || self.target.os != self.artifact.os
            || self.target.arch != self.artifact.arch
            || self.target.adapter_version != 1
            || self.operation.ownership.install_id != self.install_id
            || self.operation.ownership.product != ProductId::Qiongli
            || self.operation.ownership.schema_version != 1
            || !valid_lower_hex(&self.operation.ownership.artifact_digest_sha256, 64)
            || self
                .replaces_transaction_id
                .as_ref()
                .is_some_and(|value| !valid_identifier(value))
        {
            return Err(TransactionError::InvalidReceipt);
        }
        self.artifact
            .validate()
            .map_err(|_| TransactionError::InvalidReceipt)?;
        self.operation.validate()
    }
}

impl ManagedOperationReceiptV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if !valid_identifier(&self.operation_id)
            || !valid_identifier(&self.root_id)
            || !valid_entry_key(&self.entry_key)
            || !valid_target_leaf(&self.relative_path)
            || !valid_lower_hex(&self.pack_sha256, 64)
            || !valid_lower_hex(&self.content_root_sha256, 64)
            || !valid_lower_hex(&self.materialization_receipt_sha256, 64)
        {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(())
    }
}

impl InstallLifecycleReceiptV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != INSTALL_RECEIPT_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.install_id)
            || !valid_identifier(&self.prior_transaction_id)
            || self.completed_at_unix > JCS_MAX_SAFE_INTEGER
        {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallDisposition {
    Applied,
    AlreadyApplied,
    Repaired,
    AlreadyHealthy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallCommit {
    pub disposition: InstallDisposition,
    pub receipt: InstallReceiptV1,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallVerification {
    pub receipt: InstallReceiptV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleDisposition {
    Removed,
    AlreadyRemoved,
    RolledBack,
    AlreadyRolledBack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleCommit {
    pub disposition: LifecycleDisposition,
    pub receipt: InstallLifecycleReceiptV1,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug)]
pub struct ManagedResourceExecutor {
    root: ApprovedManagedRoot,
}

impl ManagedResourceExecutor {
    #[must_use]
    pub const fn new(root: ApprovedManagedRoot) -> Self {
        Self { root }
    }

    pub fn apply(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        now_unix: u64,
    ) -> Result<InstallCommit, TransactionError> {
        self.apply_with(plan, approval, pack, now_unix, ApplyKind::Fresh, &NoFaults)
    }

    pub fn repair(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        now_unix: u64,
    ) -> Result<InstallCommit, TransactionError> {
        self.apply_with(plan, approval, pack, now_unix, ApplyKind::Repair, &NoFaults)
    }

    pub fn verify(&self, install_id: &str) -> Result<InstallVerification, TransactionError> {
        self.root.validate()?;
        ensure_no_journal(self.root.path(), install_id)?;
        let state =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        let receipt = state.active.ok_or(TransactionError::ReceiptMissing)?;
        verify_active_receipt(self.root.path(), &self.root.root_id, &receipt)?;
        Ok(InstallVerification { receipt })
    }

    pub fn remove(
        &self,
        install_id: &str,
        now_unix: u64,
    ) -> Result<LifecycleCommit, TransactionError> {
        self.lifecycle_with(
            install_id,
            now_unix,
            InstallLifecycleKind::Removed,
            &NoFaults,
        )
    }

    pub fn rollback(
        &self,
        install_id: &str,
        now_unix: u64,
    ) -> Result<LifecycleCommit, TransactionError> {
        self.lifecycle_with(
            install_id,
            now_unix,
            InstallLifecycleKind::RolledBack,
            &NoFaults,
        )
    }

    fn apply_with<F: TransactionFaults>(
        &self,
        verified_plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        now_unix: u64,
        kind: ApplyKind,
        faults: &F,
    ) -> Result<InstallCommit, TransactionError> {
        self.root.validate()?;
        let executable = ExecutableMaterialization::from_plan(
            verified_plan,
            approval,
            pack,
            &self.root,
            now_unix,
        )?;
        ensure_no_journal(self.root.path(), executable.install_id())?;
        let prior_state = load_state(self.root.path(), executable.install_id())?;

        match kind {
            ApplyKind::Fresh => {
                if let Some(active) = prior_state.as_ref().and_then(|state| state.active.as_ref()) {
                    if active.semantic_digest_sha256 == executable.plan_digest {
                        verify_active_receipt(self.root.path(), &self.root.root_id, active)?;
                        return Ok(InstallCommit {
                            disposition: InstallDisposition::AlreadyApplied,
                            receipt: active.clone(),
                            cleanup_required: false,
                        });
                    }
                    return Err(TransactionError::DestinationConflict);
                }
                ensure_destination_absent(&executable.destination)?;
            }
            ApplyKind::Repair => {
                let active = prior_state
                    .as_ref()
                    .and_then(|state| state.active.as_ref())
                    .ok_or(TransactionError::ReceiptMissing)?;
                if !executable.matches_active(active) {
                    return Err(TransactionError::UnsupportedPlan);
                }
                if path_exists(&executable.destination)? {
                    verify_active_receipt(self.root.path(), &self.root.root_id, active)?;
                    return Ok(InstallCommit {
                        disposition: InstallDisposition::AlreadyHealthy,
                        receipt: active.clone(),
                        cleanup_required: false,
                    });
                }
            }
        }

        let transaction_id = transaction_id();
        let prior_state_sha256 = prior_state
            .as_ref()
            .map(ManagedInstallStateV1::to_canonical_json)
            .transpose()?
            .as_deref()
            .map(sha256_hex);
        let journal = TransactionJournalV1 {
            schema_version: INSTALL_JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            kind: match kind {
                ApplyKind::Fresh => JournalKind::Apply,
                ApplyKind::Repair => JournalKind::Repair,
            },
            install_id: executable.install_id().to_string(),
            plan_digest_sha256: Some(executable.plan_digest.to_string()),
            prior_state_sha256,
            target_leaf: executable.relative_path.to_string(),
            started_at_unix: now_unix,
        };
        let mut journal = TransactionJournalGuard::create(self.root.path(), journal)?;
        if let Err(error) = faults.check(FaultPoint::AfterJournal) {
            journal.finish()?;
            return Err(error);
        }

        if self.root.validate().is_err() {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        ensure_destination_absent(&executable.destination)?;
        if state_digest(self.root.path(), executable.install_id())?
            != prior_state
                .as_ref()
                .map(ManagedInstallStateV1::to_canonical_json)
                .transpose()?
                .as_deref()
                .map(sha256_hex)
        {
            journal.finish()?;
            return Err(TransactionError::ObservedStateMismatch);
        }

        let replaces_transaction_id = prior_state
            .as_ref()
            .and_then(|state| state.active.as_ref())
            .map(|receipt| receipt.transaction_id.clone());
        let target = approved_materialization_target(&executable.destination)?;
        let materialization = match materialize_profile(pack, "lite", &target) {
            Ok(materialization) => materialization,
            Err(_) => {
                return self.fail_apply_after_possible_materialization(
                    &mut journal,
                    &executable,
                    &transaction_id,
                    now_unix,
                    replaces_transaction_id,
                    MaterializedTargetOwnership::Uncertain,
                    faults,
                    TransactionError::MaterializationFailed,
                );
            }
        };
        if faults
            .check(FaultPoint::AmbiguousMaterializationResult)
            .is_err()
        {
            return self.fail_apply_after_possible_materialization(
                &mut journal,
                &executable,
                &transaction_id,
                now_unix,
                replaces_transaction_id,
                MaterializedTargetOwnership::Uncertain,
                faults,
                TransactionError::MaterializationFailed,
            );
        }
        if let Err(error) = faults.check(FaultPoint::AfterMaterialization) {
            return self.fail_apply_after_possible_materialization(
                &mut journal,
                &executable,
                &transaction_id,
                now_unix,
                replaces_transaction_id,
                MaterializedTargetOwnership::CreatedByTransaction,
                faults,
                error,
            );
        }
        if let Err(error) = executable.validate_materialization(&materialization) {
            return self.fail_apply_after_possible_materialization(
                &mut journal,
                &executable,
                &transaction_id,
                now_unix,
                replaces_transaction_id,
                MaterializedTargetOwnership::CreatedByTransaction,
                faults,
                error,
            );
        }
        let verified_materialization = match verify_materialization(&target) {
            Ok(materialization) => materialization,
            Err(_) => {
                return self.fail_apply_after_possible_materialization(
                    &mut journal,
                    &executable,
                    &transaction_id,
                    now_unix,
                    replaces_transaction_id,
                    MaterializedTargetOwnership::CreatedByTransaction,
                    faults,
                    TransactionError::ManagedStateDrift,
                );
            }
        };
        if let Err(error) = executable.validate_materialization(&verified_materialization) {
            return self.fail_apply_after_possible_materialization(
                &mut journal,
                &executable,
                &transaction_id,
                now_unix,
                replaces_transaction_id,
                MaterializedTargetOwnership::CreatedByTransaction,
                faults,
                error,
            );
        }
        let materialization_receipt_sha256 = match canonical_json(&verified_materialization) {
            Ok(bytes) => sha256_hex(&bytes),
            Err(error) => {
                return self.fail_apply_after_possible_materialization(
                    &mut journal,
                    &executable,
                    &transaction_id,
                    now_unix,
                    replaces_transaction_id,
                    MaterializedTargetOwnership::CreatedByTransaction,
                    faults,
                    error,
                );
            }
        };

        let receipt = executable.build_receipt(
            transaction_id.clone(),
            now_unix,
            materialization_receipt_sha256,
            replaces_transaction_id,
        );
        if self.root.validate().is_err() {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        if let Err(error) = faults.check(FaultPoint::BeforeStateCommit) {
            return self.fail_apply_after_materialization(
                &mut journal,
                &receipt,
                &verified_materialization,
                faults,
                error,
            );
        }

        let next_state = ManagedInstallStateV1 {
            schema_version: MANAGED_INSTALL_STATE_SCHEMA_VERSION,
            generation: next_generation(prior_state.as_ref())?,
            install_id: executable.install_id().to_string(),
            active: Some(receipt.clone()),
            last_lifecycle: prior_state.and_then(|state| state.last_lifecycle),
        };
        if let Err(error) = persist_state(self.root.path(), &next_state, &transaction_id, faults) {
            if error == TransactionError::RecoveryRequired {
                journal.retain();
                return Err(error);
            }
            return self.fail_apply_after_materialization(
                &mut journal,
                &receipt,
                &verified_materialization,
                faults,
                error,
            );
        }

        let cleanup_required = finish_committed_journal(&mut journal, faults);
        Ok(InstallCommit {
            disposition: match kind {
                ApplyKind::Fresh => InstallDisposition::Applied,
                ApplyKind::Repair => InstallDisposition::Repaired,
            },
            receipt,
            cleanup_required,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn fail_apply_after_possible_materialization<F: TransactionFaults>(
        &self,
        journal: &mut TransactionJournalGuard,
        executable: &ExecutableMaterialization<'_>,
        transaction_id: &str,
        applied_at_unix: u64,
        replaces_transaction_id: Option<String>,
        target_ownership: MaterializedTargetOwnership,
        faults: &F,
        original_error: TransactionError,
    ) -> Result<InstallCommit, TransactionError> {
        match path_exists(&executable.destination) {
            Ok(false) => {
                journal.finish()?;
                return Err(original_error);
            }
            Ok(true) => {}
            Err(_) => {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
        }
        if target_ownership == MaterializedTargetOwnership::Uncertain {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        let target = match approved_materialization_target(&executable.destination) {
            Ok(target) => target,
            Err(_) => {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
        };
        let materialization = match verify_materialization(&target) {
            Ok(materialization) => materialization,
            Err(_) => {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
        };
        if executable
            .validate_materialization(&materialization)
            .is_err()
        {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        let materialization_receipt_sha256 = match canonical_json(&materialization) {
            Ok(bytes) => sha256_hex(&bytes),
            Err(_) => {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
        };
        let receipt = executable.build_receipt(
            transaction_id.to_string(),
            applied_at_unix,
            materialization_receipt_sha256,
            replaces_transaction_id,
        );
        self.fail_apply_after_materialization(
            journal,
            &receipt,
            &materialization,
            faults,
            original_error,
        )
    }

    fn fail_apply_after_materialization<F: TransactionFaults>(
        &self,
        journal: &mut TransactionJournalGuard,
        receipt: &InstallReceiptV1,
        materialization: &MaterializationReceiptV1,
        faults: &F,
        original_error: TransactionError,
    ) -> Result<InstallCommit, TransactionError> {
        if faults.check(FaultPoint::DuringRollback).is_err()
            || rollback_created_target(
                self.root.path(),
                receipt,
                materialization,
                &receipt.transaction_id,
            )
            .is_err()
        {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        journal.finish()?;
        Err(original_error)
    }

    fn lifecycle_with<F: TransactionFaults>(
        &self,
        install_id: &str,
        now_unix: u64,
        kind: InstallLifecycleKind,
        faults: &F,
    ) -> Result<LifecycleCommit, TransactionError> {
        if now_unix > JCS_MAX_SAFE_INTEGER || !valid_identifier(install_id) {
            return Err(TransactionError::InvalidReceipt);
        }
        self.root.validate()?;
        ensure_no_journal(self.root.path(), install_id)?;
        let prior_state =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        if let Some(lifecycle) = prior_state.last_lifecycle.as_ref()
            && prior_state.active.is_none()
            && lifecycle.kind == kind
        {
            return Ok(LifecycleCommit {
                disposition: idempotent_lifecycle_disposition(kind),
                receipt: lifecycle.clone(),
                cleanup_required: false,
            });
        }
        let active = prior_state
            .active
            .as_ref()
            .ok_or(TransactionError::ReceiptMissing)?;
        if now_unix < active.applied_at_unix {
            return Err(TransactionError::ObservedStateMismatch);
        }
        let materialization = verify_active_receipt(self.root.path(), &self.root.root_id, active)?;
        let transaction_id = transaction_id();
        let journal_value = TransactionJournalV1 {
            schema_version: INSTALL_JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            kind: match kind {
                InstallLifecycleKind::Removed => JournalKind::Remove,
                InstallLifecycleKind::RolledBack => JournalKind::Rollback,
            },
            install_id: install_id.to_string(),
            plan_digest_sha256: Some(active.semantic_digest_sha256.clone()),
            prior_state_sha256: Some(sha256_hex(&prior_state.to_canonical_json()?)),
            target_leaf: active.operation.relative_path.clone(),
            started_at_unix: now_unix,
        };
        let mut journal = TransactionJournalGuard::create(self.root.path(), journal_value)?;
        if let Err(error) = faults.check(FaultPoint::AfterJournal) {
            journal.finish()?;
            return Err(error);
        }

        if self.root.validate().is_err() {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        let reloaded =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        if reloaded != prior_state {
            journal.finish()?;
            return Err(TransactionError::ObservedStateMismatch);
        }
        let active = reloaded
            .active
            .as_ref()
            .ok_or(TransactionError::ReceiptMissing)?;
        verify_active_receipt(self.root.path(), &self.root.root_id, active)?;
        let target = target_path(self.root.path(), &active.operation.relative_path)?;
        let quarantine = quarantine_path(self.root.path(), install_id, &transaction_id);
        if let Err(error) =
            move_verified_target_to_quarantine(&target, &quarantine, active, &materialization)
        {
            if error == TransactionError::RecoveryRequired {
                journal.retain();
            }
            return Err(error);
        }

        let lifecycle = InstallLifecycleReceiptV1 {
            schema_version: INSTALL_RECEIPT_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            install_id: install_id.to_string(),
            prior_transaction_id: active.transaction_id.clone(),
            kind,
            completed_at_unix: now_unix,
        };
        let next_state = ManagedInstallStateV1 {
            schema_version: MANAGED_INSTALL_STATE_SCHEMA_VERSION,
            generation: next_generation(Some(&prior_state))?,
            install_id: install_id.to_string(),
            active: None,
            last_lifecycle: Some(lifecycle.clone()),
        };

        if self.root.validate().is_err() {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }

        let commit_result = faults
            .check(FaultPoint::BeforeStateCommit)
            .and_then(|()| persist_state(self.root.path(), &next_state, &transaction_id, faults));
        if let Err(error) = commit_result {
            if error == TransactionError::RecoveryRequired {
                journal.retain();
                return Err(error);
            }
            if faults.check(FaultPoint::DuringRollback).is_err()
                || restore_quarantine(&quarantine, &target, active, &materialization).is_err()
            {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
            journal.finish()?;
            return Err(error);
        }

        let cleanup_failed = faults.check(FaultPoint::DuringCleanup).is_err()
            || remove_verified_quarantine(&quarantine, active, &materialization).is_err();
        if cleanup_failed {
            journal.retain();
            return Ok(LifecycleCommit {
                disposition: lifecycle_disposition(kind),
                receipt: lifecycle,
                cleanup_required: true,
            });
        }
        let journal_cleanup = journal.finish().is_err();
        Ok(LifecycleCommit {
            disposition: lifecycle_disposition(kind),
            receipt: lifecycle,
            cleanup_required: journal_cleanup,
        })
    }
}

#[derive(Clone, Copy)]
enum ApplyKind {
    Fresh,
    Repair,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MaterializedTargetOwnership {
    CreatedByTransaction,
    Uncertain,
}

struct ExecutableMaterialization<'a> {
    plan: &'a VerifiedInstallPlan,
    operation: &'a InstallOperationV1,
    root_id: &'a str,
    entry_key: &'a str,
    relative_path: &'a str,
    content_root_sha256: &'a str,
    ownership: &'a OwnershipMarkerV1,
    pack_sha256: &'a str,
    destination: PathBuf,
    plan_digest: &'a str,
}

impl<'a> ExecutableMaterialization<'a> {
    fn from_plan(
        verified_plan: &'a VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &'a LoadedResourcePack<'_>,
        root: &ApprovedManagedRoot,
        now_unix: u64,
    ) -> Result<Self, TransactionError> {
        let plan = verified_plan.plan();
        if now_unix < plan.created_at_unix
            || now_unix >= plan.expires_at_unix
            || now_unix >= approval.expires_at_unix
        {
            return Err(TransactionError::PlanExpired);
        }
        if approval.semantic_digest_sha256 != plan.semantic_digest_sha256
            || approval.approvals != plan.approvals_required
        {
            return Err(TransactionError::InvalidApproval);
        }
        if plan.approvals_required != [ApprovalRequirement::FilesystemWrite]
            || plan.target.profile != CapabilityProfile::Lite
            || plan.target.scope != InstallScope::User
            || plan.allowed_roots.len() != 1
            || plan.operations.len() != 1
            || plan.outstanding_host_action.is_some()
        {
            return Err(TransactionError::UnsupportedPlan);
        }
        let allowed_root = &plan.allowed_roots[0];
        if allowed_root.root != SymbolicRoot::QiongliManagedData || allowed_root.id != root.root_id
        {
            return Err(TransactionError::UnsafeManagedRoot);
        }
        let operation = &plan.operations[0];
        let InstallActionV1::MaterializeResources {
            root_id,
            entry_key,
            relative_path,
            content_root_sha256,
            ownership,
        } = &operation.action
        else {
            return Err(TransactionError::UnsupportedPlan);
        };
        if root_id != &root.root_id
            || !valid_target_leaf(relative_path)
            || relative_path.starts_with(".qiongli")
            || operation.precondition != PlanStateV1::Missing
            || operation.observed_state_sha256
                != observed_plan_state_sha256(&PlanStateV1::Missing)
                    .map_err(|_| TransactionError::UnsupportedPlan)?
        {
            return Err(TransactionError::ObservedStateMismatch);
        }
        let PlanStateV1::Managed {
            ownership: post_ownership,
            content_sha256: post_sha256,
        } = &operation.postcondition
        else {
            return Err(TransactionError::UnsupportedPlan);
        };
        let InstallActionV1::RemoveManagedEntry {
            root_id: inverse_root,
            entry_key: inverse_key,
            expected_ownership,
            expected_sha256,
        } = &operation.inverse
        else {
            return Err(TransactionError::UnsupportedPlan);
        };
        if post_ownership != ownership
            || post_sha256 != content_root_sha256
            || inverse_root != root_id
            || inverse_key != entry_key
            || expected_ownership != ownership
            || expected_sha256 != content_root_sha256
        {
            return Err(TransactionError::UnsupportedPlan);
        }
        if pack.manifest().content_root_sha256 != *content_root_sha256
            || verified_plan.grant().grant().resource_pack_sha256 != pack.pack_sha256()
            || pack.resources_for_profile("lite").is_err()
        {
            return Err(TransactionError::PayloadMismatch);
        }
        let destination = target_path(root.path(), relative_path)?;
        Ok(Self {
            plan: verified_plan,
            operation,
            root_id,
            entry_key,
            relative_path,
            content_root_sha256,
            ownership,
            pack_sha256: pack.pack_sha256(),
            destination,
            plan_digest: &plan.semantic_digest_sha256,
        })
    }

    fn install_id(&self) -> &str {
        &self.ownership.install_id
    }

    fn validate_materialization(
        &self,
        receipt: &MaterializationReceiptV1,
    ) -> Result<(), TransactionError> {
        if receipt.profile != ProfileId::MarketplaceLite
            || receipt.pack_sha256 != self.pack_sha256
            || receipt.content_root_sha256 != self.content_root_sha256
        {
            return Err(TransactionError::PayloadMismatch);
        }
        Ok(())
    }

    fn build_receipt(
        &self,
        transaction_id: String,
        applied_at_unix: u64,
        materialization_receipt_sha256: String,
        replaces_transaction_id: Option<String>,
    ) -> InstallReceiptV1 {
        InstallReceiptV1 {
            schema_version: INSTALL_RECEIPT_SCHEMA_VERSION,
            transaction_id,
            plan_id: self.plan.plan().plan_id.clone(),
            semantic_digest_sha256: self.plan_digest.to_string(),
            install_id: self.install_id().to_string(),
            artifact: self.plan.plan().artifact.clone(),
            target: self.plan.plan().target.clone(),
            operation: ManagedOperationReceiptV1 {
                operation_id: self.operation.operation_id.clone(),
                root_id: self.root_id.to_string(),
                entry_key: self.entry_key.to_string(),
                relative_path: self.relative_path.to_string(),
                ownership: self.ownership.clone(),
                pack_sha256: self.pack_sha256.to_string(),
                content_root_sha256: self.content_root_sha256.to_string(),
                materialization_receipt_sha256,
            },
            applied_at_unix,
            replaces_transaction_id,
            outstanding_host_action: None,
        }
    }

    fn matches_active(&self, active: &InstallReceiptV1) -> bool {
        active.install_id == self.install_id()
            && active.semantic_digest_sha256 == self.plan_digest
            && active.operation.root_id == self.root_id
            && active.operation.relative_path == self.relative_path
            && active.operation.ownership == *self.ownership
            && active.operation.pack_sha256 == self.pack_sha256
            && active.operation.content_root_sha256 == self.content_root_sha256
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum JournalKind {
    Apply,
    Repair,
    Remove,
    Rollback,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct TransactionJournalV1 {
    schema_version: u32,
    transaction_id: String,
    kind: JournalKind,
    install_id: String,
    plan_digest_sha256: Option<String>,
    prior_state_sha256: Option<String>,
    target_leaf: String,
    started_at_unix: u64,
}

impl TransactionJournalV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != INSTALL_JOURNAL_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.install_id)
            || !valid_target_leaf(&self.target_leaf)
            || self.started_at_unix > JCS_MAX_SAFE_INTEGER
            || self
                .plan_digest_sha256
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
            || self
                .prior_state_sha256
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
        {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(())
    }
}

struct TransactionJournalGuard {
    root: PathBuf,
    path: PathBuf,
    identity: Option<Handle>,
    armed: bool,
}

impl TransactionJournalGuard {
    fn create(root: &Path, journal: TransactionJournalV1) -> Result<Self, TransactionError> {
        journal.validate()?;
        let path = journal_path(root, &journal.install_id);
        if path_exists(&path)? {
            return Err(TransactionError::RecoveryRequired);
        }
        let bytes = canonical_json(&journal)?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(TransactionError::InvalidReceipt);
        }
        let mut file = create_private_new_file(&path)?;
        if let Err(error) = write_sync_file(&mut file, &bytes) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        drop(file);
        if sync_directory(root).is_err() {
            return Err(TransactionError::RecoveryRequired);
        }
        let identity = Handle::from_path(&path).map_err(|_| TransactionError::RecoveryRequired)?;
        Ok(Self {
            root: root.to_path_buf(),
            path,
            identity: Some(identity),
            armed: true,
        })
    }

    fn finish(&mut self) -> Result<(), TransactionError> {
        if !self.armed {
            return Ok(());
        }
        let expected = self
            .identity
            .as_ref()
            .ok_or(TransactionError::RecoveryRequired)?;
        let current =
            Handle::from_path(&self.path).map_err(|_| TransactionError::RecoveryRequired)?;
        if &current != expected {
            self.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        fs::remove_file(&self.path)
            .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
        sync_directory(&self.root)?;
        self.identity.take();
        self.armed = false;
        Ok(())
    }

    fn retain(&mut self) {
        self.identity.take();
        self.armed = false;
    }
}

impl Drop for TransactionJournalGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let still_owned = self.identity.as_ref().is_some_and(|expected| {
            Handle::from_path(&self.path).is_ok_and(|current| &current == expected)
        });
        self.identity.take();
        if still_owned {
            let _ = fs::remove_file(&self.path);
            let _ = sync_directory(&self.root);
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum FaultPoint {
    AfterJournal,
    AmbiguousMaterializationResult,
    AfterMaterialization,
    BeforeStateCommit,
    DuringStateCommit,
    AfterStateCommit,
    DuringRollback,
    DuringCleanup,
}

trait TransactionFaults {
    fn check(&self, point: FaultPoint) -> Result<(), TransactionError>;
}

struct NoFaults;

impl TransactionFaults for NoFaults {
    fn check(&self, _point: FaultPoint) -> Result<(), TransactionError> {
        Ok(())
    }
}

fn verify_active_receipt(
    root: &Path,
    expected_root_id: &str,
    receipt: &InstallReceiptV1,
) -> Result<MaterializationReceiptV1, TransactionError> {
    receipt.validate()?;
    if receipt.operation.root_id != expected_root_id {
        return Err(TransactionError::ManagedStateDrift);
    }
    let target_path = target_path(root, &receipt.operation.relative_path)?;
    let target = approved_materialization_target(&target_path)?;
    let materialization =
        verify_materialization(&target).map_err(|_| TransactionError::ManagedStateDrift)?;
    let receipt_sha256 = sha256_hex(&canonical_json(&materialization)?);
    if materialization.profile != ProfileId::MarketplaceLite
        || materialization.pack_sha256 != receipt.operation.pack_sha256
        || materialization.content_root_sha256 != receipt.operation.content_root_sha256
        || receipt_sha256 != receipt.operation.materialization_receipt_sha256
    {
        return Err(TransactionError::ManagedStateDrift);
    }
    Ok(materialization)
}

fn move_verified_target_to_quarantine(
    target: &Path,
    quarantine: &Path,
    receipt: &InstallReceiptV1,
    expected_materialization: &MaterializationReceiptV1,
) -> Result<(), TransactionError> {
    ensure_destination_absent(quarantine)?;
    let before = Handle::from_path(target).map_err(|_| TransactionError::ManagedStateDrift)?;
    let approved = approved_materialization_target(target)?;
    let observed =
        verify_materialization(&approved).map_err(|_| TransactionError::ManagedStateDrift)?;
    if &observed != expected_materialization
        || sha256_hex(&canonical_json(&observed)?)
            != receipt.operation.materialization_receipt_sha256
    {
        return Err(TransactionError::ManagedStateDrift);
    }
    let after = Handle::from_path(target).map_err(|_| TransactionError::ManagedStateDrift)?;
    if before != after {
        return Err(TransactionError::ManagedStateDrift);
    }
    rename_path(target, quarantine, false)?;
    if sync_directory(target.parent().ok_or(TransactionError::UnsafeManagedRoot)?).is_err() {
        return Err(TransactionError::RecoveryRequired);
    }
    let quarantined = approved_materialization_target(quarantine)
        .map_err(|_| TransactionError::RecoveryRequired)?;
    let observed =
        verify_materialization(&quarantined).map_err(|_| TransactionError::RecoveryRequired)?;
    if &observed != expected_materialization {
        return Err(TransactionError::RecoveryRequired);
    }
    Ok(())
}

fn restore_quarantine(
    quarantine: &Path,
    target: &Path,
    receipt: &InstallReceiptV1,
    expected_materialization: &MaterializationReceiptV1,
) -> Result<(), TransactionError> {
    ensure_destination_absent(target)?;
    let before = Handle::from_path(quarantine).map_err(|_| TransactionError::RollbackConflict)?;
    let approved = approved_materialization_target(quarantine)?;
    let observed =
        verify_materialization(&approved).map_err(|_| TransactionError::RollbackConflict)?;
    if &observed != expected_materialization
        || sha256_hex(&canonical_json(&observed)?)
            != receipt.operation.materialization_receipt_sha256
    {
        return Err(TransactionError::RollbackConflict);
    }
    let after = Handle::from_path(quarantine).map_err(|_| TransactionError::RollbackConflict)?;
    if before != after {
        return Err(TransactionError::RollbackConflict);
    }
    rename_path(quarantine, target, false)?;
    sync_directory(target.parent().ok_or(TransactionError::UnsafeManagedRoot)?)?;
    verify_active_receipt(
        target.parent().ok_or(TransactionError::UnsafeManagedRoot)?,
        &receipt.operation.root_id,
        receipt,
    )?;
    Ok(())
}

fn rollback_created_target(
    root: &Path,
    receipt: &InstallReceiptV1,
    expected_materialization: &MaterializationReceiptV1,
    transaction_id: &str,
) -> Result<(), TransactionError> {
    let target = target_path(root, &receipt.operation.relative_path)?;
    let quarantine = quarantine_path(root, &receipt.install_id, transaction_id);
    move_verified_target_to_quarantine(&target, &quarantine, receipt, expected_materialization)?;
    remove_verified_quarantine(&quarantine, receipt, expected_materialization)
}

fn remove_verified_quarantine(
    quarantine: &Path,
    receipt: &InstallReceiptV1,
    expected_materialization: &MaterializationReceiptV1,
) -> Result<(), TransactionError> {
    let before = Handle::from_path(quarantine).map_err(|_| TransactionError::RollbackConflict)?;
    let approved = approved_materialization_target(quarantine)?;
    let observed =
        verify_materialization(&approved).map_err(|_| TransactionError::RollbackConflict)?;
    if &observed != expected_materialization
        || sha256_hex(&canonical_json(&observed)?)
            != receipt.operation.materialization_receipt_sha256
    {
        return Err(TransactionError::RollbackConflict);
    }
    let after = Handle::from_path(quarantine).map_err(|_| TransactionError::RollbackConflict)?;
    if before != after {
        return Err(TransactionError::RollbackConflict);
    }
    fs::remove_dir_all(quarantine)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    sync_directory(
        quarantine
            .parent()
            .ok_or(TransactionError::UnsafeManagedRoot)?,
    )
}

fn persist_state<F: TransactionFaults>(
    root: &Path,
    state: &ManagedInstallStateV1,
    transaction_id: &str,
    faults: &F,
) -> Result<(), TransactionError> {
    let bytes = state.to_canonical_json()?;
    let destination = state_path(root, &state.install_id);
    if path_exists(&destination)? {
        let _ = read_private_file(&destination)?;
    }
    let staging = root.join(format!(
        "{STATE_FILE_PREFIX}{}.stage-{transaction_id}",
        state.install_id
    ));
    ensure_destination_absent(&staging)?;
    let mut file = create_private_new_file(&staging)?;
    if let Err(error) = write_sync_file(&mut file, &bytes) {
        drop(file);
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    drop(file);
    if let Err(error) = faults.check(FaultPoint::DuringStateCommit) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    if rename_path(&staging, &destination, path_exists(&destination)?).is_err() {
        let _ = fs::remove_file(&staging);
        return Err(TransactionError::RecoveryRequired);
    }
    if faults.check(FaultPoint::AfterStateCommit).is_err() {
        return Err(TransactionError::RecoveryRequired);
    }
    if sync_directory(root).is_err() {
        return Err(TransactionError::RecoveryRequired);
    }
    let committed =
        read_private_file(&destination).map_err(|_| TransactionError::RecoveryRequired)?;
    if committed != bytes {
        return Err(TransactionError::RecoveryRequired);
    }
    Ok(())
}

fn load_state(
    root: &Path,
    install_id: &str,
) -> Result<Option<ManagedInstallStateV1>, TransactionError> {
    if !valid_identifier(install_id) {
        return Err(TransactionError::InvalidReceipt);
    }
    let path = state_path(root, install_id);
    if !path_exists(&path)? {
        return Ok(None);
    }
    let bytes = read_private_file(&path)?;
    ManagedInstallStateV1::from_json(&bytes).map(Some)
}

fn state_digest(root: &Path, install_id: &str) -> Result<Option<String>, TransactionError> {
    let path = state_path(root, install_id);
    if !path_exists(&path)? {
        return Ok(None);
    }
    read_private_file(&path).map(|bytes| Some(sha256_hex(&bytes)))
}

fn next_generation(state: Option<&ManagedInstallStateV1>) -> Result<u64, TransactionError> {
    state
        .map_or(Some(1), |state| state.generation.checked_add(1))
        .filter(|generation| *generation <= JCS_MAX_SAFE_INTEGER)
        .ok_or(TransactionError::InvalidReceipt)
}

fn ensure_no_journal(root: &Path, install_id: &str) -> Result<(), TransactionError> {
    if !valid_identifier(install_id) {
        return Err(TransactionError::InvalidReceipt);
    }
    if path_exists(&journal_path(root, install_id))? {
        Err(TransactionError::RecoveryRequired)
    } else {
        Ok(())
    }
}

fn state_path(root: &Path, install_id: &str) -> PathBuf {
    root.join(format!(
        "{STATE_FILE_PREFIX}{install_id}{STATE_FILE_SUFFIX}"
    ))
}

fn journal_path(root: &Path, _install_id: &str) -> PathBuf {
    root.join(JOURNAL_FILE_NAME)
}

fn quarantine_path(root: &Path, install_id: &str, transaction_id: &str) -> PathBuf {
    root.join(format!(".{install_id}{QUARANTINE_MARKER}{transaction_id}"))
}

fn target_path(root: &Path, relative_path: &str) -> Result<PathBuf, TransactionError> {
    if !valid_target_leaf(relative_path) || relative_path.starts_with(".qiongli") {
        return Err(TransactionError::UnsupportedPlan);
    }
    Ok(root.join(relative_path))
}

fn approved_materialization_target(path: &Path) -> Result<MaterializationTarget, TransactionError> {
    approve_materialization_target(path).map_err(|_| TransactionError::UnsafeManagedRoot)
}

fn ensure_destination_absent(path: &Path) -> Result<(), TransactionError> {
    if path_exists(path)? {
        Err(TransactionError::DestinationConflict)
    } else {
        Ok(())
    }
}

fn path_exists(path: &Path) -> Result<bool, TransactionError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(TransactionError::PersistenceFailed(error.kind())),
    }
}

fn validate_managed_root(path: &Path) -> Result<(), TransactionError> {
    let _ =
        approve_materialization_target(path).map_err(|_| TransactionError::UnsafeManagedRoot)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| TransactionError::UnsafeManagedRoot)?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(TransactionError::UnsafeManagedRoot);
    }
    validate_private_root(path, &metadata)
}

#[cfg(unix)]
fn validate_private_root(_path: &Path, metadata: &Metadata) -> Result<(), TransactionError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    if metadata.uid() == rustix::process::geteuid().as_raw()
        && metadata.permissions().mode() & 0o077 == 0
    {
        Ok(())
    } else {
        Err(TransactionError::UnsafeManagedRoot)
    }
}

#[cfg(windows)]
fn validate_private_root(path: &Path, _metadata: &Metadata) -> Result<(), TransactionError> {
    qiongli_windows_security::open_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| TransactionError::UnsafeManagedRoot)
}

#[cfg(not(any(unix, windows)))]
fn validate_private_root(_path: &Path, _metadata: &Metadata) -> Result<(), TransactionError> {
    Err(TransactionError::UnsupportedPlatform)
}

fn read_private_file(path: &Path) -> Result<Vec<u8>, TransactionError> {
    let file = open_private_file(path)?;
    let metadata = file
        .metadata()
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    if metadata.len() > MAX_STATE_BYTES {
        return Err(TransactionError::InvalidReceipt);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_STATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    if bytes.len() as u64 > MAX_STATE_BYTES {
        return Err(TransactionError::InvalidReceipt);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn open_private_file(path: &Path) -> Result<File, TransactionError> {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;

    let linked = fs::symlink_metadata(path)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    if linked.file_type().is_symlink()
        || !linked.is_file()
        || linked.nlink() != 1
        || linked.uid() != rustix::process::geteuid().as_raw()
        || linked.permissions().mode() & 0o777 != 0o600
    {
        return Err(TransactionError::InvalidReceipt);
    }
    let file =
        File::open(path).map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    let opened = file
        .metadata()
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    if opened.dev() != linked.dev() || opened.ino() != linked.ino() || opened.nlink() != 1 {
        return Err(TransactionError::InvalidReceipt);
    }
    Ok(file)
}

#[cfg(windows)]
fn open_private_file(path: &Path) -> Result<File, TransactionError> {
    qiongli_windows_security::open_owner_only_file(path)
        .map_err(|_| TransactionError::InvalidReceipt)
}

#[cfg(not(any(unix, windows)))]
fn open_private_file(_path: &Path) -> Result<File, TransactionError> {
    Err(TransactionError::UnsupportedPlatform)
}

#[cfg(unix)]
fn create_private_new_file(path: &Path) -> Result<File, TransactionError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                TransactionError::RecoveryRequired
            } else {
                TransactionError::PersistenceFailed(error.kind())
            }
        })
}

#[cfg(windows)]
fn create_private_new_file(path: &Path) -> Result<File, TransactionError> {
    qiongli_windows_security::create_owner_only_new_file(path).map_err(|error| {
        if error.io_kind() == Some(io::ErrorKind::AlreadyExists) {
            TransactionError::RecoveryRequired
        } else {
            TransactionError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        }
    })
}

#[cfg(not(any(unix, windows)))]
fn create_private_new_file(_path: &Path) -> Result<File, TransactionError> {
    Err(TransactionError::UnsupportedPlatform)
}

fn write_sync_file(file: &mut File, bytes: &[u8]) -> Result<(), TransactionError> {
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn rename_path(
    source: &Path,
    destination: &Path,
    replace_existing: bool,
) -> Result<(), TransactionError> {
    qiongli_windows_security::move_file_write_through(source, destination, replace_existing)
        .map_err(|error| {
            TransactionError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn rename_path(
    source: &Path,
    destination: &Path,
    replace_existing: bool,
) -> Result<(), TransactionError> {
    if replace_existing {
        return fs::rename(source, destination)
            .map_err(|error| TransactionError::PersistenceFailed(error.kind()));
    }
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        source,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|error| {
        let error = io::Error::from(error);
        TransactionError::PersistenceFailed(error.kind())
    })
}

#[cfg(all(not(windows), not(any(target_os = "linux", target_os = "macos"))))]
fn rename_path(
    source: &Path,
    destination: &Path,
    replace_existing: bool,
) -> Result<(), TransactionError> {
    if !replace_existing {
        return Err(TransactionError::UnsupportedPlatform);
    }
    fs::rename(source, destination)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), TransactionError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), TransactionError> {
    Ok(())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, TransactionError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| TransactionError::InvalidReceipt)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_entry_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn valid_target_leaf(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && !matches!(value, "." | "..")
        && !value.ends_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && !is_windows_device_name(value)
}

fn is_windows_device_name(value: &str) -> bool {
    let stem = value
        .split_once('.')
        .map_or(value, |(stem, _)| stem)
        .to_ascii_lowercase();
    matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || (stem.len() == 4
            && matches!(&stem[..3], "com" | "lpt")
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(unix)]
fn has_lexical_traversal(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str()
        .as_bytes()
        .split(|byte| *byte == b'/')
        .any(|component| component == b"." || component == b"..")
}

#[cfg(windows)]
fn has_lexical_traversal(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .collect::<Vec<_>>()
        .split(|unit| matches!(*unit, 47 | 92))
        .any(|component| component == [46] || component == [46, 46])
}

#[cfg(not(any(unix, windows)))]
fn has_lexical_traversal(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn transaction_id() -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
    format!("txn-{nonce}-{sequence}")
}

fn lifecycle_disposition(kind: InstallLifecycleKind) -> LifecycleDisposition {
    match kind {
        InstallLifecycleKind::Removed => LifecycleDisposition::Removed,
        InstallLifecycleKind::RolledBack => LifecycleDisposition::RolledBack,
    }
}

fn idempotent_lifecycle_disposition(kind: InstallLifecycleKind) -> LifecycleDisposition {
    match kind {
        InstallLifecycleKind::Removed => LifecycleDisposition::AlreadyRemoved,
        InstallLifecycleKind::RolledBack => LifecycleDisposition::AlreadyRolledBack,
    }
}

fn finish_committed_journal<F: TransactionFaults>(
    journal: &mut TransactionJournalGuard,
    faults: &F,
) -> bool {
    let cleanup_failed =
        faults.check(FaultPoint::DuringCleanup).is_err() || journal.finish().is_err();
    if cleanup_failed {
        journal.retain();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use ed25519_dalek::{Signer, SigningKey};
    use qiongli_content::{
        BuiltResourcePack, CompatibleProduct, ResourcePackBuildMetadata, build_resource_pack,
        collect_canonical_sources, load_resource_pack,
    };

    use super::*;
    use crate::{
        Architecture, GrantMode, GrantSignatureV1, GrantVerificationContext, InstallPlanDraftV1,
        InstallPlanMetadataV1, InstallPlanV1, InstallerKind, IntegrationScope, LaunchGrantV1,
        LocalSurface, LocalTargetFamily, OperatingSystem, ReleaseChannel, SignatureAlgorithm,
        SignedLaunchGrantV1, TrustedPublicKey, launch_grant_signing_bytes,
    };

    const BINARY_DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const NOW: u64 = 1_750_000_000;
    const CANONICAL_DIRECTORIES: [&str; 10] = [
        "distribution",
        "mcp-contracts",
        "roles",
        "schemas",
        "skills",
        "standards",
        "subjects",
        "templates",
        "venue-profiles",
        "workflow",
    ];
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
    static NEXT_KEY: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        source: PathBuf,
        container: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-platform-transaction-tests");
            fs::create_dir_all(&base).expect("transaction-test base must exist");
            let requested_container = base.join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&requested_container).expect("transaction-test container must exist");
            let container = fs::canonicalize(requested_container)
                .expect("transaction-test container must canonicalize");
            let source = container.join("source");
            fs::create_dir(&source).expect("pack source must exist");
            for directory in CANONICAL_DIRECTORIES {
                let directory = source.join(directory);
                fs::create_dir(&directory).expect("canonical source directory must exist");
                let directory_name = directory
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("canonical directory name must be UTF-8");
                fs::write(
                    directory.join("entry.md"),
                    format!("{name}:{directory_name}\n"),
                )
                .expect("canonical source file must write");
            }
            fs::write(source.join("skills-core.md"), b"core\n").expect("skills-core must write");
            fs::write(source.join("skills-summary.md"), b"summary\n")
                .expect("skills-summary must write");

            let root = container.join("managed");
            create_private_test_root(&root);
            Self {
                root,
                source,
                container,
            }
        }

        fn pack(&self) -> BuiltResourcePack {
            let resources = collect_canonical_sources(&self.source)
                .expect("canonical transaction-test sources must collect");
            build_resource_pack(
                &ResourcePackBuildMetadata {
                    pack_id: "qiongli-core".to_string(),
                    content_version: "1.19.0-beta.1".to_string(),
                    source_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    compatible_product: CompatibleProduct {
                        minimum: "2.0.0-alpha.1".to_string(),
                        maximum_exclusive: "3.0.0".to_string(),
                    },
                },
                &resources,
            )
            .expect("transaction-test pack must build")
        }

        fn approved_root(&self) -> ApprovedManagedRoot {
            approve_managed_root(
                &AllowedRootV1 {
                    id: "qiongli-data".to_string(),
                    root: SymbolicRoot::QiongliManagedData,
                },
                &self.root,
            )
            .expect("private test root must approve")
        }

        fn target(&self) -> PathBuf {
            self.root.join("marketplace-lite")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.container);
        }
    }

    struct PlanFixture {
        verified: VerifiedInstallPlan,
    }

    fn verified_plan(pack: &BuiltResourcePack) -> PlanFixture {
        let artifact = ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.1".to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::current().expect("test target OS must be supported"),
            arch: Architecture::current().expect("test target architecture must be supported"),
            installer_kind: InstallerKind::PortableArchive,
        };
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: 7,
            artifact: artifact.clone(),
            binary_sha256: BINARY_DIGEST.to_string(),
            resource_pack_sha256: pack.pack_sha256().to_string(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![IntegrationScope::CodexLocal],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        };
        let signing_key = temporary_test_signing_key();
        let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
        let signed = SignedLaunchGrantV1 {
            grant,
            signature: GrantSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "transaction-test-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        };
        let trusted = TrustedPublicKey::new(
            "transaction-test-key",
            signing_key.verifying_key().to_bytes(),
        )
        .unwrap();
        let context = GrantVerificationContext {
            now_unix: NOW,
            minimum_generation: 7,
            expected_artifact: &artifact,
            binary_sha256: BINARY_DIGEST,
            resource_pack_sha256: pack.pack_sha256(),
            requested_mode: GrantMode::LiteMcp,
            requested_scope: IntegrationScope::CodexLocal,
        };
        let verified_grant = signed
            .verify(std::slice::from_ref(&trusted), &context)
            .expect("transaction-test grant must verify");
        let ownership = OwnershipMarkerV1 {
            schema_version: 1,
            product: ProductId::Qiongli,
            install_id: "qiongli-lite-user".to_string(),
            artifact_digest_sha256: verified_grant.signed_payload_sha256().to_string(),
        };
        let content_root = pack.manifest().content_root_sha256.clone();
        let plan = InstallPlanV1::build(
            InstallPlanMetadataV1 {
                plan_id: "r3b-test-plan".to_string(),
                created_at_unix: NOW,
                expires_at_unix: NOW + 600,
            },
            &verified_grant,
            InstallPlanDraftV1 {
                target: TargetDescriptorV1 {
                    family: LocalTargetFamily::CodexLocal,
                    surface: LocalSurface::CliLocal,
                    scope: InstallScope::User,
                    profile: CapabilityProfile::Lite,
                    os: artifact.os,
                    arch: artifact.arch,
                    adapter_version: 1,
                },
                allowed_roots: vec![AllowedRootV1 {
                    id: "qiongli-data".to_string(),
                    root: SymbolicRoot::QiongliManagedData,
                }],
                operations: vec![InstallOperationV1 {
                    operation_id: "materialize-lite".to_string(),
                    action: InstallActionV1::MaterializeResources {
                        root_id: "qiongli-data".to_string(),
                        entry_key: "marketplace-lite".to_string(),
                        relative_path: "marketplace-lite".to_string(),
                        content_root_sha256: content_root.clone(),
                        ownership: ownership.clone(),
                    },
                    precondition: PlanStateV1::Missing,
                    observed_state_sha256: observed_plan_state_sha256(&PlanStateV1::Missing)
                        .unwrap(),
                    postcondition: PlanStateV1::Managed {
                        ownership: ownership.clone(),
                        content_sha256: content_root.clone(),
                    },
                    inverse: InstallActionV1::RemoveManagedEntry {
                        root_id: "qiongli-data".to_string(),
                        entry_key: "marketplace-lite".to_string(),
                        expected_ownership: ownership,
                        expected_sha256: content_root,
                    },
                }],
                approvals_required: vec![ApprovalRequirement::FilesystemWrite],
                outstanding_host_action: None,
            },
        )
        .expect("transaction-test plan must build");
        let verified = plan
            .verify(std::slice::from_ref(&trusted), &context)
            .expect("transaction-test plan must verify");
        PlanFixture { verified }
    }

    fn execute_fixture(
        fixture: &Fixture,
        pack: &BuiltResourcePack,
    ) -> (ManagedResourceExecutor, PlanFixture, ApprovedInstallPlan) {
        let plan = verified_plan(pack);
        let approval =
            approve_install_plan(&plan.verified, &[ApprovalRequirement::FilesystemWrite], NOW)
                .expect("transaction-test plan must approve");
        (
            ManagedResourceExecutor::new(fixture.approved_root()),
            plan,
            approval,
        )
    }

    #[test]
    fn applies_verifies_replays_and_removes_idempotently() {
        let fixture = Fixture::new("apply-remove");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);

        let applied = executor
            .apply(&plan.verified, &approval, &loaded, NOW + 1)
            .expect("fresh transaction must apply");
        assert_eq!(applied.disposition, InstallDisposition::Applied);
        assert!(!applied.cleanup_required);
        assert!(fixture.target().is_dir());
        assert!(executor.verify("qiongli-lite-user").is_ok());
        let state_bytes = fs::read(state_path(&fixture.root, "qiongli-lite-user")).unwrap();
        assert!(
            !String::from_utf8_lossy(&state_bytes)
                .contains(fixture.root.to_string_lossy().as_ref())
        );

        let replay = executor
            .apply(&plan.verified, &approval, &loaded, NOW + 2)
            .expect("identical transaction replay must verify");
        assert_eq!(replay.disposition, InstallDisposition::AlreadyApplied);
        assert_eq!(replay.receipt, applied.receipt);

        assert_eq!(
            executor.remove("qiongli-lite-user", NOW),
            Err(TransactionError::ObservedStateMismatch)
        );
        assert!(fixture.target().is_dir());
        assert!(!journal_path(&fixture.root, "qiongli-lite-user").exists());

        let removed = executor
            .remove("qiongli-lite-user", NOW + 3)
            .expect("managed target must remove");
        assert_eq!(removed.disposition, LifecycleDisposition::Removed);
        assert!(!removed.cleanup_required);
        assert!(!fixture.target().exists());

        let replay = executor
            .remove("qiongli-lite-user", NOW + 4)
            .expect("matching remove must be idempotent");
        assert_eq!(replay.disposition, LifecycleDisposition::AlreadyRemoved);
        assert_eq!(replay.receipt, removed.receipt);
    }

    #[test]
    fn rollback_has_a_distinct_idempotent_lifecycle() {
        let fixture = Fixture::new("rollback");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        executor
            .apply(&plan.verified, &approval, &loaded, NOW + 1)
            .unwrap();

        let rolled_back = executor.rollback("qiongli-lite-user", NOW + 2).unwrap();
        assert_eq!(rolled_back.disposition, LifecycleDisposition::RolledBack);
        assert_eq!(rolled_back.receipt.kind, InstallLifecycleKind::RolledBack);
        assert!(!fixture.target().exists());
        assert_eq!(
            executor
                .rollback("qiongli-lite-user", NOW + 3)
                .unwrap()
                .disposition,
            LifecycleDisposition::AlreadyRolledBack
        );
    }

    #[test]
    fn repairs_only_an_absent_target_and_refuses_present_drift() {
        let fixture = Fixture::new("repair");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        executor
            .apply(&plan.verified, &approval, &loaded, NOW + 1)
            .unwrap();
        fs::remove_dir_all(fixture.target()).expect("test must remove managed target");

        let repaired = executor
            .repair(&plan.verified, &approval, &loaded, NOW + 2)
            .expect("missing managed target must repair");
        assert_eq!(repaired.disposition, InstallDisposition::Repaired);
        assert!(repaired.receipt.replaces_transaction_id.is_some());
        assert!(executor.verify("qiongli-lite-user").is_ok());

        let drifted = fixture.target().join("skills/entry.md");
        fs::write(&drifted, b"user edit\n").expect("drift fixture must write");
        assert_eq!(
            executor.repair(&plan.verified, &approval, &loaded, NOW + 3),
            Err(TransactionError::ManagedStateDrift)
        );
        assert_eq!(fs::read(&drifted).unwrap(), b"user edit\n");
    }

    #[test]
    fn approval_payload_and_receipt_parsing_fail_closed() {
        let fixture = Fixture::new("approval-receipt");
        let pack = fixture.pack();
        let plan = verified_plan(&pack);
        assert!(matches!(
            approve_install_plan(&plan.verified, &[], NOW),
            Err(TransactionError::InvalidApproval)
        ));
        assert!(matches!(
            approve_install_plan(
                &plan.verified,
                &[ApprovalRequirement::FilesystemWrite],
                NOW + 600,
            ),
            Err(TransactionError::PlanExpired)
        ));

        let state = ManagedInstallStateV1 {
            schema_version: MANAGED_INSTALL_STATE_SCHEMA_VERSION,
            generation: 1,
            install_id: "qiongli-lite-user".to_string(),
            active: None,
            last_lifecycle: Some(InstallLifecycleReceiptV1 {
                schema_version: INSTALL_RECEIPT_SCHEMA_VERSION,
                transaction_id: "txn-test".to_string(),
                install_id: "qiongli-lite-user".to_string(),
                prior_transaction_id: "txn-prior".to_string(),
                kind: InstallLifecycleKind::Removed,
                completed_at_unix: NOW,
            }),
        };
        let canonical = state.to_canonical_json().unwrap();
        assert_eq!(ManagedInstallStateV1::from_json(&canonical).unwrap(), state);
        let mut noncanonical = canonical.clone();
        noncanonical.push(b'\n');
        assert_eq!(
            ManagedInstallStateV1::from_json(&noncanonical),
            Err(TransactionError::InvalidReceipt)
        );
        let mut value = serde_json::to_value(&state).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_string(), serde_json::json!(true));
        assert_eq!(
            ManagedInstallStateV1::from_json(&serde_json::to_vec(&value).unwrap()),
            Err(TransactionError::InvalidReceipt)
        );
    }

    #[test]
    fn pre_commit_failures_restore_absence() {
        for (index, point) in [
            FaultPoint::AfterJournal,
            FaultPoint::AfterMaterialization,
            FaultPoint::BeforeStateCommit,
            FaultPoint::DuringStateCommit,
        ]
        .into_iter()
        .enumerate()
        {
            let fixture = Fixture::new(&format!("apply-fault-{index}"));
            let pack = fixture.pack();
            let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
            let (executor, plan, approval) = execute_fixture(&fixture, &pack);
            let faults = InjectedFaults::new(&[point]);

            assert_eq!(
                executor.apply_with(
                    &plan.verified,
                    &approval,
                    &loaded,
                    NOW + 1,
                    ApplyKind::Fresh,
                    &faults,
                ),
                Err(TransactionError::PersistenceFailed(io::ErrorKind::Other))
            );
            assert!(!fixture.target().exists());
            assert!(!state_path(&fixture.root, "qiongli-lite-user").exists());
            assert!(!journal_path(&fixture.root, "qiongli-lite-user").exists());
        }
    }

    #[test]
    fn ambiguous_apply_state_commit_retains_state_target_and_journal() {
        let fixture = Fixture::new("apply-state-ambiguity");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        let faults = InjectedFaults::new(&[FaultPoint::AfterStateCommit]);

        assert_eq!(
            executor.apply_with(
                &plan.verified,
                &approval,
                &loaded,
                NOW + 1,
                ApplyKind::Fresh,
                &faults,
            ),
            Err(TransactionError::RecoveryRequired)
        );
        assert!(fixture.target().is_dir());
        assert!(
            load_state(&fixture.root, "qiongli-lite-user")
                .unwrap()
                .unwrap()
                .active
                .is_some()
        );
        assert!(journal_path(&fixture.root, "qiongli-lite-user").is_file());
        assert_eq!(
            executor.verify("qiongli-lite-user"),
            Err(TransactionError::RecoveryRequired)
        );
    }

    #[test]
    fn payload_and_unmanaged_destination_conflicts_perform_no_managed_write() {
        let fixture = Fixture::new("preflight-conflicts");
        let expected_pack = fixture.pack();
        fs::write(
            fixture.source.join("skills/entry.md"),
            b"different payload\n",
        )
        .unwrap();
        let other_pack = fixture.pack();
        let other_loaded =
            load_resource_pack(other_pack.core_bytes(), other_pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &expected_pack);

        assert_eq!(
            executor.apply(&plan.verified, &approval, &other_loaded, NOW + 1),
            Err(TransactionError::PayloadMismatch)
        );
        assert!(!fixture.target().exists());
        assert!(!state_path(&fixture.root, "qiongli-lite-user").exists());
        assert!(!journal_path(&fixture.root, "qiongli-lite-user").exists());

        let expected_loaded =
            load_resource_pack(expected_pack.core_bytes(), expected_pack.pack_sha256()).unwrap();
        fs::create_dir(fixture.target()).unwrap();
        let sentinel = fixture.target().join("unmanaged.txt");
        fs::write(&sentinel, b"keep me\n").unwrap();
        assert_eq!(
            executor.apply(&plan.verified, &approval, &expected_loaded, NOW + 2),
            Err(TransactionError::DestinationConflict)
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"keep me\n");
        assert!(!state_path(&fixture.root, "qiongli-lite-user").exists());
        assert!(!journal_path(&fixture.root, "qiongli-lite-user").exists());
    }

    #[test]
    fn ambiguous_apply_rollback_retains_recovery_evidence() {
        let fixture = Fixture::new("apply-recovery");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        let faults =
            InjectedFaults::new(&[FaultPoint::AfterMaterialization, FaultPoint::DuringRollback]);

        assert_eq!(
            executor.apply_with(
                &plan.verified,
                &approval,
                &loaded,
                NOW + 1,
                ApplyKind::Fresh,
                &faults,
            ),
            Err(TransactionError::RecoveryRequired)
        );
        assert!(fixture.target().is_dir());
        assert!(journal_path(&fixture.root, "qiongli-lite-user").is_file());
        assert_eq!(
            executor.verify("qiongli-lite-user"),
            Err(TransactionError::RecoveryRequired)
        );
    }

    #[test]
    fn ambiguous_materializer_result_preserves_target_and_journal() {
        let fixture = Fixture::new("materializer-ambiguity");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        let faults = InjectedFaults::new(&[FaultPoint::AmbiguousMaterializationResult]);

        assert_eq!(
            executor.apply_with(
                &plan.verified,
                &approval,
                &loaded,
                NOW + 1,
                ApplyKind::Fresh,
                &faults,
            ),
            Err(TransactionError::RecoveryRequired)
        );
        assert!(fixture.target().is_dir());
        assert!(!state_path(&fixture.root, "qiongli-lite-user").exists());
        assert!(journal_path(&fixture.root, "qiongli-lite-user").is_file());
    }

    #[test]
    fn lifecycle_commit_failure_restores_the_active_target() {
        let fixture = Fixture::new("remove-fault");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        executor
            .apply(&plan.verified, &approval, &loaded, NOW + 1)
            .unwrap();
        let faults = InjectedFaults::new(&[FaultPoint::BeforeStateCommit]);

        assert_eq!(
            executor.lifecycle_with(
                "qiongli-lite-user",
                NOW + 2,
                InstallLifecycleKind::Removed,
                &faults,
            ),
            Err(TransactionError::PersistenceFailed(io::ErrorKind::Other))
        );
        assert!(fixture.target().is_dir());
        assert!(executor.verify("qiongli-lite-user").is_ok());
        assert!(!journal_path(&fixture.root, "qiongli-lite-user").exists());
    }

    #[test]
    fn ambiguous_lifecycle_state_commit_retains_state_quarantine_and_journal() {
        let fixture = Fixture::new("remove-state-ambiguity");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        executor
            .apply(&plan.verified, &approval, &loaded, NOW + 1)
            .unwrap();
        let faults = InjectedFaults::new(&[FaultPoint::AfterStateCommit]);

        assert_eq!(
            executor.lifecycle_with(
                "qiongli-lite-user",
                NOW + 2,
                InstallLifecycleKind::Removed,
                &faults,
            ),
            Err(TransactionError::RecoveryRequired)
        );
        assert!(!fixture.target().exists());
        let state = load_state(&fixture.root, "qiongli-lite-user")
            .unwrap()
            .unwrap();
        assert!(state.active.is_none());
        assert_eq!(
            state.last_lifecycle.unwrap().kind,
            InstallLifecycleKind::Removed
        );
        assert!(journal_path(&fixture.root, "qiongli-lite-user").is_file());
        assert!(fs::read_dir(&fixture.root).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(QUARANTINE_MARKER)
        }));
    }

    #[test]
    fn post_commit_cleanup_is_reported_without_false_rollback() {
        let fixture = Fixture::new("remove-cleanup");
        let pack = fixture.pack();
        let loaded = load_resource_pack(pack.core_bytes(), pack.pack_sha256()).unwrap();
        let (executor, plan, approval) = execute_fixture(&fixture, &pack);
        executor
            .apply(&plan.verified, &approval, &loaded, NOW + 1)
            .unwrap();
        let faults = InjectedFaults::new(&[FaultPoint::DuringCleanup]);

        let committed = executor
            .lifecycle_with(
                "qiongli-lite-user",
                NOW + 2,
                InstallLifecycleKind::Removed,
                &faults,
            )
            .expect("cleanup failure must preserve committed state");
        assert_eq!(committed.disposition, LifecycleDisposition::Removed);
        assert!(committed.cleanup_required);
        assert!(!fixture.target().exists());
        let state = load_state(&fixture.root, "qiongli-lite-user")
            .unwrap()
            .unwrap();
        assert!(state.active.is_none());
        assert!(journal_path(&fixture.root, "qiongli-lite-user").is_file());
    }

    #[test]
    fn no_replace_rename_preserves_both_directories_on_conflict() {
        let fixture = Fixture::new("rename-no-replace");
        let source = fixture.root.join("rename-source");
        let destination = fixture.root.join("rename-destination");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("sentinel.txt"), b"keep me\n").unwrap();

        assert!(rename_path(&source, &destination, false).is_err());
        assert!(source.is_dir());
        assert_eq!(
            fs::read(destination.join("sentinel.txt")).unwrap(),
            b"keep me\n"
        );
    }

    #[test]
    fn root_transaction_journal_serializes_distinct_install_ids() {
        let fixture = Fixture::new("root-journal");
        let journal_value = TransactionJournalV1 {
            schema_version: INSTALL_JOURNAL_SCHEMA_VERSION,
            transaction_id: "txn-root-lock".to_string(),
            kind: JournalKind::Apply,
            install_id: "first-install".to_string(),
            plan_digest_sha256: None,
            prior_state_sha256: None,
            target_leaf: "first-target".to_string(),
            started_at_unix: NOW,
        };
        let mut journal = TransactionJournalGuard::create(&fixture.root, journal_value).unwrap();

        assert_eq!(
            ensure_no_journal(&fixture.root, "second-install"),
            Err(TransactionError::RecoveryRequired)
        );
        journal.finish().unwrap();
        assert!(ensure_no_journal(&fixture.root, "second-install").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn managed_root_must_be_private() {
        use std::os::unix::fs::PermissionsExt;

        let fixture = Fixture::new("root-mode");
        fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            approve_managed_root(
                &AllowedRootV1 {
                    id: "qiongli-data".to_string(),
                    root: SymbolicRoot::QiongliManagedData,
                },
                &fixture.root,
            ),
            Err(TransactionError::UnsafeManagedRoot)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn approved_root_identity_rejects_path_replacement() {
        let fixture = Fixture::new("root-identity");
        let approved = fixture.approved_root();
        fs::rename(&fixture.root, fixture.container.join("managed-original")).unwrap();
        create_private_test_root(&fixture.root);

        assert_eq!(
            approved.validate(),
            Err(TransactionError::UnsafeManagedRoot)
        );
    }

    struct InjectedFaults {
        points: Vec<FaultPoint>,
    }

    impl InjectedFaults {
        fn new(points: &[FaultPoint]) -> Self {
            Self {
                points: points.to_vec(),
            }
        }
    }

    impl TransactionFaults for InjectedFaults {
        fn check(&self, point: FaultPoint) -> Result<(), TransactionError> {
            if self.points.contains(&point) {
                Err(TransactionError::PersistenceFailed(io::ErrorKind::Other))
            } else {
                Ok(())
            }
        }
    }

    #[cfg(unix)]
    fn create_private_test_root(path: &Path) {
        use std::os::unix::fs::DirBuilderExt;

        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .expect("private Unix test root must exist");
    }

    #[cfg(windows)]
    fn create_private_test_root(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path)
            .expect("private Windows test root must exist");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_private_test_root(path: &Path) {
        fs::create_dir(path).expect("test root must exist");
    }

    fn temporary_test_signing_key() -> SigningKey {
        let sequence = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
        let mut hasher = Sha256::new();
        hasher.update(b"qiongli-r3b-transaction-test-key\0");
        hasher.update(std::process::id().to_le_bytes());
        hasher.update(sequence.to_le_bytes());
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock must follow Unix epoch")
                .as_nanos()
                .to_le_bytes(),
        );
        SigningKey::from_bytes(&hasher.finalize().into())
    }

    fn encode_hex(input: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(input.len() * 2);
        for byte in input {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }
}
