//! Host-process entrypoints for local `WendaoGraph.jl` contracts.

use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use serde_json::Value;
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_ROUTE,
};

use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInputBatch, search_strategy_flow_candidate_input_batch_from_markdown,
};
#[cfg(test)]
use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowConfiguredMarkdownReplayFamily,
    configured_search_strategy_flow_markdown_replay_families,
    configured_search_strategy_flow_markdown_replay_families_with_limit,
};
use crate::integration_support::search_strategy_flow_flight::{
    SearchStrategyFlowArrowIpcFile, SearchStrategyFlowFlightMaterializationConfig,
    SearchStrategyFlowServiceFlightBindingOptions, SearchStrategyFlowServiceRequestOptions,
    materialize_search_strategy_flow_routes,
    roundtrip_search_strategy_flow_frontier_with_service_request,
    search_strategy_flow_candidate_input_batch_from_repo_search,
    search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope,
};

use super::probes::{resolve_existing_path, wendaograph_julia_project};
use super::scripts::SEARCH_STRATEGY_FLOW_JULIA;
use super::search_strategy_routes::add_search_strategy_flow_retrieval_routes;
use super::{SearchStrategyFlowServiceTraceRequest, search_strategy_flow_service_trace_json};

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
        "",
        "",
        "",
    )?;
    enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace)
}

#[cfg(test)]
pub(crate) fn run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements(
    intent: &str,
    search_root: impl Into<PathBuf>,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
    branch_judgements_arrow_ipc_path: &str,
) -> Result<String, String> {
    let trace = run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
        intent,
        search_root,
        candidate_batch,
        "",
        branch_judgements_arrow_ipc_path,
        "",
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
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_branch_judgements(
        intent,
        search_root,
        config,
        "",
    )
    .await
}

/// Runs local `WendaoGraph.jl` `SearchStrategyFlow` through the Rust owner
/// bridge with optional Agent branch-judgement rows.
///
/// # Errors
///
/// Returns an error when the Julia host request fails, route enrichment fails,
/// or configured Flight materialization cannot decode route evidence.
pub async fn run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_branch_judgements(
    intent: &str,
    search_root: impl Into<PathBuf>,
    config: Option<SearchStrategyFlowFlightMaterializationConfig>,
    branch_judgements_arrow_ipc_path: &str,
) -> Result<String, String> {
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_side_tables(
        intent,
        search_root,
        config,
        "",
        branch_judgements_arrow_ipc_path,
    )
    .await
}

/// Runs local `WendaoGraph.jl` `SearchStrategyFlow` through the Rust owner
/// bridge with optional query-understanding and Agent branch-judgement rows.
///
/// # Errors
///
/// Returns an error when the Julia host request fails, route enrichment fails,
/// or configured Flight materialization cannot decode route evidence.
pub async fn run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_side_tables(
    intent: &str,
    search_root: impl Into<PathBuf>,
    config: Option<SearchStrategyFlowFlightMaterializationConfig>,
    query_understanding_arrow_ipc_path: &str,
    branch_judgements_arrow_ipc_path: &str,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;
    let search_root = search_root.into();
    let trace = match config.as_ref() {
        Some(config) => {
            let candidate_batch =
                search_strategy_flow_candidate_input_batch_from_repo_search(intent, config).await?;
            let ontology_registry_payload =
                search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope(config)
                    .await?;
            let ontology_registry_file = SearchStrategyFlowArrowIpcFile::write(
                "ontology-registry",
                &ontology_registry_payload,
            )?;
            let ontology_registry_arrow_ipc_path =
                ontology_registry_file.path().to_string_lossy().into_owned();
            run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
                intent,
                search_root.as_path(),
                candidate_batch,
                query_understanding_arrow_ipc_path,
                branch_judgements_arrow_ipc_path,
                ontology_registry_arrow_ipc_path.as_str(),
            )?
        }
        None => run_wendaograph_search_strategy_flow_raw_json_with_side_tables(
            intent,
            search_root.as_path(),
            query_understanding_arrow_ipc_path,
            branch_judgements_arrow_ipc_path,
        )?,
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

/// Request for the production `SearchStrategyFlow` service data plane.
#[derive(Debug, Clone)]
pub(crate) struct SearchStrategyFlowServiceExecutionRequest {
    pub(crate) intent: String,
    pub(crate) search_root: PathBuf,
    pub(crate) flight_materialization_config: Option<SearchStrategyFlowFlightMaterializationConfig>,
    pub(crate) strategy_flow_service_base_url: String,
    pub(crate) strategy_flow_service_timeout_seconds: u64,
    pub(crate) query_understanding_arrow_ipc_path: String,
    pub(crate) branch_judgements_arrow_ipc_path: String,
}

/// Runs `SearchStrategyFlow` through the Julia Flight service and returns the
/// benchmark trace used by `pi-wendao`.
///
/// # Errors
///
/// Returns an error when candidate discovery, side-table loading, service
/// roundtrip, route enrichment, or Flight materialization fails.
pub(crate) async fn run_wendaograph_search_strategy_flow_json_with_service_and_flight_materialization(
    request: SearchStrategyFlowServiceExecutionRequest,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(request.intent.as_str())?;
    let search_root = request.search_root;
    let query_understanding_payload = read_optional_arrow_ipc_payload(
        "query-understanding",
        request.query_understanding_arrow_ipc_path.as_str(),
    )?;
    let branch_judgements_payload = read_optional_arrow_ipc_payload(
        "branch-judgements",
        request.branch_judgements_arrow_ipc_path.as_str(),
    )?;
    let (candidate_batch, ontology_registry_payload) =
        match request.flight_materialization_config.as_ref() {
            Some(config) => {
                let candidate_batch = search_strategy_flow_candidate_input_batch_from_repo_search(
                    request.intent.as_str(),
                    config,
                )
                .await?;
                let ontology_registry_payload =
                    search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope(config)
                        .await?;
                (candidate_batch, Some(ontology_registry_payload))
            }
            None => (
                search_strategy_flow_candidate_input_batch_from_markdown(
                    request.intent.as_str(),
                    search_root.as_path(),
                )?,
                None,
            ),
        };

    let mut request_options = SearchStrategyFlowServiceRequestOptions::default();
    if let Some(payload) = query_understanding_payload.clone() {
        request_options = request_options.with_query_understanding_arrow_ipc_stream(payload);
    }
    if let Some(payload) = ontology_registry_payload.clone() {
        request_options = request_options.with_ontology_registry_arrow_ipc_stream(payload);
    }
    if let Some(payload) = branch_judgements_payload {
        request_options = request_options.with_branch_judgements_arrow_ipc_stream(payload);
    }
    let service_options = SearchStrategyFlowServiceFlightBindingOptions::new(
        request.strategy_flow_service_base_url.as_str(),
    )
    .map_err(|error| format!("invalid SearchStrategyFlow service Flight config: {error}"))?
    .with_timeout_seconds(request.strategy_flow_service_timeout_seconds);
    let service_roundtrip = roundtrip_search_strategy_flow_frontier_with_service_request(
        &candidate_batch,
        request_options,
        service_options,
    )
    .await?;
    let trace = search_strategy_flow_service_trace_json(&SearchStrategyFlowServiceTraceRequest {
        intent: request.intent.as_str(),
        search_root: search_root.as_path(),
        candidate_batch: &candidate_batch,
        service_base_url: request.strategy_flow_service_base_url.as_str(),
        service_flight_route: service_roundtrip.flight_route.as_str(),
        service_timeout_seconds: request.strategy_flow_service_timeout_seconds,
        response: &service_roundtrip.response,
        query_understanding_payload: query_understanding_payload.as_deref(),
        ontology_registry_payload: ontology_registry_payload.as_deref(),
    })?;
    match request.flight_materialization_config {
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
    run_wendaograph_search_strategy_flow_raw_json_with_side_tables(intent, search_root, "", "")
}

fn run_wendaograph_search_strategy_flow_raw_json_with_side_tables(
    intent: &str,
    search_root: impl Into<PathBuf>,
    query_understanding_arrow_ipc_path: &str,
    branch_judgements_arrow_ipc_path: &str,
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
        query_understanding_arrow_ipc_path,
        branch_judgements_arrow_ipc_path,
        "",
    )
}

fn run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
    intent: &str,
    search_root: impl Into<PathBuf>,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
    query_understanding_arrow_ipc_path: &str,
    branch_judgements_arrow_ipc_path: &str,
    ontology_registry_arrow_ipc_path: &str,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;

    let julia_project = wendaograph_julia_project()?;
    let search_root =
        resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
    let candidate_file = SearchStrategyFlowArrowIpcFile::write(
        "strategy-candidates",
        &candidate_batch.candidate_input_arrow_ipc_stream,
    )?;
    let candidate_arrow_ipc_path = candidate_file.path().to_string_lossy().into_owned();
    let julia_command = env::var("JULIA").unwrap_or_else(|_| "julia".to_owned());
    let output = Command::new(julia_command)
        .arg(format!("--project={}", julia_project.display()))
        .arg("--startup-file=no")
        .arg("-e")
        .arg(SEARCH_STRATEGY_FLOW_JULIA)
        .arg(intent)
        .arg(search_root)
        .arg(candidate_arrow_ipc_path)
        .arg(candidate_batch.source)
        .arg(candidate_batch.discovery_receipt_json)
        .arg(branch_judgements_arrow_ipc_path)
        .arg(ontology_registry_arrow_ipc_path)
        .arg(query_understanding_arrow_ipc_path)
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

fn read_optional_arrow_ipc_payload(label: &str, path: &str) -> Result<Option<Vec<u8>>, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let payload = fs::read(trimmed).map_err(|error| {
        format!("read SearchStrategyFlow {label} Arrow IPC file `{trimmed}`: {error}")
    })?;
    if payload.is_empty() {
        return Err(format!(
            "SearchStrategyFlow {label} Arrow IPC file `{trimmed}` is empty"
        ));
    }
    Ok(Some(payload))
}

pub(crate) fn validate_search_strategy_flow_intent(intent: &str) -> Result<(), String> {
    if intent.trim().is_empty() {
        return Err("SearchStrategyFlow intent must not be blank".to_owned());
    }
    Ok(())
}

fn serialize_search_strategy_flow_trace(value: &Value) -> Result<String, String> {
    serde_json::to_string(value)
        .map(|trace| format!("{trace}\n"))
        .map_err(|error| format!("serialize enriched SearchStrategyFlow JSON trace: {error}"))
}
