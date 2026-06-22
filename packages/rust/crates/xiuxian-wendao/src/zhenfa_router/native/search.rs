//! `zhenfa_router::native::search` owns Wendao zhenfa router native search behavior.

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError};

use crate::link_graph::{
    LinkGraphCcsAudit, LinkGraphPlannedSearchPayload, LinkGraphRelatedFilter,
    LinkGraphSearchOptions,
};

use super::context::WendaoContextExt;
use super::xml_lite;

const DEFAULT_SEARCH_LIMIT: usize = 20;
const MAX_SEARCH_LIMIT: usize = 200;

/// Arguments for graph search via the native Wendao tool surface.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoSearchArgs {
    /// User search query text.
    query: String,
    /// Optional precomputed query embedding used by semantic ignition and
    /// Julia rerank when the caller already owns the vectorization step.
    #[serde(default)]
    query_vector: Option<Vec<f32>>,
    /// Maximum number of hits to return.
    #[serde(default)]
    limit: Option<usize>,
    /// Optional root directory hint for path-scoped search.
    #[serde(default)]
    root_dir: Option<String>,
    /// Additional search options forwarded to the link-graph planner.
    #[serde(default)]
    options: Option<LinkGraphSearchOptions>,
    /// Whether provisional results should be included.
    #[serde(default)]
    include_provisional: Option<bool>,
    /// Upper bound for provisional hits.
    #[serde(default)]
    provisional_limit: Option<usize>,
    /// Optional style anchors for CCS (Context Completeness Score) audit.
    #[serde(default)]
    anchors: Option<Vec<String>>,
}

/// Search the Wendao graph index and return stripped XML-Lite `<hit>` records.
/// Native tool for searching the wendao graph index.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when the query is invalid, the root argument is malformed,
/// or the graph index cannot execute the requested search.
#[allow(missing_docs)]
/// # Errors
/// Returns a [`ZhenfaError`] when validation fails or planner execution fails.
pub fn wendao_search(ctx: &ZhenfaContext, args: WendaoSearchArgs) -> Result<String, ZhenfaError> {
    let query = args.query.trim();
    if query.is_empty() {
        return Err(ZhenfaError::invalid_arguments(
            "`query` must be a non-empty string",
        ));
    }

    validate_root_dir_argument(args.root_dir.as_deref())?;
    let options = args.options.unwrap_or_default();
    let index = ctx.link_graph_index()?;
    let limit = normalize_limit(args.limit);

    // First-pass search
    let payload = if let Some(query_vector) = args.query_vector.as_deref() {
        index.search_planned_payload_with_agentic_query_vector(
            query,
            query_vector,
            limit,
            options.clone(),
            args.include_provisional,
            args.provisional_limit,
        )
    } else {
        index.search_planned_payload_with_agentic(
            query,
            limit,
            options.clone(),
            args.include_provisional,
            args.provisional_limit,
        )
    };

    // Apply CCS audit and compensation loop if anchors provided
    if let Some(anchors) = args.anchors
        && !anchors.is_empty()
    {
        let evidence: Vec<String> = payload
            .results
            .iter()
            .flat_map(|hit| vec![hit.stem.clone(), hit.title.clone()])
            .collect();

        let audit_result = super::audit::audit_search_payload(&evidence, &anchors);

        // Apply compensation if CCS < threshold
        let (mut final_payload, compensated) = if let Some(comp) = &audit_result.compensation {
            let mut compensated_options = options.clone();
            // Expand max_distance for broader retrieval
            if let Some(ref related) = compensated_options.filters.related {
                let mut related = related.clone();
                related.max_distance =
                    Some(related.max_distance.unwrap_or(2) + comp.max_distance_delta);
                compensated_options.filters.related = Some(related);
            } else {
                compensated_options.filters.related = Some(LinkGraphRelatedFilter {
                    max_distance: Some(comp.max_distance_delta + 2),
                    ..Default::default()
                });
            }

            // Re-search with compensated parameters
            let compensated_payload = index.search_planned_payload_with_agentic(
                query,
                limit,
                compensated_options,
                args.include_provisional,
                args.provisional_limit,
            );
            (compensated_payload, true)
        } else {
            (payload, false)
        };

        final_payload.ccs_audit = Some(LinkGraphCcsAudit {
            ccs_score: audit_result.ccs_score,
            passed: audit_result.passed,
            compensated,
            missing_anchors: audit_result.missing_anchors,
        });

        return Ok(xml_lite::render_xml_lite(&final_payload));
    }

    Ok(xml_lite::render_xml_lite(&payload))
}

/// Render one planned payload into XML-Lite hit rows.
///
/// This is a thin public adapter over native XML-Lite rendering logic, used by
/// integration tests and tool-facing formatting call sites.
#[must_use]
pub fn render_xml_lite_hits(payload: &LinkGraphPlannedSearchPayload) -> String {
    xml_lite::render_xml_lite(payload)
}

fn normalize_limit(raw: Option<usize>) -> usize {
    raw.unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT)
}

fn validate_root_dir_argument(root_dir: Option<&str>) -> Result<(), ZhenfaError> {
    if let Some(value) = root_dir {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ZhenfaError::invalid_arguments(
                "`root_dir` must be non-empty when provided",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/zhenfa_router/native/search.rs"]
mod tests;
