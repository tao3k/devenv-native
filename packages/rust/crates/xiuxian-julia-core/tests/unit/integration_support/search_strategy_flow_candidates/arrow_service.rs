use std::io::Cursor;

use arrow::array::{Array, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow_ipc::reader::StreamReader;

use super::{
    SearchStrategyFlowCandidateInput, search_strategy_flow_candidate_inputs_arrow_ipc,
    search_strategy_flow_candidate_inputs_arrow_record_batch,
};

#[test]
fn search_strategy_flow_candidate_rows_build_arrow_service_batch() {
    let candidates = fixture_candidates();

    let batch = must(
        search_strategy_flow_candidate_inputs_arrow_record_batch(&candidates),
        "candidate rows should build Arrow batch",
    );

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().fields().len(), 18);
    assert_eq!(batch.schema().fields()[0].name(), "candidate_id");
    assert_eq!(batch.schema().fields()[17].name(), "blocked");
    assert_eq!(
        batch.schema().metadata().get("wendao.schema_version"),
        Some(&"xiuxian_wendao.graph.search_strategy_flow.service.v1".to_owned())
    );
    assert_eq!(
        string_value(&batch, "candidate_id", 0),
        "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary"
    );
    assert_eq!(
        string_value(&batch, "candidate_kind", 1),
        "intent_exact_markdown_seed"
    );
    assert_eq!(int_value(&batch, "node_count", 0), 5);
    assert_eq!(int_value(&batch, "edge_kind_count", 0), 2);
    assert!((float_value(&batch, "authority_score", 0) - 0.95).abs() < f64::EPSILON);
    assert!(bool_value(&batch, "blocked", 1));
}

#[test]
fn search_strategy_flow_candidate_rows_encode_arrow_ipc_stream() {
    let candidates = fixture_candidates();

    let payload = must(
        search_strategy_flow_candidate_inputs_arrow_ipc(&candidates),
        "candidate rows should encode as Arrow IPC",
    );
    assert!(!payload.is_empty());

    let reader = must(
        StreamReader::try_new(Cursor::new(payload), None),
        "candidate Arrow IPC stream should open",
    );
    let batches = must(
        reader.collect::<Result<Vec<_>, _>>(),
        "candidate Arrow IPC stream should decode",
    );

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].num_rows(), 2);
    assert_eq!(
        string_value(&batches[0], "candidate_id", 1),
        "docs/10_intro.md#document"
    );
}

#[test]
fn search_strategy_flow_candidate_rows_reject_empty_arrow_request() {
    let error = must_err(search_strategy_flow_candidate_inputs_arrow_record_batch(&[]));
    assert!(
        error.contains("must not be empty"),
        "unexpected error: {error}"
    );
}

fn fixture_candidates() -> Vec<SearchStrategyFlowCandidateInput> {
    vec![
        SearchStrategyFlowCandidateInput {
            relative_path: "docs/30_search_strategy/30.01_search_strategy_flow.md".to_owned(),
            heading_anchor: "ownership-boundary".to_owned(),
            title: "Ownership Boundary".to_owned(),
            line_start: 10,
            line_end: 24,
            context_cost: 180,
            evidence_coverage: 0.94,
            graph_score: 0.91,
            authority_score: 0.95,
            structural_score: 0.9,
            uncertainty: 0.08,
            blocked: false,
            edge_kinds: vec![
                "authority".to_owned(),
                "anchor".to_owned(),
                "authority".to_owned(),
            ],
        },
        SearchStrategyFlowCandidateInput {
            relative_path: "docs/10_intro.md".to_owned(),
            heading_anchor: "document".to_owned(),
            title: "Intro".to_owned(),
            line_start: 1,
            line_end: 8,
            context_cost: 64,
            evidence_coverage: 0.98,
            graph_score: 0.96,
            authority_score: 0.95,
            structural_score: 0.94,
            uncertainty: 0.04,
            blocked: true,
            edge_kinds: vec!["intent-exact-markdown-seed".to_owned()],
        },
    ]
}

fn string_value(batch: &arrow::record_batch::RecordBatch, column: &str, row: usize) -> String {
    let array = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("column should exist"))
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap_or_else(|| panic!("column should be string"));
    assert!(!array.is_null(row));
    array.value(row).to_owned()
}

fn int_value(batch: &arrow::record_batch::RecordBatch, column: &str, row: usize) -> i64 {
    let array = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("column should exist"))
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap_or_else(|| panic!("column should be int64"));
    assert!(!array.is_null(row));
    array.value(row)
}

fn float_value(batch: &arrow::record_batch::RecordBatch, column: &str, row: usize) -> f64 {
    let array = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("column should exist"))
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap_or_else(|| panic!("column should be float64"));
    assert!(!array.is_null(row));
    array.value(row)
}

fn bool_value(batch: &arrow::record_batch::RecordBatch, column: &str, row: usize) -> bool {
    let array = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("column should exist"))
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap_or_else(|| panic!("column should be bool"));
    assert!(!array.is_null(row));
    array.value(row)
}

fn must<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_err<T: std::fmt::Debug, E>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("expected error, got {value:?}"),
        Err(error) => error,
    }
}
