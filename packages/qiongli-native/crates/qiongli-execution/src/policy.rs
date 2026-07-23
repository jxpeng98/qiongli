use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{ExecutionError, RunId, ToolCallId, ToolId};

const MAX_POLICY_REVISION: u64 = 9_007_199_254_740_991;
const MAX_PURPOSE_BYTES: usize = 512;
const MAX_ARGUMENT_BYTES: usize = 1024 * 1024;
const MAX_DECLARED_ARTIFACTS: usize = 64;
const MAX_RELATIVE_PATH_BYTES: usize = 512;
const MAX_APPROVAL_LIFETIME_SECONDS: u64 = 15 * 60;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionProfile {
    Lite,
    Full,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolClass {
    Read,
    ProjectWrite,
    OutOfProjectWrite,
    Shell,
    Process,
    Network,
    Mcp,
    Secret,
    Service,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolExecutionKind {
    InProcessReadOnly,
    ReservedChild,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolRegistrationV1 {
    pub schema_version: u32,
    pub tool_id: ToolId,
    pub class: ToolClass,
    pub execution: ToolExecutionKind,
    pub read_only: bool,
    pub requires_project: bool,
    pub requires_approval: bool,
    pub allows_network: bool,
}

impl ToolRegistrationV1 {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        if self.schema_version != 1
            || (self.execution == ToolExecutionKind::InProcessReadOnly
                && (!self.read_only
                    || self.allows_network
                    || !matches!(self.class, ToolClass::Read | ToolClass::Service)))
            || (self.read_only
                && matches!(
                    self.class,
                    ToolClass::ProjectWrite
                        | ToolClass::OutOfProjectWrite
                        | ToolClass::Shell
                        | ToolClass::Process
                        | ToolClass::Secret
                ))
            || (self.allows_network && self.class != ToolClass::Network)
        {
            return Err(ExecutionError::InvalidToolRegistration);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecutionLimitsV1 {
    pub wall_clock_seconds: u64,
    pub model_turns: u32,
    pub tool_calls: u32,
    pub processes: u32,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub network_requests: u32,
    pub artifacts: u32,
}

impl ExecutionLimitsV1 {
    #[must_use]
    pub const fn bounded_default() -> Self {
        Self {
            wall_clock_seconds: 20 * 60,
            model_turns: 32,
            tool_calls: 128,
            processes: 8,
            input_bytes: 16 * 1024 * 1024,
            output_bytes: 16 * 1024 * 1024,
            network_requests: 64,
            artifacts: 64,
        }
    }

    fn validate(&self) -> Result<(), ExecutionError> {
        if self.wall_clock_seconds == 0
            || self.wall_clock_seconds > 24 * 60 * 60
            || self.model_turns == 0
            || self.model_turns > 1_024
            || self.tool_calls == 0
            || self.tool_calls > 4_096
            || self.processes > 128
            || self.input_bytes == 0
            || self.output_bytes == 0
            || self.input_bytes > 1024 * 1024 * 1024
            || self.output_bytes > 1024 * 1024 * 1024
            || self.network_requests > 4_096
            || self.artifacts > 4_096
        {
            return Err(ExecutionError::InvalidPolicy);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecutionUsageV1 {
    pub elapsed_seconds: u64,
    pub model_turns: u32,
    pub tool_calls: u32,
    pub processes: u32,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub network_requests: u32,
    pub artifacts: u32,
}

impl ExecutionUsageV1 {
    fn exceeds(&self, limits: &ExecutionLimitsV1) -> bool {
        self.elapsed_seconds >= limits.wall_clock_seconds
            || self.model_turns >= limits.model_turns
            || self.tool_calls >= limits.tool_calls
            || self.processes > limits.processes
            || self.input_bytes > limits.input_bytes
            || self.output_bytes > limits.output_bytes
            || self.network_requests > limits.network_requests
            || self.artifacts > limits.artifacts
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RedactionPolicyV1 {
    pub maximum_result_bytes: u64,
    pub redact_absolute_paths: bool,
    pub redact_authorization_headers: bool,
    pub redact_secret_values: bool,
    pub redact_unrestricted_model_text: bool,
}

impl RedactionPolicyV1 {
    #[must_use]
    pub const fn strict_default() -> Self {
        Self {
            maximum_result_bytes: 1024 * 1024,
            redact_absolute_paths: true,
            redact_authorization_headers: true,
            redact_secret_values: true,
            redact_unrestricted_model_text: true,
        }
    }

    fn validate(&self) -> Result<(), ExecutionError> {
        if self.maximum_result_bytes == 0
            || self.maximum_result_bytes > 16 * 1024 * 1024
            || !self.redact_authorization_headers
            || !self.redact_secret_values
        {
            return Err(ExecutionError::InvalidPolicy);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ProjectExecutionScope {
    pub project_id: ProjectId,
    pub canonical_root: PathBuf,
    pub semantic_revision: u64,
}

impl ProjectExecutionScope {
    pub fn new(
        project_id: ProjectId,
        canonical_root: PathBuf,
        semantic_revision: u64,
    ) -> Result<Self, ExecutionError> {
        if !canonical_root.is_absolute()
            || semantic_revision == 0
            || semantic_revision > MAX_POLICY_REVISION
        {
            return Err(ExecutionError::InvalidPolicy);
        }
        Ok(Self {
            project_id,
            canonical_root,
            semantic_revision,
        })
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyToolRequestV1 {
    pub schema_version: u32,
    pub run_id: RunId,
    pub call_id: ToolCallId,
    pub purpose: String,
    pub tool_id: ToolId,
    pub arguments: Value,
    pub project_id: Option<ProjectId>,
    pub expected_project_revision: Option<u64>,
    pub declared_artifacts: Vec<String>,
}

impl PolicyToolRequestV1 {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        if self.schema_version != 1
            || self.purpose.trim().is_empty()
            || self.purpose.len() > MAX_PURPOSE_BYTES
            || !self.arguments.is_object()
            || serde_json::to_vec(&self.arguments)
                .map_or(true, |bytes| bytes.len() > MAX_ARGUMENT_BYTES)
            || self.declared_artifacts.len() > MAX_DECLARED_ARTIFACTS
            || self
                .declared_artifacts
                .iter()
                .any(|path| !valid_relative_artifact(path))
            || self.project_id.is_some() != self.expected_project_revision.is_some()
            || self
                .expected_project_revision
                .is_some_and(|revision| revision == 0 || revision > MAX_POLICY_REVISION)
        {
            return Err(ExecutionError::InvalidToolRequest);
        }
        Ok(())
    }

    pub fn normalized_digest(&self) -> Result<String, ExecutionError> {
        self.validate()?;
        canonical_sha256(self).map_err(|_| ExecutionError::InvalidToolRequest)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalActor {
    User,
    Administrator,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalGrantV1 {
    pub schema_version: u32,
    pub request_digest: String,
    pub run_id: RunId,
    pub tool_id: ToolId,
    pub policy_revision: u64,
    pub actor: ApprovalActor,
    pub issued_at_unix: u64,
    pub expires_at_unix: u64,
}

impl ApprovalGrantV1 {
    pub fn for_request(
        request: &PolicyToolRequestV1,
        policy_revision: u64,
        actor: ApprovalActor,
        issued_at_unix: u64,
        expires_at_unix: u64,
    ) -> Result<Self, ExecutionError> {
        let grant = Self {
            schema_version: 1,
            request_digest: request.normalized_digest()?,
            run_id: request.run_id.clone(),
            tool_id: request.tool_id.clone(),
            policy_revision,
            actor,
            issued_at_unix,
            expires_at_unix,
        };
        grant.validate_shape()?;
        Ok(grant)
    }

    fn validate_shape(&self) -> Result<(), ExecutionError> {
        if self.schema_version != 1
            || !valid_lower_hex(&self.request_digest, 64)
            || self.policy_revision == 0
            || self.policy_revision > MAX_POLICY_REVISION
            || self.issued_at_unix >= self.expires_at_unix
            || self.expires_at_unix - self.issued_at_unix > MAX_APPROVAL_LIFETIME_SECONDS
        {
            return Err(ExecutionError::InvalidApproval);
        }
        Ok(())
    }

    fn accepts(
        &self,
        request: &PolicyToolRequestV1,
        policy_revision: u64,
        now_unix: u64,
    ) -> Result<bool, ExecutionError> {
        self.validate_shape()?;
        Ok(self.request_digest == request.normalized_digest()?
            && self.run_id == request.run_id
            && self.tool_id == request.tool_id
            && self.policy_revision == policy_revision
            && now_unix >= self.issued_at_unix
            && now_unix < self.expires_at_unix)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyOutcome {
    Allow,
    Deny,
    ApprovalRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyReasonCode {
    Allowed,
    ToolNotAllowlisted,
    LiteProfileForbidden,
    ClassAuthorityUnavailable,
    ProjectScopeRequired,
    ProjectScopeMismatch,
    ProjectRevisionMismatch,
    LimitExhausted,
    ApprovalMissing,
    ApprovalInvalid,
    RequestInvalid,
}

impl PolicyReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "tool-policy-allowed",
            Self::ToolNotAllowlisted => "tool-not-allowlisted",
            Self::LiteProfileForbidden => "tool-forbidden-in-lite-profile",
            Self::ClassAuthorityUnavailable => "tool-class-authority-unavailable",
            Self::ProjectScopeRequired => "tool-project-scope-required",
            Self::ProjectScopeMismatch => "tool-project-scope-mismatch",
            Self::ProjectRevisionMismatch => "tool-project-revision-mismatch",
            Self::LimitExhausted => "tool-execution-limit-exhausted",
            Self::ApprovalMissing => "tool-approval-required",
            Self::ApprovalInvalid => "tool-approval-invalid",
            Self::RequestInvalid => "tool-request-invalid",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyDecisionV1 {
    pub schema_version: u32,
    pub outcome: PolicyOutcome,
    pub reason: PolicyReasonCode,
    pub reason_code: String,
    pub request_digest: Option<String>,
    pub policy_revision: u64,
    pub decision_digest: String,
}

impl PolicyDecisionV1 {
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self.outcome, PolicyOutcome::Allow)
    }

    pub fn validate(&self) -> Result<(), ExecutionError> {
        if self.schema_version != 1
            || self.policy_revision == 0
            || self.reason_code != self.reason.as_str()
            || self
                .request_digest
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
            || self.decision_digest
                != decision_digest(
                    self.outcome,
                    self.reason,
                    self.request_digest.as_deref(),
                    self.policy_revision,
                )
        {
            return Err(ExecutionError::PolicyDenied);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct AgentExecutionPolicy {
    revision: u64,
    profile: ExecutionProfile,
    allowed_tools: BTreeSet<ToolId>,
    project: Option<ProjectExecutionScope>,
    limits: ExecutionLimitsV1,
    redaction: RedactionPolicyV1,
    allow_shell: bool,
    allow_process: bool,
    allow_network: bool,
    allow_secret: bool,
    allow_out_of_project_write: bool,
}

impl AgentExecutionPolicy {
    pub fn locked(
        revision: u64,
        profile: ExecutionProfile,
        allowed_tools: impl IntoIterator<Item = ToolId>,
        project: Option<ProjectExecutionScope>,
        limits: ExecutionLimitsV1,
        redaction: RedactionPolicyV1,
    ) -> Result<Self, ExecutionError> {
        let policy = Self {
            revision,
            profile,
            allowed_tools: allowed_tools.into_iter().collect(),
            project,
            limits,
            redaction,
            allow_shell: false,
            allow_process: false,
            allow_network: false,
            allow_secret: false,
            allow_out_of_project_write: false,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn with_explicit_class_authority(
        mut self,
        class: ToolClass,
        allowed: bool,
    ) -> Result<Self, ExecutionError> {
        match class {
            ToolClass::Shell => self.allow_shell = allowed,
            ToolClass::Process => self.allow_process = allowed,
            ToolClass::Network => self.allow_network = allowed,
            ToolClass::Secret => self.allow_secret = allowed,
            ToolClass::OutOfProjectWrite => self.allow_out_of_project_write = allowed,
            ToolClass::Read | ToolClass::ProjectWrite | ToolClass::Mcp | ToolClass::Service => {
                return Err(ExecutionError::InvalidPolicy);
            }
        }
        self.validate()?;
        Ok(self)
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn limits(&self) -> &ExecutionLimitsV1 {
        &self.limits
    }

    #[must_use]
    pub const fn redaction(&self) -> &RedactionPolicyV1 {
        &self.redaction
    }

    pub(crate) const fn project_scope(&self) -> Option<&ProjectExecutionScope> {
        self.project.as_ref()
    }

    pub fn evaluate(
        &self,
        registration: &ToolRegistrationV1,
        request: &PolicyToolRequestV1,
        usage: &ExecutionUsageV1,
        approval: Option<&ApprovalGrantV1>,
        now_unix: u64,
    ) -> PolicyDecisionV1 {
        let request_digest = request.normalized_digest().ok();
        let result = self.evaluate_inner(registration, request, usage, approval, now_unix);
        let (outcome, reason) = match result {
            Ok(()) => (PolicyOutcome::Allow, PolicyReasonCode::Allowed),
            Err((outcome, reason)) => (outcome, reason),
        };
        let reason_code = reason.as_str().to_string();
        let decision_digest =
            decision_digest(outcome, reason, request_digest.as_deref(), self.revision);
        PolicyDecisionV1 {
            schema_version: 1,
            outcome,
            reason,
            reason_code,
            request_digest,
            policy_revision: self.revision,
            decision_digest,
        }
    }

    fn evaluate_inner(
        &self,
        registration: &ToolRegistrationV1,
        request: &PolicyToolRequestV1,
        usage: &ExecutionUsageV1,
        approval: Option<&ApprovalGrantV1>,
        now_unix: u64,
    ) -> Result<(), (PolicyOutcome, PolicyReasonCode)> {
        if self.validate().is_err()
            || registration.validate().is_err()
            || request.validate().is_err()
            || registration.tool_id != request.tool_id
        {
            return Err((PolicyOutcome::Deny, PolicyReasonCode::RequestInvalid));
        }
        if !self.allowed_tools.contains(&request.tool_id) {
            return Err((PolicyOutcome::Deny, PolicyReasonCode::ToolNotAllowlisted));
        }
        if self.profile == ExecutionProfile::Lite
            && !matches!(registration.class, ToolClass::Read | ToolClass::Service)
        {
            return Err((PolicyOutcome::Deny, PolicyReasonCode::LiteProfileForbidden));
        }
        if !self.class_allowed(registration.class) {
            return Err((
                PolicyOutcome::Deny,
                PolicyReasonCode::ClassAuthorityUnavailable,
            ));
        }
        if usage.exceeds(&self.limits) {
            return Err((PolicyOutcome::Deny, PolicyReasonCode::LimitExhausted));
        }
        if registration.requires_project {
            let Some(scope) = &self.project else {
                return Err((PolicyOutcome::Deny, PolicyReasonCode::ProjectScopeRequired));
            };
            if request.project_id.as_ref() != Some(&scope.project_id) {
                return Err((PolicyOutcome::Deny, PolicyReasonCode::ProjectScopeMismatch));
            }
            if request.expected_project_revision != Some(scope.semantic_revision) {
                return Err((
                    PolicyOutcome::Deny,
                    PolicyReasonCode::ProjectRevisionMismatch,
                ));
            }
        }
        let approval_required = registration.requires_approval
            || matches!(
                registration.class,
                ToolClass::ProjectWrite
                    | ToolClass::OutOfProjectWrite
                    | ToolClass::Shell
                    | ToolClass::Process
                    | ToolClass::Network
                    | ToolClass::Secret
            );
        if approval_required {
            let Some(approval) = approval else {
                return Err((
                    PolicyOutcome::ApprovalRequired,
                    PolicyReasonCode::ApprovalMissing,
                ));
            };
            match approval.accepts(request, self.revision, now_unix) {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    return Err((PolicyOutcome::Deny, PolicyReasonCode::ApprovalInvalid));
                }
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), ExecutionError> {
        if self.revision == 0
            || self.revision > MAX_POLICY_REVISION
            || self.allowed_tools.is_empty()
            || (self.profile == ExecutionProfile::Lite
                && (self.allow_shell
                    || self.allow_process
                    || self.allow_network
                    || self.allow_secret
                    || self.allow_out_of_project_write))
        {
            return Err(ExecutionError::InvalidPolicy);
        }
        self.limits.validate()?;
        self.redaction.validate()
    }

    const fn class_allowed(&self, class: ToolClass) -> bool {
        match class {
            ToolClass::Read | ToolClass::ProjectWrite | ToolClass::Mcp | ToolClass::Service => true,
            ToolClass::OutOfProjectWrite => self.allow_out_of_project_write,
            ToolClass::Shell => self.allow_shell,
            ToolClass::Process => self.allow_process,
            ToolClass::Network => self.allow_network,
            ToolClass::Secret => self.allow_secret,
        }
    }
}

fn valid_relative_artifact(value: &str) -> bool {
    if value.is_empty()
        || value.len() > MAX_RELATIVE_PATH_BYTES
        || value.contains('\\')
        || value.contains(':')
        || value
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        return false;
    }
    let path = Path::new(value);
    !path.is_absolute()
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && path.components().all(|component| {
            matches!(component, Component::Normal(_))
                && component
                    .as_os_str()
                    .to_str()
                    .is_some_and(|part| part != ".")
        })
}

fn decision_digest(
    outcome: PolicyOutcome,
    reason: PolicyReasonCode,
    request_digest: Option<&str>,
    policy_revision: u64,
) -> String {
    canonical_sha256(&json!({
        "schemaVersion": 1,
        "outcome": outcome,
        "reason": reason,
        "requestDigest": request_digest,
        "policyRevision": policy_revision,
    }))
    .expect("closed decision record is canonically serializable")
}

fn canonical_sha256(value: &impl Serialize) -> Result<String, serde_json::Error> {
    let canonical = serde_json_canonicalizer::to_vec(value)?;
    let digest = Sha256::digest(canonical);
    Ok(format!("{digest:x}"))
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(class: ToolClass, approval: bool) -> ToolRegistrationV1 {
        ToolRegistrationV1 {
            schema_version: 1,
            tool_id: ToolId::parse("project.capture-apply").unwrap(),
            class,
            execution: ToolExecutionKind::ReservedChild,
            read_only: false,
            requires_project: true,
            requires_approval: approval,
            allows_network: class == ToolClass::Network,
        }
    }

    fn request() -> PolicyToolRequestV1 {
        PolicyToolRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "3".repeat(32))).unwrap(),
            call_id: ToolCallId::parse(format!("call_{}", "4".repeat(32))).unwrap(),
            purpose: "Apply the exact previewed capture artifact set.".to_string(),
            tool_id: ToolId::parse("project.capture-apply").unwrap(),
            arguments: json!({"planDigest": "a".repeat(64)}),
            project_id: Some(ProjectId::parse(format!("prj_{}", "5".repeat(32))).unwrap()),
            expected_project_revision: Some(7),
            declared_artifacts: vec!["context/research_captures.jsonl".to_string()],
        }
    }

    fn policy(profile: ExecutionProfile) -> AgentExecutionPolicy {
        let scope = ProjectExecutionScope::new(
            ProjectId::parse(format!("prj_{}", "5".repeat(32))).unwrap(),
            PathBuf::from("/registered/article"),
            7,
        )
        .unwrap();
        AgentExecutionPolicy::locked(
            11,
            profile,
            [ToolId::parse("project.capture-apply").unwrap()],
            Some(scope),
            ExecutionLimitsV1::bounded_default(),
            RedactionPolicyV1::strict_default(),
        )
        .unwrap()
    }

    #[test]
    fn project_write_requires_a_request_bound_user_approval() {
        let policy = policy(ExecutionProfile::Full);
        let request = request();
        let missing = policy.evaluate(
            &tool(ToolClass::ProjectWrite, true),
            &request,
            &ExecutionUsageV1::default(),
            None,
            1_000,
        );
        assert_eq!(missing.outcome, PolicyOutcome::ApprovalRequired);
        assert_eq!(missing.reason, PolicyReasonCode::ApprovalMissing);

        let approval = ApprovalGrantV1::for_request(
            &request,
            policy.revision(),
            ApprovalActor::User,
            1_000,
            1_300,
        )
        .unwrap();
        let allowed = policy.evaluate(
            &tool(ToolClass::ProjectWrite, true),
            &request,
            &ExecutionUsageV1::default(),
            Some(&approval),
            1_100,
        );
        assert!(allowed.is_allowed());

        let mut changed = request;
        changed.arguments = json!({"planDigest": "b".repeat(64)});
        let stale = policy.evaluate(
            &tool(ToolClass::ProjectWrite, true),
            &changed,
            &ExecutionUsageV1::default(),
            Some(&approval),
            1_100,
        );
        assert_eq!(stale.reason, PolicyReasonCode::ApprovalInvalid);
    }

    #[test]
    fn lite_and_locked_full_profiles_deny_broad_authority() {
        let request = request();
        let lite = policy(ExecutionProfile::Lite).evaluate(
            &tool(ToolClass::ProjectWrite, true),
            &request,
            &ExecutionUsageV1::default(),
            None,
            1_000,
        );
        assert_eq!(lite.reason, PolicyReasonCode::LiteProfileForbidden);

        let shell = policy(ExecutionProfile::Full).evaluate(
            &tool(ToolClass::Shell, true),
            &request,
            &ExecutionUsageV1::default(),
            None,
            1_000,
        );
        assert_eq!(shell.reason, PolicyReasonCode::ClassAuthorityUnavailable);
    }

    #[test]
    fn project_scope_revision_limits_and_paths_fail_closed() {
        let policy = policy(ExecutionProfile::Full);
        let mut changed = request();
        changed.expected_project_revision = Some(8);
        let revision = policy.evaluate(
            &tool(ToolClass::ProjectWrite, true),
            &changed,
            &ExecutionUsageV1::default(),
            None,
            1_000,
        );
        assert_eq!(revision.reason, PolicyReasonCode::ProjectRevisionMismatch);

        let exhausted = policy.evaluate(
            &tool(ToolClass::ProjectWrite, true),
            &request(),
            &ExecutionUsageV1 {
                tool_calls: policy.limits().tool_calls,
                ..ExecutionUsageV1::default()
            },
            None,
            1_000,
        );
        assert_eq!(exhausted.reason, PolicyReasonCode::LimitExhausted);

        let mut traversal = request();
        traversal.declared_artifacts = vec!["../outside".to_string()];
        assert_eq!(
            traversal.validate(),
            Err(ExecutionError::InvalidToolRequest)
        );
    }
}
