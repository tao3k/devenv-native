//! Arrow request rows for the `WendaoGraph` `SearchStrategyFlow` service.

use std::collections::{BTreeSet, HashMap};
use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaContractError, ArrowSchemaDataType,
    build_arrow_schema, validate_record_batch_schema,
};

use super::SearchStrategyFlowCandidateInput;

const SEARCH_STRATEGY_FLOW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.graph.search_strategy_flow.service.v1";
const SEARCH_STRATEGY_FLOW_CANDIDATE_KIND: &str = "markdown_heading_section";
const SEARCH_STRATEGY_FLOW_EXACT_MARKDOWN_SEED_KIND: &str = "intent_exact_markdown_seed";
const EXACT_MARKDOWN_SEED_EDGE_KIND: &str = "intent-exact-markdown-seed";
const DOC_CANDIDATE_NODE_COUNT: i64 = 5;
const SEARCH_STRATEGY_FLOW_CANDIDATE_INPUT_TABLE: &str = "strategy_flow_candidate_inputs";
const SEARCH_STRATEGY_FLOW_CANDIDATE_INPUT_COLUMNS: [ArrowSchemaColumn; 18] = [
    ArrowSchemaColumn::new("candidate_id", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("relative_path", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("heading_anchor", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("title", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("candidate_kind", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("node_count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("edge_kind_count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("edge_kinds", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("line_start", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("line_end", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("evidence_coverage", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("graph_score", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("authority_score", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("semantic_score", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("structural_score", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("context_cost", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("uncertainty", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("blocked", ArrowSchemaDataType::Boolean),
];

pub(crate) fn search_strategy_flow_candidate_inputs_arrow_record_batch(
    candidates: &[SearchStrategyFlowCandidateInput],
) -> Result<RecordBatch, String> {
    if candidates.is_empty() {
        return Err("SearchStrategyFlow Arrow candidate request must not be empty".to_owned());
    }

    let schema = Arc::new(build_arrow_schema(
        &search_strategy_flow_candidate_inputs_contract(),
        search_strategy_flow_candidate_inputs_metadata(),
    ));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            strings(candidates.iter().map(candidate_id)),
            strings(
                candidates
                    .iter()
                    .map(|candidate| candidate.relative_path.as_str()),
            ),
            strings(
                candidates
                    .iter()
                    .map(|candidate| candidate.heading_anchor.as_str()),
            ),
            strings(candidates.iter().map(|candidate| candidate.title.as_str())),
            strings(candidates.iter().map(candidate_kind)),
            ints(candidates.iter().map(|_| Ok(DOC_CANDIDATE_NODE_COUNT)))?,
            ints(candidates.iter().map(edge_kind_count))?,
            strings(candidates.iter().map(edge_kinds)),
            ints(
                candidates
                    .iter()
                    .map(|candidate| usize_to_i64(candidate.line_start)),
            )?,
            ints(
                candidates
                    .iter()
                    .map(|candidate| usize_to_i64(candidate.line_end)),
            )?,
            floats(
                candidates
                    .iter()
                    .map(|candidate| candidate.evidence_coverage),
            ),
            floats(candidates.iter().map(|candidate| candidate.graph_score)),
            floats(candidates.iter().map(|candidate| candidate.authority_score)),
            floats(candidates.iter().map(|_| 0.0)),
            floats(
                candidates
                    .iter()
                    .map(|candidate| candidate.structural_score),
            ),
            ints(
                candidates
                    .iter()
                    .map(|candidate| usize_to_i64(candidate.context_cost)),
            )?,
            floats(candidates.iter().map(|candidate| candidate.uncertainty)),
            booleans(candidates.iter().map(|candidate| candidate.blocked)),
        ],
    )
    .map_err(|error| format!("build SearchStrategyFlow Arrow candidate batch: {error}"))?;
    validate_record_batch_schema(&batch, &search_strategy_flow_candidate_inputs_contract())
        .map_err(|error| search_strategy_flow_candidate_schema_error(&error))?;
    Ok(batch)
}

pub(crate) fn search_strategy_flow_candidate_inputs_arrow_ipc(
    candidates: &[SearchStrategyFlowCandidateInput],
) -> Result<Vec<u8>, String> {
    let batch = search_strategy_flow_candidate_inputs_arrow_record_batch(candidates)?;
    let mut writer = StreamWriter::try_new(Cursor::new(Vec::new()), batch.schema().as_ref())
        .map_err(|error| format!("create SearchStrategyFlow Arrow candidate stream: {error}"))?;
    writer
        .write(&batch)
        .map_err(|error| format!("write SearchStrategyFlow Arrow candidate stream: {error}"))?;
    writer
        .finish()
        .map_err(|error| format!("finish SearchStrategyFlow Arrow candidate stream: {error}"))?;
    writer
        .into_inner()
        .map(Cursor::into_inner)
        .map_err(|error| format!("finalize SearchStrategyFlow Arrow candidate stream: {error}"))
}

fn candidate_id(candidate: &SearchStrategyFlowCandidateInput) -> String {
    format!("{}#{}", candidate.relative_path, candidate.heading_anchor)
}

fn search_strategy_flow_candidate_inputs_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        SEARCH_STRATEGY_FLOW_CANDIDATE_INPUT_TABLE,
        true,
        SEARCH_STRATEGY_FLOW_CANDIDATE_INPUT_COLUMNS.to_vec(),
    )
}

fn search_strategy_flow_candidate_inputs_metadata() -> HashMap<String, String> {
    HashMap::from([(
        "wendao.schema_version".to_owned(),
        SEARCH_STRATEGY_FLOW_SCHEMA_VERSION.to_owned(),
    )])
}

fn search_strategy_flow_candidate_schema_error(error: &ArrowSchemaContractError) -> String {
    format!("SearchStrategyFlow Arrow candidate schema mismatch: {error}")
}

fn candidate_kind(candidate: &SearchStrategyFlowCandidateInput) -> &'static str {
    if candidate
        .edge_kinds
        .iter()
        .any(|kind| kind == EXACT_MARKDOWN_SEED_EDGE_KIND)
    {
        SEARCH_STRATEGY_FLOW_EXACT_MARKDOWN_SEED_KIND
    } else {
        SEARCH_STRATEGY_FLOW_CANDIDATE_KIND
    }
}

fn edge_kind_count(candidate: &SearchStrategyFlowCandidateInput) -> Result<i64, String> {
    usize_to_i64(
        candidate
            .edge_kinds
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            .len(),
    )
}

fn edge_kinds(candidate: &SearchStrategyFlowCandidateInput) -> String {
    candidate.edge_kinds.join(",")
}

fn usize_to_i64(value: usize) -> Result<i64, String> {
    i64::try_from(value).map_err(|error| {
        format!("SearchStrategyFlow Arrow candidate numeric field exceeds i64: {error}")
    })
}

fn strings(values: impl Iterator<Item = impl Into<String>>) -> ArrayRef {
    Arc::new(StringArray::from(
        values.map(Into::into).collect::<Vec<_>>(),
    ))
}

fn ints(values: impl Iterator<Item = Result<i64, String>>) -> Result<ArrayRef, String> {
    Ok(Arc::new(Int64Array::from(
        values.collect::<Result<Vec<_>, _>>()?,
    )))
}

fn floats(values: impl Iterator<Item = f64>) -> ArrayRef {
    Arc::new(Float64Array::from(values.collect::<Vec<_>>()))
}

fn booleans(values: impl Iterator<Item = bool>) -> ArrayRef {
    Arc::new(BooleanArray::from(values.collect::<Vec<_>>()))
}
