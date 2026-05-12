//! Host-process probes for local `WendaoGraph.jl` contracts.

use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};

use super::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInputBatch, search_strategy_flow_candidate_discovery_contract_json,
    search_strategy_flow_candidate_input_batch_from_markdown,
    search_strategy_flow_total_structured_candidate_index_contract_json,
};
#[cfg(test)]
use super::search_strategy_flow_candidates::{
    SearchStrategyFlowConfiguredMarkdownReplayFamily,
    configured_search_strategy_flow_markdown_replay_families,
    configured_search_strategy_flow_markdown_replay_families_with_limit,
};
use super::search_strategy_flow_flight::{
    SearchStrategyFlowFlightMaterializationConfig, materialize_search_strategy_flow_routes,
    search_strategy_flow_candidate_input_batch_from_repo_search,
};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_ROUTE,
};

const WENDAOGRAPH_PACKAGE_DIR_ENV: &str = "WENDAOGRAPH_PACKAGE_DIR";
const WENDAOGRAPH_JULIA_PROJECT_ENV: &str = "WENDAOGRAPH_JULIA_PROJECT";
const WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV: &str = "WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES";
const WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV: &str = "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS";
const PAGE_INDEX_HOST_PROBE_PREFIX: &str = "wendaograph_page_index_host_probe";
const LINK_GRAPH_HOST_PROBE_PREFIX: &str = "wendaograph_link_graph_host_probe";

/// Future `SearchStrategyFlow` probe action admitted by the Rust bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStrategyFlowProbeAction {
    /// Expand relation context through graph-neighbor materialization.
    ExpandNeighbors,
    /// Open `PageIndex` parent and child section context.
    OpenParentChild,
    /// Compare source provenance before exposing a branch to an Agent.
    CompareProvenance,
    /// Open adjacent sections around the selected section candidate.
    OpenAdjacentSections,
    /// Stop the branch without executing a materialization route.
    Stop,
}

/// Parses one `SearchStrategyFlow` probe action accepted by Rust.
///
/// # Errors
///
/// Returns an error when the action is not in the explicit Rust whitelist.
pub fn parse_search_strategy_flow_probe_action(
    action: &str,
) -> Result<SearchStrategyFlowProbeAction, String> {
    let action_kind = action
        .trim()
        .split_once(':')
        .map_or(action.trim(), |(kind, _)| kind);
    match action_kind {
        "expand_neighbors" => Ok(SearchStrategyFlowProbeAction::ExpandNeighbors),
        "open_parent_child" => Ok(SearchStrategyFlowProbeAction::OpenParentChild),
        "compare_provenance" => Ok(SearchStrategyFlowProbeAction::CompareProvenance),
        "open_adjacent_sections" => Ok(SearchStrategyFlowProbeAction::OpenAdjacentSections),
        "stop" => Ok(SearchStrategyFlowProbeAction::Stop),
        other => Err(format!(
            "SearchStrategyFlow probe action `{other}` is not whitelisted"
        )),
    }
}

/// Returns the native Flight route family for one whitelisted probe action.
///
/// # Errors
///
/// Returns an error when the action is not in the explicit Rust whitelist.
pub fn search_strategy_flow_probe_action_route(
    action: &str,
) -> Result<Option<&'static str>, String> {
    match parse_search_strategy_flow_probe_action(action)? {
        SearchStrategyFlowProbeAction::ExpandNeighbors => Ok(Some(GRAPH_NEIGHBORS_ROUTE)),
        SearchStrategyFlowProbeAction::OpenParentChild => {
            Ok(Some(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE))
        }
        SearchStrategyFlowProbeAction::CompareProvenance => Ok(Some(REPO_SEARCH_ROUTE)),
        SearchStrategyFlowProbeAction::OpenAdjacentSections => {
            Ok(Some(ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE))
        }
        SearchStrategyFlowProbeAction::Stop => Ok(None),
    }
}

#[path = "wendaograph_scripts.rs"]
mod scripts;

#[path = "wendaograph_persistent_host_report.rs"]
mod persistent_host_report;

#[path = "wendaograph_batch_replay.rs"]
mod batch_replay;

use scripts::SEARCH_STRATEGY_FLOW_JULIA;

pub use batch_replay::{
    SearchStrategyFlowPersistentBatchHost,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
};
pub use persistent_host_report::{
    SearchStrategyFlowPersistentHostStabilizationLimits,
    SearchStrategyFlowPersistentHostStabilizationReason,
    SearchStrategyFlowPersistentHostStabilizationReport,
    SearchStrategyFlowPersistentHostWarmPathStats,
};

#[path = "wendaograph_probes.rs"]
mod probes;

pub use probes::{
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
    probe_wendaograph_link_graph_full_structural_host_request,
    probe_wendaograph_link_graph_host_request, probe_wendaograph_page_index_host_request,
    probe_wendaograph_page_index_host_request_with_fixture,
    probe_wendaograph_page_index_planner_action_host_request,
    probe_wendaograph_page_index_planner_action_host_request_with_fixture,
};
#[cfg(test)]
pub(crate) use probes::{
    parse_link_graph_full_structural_probe_report_line, parse_link_graph_probe_report_line,
    parse_page_index_planner_action_probe_report_line, parse_page_index_probe_report_line,
};
pub(crate) use probes::{resolve_existing_path, wendaograph_julia_project};

/// Adds Rust-owned `SearchStrategyFlow` retrieval-route plans to a
/// `WendaoGraph.jl` trace.
///
/// The Julia side remains the owner of query understanding, graph scoring,
/// frontier pruning, and planner actions. This helper derives the Studio
/// Flight route contract from selected/planned candidates so downstream
/// `pi-wendao` execution can consume a single bridge trace without treating a
/// local fixture or static row count as executed materialization.
///
/// # Errors
///
/// Returns an error when the supplied trace is not valid JSON, the JSON root is
/// not an object, or the enriched trace cannot be serialized.
pub fn enrich_wendaograph_search_strategy_flow_retrieval_routes(
    trace: &str,
) -> Result<String, String> {
    let mut value = serde_json::from_str::<Value>(trace)
        .map_err(|error| format!("parse WendaoGraph SearchStrategyFlow JSON trace: {error}"))?;
    add_search_strategy_flow_retrieval_routes(&mut value)?;
    serialize_search_strategy_flow_trace(&value)
}

/// Adds Rust-owned `SearchStrategyFlow` retrieval-route plans to a trace, then
/// executes them through a real Arrow Flight endpoint.
///
/// # Errors
///
/// Returns an error when the supplied trace is invalid JSON, route enrichment
/// fails, the endpoint cannot be reached, or a route cannot be decoded into
/// evidence receipts.
pub async fn enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
    trace: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<String, String> {
    let mut value = serde_json::from_str::<Value>(trace)
        .map_err(|error| format!("parse WendaoGraph SearchStrategyFlow JSON trace: {error}"))?;
    add_search_strategy_flow_retrieval_routes(&mut value)?;
    materialize_search_strategy_flow_routes(&mut value, config).await?;
    serialize_search_strategy_flow_trace(&value)
}

/// Runs local `WendaoGraph.jl` `SearchStrategyFlow` through the Rust owner
/// bridge and returns the JSON trace emitted by Julia.
///
/// # Errors
///
/// Returns an error when the intent is blank, the local `WendaoGraph.jl`
/// project or search root cannot be resolved, the Julia process exits
/// unsuccessfully, or the trace is not valid JSON.
pub fn run_wendaograph_search_strategy_flow_json(
    intent: &str,
    search_root: impl Into<PathBuf>,
) -> Result<String, String> {
    let trace = run_wendaograph_search_strategy_flow_raw_json(intent, search_root)?;
    enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace)
}

#[cfg(test)]
pub(crate) fn run_wendaograph_search_strategy_flow_json_with_candidate_batch(
    intent: &str,
    search_root: impl Into<PathBuf>,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
) -> Result<String, String> {
    let trace = run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
        intent,
        search_root,
        candidate_batch,
    )?;
    enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace)
}

#[cfg(test)]
pub(crate) fn configured_wendaograph_search_strategy_flow_markdown_replay_families(
    search_root: impl Into<PathBuf>,
    intent: &str,
) -> Result<Vec<SearchStrategyFlowConfiguredMarkdownReplayFamily>, String> {
    let search_root = resolve_existing_path(
        "WendaoGraph SearchStrategyFlow configured Markdown replay root",
        search_root,
    )?;
    configured_search_strategy_flow_markdown_replay_families(search_root.as_path(), intent)
}

#[cfg(test)]
pub(crate) fn configured_wendaograph_search_strategy_flow_markdown_replay_families_with_limit(
    search_root: impl Into<PathBuf>,
    intent: &str,
    max_candidates: usize,
) -> Result<Vec<SearchStrategyFlowConfiguredMarkdownReplayFamily>, String> {
    let search_root = resolve_existing_path(
        "WendaoGraph SearchStrategyFlow configured Markdown replay root",
        search_root,
    )?;
    configured_search_strategy_flow_markdown_replay_families_with_limit(
        search_root.as_path(),
        intent,
        Some(max_candidates),
    )
}

/// Runs local `WendaoGraph.jl` `SearchStrategyFlow` through the Rust owner
/// bridge, optionally executes the planned native Flight route sequence, and
/// returns the JSON trace emitted by Julia.
///
/// # Errors
///
/// Returns an error when the Julia host request fails, route enrichment fails,
/// or configured Flight materialization cannot decode route evidence.
pub async fn run_wendaograph_search_strategy_flow_json_with_flight_materialization(
    intent: &str,
    search_root: impl Into<PathBuf>,
    config: Option<SearchStrategyFlowFlightMaterializationConfig>,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;
    let search_root = search_root.into();
    let trace = match config.as_ref() {
        Some(config) => {
            let candidate_batch =
                search_strategy_flow_candidate_input_batch_from_repo_search(intent, config).await?;
            run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
                intent,
                search_root.as_path(),
                candidate_batch,
            )?
        }
        None => run_wendaograph_search_strategy_flow_raw_json(intent, search_root.as_path())?,
    };
    match config {
        Some(config) => {
            enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
                &trace, &config,
            )
            .await
        }
        None => enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace),
    }
}

fn run_wendaograph_search_strategy_flow_raw_json(
    intent: &str,
    search_root: impl Into<PathBuf>,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;
    let search_root = search_root.into();
    let search_root =
        resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
    let candidate_batch =
        search_strategy_flow_candidate_input_batch_from_markdown(intent, search_root.as_path())?;
    run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
        intent,
        search_root.as_path(),
        candidate_batch,
    )
}

fn run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
    intent: &str,
    search_root: impl Into<PathBuf>,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;

    let julia_project = wendaograph_julia_project()?;
    let search_root =
        resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
    debug_assert_eq!(
        candidate_batch.row_count,
        candidate_batch
            .tsv
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    );
    let julia_command = env::var("JULIA").unwrap_or_else(|_| "julia".to_owned());
    let output = Command::new(julia_command)
        .arg(format!("--project={}", julia_project.display()))
        .arg("--startup-file=no")
        .arg("-e")
        .arg(SEARCH_STRATEGY_FLOW_JULIA)
        .arg(intent)
        .arg(search_root)
        .arg(candidate_batch.tsv)
        .arg(candidate_batch.source)
        .arg(candidate_batch.discovery_receipt_json)
        .output()
        .map_err(|error| format!("spawn WendaoGraph SearchStrategyFlow host request: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph SearchStrategyFlow host request exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trace = stdout.trim();
    if trace.is_empty() {
        return Err("WendaoGraph SearchStrategyFlow host request returned empty output".to_owned());
    }
    Ok(trace.to_owned())
}

fn validate_search_strategy_flow_intent(intent: &str) -> Result<(), String> {
    if intent.trim().is_empty() {
        return Err("SearchStrategyFlow intent must not be blank".to_owned());
    }
    Ok(())
}

fn add_search_strategy_flow_retrieval_routes(value: &mut Value) -> Result<(), String> {
    let routes = build_search_strategy_flow_retrieval_routes(value);
    let projected_evidence_rows =
        build_search_strategy_flow_rust_projected_evidence_rows(value, &routes);
    let candidate_discovery_contract = search_strategy_flow_candidate_discovery_contract_json(
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
        search_strategy_flow_total_structured_candidate_index_contract_json(),
    );
    object.insert(
        "candidateDiscoveryContract".to_owned(),
        candidate_discovery_contract,
    );
    Ok(())
}

fn serialize_search_strategy_flow_trace(value: &Value) -> Result<String, String> {
    serde_json::to_string(value)
        .map(|trace| format!("{trace}\n"))
        .map_err(|error| format!("serialize enriched SearchStrategyFlow JSON trace: {error}"))
}

fn build_search_strategy_flow_retrieval_routes(trace: &Value) -> Vec<Value> {
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
        .filter_map(search_strategy_flow_retrieval_route)
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

fn search_strategy_flow_retrieval_route(candidate_id: &str) -> Option<Value> {
    let section = parse_markdown_section_candidate_id(candidate_id);
    section.heading_anchor?;
    let mut route = json!({
        "candidateId": candidate_id,
        "materializationOwner": "studio-rust",
        "materializationStatus": "planned",
        "receiptSource": "rust-bridge",
        "primaryTransport": "arrow-flight",
        "sourcePath": section.source_path,
        "directFileReadAllowed": false,
        "executeBeforeAnswer": true,
        "flightSteps": search_strategy_flow_flight_steps(&section),
    });
    if let (Some(object), Some(heading_anchor)) = (route.as_object_mut(), section.heading_anchor) {
        object.insert("headingAnchor".to_owned(), json!(heading_anchor));
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
            "transport": "arrow-flight",
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
            "transport": "arrow-flight",
            "route": "/analysis/repo-projected-page-index-tree",
            "metadataTemplates": page_index_metadata,
            "note": "Select the concrete page-index node from the returned tree; do not treat the Markdown anchor as the node id.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": false,
        }),
        json!({
            "step": "flight_open_retrieval_context",
            "transport": "arrow-flight",
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
            "transport": "arrow-flight",
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

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaograph/mod.rs"]
mod tests;
