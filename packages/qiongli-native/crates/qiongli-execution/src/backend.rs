use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    AGENT_BACKEND_PROTOCOL_VERSION, AgentBackendError, AgentBackendErrorCode, BackendId,
    CancellationToken, ExecutionError, RunId, ToolCallId,
};

const MAX_MODEL_ID_BYTES: usize = 160;
const MAX_MESSAGES: usize = 256;
const MAX_MESSAGE_BYTES: usize = 256 * 1024;
const MAX_TOTAL_INPUT_BYTES: usize = 2 * 1024 * 1024;
const MAX_ATTACHMENTS: usize = 16;
const MAX_ATTACHMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_TOOLS: usize = 128;
const MAX_TOOL_SCHEMA_BYTES: usize = 128 * 1024;
const MAX_REASON_CODES: usize = 16;
const MAX_EVENT_CONTENT_BYTES: usize = 64 * 1024;
const MAX_EVENT_ARGUMENT_BYTES: usize = 1024 * 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

pub type AgentBackendFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait AgentBackend: Send + Sync {
    fn descriptor(&self) -> AgentBackendDescriptorV1;

    fn start<'a>(
        &'a self,
        request: AgentRequestV1,
        cancellation: CancellationToken,
    ) -> AgentBackendFuture<'a, Result<Box<dyn AgentEventStream>, AgentBackendError>>;
}

pub trait AgentEventStream: Send {
    fn next_event<'a>(
        &'a mut self,
        cancellation: &'a CancellationToken,
    ) -> AgentBackendFuture<'a, Option<Result<AgentEventV1, AgentBackendError>>>;
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentAuthState {
    Ready,
    NotRequired,
    MissingCredential,
    Expired,
    Unavailable,
}

impl AgentAuthState {
    #[must_use]
    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready | Self::NotRequired)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRetryClass {
    RateLimited,
    NetworkTransient,
    ProviderBusy,
    ServerError,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentCancellationSemantics {
    Unsupported,
    Cooperative,
    ProviderConfirmed,
}

impl AgentCancellationSemantics {
    #[must_use]
    pub const fn is_supported(self) -> bool {
        !matches!(self, Self::Unsupported)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentBackendCapabilitiesV1 {
    pub maximum_context_tokens: u32,
    pub maximum_output_tokens: u32,
    pub streaming: bool,
    pub structured_output: bool,
    pub tool_calls: bool,
    pub multimodal: bool,
    pub cancellation: AgentCancellationSemantics,
    pub retry_classes: BTreeSet<AgentRetryClass>,
}

impl AgentBackendCapabilitiesV1 {
    fn validate(&self) -> Result<(), ExecutionError> {
        if self.maximum_context_tokens == 0
            || self.maximum_output_tokens == 0
            || self.maximum_output_tokens > self.maximum_context_tokens
        {
            return Err(ExecutionError::InvalidBackendDescriptor);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentBackendDescriptorV1 {
    pub schema_version: u32,
    pub backend_id: BackendId,
    pub protocol_version: String,
    pub authentication: AgentAuthState,
    pub models: Vec<String>,
    pub capabilities: AgentBackendCapabilitiesV1,
    pub host_constraint_codes: Vec<String>,
}

impl AgentBackendDescriptorV1 {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        if self.schema_version != 1
            || self.protocol_version != AGENT_BACKEND_PROTOCOL_VERSION
            || self.models.is_empty()
            || self.models.len() > 128
            || self.models.iter().any(|model| !valid_model_id(model))
            || !strictly_sorted_unique(&self.models)
            || self.host_constraint_codes.len() > 32
            || self
                .host_constraint_codes
                .iter()
                .any(|code| !valid_reason_code(code))
        {
            return Err(ExecutionError::InvalidBackendDescriptor);
        }
        self.capabilities.validate()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentMessageV1 {
    pub role: AgentRole,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<ToolCallId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentAttachmentV1 {
    pub attachment_id: String,
    pub media_type: String,
    pub sha256: String,
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentToolSchemaV1 {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentResponseConstraintsV1 {
    pub maximum_output_tokens: u32,
    pub structured_output_schema: Option<Value>,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRequestV1 {
    pub schema_version: u32,
    pub run_id: RunId,
    pub model: String,
    pub messages: Vec<AgentMessageV1>,
    pub attachments: Vec<AgentAttachmentV1>,
    pub response: AgentResponseConstraintsV1,
    pub tools: Vec<AgentToolSchemaV1>,
}

impl AgentRequestV1 {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        let message_bytes = self
            .messages
            .iter()
            .try_fold(0_usize, |total, message| {
                if message.content.is_empty()
                    || message.content.len() > MAX_MESSAGE_BYTES
                    || (message.role == AgentRole::Tool) != message.tool_call_id.is_some()
                {
                    None
                } else {
                    total.checked_add(message.content.len())
                }
            })
            .ok_or(ExecutionError::InvalidAgentRequest)?;
        let attachment_bytes = self
            .attachments
            .iter()
            .try_fold(0_usize, |total, attachment| {
                if !valid_attachment(attachment) {
                    None
                } else {
                    total.checked_add(attachment.content.len())
                }
            })
            .ok_or(ExecutionError::InvalidAgentRequest)?;
        if self.schema_version != 1
            || !valid_model_id(&self.model)
            || self.messages.is_empty()
            || self.messages.len() > MAX_MESSAGES
            || self.attachments.len() > MAX_ATTACHMENTS
            || attachment_bytes > MAX_ATTACHMENT_BYTES
            || message_bytes.saturating_add(attachment_bytes) > MAX_TOTAL_INPUT_BYTES
            || self.response.maximum_output_tokens == 0
            || self.tools.len() > MAX_TOOLS
            || self.tools.iter().any(|tool| !valid_tool_schema(tool))
            || !unique_tool_names(&self.tools)
            || self
                .response
                .structured_output_schema
                .as_ref()
                .is_some_and(|schema| {
                    !schema.is_object() || json_size(schema) > MAX_TOOL_SCHEMA_BYTES
                })
        {
            return Err(ExecutionError::InvalidAgentRequest);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentRequirementsV1 {
    pub minimum_context_tokens: u32,
    pub streaming: bool,
    pub structured_output: bool,
    pub tool_calls: bool,
    pub multimodal: bool,
    pub cancellation: bool,
}

impl AgentRequirementsV1 {
    fn validate(&self) -> Result<(), ExecutionError> {
        if self.minimum_context_tokens == 0 {
            return Err(ExecutionError::InvalidAgentRequest);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentPreflightV1 {
    pub schema_version: u32,
    pub backend_id: BackendId,
    pub ready: bool,
    pub reason_codes: Vec<String>,
    pub effective_context_tokens: u32,
    pub effective_output_tokens: u32,
}

pub fn preflight_backend(
    descriptor: &AgentBackendDescriptorV1,
    request: &AgentRequestV1,
    requirements: &AgentRequirementsV1,
) -> AgentPreflightV1 {
    let mut reasons = Vec::new();
    if descriptor.validate().is_err() {
        reasons.push("agent-backend-descriptor-invalid".to_string());
    }
    if request.validate().is_err() {
        reasons.push("agent-request-invalid".to_string());
    }
    if requirements.validate().is_err() {
        reasons.push("agent-requirements-invalid".to_string());
    }
    if !descriptor.authentication.is_ready() {
        reasons.push("agent-backend-authentication-unavailable".to_string());
    }
    if !descriptor
        .models
        .iter()
        .any(|model| model == &request.model)
    {
        reasons.push("agent-model-unavailable".to_string());
    }
    let capabilities = &descriptor.capabilities;
    if requirements.minimum_context_tokens > capabilities.maximum_context_tokens {
        reasons.push("agent-context-limit-insufficient".to_string());
    }
    if request.response.maximum_output_tokens > capabilities.maximum_output_tokens {
        reasons.push("agent-output-limit-insufficient".to_string());
    }
    if requirements.streaming && !capabilities.streaming {
        reasons.push("agent-streaming-unavailable".to_string());
    }
    if (requirements.structured_output || request.response.structured_output_schema.is_some())
        && !capabilities.structured_output
    {
        reasons.push("agent-structured-output-unavailable".to_string());
    }
    if (requirements.tool_calls || !request.tools.is_empty()) && !capabilities.tool_calls {
        reasons.push("agent-tool-calls-unavailable".to_string());
    }
    if (requirements.multimodal || !request.attachments.is_empty()) && !capabilities.multimodal {
        reasons.push("agent-multimodal-unavailable".to_string());
    }
    if requirements.cancellation && !capabilities.cancellation.is_supported() {
        reasons.push("agent-cancellation-unavailable".to_string());
    }
    reasons.sort();
    reasons.dedup();
    reasons.truncate(MAX_REASON_CODES);
    AgentPreflightV1 {
        schema_version: 1,
        backend_id: descriptor.backend_id.clone(),
        ready: reasons.is_empty(),
        reason_codes: reasons,
        effective_context_tokens: capabilities.maximum_context_tokens,
        effective_output_tokens: capabilities.maximum_output_tokens,
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReasoningStatus {
    Started,
    Continued,
    Finished,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentToolRequestV1 {
    pub call_id: ToolCallId,
    pub tool_name: String,
    pub arguments: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentUsageV1 {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentFinishReason {
    Stop,
    Length,
    ToolRequest,
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AgentEventV1 {
    ContentDelta {
        content: String,
    },
    ReasoningStatus {
        status: ReasoningStatus,
    },
    ToolRequest {
        request: AgentToolRequestV1,
    },
    Usage {
        usage: AgentUsageV1,
    },
    RetryableError {
        class: AgentRetryClass,
        code: AgentBackendErrorCode,
    },
    TerminalError {
        code: AgentBackendErrorCode,
    },
    Cancelled,
    Completed {
        finish_reason: AgentFinishReason,
    },
}

impl AgentEventV1 {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        let valid = match self {
            Self::ContentDelta { content } => {
                !content.is_empty() && content.len() <= MAX_EVENT_CONTENT_BYTES
            }
            Self::ReasoningStatus { .. } | Self::Cancelled | Self::Completed { .. } => true,
            Self::ToolRequest { request } => {
                valid_tool_name(&request.tool_name)
                    && request.arguments.is_object()
                    && serde_json::to_vec(&request.arguments)
                        .is_ok_and(|bytes| bytes.len() <= MAX_EVENT_ARGUMENT_BYTES)
            }
            Self::Usage { usage } => {
                usage.input_tokens <= MAX_SAFE_INTEGER
                    && usage.output_tokens <= MAX_SAFE_INTEGER
                    && usage.cached_input_tokens <= usage.input_tokens
            }
            Self::RetryableError { code, .. } => {
                matches!(
                    code,
                    AgentBackendErrorCode::TransportUnavailable
                        | AgentBackendErrorCode::ProviderRejected
                )
            }
            Self::TerminalError { .. } => true,
        };
        if valid {
            Ok(())
        } else {
            Err(ExecutionError::InvalidAgentRequest)
        }
    }
}

fn valid_model_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_MODEL_ID_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'\\' | b'\'' | b'\"'))
}

fn valid_attachment(attachment: &AgentAttachmentV1) -> bool {
    !attachment.attachment_id.is_empty()
        && attachment.attachment_id.len() <= 96
        && attachment
            .attachment_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        && attachment.media_type.len() <= 96
        && attachment.media_type.contains('/')
        && valid_lower_hex(&attachment.sha256, 64)
        && !attachment.content.is_empty()
        && format!("{:x}", Sha256::digest(&attachment.content)) == attachment.sha256
}

fn valid_tool_schema(tool: &AgentToolSchemaV1) -> bool {
    valid_tool_name(&tool.name)
        && !tool.description.trim().is_empty()
        && tool.description.len() <= 4 * 1024
        && tool.input_schema.is_object()
        && json_size(&tool.input_schema) <= MAX_TOOL_SCHEMA_BYTES
}

fn valid_tool_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
}

fn unique_tool_names(tools: &[AgentToolSchemaV1]) -> bool {
    tools
        .iter()
        .map(|tool| tool.name.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        == tools.len()
}

fn strictly_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn json_size(value: &Value) -> usize {
    serde_json::to_vec(value).map_or(usize::MAX, |bytes| bytes.len())
}

fn valid_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
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
    use serde_json::json;

    use super::*;

    fn descriptor() -> AgentBackendDescriptorV1 {
        AgentBackendDescriptorV1 {
            schema_version: 1,
            backend_id: BackendId::parse("fake").unwrap(),
            protocol_version: AGENT_BACKEND_PROTOCOL_VERSION.to_string(),
            authentication: AgentAuthState::NotRequired,
            models: vec!["deterministic-v1".to_string()],
            capabilities: AgentBackendCapabilitiesV1 {
                maximum_context_tokens: 32_000,
                maximum_output_tokens: 4_096,
                streaming: true,
                structured_output: true,
                tool_calls: true,
                multimodal: false,
                cancellation: AgentCancellationSemantics::Cooperative,
                retry_classes: BTreeSet::from([AgentRetryClass::ProviderBusy]),
            },
            host_constraint_codes: Vec::new(),
        }
    }

    fn request() -> AgentRequestV1 {
        AgentRequestV1 {
            schema_version: 1,
            run_id: RunId::parse(format!("run_{}", "1".repeat(32))).unwrap(),
            model: "deterministic-v1".to_string(),
            messages: vec![AgentMessageV1 {
                role: AgentRole::User,
                content: "Summarize the bounded evidence.".to_string(),
                tool_call_id: None,
            }],
            attachments: Vec::new(),
            response: AgentResponseConstraintsV1 {
                maximum_output_tokens: 512,
                structured_output_schema: Some(json!({"type": "object"})),
            },
            tools: vec![AgentToolSchemaV1 {
                name: "project.graph-query".to_string(),
                description: "Query the registered academic graph.".to_string(),
                input_schema: json!({"type": "object"}),
            }],
        }
    }

    #[test]
    fn preflight_accepts_a_capability_complete_request() {
        let preflight = preflight_backend(
            &descriptor(),
            &request(),
            &AgentRequirementsV1 {
                minimum_context_tokens: 8_000,
                streaming: true,
                structured_output: true,
                tool_calls: true,
                multimodal: false,
                cancellation: true,
            },
        );
        assert!(preflight.ready);
        assert!(preflight.reason_codes.is_empty());
    }

    #[test]
    fn preflight_reports_every_missing_capability_before_execution() {
        let mut descriptor = descriptor();
        descriptor.authentication = AgentAuthState::MissingCredential;
        descriptor.capabilities.streaming = false;
        descriptor.capabilities.structured_output = false;
        descriptor.capabilities.tool_calls = false;
        descriptor.capabilities.multimodal = false;
        descriptor.capabilities.cancellation = AgentCancellationSemantics::Unsupported;
        let mut request = request();
        request.attachments.push(AgentAttachmentV1 {
            attachment_id: "figure-1".to_string(),
            media_type: "image/png".to_string(),
            sha256: format!("{:x}", Sha256::digest([1, 2, 3])),
            content: vec![1, 2, 3],
        });
        let result = preflight_backend(
            &descriptor,
            &request,
            &AgentRequirementsV1 {
                minimum_context_tokens: 64_000,
                streaming: true,
                structured_output: true,
                tool_calls: true,
                multimodal: true,
                cancellation: true,
            },
        );
        assert!(!result.ready);
        assert!(
            result
                .reason_codes
                .contains(&"agent-backend-authentication-unavailable".to_string())
        );
        assert!(
            result
                .reason_codes
                .contains(&"agent-tool-calls-unavailable".to_string())
        );
        assert!(
            result
                .reason_codes
                .contains(&"agent-cancellation-unavailable".to_string())
        );
    }

    #[test]
    fn request_bounds_messages_attachments_and_schemas() {
        let mut request = request();
        assert!(request.validate().is_ok());
        let legacy_message: AgentMessageV1 = serde_json::from_value(json!({
            "role": "user",
            "content": "Backward-compatible v1 message"
        }))
        .unwrap();
        assert_eq!(legacy_message.tool_call_id, None);
        request.messages[0].content = "x".repeat(MAX_MESSAGE_BYTES + 1);
        assert_eq!(request.validate(), Err(ExecutionError::InvalidAgentRequest));
    }

    #[test]
    fn attachment_digest_and_provider_events_are_revalidated() {
        let mut request = request();
        request.attachments.push(AgentAttachmentV1 {
            attachment_id: "figure-1".to_string(),
            media_type: "image/png".to_string(),
            sha256: "a".repeat(64),
            content: vec![1, 2, 3],
        });
        assert_eq!(request.validate(), Err(ExecutionError::InvalidAgentRequest));

        assert_eq!(
            AgentEventV1::ToolRequest {
                request: AgentToolRequestV1 {
                    call_id: ToolCallId::parse(format!("call_{}", "9".repeat(32))).unwrap(),
                    tool_name: "project.graph-query".to_string(),
                    arguments: json!(["not", "an", "object"]),
                },
            }
            .validate(),
            Err(ExecutionError::InvalidAgentRequest)
        );
        assert_eq!(
            AgentEventV1::Usage {
                usage: AgentUsageV1 {
                    input_tokens: 1,
                    output_tokens: 1,
                    cached_input_tokens: 2,
                },
            }
            .validate(),
            Err(ExecutionError::InvalidAgentRequest)
        );
    }
}
