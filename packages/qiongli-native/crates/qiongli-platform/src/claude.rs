use std::fmt::{self, Debug, Display, Formatter};
#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(any(unix, windows))]
use std::fs::TryLockError;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use same_file::Handle;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::transaction::ApprovedInstallPlan;
use crate::{
    AllowedRootV1, ApprovalRequirement, ArtifactIdentityV1, CapabilityProfile, GrantMode,
    HostAction, InstallActionV1, InstallOperationV1, InstallPlanDraftV1, InstallPlanMetadataV1,
    InstallPlanV1, InstallScope, LocalSurface, LocalTargetFamily, OwnershipMarkerV1, PlanStateV1,
    ProductId, SymbolicRoot, TargetDescriptorV1, VerifiedInstallPlan, VerifiedLaunchGrant,
    observed_plan_state_sha256,
};

pub const CLAUDE_ADAPTER_SCHEMA_VERSION: u32 = 1;
pub const CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION: u32 = 1;
pub const CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION: u32 = 1;

pub const CLAUDE_MARKETPLACE_SYMBOLIC_PATH: &str =
    "<user-home>/.qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json";
pub const CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH: &str =
    "<user-home>/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next";
pub const CLAUDE_PLUGIN_SOURCE_MARKETPLACE_PATH: &str = "./plugins/qiongli-next";
pub const CLAUDE_SKILLS_PLUGIN_SYMBOLIC_PATH: &str = "<claude-config>/skills/qiongli-next";

const MARKETPLACE_RELATIVE_PATH: [&str; 6] = [
    ".qiongli",
    "plugins",
    "claude-code",
    "qiongli-local",
    ".claude-plugin",
    "marketplace.json",
];
const STATE_ROOT_RELATIVE_PATH: [&str; 5] = [".qiongli", "v2", "integrations", "claude-code", ""];
const MARKETPLACE_ROOT_RELATIVE_PATH: [&str; 3] = [".qiongli", "plugins", "claude-code"];
const PLUGIN_SOURCE_RELATIVE_PATH: [&str; 3] = ["qiongli-local", "plugins", "qiongli-next"];
const PLUGIN_SOURCE_LEAF: &str = "qiongli-next";
const INSTALL_ID: &str = "qiongli-next-claude-code-user";
const ROOT_ID: &str = "claude-marketplace-source";
const ENTRY_KEY: &str = "qiongli-next";
const SOURCE_ID: &str = "qiongli-local";
const OPERATION_ID: &str = "claude-register-qiongli-next";
const STATE_FILE_NAME: &str = ".qiongli-next-claude-registration.json";
const JOURNAL_FILE_NAME: &str = ".qiongli-next-claude-registration-journal.json";
const LOCK_FILE_NAME: &str = ".qiongli-next-claude-registration.lock";
const MAX_DOCUMENT_BYTES: u64 = 1024 * 1024;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const EXACT_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeSourceState {
    Missing,
    Ready,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeMarketplaceState {
    Missing,
    Ready,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeSkillsPluginState {
    Missing,
    Ready,
    Conflict,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeRegistrationState {
    Absent,
    Registered,
    Conflict,
    Drifted,
    RecoveryRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeDiscoverySummaryV1 {
    pub schema_version: u32,
    pub skills_plugin_path: &'static str,
    pub skills_plugin: ClaudeSkillsPluginState,
    pub marketplace_path: &'static str,
    pub plugin_source_path: &'static str,
    pub marketplace_source: &'static str,
    pub source: ClaudeSourceState,
    pub marketplace: ClaudeMarketplaceState,
    pub registration: ClaudeRegistrationState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaudeAdapterError {
    UnsupportedPlatform,
    HomeUnavailable,
    UnsafePath,
    SourceMissing,
    SourceInvalid,
    MarketplaceInvalid,
    DocumentTooLarge,
    RegistrationConflict,
    RegistrationDrift,
    RecoveryRequired,
    InvalidPlan,
    InvalidApproval,
    PlanExpired,
    ReceiptMissing,
    ReceiptInvalid,
    LockBusy,
    ObservedStateMismatch,
    PersistenceFailed(io::ErrorKind),
    RollbackFailed,
}

impl ClaudeAdapterError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "claude-adapter-platform-unsupported",
            Self::HomeUnavailable => "claude-home-unavailable",
            Self::UnsafePath => "claude-adapter-path-unsafe",
            Self::SourceMissing => "claude-plugin-source-missing",
            Self::SourceInvalid => "claude-plugin-source-invalid",
            Self::MarketplaceInvalid => "claude-marketplace-invalid",
            Self::DocumentTooLarge => "claude-adapter-document-too-large",
            Self::RegistrationConflict => "claude-registration-conflict",
            Self::RegistrationDrift => "claude-registration-drift",
            Self::RecoveryRequired => "claude-registration-recovery-required",
            Self::InvalidPlan => "claude-registration-plan-invalid",
            Self::InvalidApproval => "claude-registration-approval-invalid",
            Self::PlanExpired => "claude-registration-plan-expired",
            Self::ReceiptMissing => "claude-registration-receipt-missing",
            Self::ReceiptInvalid => "claude-registration-receipt-invalid",
            Self::LockBusy => "claude-registration-busy",
            Self::ObservedStateMismatch => "claude-registration-observed-state-mismatch",
            Self::PersistenceFailed(_) => "claude-registration-persistence-failed",
            Self::RollbackFailed => "claude-registration-rollback-failed",
        }
    }
}

impl Display for ClaudeAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())?;
        if let Self::PersistenceFailed(kind) = self {
            write!(formatter, " ({kind:?})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ClaudeAdapterError {}

#[derive(Clone)]
pub struct ClaudeUserTarget {
    home: PathBuf,
    marketplace_path: PathBuf,
    state_root: PathBuf,
    marketplace: MarketplaceSnapshot,
    source: Option<ClaudeSourceEvidence>,
    registration_state: Option<ClaudeRegistrationStateV1>,
    summary: ClaudeDiscoverySummaryV1,
}

impl ClaudeUserTarget {
    #[must_use]
    pub const fn summary(&self) -> &ClaudeDiscoverySummaryV1 {
        &self.summary
    }

    #[must_use]
    pub fn registration_state(&self) -> Option<&ClaudeRegistrationStateV1> {
        self.registration_state.as_ref()
    }

    #[must_use]
    pub fn registration_state_path(&self) -> PathBuf {
        self.state_root.join(STATE_FILE_NAME)
    }
}

impl Debug for ClaudeUserTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClaudeUserTarget")
            .field("skills_plugin_path", &CLAUDE_SKILLS_PLUGIN_SYMBOLIC_PATH)
            .field("marketplace_path", &CLAUDE_MARKETPLACE_SYMBOLIC_PATH)
            .field("plugin_source", &CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH)
            .field("summary", &self.summary)
            .finish()
    }
}

#[derive(Clone, Debug)]
struct MarketplaceSnapshot {
    present: bool,
    document: Value,
    digest_sha256: String,
}

#[derive(Clone, Debug)]
struct ClaudeSourceEvidence {
    receipt_sha256: String,
    content_root_sha256: String,
    artifact: ArtifactIdentityV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaudeRegistrationEffect {
    Register,
    AlreadyRegistered,
}

#[derive(Clone, Debug)]
pub struct ClaudeRegistrationPreview {
    pub plan: InstallPlanV1,
    pub effect: ClaudeRegistrationEffect,
    pub discovery: ClaudeDiscoverySummaryV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeRegistrationReceiptV1 {
    pub schema_version: u32,
    pub transaction_id: String,
    pub plan_id: String,
    pub semantic_digest_sha256: String,
    pub install_id: String,
    pub artifact: ArtifactIdentityV1,
    pub target: TargetDescriptorV1,
    pub ownership: OwnershipMarkerV1,
    pub source_receipt_sha256: String,
    pub source_content_root_sha256: String,
    pub marketplace_entry_sha256: String,
    pub marketplace_document_sha256: String,
    pub registered_at_unix: u64,
    pub outstanding_host_action: HostAction,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeRegistrationLifecycleKind {
    Removed,
    RolledBack,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeRegistrationLifecycleReceiptV1 {
    pub schema_version: u32,
    pub transaction_id: String,
    pub install_id: String,
    pub prior_transaction_id: String,
    pub kind: ClaudeRegistrationLifecycleKind,
    pub completed_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeRegistrationStateV1 {
    pub schema_version: u32,
    pub generation: u64,
    pub install_id: String,
    pub active: Option<ClaudeRegistrationReceiptV1>,
    pub last_lifecycle: Option<ClaudeRegistrationLifecycleReceiptV1>,
}

impl ClaudeRegistrationStateV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, ClaudeAdapterError> {
        if input.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(ClaudeAdapterError::DocumentTooLarge);
        }
        let state: Self =
            serde_json::from_slice(input).map_err(|_| ClaudeAdapterError::ReceiptInvalid)?;
        state.validate()?;
        if canonical_json(&state)? != input {
            return Err(ClaudeAdapterError::ReceiptInvalid);
        }
        Ok(state)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ClaudeAdapterError> {
        self.validate()?;
        let bytes = canonical_json(self)?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(ClaudeAdapterError::DocumentTooLarge);
        }
        Ok(bytes)
    }

    fn validate(&self) -> Result<(), ClaudeAdapterError> {
        if self.schema_version != CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION
            || self.generation == 0
            || self.generation > JCS_MAX_SAFE_INTEGER
            || self.install_id != INSTALL_ID
            || (self.active.is_none() && self.last_lifecycle.is_none())
        {
            return Err(ClaudeAdapterError::ReceiptInvalid);
        }
        if let Some(active) = &self.active {
            active.validate()?;
            if active.install_id != self.install_id {
                return Err(ClaudeAdapterError::ReceiptInvalid);
            }
        }
        if let Some(lifecycle) = &self.last_lifecycle {
            lifecycle.validate()?;
            if lifecycle.install_id != self.install_id {
                return Err(ClaudeAdapterError::ReceiptInvalid);
            }
        }
        Ok(())
    }
}

impl ClaudeRegistrationReceiptV1 {
    fn validate(&self) -> Result<(), ClaudeAdapterError> {
        if self.schema_version != CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.plan_id)
            || !valid_lower_hex(&self.semantic_digest_sha256, 64)
            || self.install_id != INSTALL_ID
            || self.ownership.install_id != INSTALL_ID
            || self.ownership.product != ProductId::Qiongli
            || self.ownership.schema_version != 1
            || !valid_lower_hex(&self.ownership.artifact_digest_sha256, 64)
            || !valid_lower_hex(&self.source_receipt_sha256, 64)
            || !valid_lower_hex(&self.source_content_root_sha256, 64)
            || !valid_lower_hex(&self.marketplace_entry_sha256, 64)
            || !valid_lower_hex(&self.marketplace_document_sha256, 64)
            || self.registered_at_unix > JCS_MAX_SAFE_INTEGER
            || self.outstanding_host_action != HostAction::InstallOrEnablePlugin
            || self.target.family != LocalTargetFamily::ClaudeCodeLocal
            || self.target.surface != LocalSurface::CliLocal
            || self.target.scope != InstallScope::User
            || self.target.profile != CapabilityProfile::Lite
            || self.target.adapter_version != 1
            || self.target.os != self.artifact.os
            || self.target.arch != self.artifact.arch
        {
            return Err(ClaudeAdapterError::ReceiptInvalid);
        }
        self.artifact
            .validate()
            .map_err(|_| ClaudeAdapterError::ReceiptInvalid)
    }
}

impl ClaudeRegistrationLifecycleReceiptV1 {
    fn validate(&self) -> Result<(), ClaudeAdapterError> {
        if self.schema_version != CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || self.install_id != INSTALL_ID
            || !valid_identifier(&self.prior_transaction_id)
            || self.completed_at_unix > JCS_MAX_SAFE_INTEGER
        {
            return Err(ClaudeAdapterError::ReceiptInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaudeRegistrationDisposition {
    Registered,
    AlreadyRegistered,
    Repaired,
    AlreadyHealthy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeRegistrationCommit {
    pub disposition: ClaudeRegistrationDisposition,
    pub receipt: ClaudeRegistrationReceiptV1,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeRegistrationVerification {
    pub receipt: ClaudeRegistrationReceiptV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaudeRegistrationLifecycleDisposition {
    Removed,
    AlreadyRemoved,
    RolledBack,
    AlreadyRolledBack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeRegistrationLifecycleCommit {
    pub disposition: ClaudeRegistrationLifecycleDisposition,
    pub receipt: ClaudeRegistrationLifecycleReceiptV1,
    pub cleanup_required: bool,
}

pub fn discover_claude_user(
    home: impl AsRef<Path>,
) -> Result<ClaudeUserTarget, ClaudeAdapterError> {
    let home = home.as_ref();
    discover_claude_user_with_config(home, home.join(".claude"))
}

pub fn discover_claude_user_with_config(
    home: impl AsRef<Path>,
    claude_config_root: impl AsRef<Path>,
) -> Result<ClaudeUserTarget, ClaudeAdapterError> {
    let home = home.as_ref();
    let claude_config_root = claude_config_root.as_ref();
    validate_home(home)?;
    if !claude_config_root.is_absolute() || has_lexical_traversal(claude_config_root) {
        return Err(ClaudeAdapterError::UnsafePath);
    }
    let skills_plugin = discover_skills_plugin(claude_config_root)?;
    let marketplace_path = marketplace_path(home);
    let state_root = state_root_path(home);
    let plugin_source = plugin_source_path(home);

    // Claude's existing marketplace is client-observable content and may predate
    // Qiongli 2. Safe owner-controlled 0755 directories are valid for read-only
    // discovery; Qiongli 2 transaction state has its own owner-private root.
    validate_optional_directory_chain(home, &[".qiongli", "plugins", "claude-code"], false)?;
    validate_optional_directory_chain(
        home,
        &[".qiongli", "plugins", "claude-code", "qiongli-local"],
        false,
    )?;
    validate_optional_directory_chain(
        home,
        &[".qiongli", "v2", "integrations", "claude-code"],
        true,
    )?;
    let marketplace = read_marketplace(&marketplace_path)?;
    let marketplace_state = if marketplace.present {
        ClaudeMarketplaceState::Ready
    } else {
        ClaudeMarketplaceState::Missing
    };

    let state_root_exists = path_exists(&state_root)?;
    if state_root_exists {
        validate_directory(&state_root, true)?;
    }
    let source = if path_exists(&plugin_source)? {
        Some(validate_plugin_source(&plugin_source)?)
    } else {
        None
    };
    let source_state = if source.is_some() {
        ClaudeSourceState::Ready
    } else {
        ClaudeSourceState::Missing
    };

    let registration_state = if state_root_exists {
        load_registration_state(&state_root)?
    } else {
        None
    };
    let recovery = state_root_exists && path_exists(&state_root.join(JOURNAL_FILE_NAME))?;
    let registration = classify_registration(
        &marketplace.document,
        source.as_ref(),
        registration_state.as_ref(),
        recovery,
    )?;

    Ok(ClaudeUserTarget {
        home: home.to_path_buf(),
        marketplace_path,
        state_root,
        marketplace,
        source,
        registration_state,
        summary: ClaudeDiscoverySummaryV1 {
            schema_version: CLAUDE_ADAPTER_SCHEMA_VERSION,
            skills_plugin_path: CLAUDE_SKILLS_PLUGIN_SYMBOLIC_PATH,
            skills_plugin,
            marketplace_path: CLAUDE_MARKETPLACE_SYMBOLIC_PATH,
            plugin_source_path: CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH,
            marketplace_source: CLAUDE_PLUGIN_SOURCE_MARKETPLACE_PATH,
            source: source_state,
            marketplace: marketplace_state,
            registration,
        },
    })
}

pub(crate) fn prepare_claude_plugin_source_target(
    home: impl AsRef<Path>,
) -> Result<crate::ClaudePluginBundleTarget, ClaudeAdapterError> {
    let home = home.as_ref();
    validate_home(home)?;
    let qiongli_root = home.join(".qiongli");
    ensure_private_directory(&qiongli_root)?;
    let plugin_root = qiongli_root.join("plugins");
    ensure_private_directory(&plugin_root)?;
    let client_plugin_root = plugin_root.join("claude-code");
    ensure_private_directory(&client_plugin_root)?;
    let marketplace_root = client_plugin_root.join("qiongli-local");
    ensure_private_directory(&marketplace_root)?;
    let marketplace_plugins = marketplace_root.join("plugins");
    ensure_private_directory(&marketplace_plugins)?;

    prepare_claude_state_root(home)?;
    crate::approve_claude_plugin_bundle_target(marketplace_plugins.join(PLUGIN_SOURCE_LEAF))
        .map_err(|_| ClaudeAdapterError::UnsafePath)
}

fn prepare_claude_state_root(home: &Path) -> Result<(), ClaudeAdapterError> {
    validate_home(home)?;
    let qiongli_root = home.join(".qiongli");
    ensure_private_directory(&qiongli_root)?;
    let v2_root = qiongli_root.join("v2");
    ensure_private_directory(&v2_root)?;
    let integrations_root = v2_root.join("integrations");
    ensure_private_directory(&integrations_root)?;
    ensure_owner_private_directory(&integrations_root.join("claude-code"))
}

fn discover_skills_plugin(
    claude_config_root: &Path,
) -> Result<ClaudeSkillsPluginState, ClaudeAdapterError> {
    if !path_exists(claude_config_root)? {
        return Ok(ClaudeSkillsPluginState::Missing);
    }
    validate_directory(claude_config_root, false)?;
    let skills_root = claude_config_root.join("skills");
    if !path_exists(&skills_root)? {
        return Ok(ClaudeSkillsPluginState::Missing);
    }
    if validate_directory(&skills_root, false).is_err() {
        return Ok(ClaudeSkillsPluginState::Conflict);
    }
    let plugin = skills_root.join("qiongli-next");
    if !path_exists(&plugin)? {
        return Ok(ClaudeSkillsPluginState::Missing);
    }
    let target = match crate::approve_claude_plugin_bundle_target(&plugin) {
        Ok(target) => target,
        Err(_) => return Ok(ClaudeSkillsPluginState::Conflict),
    };
    match crate::verify_claude_plugin_bundle(&target) {
        Ok(_) => Ok(ClaudeSkillsPluginState::Ready),
        Err(_) => Ok(ClaudeSkillsPluginState::Conflict),
    }
}

pub fn preview_claude_registration(
    target: &ClaudeUserTarget,
    metadata: InstallPlanMetadataV1,
    grant: &VerifiedLaunchGrant,
) -> Result<ClaudeRegistrationPreview, ClaudeAdapterError> {
    if grant.authorized_mode() != GrantMode::FullMcp {
        return Err(ClaudeAdapterError::InvalidPlan);
    }
    if target.summary.registration == ClaudeRegistrationState::RecoveryRequired {
        return Err(ClaudeAdapterError::RecoveryRequired);
    }
    if matches!(
        target.summary.registration,
        ClaudeRegistrationState::Conflict | ClaudeRegistrationState::Drifted
    ) {
        return Err(match target.summary.registration {
            ClaudeRegistrationState::Conflict => ClaudeAdapterError::RegistrationConflict,
            ClaudeRegistrationState::Drifted => ClaudeAdapterError::RegistrationDrift,
            _ => ClaudeAdapterError::InvalidPlan,
        });
    }
    let source = target
        .source
        .as_ref()
        .ok_or(ClaudeAdapterError::SourceMissing)?;
    if source.artifact != grant.grant().artifact {
        return Err(ClaudeAdapterError::SourceInvalid);
    }
    let ownership = OwnershipMarkerV1 {
        schema_version: 1,
        product: ProductId::Qiongli,
        install_id: INSTALL_ID.to_string(),
        artifact_digest_sha256: grant.signed_payload_sha256().to_string(),
    };
    let entry_digest = marketplace_entry_digest()?;
    let precondition = match target.summary.registration {
        ClaudeRegistrationState::Absent => PlanStateV1::Missing,
        ClaudeRegistrationState::Registered => {
            let active = target
                .registration_state
                .as_ref()
                .and_then(|state| state.active.as_ref())
                .ok_or(ClaudeAdapterError::RegistrationDrift)?;
            if active.ownership != ownership
                || active.source_receipt_sha256 != source.receipt_sha256
                || active.marketplace_entry_sha256 != entry_digest
            {
                return Err(ClaudeAdapterError::RegistrationConflict);
            }
            PlanStateV1::Managed {
                ownership: ownership.clone(),
                content_sha256: entry_digest.clone(),
            }
        }
        _ => return Err(ClaudeAdapterError::InvalidPlan),
    };
    let postcondition = PlanStateV1::Managed {
        ownership: ownership.clone(),
        content_sha256: entry_digest.clone(),
    };
    let observed_state_sha256 =
        observed_plan_state_sha256(&precondition).map_err(|_| ClaudeAdapterError::InvalidPlan)?;
    let artifact = grant.grant().artifact.clone();
    let plan = InstallPlanV1::build(
        metadata,
        grant,
        InstallPlanDraftV1 {
            target: TargetDescriptorV1 {
                family: LocalTargetFamily::ClaudeCodeLocal,
                surface: LocalSurface::CliLocal,
                scope: InstallScope::User,
                profile: CapabilityProfile::Lite,
                os: artifact.os,
                arch: artifact.arch,
                adapter_version: 1,
            },
            allowed_roots: vec![AllowedRootV1 {
                id: ROOT_ID.to_string(),
                root: SymbolicRoot::ClaudeMarketplaceSource,
            }],
            operations: vec![InstallOperationV1 {
                operation_id: OPERATION_ID.to_string(),
                action: InstallActionV1::RegisterPluginSource {
                    root_id: ROOT_ID.to_string(),
                    entry_key: ENTRY_KEY.to_string(),
                    source_id: SOURCE_ID.to_string(),
                    source_digest_sha256: source.receipt_sha256.clone(),
                    ownership: ownership.clone(),
                },
                precondition,
                observed_state_sha256,
                postcondition,
                inverse: InstallActionV1::RemoveManagedEntry {
                    root_id: ROOT_ID.to_string(),
                    entry_key: ENTRY_KEY.to_string(),
                    expected_ownership: ownership,
                    expected_sha256: entry_digest,
                },
            }],
            approvals_required: EXACT_APPROVALS.to_vec(),
            outstanding_host_action: Some(HostAction::InstallOrEnablePlugin),
        },
    )
    .map_err(|_| ClaudeAdapterError::InvalidPlan)?;

    Ok(ClaudeRegistrationPreview {
        plan,
        effect: if target.summary.registration == ClaudeRegistrationState::Registered {
            ClaudeRegistrationEffect::AlreadyRegistered
        } else {
            ClaudeRegistrationEffect::Register
        },
        discovery: target.summary.clone(),
    })
}

#[derive(Clone, Debug)]
pub struct ClaudeRegistrationExecutor {
    home: PathBuf,
}

impl ClaudeRegistrationExecutor {
    #[must_use]
    pub fn new(target: ClaudeUserTarget) -> Self {
        Self { home: target.home }
    }

    pub fn apply(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        now_unix: u64,
    ) -> Result<ClaudeRegistrationCommit, ClaudeAdapterError> {
        self.apply_or_repair(plan, approval, now_unix, false)
    }

    pub fn repair(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        now_unix: u64,
    ) -> Result<ClaudeRegistrationCommit, ClaudeAdapterError> {
        self.apply_or_repair(plan, approval, now_unix, true)
    }

    pub fn verify(&self) -> Result<ClaudeRegistrationVerification, ClaudeAdapterError> {
        let target = discover_claude_user(&self.home)?;
        if target.summary.registration == ClaudeRegistrationState::RecoveryRequired {
            return Err(ClaudeAdapterError::RecoveryRequired);
        }
        if target.summary.registration != ClaudeRegistrationState::Registered {
            return Err(ClaudeAdapterError::RegistrationDrift);
        }
        let receipt = target
            .registration_state
            .and_then(|state| state.active)
            .ok_or(ClaudeAdapterError::ReceiptMissing)?;
        receipt.validate()?;
        Ok(ClaudeRegistrationVerification { receipt })
    }

    pub fn remove(
        &self,
        now_unix: u64,
    ) -> Result<ClaudeRegistrationLifecycleCommit, ClaudeAdapterError> {
        self.lifecycle(now_unix, ClaudeRegistrationLifecycleKind::Removed)
    }

    pub fn rollback(
        &self,
        now_unix: u64,
    ) -> Result<ClaudeRegistrationLifecycleCommit, ClaudeAdapterError> {
        self.lifecycle(now_unix, ClaudeRegistrationLifecycleKind::RolledBack)
    }

    fn apply_or_repair(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        now_unix: u64,
        repair: bool,
    ) -> Result<ClaudeRegistrationCommit, ClaudeAdapterError> {
        approval.validate_for(plan, now_unix).map_err(|error| {
            use crate::TransactionError;
            match error {
                TransactionError::PlanExpired => ClaudeAdapterError::PlanExpired,
                _ => ClaudeAdapterError::InvalidApproval,
            }
        })?;
        let initial = discover_claude_user(&self.home)?;
        let executable = ExecutableClaudeRegistration::from_plan(plan, &initial)?;
        // Discovery and preview remain read-only. An approved apply may create the
        // private transaction root before taking its mutation lock.
        prepare_claude_state_root(&self.home)?;
        let _lock = acquire_lock(&initial.state_root)?;
        let current = discover_claude_user(&self.home)?;
        executable.revalidate_source(&current)?;
        if current.summary.registration == ClaudeRegistrationState::RecoveryRequired {
            return Err(ClaudeAdapterError::RecoveryRequired);
        }

        if !repair {
            match current.summary.registration {
                ClaudeRegistrationState::Registered => {
                    let receipt = current
                        .registration_state
                        .and_then(|state| state.active)
                        .ok_or(ClaudeAdapterError::ReceiptMissing)?;
                    executable.validate_active(&receipt)?;
                    return Ok(ClaudeRegistrationCommit {
                        disposition: ClaudeRegistrationDisposition::AlreadyRegistered,
                        receipt,
                        cleanup_required: false,
                    });
                }
                ClaudeRegistrationState::Absent => {}
                ClaudeRegistrationState::Conflict => {
                    return Err(ClaudeAdapterError::RegistrationConflict);
                }
                ClaudeRegistrationState::Drifted => {
                    return Err(ClaudeAdapterError::RegistrationDrift);
                }
                ClaudeRegistrationState::RecoveryRequired => {
                    return Err(ClaudeAdapterError::RecoveryRequired);
                }
            }
            if executable.precondition != PlanStateV1::Missing {
                return Err(ClaudeAdapterError::ObservedStateMismatch);
            }
        } else {
            match current.summary.registration {
                ClaudeRegistrationState::Registered => {
                    let receipt = current
                        .registration_state
                        .and_then(|state| state.active)
                        .ok_or(ClaudeAdapterError::ReceiptMissing)?;
                    executable.validate_active(&receipt)?;
                    return Ok(ClaudeRegistrationCommit {
                        disposition: ClaudeRegistrationDisposition::AlreadyHealthy,
                        receipt,
                        cleanup_required: false,
                    });
                }
                ClaudeRegistrationState::Drifted => {}
                _ => return Err(ClaudeAdapterError::RegistrationDrift),
            }
        }

        let prior_state = current.registration_state.clone();
        let prior_active = prior_state.as_ref().and_then(|state| state.active.as_ref());
        if repair {
            executable.validate_active(prior_active.ok_or(ClaudeAdapterError::ReceiptMissing)?)?;
            if !qiongli_entries(&current.marketplace.document)?.is_empty() {
                return Err(ClaudeAdapterError::RegistrationDrift);
            }
        }

        let next_document = insert_marketplace_entry(&current.marketplace.document)?;
        let next_digest = document_digest(&next_document)?;
        let transaction_id = transaction_id();
        let journal = ClaudeRegistrationJournalV1 {
            schema_version: CLAUDE_ADAPTER_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            kind: if repair {
                ClaudeJournalKind::Repair
            } else {
                ClaudeJournalKind::Apply
            },
            prior_marketplace_present: current.marketplace.present,
            prior_marketplace: current.marketplace.document.clone(),
            prior_marketplace_sha256: current.marketplace.digest_sha256.clone(),
            next_marketplace_sha256: next_digest.clone(),
            prior_state_sha256: prior_state
                .as_ref()
                .map(ClaudeRegistrationStateV1::to_canonical_json)
                .transpose()?
                .as_deref()
                .map(sha256_hex),
            started_at_unix: now_unix,
        };
        persist_new_journal(&current.state_root, &journal)?;

        if let Err(error) = activate_marketplace(
            &current.marketplace_path,
            &current.marketplace,
            &next_document,
            &transaction_id,
        ) {
            return rollback_after_activation(&current, &journal, error);
        }

        let receipt = if repair {
            prior_active
                .cloned()
                .ok_or(ClaudeAdapterError::ReceiptMissing)?
        } else {
            ClaudeRegistrationReceiptV1 {
                schema_version: CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION,
                transaction_id: transaction_id.clone(),
                plan_id: executable.plan.plan().plan_id.clone(),
                semantic_digest_sha256: executable.plan.plan().semantic_digest_sha256.clone(),
                install_id: INSTALL_ID.to_string(),
                artifact: executable.plan.plan().artifact.clone(),
                target: executable.plan.plan().target.clone(),
                ownership: executable.ownership.clone(),
                source_receipt_sha256: executable.source_receipt_sha256.clone(),
                source_content_root_sha256: executable.source_content_root_sha256.clone(),
                marketplace_entry_sha256: executable.entry_sha256.clone(),
                marketplace_document_sha256: next_digest,
                registered_at_unix: now_unix,
                outstanding_host_action: HostAction::InstallOrEnablePlugin,
            }
        };
        receipt.validate()?;

        if !repair {
            let state = ClaudeRegistrationStateV1 {
                schema_version: CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
                generation: next_generation(prior_state.as_ref())?,
                install_id: INSTALL_ID.to_string(),
                active: Some(receipt.clone()),
                last_lifecycle: prior_state.and_then(|state| state.last_lifecycle),
            };
            if let Err(error) =
                persist_registration_state(&current.state_root, &state, &transaction_id)
            {
                return recover_after_state_failure(&current, &journal, error);
            }
        }

        let cleanup_required = finish_journal(&current.state_root).is_err();
        Ok(ClaudeRegistrationCommit {
            disposition: if repair {
                ClaudeRegistrationDisposition::Repaired
            } else {
                ClaudeRegistrationDisposition::Registered
            },
            receipt,
            cleanup_required,
        })
    }

    fn lifecycle(
        &self,
        now_unix: u64,
        kind: ClaudeRegistrationLifecycleKind,
    ) -> Result<ClaudeRegistrationLifecycleCommit, ClaudeAdapterError> {
        let initial = discover_claude_user(&self.home)?;
        if !path_exists(&initial.state_root)? {
            return Err(ClaudeAdapterError::ReceiptMissing);
        }
        let _lock = acquire_lock(&initial.state_root)?;
        let current = discover_claude_user(&self.home)?;
        if current.summary.registration == ClaudeRegistrationState::RecoveryRequired {
            return Err(ClaudeAdapterError::RecoveryRequired);
        }
        let prior_state = current
            .registration_state
            .clone()
            .ok_or(ClaudeAdapterError::ReceiptMissing)?;
        let Some(active) = prior_state.active.as_ref() else {
            let lifecycle = prior_state
                .last_lifecycle
                .ok_or(ClaudeAdapterError::ReceiptMissing)?;
            if lifecycle.kind != kind {
                return Err(ClaudeAdapterError::ReceiptMissing);
            }
            return Ok(ClaudeRegistrationLifecycleCommit {
                disposition: already_lifecycle_disposition(kind),
                receipt: lifecycle,
                cleanup_required: false,
            });
        };
        active.validate()?;
        let entries = qiongli_entries(&current.marketplace.document)?;
        if let Some(entry) = entries.first()
            && (entries.len() != 1 || value_digest(entry)? != active.marketplace_entry_sha256)
        {
            return Err(ClaudeAdapterError::RegistrationDrift);
        }
        let next_document = remove_marketplace_entry(&current.marketplace.document)?;
        let next_digest = document_digest(&next_document)?;
        let transaction_id = transaction_id();
        let journal = ClaudeRegistrationJournalV1 {
            schema_version: CLAUDE_ADAPTER_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            kind: match kind {
                ClaudeRegistrationLifecycleKind::Removed => ClaudeJournalKind::Remove,
                ClaudeRegistrationLifecycleKind::RolledBack => ClaudeJournalKind::Rollback,
            },
            prior_marketplace_present: current.marketplace.present,
            prior_marketplace: current.marketplace.document.clone(),
            prior_marketplace_sha256: current.marketplace.digest_sha256.clone(),
            next_marketplace_sha256: next_digest,
            prior_state_sha256: Some(sha256_hex(&prior_state.to_canonical_json()?)),
            started_at_unix: now_unix,
        };
        persist_new_journal(&current.state_root, &journal)?;
        if next_document != current.marketplace.document
            && let Err(error) = activate_marketplace(
                &current.marketplace_path,
                &current.marketplace,
                &next_document,
                &transaction_id,
            )
        {
            return rollback_after_activation(&current, &journal, error);
        }
        let lifecycle = ClaudeRegistrationLifecycleReceiptV1 {
            schema_version: CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            install_id: INSTALL_ID.to_string(),
            prior_transaction_id: active.transaction_id.clone(),
            kind,
            completed_at_unix: now_unix,
        };
        lifecycle.validate()?;
        let state = ClaudeRegistrationStateV1 {
            schema_version: CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
            generation: next_generation(Some(&prior_state))?,
            install_id: INSTALL_ID.to_string(),
            active: None,
            last_lifecycle: Some(lifecycle.clone()),
        };
        if let Err(error) = persist_registration_state(&current.state_root, &state, &transaction_id)
        {
            return recover_after_state_failure(&current, &journal, error);
        }
        let cleanup_required = finish_journal(&current.state_root).is_err();
        Ok(ClaudeRegistrationLifecycleCommit {
            disposition: lifecycle_disposition(kind),
            receipt: lifecycle,
            cleanup_required,
        })
    }
}

struct ExecutableClaudeRegistration<'a> {
    plan: &'a VerifiedInstallPlan,
    precondition: PlanStateV1,
    ownership: OwnershipMarkerV1,
    source_receipt_sha256: String,
    source_content_root_sha256: String,
    entry_sha256: String,
}

impl<'a> ExecutableClaudeRegistration<'a> {
    fn from_plan(
        plan: &'a VerifiedInstallPlan,
        target: &ClaudeUserTarget,
    ) -> Result<Self, ClaudeAdapterError> {
        let plan_value = plan.plan();
        if plan_value.target.family != LocalTargetFamily::ClaudeCodeLocal
            || plan_value.target.surface != LocalSurface::CliLocal
            || plan_value.target.scope != InstallScope::User
            || plan_value.target.profile != CapabilityProfile::Lite
            || plan_value.target.adapter_version != 1
            || plan_value.allowed_roots
                != [AllowedRootV1 {
                    id: ROOT_ID.to_string(),
                    root: SymbolicRoot::ClaudeMarketplaceSource,
                }]
            || plan_value.approvals_required != EXACT_APPROVALS
            || plan_value.outstanding_host_action != Some(HostAction::InstallOrEnablePlugin)
            || plan_value.operations.len() != 1
        {
            return Err(ClaudeAdapterError::InvalidPlan);
        }
        let operation = &plan_value.operations[0];
        if operation.operation_id != OPERATION_ID {
            return Err(ClaudeAdapterError::InvalidPlan);
        }
        let InstallActionV1::RegisterPluginSource {
            root_id,
            entry_key,
            source_id,
            source_digest_sha256,
            ownership,
        } = &operation.action
        else {
            return Err(ClaudeAdapterError::InvalidPlan);
        };
        let source = target
            .source
            .as_ref()
            .ok_or(ClaudeAdapterError::SourceMissing)?;
        let entry_sha256 = marketplace_entry_digest()?;
        let postcondition = PlanStateV1::Managed {
            ownership: ownership.clone(),
            content_sha256: entry_sha256.clone(),
        };
        if root_id != ROOT_ID
            || entry_key != ENTRY_KEY
            || source_id != SOURCE_ID
            || source_digest_sha256 != &source.receipt_sha256
            || source.artifact != plan.grant().grant().artifact
            || ownership.install_id != INSTALL_ID
            || ownership.product != ProductId::Qiongli
            || ownership.artifact_digest_sha256 != plan.grant().signed_payload_sha256()
            || operation.observed_state_sha256
                != observed_plan_state_sha256(&operation.precondition)
                    .map_err(|_| ClaudeAdapterError::InvalidPlan)?
            || operation.postcondition != postcondition
        {
            return Err(ClaudeAdapterError::InvalidPlan);
        }
        let InstallActionV1::RemoveManagedEntry {
            root_id: inverse_root,
            entry_key: inverse_entry,
            expected_ownership,
            expected_sha256,
        } = &operation.inverse
        else {
            return Err(ClaudeAdapterError::InvalidPlan);
        };
        if inverse_root != ROOT_ID
            || inverse_entry != ENTRY_KEY
            || expected_ownership != ownership
            || expected_sha256 != &entry_sha256
        {
            return Err(ClaudeAdapterError::InvalidPlan);
        }
        Ok(Self {
            plan,
            precondition: operation.precondition.clone(),
            ownership: ownership.clone(),
            source_receipt_sha256: source.receipt_sha256.clone(),
            source_content_root_sha256: source.content_root_sha256.clone(),
            entry_sha256,
        })
    }

    fn revalidate_source(&self, target: &ClaudeUserTarget) -> Result<(), ClaudeAdapterError> {
        let source = target
            .source
            .as_ref()
            .ok_or(ClaudeAdapterError::SourceMissing)?;
        if source.receipt_sha256 != self.source_receipt_sha256
            || source.content_root_sha256 != self.source_content_root_sha256
        {
            return Err(ClaudeAdapterError::ObservedStateMismatch);
        }
        Ok(())
    }

    fn validate_active(
        &self,
        active: &ClaudeRegistrationReceiptV1,
    ) -> Result<(), ClaudeAdapterError> {
        active.validate()?;
        if active.ownership != self.ownership
            || active.source_receipt_sha256 != self.source_receipt_sha256
            || active.source_content_root_sha256 != self.source_content_root_sha256
            || active.marketplace_entry_sha256 != self.entry_sha256
        {
            return Err(ClaudeAdapterError::RegistrationConflict);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ClaudeJournalKind {
    Apply,
    Repair,
    Remove,
    Rollback,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClaudeRegistrationJournalV1 {
    schema_version: u32,
    transaction_id: String,
    kind: ClaudeJournalKind,
    prior_marketplace_present: bool,
    prior_marketplace: Value,
    prior_marketplace_sha256: String,
    next_marketplace_sha256: String,
    prior_state_sha256: Option<String>,
    started_at_unix: u64,
}

fn classify_registration(
    marketplace: &Value,
    source: Option<&ClaudeSourceEvidence>,
    state: Option<&ClaudeRegistrationStateV1>,
    recovery: bool,
) -> Result<ClaudeRegistrationState, ClaudeAdapterError> {
    if recovery {
        return Ok(ClaudeRegistrationState::RecoveryRequired);
    }
    let entries = qiongli_entries(marketplace)?;
    if entries.len() > 1 {
        return Ok(ClaudeRegistrationState::Conflict);
    }
    let active = state.and_then(|state| state.active.as_ref());
    match (entries.first(), active) {
        (None, None) => Ok(ClaudeRegistrationState::Absent),
        (Some(_), None) => Ok(ClaudeRegistrationState::Conflict),
        (None, Some(_)) => Ok(ClaudeRegistrationState::Drifted),
        (Some(entry), Some(active)) => {
            if source.is_some_and(|source| {
                source.receipt_sha256 == active.source_receipt_sha256
                    && source.content_root_sha256 == active.source_content_root_sha256
            }) && value_digest(entry)? == active.marketplace_entry_sha256
            {
                Ok(ClaudeRegistrationState::Registered)
            } else {
                Ok(ClaudeRegistrationState::Drifted)
            }
        }
    }
}

fn validate_plugin_source(path: &Path) -> Result<ClaudeSourceEvidence, ClaudeAdapterError> {
    let target = crate::approve_claude_plugin_bundle_target(path)
        .map_err(|_| ClaudeAdapterError::SourceInvalid)?;
    let verified = crate::verify_claude_plugin_bundle(&target)
        .map_err(|_| ClaudeAdapterError::SourceInvalid)?;
    let receipt = verified.receipt();
    Ok(ClaudeSourceEvidence {
        receipt_sha256: verified.receipt_sha256().to_string(),
        content_root_sha256: receipt.package_content_root_sha256.clone(),
        artifact: receipt.artifact.clone(),
    })
}

fn marketplace_entry() -> Value {
    json!({
        "name": "qiongli-next",
        "source": CLAUDE_PLUGIN_SOURCE_MARKETPLACE_PATH
    })
}

fn marketplace_entry_digest() -> Result<String, ClaudeAdapterError> {
    value_digest(&marketplace_entry())
}

fn default_marketplace() -> Value {
    json!({
        "name": "qiongli-local",
        "description": "Local Qiongli-managed Claude Code plugins.",
        "owner": { "name": "Qiongli" },
        "plugins": []
    })
}

fn read_marketplace(path: &Path) -> Result<MarketplaceSnapshot, ClaudeAdapterError> {
    if !path_exists(path)? {
        let document = default_marketplace();
        return Ok(MarketplaceSnapshot {
            present: false,
            digest_sha256: document_digest(&document)?,
            document,
        });
    }
    let bytes = read_bounded_config_file(path, MAX_DOCUMENT_BYTES)?;
    let document: Value =
        serde_json::from_slice(&bytes).map_err(|_| ClaudeAdapterError::MarketplaceInvalid)?;
    validate_marketplace_document(&document)?;
    Ok(MarketplaceSnapshot {
        present: true,
        digest_sha256: document_digest(&document)?,
        document,
    })
}

fn validate_marketplace_document(document: &Value) -> Result<(), ClaudeAdapterError> {
    let object = document
        .as_object()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    if object.get("name").and_then(Value::as_str) != Some("qiongli-local")
        || object
            .get("owner")
            .and_then(Value::as_object)
            .and_then(|owner| owner.get("name"))
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || object
            .get("plugins")
            .is_none_or(|plugins| !plugins.is_array())
    {
        return Err(ClaudeAdapterError::MarketplaceInvalid);
    }
    if qiongli_entries(document)?.len() > 1 {
        return Err(ClaudeAdapterError::RegistrationConflict);
    }
    Ok(())
}

fn qiongli_entries(document: &Value) -> Result<Vec<&Value>, ClaudeAdapterError> {
    let object = document
        .as_object()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    let Some(plugins) = object.get("plugins") else {
        return Ok(Vec::new());
    };
    let plugins = plugins
        .as_array()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    Ok(plugins
        .iter()
        .filter(|entry| entry.get("name").and_then(Value::as_str) == Some(ENTRY_KEY))
        .collect())
}

fn insert_marketplace_entry(document: &Value) -> Result<Value, ClaudeAdapterError> {
    if !qiongli_entries(document)?.is_empty() {
        return Err(ClaudeAdapterError::RegistrationConflict);
    }
    let mut object = document
        .as_object()
        .cloned()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    object
        .entry("name")
        .or_insert_with(|| Value::String("qiongli-local".to_string()));
    object
        .entry("description")
        .or_insert_with(|| Value::String("Local Qiongli-managed Claude Code plugins.".to_string()));
    object
        .entry("owner")
        .or_insert_with(|| json!({ "name": "Qiongli" }));
    let plugins = object
        .entry("plugins")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    plugins.push(marketplace_entry());
    Ok(Value::Object(object))
}

fn remove_marketplace_entry(document: &Value) -> Result<Value, ClaudeAdapterError> {
    let mut object = document
        .as_object()
        .cloned()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    let Some(plugins) = object.get_mut("plugins") else {
        return Ok(Value::Object(object));
    };
    let plugins = plugins
        .as_array_mut()
        .ok_or(ClaudeAdapterError::MarketplaceInvalid)?;
    plugins.retain(|entry| entry.get("name").and_then(Value::as_str) != Some(ENTRY_KEY));
    Ok(Value::Object(object))
}

fn activate_marketplace(
    path: &Path,
    prior: &MarketplaceSnapshot,
    next: &Value,
    transaction_id: &str,
) -> Result<(), ClaudeAdapterError> {
    prepare_marketplace_parent(path)?;
    let observed = read_marketplace(path)?;
    if observed.present != prior.present || observed.digest_sha256 != prior.digest_sha256 {
        return Err(ClaudeAdapterError::ObservedStateMismatch);
    }
    write_marketplace_document(path, next, transaction_id, prior.present)?;
    let committed = read_marketplace(path).map_err(|_| ClaudeAdapterError::RecoveryRequired)?;
    if !committed.present || committed.digest_sha256 != document_digest(next)? {
        return Err(ClaudeAdapterError::RecoveryRequired);
    }
    Ok(())
}

fn rollback_after_activation<T>(
    target: &ClaudeUserTarget,
    journal: &ClaudeRegistrationJournalV1,
    original: ClaudeAdapterError,
) -> Result<T, ClaudeAdapterError> {
    if restore_prior_marketplace(&target.marketplace_path, journal).is_ok()
        && finish_journal(&target.state_root).is_ok()
    {
        Err(original)
    } else {
        Err(ClaudeAdapterError::RecoveryRequired)
    }
}

fn recover_after_state_failure<T>(
    target: &ClaudeUserTarget,
    journal: &ClaudeRegistrationJournalV1,
    _original: ClaudeAdapterError,
) -> Result<T, ClaudeAdapterError> {
    // State activation may already have happened even when durability or
    // verification reports an error. Restore the marketplace when it is still
    // safe to do so, but retain the journal until a recovery path can prove the
    // state receipt outcome.
    let _ = restore_prior_marketplace(&target.marketplace_path, journal);
    Err(ClaudeAdapterError::RecoveryRequired)
}

fn restore_prior_marketplace(
    path: &Path,
    journal: &ClaudeRegistrationJournalV1,
) -> Result<(), ClaudeAdapterError> {
    let current = read_marketplace(path)?;
    if current.present == journal.prior_marketplace_present
        && current.digest_sha256 == journal.prior_marketplace_sha256
    {
        return Ok(());
    }
    if current.digest_sha256 != journal.next_marketplace_sha256 {
        return Err(ClaudeAdapterError::RollbackFailed);
    }
    if journal.prior_marketplace_present {
        write_marketplace_document(
            path,
            &journal.prior_marketplace,
            &format!("{}-rollback", journal.transaction_id),
            true,
        )?;
    } else {
        fs::remove_file(path)
            .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
        sync_directory(path.parent().ok_or(ClaudeAdapterError::UnsafePath)?)?;
    }
    let restored = read_marketplace(path)?;
    if restored.present != journal.prior_marketplace_present
        || restored.digest_sha256 != journal.prior_marketplace_sha256
    {
        return Err(ClaudeAdapterError::RollbackFailed);
    }
    Ok(())
}

fn write_marketplace_document(
    path: &Path,
    document: &Value,
    transaction_id: &str,
    replace_existing: bool,
) -> Result<(), ClaudeAdapterError> {
    validate_marketplace_document(document)?;
    let mut bytes =
        serde_json::to_vec_pretty(document).map_err(|_| ClaudeAdapterError::MarketplaceInvalid)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(ClaudeAdapterError::DocumentTooLarge);
    }
    let parent = path.parent().ok_or(ClaudeAdapterError::UnsafePath)?;
    let staging = parent.join(format!(".marketplace.json.qiongli-stage-{transaction_id}"));
    if path_exists(&staging)? {
        return Err(ClaudeAdapterError::RecoveryRequired);
    }
    let mut file = create_private_new_file(&staging)?;
    if let Err(error) = write_sync_file(&mut file, &bytes) {
        drop(file);
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    drop(file);
    if let Err(error) = replace_file(&staging, path, replace_existing) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    sync_directory(parent)
}

fn persist_registration_state(
    root: &Path,
    state: &ClaudeRegistrationStateV1,
    transaction_id: &str,
) -> Result<(), ClaudeAdapterError> {
    let bytes = state.to_canonical_json()?;
    let destination = root.join(STATE_FILE_NAME);
    let staging = root.join(format!("{STATE_FILE_NAME}.stage-{transaction_id}"));
    if path_exists(&staging)? {
        return Err(ClaudeAdapterError::RecoveryRequired);
    }
    let mut file = create_private_new_file(&staging)?;
    if let Err(error) = write_sync_file(&mut file, &bytes) {
        drop(file);
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    drop(file);
    let replace_existing = path_exists(&destination)?;
    if let Err(error) = replace_file(&staging, &destination, replace_existing) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    sync_directory(root)?;
    let committed = read_private_file(&destination, MAX_DOCUMENT_BYTES)?;
    if committed != bytes {
        return Err(ClaudeAdapterError::RecoveryRequired);
    }
    Ok(())
}

fn load_registration_state(
    root: &Path,
) -> Result<Option<ClaudeRegistrationStateV1>, ClaudeAdapterError> {
    let path = root.join(STATE_FILE_NAME);
    if !path_exists(&path)? {
        return Ok(None);
    }
    ClaudeRegistrationStateV1::from_json(&read_private_file(&path, MAX_DOCUMENT_BYTES)?).map(Some)
}

fn persist_new_journal(
    root: &Path,
    journal: &ClaudeRegistrationJournalV1,
) -> Result<(), ClaudeAdapterError> {
    if journal.schema_version != CLAUDE_ADAPTER_SCHEMA_VERSION
        || !valid_identifier(&journal.transaction_id)
        || !valid_lower_hex(&journal.prior_marketplace_sha256, 64)
        || !valid_lower_hex(&journal.next_marketplace_sha256, 64)
        || journal
            .prior_state_sha256
            .as_ref()
            .is_some_and(|digest| !valid_lower_hex(digest, 64))
        || journal.started_at_unix > JCS_MAX_SAFE_INTEGER
        || document_digest(&journal.prior_marketplace)? != journal.prior_marketplace_sha256
    {
        return Err(ClaudeAdapterError::ReceiptInvalid);
    }
    let path = root.join(JOURNAL_FILE_NAME);
    if path_exists(&path)? {
        return Err(ClaudeAdapterError::RecoveryRequired);
    }
    let bytes = canonical_json(journal)?;
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(ClaudeAdapterError::DocumentTooLarge);
    }
    let mut file = create_private_new_file(&path)?;
    write_sync_file(&mut file, &bytes)?;
    drop(file);
    sync_directory(root)
}

fn finish_journal(root: &Path) -> Result<(), ClaudeAdapterError> {
    let path = root.join(JOURNAL_FILE_NAME);
    fs::remove_file(path).map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    sync_directory(root)
}

fn acquire_lock(root: &Path) -> Result<File, ClaudeAdapterError> {
    validate_directory(root, true)?;
    let path = root.join(LOCK_FILE_NAME);
    let file = open_or_create_private_lock(&path)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(ClaudeAdapterError::LockBusy),
        Err(TryLockError::Error(error)) => Err(ClaudeAdapterError::PersistenceFailed(error.kind())),
    }
}

fn prepare_marketplace_parent(path: &Path) -> Result<(), ClaudeAdapterError> {
    let metadata_root = path.parent().ok_or(ClaudeAdapterError::UnsafePath)?;
    let marketplace_root = metadata_root
        .parent()
        .ok_or(ClaudeAdapterError::UnsafePath)?;
    validate_directory(marketplace_root, false)?;
    ensure_private_directory(metadata_root)?;
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), ClaudeAdapterError> {
    match fs::symlink_metadata(path) {
        Ok(_) => validate_directory(path, false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create_private_directory(path)?;
            validate_directory(path, true)
        }
        Err(error) => Err(ClaudeAdapterError::PersistenceFailed(error.kind())),
    }
}

fn ensure_owner_private_directory(path: &Path) -> Result<(), ClaudeAdapterError> {
    match fs::symlink_metadata(path) {
        Ok(_) => validate_directory(path, true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create_private_directory(path)?;
            validate_directory(path, true)
        }
        Err(error) => Err(ClaudeAdapterError::PersistenceFailed(error.kind())),
    }
}

fn validate_optional_directory_chain(
    home: &Path,
    components: &[&str],
    private_final: bool,
) -> Result<(), ClaudeAdapterError> {
    let mut current = home.to_path_buf();
    let mut missing = false;
    for (index, component) in components.iter().enumerate() {
        current.push(component);
        if missing {
            if path_exists(&current)? {
                return Err(ClaudeAdapterError::UnsafePath);
            }
            continue;
        }
        if path_exists(&current)? {
            validate_directory(&current, private_final && index + 1 == components.len())?;
        } else {
            missing = true;
        }
    }
    Ok(())
}

fn validate_home(path: &Path) -> Result<(), ClaudeAdapterError> {
    if !path.is_absolute() || has_lexical_traversal(path) {
        return Err(ClaudeAdapterError::HomeUnavailable);
    }
    validate_directory(path, false).map_err(|_| ClaudeAdapterError::HomeUnavailable)
}

fn validate_directory(path: &Path, private: bool) -> Result<(), ClaudeAdapterError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            ClaudeAdapterError::UnsafePath
        } else {
            ClaudeAdapterError::PersistenceFailed(error.kind())
        }
    })?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(ClaudeAdapterError::UnsafePath);
    }
    validate_directory_security(path, &metadata, private)
}

#[cfg(unix)]
fn validate_directory_security(
    _path: &Path,
    metadata: &Metadata,
    private: bool,
) -> Result<(), ClaudeAdapterError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let mode = metadata.permissions().mode();
    if metadata.uid() != rustix::process::geteuid().as_raw()
        || mode & 0o022 != 0
        || (private && mode & 0o077 != 0)
    {
        return Err(ClaudeAdapterError::UnsafePath);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_directory_security(
    path: &Path,
    _metadata: &Metadata,
    private: bool,
) -> Result<(), ClaudeAdapterError> {
    if private {
        qiongli_windows_security::open_owner_only_directory(path)
            .map(|_| ())
            .map_err(|_| ClaudeAdapterError::UnsafePath)
    } else {
        qiongli_windows_security::open_directory_no_reparse(path)
            .map(|_| ())
            .map_err(|_| ClaudeAdapterError::UnsafePath)
    }
}

#[cfg(not(any(unix, windows)))]
fn validate_directory_security(
    _path: &Path,
    _metadata: &Metadata,
    _private: bool,
) -> Result<(), ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

fn read_bounded_config_file(path: &Path, max: u64) -> Result<Vec<u8>, ClaudeAdapterError> {
    let linked = validate_config_file(path)?;
    if linked.len() > max {
        return Err(ClaudeAdapterError::DocumentTooLarge);
    }
    let file =
        File::open(path).map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    let opened = file
        .metadata()
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    let before = Handle::from_path(path).map_err(|_| ClaudeAdapterError::UnsafePath)?;
    let cloned = file
        .try_clone()
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    let after = Handle::from_file(cloned).map_err(|_| ClaudeAdapterError::UnsafePath)?;
    if before != after || opened.len() != linked.len() {
        return Err(ClaudeAdapterError::UnsafePath);
    }
    read_bounded(file, max)
}

fn read_private_file(path: &Path, max: u64) -> Result<Vec<u8>, ClaudeAdapterError> {
    validate_private_file(path)?;
    read_bounded_config_file(path, max)
}

fn read_bounded(file: File, max: u64) -> Result<Vec<u8>, ClaudeAdapterError> {
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    if bytes.len() as u64 > max {
        return Err(ClaudeAdapterError::DocumentTooLarge);
    }
    Ok(bytes)
}

fn validate_config_file(path: &Path) -> Result<Metadata, ClaudeAdapterError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_file() {
        return Err(ClaudeAdapterError::UnsafePath);
    }
    validate_config_file_security(&metadata)?;
    Ok(metadata)
}

#[cfg(unix)]
fn validate_config_file_security(metadata: &Metadata) -> Result<(), ClaudeAdapterError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if metadata.nlink() != 1
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o022 != 0
    {
        return Err(ClaudeAdapterError::UnsafePath);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_config_file_security(_metadata: &Metadata) -> Result<(), ClaudeAdapterError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn validate_config_file_security(_metadata: &Metadata) -> Result<(), ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

#[cfg(unix)]
fn validate_private_file(path: &Path) -> Result<(), ClaudeAdapterError> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = validate_config_file(path)?;
    if metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(ClaudeAdapterError::ReceiptInvalid);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_private_file(path: &Path) -> Result<(), ClaudeAdapterError> {
    qiongli_windows_security::open_owner_only_file(path)
        .map(|_| ())
        .map_err(|_| ClaudeAdapterError::ReceiptInvalid)
}

#[cfg(not(any(unix, windows)))]
fn validate_private_file(_path: &Path) -> Result<(), ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), ClaudeAdapterError> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), ClaudeAdapterError> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|error| {
            ClaudeAdapterError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

#[cfg(unix)]
fn create_private_new_file(path: &Path) -> Result<File, ClaudeAdapterError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                ClaudeAdapterError::RecoveryRequired
            } else {
                ClaudeAdapterError::PersistenceFailed(error.kind())
            }
        })
}

#[cfg(windows)]
fn create_private_new_file(path: &Path) -> Result<File, ClaudeAdapterError> {
    qiongli_windows_security::create_owner_only_new_file(path).map_err(|error| {
        if error.io_kind() == Some(io::ErrorKind::AlreadyExists) {
            ClaudeAdapterError::RecoveryRequired
        } else {
            ClaudeAdapterError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        }
    })
}

#[cfg(not(any(unix, windows)))]
fn create_private_new_file(_path: &Path) -> Result<File, ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

#[cfg(unix)]
fn open_or_create_private_lock(path: &Path) -> Result<File, ClaudeAdapterError> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))?;
    validate_private_file(path)?;
    Ok(file)
}

#[cfg(windows)]
fn open_or_create_private_lock(path: &Path) -> Result<File, ClaudeAdapterError> {
    qiongli_windows_security::open_or_create_owner_only_lock(path).map_err(|error| {
        ClaudeAdapterError::PersistenceFailed(
            error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
        )
    })
}

#[cfg(not(any(unix, windows)))]
fn open_or_create_private_lock(_path: &Path) -> Result<File, ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

fn write_sync_file(file: &mut File, bytes: &[u8]) -> Result<(), ClaudeAdapterError> {
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))
}

#[cfg(unix)]
fn replace_file(
    source: &Path,
    destination: &Path,
    _replace_existing: bool,
) -> Result<(), ClaudeAdapterError> {
    fs::rename(source, destination)
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn replace_file(
    source: &Path,
    destination: &Path,
    replace_existing: bool,
) -> Result<(), ClaudeAdapterError> {
    qiongli_windows_security::move_file_write_through(source, destination, replace_existing)
        .map_err(|error| {
            ClaudeAdapterError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn replace_file(
    _source: &Path,
    _destination: &Path,
    _replace_existing: bool,
) -> Result<(), ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ClaudeAdapterError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| ClaudeAdapterError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), ClaudeAdapterError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn sync_directory(_path: &Path) -> Result<(), ClaudeAdapterError> {
    Err(ClaudeAdapterError::UnsupportedPlatform)
}

fn path_exists(path: &Path) -> Result<bool, ClaudeAdapterError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(ClaudeAdapterError::PersistenceFailed(error.kind())),
    }
}

fn marketplace_path(home: &Path) -> PathBuf {
    MARKETPLACE_RELATIVE_PATH
        .iter()
        .fold(home.to_path_buf(), |path, component| path.join(component))
}

fn state_root_path(home: &Path) -> PathBuf {
    STATE_ROOT_RELATIVE_PATH
        .iter()
        .filter(|component| !component.is_empty())
        .fold(home.to_path_buf(), |path, component| path.join(component))
}

fn plugin_source_path(home: &Path) -> PathBuf {
    PLUGIN_SOURCE_RELATIVE_PATH
        .iter()
        .fold(marketplace_root_path(home), |path, component| {
            path.join(component)
        })
}

fn marketplace_root_path(home: &Path) -> PathBuf {
    MARKETPLACE_ROOT_RELATIVE_PATH
        .iter()
        .fold(home.to_path_buf(), |path, component| path.join(component))
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ClaudeAdapterError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| ClaudeAdapterError::ReceiptInvalid)
}

fn document_digest(value: &Value) -> Result<String, ClaudeAdapterError> {
    value_digest(value)
}

fn value_digest(value: &Value) -> Result<String, ClaudeAdapterError> {
    Ok(sha256_hex(&canonical_json(value)?))
}

fn sha256_hex(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn next_generation(state: Option<&ClaudeRegistrationStateV1>) -> Result<u64, ClaudeAdapterError> {
    state
        .map_or(Some(1), |state| state.generation.checked_add(1))
        .filter(|generation| *generation <= JCS_MAX_SAFE_INTEGER)
        .ok_or(ClaudeAdapterError::ReceiptInvalid)
}

fn transaction_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
    format!("claude-{}-{nanos}-{counter}", std::process::id())
}

fn lifecycle_disposition(
    kind: ClaudeRegistrationLifecycleKind,
) -> ClaudeRegistrationLifecycleDisposition {
    match kind {
        ClaudeRegistrationLifecycleKind::Removed => ClaudeRegistrationLifecycleDisposition::Removed,
        ClaudeRegistrationLifecycleKind::RolledBack => {
            ClaudeRegistrationLifecycleDisposition::RolledBack
        }
    }
}

fn already_lifecycle_disposition(
    kind: ClaudeRegistrationLifecycleKind,
) -> ClaudeRegistrationLifecycleDisposition {
    match kind {
        ClaudeRegistrationLifecycleKind::Removed => {
            ClaudeRegistrationLifecycleDisposition::AlreadyRemoved
        }
        ClaudeRegistrationLifecycleKind::RolledBack => {
            ClaudeRegistrationLifecycleDisposition::AlreadyRolledBack
        }
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_lower_hex(value: &str, len: usize) -> bool {
    value.len() == len
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
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )
    })
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use ed25519_dalek::{Signer, SigningKey};
    use qiongli_content::{
        BuiltResourcePack, CompatibleProduct, ResourcePackBuildMetadata, build_resource_pack,
        collect_canonical_sources, load_resource_pack,
    };

    use super::*;
    use crate::{
        Architecture, GrantMode, GrantSignatureV1, GrantVerificationContext, InstallerKind,
        IntegrationScope, LaunchGrantV1, OperatingSystem, ReleaseChannel, SignatureAlgorithm,
        SignedLaunchGrantV1, TrustedPublicKey, approve_claude_plugin_bundle_target,
        approve_install_plan, compose_claude_plugin_bundle, launch_grant_signing_bytes,
    };

    const NOW: u64 = 1_750_100_000;
    const PLUGIN_MANIFEST_RELATIVE_PATH: &str = ".claude-plugin/plugin.json";
    const TEST_BINARY_BYTES: &[u8] = b"qiongli native Claude fixture\n";
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
    static PACK: OnceLock<BuiltResourcePack> = OnceLock::new();

    struct Fixture {
        container: PathBuf,
        home: PathBuf,
    }

    impl Fixture {
        fn empty(name: &str) -> Self {
            let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-platform-claude-tests");
            fs::create_dir_all(&base).expect("Claude test base must exist");
            let requested = base.join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&requested).expect("Claude test container must exist");
            let container = fs::canonicalize(requested).expect("test container must canonicalize");
            let home = container.join("home");
            create_private_test_directory(&home);
            Self { container, home }
        }

        fn with_source(name: &str) -> Self {
            let fixture = Self::empty(name);
            let qiongli = fixture.home.join(".qiongli");
            create_private_test_directory(&qiongli);
            let plugins = qiongli.join("plugins");
            create_private_test_directory(&plugins);
            let claude = plugins.join("claude-code");
            create_private_test_directory(&claude);
            let marketplace = claude.join("qiongli-local");
            create_private_test_directory(&marketplace);
            let marketplace_plugins = marketplace.join("plugins");
            create_private_test_directory(&marketplace_plugins);
            let v2 = qiongli.join("v2");
            create_private_test_directory(&v2);
            let integrations = v2.join("integrations");
            create_private_test_directory(&integrations);
            create_private_test_directory(&integrations.join("claude-code"));
            let source = marketplace_plugins.join("qiongli-next");
            let binary = fixture.container.join("qiongli-claude-fixture-binary");
            fs::write(&binary, TEST_BINARY_BYTES).expect("Claude test binary must write");
            set_test_executable_mode(&binary);
            let artifact = test_artifact();
            let (verified_grant, _) = verified_test_grant(&artifact);
            let target = approve_claude_plugin_bundle_target(&source)
                .expect("Claude test bundle target must approve");
            compose_claude_plugin_bundle(test_pack(), &verified_grant, &binary, &target)
                .expect("Claude test plugin bundle must compose");
            fixture
        }

        fn marketplace(&self) -> Value {
            read_marketplace(&marketplace_path(&self.home))
                .expect("test marketplace must read")
                .document
        }

        fn write_marketplace(&self, document: &Value) {
            let path = marketplace_path(&self.home);
            prepare_marketplace_parent(&path).expect("marketplace parent must prepare");
            let replace = path_exists(&path).expect("marketplace path must inspect");
            write_marketplace_document(&path, document, "test-write", replace)
                .expect("test marketplace must write");
        }

        fn plan(
            &self,
        ) -> (
            VerifiedInstallPlan,
            ApprovedInstallPlan,
            ClaudeRegistrationExecutor,
        ) {
            let artifact = test_artifact();
            let binary_digest = test_binary_digest();
            let (verified_grant, trusted) = verified_test_grant(&artifact);
            let context = GrantVerificationContext {
                now_unix: NOW,
                minimum_generation: 9,
                expected_artifact: &artifact,
                binary_sha256: &binary_digest,
                resource_pack_sha256: test_pack().pack_sha256(),
                requested_mode: GrantMode::FullMcp,
                requested_scope: IntegrationScope::ClaudeCodeLocal,
            };
            let target = discover_claude_user(&self.home).expect("Claude target must discover");
            let executor = ClaudeRegistrationExecutor::new(target.clone());
            let preview = preview_claude_registration(
                &target,
                InstallPlanMetadataV1 {
                    plan_id: "r3c-claude-test".to_string(),
                    created_at_unix: NOW,
                    expires_at_unix: NOW + 600,
                },
                &verified_grant,
            )
            .expect("Claude plan must preview");
            let verified = preview
                .plan
                .verify(std::slice::from_ref(&trusted), &context)
                .expect("Claude plan must verify");
            let approval = approve_install_plan(&verified, &EXACT_APPROVALS, NOW)
                .expect("Claude plan must approve");
            (verified, approval, executor)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.container);
        }
    }

    fn test_artifact() -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.1".to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::current().expect("test OS must be supported"),
            arch: Architecture::current().expect("test architecture must be supported"),
            installer_kind: InstallerKind::PluginBundle,
        }
    }

    fn test_binary_digest() -> String {
        sha256_hex(TEST_BINARY_BYTES)
    }

    fn verified_test_grant(
        artifact: &ArtifactIdentityV1,
    ) -> (VerifiedLaunchGrant, TrustedPublicKey) {
        let binary_digest = test_binary_digest();
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: 9,
            artifact: artifact.clone(),
            binary_sha256: binary_digest.clone(),
            resource_pack_sha256: test_pack().pack_sha256().to_string(),
            allowed_modes: vec![GrantMode::LiteMcp, GrantMode::FullMcp],
            integration_scopes: vec![IntegrationScope::ClaudeCodeLocal],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        };
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
        let signed = SignedLaunchGrantV1 {
            grant,
            signature: GrantSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "claude-test-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        };
        let trusted =
            TrustedPublicKey::new("claude-test-key", signing_key.verifying_key().to_bytes())
                .unwrap();
        let context = GrantVerificationContext {
            now_unix: NOW,
            minimum_generation: 9,
            expected_artifact: artifact,
            binary_sha256: &binary_digest,
            resource_pack_sha256: test_pack().pack_sha256(),
            requested_mode: GrantMode::FullMcp,
            requested_scope: IntegrationScope::ClaudeCodeLocal,
        };
        let verified = signed
            .verify(std::slice::from_ref(&trusted), &context)
            .expect("Claude test grant must verify");
        (verified, trusted)
    }

    #[cfg(unix)]
    fn set_test_executable_mode(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("Claude test binary must be executable");
    }

    #[cfg(not(unix))]
    fn set_test_executable_mode(_path: &Path) {}

    fn test_pack() -> &'static qiongli_content::LoadedResourcePack<'static> {
        static LOADED: OnceLock<qiongli_content::LoadedResourcePack<'static>> = OnceLock::new();
        let built = PACK.get_or_init(|| {
            const DIRECTORIES: [&str; 12] = [
                ".claude-plugin",
                ".codex-plugin",
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
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-platform-claude-pack-source");
            let _ = fs::remove_dir_all(&source);
            fs::create_dir_all(&source).expect("Claude pack source must create");
            for directory in DIRECTORIES {
                fs::create_dir(source.join(directory)).expect("canonical directory must create");
                match directory {
                    ".claude-plugin" => fs::write(
                        source.join(PLUGIN_MANIFEST_RELATIVE_PATH),
                        br#"{"name":"qiongli","version":"2.0.0-alpha.1","skills":"./"}"#,
                    )
                    .expect("Claude manifest must write"),
                    ".codex-plugin" => fs::write(
                        source.join(".codex-plugin/plugin.json"),
                        br#"{"name":"qiongli","version":"2.0.0-alpha.1","skills":"./"}"#,
                    )
                    .expect("Codex manifest must write"),
                    "workflow" => fs::write(
                        source.join("workflow/SKILL.md"),
                        b"---\nname: qiongli\ndescription: test\n---\n",
                    )
                    .expect("Claude skill must write"),
                    _ => fs::write(
                        source.join(directory).join("entry.txt"),
                        directory.as_bytes(),
                    )
                    .expect("canonical entry must write"),
                }
            }
            fs::write(source.join("skills-core.md"), b"core\n").unwrap();
            fs::write(source.join("skills-summary.md"), b"summary\n").unwrap();
            let resources = collect_canonical_sources(&source)
                .expect("synthetic content must collect for Claude tests");
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
            .inspect(|_| {
                let _ = fs::remove_dir_all(source);
            })
            .expect("Claude test pack must build")
        });
        LOADED.get_or_init(|| {
            load_resource_pack(built.core_bytes(), built.pack_sha256())
                .expect("Claude test pack must load")
        })
    }

    #[test]
    fn discovery_is_read_only_and_redacted() {
        let fixture = Fixture::empty("discovery");
        let target = discover_claude_user(&fixture.home).expect("empty target must discover");
        assert_eq!(
            target.summary.skills_plugin,
            ClaudeSkillsPluginState::Missing
        );
        assert_eq!(target.summary.source, ClaudeSourceState::Missing);
        assert_eq!(target.summary.marketplace, ClaudeMarketplaceState::Missing);
        assert_eq!(target.summary.registration, ClaudeRegistrationState::Absent);
        assert!(!fixture.home.join(".qiongli").exists());
        let rendered = format!("{target:?}");
        assert!(!rendered.contains(fixture.home.to_string_lossy().as_ref()));
        let summary = serde_json::to_string(target.summary()).unwrap();
        assert!(!summary.contains(fixture.home.to_string_lossy().as_ref()));
    }

    #[test]
    fn skills_directory_discovery_requires_an_exact_verified_bundle() {
        let fixture = Fixture::empty("skills-directory");
        let claude_config = fixture.home.join(".claude");
        create_private_test_directory(&claude_config);
        let skills = claude_config.join("skills");
        create_private_test_directory(&skills);
        let plugin = skills.join("qiongli-next");
        let binary = fixture.container.join("qiongli-claude-skills-binary");
        fs::write(&binary, TEST_BINARY_BYTES).unwrap();
        set_test_executable_mode(&binary);
        let artifact = test_artifact();
        let (grant, _) = verified_test_grant(&artifact);
        let target = approve_claude_plugin_bundle_target(&plugin).unwrap();
        compose_claude_plugin_bundle(test_pack(), &grant, &binary, &target).unwrap();

        let discovered = discover_claude_user_with_config(&fixture.home, &claude_config).unwrap();
        assert_eq!(
            discovered.summary.skills_plugin,
            ClaudeSkillsPluginState::Ready
        );
        fs::write(plugin.join(PLUGIN_MANIFEST_RELATIVE_PATH), b"{}").unwrap();
        let drifted = discover_claude_user_with_config(&fixture.home, &claude_config).unwrap();
        assert_eq!(
            drifted.summary.skills_plugin,
            ClaudeSkillsPluginState::Conflict
        );
    }

    #[test]
    fn preview_apply_verify_replay_and_remove_preserve_unrelated_entries() {
        let fixture = Fixture::with_source("lifecycle");
        fixture.write_marketplace(&json!({
            "name": "qiongli-local",
            "description": "Custom local marketplace.",
            "owner": {"name": "Qiongli", "team": "research"},
            "custom": {"keep": true},
            "plugins": [{
                "name": "other",
                "source": "./plugins/other"
            }]
        }));
        let (plan, approval, executor) = fixture.plan();
        let applied = executor.apply(&plan, &approval, NOW + 1).unwrap();
        assert_eq!(
            applied.disposition,
            ClaudeRegistrationDisposition::Registered
        );
        assert!(!applied.cleanup_required);
        assert_eq!(executor.verify().unwrap().receipt, applied.receipt);
        let replay = executor.apply(&plan, &approval, NOW + 2).unwrap();
        assert_eq!(
            replay.disposition,
            ClaudeRegistrationDisposition::AlreadyRegistered
        );
        let document = fixture.marketplace();
        assert_eq!(document["custom"]["keep"], true);
        assert_eq!(document["owner"]["team"], "research");
        assert_eq!(document["plugins"].as_array().unwrap().len(), 2);

        let removed = executor.remove(NOW + 3).unwrap();
        assert_eq!(
            removed.disposition,
            ClaudeRegistrationLifecycleDisposition::Removed
        );
        let after = fixture.marketplace();
        assert_eq!(after["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(after["plugins"][0]["name"], "other");
        assert_eq!(after["custom"]["keep"], true);
        assert_eq!(
            executor.remove(NOW + 4).unwrap().disposition,
            ClaudeRegistrationLifecycleDisposition::AlreadyRemoved
        );
    }

    #[test]
    fn repair_restores_only_a_receipt_owned_missing_entry() {
        let fixture = Fixture::with_source("repair");
        let (plan, approval, executor) = fixture.plan();
        executor.apply(&plan, &approval, NOW + 1).unwrap();
        let without = remove_marketplace_entry(&fixture.marketplace()).unwrap();
        fixture.write_marketplace(&without);
        assert_eq!(
            discover_claude_user(&fixture.home)
                .unwrap()
                .summary
                .registration,
            ClaudeRegistrationState::Drifted
        );
        let repaired = executor.repair(&plan, &approval, NOW + 2).unwrap();
        assert_eq!(
            repaired.disposition,
            ClaudeRegistrationDisposition::Repaired
        );
        assert!(executor.verify().is_ok());
        assert_eq!(
            executor.rollback(NOW + 3).unwrap().disposition,
            ClaudeRegistrationLifecycleDisposition::RolledBack
        );
    }

    #[test]
    fn conflict_drift_and_recovery_states_fail_closed() {
        let conflict = Fixture::with_source("conflict");
        conflict.write_marketplace(&json!({
            "name": "qiongli-local",
            "owner": {"name": "Qiongli"},
            "plugins": [{
                "name": "qiongli-next",
                "source": "./someone-else"
            }]
        }));
        let target = discover_claude_user(&conflict.home).unwrap();
        assert_eq!(
            target.summary.registration,
            ClaudeRegistrationState::Conflict
        );

        let drift = Fixture::with_source("drift");
        let (plan, approval, executor) = drift.plan();
        executor.apply(&plan, &approval, NOW + 1).unwrap();
        let mut document = drift.marketplace();
        let entry = document["plugins"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["name"] == "qiongli-next")
            .unwrap();
        entry["source"] = Value::String("./changed".to_string());
        drift.write_marketplace(&document);
        assert_eq!(
            executor.verify().unwrap_err(),
            ClaudeAdapterError::RegistrationDrift
        );

        let recovery = Fixture::with_source("recovery");
        let journal_path = state_root_path(&recovery.home).join(JOURNAL_FILE_NAME);
        let mut journal = create_private_new_file(&journal_path).unwrap();
        write_sync_file(&mut journal, b"pending").unwrap();
        drop(journal);
        assert_eq!(
            discover_claude_user(&recovery.home)
                .unwrap()
                .summary
                .registration,
            ClaudeRegistrationState::RecoveryRequired
        );
    }

    #[test]
    fn malformed_marketplace_and_incomplete_approval_are_rejected() {
        let fixture = Fixture::with_source("invalid");
        let path = marketplace_path(&fixture.home);
        prepare_marketplace_parent(&path).unwrap();
        let mut file = create_private_new_file(&path).unwrap();
        write_sync_file(&mut file, b"{not-json").unwrap();
        drop(file);
        assert_eq!(
            discover_claude_user(&fixture.home).unwrap_err(),
            ClaudeAdapterError::MarketplaceInvalid
        );

        fs::remove_file(path).unwrap();
        let artifact = ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.1".to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::current().unwrap(),
            arch: Architecture::current().unwrap(),
            installer_kind: InstallerKind::PortableArchive,
        };
        assert!(artifact.validate().is_ok());
        let (plan, _, _) = fixture.plan();
        assert_eq!(
            approve_install_plan(
                &plan,
                &[
                    ApprovalRequirement::FilesystemWrite,
                    ApprovalRequirement::ClientConfigChange
                ],
                NOW
            )
            .unwrap_err(),
            crate::TransactionError::InvalidApproval
        );
    }

    #[test]
    fn post_activation_failure_restores_prior_document_or_retains_recovery_evidence() {
        let restored = Fixture::with_source("rollback-restored");
        let target = discover_claude_user(&restored.home).unwrap();
        let next = insert_marketplace_entry(&target.marketplace.document).unwrap();
        let journal = ClaudeRegistrationJournalV1 {
            schema_version: CLAUDE_ADAPTER_SCHEMA_VERSION,
            transaction_id: "rollback-restored".to_string(),
            kind: ClaudeJournalKind::Apply,
            prior_marketplace_present: target.marketplace.present,
            prior_marketplace: target.marketplace.document.clone(),
            prior_marketplace_sha256: target.marketplace.digest_sha256.clone(),
            next_marketplace_sha256: document_digest(&next).unwrap(),
            prior_state_sha256: None,
            started_at_unix: NOW,
        };
        persist_new_journal(&target.state_root, &journal).unwrap();
        activate_marketplace(
            &target.marketplace_path,
            &target.marketplace,
            &next,
            &journal.transaction_id,
        )
        .unwrap();
        assert_eq!(
            rollback_after_activation::<()>(
                &target,
                &journal,
                ClaudeAdapterError::PersistenceFailed(io::ErrorKind::PermissionDenied)
            )
            .unwrap_err(),
            ClaudeAdapterError::PersistenceFailed(io::ErrorKind::PermissionDenied)
        );
        assert!(!marketplace_path(&restored.home).exists());
        assert!(!target.state_root.join(JOURNAL_FILE_NAME).exists());

        let ambiguous = Fixture::with_source("rollback-ambiguous");
        let target = discover_claude_user(&ambiguous.home).unwrap();
        let next = insert_marketplace_entry(&target.marketplace.document).unwrap();
        let journal = ClaudeRegistrationJournalV1 {
            schema_version: CLAUDE_ADAPTER_SCHEMA_VERSION,
            transaction_id: "rollback-ambiguous".to_string(),
            kind: ClaudeJournalKind::Apply,
            prior_marketplace_present: target.marketplace.present,
            prior_marketplace: target.marketplace.document.clone(),
            prior_marketplace_sha256: target.marketplace.digest_sha256.clone(),
            next_marketplace_sha256: document_digest(&next).unwrap(),
            prior_state_sha256: None,
            started_at_unix: NOW,
        };
        persist_new_journal(&target.state_root, &journal).unwrap();
        activate_marketplace(
            &target.marketplace_path,
            &target.marketplace,
            &next,
            &journal.transaction_id,
        )
        .unwrap();
        let mut concurrent = next;
        concurrent["concurrent"] = Value::Bool(true);
        ambiguous.write_marketplace(&concurrent);
        assert_eq!(
            rollback_after_activation::<()>(
                &target,
                &journal,
                ClaudeAdapterError::PersistenceFailed(io::ErrorKind::PermissionDenied)
            )
            .unwrap_err(),
            ClaudeAdapterError::RecoveryRequired
        );
        assert_eq!(ambiguous.marketplace()["concurrent"], true);
        assert!(target.state_root.join(JOURNAL_FILE_NAME).is_file());

        let state_ambiguous = Fixture::with_source("state-ambiguous");
        let target = discover_claude_user(&state_ambiguous.home).unwrap();
        let next = insert_marketplace_entry(&target.marketplace.document).unwrap();
        let journal = ClaudeRegistrationJournalV1 {
            schema_version: CLAUDE_ADAPTER_SCHEMA_VERSION,
            transaction_id: "state-ambiguous".to_string(),
            kind: ClaudeJournalKind::Apply,
            prior_marketplace_present: target.marketplace.present,
            prior_marketplace: target.marketplace.document.clone(),
            prior_marketplace_sha256: target.marketplace.digest_sha256.clone(),
            next_marketplace_sha256: document_digest(&next).unwrap(),
            prior_state_sha256: None,
            started_at_unix: NOW,
        };
        persist_new_journal(&target.state_root, &journal).unwrap();
        activate_marketplace(
            &target.marketplace_path,
            &target.marketplace,
            &next,
            &journal.transaction_id,
        )
        .unwrap();
        assert_eq!(
            recover_after_state_failure::<()>(
                &target,
                &journal,
                ClaudeAdapterError::PersistenceFailed(io::ErrorKind::PermissionDenied)
            )
            .unwrap_err(),
            ClaudeAdapterError::RecoveryRequired
        );
        assert!(!marketplace_path(&state_ambiguous.home).exists());
        assert!(target.state_root.join(JOURNAL_FILE_NAME).is_file());
    }

    #[test]
    fn source_drift_and_oversized_marketplace_are_rejected() {
        let drift = Fixture::with_source("source-drift");
        fs::write(
            plugin_source_path(&drift.home).join(PLUGIN_MANIFEST_RELATIVE_PATH),
            b"{}",
        )
        .unwrap();
        assert_eq!(
            discover_claude_user(&drift.home).unwrap_err(),
            ClaudeAdapterError::SourceInvalid
        );

        let oversized = Fixture::with_source("oversized");
        let path = marketplace_path(&oversized.home);
        prepare_marketplace_parent(&path).unwrap();
        let file = create_private_new_file(&path).unwrap();
        file.set_len(MAX_DOCUMENT_BYTES + 1).unwrap();
        drop(file);
        assert_eq!(
            discover_claude_user(&oversized.home).unwrap_err(),
            ClaudeAdapterError::DocumentTooLarge
        );
    }

    fn encode_hex(bytes: &[u8]) -> String {
        lower_hex(bytes)
    }

    #[cfg(unix)]
    fn create_private_test_directory(path: &Path) {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .expect("private test directory must create");
    }

    #[cfg(windows)]
    fn create_private_test_directory(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path)
            .expect("private test directory must create");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_private_test_directory(_path: &Path) {
        panic!("Claude adapter tests require Unix or Windows");
    }
}
