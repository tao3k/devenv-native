use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema,
    LanceStringArray as StringArray,
};

use crate::transport::{RerankScoreWeights, WendaoFlightService};

use super::assertions::{must_err, must_ok};
use super::providers::RecordingRepoSearchProvider;

#[test]
fn wendao_flight_service_rejects_blank_schema_version() {
    let query_response_batch = must_ok(
        LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("doc_id", LanceDataType::Utf8, false),
                LanceField::new("path", LanceDataType::Utf8, false),
                LanceField::new("title", LanceDataType::Utf8, false),
                LanceField::new("score", LanceDataType::Float64, false),
                LanceField::new("language", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec!["doc-1"])),
                Arc::new(StringArray::from(vec!["src/lib.rs"])),
                Arc::new(StringArray::from(vec!["Repo Search Result"])),
                Arc::new(LanceFloat64Array::from(vec![0.91_f64])),
                Arc::new(StringArray::from(vec!["rust"])),
            ],
        ),
        "query response batch should build",
    );

    let error = must_err(
        WendaoFlightService::new("   ", query_response_batch, 3),
        "blank schema-version service construction should fail",
    );
    assert_eq!(
        error,
        "wendao flight service schema version must not be blank"
    );
}

#[test]
fn wendao_flight_service_accepts_pluggable_repo_search_provider() {
    let service = must_ok(
        WendaoFlightService::new_with_provider(
            "v2",
            Arc::new(RecordingRepoSearchProvider),
            3,
            RerankScoreWeights::default(),
        ),
        "service should build from a pluggable repo-search provider",
    );

    assert_eq!(service.expected_schema_version, "v2");
}
