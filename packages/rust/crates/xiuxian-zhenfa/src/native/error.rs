//! Native tool dispatch error taxonomy.

use thiserror::Error;

/// Native tool dispatch errors in the zhenfa microkernel.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZhenfaError {
    /// Tool id was not found in the in-process registry.
    #[error("zhenfa tool not found: {tool_id}")]
    NotFound {
        /// Missing tool id.
        tool_id: String,
    },
    /// Caller payload is invalid for the target tool.
    #[error("invalid zhenfa tool arguments: {reason}")]
    InvalidArguments {
        /// Human-readable reason.
        reason: String,
    },
    /// Underlying domain execution failed.
    #[error("zhenfa tool execution failed: {message}")]
    Execution {
        /// Optional domain/system error code (for example JSON-RPC codes).
        code: Option<i32>,
        /// Human-readable failure reason.
        message: String,
    },
}

impl ZhenfaError {
    /// Build a `NotFound` error for one tool id.
    #[must_use]
    pub fn not_found(tool_id: impl Into<String>) -> Self {
        Self::NotFound {
            tool_id: tool_id.into(),
        }
    }

    /// Build an `InvalidArguments` error.
    #[must_use]
    pub fn invalid_arguments(reason: impl Into<String>) -> Self {
        Self::InvalidArguments {
            reason: reason.into(),
        }
    }

    /// Build an `Execution` error.
    #[must_use]
    pub fn execution(message: impl Into<String>) -> Self {
        Self::Execution {
            code: None,
            message: message.into(),
        }
    }

    /// Build an `Execution` error with an explicit domain code.
    #[must_use]
    pub fn execution_with_code(code: i32, message: impl Into<String>) -> Self {
        Self::Execution {
            code: Some(code),
            message: message.into(),
        }
    }

    /// Returns one LLM-safe semantic summary without exposing internal runtime
    /// details such as local paths, backend internals, or transport traces.
    #[must_use]
    pub fn llm_safe_message(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "requested tool is unavailable in the current runtime",
            Self::InvalidArguments { .. } => {
                "tool arguments are invalid; adjust parameters and retry"
            }
            Self::Execution { .. } => {
                "tool execution failed in the current environment; retry with a simpler request"
            }
        }
    }
}
