use arrow::array::{Array, BooleanArray, StringArray, UInt64Array};
use xiuxian_wendao_sql::candidate_read_model::{
    CandidateReadModelDuckDbInspectionReport, CandidateReadModelKindCount,
};

use crate::studio::search::handlers::flight::candidate_inspection_report_batch;

#[test]
fn ontology_candidate_inspection_report_batch_contains_summary_columns() {
    let report = CandidateReadModelDuckDbInspectionReport {
        schema_version: "xiuxian_wendao.sql.candidate_read_model_duckdb_inspection.v1",
        execution_engine: "duckdb",
        registration_strategy: "duckdb_read_parquet_view",
        candidate_object_count: 4,
        candidate_relation_count: 3,
        candidate_evidence_count: 2,
        object_kind_counts: vec![CandidateReadModelKindCount {
            kind: "source_file".to_string(),
            row_count: 4,
        }],
        relation_kind_counts: Vec::new(),
        evidence_kind_counts: Vec::new(),
        review_status_violation_count: 0,
        promotion_status_violation_count: 0,
        ontology_truth_violation_count: 0,
        raw_to_rdf_promotion_violation_count: 0,
        missing_relation_endpoint_count: 0,
        missing_relation_endpoints: Vec::new(),
        inspection_passed: true,
    };

    let batch = candidate_inspection_report_batch(&report)
        .unwrap_or_else(|error| panic!("candidate inspection batch: {error}"));

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        string_value(&batch, "execution_engine"),
        "duckdb",
        "execution engine should stay in the Arrow row"
    );
    assert_eq!(
        u64_value(&batch, "candidate_object_count"),
        4,
        "object count should be projected into the Arrow row"
    );
    assert!(
        bool_value(&batch, "inspection_passed"),
        "passed status should be projected into the Arrow row"
    );
}

fn string_value(batch: &arrow::record_batch::RecordBatch, column: &str) -> String {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .map(|array| array.value(0).to_string())
        .unwrap_or_else(|| panic!("missing string column `{column}`"))
}

fn u64_value(batch: &arrow::record_batch::RecordBatch, column: &str) -> u64 {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
        .map(|array| array.value(0))
        .unwrap_or_else(|| panic!("missing UInt64 column `{column}`"))
}

fn bool_value(batch: &arrow::record_batch::RecordBatch, column: &str) -> bool {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<BooleanArray>())
        .map(|array| array.value(0))
        .unwrap_or_else(|| panic!("missing Boolean column `{column}`"))
}
