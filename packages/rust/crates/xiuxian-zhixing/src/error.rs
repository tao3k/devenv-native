//! Shared error type and result alias for Zhixing workflows.

use thiserror::Error;

/// Core error type for the Xiuxian-Zhixing system.
#[derive(Debug, Error)]
pub enum Error {
    /// Error in internal logic or state management.
    #[error("Zhixing Logic Error: {0}")]
    Logic(String),

    /// Error related to external service or library interaction.
    #[error("Zhixing Internal Error: {0}")]
    Internal(String),

    /// Missing dependency or configuration.
    #[error("Zhixing Config Error: {0}")]
    Config(String),
}

/// Specialized Result type.
pub type Result<T> = std::result::Result<T, Error>;
