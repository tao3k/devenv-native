use std::path::Path;

use crate::local_relation::{DataFusionLocalRelationEngine, LocalRelationEngine};

use super::discovery::discover_bounded_work_markdown_files;
use super::rows::{BoundedWorkMarkdownRow, build_rows_for_file};
use super::schema::{bounded_work_markdown_schema, build_markdown_record_batch};

/// The default `DataFusion` table name for bounded-work markdown retrieval.
pub const BOUNDED_WORK_MARKDOWN_TABLE_NAME: &str = "markdown";

pub(super) struct BoundedWorkMarkdownRegistration {
    pub(super) rows: Vec<BoundedWorkMarkdownRow>,
    pub(super) input_batch_count: usize,
    pub(super) input_row_count: usize,
    pub(super) input_bytes: u64,
}

/// Build bounded-work markdown rows from the `blueprint/` and `plan/` surfaces.
///
/// # Errors
///
/// Returns an error when a required markdown file cannot be discovered, read,
/// parsed, or normalized into the bounded-work row surface.
pub fn build_bounded_work_markdown_rows(
    root: &Path,
) -> Result<Vec<BoundedWorkMarkdownRow>, String> {
    let files = discover_bounded_work_markdown_files(root)?;
    let mut rows = Vec::new();
    for file in files {
        rows.extend(build_rows_for_file(root, &file)?);
    }
    Ok(rows)
}

/// Register the bounded-work markdown rows into a bounded local relation engine.
///
/// # Errors
///
/// Returns an error when bounded-work markdown rows cannot be built or when
/// the in-memory SQL table cannot be registered into the provided engine.
pub fn register_bounded_work_markdown_table(
    query_engine: &impl LocalRelationEngine,
    root: &Path,
) -> Result<Vec<BoundedWorkMarkdownRow>, String> {
    Ok(register_bounded_work_markdown_table_with_stats(query_engine, root)?.rows)
}

pub(super) fn register_bounded_work_markdown_table_with_stats(
    query_engine: &impl LocalRelationEngine,
    root: &Path,
) -> Result<BoundedWorkMarkdownRegistration, String> {
    let rows = build_bounded_work_markdown_rows(root)?;
    let schema = bounded_work_markdown_schema();
    let batch = build_markdown_record_batch(&rows)?;
    let input_row_count = rows.len();
    let input_bytes = u64::try_from(batch.get_array_memory_size()).unwrap_or(u64::MAX);
    query_engine.register_record_batches(BOUNDED_WORK_MARKDOWN_TABLE_NAME, schema, vec![batch])?;
    Ok(BoundedWorkMarkdownRegistration {
        rows,
        input_batch_count: 1,
        input_row_count,
        input_bytes,
    })
}

/// Bootstrap a fresh bounded local relation engine with the bounded-work `markdown` table.
///
/// # Errors
///
/// Returns an error when the bounded-work markdown rows cannot be built or
/// when the in-memory `markdown` table cannot be registered into the fresh
/// query engine.
pub fn bootstrap_bounded_work_markdown_query_engine(
    root: &Path,
) -> Result<(DataFusionLocalRelationEngine, Vec<BoundedWorkMarkdownRow>), String> {
    let query_engine = DataFusionLocalRelationEngine::new_with_information_schema();
    let rows = register_bounded_work_markdown_table(&query_engine, root)?;
    Ok((query_engine, rows))
}
