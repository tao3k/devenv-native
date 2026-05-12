use crate::link_graph::models::{
    LinkGraphJuliaRerankTelemetry, LinkGraphRetrievalPlanRecord, QuantumContext,
};
#[cfg(feature = "julia")]
use crate::link_graph::models::{QuantumAnchorHit, QuantumSemanticSearchRequest};
use crate::link_graph::{OpenAiCompatibleSemanticIgnition, VectorStoreSemanticIgnition};
#[cfg(feature = "julia")]
use arrow::record_batch::RecordBatch;
#[cfg(feature = "julia")]
use std::cmp::Ordering;
#[cfg(feature = "julia")]
use std::collections::BTreeMap;
use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;
#[cfg(feature = "julia")]
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_SCHEMA_VERSION, NegotiatedTransportSelection, PluginArrowScoreRoundtripError,
    PluginArrowScoreRow, roundtrip_plugin_arrow_score_rows_with_binding,
};

#[cfg(feature = "julia")]
pub(super) async fn apply_vector_store_plugin_rerank(
    ignition: &VectorStoreSemanticIgnition,
    binding: Option<&PluginCapabilityBinding>,
    query_text: &str,
    query_vector: &[f32],
    retrieval_plan: &LinkGraphRetrievalPlanRecord,
    contexts: &mut [QuantumContext],
) -> Option<LinkGraphJuliaRerankTelemetry> {
    let binding = binding?;

    if query_vector.is_empty() || contexts.is_empty() {
        return Some(LinkGraphJuliaRerankTelemetry {
            applied: false,
            response_row_count: 0,
            selected_transport: None,
            fallback_from: Some(binding.transport),
            fallback_reason: Some(
                "link-graph Julia rerank currently requires a precomputed query vector for the vector-store semantic ignition backend".to_string(),
            ),
            trace_ids: Vec::new(),
            error: Some(
                "link-graph Julia rerank currently requires a precomputed query vector for the vector-store semantic ignition backend".to_string(),
            ),
        });
    }

    let request = QuantumSemanticSearchRequest::from_retrieval_budget(
        Some(query_text),
        query_vector,
        Some(&retrieval_plan.budget),
        None,
    );
    let anchors = collect_plugin_rerank_anchors(contexts);
    complete_plugin_rerank_roundtrip(
        binding,
        ignition
            .build_plugin_rerank_request_batch_with_metadata(
                request,
                &anchors,
                binding.selector.provider.0.as_str(),
                query_text,
                plugin_rerank_request_schema_version(binding),
            )
            .await,
        contexts,
    )
    .await
}

#[cfg(not(feature = "julia"))]
pub(super) async fn apply_vector_store_plugin_rerank(
    _ignition: &VectorStoreSemanticIgnition,
    binding: Option<&PluginCapabilityBinding>,
    _query_text: &str,
    _query_vector: &[f32],
    _retrieval_plan: &LinkGraphRetrievalPlanRecord,
    _contexts: &mut [QuantumContext],
) -> Option<LinkGraphJuliaRerankTelemetry> {
    binding
        .and_then(|binding| binding.endpoint.base_url.as_ref())
        .as_ref()
        .map(|_| LinkGraphJuliaRerankTelemetry {
            applied: false,
            response_row_count: 0,
            selected_transport: None,
            fallback_from: binding.map(|binding| binding.transport),
            fallback_reason: Some(
                "link-graph Julia rerank is configured but `xiuxian-wendao` was built without the `julia` feature".to_string(),
            ),
            trace_ids: Vec::new(),
            error: Some(
                "link-graph Julia rerank is configured but `xiuxian-wendao` was built without the `julia` feature".to_string(),
            ),
        })
}

#[cfg(feature = "julia")]
pub(super) async fn apply_openai_plugin_rerank(
    ignition: &OpenAiCompatibleSemanticIgnition,
    binding: Option<&PluginCapabilityBinding>,
    query_text: &str,
    retrieval_plan: &LinkGraphRetrievalPlanRecord,
    contexts: &mut [QuantumContext],
) -> Option<LinkGraphJuliaRerankTelemetry> {
    let binding = binding?;
    if binding.endpoint.base_url.is_none() || contexts.is_empty() {
        return None;
    }

    let request = QuantumSemanticSearchRequest::from_retrieval_budget(
        Some(query_text),
        &[],
        Some(&retrieval_plan.budget),
        None,
    );
    let anchors = collect_plugin_rerank_anchors(contexts);
    complete_plugin_rerank_roundtrip(
        binding,
        ignition
            .build_plugin_rerank_request_batch_with_metadata(
                request,
                &anchors,
                binding.selector.provider.0.as_str(),
                query_text,
                plugin_rerank_request_schema_version(binding),
            )
            .await,
        contexts,
    )
    .await
}

#[cfg(not(feature = "julia"))]
pub(super) async fn apply_openai_plugin_rerank(
    _ignition: &OpenAiCompatibleSemanticIgnition,
    binding: Option<&PluginCapabilityBinding>,
    _query_text: &str,
    _retrieval_plan: &LinkGraphRetrievalPlanRecord,
    _contexts: &mut [QuantumContext],
) -> Option<LinkGraphJuliaRerankTelemetry> {
    binding
        .and_then(|binding| binding.endpoint.base_url.as_ref())
        .as_ref()
        .map(|_| LinkGraphJuliaRerankTelemetry {
            applied: false,
            response_row_count: 0,
            selected_transport: None,
            fallback_from: binding.map(|binding| binding.transport),
            fallback_reason: Some(
                "link-graph Julia rerank is configured but `xiuxian-wendao` was built without the `julia` feature".to_string(),
            ),
            trace_ids: Vec::new(),
            error: Some(
                "link-graph Julia rerank is configured but `xiuxian-wendao` was built without the `julia` feature".to_string(),
            ),
        })
}

#[cfg(feature = "julia")]
pub(super) fn build_plugin_rerank_telemetry(
    selection: Option<&NegotiatedTransportSelection>,
    applied: bool,
    response_row_count: usize,
    trace_ids: Vec<String>,
    error: Option<String>,
) -> LinkGraphJuliaRerankTelemetry {
    LinkGraphJuliaRerankTelemetry {
        applied,
        response_row_count,
        selected_transport: selection.map(|selection| selection.selected_transport),
        fallback_from: selection.and_then(|selection| selection.fallback_from),
        fallback_reason: selection.and_then(|selection| selection.fallback_reason.clone()),
        trace_ids,
        error,
    }
}

#[cfg(feature = "julia")]
pub(super) fn collect_plugin_rerank_anchors(contexts: &[QuantumContext]) -> Vec<QuantumAnchorHit> {
    contexts
        .iter()
        .map(|context| QuantumAnchorHit {
            anchor_id: context.anchor_id.clone(),
            vector_score: context.vector_score,
        })
        .collect()
}

#[cfg(feature = "julia")]
fn plugin_rerank_request_build_error_telemetry(error: String) -> LinkGraphJuliaRerankTelemetry {
    build_plugin_rerank_telemetry(None, false, 0, Vec::new(), Some(error))
}

#[cfg(feature = "julia")]
async fn complete_plugin_rerank_roundtrip(
    binding: &PluginCapabilityBinding,
    request_batch: Result<RecordBatch, String>,
    contexts: &mut [QuantumContext],
) -> Option<LinkGraphJuliaRerankTelemetry> {
    let request_batch = match request_batch {
        Ok(batch) => batch,
        Err(error) => return Some(plugin_rerank_request_build_error_telemetry(error)),
    };
    let roundtrip =
        match roundtrip_plugin_arrow_score_rows_with_binding(binding, &request_batch).await {
            Ok(Some(roundtrip)) => roundtrip,
            Ok(None) => return None,
            Err(error) => return Some(plugin_rerank_roundtrip_error_telemetry(binding, error)),
        };
    let updated = apply_plugin_rerank_scores(contexts, &roundtrip.rows);
    Some(build_plugin_rerank_telemetry(
        Some(&roundtrip.selection),
        updated > 0,
        roundtrip.rows.len(),
        collect_plugin_rerank_trace_ids(&roundtrip.rows),
        None,
    ))
}

#[cfg(feature = "julia")]
fn plugin_rerank_roundtrip_error_telemetry(
    binding: &PluginCapabilityBinding,
    error: PluginArrowScoreRoundtripError,
) -> LinkGraphJuliaRerankTelemetry {
    let selection = error.selection.as_ref();
    LinkGraphJuliaRerankTelemetry {
        applied: false,
        response_row_count: 0,
        selected_transport: selection.map(|selection| selection.selected_transport),
        fallback_from: selection
            .and_then(|selection| selection.fallback_from)
            .or(Some(binding.transport)),
        fallback_reason: selection
            .and_then(|selection| selection.fallback_reason.clone())
            .or_else(|| error.selection.is_none().then(|| error.error.clone())),
        trace_ids: Vec::new(),
        error: Some(error.error),
    }
}

#[cfg(feature = "julia")]
pub(super) fn apply_plugin_rerank_scores(
    contexts: &mut [QuantumContext],
    response_rows: &BTreeMap<String, PluginArrowScoreRow>,
) -> usize {
    let updated = contexts
        .iter_mut()
        .filter_map(|context| {
            let score_row = response_rows.get(context.anchor_id.as_str())?;
            context.saliency_score = score_row.final_score;
            Some(())
        })
        .count();
    contexts.sort_by(|left, right| {
        right
            .saliency_score
            .partial_cmp(&left.saliency_score)
            .unwrap_or(Ordering::Equal)
            .then(left.anchor_id.cmp(&right.anchor_id))
    });
    updated
}

#[cfg(feature = "julia")]
pub(super) fn collect_plugin_rerank_trace_ids(
    response_rows: &BTreeMap<String, PluginArrowScoreRow>,
) -> Vec<String> {
    response_rows
        .values()
        .filter_map(|row| row.trace_id.as_ref())
        .filter(|trace_id| !trace_id.trim().is_empty())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(feature = "julia")]
fn plugin_rerank_request_schema_version(binding: &PluginCapabilityBinding) -> &str {
    let schema_version = binding.contract_version.0.trim();
    if schema_version.is_empty() {
        DEFAULT_FLIGHT_SCHEMA_VERSION
    } else {
        schema_version
    }
}
