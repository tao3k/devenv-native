//! Error surface for `qianji-client`.

use thiserror::Error;

/// Error type for Qianji client command parsing and validation.
#[derive(Debug, Error)]
pub enum QianjiClientError {
    /// User input, filesystem, or contract validation failed.
    #[error("{0}")]
    Message(String),
}

impl QianjiClientError {
    pub(crate) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
