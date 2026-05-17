use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::semantic_scope_batches_to_ontology_registry_tsv;

#[test]
fn semantic_scope_batches_convert_to_ontology_registry_tsv() {
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

    let tsv = semantic_scope_batches_to_ontology_registry_tsv(&[batch]);
    let rows = tsv.lines().collect::<Vec<_>>();

    assert!(rows.contains(&"action_type\tpostTransaction\ttrue"));
    assert!(rows.contains(&"link_type\tEncounter\tfalse"));
    assert!(rows.contains(&"link_type\tPatientRecord.Encounter\tfalse"));
    assert!(rows.contains(&"object_type\tPatient Record\tfalse"));
    assert!(rows.contains(&"object_type\tPatientRecord\tfalse"));
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

    assert!(semantic_scope_batches_to_ontology_registry_tsv(&[batch]).is_empty());
}
