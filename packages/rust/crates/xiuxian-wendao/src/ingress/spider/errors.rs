//! `ingress::spider::errors` owns Wendao ingress spider errors behavior.

use thiserror::Error;

/// Spider ingress pipeline errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpiderIngressError {
    /// URL is not a valid absolute web URL.
    #[error("invalid web URL `{url}`")]
    InvalidWebUrl {
        /// Invalid URL value.
        url: String,
    },
    /// URL scheme is unsupported for ingress.
    #[error("unsupported web URL scheme `{scheme}` for `{url}`")]
    UnsupportedWebScheme {
        /// Input URL.
        url: String,
        /// Scheme extracted from URL.
        scheme: String,
    },
    /// Zhenfa transmutation failed.
    #[error("zhenfa transmutation failed for `{uri}`: {reason}")]
    TransmutationFailed {
        /// Canonical `wendao://web/...` URI.
        uri: String,
        /// Sanitized error message.
        reason: String,
    },
    /// Assimilation sink failed.
    #[error("web assimilation failed for `{uri}`: {reason}")]
    AssimilationFailed {
        /// Canonical `wendao://web/...` URI.
        uri: String,
        /// Sink error details.
        reason: String,
    },
    /// Partial re-index hook failed.
    #[error("partial re-index failed for namespace `{namespace}`: {reason}")]
    PartialReindexFailed {
        /// Namespace selected for refresh.
        namespace: String,
        /// Hook error details.
        reason: String,
    },
    /// Internal ingestion lock is poisoned.
    #[error("ingestion state lock poisoned")]
    StateLockPoisoned,
}
