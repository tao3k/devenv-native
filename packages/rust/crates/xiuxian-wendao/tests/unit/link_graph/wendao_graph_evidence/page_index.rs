use xiuxian_julia_core::validate_wendao_graph_page_index_reasoning_request_schema;

use super::support::{
    fixture_index, float_column, int64_column, page_index_edge_rows, page_index_node_rows,
    page_index_seed_rows, read_tsv_rows, string_column,
};
use crate::link_graph::{
    WendaoGraphPageIndexReasoningRequestOptions, build_wendao_graph_evidence_request_bundle,
    build_wendao_graph_page_index_reasoning_request_bundle,
    build_wendao_graph_page_index_reasoning_request_bundle_with_options,
};

#[test]
fn page_index_reasoning_host_fixture_matches_builder_output() {
    let index = fixture_index();
    let options = WendaoGraphPageIndexReasoningRequestOptions::default().with_seed(
        "docs/alpha#alpha",
        1.0,
        "query_match",
    );
    let bundle =
        build_wendao_graph_page_index_reasoning_request_bundle_with_options(&index, &options)
            .unwrap_or_else(|error| panic!("build seeded PageIndex bundle: {error}"));

    assert_eq!(
        page_index_node_rows(&bundle.nodes),
        read_tsv_rows("page_index_nodes.tsv")
    );
    assert_eq!(
        page_index_edge_rows(&bundle.edges),
        read_tsv_rows("page_index_edges.tsv")
    );
    assert_eq!(
        page_index_seed_rows(&bundle.seeds),
        read_tsv_rows("page_index_seeds.tsv")
    );
}

#[test]
fn page_index_reasoning_bundle_projects_wendaograph_sidecar_tables() {
    let index = fixture_index();
    let bundle = build_wendao_graph_page_index_reasoning_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build WendaoGraph PageIndex request bundle: {error}"));

    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_nodes",
        bundle.nodes.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_nodes schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_edges",
        bundle.edges.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_edges schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_seeds",
        bundle.seeds.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_seeds schema: {error}"));

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(
        table_names,
        vec!["page_index_nodes", "page_index_edges", "page_index_seeds"]
    );

    let node_ids = string_column(&bundle.nodes, 0);
    let page_ids = string_column(&bundle.nodes, 1);
    let parent_ids = string_column(&bundle.nodes, 2);
    let depths = int64_column(&bundle.nodes, 3);
    let line_starts = int64_column(&bundle.nodes, 7);
    let token_counts = int64_column(&bundle.nodes, 9);
    assert!(!node_ids.is_empty());
    assert!(page_ids.iter().any(|page_id| page_id == "docs/alpha"));
    assert!(parent_ids.iter().any(String::is_empty));
    assert!(depths.iter().all(|depth| *depth >= 0));
    assert!(line_starts.iter().all(|line_start| *line_start > 0));
    assert!(token_counts.iter().all(|token_count| *token_count > 0));

    let edge_kinds = string_column(&bundle.edges, 2);
    assert!(edge_kinds.iter().all(|edge_kind| edge_kind == "hierarchy"));
    assert_eq!(bundle.seeds.num_rows(), 0);

    let link_bundle = build_wendao_graph_evidence_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build link evidence bundle: {error}"));
    assert!(link_bundle.table("page_index_nodes").is_none());
}

#[test]
fn page_index_reasoning_bundle_projects_valid_seed_rows() {
    let index = fixture_index();
    let seed_source = build_wendao_graph_page_index_reasoning_request_bundle(&index)
        .unwrap_or_else(|error| panic!("build PageIndex source bundle: {error}"));
    let seed_node = string_column(&seed_source.nodes, 0)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("fixture should produce at least one PageIndex node"));
    let options = WendaoGraphPageIndexReasoningRequestOptions::default().with_seed(
        seed_node.clone(),
        0.75,
        "query_match",
    );
    let seeded =
        build_wendao_graph_page_index_reasoning_request_bundle_with_options(&index, &options)
            .unwrap_or_else(|error| panic!("build seeded PageIndex bundle: {error}"));

    assert_eq!(string_column(&seeded.seeds, 0), vec![seed_node]);
    assert_eq!(float_column(&seeded.seeds, 1), vec![0.75]);
    assert_eq!(
        string_column(&seeded.seeds, 2),
        vec!["query_match".to_string()]
    );
}
