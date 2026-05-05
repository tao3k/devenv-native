use std::collections::BTreeMap;

use crate::transport::plugin_arrow_exchange::{
    PluginArrowRequestBatchBuildError, PluginArrowRequestRow, PluginArrowScoredCandidate,
    build_plugin_arrow_request_batch, build_plugin_arrow_request_batch_from_embeddings,
    build_plugin_arrow_request_batch_from_embeddings_with_metadata,
    project_plugin_arrow_scored_candidates,
};

#[test]
fn build_plugin_arrow_request_batch_uses_contract_columns() {
    let batch = build_plugin_arrow_request_batch(
        &[
            PluginArrowRequestRow {
                doc_id: "doc-1".to_string(),
                vector_score: 0.3,
                embedding: vec![1.0, 2.0, 3.0],
            },
            PluginArrowRequestRow {
                doc_id: "doc-2".to_string(),
                vector_score: 0.4,
                embedding: vec![4.0, 5.0, 6.0],
            },
        ],
        &[9.0, 8.0, 7.0],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().field(0).name(), "doc_id");
    assert_eq!(batch.schema().field(1).name(), "vector_score");
    assert_eq!(batch.schema().field(2).name(), "embedding");
    assert_eq!(batch.schema().field(3).name(), "query_embedding");
}

#[test]
fn plugin_arrow_request_schema_uses_contract_columns() {
    let schema = xiuxian_wendao_core::repo_intelligence::julia_arrow_request_schema(3);

    assert_eq!(schema.field(0).name(), "doc_id");
    assert_eq!(schema.field(1).name(), "vector_score");
    assert_eq!(schema.field(2).name(), "embedding");
    assert_eq!(schema.field(3).name(), "query_embedding");
}

#[test]
fn plugin_arrow_response_schema_optionally_includes_trace_id() {
    let base = xiuxian_wendao_core::repo_intelligence::julia_arrow_response_schema(false);
    let traced = xiuxian_wendao_core::repo_intelligence::julia_arrow_response_schema(true);

    assert_eq!(base.fields().len(), 3);
    assert_eq!(traced.fields().len(), 4);
    assert_eq!(traced.field(3).name(), "trace_id");
}

#[test]
fn build_plugin_arrow_request_batch_rejects_dimension_mismatch() {
    let error = super::support::err_or_panic(
        build_plugin_arrow_request_batch(
            &[PluginArrowRequestRow {
                doc_id: "doc-1".to_string(),
                vector_score: 0.3,
                embedding: vec![1.0, 2.0],
            }],
            &[9.0, 8.0, 7.0],
        ),
        "dimension mismatch should fail",
    );

    assert!(
        error.to_string().contains("embedding dimension mismatch"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_plugin_arrow_request_batch_from_embeddings_uses_candidate_ids_as_doc_ids() {
    let embeddings = BTreeMap::from([
        ("doc-1#alpha".to_string(), vec![1.0, 2.0, 3.0]),
        ("doc-2#beta".to_string(), vec![4.0, 5.0, 6.0]),
    ]);
    let batch = build_plugin_arrow_request_batch_from_embeddings(
        &[
            PluginArrowScoredCandidate {
                doc_id: "doc-1#alpha",
                vector_score: 0.31,
            },
            PluginArrowScoredCandidate {
                doc_id: "doc-2#beta",
                vector_score: 0.42,
            },
        ],
        &embeddings,
        &[9.0, 8.0, 7.0],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    let Some(doc_ids) = batch
        .column_by_name("doc_id")
        .and_then(|column| column.as_any().downcast_ref::<arrow_array::StringArray>())
    else {
        panic!("doc_id column");
    };
    assert_eq!(doc_ids.value(0), "doc-1#alpha");
    assert_eq!(doc_ids.value(1), "doc-2#beta");
}

#[test]
fn build_plugin_arrow_request_batch_from_embeddings_rejects_missing_embeddings() {
    let error = super::support::err_or_panic(
        build_plugin_arrow_request_batch_from_embeddings(
            &[PluginArrowScoredCandidate {
                doc_id: "doc-1#alpha",
                vector_score: 0.31,
            }],
            &BTreeMap::new(),
            &[9.0, 8.0, 7.0],
        ),
        "missing embedding should fail",
    );

    assert!(matches!(
        error,
        PluginArrowRequestBatchBuildError::MissingEmbedding { doc_id }
        if doc_id == "doc-1#alpha"
    ));
}

#[test]
fn project_plugin_arrow_scored_candidates_collects_doc_ids_and_scores() {
    let projection =
        project_plugin_arrow_scored_candidates([("doc-1#alpha", 0.25), ("doc-2#beta", 0.5)]);

    assert_eq!(
        projection.doc_ids,
        vec!["doc-1#alpha".to_string(), "doc-2#beta".to_string()]
    );
    assert_eq!(
        projection.candidates,
        vec![
            PluginArrowScoredCandidate {
                doc_id: "doc-1#alpha",
                vector_score: 0.25,
            },
            PluginArrowScoredCandidate {
                doc_id: "doc-2#beta",
                vector_score: 0.5,
            },
        ]
    );
}

#[test]
fn build_plugin_arrow_request_batch_from_embeddings_with_metadata_sets_trace_id() {
    let batch = build_plugin_arrow_request_batch_from_embeddings_with_metadata(
        &[PluginArrowScoredCandidate {
            doc_id: "doc-1#alpha",
            vector_score: 0.25,
        }],
        &BTreeMap::from([("doc-1#alpha".to_string(), vec![1.0, 2.0, 3.0])]),
        &[9.0, 8.0, 7.0],
        "xiuxian-wendao-julia",
        "alpha signal",
        "v1",
    )
    .unwrap_or_else(|error| panic!("request batch with metadata should build: {error}"));

    assert_eq!(
        batch.schema().metadata().get("trace_id"),
        Some(&"plugin-rerank:xiuxian-wendao-julia:alpha_signal".to_string())
    );
    assert_eq!(
        batch.schema().metadata().get("wendao.schema_version"),
        Some(&"v1".to_string())
    );
}
