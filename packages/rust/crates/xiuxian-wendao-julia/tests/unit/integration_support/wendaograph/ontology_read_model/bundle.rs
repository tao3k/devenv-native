use std::io;

use tempfile::tempdir;
use xiuxian_wendao_parsers::semantic_ssot::load_semantic_repository;
use xiuxian_wendao_sql::semantic_read_model::build_semantic_read_model_record_batches;

use super::support::{
    assert_binary_column_matches, decode_single_batch, sample_request_batches,
    write_semantic_read_model_fixture,
};
use super::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_descriptor,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
};

#[test]
fn ontology_read_model_quality_flight_descriptor_uses_route_path() {
    let descriptor = build_wendaograph_ontology_read_model_quality_flight_descriptor();

    assert_eq!(
        descriptor.path,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
}

#[test]
fn ontology_read_model_quality_flight_request_batch_bundles_three_payloads() {
    let request =
        build_wendaograph_ontology_read_model_quality_arrow_request(&sample_request_batches())
            .unwrap_or_else(|error| panic!("build ontology read-model quality request: {error}"));
    let batch = build_wendaograph_ontology_read_model_quality_flight_request_batch(&request)
        .unwrap_or_else(|error| panic!("build ontology read-model quality Flight batch: {error}"));

    assert_eq!(
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH,
        ["wendao", "graph", "ontology_read_model_quality"]
    );
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        batch
            .schema()
            .metadata()
            .get("wendao.table")
            .map(String::as_str),
        Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE)
    );
    assert_binary_column_matches(
        &batch,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
        request.semantic_objects_payload.as_slice(),
    );
    assert_binary_column_matches(
        &batch,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
        request.semantic_relations_payload.as_slice(),
    );
    assert_binary_column_matches(
        &batch,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
        request.semantic_projection_state_payload.as_slice(),
    );
}

#[test]
fn ontology_read_model_quality_accepts_sql_materializer_record_batches() -> io::Result<()> {
    let temp = tempdir()?;
    write_semantic_read_model_fixture(temp.path())?;
    let repository = load_semantic_repository(temp.path());
    let sql_batches =
        build_semantic_read_model_record_batches(&repository).map_err(io::Error::other)?;
    let request_batches = WendaoGraphOntologyReadModelQualityRequestBatches::new(
        sql_batches.objects,
        sql_batches.relations,
        sql_batches.projection_state,
    );

    assert_eq!(request_batches.row_counts(), [2, 1, 1]);

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&request_batches)
        .map_err(io::Error::other)?;
    let bundle = build_wendaograph_ontology_read_model_quality_flight_request_batch(&request)
        .map_err(io::Error::other)?;

    assert_eq!(bundle.num_rows(), 1);
    let objects = decode_single_batch(
        request.semantic_objects_payload.as_slice(),
        "semantic_objects",
    );
    let relations = decode_single_batch(
        request.semantic_relations_payload.as_slice(),
        "semantic_relations",
    );
    let projection_state = decode_single_batch(
        request.semantic_projection_state_payload.as_slice(),
        "semantic_projection_state",
    );

    assert_eq!(objects.schema().field(0).name(), "id");
    assert_eq!(relations.schema().field(0).name(), "source");
    assert_eq!(projection_state.schema().field(0).name(), "projection");
    assert_binary_column_matches(
        &bundle,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
        request.semantic_objects_payload.as_slice(),
    );

    Ok(())
}
