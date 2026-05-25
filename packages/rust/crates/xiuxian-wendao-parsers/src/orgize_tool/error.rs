//! Shared error types for Orgize-backed tooling.

use std::path::PathBuf;

use orgize::ast::BabelEvalPlanError;
use thiserror::Error;

/// Errors returned by Orgize-backed Wendao tooling.
#[derive(Debug, Error)]
pub enum OrgizeToolError {
    /// A path cannot be read, written, or inspected.
    #[error("{path}: {source}")]
    Io {
        /// Path associated with the filesystem error.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// A supplied path is not an Org file.
    #[error("{path}: expected .org file")]
    NotOrgFile {
        /// Path that failed the `.org` extension check.
        path: PathBuf,
    },
    /// A supplied path is neither a regular file nor a directory.
    #[error("{path}: unsupported path type")]
    UnsupportedPath {
        /// Unsupported path.
        path: PathBuf,
    },
    /// A date does not use the supported `YYYY-MM-DD` form.
    #[error("invalid date `{value}`; expected YYYY-MM-DD")]
    InvalidDate {
        /// Raw date value.
        value: String,
    },
    /// A priority flag value is invalid.
    #[error("unsupported priority value `{value}`")]
    InvalidPriority {
        /// Raw priority value.
        value: String,
    },
    /// Priority profile bounds are not a valid Org priority profile.
    #[error(
        "priority profile must use one priority family and satisfy highest <= default <= lowest"
    )]
    InvalidPriorityProfile,
    /// An Org agenda match expression failed to parse.
    #[error("invalid agenda match expression `{expression}`: {message}")]
    InvalidMatchExpression {
        /// Raw match expression.
        expression: String,
        /// Parser diagnostic.
        message: String,
    },
    /// An agent task heading is missing its required Org section ID.
    #[error("{path}:{line}: agent task `{title}` is missing required :ID: property")]
    MissingAgentOrgid {
        /// Source Org file.
        path: PathBuf,
        /// One-based source line.
        line: usize,
        /// Heading title.
        title: String,
    },
    /// A named Org Babel eval block cannot be resolved.
    #[error("{name}: {message}", message = eval_plan_error_message(reason))]
    EvalPlan {
        /// Requested `#+NAME:` value.
        name: String,
        /// Upstream Orgize eval-plan error.
        reason: BabelEvalPlanError,
    },
}

fn eval_plan_error_message(error: &BabelEvalPlanError) -> String {
    match error {
        BabelEvalPlanError::EmptyName => "eval name is empty".to_string(),
        BabelEvalPlanError::NotFound { name } => format!("eval block `{name}` not found"),
        BabelEvalPlanError::Ambiguous { name, matches } => {
            format!("eval block `{name}` is ambiguous ({matches} matches)")
        }
    }
}
