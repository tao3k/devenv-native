use super::support::{decode_single_batch, sample_request_batches};
use super::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE,
    WendaoGraphOntologyReadModelQualityArrowRequest,
    build_wendaograph_ontology_read_model_quality_arrow_request,
};

#[test]
fn ontology_read_model_quality_arrow_request_preserves_service_contract() {
    let batches = sample_request_batches();
    let request: WendaoGraphOntologyReadModelQualityArrowRequest =
        build_wendaograph_ontology_read_model_quality_arrow_request(&batches)
            .unwrap_or_else(|error| panic!("build ontology read-model quality request: {error}"));

    assert_eq!(
        request.schema_version,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION
    );
    assert_eq!(
        request.request_mime_type,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME
    );
    assert_eq!(
        request.response_mime_type,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME
    );
    assert_eq!(
        request.request_tables,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES
    );
    assert_eq!(
        request.response_table,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE
    );
    assert!(
        request
            .payload_byte_sizes()
            .into_iter()
            .all(|size| size > 0)
    );
}

#[test]
fn ontology_read_model_quality_arrow_request_encodes_metadata_for_each_table() {
    let request =
        build_wendaograph_ontology_read_model_quality_arrow_request(&sample_request_batches())
            .unwrap_or_else(|error| panic!("build ontology read-model quality request: {error}"));

    for (payload, table_name) in [
        (
            request.semantic_objects_payload.as_slice(),
            "semantic_objects",
        ),
        (
            request.semantic_relations_payload.as_slice(),
            "semantic_relations",
        ),
        (
            request.semantic_projection_state_payload.as_slice(),
            "semantic_projection_state",
        ),
    ] {
        let batch = decode_single_batch(payload, table_name);
        let schema = batch.schema();
        let metadata = schema.metadata();
        assert_eq!(
            metadata.get("wendao.service").map(String::as_str),
            Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE)
        );
        assert_eq!(
            metadata.get("wendao.method").map(String::as_str),
            Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD)
        );
        assert_eq!(
            metadata.get("wendao.schema_version").map(String::as_str),
            Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION)
        );
        assert_eq!(
            metadata.get("wendao.table").map(String::as_str),
            Some(table_name)
        );
        assert_eq!(batch.num_rows(), 1);
    }
}

#[test]
fn ontology_read_model_quality_request_batches_report_row_counts() {
    let batches = sample_request_batches();

    assert_eq!(batches.row_counts(), [1, 1, 1]);
}
