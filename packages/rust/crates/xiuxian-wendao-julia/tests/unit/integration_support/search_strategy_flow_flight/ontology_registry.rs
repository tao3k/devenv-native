use std::{io::Cursor, sync::Arc};

use arrow::array::{Array, BooleanArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;

use super::semantic_scope_batches_to_ontology_registry_arrow_ipc;

#[test]
fn semantic_scope_batches_convert_to_ontology_registry_arrow_ipc() {
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("objectId", DataType::Utf8, true),
            Field::new("kind", DataType::Utf8, true),
            Field::new("title", DataType::Utf8, true),
            Field::new("relationTargetsJson", DataType::Utf8, true),
            Field::new("requiredValidationsJson", DataType::Utf8, true),
        ])),
        vec![
            Arc::new(StringArray::from(vec![Some("PatientRecord")])),
            Arc::new(StringArray::from(vec![Some("object")])),
            Arc::new(StringArray::from(vec![Some("Patient Record")])),
            Arc::new(StringArray::from(vec![Some("[\"Encounter\"]")])),
            Arc::new(StringArray::from(vec![Some("[\"postTransaction\"]")])),
        ],
    )
    .unwrap_or_else(|error| panic!("semantic scope fixture batch should build: {error}"));

    let payload = semantic_scope_batches_to_ontology_registry_arrow_ipc(&[batch])
        .expect("ontology registry Arrow IPC should build");
    let rows = ontology_registry_rows_from_arrow_ipc(&payload);

    assert!(rows.contains(&("action_type".to_owned(), "postTransaction".to_owned(), true)));
    assert!(rows.contains(&("link_type".to_owned(), "Encounter".to_owned(), false)));
    assert!(rows.contains(&(
        "link_type".to_owned(),
        "PatientRecord.Encounter".to_owned(),
        false
    )));
    assert!(rows.contains(&("object_type".to_owned(), "Patient Record".to_owned(), false)));
    assert!(rows.contains(&("object_type".to_owned(), "PatientRecord".to_owned(), false)));
}

#[test]
fn semantic_scope_batches_skip_empty_object_ids() {
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("objectId", DataType::Utf8, true),
            Field::new("kind", DataType::Utf8, true),
        ])),
        vec![
            Arc::new(StringArray::from(vec![Some("")])),
            Arc::new(StringArray::from(vec![Some("task")])),
        ],
    )
    .unwrap_or_else(|error| panic!("empty object fixture batch should build: {error}"));

    let payload = semantic_scope_batches_to_ontology_registry_arrow_ipc(&[batch])
        .expect("empty ontology registry Arrow IPC should build");
    let rows = ontology_registry_rows_from_arrow_ipc(&payload);
    assert!(rows.is_empty());
}

fn ontology_registry_rows_from_arrow_ipc(payload: &[u8]) -> Vec<(String, String, bool)> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .expect("ontology registry Arrow IPC should decode");
    let mut rows = Vec::new();
    for batch in reader {
        let batch = batch.expect("ontology registry batch should read");
        let families = batch
            .column_by_name("resource_family")
            .expect("resource family column")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("resource family StringArray");
        let names = batch
            .column_by_name("api_name")
            .expect("api name column")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("api name StringArray");
        let requires = batch
            .column_by_name("requires_evidence")
            .expect("requires evidence column")
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("requires evidence BooleanArray");
        for row_index in 0..batch.num_rows() {
            assert!(!families.is_null(row_index));
            assert!(!names.is_null(row_index));
            assert!(!requires.is_null(row_index));
            rows.push((
                families.value(row_index).to_owned(),
                names.value(row_index).to_owned(),
                requires.value(row_index),
            ));
        }
    }
    rows
}
