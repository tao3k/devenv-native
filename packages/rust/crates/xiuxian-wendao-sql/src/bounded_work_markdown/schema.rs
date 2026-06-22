use std::sync::Arc;

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;

use crate::arrow_contract::{ArrowFieldContract, ArrowFieldType, ArrowTableContract};

use super::rows::BoundedWorkMarkdownRow;

const BOUNDED_WORK_MARKDOWN_SCHEMA_VERSION: &str = "xiuxian_wendao.bounded_work_markdown.v1";

const BOUNDED_WORK_MARKDOWN_FIELDS: [ArrowFieldContract; 8] = [
    ArrowFieldContract::new("path", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("surface", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("surface_kind", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("heading_path", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("title", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("level", ArrowFieldType::Int64, false),
    ArrowFieldContract::new("skeleton", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("body", ArrowFieldType::Utf8, false),
];

pub(crate) const fn bounded_work_markdown_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.bounded_work_markdown.markdown",
        BOUNDED_WORK_MARKDOWN_SCHEMA_VERSION,
        "markdown",
        &BOUNDED_WORK_MARKDOWN_FIELDS,
    )
}

pub(crate) fn bounded_work_markdown_schema() -> SchemaRef {
    bounded_work_markdown_contract().schema()
}

pub(crate) fn build_markdown_record_batch(
    rows: &[BoundedWorkMarkdownRow],
) -> Result<RecordBatch, String> {
    let schema = bounded_work_markdown_schema();
    RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.path.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.surface.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.surface_kind.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.heading_path.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.title.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int64Array::from(
                rows.iter().map(|row| Some(row.level)).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.skeleton.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.body.as_str()))
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| format!("failed to build bounded work markdown batch: {error}"))
}
