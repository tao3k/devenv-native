use std::io;

use tempfile::tempdir;
use xiuxian_wendao_parsers::semantic_ssot::load_semantic_repository;
use xiuxian_wendao_sql::semantic_read_model::build_semantic_read_model_record_batches;

use super::support::{
    assert_binary_column_matches, dataset_ontology_envelope_batch, decode_single_batch,
    sample_request_batches, string_column_value, write_semantic_read_model_fixture,
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
    build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope,
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

#[test]
fn ontology_read_model_quality_accepts_gateway_dataset_ontology_envelope() -> io::Result<()> {
    let request_batches =
        build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope(
            &valid_gateway_dataset_ontology_envelope_batches(),
        )
        .map_err(io::Error::other)?;

    assert_eq!(request_batches.row_counts(), [1, 1, 1]);
    assert_eq!(
        string_column_value(&request_batches.objects, "id", 0),
        "healthcare.patient.patient-001"
    );
    assert_eq!(
        string_column_value(&request_batches.relations, "target", 0),
        "healthcare.patient.patient-001"
    );
    assert_eq!(
        string_column_value(&request_batches.projection_state, "projection", 0),
        "healthcare_synthetic_care_delivery"
    );

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&request_batches)
        .map_err(io::Error::other)?;
    let bundle = build_wendaograph_ontology_read_model_quality_flight_request_batch(&request)
        .map_err(io::Error::other)?;
    let objects = decode_single_batch(
        request.semantic_objects_payload.as_slice(),
        "semantic_objects",
    );

    assert_eq!(bundle.num_rows(), 1);
    assert_eq!(
        string_column_value(&objects, "source_path", 0),
        "ontology/30_Healthcare/fixtures/patients.csv"
    );
    assert_binary_column_matches(
        &bundle,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
        request.semantic_objects_payload.as_slice(),
    );

    Ok(())
}

#[test]
fn ontology_read_model_quality_rejects_incomplete_gateway_dataset_ontology_envelope() {
    let objects_batch = dataset_ontology_envelope_batch(
        "semantic_read_model",
        "semantic_objects",
        &[serde_json::json!({
            "id": "healthcare.patient.patient-001",
            "kind": "healthcare.patient",
            "title": "Synthetic Patient 001",
            "status": "active",
            "confidence_score": 1.0,
            "confidence_source": "duckdb_mapping",
            "owner_count": 1,
            "owners_json": "[]",
            "provenance_source": "ontology/30_Healthcare/mappings/org/synthetic_care_delivery.org",
            "provenance_recorded_by": "dataset-to-ontology-contract",
            "provenance_recorded_at": "2026-05-05",
            "verification_required_json": "[]",
            "verification_evidence_json": "[]",
            "relation_count": 0,
            "source_path": "ontology/30_Healthcare/fixtures/patients.csv",
            "read_model_source_revision": "fixture-revision",
            "read_model_projection_revision": "dataset-ontology-v1",
            "read_model_projection_staleness": "fresh"
        })
        .to_string()],
    );

    let Err(error) =
        build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope(
            &[objects_batch],
        )
    else {
        panic!("incomplete Gateway envelope should be rejected");
    };

    assert!(
        error.contains("semantic_relations"),
        "unexpected error: {error}"
    );
}

fn valid_gateway_dataset_ontology_envelope_batches() -> Vec<arrow::record_batch::RecordBatch> {
    vec![
        dataset_ontology_envelope_batch(
            "materialization_report",
            "materialization_report",
            &[r#"{"passed":true}"#.to_string()],
        ),
        dataset_ontology_envelope_batch(
            "semantic_read_model",
            "semantic_objects",
            &[valid_gateway_dataset_ontology_object_row()],
        ),
        dataset_ontology_envelope_batch(
            "semantic_read_model",
            "semantic_relations",
            &[valid_gateway_dataset_ontology_relation_row()],
        ),
        dataset_ontology_envelope_batch(
            "semantic_read_model",
            "semantic_projection_state",
            &[valid_gateway_dataset_ontology_projection_row()],
        ),
    ]
}

fn valid_gateway_dataset_ontology_object_row() -> String {
    serde_json::json!({
        "id": "healthcare.patient.patient-001",
        "kind": "healthcare.patient",
        "title": "Synthetic Patient 001",
        "status": "active",
        "confidence_score": 1.0,
        "confidence_source": "duckdb_mapping",
        "owner_count": 1,
        "owners_json": "[{\"scope\":\"wendao-episteme\",\"role\":\"dataset_mapping\"}]",
        "provenance_source": "ontology/30_Healthcare/mappings/org/synthetic_care_delivery.org",
        "provenance_recorded_by": "dataset-to-ontology-contract",
        "provenance_recorded_at": "2026-05-05",
        "verification_required_json": "[\"dataset_mapping_manifest\"]",
        "verification_evidence_json": "[\"patients.csv#patient-001\"]",
        "relation_count": 1,
        "source_path": "ontology/30_Healthcare/fixtures/patients.csv",
        "read_model_source_revision": "fixture-revision",
        "read_model_projection_revision": "dataset-ontology-v1",
        "read_model_projection_staleness": "fresh"
    })
    .to_string()
}

fn valid_gateway_dataset_ontology_relation_row() -> String {
    serde_json::json!({
        "source": "healthcare.encounter.enc-001",
        "kind": "healthcare.encounter.has_patient",
        "target": "healthcare.patient.patient-001",
        "source_path": "ontology/30_Healthcare/fixtures/encounters.csv",
        "read_model_source_revision": "fixture-revision",
        "read_model_projection_revision": "dataset-ontology-v1",
        "read_model_projection_staleness": "fresh"
    })
    .to_string()
}

fn valid_gateway_dataset_ontology_projection_row() -> String {
    serde_json::json!({
        "projection": "healthcare_synthetic_care_delivery",
        "status": "active",
        "source_revision": "fixture-revision",
        "current_source_revision": "fixture-revision",
        "projection_revision": "dataset-ontology-v1",
        "staleness": "fresh",
        "source_object_count": 1,
        "source_objects_json": "[\"healthcare.patient.patient-001\"]",
        "source_path": "ontology/30_Healthcare/mappings/org/synthetic_care_delivery.org"
    })
    .to_string()
}
