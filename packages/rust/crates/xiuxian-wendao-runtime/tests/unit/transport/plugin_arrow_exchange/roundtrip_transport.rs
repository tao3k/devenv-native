use super::support::{err_or_panic, sample_binding};
use crate::transport::plugin_arrow_exchange::{
    PluginArrowRequestRow, build_plugin_arrow_request_batch,
    roundtrip_plugin_arrow_score_rows_with_binding,
};

#[tokio::test]
async fn roundtrip_plugin_arrow_score_rows_with_binding_reports_negotiation_errors() {
    let request_batch = build_plugin_arrow_request_batch(
        &[PluginArrowRequestRow {
            doc_id: "doc-a".to_string(),
            vector_score: 0.2,
            embedding: vec![1.0, 2.0, 3.0],
        }],
        &[9.0, 8.0, 7.0],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    let error = err_or_panic(
        roundtrip_plugin_arrow_score_rows_with_binding(
            &sample_binding(Some("not a url")),
            &request_batch,
        )
        .await,
        "invalid base_url should fail",
    );

    assert_eq!(error.selection, None);
    assert!(
        error.error.contains("invalid") || error.error.contains("URL"),
        "unexpected error: {}",
        error.error
    );
}
