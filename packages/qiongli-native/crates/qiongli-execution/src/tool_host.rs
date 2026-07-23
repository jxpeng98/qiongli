use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AgentExecutionPolicy, ExecutionError, ExecutionLimitsV1, PolicyDecisionV1, PolicyOutcome,
    PolicyToolRequestV1, RedactionPolicyV1, TOOL_HOST_PROTOCOL_VERSION, ToolExecutionKind, ToolId,
    ToolRegistrationV1,
};

const MAX_REGISTERED_TOOLS: usize = 256;
const MAX_AUDIT_REASON_BYTES: usize = 96;

#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone, Default)]
pub struct ToolHostRegistry {
    registrations: BTreeMap<ToolId, ToolRegistrationV1>,
}

impl ToolHostRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registrations: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, registration: ToolRegistrationV1) -> Result<(), ExecutionError> {
        registration.validate()?;
        if self.registrations.len() >= MAX_REGISTERED_TOOLS
            || self.registrations.contains_key(&registration.tool_id)
        {
            return Err(ExecutionError::InvalidToolRegistration);
        }
        self.registrations
            .insert(registration.tool_id.clone(), registration);
        Ok(())
    }

    #[must_use]
    pub fn registration(&self, tool_id: &ToolId) -> Option<&ToolRegistrationV1> {
        self.registrations.get(tool_id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    pub fn prepare(
        &self,
        policy: &AgentExecutionPolicy,
        request: PolicyToolRequestV1,
        decision: &PolicyDecisionV1,
    ) -> Result<ToolHostInvocationV1, ExecutionError> {
        let registration = self
            .registration(&request.tool_id)
            .ok_or(ExecutionError::ToolNotRegistered)?
            .clone();
        decision.validate()?;
        if decision.outcome == PolicyOutcome::ApprovalRequired {
            return Err(ExecutionError::ApprovalRequired);
        }
        if !decision.is_allowed() {
            return Err(ExecutionError::PolicyDenied);
        }
        let request_digest = request.normalized_digest()?;
        if decision.schema_version != 1
            || decision.request_digest.as_deref() != Some(request_digest.as_str())
            || decision.policy_revision != policy.revision()
            || !valid_lower_hex(&decision.decision_digest, 64)
        {
            return Err(ExecutionError::ToolHostContractInvalid);
        }
        let project_root = match (registration.requires_project, policy.project_scope()) {
            (true, Some(scope)) => Some(scope.canonical_root.clone()),
            (true, None) => return Err(ExecutionError::ToolHostContractInvalid),
            (false, _) => None,
        };
        Ok(ToolHostInvocationV1 {
            schema_version: 1,
            protocol_version: TOOL_HOST_PROTOCOL_VERSION.to_string(),
            request,
            registration,
            project_root,
            request_digest,
            decision_digest: decision.decision_digest.clone(),
            decision: decision.clone(),
            policy_revision: decision.policy_revision,
            limits: policy.limits().clone(),
            redaction: policy.redaction().clone(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolHostInvocationV1 {
    pub schema_version: u32,
    pub protocol_version: String,
    pub request: PolicyToolRequestV1,
    pub registration: ToolRegistrationV1,
    pub project_root: Option<PathBuf>,
    pub request_digest: String,
    pub decision_digest: String,
    pub decision: PolicyDecisionV1,
    pub policy_revision: u64,
    pub limits: ExecutionLimitsV1,
    pub redaction: RedactionPolicyV1,
}

impl ToolHostInvocationV1 {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        if self.schema_version != 1
            || self.protocol_version != TOOL_HOST_PROTOCOL_VERSION
            || self.request.normalized_digest().as_deref() != Ok(self.request_digest.as_str())
            || !valid_lower_hex(&self.decision_digest, 64)
            || self.decision.validate().is_err()
            || !self.decision.is_allowed()
            || self.decision.decision_digest != self.decision_digest
            || self.decision.request_digest.as_deref() != Some(self.request_digest.as_str())
            || self.decision.policy_revision != self.policy_revision
            || self.policy_revision == 0
            || self.registration.tool_id != self.request.tool_id
            || self.registration.requires_project != self.project_root.is_some()
            || self
                .project_root
                .as_ref()
                .is_some_and(|root| !root.is_absolute())
            || (self.registration.execution == ToolExecutionKind::InProcessReadOnly
                && !self.registration.read_only)
        {
            return Err(ExecutionError::ToolHostContractInvalid);
        }
        self.registration.validate()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolResultStatus {
    Completed,
    Failed,
    Cancelled,
    LimitExceeded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolAuditOutcome {
    Completed,
    Failed,
    Cancelled,
    LimitExceeded,
}

impl From<ToolResultStatus> for ToolAuditOutcome {
    fn from(value: ToolResultStatus) -> Self {
        match value {
            ToolResultStatus::Completed => Self::Completed,
            ToolResultStatus::Failed => Self::Failed,
            ToolResultStatus::Cancelled => Self::Cancelled,
            ToolResultStatus::LimitExceeded => Self::LimitExceeded,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolAuditRecordV1 {
    pub schema_version: u32,
    pub run_id: String,
    pub call_id: String,
    pub tool_id: String,
    pub tool_class: crate::ToolClass,
    pub request_digest: String,
    pub decision_digest: String,
    pub policy_revision: u64,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: u64,
    pub outcome: ToolAuditOutcome,
    pub reason_code: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub redaction_count: u32,
    pub truncated: bool,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolHostResultV1 {
    pub schema_version: u32,
    pub status: ToolResultStatus,
    pub content: Value,
    pub truncated: bool,
    pub redaction_count: u32,
    pub audit: ToolAuditRecordV1,
}

pub struct ToolHostResultInput {
    pub status: ToolResultStatus,
    pub content: Value,
    pub truncated: bool,
    pub redaction_count: u32,
    pub reason_code: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: u64,
}

impl ToolHostResultV1 {
    pub fn from_redacted(
        invocation: &ToolHostInvocationV1,
        input: ToolHostResultInput,
    ) -> Result<Self, ExecutionError> {
        invocation.validate()?;
        let serialized_bytes = serde_json::to_vec(&input.content)
            .map_err(|_| ExecutionError::ToolHostContractInvalid)?
            .len() as u64;
        if !valid_reason_code(&input.reason_code)
            || input.finished_at_unix_ms < input.started_at_unix_ms
            || input.output_bytes != serialized_bytes
            || input.output_bytes > invocation.redaction.maximum_result_bytes
            || (input.output_bytes > invocation.limits.output_bytes
                && input.status == ToolResultStatus::Completed)
            || (input.input_bytes > invocation.limits.input_bytes
                && input.status != ToolResultStatus::LimitExceeded)
        {
            return Err(ExecutionError::ToolHostContractInvalid);
        }
        let audit = ToolAuditRecordV1 {
            schema_version: 1,
            run_id: invocation.request.run_id.as_str().to_string(),
            call_id: invocation.request.call_id.as_str().to_string(),
            tool_id: invocation.request.tool_id.as_str().to_string(),
            tool_class: invocation.registration.class,
            request_digest: invocation.request_digest.clone(),
            decision_digest: invocation.decision_digest.clone(),
            policy_revision: invocation.policy_revision,
            started_at_unix_ms: input.started_at_unix_ms,
            finished_at_unix_ms: input.finished_at_unix_ms,
            outcome: input.status.into(),
            reason_code: input.reason_code,
            input_bytes: input.input_bytes,
            output_bytes: input.output_bytes,
            redaction_count: input.redaction_count,
            truncated: input.truncated,
        };
        Ok(Self {
            schema_version: 1,
            status: input.status,
            content: input.content,
            truncated: input.truncated,
            redaction_count: input.redaction_count,
            audit,
        })
    }
}

fn valid_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_AUDIT_REASON_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use qiongli_project::ProjectId;
    use serde_json::json;

    use crate::{
        ApprovalActor, ApprovalGrantV1, ExecutionProfile, ExecutionUsageV1, ProjectExecutionScope,
        RunId, ToolCallId, ToolClass,
    };

    use super::*;

    fn registration() -> ToolRegistrationV1 {
        ToolRegistrationV1 {
            schema_version: 1,
            tool_id: ToolId::parse("project.capture-apply").unwrap(),
            class: ToolClass::ProjectWrite,
            execution: ToolExecutionKind::ReservedChild,
            read_only: false,
            requires_project: true,
            requires_approval: true,
            allows_network: false,
        }
    }

    fn request() -> PolicyToolRequestV1 {
        PolicyToolRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "6".repeat(32))).unwrap(),
            call_id: ToolCallId::parse(format!("call_{}", "7".repeat(32))).unwrap(),
            purpose: "Apply one previewed capture.".to_string(),
            tool_id: ToolId::parse("project.capture-apply").unwrap(),
            arguments: json!({"private": "private-tool-canary"}),
            project_id: Some(ProjectId::parse(format!("prj_{}", "8".repeat(32))).unwrap()),
            expected_project_revision: Some(3),
            declared_artifacts: vec!["context/research_captures.jsonl".to_string()],
        }
    }

    fn policy() -> AgentExecutionPolicy {
        AgentExecutionPolicy::locked(
            2,
            ExecutionProfile::Full,
            [ToolId::parse("project.capture-apply").unwrap()],
            Some(
                ProjectExecutionScope::new(
                    ProjectId::parse(format!("prj_{}", "8".repeat(32))).unwrap(),
                    PathBuf::from("/registered/private/article"),
                    3,
                )
                .unwrap(),
            ),
            ExecutionLimitsV1::bounded_default(),
            RedactionPolicyV1::strict_default(),
        )
        .unwrap()
    }

    fn invocation() -> ToolHostInvocationV1 {
        let policy = policy();
        let request = request();
        let approval = ApprovalGrantV1::for_request(
            &request,
            policy.revision(),
            ApprovalActor::User,
            100,
            200,
        )
        .unwrap();
        let decision = policy.evaluate(
            &registration(),
            &request,
            &ExecutionUsageV1::default(),
            Some(&approval),
            150,
        );
        let mut registry = ToolHostRegistry::new();
        registry.register(registration()).unwrap();
        registry.prepare(&policy, request, &decision).unwrap()
    }

    #[test]
    fn registry_rejects_duplicates_and_unsafe_in_process_tools() {
        let mut registry = ToolHostRegistry::new();
        registry.register(registration()).unwrap();
        assert_eq!(
            registry.register(registration()),
            Err(ExecutionError::InvalidToolRegistration)
        );

        let mut unsafe_in_process = registration();
        unsafe_in_process.execution = ToolExecutionKind::InProcessReadOnly;
        assert_eq!(
            unsafe_in_process.validate(),
            Err(ExecutionError::InvalidToolRegistration)
        );
    }

    #[test]
    fn tool_host_accepts_only_an_allowed_request_bound_decision() {
        let invocation = invocation();
        assert!(invocation.validate().is_ok());

        let policy = policy();
        let request = request();
        let decision = policy.evaluate(
            &registration(),
            &request,
            &ExecutionUsageV1::default(),
            None,
            150,
        );
        let mut registry = ToolHostRegistry::new();
        registry.register(registration()).unwrap();
        assert_eq!(
            registry.prepare(&policy, request, &decision).err(),
            Some(ExecutionError::ApprovalRequired)
        );
    }

    #[test]
    fn tool_host_rejects_a_tampered_allow_decision() {
        let policy = policy();
        let request = request();
        let approval = ApprovalGrantV1::for_request(
            &request,
            policy.revision(),
            ApprovalActor::User,
            100,
            200,
        )
        .unwrap();
        let mut decision = policy.evaluate(
            &registration(),
            &request,
            &ExecutionUsageV1::default(),
            Some(&approval),
            150,
        );
        decision.policy_revision += 1;
        let mut registry = ToolHostRegistry::new();
        registry.register(registration()).unwrap();
        assert_eq!(
            registry.prepare(&policy, request, &decision).err(),
            Some(ExecutionError::PolicyDenied)
        );
    }

    #[test]
    fn audit_contains_hashes_and_fixed_outcomes_but_no_arguments_or_paths() {
        let invocation = invocation();
        let content = json!({"status": "applied"});
        let output_bytes = serde_json::to_vec(&content).unwrap().len() as u64;
        let result = ToolHostResultV1::from_redacted(
            &invocation,
            ToolHostResultInput {
                status: ToolResultStatus::Completed,
                content,
                truncated: false,
                redaction_count: 1,
                reason_code: "tool-completed".to_string(),
                input_bytes: 256,
                output_bytes,
                started_at_unix_ms: 1_000,
                finished_at_unix_ms: 1_050,
            },
        )
        .unwrap();
        let audit = serde_json::to_string(&result.audit).unwrap();
        assert!(!audit.contains("private-tool-canary"));
        assert!(!audit.contains("/registered/private/article"));
        assert!(audit.contains(&invocation.request_digest));
        assert!(audit.contains("tool-completed"));
    }

    #[test]
    fn cancellation_is_monotonic_and_shared() {
        let token = CancellationToken::new();
        let observer = token.clone();
        assert!(!observer.is_cancelled());
        token.cancel();
        assert!(observer.is_cancelled());
    }
}
