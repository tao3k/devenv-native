//! Utilities for working with Arrow record batches.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

mod batch;

pub use batch::{
    CATEGORY_COLUMN, CONTENT_COLUMN, DEFAULT_DIMENSION, FILE_PATH_COLUMN, ID_COLUMN,
    INTENTS_COLUMN, METADATA_COLUMN, ROUTING_KEYWORDS_COLUMN, SKILL_NAME_COLUMN, THREAD_ID_COLUMN,
    TOOL_NAME_COLUMN, VECTOR_COLUMN, VectorRecordBatchReader, extract_optional_string,
    extract_string,
};
