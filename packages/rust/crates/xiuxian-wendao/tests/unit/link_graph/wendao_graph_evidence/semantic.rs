use xiuxian_julia_core::validate_wendao_graph_evidence_request_schema;

use super::support::{
    fixture_index, float_column, int64_column, semantic_overlay_edge, string_column,
};
use crate::link_graph::{
    WendaoGraphEvidenceRequestOptions, build_wendao_graph_evidence_request_bundle_with_options,
};

#[test]
fn request_bundle_projects_semantic_neighbors_in_canonical_order() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default().with_semantic_neighbor(
        "docs/alpha",
        "docs/beta",
        1,
        2,
        1,
        0.25,
    );
    let bundle = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    let Some(semantic_neighbors) = bundle.table("semantic_neighbors") else {
        panic!("semantic_neighbors batch should be present");
    };
    validate_wendao_graph_evidence_request_schema(
        "semantic_neighbors",
        semantic_neighbors.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate semantic_neighbors schema: {error}"));
    assert_eq!(
        string_column(semantic_neighbors, 0),
        vec!["docs/alpha".to_string()]
    );
    assert_eq!(
        string_column(semantic_neighbors, 1),
        vec!["docs/beta".to_string()]
    );
    assert_eq!(int64_column(semantic_neighbors, 2), vec![1]);
    assert_eq!(int64_column(semantic_neighbors, 3), vec![2]);
    assert_eq!(int64_column(semantic_neighbors, 4), vec![1]);
    assert_eq!(float_column(semantic_neighbors, 5), vec![0.25]);

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(table_names, vec!["nodes", "edges", "semantic_neighbors"]);
}

#[test]
fn request_bundle_projects_semantic_overlay_in_canonical_order() {
    let index = fixture_index();
    let options = WendaoGraphEvidenceRequestOptions::default()
        .with_semantic_overlay_edge(semantic_overlay_edge("docs/alpha", "docs/beta", 1, 2));
    let bundle = build_wendao_graph_evidence_request_bundle_with_options(&index, &options)
        .unwrap_or_else(|error| panic!("build WendaoGraph request bundle: {error}"));

    let Some(semantic_overlay) = bundle.table("semantic_overlay") else {
        panic!("semantic_overlay batch should be present");
    };
    validate_wendao_graph_evidence_request_schema(
        "semantic_overlay",
        semantic_overlay.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate semantic_overlay schema: {error}"));
    assert_eq!(
        string_column(semantic_overlay, 0),
        vec!["docs/alpha".to_string()]
    );
    assert_eq!(
        string_column(semantic_overlay, 1),
        vec!["docs/beta".to_string()]
    );
    assert_eq!(int64_column(semantic_overlay, 2), vec![1]);
    assert_eq!(int64_column(semantic_overlay, 3), vec![2]);
    assert_eq!(int64_column(semantic_overlay, 4), vec![1]);
    assert_eq!(float_column(semantic_overlay, 5), vec![0.25]);
    assert_eq!(float_column(semantic_overlay, 6), vec![0.8]);
    assert_eq!(
        string_column(semantic_overlay, 7),
        vec!["semantic".to_string()]
    );

    let table_names = bundle
        .record_batches()
        .into_iter()
        .map(|(table_name, _)| table_name)
        .collect::<Vec<_>>();
    assert_eq!(table_names, vec!["nodes", "edges", "semantic_overlay"]);
}
