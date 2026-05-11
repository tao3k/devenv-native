use crate::studio::arrow_types::LanceArray;

use crate::studio::router::handlers::repo::analysis::flight::{
    build_repo_doc_coverage_flight_batch, build_repo_doc_coverage_flight_metadata,
};
use xiuxian_wendao::analyzers::{DocCoverageResult, DocRecord, DocTargetRecord};

#[test]
fn repo_doc_coverage_flight_batch_preserves_doc_rows() {
    let batch = build_repo_doc_coverage_flight_batch(&[
        DocRecord {
            repo_id: "gateway-sync".to_string().into(),
            doc_id: "repo:gateway-sync:doc:README.md".to_string().into(),
            title: "README".to_string(),
            path: "README.md".to_string().into(),
            format: Some("markdown".to_string()),
            doc_target: Some(DocTargetRecord {
                kind: "module".to_string().into(),
                name: "GatewaySyncPkg".to_string(),
                path: Some("GatewaySyncPkg".to_string()),
                line_start: Some(1),
                line_end: Some(12),
            }),
        },
        DocRecord {
            repo_id: "gateway-sync".to_string().into(),
            doc_id: "repo:gateway-sync:doc:docs/solve.md".to_string().into(),
            title: "solve".to_string(),
            path: "docs/solve.md".to_string().into(),
            format: None,
            doc_target: None,
        },
    ])
    .unwrap_or_else(|error| panic!("repo doc coverage batch should build: {error}"));

    assert_eq!(batch.num_rows(), 2);
    let Some(doc_id_column) = batch.column_by_name("docId") else {
        panic!("docId column");
    };
    let Some(doc_ids) = doc_id_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceStringArray>()
    else {
        panic!("docId should be utf8");
    };
    assert_eq!(doc_ids.value(0), "repo:gateway-sync:doc:README.md");
    assert_eq!(doc_ids.value(1), "repo:gateway-sync:doc:docs/solve.md");

    let Some(format_column) = batch.column_by_name("format") else {
        panic!("format column");
    };
    let Some(formats) = format_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceStringArray>()
    else {
        panic!("format should be utf8");
    };
    assert_eq!(formats.value(0), "markdown");
    assert!(formats.is_null(1));

    let Some(target_name_column) = batch.column_by_name("targetName") else {
        panic!("targetName column");
    };
    let Some(target_names) = target_name_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceStringArray>()
    else {
        panic!("targetName should be utf8");
    };
    assert_eq!(target_names.value(0), "GatewaySyncPkg");
    assert!(target_names.is_null(1));

    let Some(target_line_start_column) = batch.column_by_name("targetLineStart") else {
        panic!("targetLineStart column");
    };
    let Some(target_line_starts) = target_line_start_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceInt32Array>()
    else {
        panic!("targetLineStart should be int32");
    };
    assert_eq!(target_line_starts.value(0), 1);
    assert!(target_line_starts.is_null(1));
}

#[test]
fn repo_doc_coverage_flight_metadata_preserves_summary_fields() {
    let metadata = build_repo_doc_coverage_flight_metadata(&DocCoverageResult {
        repo_id: "gateway-sync".to_string(),
        module_id: Some("GatewaySyncPkg".to_string()),
        docs: Vec::new(),
        covered_symbols: 3,
        uncovered_symbols: 1,
        hierarchical_uri: Some("repo://gateway-sync/docs".to_string()),
        hierarchy: Some(vec!["repo".to_string(), "gateway-sync".to_string()]),
    })
    .unwrap_or_else(|error| panic!("repo doc coverage metadata should encode: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["repoId"], "gateway-sync");
    assert_eq!(payload["moduleId"], "GatewaySyncPkg");
    assert_eq!(payload["coveredSymbols"], 3);
    assert_eq!(payload["uncoveredSymbols"], 1);
    assert_eq!(payload["hierarchicalUri"], "repo://gateway-sync/docs");
    assert_eq!(payload["hierarchy"][0], "repo");
}
