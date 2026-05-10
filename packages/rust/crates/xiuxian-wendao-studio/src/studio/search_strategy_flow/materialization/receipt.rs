//! Owns the Studio search strategy flow materialization receipt surface.

use serde::Serialize;

use crate::contracts::{StudioContractStatus, StudioContractToken};
use thiserror::Error;

/// Error raised while building a SearchStrategyFlow materialization receipt.
#[derive(Debug, Error)]
pub enum SearchStrategyFlowMaterializationError {
    /// Human-readable validation or transport error.
    #[error("{0}")]
    Message(String),
    /// Filesystem or process I/O failed while building the proof fixture.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// JSON encoding or decoding failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl SearchStrategyFlowMaterializationError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

/// Row-count receipt for one executed Flight route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteMaterializationReceipt {
    /// Flight route identifier.
    pub route: String,
    /// Number of rows decoded from the route output.
    pub row_count: usize,
}

impl RouteMaterializationReceipt {
    /// Creates a route materialization receipt.
    ///
    /// # Errors
    ///
    /// Returns an error when the route did not materialize any rows.
    pub fn new(
        route: impl Into<String>,
        row_count: usize,
    ) -> Result<Self, SearchStrategyFlowMaterializationError> {
        let route = route.into();
        if row_count == 0 {
            return Err(SearchStrategyFlowMaterializationError::message(format!(
                "{route} should materialize at least one row"
            )));
        }
        Ok(Self { route, row_count })
    }
}

/// Decoded payload receipt for one executed Flight route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDecodedPayloadReceipt {
    /// Flight route identifier.
    pub route: String,
    /// Number of rows decoded from the route output.
    pub row_count: usize,
    /// Route columns that were decoded into proof evidence.
    pub decoded_columns: Vec<String>,
    /// Stable evidence anchor extracted from the decoded route output.
    pub evidence_anchor: String,
}

impl RouteDecodedPayloadReceipt {
    /// Creates a decoded payload receipt for one route.
    ///
    /// # Errors
    ///
    /// Returns an error when the route has no rows, decoded columns, or
    /// evidence anchor.
    pub fn new(
        route: impl Into<String>,
        row_count: usize,
        decoded_columns: Vec<String>,
        evidence_anchor: impl Into<String>,
    ) -> Result<Self, SearchStrategyFlowMaterializationError> {
        let route = route.into();
        if row_count == 0 {
            return Err(SearchStrategyFlowMaterializationError::message(format!(
                "{route} should decode at least one row"
            )));
        }
        if decoded_columns.is_empty() {
            return Err(SearchStrategyFlowMaterializationError::message(format!(
                "{route} should record decoded columns"
            )));
        }
        let evidence_anchor = evidence_anchor.into();
        if evidence_anchor.is_empty() {
            return Err(SearchStrategyFlowMaterializationError::message(format!(
                "{route} should record an evidence anchor"
            )));
        }
        Ok(Self {
            route,
            row_count,
            decoded_columns,
            evidence_anchor,
        })
    }
}

/// SearchStrategyFlow decoded materialization receipt emitted by Studio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchStrategyFlowMaterializationReceipt {
    /// Materialization status for the route sequence.
    pub materialization_status: StudioContractStatus,
    /// Component that produced the receipt.
    pub receipt_source: StudioContractToken,
    /// Primary transport used to execute retrieval routes.
    pub primary_transport: StudioContractToken,
    /// Whether direct file reads were allowed in the proof path.
    pub direct_file_read_allowed: bool,
    /// Whether routes must execute before external agents answer.
    pub execute_before_answer: bool,
    /// Sum of decoded rows across all materialized routes.
    pub materialized_rows: usize,
    /// Status of decoded payload validation.
    pub decoded_payload_status: StudioContractStatus,
    /// Per-route materialization receipts.
    pub route_receipts: Vec<RouteMaterializationReceipt>,
    /// Per-route decoded payload receipts.
    pub decoded_payload_receipts: Vec<RouteDecodedPayloadReceipt>,
}

impl SearchStrategyFlowMaterializationReceipt {
    /// Creates an executed Studio Flight decoded materialization receipt.
    #[must_use]
    pub fn executed(
        receipt_source: impl Into<String>,
        route_receipts: Vec<RouteMaterializationReceipt>,
        decoded_payload_receipts: Vec<RouteDecodedPayloadReceipt>,
    ) -> Self {
        let materialized_rows = route_receipts.iter().map(|route| route.row_count).sum();
        Self {
            materialization_status: "executed".into(),
            receipt_source: receipt_source.into().into(),
            primary_transport: "arrow-flight".into(),
            direct_file_read_allowed: false,
            execute_before_answer: true,
            materialized_rows,
            decoded_payload_status: "decoded".into(),
            route_receipts,
            decoded_payload_receipts,
        }
    }

    /// Serializes the receipt as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the receipt cannot be represented as JSON.
    pub fn to_json(&self) -> Result<serde_json::Value, SearchStrategyFlowMaterializationError> {
        serde_json::to_value(self).map_err(Into::into)
    }
}
