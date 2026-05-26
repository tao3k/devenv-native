use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInput, SearchStrategyFlowCandidateInputBatch,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};
use crate::integration_support::search_strategy_flow_frontier_response_schema;

pub(super) fn fixture_candidate_batch() -> SearchStrategyFlowCandidateInputBatch {
    let candidates = vec![SearchStrategyFlowCandidateInput {
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
        edge_kinds: vec!["authority".to_owned(), "anchor".to_owned()],
    }];
    must(
        search_strategy_flow_candidate_input_batch_with_discovery_receipt(
            "rust-code-intelligence-inventory",
            &candidates,
            &serde_json::json!({
                "receiptSource": "rust-code-intelligence-inventory",
                "candidateInputSource": "rust-code-intelligence-inventory",
                "candidateInputCount": candidates.len(),
                "transport": "unit-arrow-service",
                "route": "unit-arrow-service",
                "attemptCount": 1,
                "mergedCandidateCount": candidates.len()
            }),
        ),
        "candidate batch should build",
    )
}

pub(super) fn frontier_response_batch() -> RecordBatch {
    let schema = Arc::new(search_strategy_flow_frontier_response_schema(HashMap::new()));
    must(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec![
                    "flight-service-flow",
                    "flight-service-flow",
                ])),
                Arc::new(StringArray::from(vec![
                    "flight-service-flow-frontier-1",
                    "flight-service-flow-frontier-2",
                ])),
                Arc::new(StringArray::from(vec![
                    "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
                    "docs/90_validation/90.01_validation.md#promotion-boundary",
                ])),
                Arc::new(StringArray::from(vec!["revision-1", "revision-2"])),
                Arc::new(Int64Array::from(vec![1, 2])),
                Arc::new(BooleanArray::from(vec![true, false])),
                Arc::new(Float64Array::from(vec![0.95, 0.12])),
                Arc::new(StringArray::from(vec!["keep", "prune"])),
                Arc::new(Int64Array::from(vec![180, 0])),
                Arc::new(StringArray::from(vec!["authority", "not_selected"])),
            ],
        ),
        "frontier batch should build",
    )
}

pub(super) fn frontier_response_batch_with_rank_float() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flow_id", DataType::Utf8, false),
        Field::new("frontier_id", DataType::Utf8, false),
        Field::new("candidate_id", DataType::Utf8, false),
        Field::new("revision_id", DataType::Utf8, false),
        Field::new("rank", DataType::Float64, false),
        Field::new("selected", DataType::Boolean, false),
        Field::new("final_score", DataType::Float64, false),
        Field::new("action", DataType::Utf8, false),
        Field::new("context_budget", DataType::Int64, false),
        Field::new("judgement_kind", DataType::Utf8, false),
    ]));
    must(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["flight-service-flow"])),
                Arc::new(StringArray::from(vec!["flight-service-flow-frontier-1"])),
                Arc::new(StringArray::from(vec![
                    "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
                ])),
                Arc::new(StringArray::from(vec!["revision-1"])),
                Arc::new(Float64Array::from(vec![1.0])),
                Arc::new(BooleanArray::from(vec![true])),
                Arc::new(Float64Array::from(vec![0.95])),
                Arc::new(StringArray::from(vec!["keep"])),
                Arc::new(Int64Array::from(vec![180])),
                Arc::new(StringArray::from(vec!["authority"])),
            ],
        ),
        "frontier schema drift batch should build",
    )
}

pub(super) fn branch_judgement_arrow_ipc(flow_id: &str, candidate_id: &str) -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flow_id", DataType::Utf8, false),
        Field::new("candidate_id", DataType::Utf8, false),
        Field::new("branch_role", DataType::Utf8, false),
        Field::new("judgement_score", DataType::Float64, false),
        Field::new("confidence", DataType::Float64, false),
        Field::new("decision", DataType::Utf8, false),
        Field::new("blocked", DataType::Boolean, false),
        Field::new("reason", DataType::Utf8, false),
    ]));
    let batch = must(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec![flow_id])),
                Arc::new(StringArray::from(vec![candidate_id])),
                Arc::new(StringArray::from(vec!["authority"])),
                Arc::new(Float64Array::from(vec![0.1])),
                Arc::new(Float64Array::from(vec![1.0])),
                Arc::new(StringArray::from(vec!["reject"])),
                Arc::new(BooleanArray::from(vec![true])),
                Arc::new(StringArray::from(vec!["negative guard"])),
            ],
        ),
        "branch judgement batch should build",
    );
    arrow_ipc_stream(&batch)
}

pub(super) fn branch_judgement_arrow_ipc_without_reason() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flow_id", DataType::Utf8, false),
        Field::new("candidate_id", DataType::Utf8, false),
        Field::new("branch_role", DataType::Utf8, false),
        Field::new("judgement_score", DataType::Float64, false),
        Field::new("confidence", DataType::Float64, false),
        Field::new("decision", DataType::Utf8, false),
        Field::new("blocked", DataType::Boolean, false),
    ]));
    let batch = must(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["flight-service-flow"])),
                Arc::new(StringArray::from(vec![
                    "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
                ])),
                Arc::new(StringArray::from(vec!["authority"])),
                Arc::new(Float64Array::from(vec![0.1])),
                Arc::new(Float64Array::from(vec![1.0])),
                Arc::new(StringArray::from(vec!["reject"])),
                Arc::new(BooleanArray::from(vec![true])),
            ],
        ),
        "invalid branch judgement batch should build",
    );
    arrow_ipc_stream(&batch)
}

pub(super) fn arrow_ipc_stream(batch: &RecordBatch) -> Vec<u8> {
    let mut writer = must(
        StreamWriter::try_new(Cursor::new(Vec::new()), batch.schema().as_ref()),
        "Arrow IPC stream writer should build",
    );
    must(writer.write(batch), "Arrow IPC batch should write");
    must(writer.finish(), "Arrow IPC stream should finish");
    must(writer.into_inner(), "Arrow IPC stream should finalize").into_inner()
}

fn must<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}
