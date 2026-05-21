use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::duckdb::DatasetOntologyArrowIpcSourceTableSpec;
use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_sql::dataset_ontology::{
    DatasetOntologyMappingSql, DatasetOntologySourceTable, DatasetOntologyValidationRule,
};

pub(super) fn healthcare_contract_mapping_sql() -> Result<DatasetOntologyMappingSql, std::io::Error>
{
    Ok(DatasetOntologyMappingSql {
        object_observations: ontology_file(
            "30_Healthcare/mappings/sql/01_object_observations.sql",
        )?
        .into(),
        link_observations: ontology_file("30_Healthcare/mappings/sql/02_link_observations.sql")?
            .into(),
        evidence: ontology_file("30_Healthcare/mappings/sql/03_evidence.sql")?.into(),
        semantic_objects: ontology_file("30_Healthcare/mappings/sql/04_semantic_objects.sql")?
            .into(),
        semantic_relations: ontology_file("30_Healthcare/mappings/sql/05_semantic_relations.sql")?
            .into(),
        semantic_projection_state: ontology_file(
            "30_Healthcare/mappings/sql/06_semantic_projection_state.sql",
        )?
        .into(),
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

pub(super) fn healthcare_source_tables()
-> Result<Vec<DatasetOntologySourceTable>, Box<dyn std::error::Error>> {
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

pub(super) fn healthcare_arrow_ipc_specs(
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

#[cfg(feature = "julia")]
pub(super) fn healthcare_parent_registry_batches()
-> Result<(RecordBatch, RecordBatch), Box<dyn std::error::Error>> {
    let (_, parent_object_types) = string_record_batch(
        &["api_name", "domain", "rdf_class"],
        &[
            &[
                "Patient",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#Patient",
            ],
            &[
                "CareProvider",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#CareProvider",
            ],
            &[
                "Encounter",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#Encounter",
            ],
            &[
                "MedicalCondition",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#MedicalCondition",
            ],
        ],
    )?;
    let (_, parent_link_types) = string_record_batch(
        &[
            "api_name",
            "domain",
            "rdf_property",
            "from_object_type",
            "to_object_type",
        ],
        &[
            &[
                "Patient.encounters",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#hasEncounter",
                "Patient",
                "Encounter",
            ],
            &[
                "Patient.conditions",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#hasCondition",
                "Patient",
                "MedicalCondition",
            ],
            &[
                "CareProvider.performsEncounter",
                "episteme://30_Healthcare",
                "https://wendao.ai/ontology/healthcare#performsEncounter",
                "CareProvider",
                "Encounter",
            ],
        ],
    )?;
    Ok((parent_object_types, parent_link_types))
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

pub(super) fn string_record_batch(
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
