//! Canonical cross-language payload boundary for Wendao transport.

use std::fmt;

/// Canonical data-plane token for cross-language Arrow Flight exchange.
pub const WENDAO_ARROW_FLIGHT_DATA_PLANE: &str = "arrow-flight";

/// Canonical table payload token carried by the Arrow Flight data plane.
pub const WENDAO_ARROW_RECORD_BATCH_PAYLOAD: &str = "arrow-record-batch";

/// JSON control messages may coordinate work but must not carry Arrow tables.
pub const WENDAO_JSON_CONTROL_PLANE: &str = "json-control";

/// JSONL stdio control messages may coordinate long-lived bridge sessions only.
pub const WENDAO_JSONL_STDIO_CONTROL_PLANE: &str = "jsonl-stdio-control";

/// Process arguments may configure a local bridge process only.
pub const WENDAO_PROCESS_ARGS_CONTROL_PLANE: &str = "process-args-control";

/// REST metadata may select or describe routes only.
pub const WENDAO_REST_METADATA_CONTROL_PLANE: &str = "rest-metadata-control";

/// Arrow payload wrapper tokens that are forbidden across Wendao boundaries.
pub const WENDAO_FORBIDDEN_ARROW_PAYLOAD_WRAPPERS: &[&str] = &[
    "base64-arrow-ipc",
    "json-base64-arrow-ipc",
    "jsonl-base64-arrow-ipc",
];

/// High-level role for a cross-language payload surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoPayloadPlane {
    /// Data-plane surfaces carry table payloads.
    Data,
    /// Control-plane surfaces coordinate routing, sessions, or metadata only.
    Control,
}

/// Canonical surfaces allowed at the Wendao cross-language boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoPayloadSurface {
    /// Runtime-owned Arrow Flight route.
    ArrowFlight,
    /// In-process or already-decoded Arrow record batch.
    ArrowRecordBatch,
    /// JSON control receipt or request metadata.
    JsonControl,
    /// JSONL stdio control stream.
    JsonlStdioControl,
    /// Process argument control surface.
    ProcessArgsControl,
    /// REST metadata control surface.
    RestMetadataControl,
}

impl WendaoPayloadSurface {
    /// Return the canonical token used in reports and package docs.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::ArrowFlight => WENDAO_ARROW_FLIGHT_DATA_PLANE,
            Self::ArrowRecordBatch => WENDAO_ARROW_RECORD_BATCH_PAYLOAD,
            Self::JsonControl => WENDAO_JSON_CONTROL_PLANE,
            Self::JsonlStdioControl => WENDAO_JSONL_STDIO_CONTROL_PLANE,
            Self::ProcessArgsControl => WENDAO_PROCESS_ARGS_CONTROL_PLANE,
            Self::RestMetadataControl => WENDAO_REST_METADATA_CONTROL_PLANE,
        }
    }

    /// Return whether this surface is a data plane or a control plane.
    #[must_use]
    pub const fn plane(self) -> WendaoPayloadPlane {
        match self {
            Self::ArrowFlight | Self::ArrowRecordBatch => WendaoPayloadPlane::Data,
            Self::JsonControl
            | Self::JsonlStdioControl
            | Self::ProcessArgsControl
            | Self::RestMetadataControl => WendaoPayloadPlane::Control,
        }
    }

    /// Return whether this surface may carry Arrow table payloads.
    #[must_use]
    pub const fn carries_arrow_table_payloads(self) -> bool {
        matches!(self.plane(), WendaoPayloadPlane::Data)
    }
}

/// Payload-boundary validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoPayloadBoundaryError {
    message: String,
}

impl WendaoPayloadBoundaryError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Return the validation error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for WendaoPayloadBoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WendaoPayloadBoundaryError {}

/// Parse a canonical payload-surface token.
#[must_use]
pub fn parse_wendao_payload_surface(token: &str) -> Option<WendaoPayloadSurface> {
    let normalized = token.trim().to_ascii_lowercase();
    match normalized.as_str() {
        WENDAO_ARROW_FLIGHT_DATA_PLANE => Some(WendaoPayloadSurface::ArrowFlight),
        WENDAO_ARROW_RECORD_BATCH_PAYLOAD => Some(WendaoPayloadSurface::ArrowRecordBatch),
        WENDAO_JSON_CONTROL_PLANE => Some(WendaoPayloadSurface::JsonControl),
        WENDAO_JSONL_STDIO_CONTROL_PLANE => Some(WendaoPayloadSurface::JsonlStdioControl),
        WENDAO_PROCESS_ARGS_CONTROL_PLANE => Some(WendaoPayloadSurface::ProcessArgsControl),
        WENDAO_REST_METADATA_CONTROL_PLANE => Some(WendaoPayloadSurface::RestMetadataControl),
        _ => None,
    }
}

/// Validate that an Arrow table payload uses a data-plane surface.
///
/// # Errors
///
/// Returns an error when a control-only surface is used to carry Arrow table
/// data.
pub fn validate_arrow_table_payload_surface(
    surface: WendaoPayloadSurface,
) -> Result<(), WendaoPayloadBoundaryError> {
    if surface.carries_arrow_table_payloads() {
        return Ok(());
    }

    Err(WendaoPayloadBoundaryError::new(format!(
        "Arrow table payloads must use `{}`/`{}`; `{}` is control-only",
        WENDAO_ARROW_FLIGHT_DATA_PLANE,
        WENDAO_ARROW_RECORD_BATCH_PAYLOAD,
        surface.token()
    )))
}

/// Validate that an encoding token does not wrap Arrow payloads in base64.
///
/// # Errors
///
/// Returns an error when the token names a forbidden Arrow/base64 wrapper.
pub fn validate_arrow_payload_encoding_token(
    token: &str,
) -> Result<(), WendaoPayloadBoundaryError> {
    let normalized = token.trim().to_ascii_lowercase();
    let forbidden = WENDAO_FORBIDDEN_ARROW_PAYLOAD_WRAPPERS
        .iter()
        .any(|candidate| normalized == *candidate);
    let ad_hoc_arrow_base64 = normalized.contains("arrow") && normalized.contains("base64");
    if forbidden || ad_hoc_arrow_base64 {
        return Err(WendaoPayloadBoundaryError::new(format!(
            "`{token}` is not a Wendao payload boundary; use `{WENDAO_ARROW_FLIGHT_DATA_PLANE}` for table data and keep JSON/JSONL as control only"
        )));
    }

    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/transport/data_plane.rs"]
mod tests;
