use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};

use arrow::array::{Array, BinaryArray, BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;

use super::WendaoGraphOntologyReadModelQualityRequestBatches;

pub(super) fn sample_request_batches() -> WendaoGraphOntologyReadModelQualityRequestBatches {
    WendaoGraphOntologyReadModelQualityRequestBatches::new(
        string_batch("id", "PatientRecord"),
        relation_batch(),
        projection_state_batch(),
    )
}

pub(super) fn parent_registry_batches() -> (RecordBatch, RecordBatch) {
    (parent_object_types_batch(), parent_link_types_batch())
}

pub(super) fn decode_single_batch(payload: &[u8], table_name: &str) -> RecordBatch {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .unwrap_or_else(|error| panic!("open `{table_name}` Arrow stream: {error}"));
    let batches = reader
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("decode `{table_name}` Arrow stream: {error}"));

    let [batch] = batches.as_slice() else {
        panic!("expected one `{table_name}` batch, got {}", batches.len());
    };
    batch.clone()
}

pub(super) fn assert_binary_column_matches(
    batch: &RecordBatch,
    column_name: &str,
    expected: &[u8],
) {
    let column = batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
        .unwrap_or_else(|| panic!("missing binary `{column_name}` column"));
    assert!(!column.is_null(0));
    assert_eq!(column.value(0), expected);
}

pub(super) fn string_column_value(
    batch: &RecordBatch,
    column_name: &str,
    row_index: usize,
) -> String {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("missing string `{column_name}` column"))
        .value(row_index)
        .to_owned()
}

pub(super) fn dataset_ontology_envelope_batch(
    record_kind: &str,
    table_name: &str,
    payload_json_values: &[String],
) -> RecordBatch {
    let row_count = payload_json_values.len();
    let payload_json_refs = payload_json_values
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("contractId", DataType::Utf8, false),
            Field::new("mappingId", DataType::Utf8, false),
            Field::new("recordKind", DataType::Utf8, false),
            Field::new("tableName", DataType::Utf8, false),
            Field::new("rowIndex", DataType::UInt64, false),
            Field::new("passed", DataType::Boolean, false),
            Field::new("executionEngine", DataType::Utf8, false),
            Field::new("sourceTableCount", DataType::UInt64, false),
            Field::new("observationTableCount", DataType::UInt64, false),
            Field::new("semanticReadModelTableCount", DataType::UInt64, false),
            Field::new("validationFailureCount", DataType::UInt64, false),
            Field::new("payloadJson", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec![
                "healthcare.synthetic_care_delivery.contract.v1";
                row_count
            ])),
            Arc::new(StringArray::from(vec![
                "healthcare.synthetic_care_delivery.v1";
                row_count
            ])),
            Arc::new(StringArray::from(vec![record_kind; row_count])),
            Arc::new(StringArray::from(vec![table_name; row_count])),
            Arc::new(UInt64Array::from((0..row_count as u64).collect::<Vec<_>>())),
            Arc::new(BooleanArray::from(vec![true; row_count])),
            Arc::new(StringArray::from(vec!["rust-duckdb-arrow"; row_count])),
            Arc::new(UInt64Array::from(vec![4; row_count])),
            Arc::new(UInt64Array::from(vec![3; row_count])),
            Arc::new(UInt64Array::from(vec![3; row_count])),
            Arc::new(UInt64Array::from(vec![0; row_count])),
            Arc::new(StringArray::from(payload_json_refs)),
        ],
    )
    .unwrap_or_else(|error| panic!("build dataset ontology envelope `{table_name}`: {error}"))
}

pub(super) fn write_semantic_read_model_fixture(root: &Path) -> io::Result<()> {
    write_file(
        &root.join("objects/component/demo.md"),
        r"---
id: component.demo
kind: component
title: Demo Component
status: active
confidence:
  score: 0.95
  source: verified
owners:
  - scope: xiuxian-wendao-sql
    role: read_model_source
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-sql semantic_read_model
relations:
  - kind: validates
    target: task.demo
---

# Demo Component
",
    )?;
    write_file(
        &root.join("objects/task/demo.md"),
        r"---
id: task.demo
kind: task
title: Demo Task
status: active
confidence:
  score: 0.9
  source: human_signed
owners:
  - scope: xiuxian-wendao-sql
    role: read_model_target
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-sql semantic_read_model
relations: []
---

# Demo Task
",
    )?;
    write_file(
        &root.join("projections/llm-compression.md"),
        r"---
type: semantic_projection
projection: llm_compression
source_objects:
  - component.demo
  - task.demo
source_revision: stale-demo
projection_revision: semantic-read-model-demo
staleness: stale
status: active
---

# LLM Compression
",
    )
}

fn string_batch(column_name: &str, value: &str) -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            column_name,
            DataType::Utf8,
            false,
        )])),
        vec![Arc::new(StringArray::from(vec![value]))],
    )
    .unwrap_or_else(|error| panic!("build `{column_name}` batch: {error}"))
}

fn relation_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("source", DataType::Utf8, false),
            Field::new("target", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["PatientRecord"])),
            Arc::new(StringArray::from(vec!["Encounter"])),
        ],
    )
    .unwrap_or_else(|error| panic!("build relation batch: {error}"))
}

fn projection_state_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("projection", DataType::Utf8, false),
            Field::new("active", DataType::Boolean, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["semantic_read_model"])),
            Arc::new(BooleanArray::from(vec![true])),
        ],
    )
    .unwrap_or_else(|error| panic!("build projection state batch: {error}"))
}

fn parent_object_types_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("api_name", DataType::Utf8, false),
            Field::new("domain", DataType::Utf8, false),
            Field::new("rdf_class", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["Patient", "Encounter"])),
            Arc::new(StringArray::from(vec![
                "episteme://30_Healthcare",
                "episteme://30_Healthcare",
            ])),
            Arc::new(StringArray::from(vec![
                "https://wendao.ai/ontology/healthcare#Patient",
                "https://wendao.ai/ontology/healthcare#Encounter",
            ])),
        ],
    )
    .unwrap_or_else(|error| panic!("build parent object types batch: {error}"))
}

fn parent_link_types_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("api_name", DataType::Utf8, false),
            Field::new("domain", DataType::Utf8, false),
            Field::new("rdf_property", DataType::Utf8, false),
            Field::new("from_object_type", DataType::Utf8, false),
            Field::new("to_object_type", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["Patient.encounters"])),
            Arc::new(StringArray::from(vec!["episteme://30_Healthcare"])),
            Arc::new(StringArray::from(vec![
                "https://wendao.ai/ontology/healthcare#hasEncounter",
            ])),
            Arc::new(StringArray::from(vec!["Patient"])),
            Arc::new(StringArray::from(vec!["Encounter"])),
        ],
    )
    .unwrap_or_else(|error| panic!("build parent link types batch: {error}"))
}

fn write_file(path: &Path, body: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)
}
