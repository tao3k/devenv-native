//! Arrow transport schema helpers for Julia repository-intelligence analyzers.

use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema};

/// Canonical `WendaoArrow` request/response `doc_id` column.
pub const JULIA_ARROW_DOC_ID_COLUMN: &str = "doc_id";
/// Canonical `WendaoArrow` request `vector_score` column.
pub const JULIA_ARROW_VECTOR_SCORE_COLUMN: &str = "vector_score";
/// Canonical `WendaoArrow` request `embedding` column.
pub const JULIA_ARROW_EMBEDDING_COLUMN: &str = "embedding";
/// Canonical `WendaoArrow` request `query_embedding` column.
pub const JULIA_ARROW_QUERY_EMBEDDING_COLUMN: &str = "query_embedding";
/// Canonical `WendaoArrow` response `analyzer_score` column.
pub const JULIA_ARROW_ANALYZER_SCORE_COLUMN: &str = "analyzer_score";
/// Canonical `WendaoArrow` response `final_score` column.
pub const JULIA_ARROW_FINAL_SCORE_COLUMN: &str = "final_score";
/// Canonical additive `WendaoArrow` response `trace_id` column.
pub const JULIA_ARROW_TRACE_ID_COLUMN: &str = "trace_id";

fn julia_arrow_vector_item_field() -> Arc<Field> {
    Arc::new(Field::new("item", DataType::Float32, true))
}

/// Build the canonical `WendaoArrow` `v1` request schema for one embedding size.
#[must_use]
pub fn julia_arrow_request_schema(vector_dim: i32) -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(JULIA_ARROW_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(JULIA_ARROW_VECTOR_SCORE_COLUMN, DataType::Float64, false),
        Field::new(
            JULIA_ARROW_EMBEDDING_COLUMN,
            DataType::FixedSizeList(julia_arrow_vector_item_field(), vector_dim),
            false,
        ),
        Field::new(
            JULIA_ARROW_QUERY_EMBEDDING_COLUMN,
            DataType::FixedSizeList(julia_arrow_vector_item_field(), vector_dim),
            false,
        ),
    ]))
}

/// Build the canonical `WendaoArrow` `v1` response schema.
#[must_use]
pub fn julia_arrow_response_schema(include_trace_id: bool) -> Arc<Schema> {
    let mut fields = vec![
        Field::new(JULIA_ARROW_DOC_ID_COLUMN, DataType::Utf8, false),
        Field::new(JULIA_ARROW_ANALYZER_SCORE_COLUMN, DataType::Float64, false),
        Field::new(JULIA_ARROW_FINAL_SCORE_COLUMN, DataType::Float64, false),
    ];
    if include_trace_id {
        fields.push(Field::new(
            JULIA_ARROW_TRACE_ID_COLUMN,
            DataType::Utf8,
            false,
        ));
    }
    Arc::new(Schema::new(fields))
}

#[cfg(test)]
#[path = "../../tests/unit/repo_intelligence/arrow_transport.rs"]
mod tests;
