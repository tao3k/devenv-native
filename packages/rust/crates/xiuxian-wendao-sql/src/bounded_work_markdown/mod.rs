//! Bounded-work markdown SQL surface for `blueprint/` and `plan/` work surfaces.

mod discovery;
mod query;
mod register;
mod rows;
mod schema;
mod skeleton;

pub use query::{
    query_bounded_work_markdown_payload, query_bounded_work_markdown_payload_with_engine,
};
pub use register::{
    BOUNDED_WORK_MARKDOWN_TABLE_NAME, bootstrap_bounded_work_markdown_query_engine,
    build_bounded_work_markdown_rows, register_bounded_work_markdown_table,
};
pub use rows::BoundedWorkMarkdownRow;

#[cfg(test)]
#[path = "../../tests/unit/bounded_work_markdown.rs"]
mod tests;
