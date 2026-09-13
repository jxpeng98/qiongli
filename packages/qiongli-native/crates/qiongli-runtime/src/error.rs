use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeErrorCode {
    LiteContractUnavailable,
    LiteContractTooLarge,
    InvalidLiteContract,
    FullProjectContractUnavailable,
    FullProjectContractTooLarge,
    InvalidFullProjectContract,
    McpMessageTooLarge,
    McpHeadersTooLarge,
    InvalidMcpMessage,
    McpPayloadNotUtf8,
    IncompleteMcpMessage,
    McpInputFailed,
    McpOutputFailed,
    McpOutputSerializationFailed,
}

impl RuntimeErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LiteContractUnavailable => "lite-contract-unavailable",
            Self::LiteContractTooLarge => "lite-contract-too-large",
            Self::InvalidLiteContract => "invalid-lite-contract",
            Self::FullProjectContractUnavailable => "full-project-contract-unavailable",
            Self::FullProjectContractTooLarge => "full-project-contract-too-large",
            Self::InvalidFullProjectContract => "invalid-full-project-contract",
            Self::McpMessageTooLarge => "mcp-message-too-large",
            Self::McpHeadersTooLarge => "mcp-headers-too-large",
            Self::InvalidMcpMessage => "invalid-mcp-message",
            Self::McpPayloadNotUtf8 => "mcp-payload-not-utf8",
            Self::IncompleteMcpMessage => "incomplete-mcp-message",
            Self::McpInputFailed => "mcp-input-failed",
            Self::McpOutputFailed => "mcp-output-failed",
            Self::McpOutputSerializationFailed => "mcp-output-serialization-failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeError {
    code: RuntimeErrorCode,
    io_kind: Option<io::ErrorKind>,
}

impl RuntimeError {
    #[must_use]
    pub const fn new(code: RuntimeErrorCode) -> Self {
        Self {
            code,
            io_kind: None,
        }
    }

    pub(crate) const fn input_failed(kind: io::ErrorKind) -> Self {
        Self {
            code: RuntimeErrorCode::McpInputFailed,
            io_kind: Some(kind),
        }
    }

    pub(crate) const fn output_failed(kind: io::ErrorKind) -> Self {
        Self {
            code: RuntimeErrorCode::McpOutputFailed,
            io_kind: Some(kind),
        }
    }

    #[must_use]
    pub const fn code(&self) -> RuntimeErrorCode {
        self.code
    }

    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        self.code.as_str()
    }

    #[must_use]
    pub const fn io_kind(&self) -> Option<io::ErrorKind> {
        self.io_kind
    }

    #[must_use]
    pub fn into_io_error(self) -> io::Error {
        let kind = self.io_kind.unwrap_or(match self.code {
            RuntimeErrorCode::LiteContractUnavailable
            | RuntimeErrorCode::FullProjectContractUnavailable => io::ErrorKind::NotFound,
            RuntimeErrorCode::IncompleteMcpMessage => io::ErrorKind::UnexpectedEof,
            RuntimeErrorCode::McpOutputSerializationFailed => io::ErrorKind::Other,
            RuntimeErrorCode::LiteContractTooLarge
            | RuntimeErrorCode::InvalidLiteContract
            | RuntimeErrorCode::FullProjectContractTooLarge
            | RuntimeErrorCode::InvalidFullProjectContract
            | RuntimeErrorCode::McpMessageTooLarge
            | RuntimeErrorCode::McpHeadersTooLarge
            | RuntimeErrorCode::InvalidMcpMessage
            | RuntimeErrorCode::McpPayloadNotUtf8 => io::ErrorKind::InvalidData,
            RuntimeErrorCode::McpInputFailed | RuntimeErrorCode::McpOutputFailed => {
                io::ErrorKind::Other
            }
        });
        io::Error::new(kind, self.reason_code())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for RuntimeError {}
