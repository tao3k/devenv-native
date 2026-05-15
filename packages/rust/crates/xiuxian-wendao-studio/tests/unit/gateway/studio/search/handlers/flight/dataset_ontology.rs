use std::fs;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::flight_service_server::FlightService;
use tonic::{Code, Request};

use crate::transport::{ONTOLOGY_DATASET_MATERIALIZE_ROUTE, flight_descriptor_path};

use super::{
    collect_route_batches, first_string, make_gateway_state_with_search_routes,
    populate_dataset_ontology_headers,
};

#[tokio::test]
async fn studio_gateway_flight_rejects_missing_dataset_ontology_payloads() {
    let fixture = make_gateway_state_with_search_routes().await;
    let service = crate::studio::build_studio_flight_service(
        Arc::new(fixture.state.studio.search_plane.clone()),
        fixture.state.clone(),
        "v2",
        3,
    )
    .unwrap_or_else(|error| panic!("build dataset ontology Studio Flight service: {error}"));

    let descriptor = FlightDescriptor::new_path(
        flight_descriptor_path(ONTOLOGY_DATASET_MATERIALIZE_ROUTE)
            .unwrap_or_else(|error| panic!("dataset ontology descriptor path: {error}")),
    );
    let mut request = Request::new(descriptor);
    populate_dataset_ontology_headers(request.metadata_mut());

    let result = service.get_flight_info(request).await;
    let Err(error) = result else {
        panic!("dataset ontology route should reject missing source payloads");
    };

    assert_eq!(error.code(), Code::Internal);
    assert!(
        error
            .message()
            .contains("dataset ontology Arrow IPC payload `patients-arrow`"),
        "unexpected error: {error}"
    );
    assert!(
        !error
            .message()
            .contains("not configured for this transport host"),
        "Studio should attach a dataset ontology provider before payload transport exists"
    );
}

#[tokio::test]
async fn studio_gateway_flight_materializes_dataset_ontology_arrow_payloads() {
    let fixture = make_gateway_state_with_search_routes().await;
    write_healthcare_source_contract(fixture.state.studio.project_root.as_path());
    write_healthcare_arrow_payloads(fixture.state.studio.project_root.as_path());

    let service = crate::studio::build_studio_flight_service(
        Arc::new(fixture.state.studio.search_plane.clone()),
        fixture.state.clone(),
        "v2",
        3,
    )
    .unwrap_or_else(|error| panic!("build dataset ontology Studio Flight service: {error}"));

    let batches = collect_route_batches(
        &service,
        ONTOLOGY_DATASET_MATERIALIZE_ROUTE,
        "dataset ontology materialization",
        populate_dataset_ontology_headers,
    )
    .await;

    assert_eq!(batches.len(), 4);
    let batch = &batches[0];
    assert_eq!(
        first_string(batch, "contractId"),
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(
        first_string(batch, "mappingId"),
        "healthcare.synthetic_care_delivery.v1"
    );
    assert_bool(batch, "passed", true);
    assert_u64(batch, "sourceTableCount", 4);
    assert_u64(batch, "validationFailureCount", 0);
    assert!(
        first_string(batch, "payloadJson").contains("\"semantic_objects\""),
        "report JSON should include semantic read-model counts"
    );
    assert_eq!(first_string(batch, "recordKind"), "materialization_report");
    assert_eq!(first_string(batch, "tableName"), "materialization_report");

    assert_read_model_batch(&batches[1], "semantic_objects", 8);
    assert_read_model_batch(&batches[2], "semantic_relations", 6);
    assert_read_model_batch(&batches[3], "semantic_projection_state", 1);
    assert!(
        first_string(&batches[1], "payloadJson").contains("\"id\""),
        "semantic object payload should include row fields"
    );
}

fn assert_bool(batch: &RecordBatch, column: &str, expected: bool) {
    let actual = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("missing column `{column}`"))
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap_or_else(|| panic!("column `{column}` should be boolean"))
        .value(0);
    assert_eq!(actual, expected);
}

fn assert_u64(batch: &RecordBatch, column: &str, expected: u64) {
    let actual = batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("missing column `{column}`"))
        .as_any()
        .downcast_ref::<UInt64Array>()
        .unwrap_or_else(|| panic!("column `{column}` should be UInt64"))
        .value(0);
    assert_eq!(actual, expected);
}

fn assert_read_model_batch(batch: &RecordBatch, table_name: &str, expected_rows: usize) {
    assert_eq!(batch.num_rows(), expected_rows);
    assert_eq!(first_string(batch, "recordKind"), "semantic_read_model");
    assert_eq!(first_string(batch, "tableName"), table_name);
    assert_bool(batch, "passed", true);
}

fn write_healthcare_source_contract(project_root: &Path) {
    let source_root = real_ontology_root();
    let target_root = project_root.join("wendao-episteme").join("ontology");
    for relative_path in [
        "30_Healthcare/mappings/sql/01_object_observations.sql",
        "30_Healthcare/mappings/sql/02_link_observations.sql",
        "30_Healthcare/mappings/sql/03_evidence.sql",
        "30_Healthcare/mappings/sql/04_semantic_objects.sql",
        "30_Healthcare/mappings/sql/05_semantic_relations.sql",
        "30_Healthcare/mappings/sql/06_semantic_projection_state.sql",
        "30_Healthcare/rules/01_encounter_must_link_patient_provider.sql",
    ] {
        let target = target_root.join(relative_path);
        fs::create_dir_all(
            target
                .parent()
                .unwrap_or_else(|| panic!("target parent for {relative_path}")),
        )
        .unwrap_or_else(|error| panic!("create healthcare SQL fixture dirs: {error}"));
        fs::copy(source_root.join(relative_path), &target)
            .unwrap_or_else(|error| panic!("copy healthcare SQL fixture {relative_path}: {error}"));
    }
}

fn real_ontology_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../wendao-episteme/ontology")
        .canonicalize()
        .unwrap_or_else(|error| panic!("canonicalize source ontology root: {error}"))
}

fn write_healthcare_arrow_payloads(project_root: &Path) {
    let payload_root = project_root
        .join(".cache")
        .join("ontology")
        .join("dataset-payloads")
        .join("healthcare.synthetic_care_delivery.contract.v1")
        .join("healthcare.synthetic_care_delivery.v1");
    fs::create_dir_all(&payload_root)
        .unwrap_or_else(|error| panic!("create healthcare payload root: {error}"));

    write_string_table_ipc(
        payload_root.as_path(),
        "patients-arrow",
        &["patient_id", "patient_name", "birth_year", "source_system"],
        &[
            &["P001", "Ada Lovelace", "1981", "synthetic_ehr"],
            &["P002", "Grace Hopper", "1975", "synthetic_ehr"],
        ],
    );
    write_string_table_ipc(
        payload_root.as_path(),
        "providers-arrow",
        &[
            "provider_id",
            "provider_name",
            "provider_kind",
            "source_system",
        ],
        &[
            &["PR001", "North Clinic", "clinic", "synthetic_ehr"],
            &["PR002", "River Hospital", "hospital", "synthetic_ehr"],
        ],
    );
    write_string_table_ipc(
        payload_root.as_path(),
        "encounters-arrow",
        &[
            "encounter_id",
            "patient_id",
            "provider_id",
            "encounter_label",
            "encounter_date",
            "source_system",
        ],
        &[
            &[
                "E001",
                "P001",
                "PR001",
                "Annual wellness",
                "2026-01-12",
                "synthetic_ehr",
            ],
            &[
                "E002",
                "P002",
                "PR002",
                "Follow-up cardiology",
                "2026-01-13",
                "synthetic_ehr",
            ],
        ],
    );
    write_string_table_ipc(
        payload_root.as_path(),
        "conditions-arrow",
        &[
            "condition_id",
            "patient_id",
            "condition_name",
            "recorded_date",
            "source_system",
        ],
        &[
            &[
                "C001",
                "P001",
                "Hypertension",
                "2026-01-12",
                "synthetic_ehr",
            ],
            &["C002", "P002", "Asthma", "2026-01-13", "synthetic_ehr"],
        ],
    );
}

fn write_string_table_ipc(root: &Path, payload_id: &str, columns: &[&str], rows: &[&[&str]]) {
    let schema = Arc::new(Schema::new(
        columns
            .iter()
            .map(|column| Field::new(*column, DataType::Utf8, false))
            .collect::<Vec<_>>(),
    ));
    let arrays = columns
        .iter()
        .enumerate()
        .map(|(column_index, _)| {
            Arc::new(StringArray::from(
                rows.iter().map(|row| row[column_index]).collect::<Vec<_>>(),
            )) as ArrayRef
        })
        .collect::<Vec<_>>();
    let batch = RecordBatch::try_new(Arc::clone(&schema), arrays)
        .unwrap_or_else(|error| panic!("build healthcare Arrow IPC batch: {error}"));
    let file = File::create(root.join(format!("{payload_id}.arrow")))
        .unwrap_or_else(|error| panic!("create healthcare Arrow IPC file: {error}"));
    let mut writer = StreamWriter::try_new(file, schema.as_ref())
        .unwrap_or_else(|error| panic!("open healthcare Arrow IPC writer: {error}"));
    writer
        .write(&batch)
        .unwrap_or_else(|error| panic!("write healthcare Arrow IPC batch: {error}"));
    writer
        .finish()
        .unwrap_or_else(|error| panic!("finish healthcare Arrow IPC stream: {error}"));
}
