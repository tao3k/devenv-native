//! `zhenfa_router::rpc` owns Wendao zhenfa router rpc behavior.

use std::path::PathBuf;

use serde_json::{Value, json};
use xiuxian_zhenfa::{INTERNAL_ERROR_CODE, JsonRpcErrorObject};

use super::models::{WendaoSearchRequest, WendaoSearchResponseFormat};
use super::native::{WendaoPluginArtifactArgs, export_plugin_artifact};
use crate::link_graph::{LinkGraphIndex, LinkGraphPlannedSearchPayload};

pub(super) const DEFAULT_SEARCH_LIMIT: usize = 20;
pub(super) const MAX_SEARCH_LIMIT: usize = 200;

/// Execute `wendao.search` from JSON-RPC parameters.
///
/// # Errors
/// Returns JSON-RPC error payloads when params are invalid or search execution fails.
pub fn search_from_rpc_params(params: Value) -> Result<String, JsonRpcErrorObject> {
    let request: WendaoSearchRequest = serde_json::from_value(params).map_err(|error| {
        JsonRpcErrorObject::invalid_params(format!("invalid wendao.search params: {error}"))
    })?;
    execute_search(&request).map_err(|error| {
        JsonRpcErrorObject::new(
            INTERNAL_ERROR_CODE,
            "wendao search failed",
            Some(json!({ "details": error })),
        )
    })
}

/// Execute `wendao.plugin_artifact` from JSON-RPC parameters.
///
/// # Errors
/// Returns JSON-RPC error payloads when params are invalid or export fails.
pub fn export_plugin_artifact_from_rpc_params(params: Value) -> Result<String, JsonRpcErrorObject> {
    let request: WendaoPluginArtifactArgs = serde_json::from_value(params).map_err(|error| {
        JsonRpcErrorObject::invalid_params(format!(
            "invalid wendao.plugin_artifact params: {error}"
        ))
    })?;

    export_plugin_artifact(&request).map_err(|error: xiuxian_zhenfa::ZhenfaError| {
        JsonRpcErrorObject::new(
            INTERNAL_ERROR_CODE,
            "wendao plugin artifact export failed",
            Some(json!({ "details": error.to_string() })),
        )
    })
}

/// Execute one Wendao search request.
///
/// # Errors
/// Returns an error when index construction, query execution, or payload serialization fails.
pub fn execute_search(request: &WendaoSearchRequest) -> Result<String, String> {
    let (root, root_dir, query, limit, base_options, response_format) =
        validate_search_request(request)?;
    let index = LinkGraphIndex::build(&root)
        .map_err(|error| format!("failed to build LinkGraph index at `{root_dir}`: {error}"))?;
    let payload = if let Some(query_vector) = request.query_vector.as_deref() {
        index.search_planned_payload_with_agentic_query_vector(
            query,
            query_vector,
            limit,
            base_options,
            request.include_provisional,
            request.provisional_limit,
        )
    } else {
        index.search_planned_payload_with_agentic(
            query,
            limit,
            base_options,
            request.include_provisional,
            request.provisional_limit,
        )
    };
    render_payload(&payload, response_format)
}

/// Execute one Wendao search request on the async planned-payload path.
///
/// # Errors
/// Returns an error when index construction, query execution, or payload serialization fails.
#[cfg(feature = "zhenfa-router")]
pub async fn execute_search_async(request: &WendaoSearchRequest) -> Result<String, String> {
    let (root, root_dir, query, limit, base_options, response_format) =
        validate_search_request(request)?;
    let index = LinkGraphIndex::build(&root)
        .map_err(|error| format!("failed to build LinkGraph index at `{root_dir}`: {error}"))?;
    let payload = index
        .search_planned_payload_with_agentic_async_with_query_vector(
            query,
            request.query_vector.as_deref().unwrap_or(&[]),
            limit,
            base_options,
            request.include_provisional,
            request.provisional_limit,
        )
        .await;
    render_payload(&payload, response_format)
}

fn validate_search_request(
    request: &WendaoSearchRequest,
) -> Result<
    (
        PathBuf,
        String,
        &str,
        usize,
        crate::link_graph::LinkGraphSearchOptions,
        WendaoSearchResponseFormat,
    ),
    String,
> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err("`query` must be a non-empty string".to_string());
    }

    let root_dir = request.root_dir.as_deref().unwrap_or(".").trim();
    if root_dir.is_empty() {
        return Err("`root_dir` must be non-empty when provided".to_string());
    }

    let root = PathBuf::from(root_dir);
    let limit = normalize_limit(request.limit);
    let base_options = request.options.clone().unwrap_or_default();
    Ok((
        root,
        root_dir.to_string(),
        query,
        limit,
        base_options,
        request.response_format,
    ))
}

fn render_payload(
    payload: &LinkGraphPlannedSearchPayload,
    response_format: WendaoSearchResponseFormat,
) -> Result<String, String> {
    match response_format {
        WendaoSearchResponseFormat::Markdown => Ok(render_markdown(payload)),
        WendaoSearchResponseFormat::Json => serde_json::to_string(&payload)
            .map_err(|error| format!("failed to serialize search payload: {error}")),
    }
}

pub(super) fn normalize_limit(raw: Option<usize>) -> usize {
    raw.unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT)
}

pub(super) fn render_markdown(payload: &LinkGraphPlannedSearchPayload) -> String {
    let mut lines = Vec::new();
    lines.push("## Wendao Search Results".to_string());
    lines.push(format!("- query: {}", payload.query));
    lines.push(format!("- total_hits: {}", payload.hit_count));
    lines.push(format!(
        "- retrieval_mode: {:?} (reason: {})",
        payload.selected_mode, payload.reason
    ));
    if let Some(semantic_ignition) = payload.semantic_ignition.as_ref() {
        let mut telemetry = format!(
            "- semantic_ignition: {}",
            semantic_ignition
                .backend_name
                .as_deref()
                .unwrap_or(semantic_ignition.backend.as_str())
        );
        if let Some(error) = semantic_ignition.error.as_deref() {
            telemetry.push_str(format!(" error={error}").as_str());
        } else {
            telemetry.push_str(format!(" contexts={}", semantic_ignition.context_count).as_str());
        }
        lines.push(telemetry);
    }

    if payload.hits.is_empty() {
        lines.push("- hits: none".to_string());
        if payload.quantum_contexts.is_empty() {
            return lines.join("\n");
        }
    }

    if !payload.hits.is_empty() {
        lines.push("### Hits".to_string());
        for (index, hit) in payload.hits.iter().enumerate() {
            let title = if hit.title.trim().is_empty() {
                hit.stem.as_str()
            } else {
                hit.title.as_str()
            };
            lines.push(format!(
                "{}. {} (`{}`) score={:.3}",
                index + 1,
                title,
                hit.path,
                hit.score
            ));
            if !hit.best_section.trim().is_empty() {
                lines.push(format!("   section: {}", hit.best_section));
            }
        }
    }

    if !payload.quantum_contexts.is_empty() {
        lines.push("### Quantum Contexts".to_string());
        for (index, context) in payload.quantum_contexts.iter().enumerate() {
            lines.push(format!(
                "{}. {} (`{}`) saliency={:.3} vector={:.3}",
                index + 1,
                context.doc_id,
                context.path,
                context.saliency_score,
                context.vector_score
            ));
            if !context.semantic_path.is_empty() {
                lines.push(format!("   path: {}", context.semantic_path.join(" > ")));
            }
        }
    }

    lines.join("\n")
}

#[cfg(test)]
#[path = "../../tests/unit/zhenfa_router/rpc.rs"]
mod tests;
