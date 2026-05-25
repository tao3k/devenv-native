use super::{
    SearchStrategyFlowCandidateInput, SearchStrategyFlowCandidateInputBatch,
    SearchStrategyFlowFlightMaterializationConfig, SearchStrategyFlowPersistentBatchHost,
    SearchStrategyFlowPersistentHostStabilizationLimits,
    SearchStrategyFlowPersistentHostStabilizationReason,
    SearchStrategyFlowPersistentHostStabilizationReport,
    SearchStrategyFlowPersistentHostWarmPathStats, WENDAOGRAPH_PACKAGE_DIR_ENV,
    WendaoGraphLinkGraphFullStructuralHostProbeReport, WendaoGraphLinkGraphHostProbeReport,
    WendaoGraphPageIndexHostProbeReport, WendaoGraphPageIndexPlannerActionHostProbeReport,
    configured_wendaograph_search_strategy_flow_markdown_replay_families,
    configured_wendaograph_search_strategy_flow_markdown_replay_families_with_limit,
    enrich_wendaograph_search_strategy_flow_retrieval_routes,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    parse_link_graph_full_structural_probe_report_line, parse_link_graph_probe_report_line,
    parse_page_index_planner_action_probe_report_line, parse_page_index_probe_report_line,
    parse_search_strategy_flow_probe_action,
    probe_wendaograph_link_graph_full_structural_host_request,
    probe_wendaograph_page_index_host_request,
    probe_wendaograph_page_index_planner_action_host_request,
    run_wendaograph_search_strategy_flow_json,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization,
    search_strategy_flow_candidate_input_batch,
    search_strategy_flow_candidate_input_batch_from_repo_search,
    search_strategy_flow_probe_action_route,
    search_strategy_flow_registry_authority_candidate_input_batch,
};
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

const RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_PAGE_INDEX_HOST_PROBE_TEST";
const RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_LINK_GRAPH_HOST_PROBE_TEST";
const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS";

mod live_probes;
mod ontology_read_model;
mod relationship_search;
mod reports;
mod search_strategy;

#[test]
fn wendaograph_bridge_source_and_tests_use_runtime_arrow_data_plane_constant() {
    let mut offenders = Vec::new();
    for source_root in [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/integration_support"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/unit"),
    ] {
        collect_arrow_flight_literal_offenders(&source_root, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "WendaoGraph bridge source and tests must import the runtime data-plane token instead of spelling {WENDAO_ARROW_FLIGHT_DATA_PLANE:?}: {offenders:?}"
    );
}

fn collect_arrow_flight_literal_offenders(path: &Path, offenders: &mut Vec<PathBuf>) {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => panic!("failed to inspect {}: {error}", path.display()),
    };
    if metadata.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) => panic!("failed to read {}: {error}", path.display()),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => panic!(
                    "failed to read directory entry in {}: {error}",
                    path.display()
                ),
            };
            collect_arrow_flight_literal_offenders(&entry.path(), offenders);
        }
        return;
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return;
    }
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => panic!("failed to read {}: {error}", path.display()),
    };
    let token_literal = format!("\"{WENDAO_ARROW_FLIGHT_DATA_PLANE}\"");
    if source.contains(token_literal.as_str()) {
        offenders.push(path.to_path_buf());
    }
}
