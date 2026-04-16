use crate::query_core::operators::{
    GraphDirection, GraphNeighborsOp, RetrievalCorpus, VectorSearchOp,
};
use crate::query_core::telemetry::WendaoExplainEvent;
use crate::query_core::{WendaoBackendKind, WendaoOperatorKind, explain_events_summary};

#[test]
fn vector_search_op_defaults_are_stable() {
    let op = VectorSearchOp::default();
    assert_eq!(op.limit, 10);
    assert_eq!(op.corpus, RetrievalCorpus::RepoContent);
    assert!(op.repo_id.is_empty());
    assert!(op.search_term.is_empty());
    assert!(op.kind_filters.is_empty());
}

#[test]
fn graph_neighbors_op_defaults_are_stable() {
    let op = GraphNeighborsOp::default();
    assert_eq!(op.direction, GraphDirection::Both);
    assert_eq!(op.hops, 1);
    assert_eq!(op.limit, 20);
}

#[test]
fn explain_events_summary_captures_operator_and_row_counts() {
    let summary = explain_events_summary(&[WendaoExplainEvent {
        operator_kind: WendaoOperatorKind::GraphNeighbors,
        backend_kind: WendaoBackendKind::LinkGraphBackend,
        legacy_adapter: true,
        input_row_count: Some(1),
        output_row_count: Some(3),
        payload_fetch: false,
        narrow_phase_surviving_count: None,
        payload_phase_fetched_count: None,
        note: Some("link-graph backend".to_string()),
    }]);

    assert!(summary.contains("operator=GraphNeighbors"));
    assert!(summary.contains("backend=LinkGraphBackend"));
    assert!(summary.contains("rows=1->3"));
}
