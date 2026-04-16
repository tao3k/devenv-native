//! Compatibility seam for the canonical bounded-work markdown SQL helper owner.

pub use xiuxian_wendao_sql::bounded_work_markdown::BoundedWorkMarkdownRow;
pub use xiuxian_wendao_sql::bounded_work_markdown::{
    BOUNDED_WORK_MARKDOWN_TABLE_NAME, bootstrap_bounded_work_markdown_query_engine,
    build_bounded_work_markdown_rows, query_bounded_work_markdown_payload,
    query_bounded_work_markdown_payload_with_engine, register_bounded_work_markdown_table,
};

#[cfg(test)]
#[path = "../../../../../tests/unit/search/queries/sql/bounded_work_markdown.rs"]
mod tests;
