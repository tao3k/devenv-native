use std::sync::Arc;

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::rows::BoundedWorkMarkdownRow;

pub(crate) fn bounded_work_markdown_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("path", DataType::Utf8, false),
        Field::new("surface", DataType::Utf8, false),
        Field::new("heading_path", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("level", DataType::Int64, false),
        Field::new("skeleton", DataType::Utf8, false),
        Field::new("body", DataType::Utf8, false),
    ]))
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
