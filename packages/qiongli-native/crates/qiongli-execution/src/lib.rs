//! Native model transport and local tool-execution trust boundary.
//!
//! Provider adapters implement [`AgentBackend`] and receive only normalized
//! model input plus policy-selected tool schemas. Local authority stays behind
//! [`AgentExecutionPolicy`] and [`ToolHostRegistry`].

mod acp;
mod all_chat;
mod artifact_review;
mod backend;
mod control;
mod dispatch;
mod error;
mod fake;
mod host_acceptance;
mod host_handoff;
mod identity;
mod openai;
mod orchestration;
mod orchestration_input;
mod orchestration_runtime;
mod policy;
mod runner;
mod tool_host;
mod worker_orchestration;
mod worker_orchestration_input;
mod worker_orchestration_runtime;

pub use acp::{AcpDevelopmentPresetV1, AcpV1Client, AcpV1TurnOutcome};
pub use all_chat::{
    ALL_CHAT_STATE_SCHEMA_VERSION, AllChatEventKindV1, AllChatEventV1, AllChatParticipantV1,
    AllChatStateError, AllChatStateV1,
};
pub use artifact_review::{
    ARTIFACT_REVIEW_SCHEMA_VERSION, ArtifactCandidateOperation, ArtifactCandidateV1,
    ArtifactReviewCheckpointV1, ArtifactReviewError, ArtifactReviewPlanV1, ArtifactReviewRunStatus,
    ArtifactReviewSourceKind, QualityGateCheckpointV1, QualityGateId, QualityGateStatus,
    ReviewerVerdict,
};
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
    openai_backend_metadata_status, openai_backend_status,
};
pub use dispatch::{InProcessToolHost, ReadOnlyToolRequest, ReadOnlyToolService, ToolServiceError};
pub use error::{AgentBackendError, AgentBackendErrorCode, ExecutionError};
pub use fake::DeterministicFakeBackend;
pub use host_acceptance::{
    HOST_ACCEPTANCE_RECORD_TYPE, HOST_ACCEPTANCE_SCHEMA_VERSION, HostAcceptanceCandidateContractV1,
    HostAcceptanceCheckpointTransitionV1, HostAcceptanceError, HostAcceptanceFactV1,
    HostAcceptanceFixtureV1, HostAcceptanceProfileScopeV1, HostAcceptanceReceiptV1,
    HostAcceptanceRejectionContractV1, HostAcceptanceStatusV1, HostAcceptanceTransitionV1,
    HostAcceptanceVerdictV1,
};
pub use host_handoff::{
    FULL_MCP_HOST_PROTOCOL_VERSION, HOST_CANDIDATE_SCHEMA_VERSION, HOST_HANDOFF_PROTOCOL_VERSION,
    HOST_HANDOFF_SCHEMA_VERSION, HostCandidateEnvelopeV1, HostCandidateKindV1, HostCapabilityV1,
    HostComponentStateV1, HostEvidenceReferenceV1, HostExecutionLimitsV1, HostFamilyV1,
    HostHandoffError, HostReviewResultV1, HostRuntimeDescriptorV1, OrchestrationHandoffV1,
};
pub use identity::{
    BackendId, OrchestrationProfileId, OrchestrationTaskId, RunId, ToolCallId, ToolId, WorkerId,
};
pub use openai::{OpenAiBackendConfigV1, OpenAiResponsesBackend};
pub use orchestration::{
    ORCHESTRATION_SCHEMA_VERSION, ORCHESTRATION_WORKFLOW_SOURCE_PATH, OrchestrationCheckpointV1,
    OrchestrationError, OrchestrationExecutionMode, OrchestrationFailureCode, OrchestrationPlanV1,
    OrchestrationProfileV1, OrchestrationRole, OrchestrationRunStatus, OrchestrationTaskGraphV1,
    OrchestrationTaskSpecV1, OrchestrationTaskState, RoleCheckpointV1, TaskCheckpointV1,
};
pub use orchestration_input::{
    EmbeddedWorkflowHostHandoffBuilder, EmbeddedWorkflowRoleInputBuilder, HostRolePacketV1,
};
pub use orchestration_runtime::{
    DiscoveredOrchestrationRunV1, OrchestrationCheckpointStore, OrchestrationRoleInputBuilder,
    OrchestrationRoleInputContextV1, OrchestrationRoleInputError, OrchestrationRoleResultV1,
    OrchestrationRuntimeError, OrchestrationStepOutcome, OrchestrationTaskExecutor,
    OrchestrationTaskRunResultV1, PersistedOrchestrationCheckpointV1,
};
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
pub use worker_orchestration::{
    WORKER_ORCHESTRATION_SCHEMA_VERSION, WorkerBarrierFailurePolicy, WorkerBarrierStatus,
    WorkerCheckpointV1, WorkerMergePolicy, WorkerOrchestrationCheckpointV1,
    WorkerOrchestrationError, WorkerOrchestrationFailureCode, WorkerOrchestrationMode,
    WorkerOrchestrationPlanV1, WorkerOrchestrationRunStatus, WorkerSpecV1, WorkerStatus,
};
pub use worker_orchestration_input::EmbeddedWorkerOrchestrationInputBuilder;
pub use worker_orchestration_runtime::{
    DiscoveredWorkerOrchestrationRunV1, PersistedWorkerOrchestrationCheckpointV1,
    WorkerOrchestrationAgentPhase, WorkerOrchestrationAgentResultV1,
    WorkerOrchestrationCheckpointStore, WorkerOrchestrationExecutor,
    WorkerOrchestrationInputBuilder, WorkerOrchestrationInputContextV1,
    WorkerOrchestrationInputError, WorkerOrchestrationRunResultV1, WorkerOrchestrationRuntimeError,
    WorkerOrchestrationStepOutcome,
};

pub const AGENT_BACKEND_PROTOCOL_VERSION: &str = "qiongli-agent-backend/1";
pub const TOOL_HOST_PROTOCOL_VERSION: &str = "qiongli-tool-host/1";
