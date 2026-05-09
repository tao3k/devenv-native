//! Structured error types for LLM runtime paths.

use reqwest::StatusCode;
use thiserror::Error;

/// Unified result type for LLM module operations.
pub type LlmResult<T> = Result<T, LlmError>;

/// HTTP content type reported by an upstream provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpContentType(String);

impl HttpContentType {
    /// Creates a provider content-type label.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw content-type label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HttpContentType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Structured errors produced by LLM runtime clients and utilities.
#[derive(Debug, Error)]
pub enum LlmError {
    /// HTTP connection/request setup failure.
    #[error("LLM connection failed")]
    ConnectionFailed {
        /// Upstream transport error.
        #[source]
        source: reqwest::Error,
    },
    /// Failed to read response body.
    #[error("LLM response body read failed")]
    ResponseBodyReadFailed {
        /// Upstream body-read error.
        #[source]
        source: reqwest::Error,
    },
    /// Timed out waiting for provider response headers.
    #[error("LLM response headers were not received within {timeout_secs}s")]
    ResponseHeadersTimedOut {
        /// Header wait timeout in seconds.
        timeout_secs: u64,
    },
    /// Non-success status from provider.
    #[error("LLM request failed with status {status} (content-type: {content_type}): {reason}")]
    RequestFailed {
        /// HTTP status code from provider.
        status: StatusCode,
        /// Response content type.
        content_type: HttpContentType,
        /// Sanitized provider reason.
        reason: String,
    },
    /// Failed to decode provider response body.
    #[error(
        "LLM response decoding failed (status={status}, content-type={content_type}, body_preview={body_preview})"
    )]
    ResponseDecodingFailed {
        /// HTTP status code from provider.
        status: StatusCode,
        /// Response content type.
        content_type: HttpContentType,
        /// Sanitized body preview.
        body_preview: String,
        /// JSON decode error.
        #[source]
        source: serde_json::Error,
    },
    /// Provider response does not contain text content.
    #[error("LLM response did not contain text content")]
    EmptyTextChoice,
    /// Invalid image source format.
    #[error("image reference must be a data URI or http(s) URL")]
    InvalidImageReference,
    /// Image download request failed.
    #[error("image download request failed")]
    ImageDownloadRequestFailed {
        /// HTTP request error.
        #[source]
        source: reqwest::Error,
    },
    /// Image download returned non-success status.
    #[error("image download failed with status {status}")]
    ImageDownloadFailed {
        /// HTTP status code from image source.
        status: StatusCode,
    },
    /// Failed to read image bytes.
    #[error("image payload read failed")]
    ImageBytesReadFailed {
        /// Body read error.
        #[source]
        source: reqwest::Error,
    },
    /// Image source returned empty body.
    #[error("image download returned an empty body")]
    ImageEmptyBody,
    /// Provider backend is unavailable in current feature build.
    #[error("LLM provider `{provider}` is unavailable in this build")]
    ProviderUnavailable {
        /// Provider name.
        provider: &'static str,
    },
    /// Provider initialization failure.
    #[error("LLM provider `{provider}` initialization failed: {reason}")]
    ProviderInitializationFailed {
        /// Provider name.
        provider: &'static str,
        /// Sanitized failure reason.
        reason: String,
    },
    /// Provider registry definition is missing.
    #[error("LLM provider `{provider}` registry entry is missing: {reason}")]
    ProviderRegistryMissing {
        /// Provider name.
        provider: &'static str,
        /// Missing-definition reason.
        reason: String,
    },
    /// Generic sanitized runtime error used by tests and adapters.
    #[error("{message}")]
    Internal {
        /// Sanitized message.
        message: String,
    },
}

/// Redact URLs and secret-looking tokens for user-facing error messages.
#[must_use]
pub fn sanitize_user_visible(input: &str) -> String {
    const MAX_LEN: usize = 320;
    let compact = input
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .map(redact_token)
        .collect::<Vec<_>>()
        .join(" ");
    if compact.len() <= MAX_LEN {
        compact
    } else {
        format!("{}...", &compact[..MAX_LEN])
    }
}

fn redact_token(token: &str) -> String {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if looks_like_url(trimmed) {
        return "[redacted-url]".to_string();
    }
    if looks_like_secret(trimmed) {
        return "[redacted-secret]".to_string();
    }
    trimmed.to_string()
}

fn looks_like_url(token: &str) -> bool {
    let normalized = token
        .trim_matches(|ch: char| ['"', '\'', ',', ';', ')', ']', '>'].contains(&ch))
        .to_ascii_lowercase();
    normalized.starts_with("http://") || normalized.starts_with("https://")
}

fn looks_like_secret(token: &str) -> bool {
    let normalized = token.trim_matches(|ch: char| ['"', '\'', ',', ';'].contains(&ch));
    if normalized.starts_with("sk-")
        || normalized.starts_with("rk-")
        || normalized.starts_with("pk-")
    {
        return true;
    }
    normalized.len() >= 24
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}
