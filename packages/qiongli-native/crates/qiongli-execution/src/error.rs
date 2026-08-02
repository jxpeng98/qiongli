use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::AgentRetryClass;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionError {
    InvalidIdentity,
    InvalidBackendDescriptor,
    InvalidAgentRequest,
    InvalidToolRegistration,
    InvalidToolRequest,
    InvalidApproval,
    InvalidPolicy,
    PolicyDenied,
    ApprovalRequired,
    ToolNotRegistered,
    ToolHostContractInvalid,
}

impl ExecutionError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidIdentity => "execution-identity-invalid",
            Self::InvalidBackendDescriptor => "agent-backend-descriptor-invalid",
            Self::InvalidAgentRequest => "agent-request-invalid",
            Self::InvalidToolRegistration => "tool-registration-invalid",
            Self::InvalidToolRequest => "tool-request-invalid",
            Self::InvalidApproval => "tool-approval-invalid",
            Self::InvalidPolicy => "execution-policy-invalid",
            Self::PolicyDenied => "tool-policy-denied",
            Self::ApprovalRequired => "tool-approval-required",
            Self::ToolNotRegistered => "tool-not-registered",
            Self::ToolHostContractInvalid => "tool-host-contract-invalid",
        }
    }
}

impl Display for ExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for ExecutionError {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentBackendErrorCode {
    InvalidRequest,
    AuthenticationUnavailable,
    CapabilityUnavailable,
    TransportUnavailable,
    ProviderRejected,
    ResponseInvalid,
    Cancelled,
}

impl AgentBackendErrorCode {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "agent-request-invalid",
            Self::AuthenticationUnavailable => "agent-backend-authentication-unavailable",
            Self::CapabilityUnavailable => "agent-backend-capability-unavailable",
            Self::TransportUnavailable => "agent-backend-transport-unavailable",
            Self::ProviderRejected => "agent-backend-provider-rejected",
            Self::ResponseInvalid => "agent-backend-response-invalid",
            Self::Cancelled => "agent-run-cancelled",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentBackendError {
    pub code: AgentBackendErrorCode,
    pub retry_class: Option<AgentRetryClass>,
}

impl AgentBackendError {
    #[must_use]
    pub const fn new(code: AgentBackendErrorCode, retry_class: Option<AgentRetryClass>) -> Self {
        Self { code, retry_class }
    }

    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        self.code.reason_code()
    }
}

impl Display for AgentBackendError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for AgentBackendError {}
