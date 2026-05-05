//! Negotiated Plugin Arrow exchange roundtrip execution.

use std::collections::BTreeMap;

use arrow_array::RecordBatch;
use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;

use crate::transport::negotiation::{
    NegotiatedFlightTransportClient, NegotiatedTransportSelection,
    negotiate_flight_transport_client_from_bindings,
};

use super::{
    PluginArrowScoreRow, decode_plugin_arrow_score_rows, validate_plugin_arrow_response_batches,
};

/// One negotiated `WendaoArrow` rerank roundtrip decoded into typed score rows.
#[derive(Debug, Clone, PartialEq)]
pub struct NegotiatedPluginArrowScoreRows {
    /// Runtime transport selection used for the roundtrip.
    pub selection: NegotiatedTransportSelection,
    /// Typed score rows keyed by `doc_id`.
    pub rows: BTreeMap<String, PluginArrowScoreRow>,
}

/// Error returned when one negotiated `WendaoArrow` rerank roundtrip fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginArrowScoreRoundtripError {
    /// Runtime transport selection, when a client was negotiated successfully.
    pub selection: Option<NegotiatedTransportSelection>,
    /// Human-readable failure detail.
    pub error: String,
}

/// Negotiate one plugin transport client from the provided binding and decode
/// one `WendaoArrow` rerank response into typed score rows.
///
/// # Errors
///
/// Returns [`PluginArrowScoreRoundtripError`] when the runtime cannot
/// negotiate the plugin transport client, send the request batch, or decode
/// the response batches into typed score rows.
pub async fn roundtrip_plugin_arrow_score_rows_with_binding(
    binding: &PluginCapabilityBinding,
    batch: &RecordBatch,
) -> Result<Option<NegotiatedPluginArrowScoreRows>, PluginArrowScoreRoundtripError> {
    let Some(transport) = negotiate_flight_transport_client_from_bindings(std::slice::from_ref(
        binding,
    ))
    .map_err(|error| PluginArrowScoreRoundtripError {
        selection: None,
        error,
    })?
    else {
        return Ok(None);
    };

    roundtrip_plugin_arrow_score_rows_with_transport(&transport, batch)
        .await
        .map(Some)
}

async fn roundtrip_plugin_arrow_score_rows_with_transport(
    transport: &NegotiatedFlightTransportClient,
    batch: &RecordBatch,
) -> Result<NegotiatedPluginArrowScoreRows, PluginArrowScoreRoundtripError> {
    let selection = transport.selection().clone();
    let response_batches =
        transport
            .process_batch(batch)
            .await
            .map_err(|error| PluginArrowScoreRoundtripError {
                selection: Some(selection.clone()),
                error: format!("plugin rerank transport failed: {error}"),
            })?;
    let rows = decode_plugin_arrow_score_rows_from_batches(response_batches.as_slice()).map_err(
        |error| PluginArrowScoreRoundtripError {
            selection: Some(selection.clone()),
            error,
        },
    )?;

    Ok(NegotiatedPluginArrowScoreRows { selection, rows })
}

fn decode_plugin_arrow_score_rows_from_batches(
    response_batches: &[RecordBatch],
) -> Result<BTreeMap<String, PluginArrowScoreRow>, String> {
    validate_plugin_arrow_response_batches(response_batches)
        .map_err(|error| format!("plugin rerank response contract validation failed: {error}"))?;
    decode_plugin_arrow_score_rows(response_batches)
        .map_err(|error| format!("failed to decode plugin rerank response rows: {error}"))
}
