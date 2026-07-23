use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use qiongli_project::ProjectId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{OrchestrationRole, OrchestrationTaskId, RunId, ToolCallId, ToolId};

pub const HOST_HANDOFF_SCHEMA_VERSION: u32 = 1;
pub const HOST_HANDOFF_PROTOCOL_VERSION: &str = "qiongli-host-handoff/1";
pub const FULL_MCP_HOST_PROTOCOL_VERSION: &str = "qiongli-full-mcp/1";

const MAX_RUNTIME_BYTES: usize = 65_536;
const MAX_HANDOFF_BYTES: usize = 262_144;
const MAX_CANDIDATE_ENVELOPE_BYTES: usize = 262_144;
const MAX_INSTRUCTIONS_BYTES: usize = 32_768;
const MAX_CANDIDATE_BYTES: u64 = 65_536;
const MAX_ALLOWED_TOOLS: usize = 32;
const MAX_EVIDENCE_REFERENCES: usize = 32;
const MAX_DISCLOSURES: usize = 16;
const MAX_DISCLOSURE_BYTES: usize = 1_024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostFamilyV1 {
    Codex,
    ClaudeCode,
    ClaudeDesktop,
    OtherLocal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostCapabilityV1 {
    SingleAgent,
    NativeSubagents,
    Attachments,
    StructuredOutput,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostComponentStateV1 {
    Missing,
    Present,
    HostActionRequired,
    Ready,
    Unsupported,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostCandidateKindV1 {
    ResearchTask,
    Review,
    Verification,
    Worker,
    Synthesis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostHandoffError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidRuntime,
    InvalidHandoff,
    InvalidCandidate,
    BindingMismatch,
    SerializationFailed,
}

impl HostHandoffError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "host-handoff-input-too-large",
            Self::InvalidJson => "host-handoff-json-invalid",
            Self::NonCanonicalJson => "host-handoff-json-noncanonical",
            Self::InvalidRuntime => "host-runtime-invalid",
            Self::InvalidHandoff => "host-handoff-invalid",
            Self::InvalidCandidate => "host-candidate-invalid",
            Self::BindingMismatch => "host-candidate-binding-mismatch",
            Self::SerializationFailed => "host-handoff-serialization-failed",
        }
    }
}

impl Display for HostHandoffError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for HostHandoffError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostRuntimeDescriptorV1 {
    pub schema_version: u32,
    pub family: HostFamilyV1,
    pub host_version: String,
    pub adapter_version: String,
    pub full_mcp_protocol: String,
    pub capabilities: Vec<HostCapabilityV1>,
    pub plugin_state: HostComponentStateV1,
    pub registration_state: HostComponentStateV1,
    pub enablement_state: HostComponentStateV1,
    pub trust_state: HostComponentStateV1,
    pub activation_state: HostComponentStateV1,
}

impl HostRuntimeDescriptorV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        family: HostFamilyV1,
        host_version: impl Into<String>,
        adapter_version: impl Into<String>,
        mut capabilities: Vec<HostCapabilityV1>,
        plugin_state: HostComponentStateV1,
        registration_state: HostComponentStateV1,
        enablement_state: HostComponentStateV1,
        trust_state: HostComponentStateV1,
        activation_state: HostComponentStateV1,
    ) -> Result<Self, HostHandoffError> {
        capabilities.sort_unstable();
        let descriptor = Self {
            schema_version: HOST_HANDOFF_SCHEMA_VERSION,
            family,
            host_version: host_version.into(),
            adapter_version: adapter_version.into(),
            full_mcp_protocol: FULL_MCP_HOST_PROTOCOL_VERSION.to_owned(),
            capabilities,
            plugin_state,
            registration_state,
            enablement_state,
            trust_state,
            activation_state,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub fn from_canonical_json(input: &[u8]) -> Result<Self, HostHandoffError> {
        if input.len() > MAX_RUNTIME_BYTES {
            return Err(HostHandoffError::InputTooLarge);
        }
        let descriptor =
            serde_json::from_slice::<Self>(input).map_err(|_| HostHandoffError::InvalidJson)?;
        descriptor.validate()?;
        if descriptor.to_canonical_json()? != input {
            return Err(HostHandoffError::NonCanonicalJson);
        }
        Ok(descriptor)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, HostHandoffError> {
        self.validate()?;
        canonical_json(self, MAX_RUNTIME_BYTES)
    }

    pub fn digest(&self) -> Result<String, HostHandoffError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.plugin_state, HostComponentStateV1::Ready)
            && matches!(self.registration_state, HostComponentStateV1::Ready)
            && matches!(self.enablement_state, HostComponentStateV1::Ready)
            && matches!(self.trust_state, HostComponentStateV1::Ready)
            && matches!(self.activation_state, HostComponentStateV1::Ready)
    }

    fn validate(&self) -> Result<(), HostHandoffError> {
        let capabilities = self.capabilities.iter().copied().collect::<BTreeSet<_>>();
        if self.schema_version != HOST_HANDOFF_SCHEMA_VERSION
            || !valid_version_token(&self.host_version)
            || !valid_version_token(&self.adapter_version)
            || self.full_mcp_protocol != FULL_MCP_HOST_PROTOCOL_VERSION
            || self.capabilities.is_empty()
            || self.capabilities.len() > HostCapabilityV1::COUNT
            || capabilities.len() != self.capabilities.len()
            || !strictly_sorted(&self.capabilities)
            || !capabilities.contains(&HostCapabilityV1::SingleAgent)
        {
            return Err(HostHandoffError::InvalidRuntime);
        }
        Ok(())
    }
}

impl HostCapabilityV1 {
    const COUNT: usize = 4;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostExecutionLimitsV1 {
    pub max_candidate_bytes: u64,
    pub max_tool_calls: u16,
    pub max_wall_seconds: u32,
    pub max_retries: u8,
}

impl HostExecutionLimitsV1 {
    pub fn try_new(
        max_candidate_bytes: u64,
        max_tool_calls: u16,
        max_wall_seconds: u32,
        max_retries: u8,
    ) -> Result<Self, HostHandoffError> {
        let limits = Self {
            max_candidate_bytes,
            max_tool_calls,
            max_wall_seconds,
            max_retries,
        };
        limits.validate()?;
        Ok(limits)
    }

    #[must_use]
    pub const fn bounded_default() -> Self {
        Self {
            max_candidate_bytes: 32_768,
            max_tool_calls: 16,
            max_wall_seconds: 900,
            max_retries: 1,
        }
    }

    fn validate(&self) -> Result<(), HostHandoffError> {
        if self.max_candidate_bytes == 0
            || self.max_candidate_bytes > MAX_CANDIDATE_BYTES
            || self.max_tool_calls == 0
            || usize::from(self.max_tool_calls) > MAX_EVIDENCE_REFERENCES
            || self.max_wall_seconds == 0
            || self.max_wall_seconds > 3_600
            || self.max_retries > 3
        {
            return Err(HostHandoffError::InvalidHandoff);
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrchestrationHandoffV1 {
    pub schema_version: u32,
    pub protocol_version: String,
    pub host: HostRuntimeDescriptorV1,
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub task_id: OrchestrationTaskId,
    pub role: OrchestrationRole,
    pub attempt: u8,
    pub checkpoint_generation: u64,
    pub checkpoint_document_sha256: String,
    pub workflow_sha256: String,
    pub profile_sha256: String,
    pub task_packet_sha256: String,
    pub candidate_kind: HostCandidateKindV1,
    pub instructions: String,
    pub allowed_tool_ids: Vec<ToolId>,
    pub minimum_evidence_count: u16,
    pub limits: HostExecutionLimitsV1,
}

impl OrchestrationHandoffV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        host: HostRuntimeDescriptorV1,
        run_id: RunId,
        project_id: ProjectId,
        expected_project_revision: u64,
        task_id: OrchestrationTaskId,
        role: OrchestrationRole,
        attempt: u8,
        checkpoint_generation: u64,
        checkpoint_document_sha256: impl Into<String>,
        workflow_sha256: impl Into<String>,
        profile_sha256: impl Into<String>,
        task_packet_sha256: impl Into<String>,
        candidate_kind: HostCandidateKindV1,
        instructions: impl Into<String>,
        mut allowed_tool_ids: Vec<ToolId>,
        minimum_evidence_count: u16,
        limits: HostExecutionLimitsV1,
    ) -> Result<Self, HostHandoffError> {
        allowed_tool_ids.sort();
        let handoff = Self {
            schema_version: HOST_HANDOFF_SCHEMA_VERSION,
            protocol_version: HOST_HANDOFF_PROTOCOL_VERSION.to_owned(),
            host,
            run_id,
            project_id,
            expected_project_revision,
            task_id,
            role,
            attempt,
            checkpoint_generation,
            checkpoint_document_sha256: checkpoint_document_sha256.into(),
            workflow_sha256: workflow_sha256.into(),
            profile_sha256: profile_sha256.into(),
            task_packet_sha256: task_packet_sha256.into(),
            candidate_kind,
            instructions: instructions.into(),
            allowed_tool_ids,
            minimum_evidence_count,
            limits,
        };
        handoff.validate()?;
        Ok(handoff)
    }

    pub fn from_canonical_json(input: &[u8]) -> Result<Self, HostHandoffError> {
        if input.len() > MAX_HANDOFF_BYTES {
            return Err(HostHandoffError::InputTooLarge);
        }
        let handoff =
            serde_json::from_slice::<Self>(input).map_err(|_| HostHandoffError::InvalidJson)?;
        handoff.validate()?;
        if handoff.to_canonical_json()? != input {
            return Err(HostHandoffError::NonCanonicalJson);
        }
        Ok(handoff)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, HostHandoffError> {
        self.validate()?;
        canonical_json(self, MAX_HANDOFF_BYTES)
    }

    pub fn digest(&self) -> Result<String, HostHandoffError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    pub fn validate_candidate(
        &self,
        candidate: &HostCandidateEnvelopeV1,
    ) -> Result<(), HostHandoffError> {
        candidate.validate_against(self)
    }

    fn validate(&self) -> Result<(), HostHandoffError> {
        self.host.validate()?;
        let allowed_tools = self.allowed_tool_ids.iter().collect::<BTreeSet<_>>();
        if self.schema_version != HOST_HANDOFF_SCHEMA_VERSION
            || self.protocol_version != HOST_HANDOFF_PROTOCOL_VERSION
            || RunId::parse(self.run_id.as_str()).is_err()
            || ProjectId::parse(self.project_id.as_str()).is_err()
            || OrchestrationTaskId::parse(self.task_id.as_str()).is_err()
            || self.expected_project_revision == 0
            || self.expected_project_revision > MAX_SAFE_INTEGER
            || self.attempt == 0
            || self.attempt > 3
            || self.checkpoint_generation > MAX_SAFE_INTEGER
            || !valid_sha256(&self.checkpoint_document_sha256)
            || !valid_sha256(&self.workflow_sha256)
            || !valid_sha256(&self.profile_sha256)
            || !valid_sha256(&self.task_packet_sha256)
            || !valid_private_text(&self.instructions, MAX_INSTRUCTIONS_BYTES)
            || self.allowed_tool_ids.len() > MAX_ALLOWED_TOOLS
            || allowed_tools.len() != self.allowed_tool_ids.len()
            || !strictly_sorted(&self.allowed_tool_ids)
            || usize::from(self.minimum_evidence_count) > self.allowed_tool_ids.len()
            || self.minimum_evidence_count > self.limits.max_tool_calls
            || self.limits.validate().is_err()
        {
            return Err(HostHandoffError::InvalidHandoff);
        }
        Ok(())
    }
}

impl Debug for OrchestrationHandoffV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OrchestrationHandoffV1")
            .field("schema_version", &self.schema_version)
            .field("protocol_version", &self.protocol_version)
            .field("host", &self.host)
            .field("run_id", &self.run_id)
            .field("project_id", &self.project_id)
            .field("expected_project_revision", &self.expected_project_revision)
            .field("task_id", &self.task_id)
            .field("role", &self.role)
            .field("attempt", &self.attempt)
            .field("checkpoint_generation", &self.checkpoint_generation)
            .field(
                "checkpoint_document_sha256",
                &self.checkpoint_document_sha256,
            )
            .field("workflow_sha256", &self.workflow_sha256)
            .field("profile_sha256", &self.profile_sha256)
            .field("task_packet_sha256", &self.task_packet_sha256)
            .field("candidate_kind", &self.candidate_kind)
            .field("instructions", &"<private-host-instructions>")
            .field("allowed_tool_ids", &self.allowed_tool_ids)
            .field("minimum_evidence_count", &self.minimum_evidence_count)
            .field("limits", &self.limits)
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostEvidenceReferenceV1 {
    pub run_id: RunId,
    pub call_id: ToolCallId,
    pub tool_id: ToolId,
    pub request_sha256: String,
    pub decision_sha256: String,
    pub result_sha256: String,
}

impl HostEvidenceReferenceV1 {
    pub fn try_new(
        run_id: RunId,
        call_id: ToolCallId,
        tool_id: ToolId,
        request_sha256: impl Into<String>,
        decision_sha256: impl Into<String>,
        result_sha256: impl Into<String>,
    ) -> Result<Self, HostHandoffError> {
        let evidence = Self {
            run_id,
            call_id,
            tool_id,
            request_sha256: request_sha256.into(),
            decision_sha256: decision_sha256.into(),
            result_sha256: result_sha256.into(),
        };
        evidence.validate()?;
        Ok(evidence)
    }

    fn validate(&self) -> Result<(), HostHandoffError> {
        if RunId::parse(self.run_id.as_str()).is_err()
            || ToolCallId::parse(self.call_id.as_str()).is_err()
            || ToolId::parse(self.tool_id.as_str()).is_err()
            || !valid_sha256(&self.request_sha256)
            || !valid_sha256(&self.decision_sha256)
            || !valid_sha256(&self.result_sha256)
        {
            return Err(HostHandoffError::InvalidCandidate);
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostCandidateEnvelopeV1 {
    pub schema_version: u32,
    pub handoff_sha256: String,
    pub run_id: RunId,
    pub project_id: ProjectId,
    pub expected_project_revision: u64,
    pub task_id: OrchestrationTaskId,
    pub role: OrchestrationRole,
    pub attempt: u8,
    pub candidate_kind: HostCandidateKindV1,
    pub content: String,
    pub evidence: Vec<HostEvidenceReferenceV1>,
    pub conflicts: Vec<String>,
    pub evidence_gaps: Vec<String>,
}

impl HostCandidateEnvelopeV1 {
    pub fn try_new(
        handoff: &OrchestrationHandoffV1,
        content: impl Into<String>,
        evidence: Vec<HostEvidenceReferenceV1>,
        conflicts: Vec<String>,
        evidence_gaps: Vec<String>,
    ) -> Result<Self, HostHandoffError> {
        let candidate = Self {
            schema_version: HOST_HANDOFF_SCHEMA_VERSION,
            handoff_sha256: handoff.digest()?,
            run_id: handoff.run_id.clone(),
            project_id: handoff.project_id.clone(),
            expected_project_revision: handoff.expected_project_revision,
            task_id: handoff.task_id.clone(),
            role: handoff.role,
            attempt: handoff.attempt,
            candidate_kind: handoff.candidate_kind,
            content: content.into(),
            evidence,
            conflicts,
            evidence_gaps,
        };
        candidate.validate_against(handoff)?;
        Ok(candidate)
    }

    pub fn from_canonical_json(
        handoff: &OrchestrationHandoffV1,
        input: &[u8],
    ) -> Result<Self, HostHandoffError> {
        if input.len() > MAX_CANDIDATE_ENVELOPE_BYTES {
            return Err(HostHandoffError::InputTooLarge);
        }
        let candidate =
            serde_json::from_slice::<Self>(input).map_err(|_| HostHandoffError::InvalidJson)?;
        candidate.validate_against(handoff)?;
        if candidate.to_canonical_json(handoff)? != input {
            return Err(HostHandoffError::NonCanonicalJson);
        }
        Ok(candidate)
    }

    pub fn to_canonical_json(
        &self,
        handoff: &OrchestrationHandoffV1,
    ) -> Result<Vec<u8>, HostHandoffError> {
        self.validate_against(handoff)?;
        canonical_json(self, MAX_CANDIDATE_ENVELOPE_BYTES)
    }

    pub fn digest(&self, handoff: &OrchestrationHandoffV1) -> Result<String, HostHandoffError> {
        Ok(sha256(&self.to_canonical_json(handoff)?))
    }

    fn validate_against(&self, handoff: &OrchestrationHandoffV1) -> Result<(), HostHandoffError> {
        handoff.validate()?;
        if self.schema_version != HOST_HANDOFF_SCHEMA_VERSION
            || self.handoff_sha256 != handoff.digest()?
            || self.run_id != handoff.run_id
            || self.project_id != handoff.project_id
            || self.expected_project_revision != handoff.expected_project_revision
            || self.task_id != handoff.task_id
            || self.role != handoff.role
            || self.attempt != handoff.attempt
            || self.candidate_kind != handoff.candidate_kind
        {
            return Err(HostHandoffError::BindingMismatch);
        }
        let content_limit = usize::try_from(handoff.limits.max_candidate_bytes)
            .map_err(|_| HostHandoffError::InvalidCandidate)?;
        let evidence_calls = self
            .evidence
            .iter()
            .map(|evidence| &evidence.call_id)
            .collect::<BTreeSet<_>>();
        if !valid_private_text(&self.content, content_limit)
            || self.evidence.len() < usize::from(handoff.minimum_evidence_count)
            || self.evidence.len() > usize::from(handoff.limits.max_tool_calls)
            || self.evidence.len() > MAX_EVIDENCE_REFERENCES
            || evidence_calls.len() != self.evidence.len()
            || self.evidence.iter().any(|evidence| {
                evidence.validate().is_err()
                    || evidence.run_id != self.run_id
                    || !handoff.allowed_tool_ids.contains(&evidence.tool_id)
            })
            || !valid_disclosures(&self.conflicts)
            || !valid_disclosures(&self.evidence_gaps)
        {
            return Err(HostHandoffError::InvalidCandidate);
        }
        Ok(())
    }
}

impl Debug for HostCandidateEnvelopeV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostCandidateEnvelopeV1")
            .field("schema_version", &self.schema_version)
            .field("handoff_sha256", &self.handoff_sha256)
            .field("run_id", &self.run_id)
            .field("project_id", &self.project_id)
            .field("expected_project_revision", &self.expected_project_revision)
            .field("task_id", &self.task_id)
            .field("role", &self.role)
            .field("attempt", &self.attempt)
            .field("candidate_kind", &self.candidate_kind)
            .field("content", &"<private-host-candidate>")
            .field("evidence", &self.evidence)
            .field("conflict_count", &self.conflicts.len())
            .field("evidence_gap_count", &self.evidence_gaps.len())
            .finish()
    }
}

fn valid_version_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
}

fn valid_private_text(value: &str, maximum_bytes: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= maximum_bytes
        && value
            .chars()
            .all(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
}

fn valid_disclosures(values: &[String]) -> bool {
    values.len() <= MAX_DISCLOSURES
        && values
            .iter()
            .all(|value| valid_private_text(value, MAX_DISCLOSURE_BYTES))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn canonical_json<T: Serialize>(
    value: &T,
    maximum_bytes: usize,
) -> Result<Vec<u8>, HostHandoffError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| HostHandoffError::SerializationFailed)?;
    if bytes.len() > maximum_bytes {
        return Err(HostHandoffError::InputTooLarge);
    }
    Ok(bytes)
}

fn sha256(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> String {
        character.to_string().repeat(64)
    }

    fn run_id() -> RunId {
        RunId::parse(format!("run_{}", "1".repeat(32))).unwrap()
    }

    fn project_id() -> ProjectId {
        ProjectId::parse(format!("prj_{}", "2".repeat(32))).unwrap()
    }

    fn runtime() -> HostRuntimeDescriptorV1 {
        HostRuntimeDescriptorV1::try_new(
            HostFamilyV1::Codex,
            "0.144.6",
            "2.0.0-alpha.1",
            vec![
                HostCapabilityV1::NativeSubagents,
                HostCapabilityV1::SingleAgent,
            ],
            HostComponentStateV1::Ready,
            HostComponentStateV1::Ready,
            HostComponentStateV1::Ready,
            HostComponentStateV1::Ready,
            HostComponentStateV1::Ready,
        )
        .unwrap()
    }

    fn handoff() -> OrchestrationHandoffV1 {
        OrchestrationHandoffV1::try_new(
            runtime(),
            run_id(),
            project_id(),
            7,
            OrchestrationTaskId::parse("B1").unwrap(),
            OrchestrationRole::Primary,
            1,
            3,
            digest('3'),
            digest('4'),
            digest('5'),
            digest('6'),
            HostCandidateKindV1::ResearchTask,
            "Read the registered project and return an evidence-grounded candidate.",
            vec![
                ToolId::parse("project.graph-query").unwrap(),
                ToolId::parse("project.read").unwrap(),
            ],
            1,
            HostExecutionLimitsV1::bounded_default(),
        )
        .unwrap()
    }

    fn evidence() -> HostEvidenceReferenceV1 {
        HostEvidenceReferenceV1::try_new(
            run_id(),
            ToolCallId::parse(format!("call_{}", "7".repeat(32))).unwrap(),
            ToolId::parse("project.read").unwrap(),
            digest('8'),
            digest('9'),
            digest('a'),
        )
        .unwrap()
    }

    #[test]
    fn runtime_is_canonical_and_requires_single_agent_capability() {
        let runtime = runtime();
        let bytes = runtime.to_canonical_json().unwrap();
        assert_eq!(
            HostRuntimeDescriptorV1::from_canonical_json(&bytes).unwrap(),
            runtime
        );

        let mut invalid = runtime.clone();
        invalid.capabilities = vec![HostCapabilityV1::NativeSubagents];
        assert_eq!(
            invalid.to_canonical_json(),
            Err(HostHandoffError::InvalidRuntime)
        );
    }

    #[test]
    fn handoff_and_candidate_round_trip_with_exact_binding() {
        let handoff = handoff();
        let handoff_bytes = handoff.to_canonical_json().unwrap();
        let restored = OrchestrationHandoffV1::from_canonical_json(&handoff_bytes).unwrap();
        assert_eq!(restored.digest().unwrap(), handoff.digest().unwrap());

        let candidate = HostCandidateEnvelopeV1::try_new(
            &handoff,
            "The project evidence supports the bounded candidate.",
            vec![evidence()],
            vec!["The external validity remains unknown.".to_owned()],
            vec!["No replication dataset is registered.".to_owned()],
        )
        .unwrap();
        let candidate_bytes = candidate.to_canonical_json(&handoff).unwrap();
        assert_eq!(
            HostCandidateEnvelopeV1::from_canonical_json(&handoff, &candidate_bytes).unwrap(),
            candidate
        );
    }

    #[test]
    fn candidate_rejects_binding_and_unoffered_evidence_substitution() {
        let handoff = handoff();
        let mut candidate = HostCandidateEnvelopeV1::try_new(
            &handoff,
            "Grounded candidate.",
            vec![evidence()],
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        candidate.expected_project_revision += 1;
        assert_eq!(
            handoff.validate_candidate(&candidate),
            Err(HostHandoffError::BindingMismatch)
        );

        candidate.expected_project_revision = handoff.expected_project_revision;
        candidate.evidence[0].tool_id = ToolId::parse("project.capture-apply").unwrap();
        assert_eq!(
            handoff.validate_candidate(&candidate),
            Err(HostHandoffError::InvalidCandidate)
        );
    }

    #[test]
    fn canonical_decoding_rejects_unknown_fields_and_pretty_json() {
        let handoff = handoff();
        let mut value =
            serde_json::from_slice::<serde_json::Value>(&handoff.to_canonical_json().unwrap())
                .unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("providerApiKey".to_owned(), serde_json::json!("private"));
        let unknown = serde_json_canonicalizer::to_vec(&value).unwrap();
        assert_eq!(
            OrchestrationHandoffV1::from_canonical_json(&unknown),
            Err(HostHandoffError::InvalidJson)
        );

        let pretty = serde_json::to_vec_pretty(&handoff).unwrap();
        assert_eq!(
            OrchestrationHandoffV1::from_canonical_json(&pretty),
            Err(HostHandoffError::NonCanonicalJson)
        );
    }

    #[test]
    fn debug_output_redacts_instructions_candidate_and_disclosures() {
        let handoff = handoff();
        let candidate = HostCandidateEnvelopeV1::try_new(
            &handoff,
            "private candidate canary",
            vec![evidence()],
            vec!["private conflict canary".to_owned()],
            vec!["private gap canary".to_owned()],
        )
        .unwrap();
        let handoff_debug = format!("{handoff:?}");
        let candidate_debug = format!("{candidate:?}");
        assert!(!handoff_debug.contains("Read the registered project"));
        assert!(!candidate_debug.contains("private candidate canary"));
        assert!(!candidate_debug.contains("private conflict canary"));
        assert!(!candidate_debug.contains("private gap canary"));
    }
}
