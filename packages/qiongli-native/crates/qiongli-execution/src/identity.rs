use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::ExecutionError;

macro_rules! bounded_identity {
    ($name:ident, $validator:ident) => {
        #[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, ExecutionError> {
                let value = value.into();
                if !$validator(&value) {
                    return Err(ExecutionError::InvalidIdentity);
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str(stringify!($name))?;
                formatter.write_str("(")?;
                formatter.write_str(&self.0)?;
                formatter.write_str(")")
            }
        }
    };
}

bounded_identity!(BackendId, valid_backend_id);
bounded_identity!(RunId, valid_run_id);
bounded_identity!(ToolCallId, valid_tool_call_id);
bounded_identity!(ToolId, valid_tool_id);

fn valid_backend_id(value: &str) -> bool {
    valid_slug(value, 64)
}

fn valid_tool_id(value: &str) -> bool {
    valid_slug(value, 96)
}

fn valid_slug(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_run_id(value: &str) -> bool {
    valid_prefixed_hex(value, "run_")
}

fn valid_tool_call_id(value: &str) -> bool {
    valid_prefixed_hex(value, "call_")
}

fn valid_prefixed_hex(value: &str, prefix: &str) -> bool {
    value.len() == prefix.len() + 32
        && value.starts_with(prefix)
        && value[prefix.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_closed_and_bounded() {
        assert!(BackendId::parse("openai-direct").is_ok());
        assert!(BackendId::parse("OpenAI direct").is_err());
        assert!(ToolId::parse("project.graph-query").is_ok());
        assert!(RunId::parse(format!("run_{}", "a".repeat(32))).is_ok());
        assert!(RunId::parse(format!("run_{}", "A".repeat(32))).is_err());
        assert!(ToolCallId::parse(format!("call_{}", "b".repeat(32))).is_ok());
    }
}
