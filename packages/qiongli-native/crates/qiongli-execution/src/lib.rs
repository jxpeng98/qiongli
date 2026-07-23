//! Native model transport and local tool-execution trust boundary.
//!
//! Provider adapters implement [`AgentBackend`] and receive only normalized
//! model input plus policy-selected tool schemas. Local authority stays behind
//! [`AgentExecutionPolicy`] and [`ToolHostRegistry`].

mod backend;
mod control;
mod dispatch;
mod error;
mod fake;
mod identity;
mod openai;
mod policy;
mod runner;
mod tool_host;

pub use backend::{
    AgentAttachmentV1, AgentAuthState, AgentBackend, AgentBackendCapabilitiesV1,
    AgentBackendDescriptorV1, AgentBackendFuture, AgentCancellationSemantics, AgentEventStream,
    AgentEventV1, AgentFinishReason, AgentMessageV1, AgentPreflightV1, AgentRequestV1,
    AgentRequirementsV1, AgentResponseConstraintsV1, AgentRetryClass, AgentRole,
    AgentToolRequestV1, AgentToolSchemaV1, AgentUsageV1, ReasoningStatus, preflight_backend,
};
pub use control::{
    BACKEND_CONTROL_SCHEMA_VERSION, BackendConnectionTestOutcomeV1, BackendConnectionTestV1,
    BackendControlError, BackendControlService, BackendReadinessV1, BackendStatusV1,
    openai_backend_status,
};
pub use dispatch::{InProcessToolHost, ReadOnlyToolRequest, ReadOnlyToolService, ToolServiceError};
pub use error::{AgentBackendError, AgentBackendErrorCode, ExecutionError};
pub use fake::DeterministicFakeBackend;
pub use identity::{BackendId, RunId, ToolCallId, ToolId};
pub use openai::{OpenAiBackendConfigV1, OpenAiResponsesBackend};
pub use policy::{
    AgentExecutionPolicy, ApprovalActor, ApprovalGrantV1, ExecutionLimitsV1, ExecutionProfile,
    ExecutionUsageV1, PolicyDecisionV1, PolicyOutcome, PolicyReasonCode, PolicyToolRequestV1,
    ProjectExecutionScope, RedactionPolicyV1, ToolClass, ToolExecutionKind, ToolRegistrationV1,
};
pub use runner::{
    AgentRunError, AgentRunInputV1, AgentRunResultV1, BOUNDED_AGENT_RUN_SCHEMA_VERSION,
    BoundedAgentRunner,
};
pub use tool_host::{
    CancellationToken, ToolAuditOutcome, ToolAuditRecordV1, ToolHostInvocationV1, ToolHostRegistry,
    ToolHostResultInput, ToolHostResultV1, ToolResultStatus,
};

pub const AGENT_BACKEND_PROTOCOL_VERSION: &str = "qiongli-agent-backend/1";
pub const TOOL_HOST_PROTOCOL_VERSION: &str = "qiongli-tool-host/1";
