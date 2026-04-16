use std::{fs, path::Path};

use tempfile::TempDir;
use xiuxian_wendao::{LinkGraphAgenticExpansionConfig, LinkGraphIndex};

#[cfg(feature = "julia")]
pub(super) use arrow::array::{Float64Array, Int32Array, ListArray, StringArray};
#[cfg(feature = "julia")]
pub(super) use xiuxian_wendao::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};
#[cfg(feature = "julia")]
pub(super) use xiuxian_wendao_builtin::{
    GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN, GraphStructuralFilterRequestRow,
    build_graph_structural_filter_request_batch,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    linked_builtin_spawn_wendaosearch_solver_demo_multi_route_service,
    linked_builtin_spawn_wendaosearch_solver_demo_structural_rerank_service,
};

#[cfg(feature = "julia")]
pub(super) use super::super::expansion_support::{
    GenericTopologyCandidateBuildOptions, GenericTopologyCandidateScores,
    assert_solver_demo_generic_topology_row_basics,
    assert_solver_demo_generic_topology_row_infeasible,
    assert_solver_demo_generic_topology_row_shape, build_pair_rerank_request_batch,
    build_raw_connected_pair_collection_candidate_from_pairs,
    build_raw_connected_pair_collection_candidates_from_plan,
    build_raw_seed_centered_pair_collection_candidates_from_plan,
    build_worker_partition_generic_topology_candidate_fixtures_from_plan,
    default_agentic_execution_relation_edge_kind,
    fetch_generic_topology_rows_via_manifest_discovery, first_connected_pair_collection,
    first_worker_pair, required_column,
};

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) struct AgenticIndexFixture {
    pub(super) _tmp: TempDir,
    pub(super) index: LinkGraphIndex,
}

pub(super) fn write_file(path: &Path, content: &str) -> TestResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

pub(super) fn build_index(root: &Path) -> Result<LinkGraphIndex, Box<dyn std::error::Error>> {
    LinkGraphIndex::build(root).map_err(Box::<dyn std::error::Error>::from)
}

pub(super) fn build_index_fixture(
    files: &[(&str, &str)],
) -> Result<AgenticIndexFixture, Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    for (relative_path, content) in files {
        write_file(&tmp.path().join(relative_path), content)?;
    }
    let index = build_index(tmp.path())?;
    Ok(AgenticIndexFixture { _tmp: tmp, index })
}

pub(super) fn expansion_config(
    max_workers: usize,
    max_candidates: usize,
    max_pairs_per_worker: usize,
) -> LinkGraphAgenticExpansionConfig {
    LinkGraphAgenticExpansionConfig {
        max_workers,
        max_candidates,
        max_pairs_per_worker,
        time_budget_ms: 1_000.0,
    }
}
