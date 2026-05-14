use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::duckdb::{
    DatasetOntologyArrowIpcSourceTableSpec, DatasetOntologyDuckDbMaterializer,
    DatasetOntologyRuntimeMaterializationRequest,
};
use arrow::array::ArrayRef;
use arrow::ipc::writer::StreamWriter;
use xiuxian_wendao_sql::dataset_ontology::{
    DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME, DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME, DatasetOntologyMappingSql,
    DatasetOntologySourceTable, DatasetOntologyValidationRule,
    materialize_dataset_ontology_with_engine,
};

use super::{
    Arc, DataType, DuckDbLocalRelationEngine, Field, RecordBatch, Schema, StringArray, TestResult,
    in_memory_search_duckdb_runtime,
};

#[tokio::test]
async fn duckdb_materializes_healthcare_dataset_ontology_mapping_sql() -> TestResult {
    let temp = tempfile::tempdir()?;
    let engine =
        DuckDbLocalRelationEngine::from_runtime(in_memory_search_duckdb_runtime(temp.path()))
            .map_err(std::io::Error::other)?;

    let report = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &healthcare_contract_mapping_sql()?,
    )
    .await
    .map_err(std::io::Error::other)?;

    assert_eq!(report.execution_engine, "duckdb");
    assert!(report.passed(), "{:?}", report.validation_failures);
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME),
        Some(6)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME),
        Some(1)
    );
    assert_eq!(
        xiuxian_wendao_sql::LocalRelationEngine::relation_materialization_state(
            &engine,
            "raw_patients"
        )
        .map(|state| state.as_str()),
        Some("materialized")
    );
    Ok(())
}

#[tokio::test]
async fn duckdb_runtime_materializer_wraps_contract_metadata() -> TestResult {
    let temp = tempfile::tempdir()?;
    let materializer = DatasetOntologyDuckDbMaterializer::from_runtime(
        in_memory_search_duckdb_runtime(temp.path()),
    )
    .map_err(std::io::Error::other)?;

    let request = DatasetOntologyRuntimeMaterializationRequest::new(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        healthcare_source_tables()?,
        healthcare_contract_mapping_sql()?,
    )
    .map_err(std::io::Error::other)?;

    assert_eq!(
        request.contract_id(),
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(
        request.mapping_id(),
        "healthcare.synthetic_care_delivery.v1"
    );
    assert_eq!(request.source_table_count(), 4);

    let report = materializer
        .materialize(request)
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(
        report.contract_id,
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(report.mapping_id, "healthcare.synthetic_care_delivery.v1");
    assert_eq!(report.source_table_count, 4);
    assert!(report.passed(), "{:?}", report.materialization);
    assert_eq!(
        report
            .materialization
            .row_count_for(DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME),
        Some(8)
    );
    Ok(())
}

#[tokio::test]
async fn duckdb_runtime_materializer_reads_arrow_ipc_source_tables() -> TestResult {
    let temp = tempfile::tempdir()?;
    let materializer = DatasetOntologyDuckDbMaterializer::from_runtime(
        in_memory_search_duckdb_runtime(temp.path()),
    )
    .map_err(std::io::Error::other)?;
    let specs = healthcare_arrow_ipc_specs(temp.path())?;

    let request = DatasetOntologyRuntimeMaterializationRequest::from_arrow_ipc_streams(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        &specs,
        healthcare_contract_mapping_sql()?,
    )
    .map_err(std::io::Error::other)?;

    assert_eq!(request.source_table_count(), 4);

    let report = materializer
        .materialize(request)
        .await
        .map_err(std::io::Error::other)?;

    assert!(report.passed(), "{:?}", report.materialization);
    assert_eq!(
        report
            .materialization
            .row_count_for(DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report
            .materialization
            .row_count_for(DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME),
        Some(6)
    );
    Ok(())
}

#[test]
fn dataset_ontology_runtime_request_rejects_empty_identifiers() -> TestResult {
    let mapping_sql = healthcare_contract_mapping_sql()?;
    let source_tables = healthcare_source_tables()?;

    assert!(
        DatasetOntologyRuntimeMaterializationRequest::new(
            " ",
            "healthcare.synthetic_care_delivery.v1",
            source_tables.clone(),
            mapping_sql.clone(),
        )
        .is_err()
    );
    assert!(
        DatasetOntologyRuntimeMaterializationRequest::new(
            "healthcare.synthetic_care_delivery.contract.v1",
            "",
            source_tables,
            mapping_sql,
        )
        .is_err()
    );
    Ok(())
}

fn healthcare_contract_mapping_sql() -> Result<DatasetOntologyMappingSql, std::io::Error> {
    Ok(DatasetOntologyMappingSql {
        object_observations: ontology_file(
            "30_Healthcare/mappings/sql/01_object_observations.sql",
        )?,
        link_observations: ontology_file("30_Healthcare/mappings/sql/02_link_observations.sql")?,
        evidence: ontology_file("30_Healthcare/mappings/sql/03_evidence.sql")?,
        semantic_objects: ontology_file("30_Healthcare/mappings/sql/04_semantic_objects.sql")?,
        semantic_relations: ontology_file("30_Healthcare/mappings/sql/05_semantic_relations.sql")?,
        semantic_projection_state: ontology_file(
            "30_Healthcare/mappings/sql/06_semantic_projection_state.sql",
        )?,
        validation_rules: vec![DatasetOntologyValidationRule::new(
            "HEALTHCARE_ENCOUNTER_MISSING_CONTEXT",
            ontology_file("30_Healthcare/rules/01_encounter_must_link_patient_provider.sql")?,
        )],
    })
}

fn ontology_file(relative_path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(ontology_root().join(relative_path))
}

fn ontology_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../wendao-episteme/ontology")
}

fn healthcare_source_tables() -> Result<Vec<DatasetOntologySourceTable>, Box<dyn std::error::Error>>
{
    Ok(vec![
        string_table(
            "raw_patients",
            &["patient_id", "patient_name", "birth_year", "source_system"],
            &[
                &["P001", "Ada Lovelace", "1981", "synthetic_ehr"],
                &["P002", "Grace Hopper", "1975", "synthetic_ehr"],
            ],
        )?,
        string_table(
            "raw_providers",
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
        )?,
        string_table(
            "raw_encounters",
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
        )?,
        string_table(
            "raw_conditions",
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
        )?,
    ])
}

fn healthcare_arrow_ipc_specs(
    root: &Path,
) -> Result<Vec<DatasetOntologyArrowIpcSourceTableSpec>, Box<dyn std::error::Error>> {
    Ok(vec![
        write_string_table_ipc(
            root,
            "raw_patients",
            &["patient_id", "patient_name", "birth_year", "source_system"],
            &[
                &["P001", "Ada Lovelace", "1981", "synthetic_ehr"],
                &["P002", "Grace Hopper", "1975", "synthetic_ehr"],
            ],
        )?,
        write_string_table_ipc(
            root,
            "raw_providers",
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
        )?,
        write_string_table_ipc(
            root,
            "raw_encounters",
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
        )?,
        write_string_table_ipc(
            root,
            "raw_conditions",
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
        )?,
    ])
}

fn write_string_table_ipc(
    root: &Path,
    table_name: &str,
    columns: &[&str],
    rows: &[&[&str]],
) -> Result<DatasetOntologyArrowIpcSourceTableSpec, Box<dyn std::error::Error>> {
    let (schema, batch) = string_record_batch(columns, rows)?;
    let path = root.join(format!("{table_name}.arrow"));
    let file = File::create(&path)?;
    let mut writer = StreamWriter::try_new(file, schema.as_ref())?;
    writer.write(&batch)?;
    writer.finish()?;
    Ok(DatasetOntologyArrowIpcSourceTableSpec::new(
        table_name, path,
    )?)
}

fn string_table(
    table_name: &str,
    columns: &[&str],
    rows: &[&[&str]],
) -> Result<DatasetOntologySourceTable, Box<dyn std::error::Error>> {
    let (schema, batch) = string_record_batch(columns, rows)?;
    Ok(DatasetOntologySourceTable::new(
        table_name,
        schema,
        vec![batch],
    )?)
}

fn string_record_batch(
    columns: &[&str],
    rows: &[&[&str]],
) -> Result<(Arc<Schema>, RecordBatch), Box<dyn std::error::Error>> {
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
                rows.iter()
                    .map(|row| row[column_index])
                    .collect::<Vec<&str>>(),
            )) as ArrayRef
        })
        .collect::<Vec<_>>();
    let batch = RecordBatch::try_new(Arc::clone(&schema), arrays)?;
    Ok((schema, batch))
}
