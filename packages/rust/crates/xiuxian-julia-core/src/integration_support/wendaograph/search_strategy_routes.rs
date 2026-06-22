//! Retrieval-route enrichment for `SearchStrategyFlow` bridge traces.

use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

use crate::integration_support::search_strategy_flow_candidates::{
    CODE_INTELLIGENCE_CANDIDATE_SOURCE, MARKDOWN_HEADING_CANDIDATE_SOURCE,
    REGISTRY_METADATA_CANDIDATE_SOURCE, SearchStrategyFlowStructuredCandidateCounts,
    WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE, link_search_strategy_flow_offline_audit_entrypoints,
    search_strategy_flow_candidate_discovery_contract_json,
    search_strategy_flow_total_structured_candidate_index_contract_json,
};

pub(super) fn add_search_strategy_flow_retrieval_routes(value: &mut Value) -> Result<(), String> {
    let routes = build_search_strategy_flow_retrieval_routes(value);
    let projected_evidence_rows =
        build_search_strategy_flow_rust_projected_evidence_rows(value, &routes);
    let (candidate_counts, inventory_source) = search_strategy_flow_trace_candidate_counts(value);
    let candidate_discovery_contract = search_strategy_flow_candidate_discovery_contract_json(
        candidate_counts,
        json_string(value, "candidateInputSource"),
        json_usize(value, "candidateInputCount"),
        value.get("candidateInputDiscovery"),
    );
    let object = value.as_object_mut().ok_or_else(|| {
        "WendaoGraph SearchStrategyFlow JSON trace root must be an object".to_owned()
    })?;
    object.insert("retrievalRoutes".to_owned(), Value::Array(routes));
    object.insert(
        "rustProjectedEvidenceRows".to_owned(),
        Value::Array(projected_evidence_rows),
    );
    object.insert(
        "structuredCandidateIndexContract".to_owned(),
        search_strategy_flow_total_structured_candidate_index_contract_json(
            candidate_counts,
            inventory_source,
        ),
    );
    object.insert(
        "candidateDiscoveryContract".to_owned(),
        candidate_discovery_contract,
    );
    Ok(())
}

fn search_strategy_flow_trace_candidate_counts(
    value: &Value,
) -> (SearchStrategyFlowStructuredCandidateCounts, &'static str) {
    link_search_strategy_flow_offline_audit_entrypoints();
    let candidate_input_count = value
        .get("candidateInputCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_else(|| {
            value
                .get("candidates")
                .and_then(Value::as_array)
                .map_or(0, Vec::len)
        });
    let mut counts = SearchStrategyFlowStructuredCandidateCounts::default();
    match json_string(value, "candidateInputSource") {
        Some(CODE_INTELLIGENCE_CANDIDATE_SOURCE) => {
            counts.code_intelligence = candidate_input_count;
        }
        Some(REGISTRY_METADATA_CANDIDATE_SOURCE) => {
            counts.registry_authority = candidate_input_count;
        }
        Some(MARKDOWN_HEADING_CANDIDATE_SOURCE) => {
            counts.primary_markdown = candidate_input_count;
        }
        Some(WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE) => {
            counts.gateway_retrieval = candidate_input_count;
        }
        _ => {}
    }
    (counts, "gateway-flight-trace")
}

fn build_search_strategy_flow_retrieval_routes(trace: &Value) -> Vec<Value> {
    let candidate_input_source = json_string(trace, "candidateInputSource").map(str::to_owned);
    let selected_candidate_ids = trace
        .get("frontier")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| json_bool(row, "selected"))
        .filter_map(|row| json_string(row, "candidateId"))
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();

    let action_candidate_ids = trace
        .get("plannerActions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| json_string(row, "actionKind") != Some("stop"))
        .flat_map(|row| {
            [
                json_string(row, "candidateId"),
                json_string(row, "targetCandidateId"),
            ]
        })
        .flatten()
        .filter(|id| !id.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();

    trace
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|candidate| !json_bool(candidate, "blocked"))
        .filter_map(|candidate| {
            let candidate_id = json_string(candidate, "candidateId")?;
            (selected_candidate_ids.contains(candidate_id)
                || action_candidate_ids.contains(candidate_id))
            .then_some(candidate_id)
        })
        .filter_map(|candidate_id| {
            search_strategy_flow_retrieval_route(candidate_id, candidate_input_source.as_deref())
        })
        .collect()
}

fn build_search_strategy_flow_rust_projected_evidence_rows(
    trace: &Value,
    routes: &[Value],
) -> Vec<Value> {
    let selected_candidate_ids = trace
        .get("frontier")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| json_bool(row, "selected"))
        .filter_map(|row| json_string(row, "candidateId"))
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();
    let materialized_candidate_ids = trace
        .get("plannerActions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| json_string(row, "actionKind") == Some("materialize"))
        .flat_map(|row| {
            [
                json_string(row, "candidateId"),
                json_string(row, "targetCandidateId"),
            ]
        })
        .flatten()
        .filter(|id| !id.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();
    let route_counts = search_strategy_flow_route_counts(routes);

    trace
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|candidate| {
            let candidate_id = json_string(candidate, "candidateId")?;
            Some(search_strategy_flow_rust_projected_evidence_row(
                candidate_id,
                candidate,
                &selected_candidate_ids,
                &materialized_candidate_ids,
                &route_counts,
            ))
        })
        .collect()
}

fn search_strategy_flow_route_counts(routes: &[Value]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for route in routes {
        if let Some(candidate_id) = json_string(route, "candidateId") {
            *counts.entry(candidate_id.to_owned()).or_insert(0) += 1;
        }
    }
    counts
}

fn search_strategy_flow_rust_projected_evidence_row(
    candidate_id: &str,
    candidate: &Value,
    selected_candidate_ids: &HashSet<String>,
    materialized_candidate_ids: &HashSet<String>,
    route_counts: &HashMap<String, usize>,
) -> Value {
    let section = parse_markdown_section_candidate_id(candidate_id);
    let source_path = section.source_path;
    let heading_anchor = section.heading_anchor.unwrap_or("");
    let route_count = route_counts.get(candidate_id).copied().unwrap_or(0);
    json!({
        "projectionSource": "rust-bridge-search-strategy-flow-v1",
        "candidateId": candidate_id,
        "sourcePath": source_path,
        "headingAnchor": heading_anchor,
        "evidenceKind": search_strategy_flow_evidence_kind(source_path, heading_anchor),
        "evidenceCoverage": json_number(candidate, "evidenceCoverage"),
        "graphScore": json_number(candidate, "graphScore"),
        "authorityScore": json_number(candidate, "authorityScore"),
        "structuralScore": json_number(candidate, "structuralScore"),
        "contextCost": json_usize(candidate, "contextCost"),
        "blocked": json_bool(candidate, "blocked"),
        "selected": selected_candidate_ids.contains(candidate_id),
        "plannerMaterialized": materialized_candidate_ids.contains(candidate_id),
        "retrievalRouteCount": route_count,
        "routePlanned": route_count > 0,
        "proofTags": search_strategy_flow_projection_tags(
            source_path,
            selected_candidate_ids.contains(candidate_id),
            materialized_candidate_ids.contains(candidate_id),
            route_count,
        ),
    })
}

fn search_strategy_flow_evidence_kind(source_path: &str, heading_anchor: &str) -> &'static str {
    let combined = format!(
        "{} {}",
        source_path.to_ascii_lowercase(),
        heading_anchor.to_ascii_lowercase()
    );
    if combined.contains("page_index") || combined.contains("page-index") {
        return "page_index_reasoning_tree";
    }
    if combined.contains("graph_compute")
        || combined.contains("link_graph")
        || combined.contains("link-graph")
    {
        return "link_graph_dependency_path";
    }
    if combined.contains("validation") {
        return "validation_guard";
    }
    if combined.contains("notebook") || combined.contains("pluto") {
        return "notebook_validation_surface";
    }
    "search_strategy_flow_authority"
}

fn search_strategy_flow_projection_tags(
    source_path: &str,
    selected: bool,
    materialized: bool,
    route_count: usize,
) -> Vec<&'static str> {
    let mut tags = vec!["rust_projected", "real_document"];
    match search_strategy_flow_evidence_kind(source_path, "") {
        "page_index_reasoning_tree" => tags.push("page_index"),
        "link_graph_dependency_path" => tags.push("link_graph"),
        "validation_guard" => tags.push("negative_guard"),
        "notebook_validation_surface" => tags.push("notebook"),
        _ => tags.push("search_strategy"),
    }
    if selected {
        tags.push("frontier_selected");
    }
    if materialized {
        tags.push("planner_materialized");
    }
    if route_count > 0 {
        tags.push("route_planned");
    }
    tags
}

fn search_strategy_flow_retrieval_route(
    candidate_id: &str,
    candidate_input_source: Option<&str>,
) -> Option<Value> {
    let section = parse_markdown_section_candidate_id(candidate_id);
    section.heading_anchor?;
    let heading_anchor = section.heading_anchor.unwrap_or("");
    let mut route = json!({
        "candidateId": candidate_id,
        "materializationOwner": "studio-rust",
        "materializationStatus": "planned",
        "receiptSource": "rust-bridge",
        "primaryTransport": WENDAO_ARROW_FLIGHT_DATA_PLANE,
        "sourcePath": section.source_path,
        "directFileReadAllowed": false,
        "executeBeforeAnswer": true,
        "evidenceKind": search_strategy_flow_evidence_kind(section.source_path, heading_anchor),
        "flightSteps": search_strategy_flow_flight_steps(&section),
    });
    if let (Some(object), Some(heading_anchor)) = (route.as_object_mut(), section.heading_anchor) {
        object.insert("headingAnchor".to_owned(), json!(heading_anchor));
    }
    if let (Some(object), Some(candidate_input_source)) =
        (route.as_object_mut(), candidate_input_source)
    {
        object.insert(
            "candidateInputSource".to_owned(),
            json!(candidate_input_source),
        );
    }
    Some(route)
}

struct MarkdownSectionCandidate<'a> {
    source_path: &'a str,
    heading_anchor: Option<&'a str>,
}

fn parse_markdown_section_candidate_id(candidate_id: &str) -> MarkdownSectionCandidate<'_> {
    let (source_path, heading_anchor) = candidate_id.split_once('#').map_or(
        (candidate_id, None),
        |(source_path, heading_anchor)| {
            (
                source_path,
                (!heading_anchor.is_empty()).then_some(heading_anchor),
            )
        },
    );
    MarkdownSectionCandidate {
        source_path,
        heading_anchor,
    }
}

fn search_strategy_flow_flight_steps(section: &MarkdownSectionCandidate<'_>) -> Vec<Value> {
    let query = match section.heading_anchor {
        Some(heading_anchor) => format!("{}#{heading_anchor}", section.source_path),
        None => section.source_path.to_owned(),
    };
    let mut page_index_metadata = vec![
        "x-wendao-repo-projected-page-index-tree-repo=<repo>".to_owned(),
        "x-wendao-repo-projected-page-index-tree-page-id=<resolved-page-id>".to_owned(),
    ];
    if let Some(heading_anchor) = section.heading_anchor {
        page_index_metadata.push(format!("candidate-heading-anchor={heading_anchor}"));
    }

    vec![
        json!({
            "step": "flight_search_page",
            "transport": WENDAO_ARROW_FLIGHT_DATA_PLANE,
            "route": "/search/repos/main",
            "metadataTemplates": [
                "x-wendao-repo-search-repo=<repo>",
                format!("x-wendao-repo-search-query={query}"),
                "x-wendao-repo-search-limit=5".to_owned(),
                format!("x-wendao-repo-search-path-prefixes={}", section.source_path),
            ],
            "note": "Resolve the Markdown section candidate to a page hit through native repo search.",
            "requiresResolvedPageId": false,
            "requiresResolvedNodeId": false,
        }),
        json!({
            "step": "flight_resolve_page_index_tree",
            "transport": WENDAO_ARROW_FLIGHT_DATA_PLANE,
            "route": "/analysis/repo-projected-page-index-tree",
            "metadataTemplates": page_index_metadata,
            "note": "Select the concrete page-index node from the returned tree; do not treat the Markdown anchor as the node id.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": false,
        }),
        json!({
            "step": "flight_open_retrieval_context",
            "transport": WENDAO_ARROW_FLIGHT_DATA_PLANE,
            "route": "/analysis/repo-projected-retrieval-context",
            "metadataTemplates": [
                "x-wendao-repo-projected-retrieval-context-repo=<repo>",
                "x-wendao-repo-projected-retrieval-context-page-id=<resolved-page-id>",
                "x-wendao-repo-projected-retrieval-context-node-id=<resolved-node-id>",
                "x-wendao-repo-projected-retrieval-context-related-limit=5",
            ],
            "note": "Open the section-level projected retrieval context through the native Flight route.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": true,
        }),
        json!({
            "step": "flight_expand_graph_context",
            "transport": WENDAO_ARROW_FLIGHT_DATA_PLANE,
            "route": "/graph/neighbors",
            "metadataTemplates": [
                "x-wendao-graph-node-id=<resolved-graph-node-id>",
                "x-wendao-graph-direction=both",
                "x-wendao-graph-hops=2",
                "x-wendao-graph-limit=50",
            ],
            "note": "Expand document-level graph context through the graph relation layer before the next reasoning-tree branch.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": true,
            "requiresResolvedGraphNodeId": true,
        }),
    ]
}

fn json_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn json_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn json_number(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn json_usize(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
}
