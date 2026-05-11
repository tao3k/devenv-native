//! Utilities for working with Arrow record batches.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);

mod batch;

pub use batch::{
    CATEGORY_COLUMN, CONTENT_COLUMN, DEFAULT_DIMENSION, FILE_PATH_COLUMN, ID_COLUMN,
    INTENTS_COLUMN, METADATA_COLUMN, ROUTING_KEYWORDS_COLUMN, SKILL_NAME_COLUMN, THREAD_ID_COLUMN,
    TOOL_NAME_COLUMN, VECTOR_COLUMN, VectorBatchInput, VectorRecordBatchReader,
    extract_optional_string, extract_string,
};
