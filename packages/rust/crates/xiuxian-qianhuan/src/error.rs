//! Error type for Qianhuan prompt-injection rendering and parsing.

use thiserror::Error;

/// Error during prompt context injection.
#[derive(Debug, Error, PartialEq)]
pub enum InjectionError {
    /// The requested template name was not found in the template registry.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    /// Template rendering failed due to invalid context or rendering runtime error.
    #[error("Render failed: {0}")]
    RenderFailed(String),
    /// Internal unexpected failure.
    #[error("Internal error: {0}")]
    Internal(String),
    /// Context quality is below minimum threshold.
    #[error("Context insufficient: CCS={ccs}. Missing: {missing_info}")]
    ContextInsufficient {
        /// Computed context confidence score.
        ccs: f64,
        /// Human-readable missing context summary.
        missing_info: String,
    },
    /// XML payload or intermediate XML validation failed.
    #[error("XML validation error: {0}")]
    XmlValidationError(String),
    /// Input payload is empty.
    #[error("Empty payload")]
    EmptyPayload,
    /// Required QA block is missing.
    #[error("Missing QA block")]
    MissingQaBlock,
    /// Required question field is missing.
    #[error("Missing question")]
    MissingQuestion,
    /// Required answer field is missing.
    #[error("Missing answer")]
    MissingAnswer,
}

/// Convenience result alias for manifestation operations.
pub(crate) type Result<T> = std::result::Result<T, InjectionError>;
