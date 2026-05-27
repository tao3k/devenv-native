//! Arrow schema contracts for the link-graph snapshot cache.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

pub(super) fn docs_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "link_graph_snapshot_docs",
        true,
        vec![
            utf8_column("id"),
            utf8_column("stem"),
            utf8_column("path"),
            utf8_column("title"),
            utf8_column("lead"),
            nullable_utf8_column("doc_type"),
            utf8_list_column("tags"),
            uint64_column("word_count"),
            utf8_column("search_text"),
            float64_column("saliency_base"),
            float64_column("decay_rate"),
            nullable_int64_column("created_ts"),
            nullable_int64_column("modified_ts"),
        ],
    )
}

pub(super) fn sections_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "link_graph_snapshot_sections",
        true,
        vec![
            utf8_column("doc_id"),
            utf8_column("heading_title"),
            utf8_column("heading_path"),
            utf8_column("heading_path_lower"),
            uint64_column("heading_level"),
            uint64_column("line_start"),
            uint64_column("line_end"),
            uint64_column("byte_start"),
            uint64_column("byte_end"),
            utf8_column("section_text"),
            utf8_column("section_text_lower"),
            utf8_list_column("entities"),
            utf8_column("attributes_json"),
            utf8_column("logbook_json"),
            utf8_column("observations_json"),
        ],
    )
}

pub(super) fn edges_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "link_graph_snapshot_edges",
        true,
        vec![utf8_column("source_id"), utf8_column("target_id")],
    )
}

pub(super) fn aliases_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "link_graph_snapshot_aliases",
        true,
        vec![utf8_column("alias"), utf8_column("doc_id")],
    )
}

pub(super) fn snapshot_schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    let mut metadata = HashMap::new();
    metadata.insert(
        WENDAO_TABLE_METADATA_KEY.to_string(),
        contract.table_name().to_string(),
    );
    Arc::new(build_arrow_schema(contract, metadata))
}

pub(super) fn validate_snapshot_batch(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    context: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("{context}: {error}"))
}

fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

fn uint64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::UInt64)
}

fn nullable_int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int64)
}

fn float64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Float64)
}

fn utf8_list_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8List)
}
