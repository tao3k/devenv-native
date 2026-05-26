//! Error model shared by artifact cache key and backend operations.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Error returned by artifact cache contracts and implementations.
#[derive(Debug)]
pub enum ArtifactCacheError {
    /// An artifact key component cannot be represented as a safe storage path
    /// segment.
    InvalidComponent {
        /// Logical field that failed validation.
        field: &'static str,
        /// Caller-provided component value.
        value: String,
        /// Human-readable validation failure.
        reason: &'static str,
    },
    /// A filesystem operation failed while reading or writing cached artifact
    /// bytes.
    Io {
        /// Operation being attempted.
        action: &'static str,
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Original IO error.
        source: std::io::Error,
    },
    /// A backend-specific operation failed.
    Backend {
        /// Backend name.
        backend: &'static str,
        /// Operation being attempted.
        action: &'static str,
        /// Backend error message.
        message: String,
    },
}

impl ArtifactCacheError {
    /// Build an invalid component error for caller-owned key validation.
    #[must_use]
    pub fn invalid_component(
        field: &'static str,
        value: impl Into<String>,
        reason: &'static str,
    ) -> Self {
        Self::InvalidComponent {
            field,
            value: value.into(),
            reason,
        }
    }

    /// Build an IO error for caller-owned artifact byte materialization.
    #[must_use]
    pub fn io(action: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    /// Build a backend error for caller-owned artifact byte materialization.
    #[must_use]
    pub fn backend(
        backend: &'static str,
        action: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self::Backend {
            backend,
            action,
            message: message.into(),
        }
    }
}

impl Display for ArtifactCacheError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidComponent {
                field,
                value,
                reason,
            } => write!(
                formatter,
                "invalid artifact key component `{field}`=`{value}`: {reason}"
            ),
            Self::Io {
                action,
                path,
                source,
            } => write!(
                formatter,
                "artifact cache IO failed while {action} `{}`: {source}",
                path.display()
            ),
            Self::Backend {
                backend,
                action,
                message,
            } => write!(
                formatter,
                "artifact cache backend `{backend}` failed while {action}: {message}"
            ),
        }
    }
}

impl Error for ArtifactCacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidComponent { .. } | Self::Backend { .. } => None,
        }
    }
}
