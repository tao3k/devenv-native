use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use crate::DuckDbLocalRelationEngine;
use crate::dataset_ontology::{
    DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME, DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME, DatasetOntologyMappingSql,
    DatasetOntologySourceTable, DatasetOntologyValidationRule,
    materialize_dataset_ontology_with_engine, validate_dataset_ontology_select_only_sql,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

mod materialization;
mod sql;

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

fn string_table(
    table_name: &str,
    columns: &[&str],
    rows: &[&[&str]],
) -> Result<DatasetOntologySourceTable, Box<dyn std::error::Error>> {
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
            )) as _
        })
        .collect::<Vec<_>>();
    let batch = RecordBatch::try_new(Arc::clone(&schema), arrays)?;
    Ok(DatasetOntologySourceTable::new(
        table_name,
        schema,
        vec![batch],
    )?)
}

fn healthcare_mapping_sql(include_provider_links: bool) -> DatasetOntologyMappingSql {
    DatasetOntologyMappingSql {
        object_observations: object_observations_sql().into(),
        link_observations: link_observations_sql(include_provider_links).into(),
        evidence: evidence_sql().into(),
        semantic_objects: semantic_objects_sql().into(),
        semantic_relations: semantic_relations_sql().into(),
        semantic_projection_state: semantic_projection_state_sql().into(),
        validation_rules: vec![DatasetOntologyValidationRule::new(
            "HEALTHCARE_ENCOUNTER_MISSING_CONTEXT",
            encounter_context_validation_sql(),
        )],
    }
}

fn object_observations_sql() -> String {
    r"
WITH object_rows AS (
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'Patient' AS object_type,
    'https://wendao.ai/ontology/healthcare#Patient' AS rdf_class,
    'healthcare://Patient/' || patient_id AS object_id,
    patient_name AS display_name,
    'raw_patients' AS source_table,
    patient_id AS source_row_id,
    patient_id AS source_hash
  FROM raw_patients
  UNION ALL
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'CareProvider' AS object_type,
    'https://wendao.ai/ontology/healthcare#CareProvider' AS rdf_class,
    'healthcare://CareProvider/' || provider_id AS object_id,
    provider_name AS display_name,
    'raw_providers' AS source_table,
    provider_id AS source_row_id,
    provider_id AS source_hash
  FROM raw_providers
  UNION ALL
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'Encounter' AS object_type,
    'https://wendao.ai/ontology/healthcare#Encounter' AS rdf_class,
    'healthcare://Encounter/' || encounter_id AS object_id,
    encounter_label AS display_name,
    'raw_encounters' AS source_table,
    encounter_id AS source_row_id,
    encounter_id AS source_hash
  FROM raw_encounters
  UNION ALL
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'MedicalCondition' AS object_type,
    'https://wendao.ai/ontology/healthcare#MedicalCondition' AS rdf_class,
    'healthcare://MedicalCondition/' || condition_id AS object_id,
    condition_name AS display_name,
    'raw_conditions' AS source_table,
    condition_id AS source_row_id,
    condition_id AS source_hash
  FROM raw_conditions
)
SELECT *
FROM object_rows
ORDER BY object_id
"
    .to_string()
}

fn link_observations_sql(include_provider_links: bool) -> String {
    let provider_links = if include_provider_links {
        r"
  UNION ALL
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'CareProvider.performsEncounter' AS link_type,
    'https://wendao.ai/ontology/core#performs' AS rdf_property,
    'healthcare://CareProvider/' || provider_id AS source_object_id,
    'healthcare://Encounter/' || encounter_id AS target_object_id,
    'raw_encounters' AS source_table,
    encounter_id AS source_row_id,
    provider_id AS source_hash
  FROM raw_encounters
"
    } else {
        ""
    };
    format!(
        r"
WITH link_rows AS (
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'Patient.encounters' AS link_type,
    'https://wendao.ai/ontology/healthcare#hasEncounter' AS rdf_property,
    'healthcare://Patient/' || patient_id AS source_object_id,
    'healthcare://Encounter/' || encounter_id AS target_object_id,
    'raw_encounters' AS source_table,
    encounter_id AS source_row_id,
    patient_id AS source_hash
  FROM raw_encounters
{provider_links}
  UNION ALL
  SELECT
    'healthcare.synthetic_care_delivery.v1' AS mapping_id,
    'episteme://30_Healthcare' AS domain,
    'Patient.conditions' AS link_type,
    'https://wendao.ai/ontology/healthcare#diagnosedWith' AS rdf_property,
    'healthcare://Patient/' || patient_id AS source_object_id,
    'healthcare://MedicalCondition/' || condition_id AS target_object_id,
    'raw_conditions' AS source_table,
    condition_id AS source_row_id,
    condition_id AS source_hash
  FROM raw_conditions
)
SELECT *
FROM link_rows
ORDER BY source_object_id, rdf_property, target_object_id
"
    )
}

fn evidence_sql() -> String {
    r"
SELECT
  'evidence:raw_patients:' || patient_id AS evidence_id,
  'table_row' AS evidence_kind,
  'raw_patients' AS source_table,
  patient_id AS source_row_id,
  patient_id AS source_hash,
  'Patient row for ' || patient_name AS evidence_text
FROM raw_patients
UNION ALL
SELECT
  'evidence:raw_providers:' || provider_id AS evidence_id,
  'table_row' AS evidence_kind,
  'raw_providers' AS source_table,
  provider_id AS source_row_id,
  provider_id AS source_hash,
  'Care provider row for ' || provider_name AS evidence_text
FROM raw_providers
UNION ALL
SELECT
  'evidence:raw_encounters:' || encounter_id AS evidence_id,
  'table_row' AS evidence_kind,
  'raw_encounters' AS source_table,
  encounter_id AS source_row_id,
  encounter_id AS source_hash,
  'Encounter row for ' || encounter_label AS evidence_text
FROM raw_encounters
UNION ALL
SELECT
  'evidence:raw_conditions:' || condition_id AS evidence_id,
  'table_row' AS evidence_kind,
  'raw_conditions' AS source_table,
  condition_id AS source_row_id,
  condition_id AS source_hash,
  'Condition row for ' || condition_name AS evidence_text
FROM raw_conditions
ORDER BY evidence_id
"
    .to_string()
}

fn semantic_objects_sql() -> String {
    r"
SELECT
  object_id AS id,
  object_type AS kind,
  display_name AS title,
  'active' AS status,
  0 AS relation_count,
  'fresh' AS read_model_projection_staleness
FROM ontology_object_observation
ORDER BY id
"
    .to_string()
}

fn semantic_relations_sql() -> String {
    r"
SELECT
  source_object_id AS source,
  link_type AS kind,
  target_object_id AS target,
  'fresh' AS read_model_projection_staleness
FROM ontology_link_observation
ORDER BY source, kind, target
"
    .to_string()
}

fn semantic_projection_state_sql() -> String {
    r"
SELECT
  'healthcare.synthetic_care_delivery.v1' AS projection,
  'active' AS status,
  'fresh' AS staleness,
  count(*) AS source_object_count
FROM ontology_object_observation
"
    .to_string()
}

fn encounter_context_validation_sql() -> String {
    r"
WITH encounters AS (
  SELECT entity_id
  FROM ontology_entity
  WHERE class_iri = 'https://wendao.ai/ontology/healthcare#Encounter'
),
patient_links AS (
  SELECT DISTINCT target_id
  FROM ontology_relation
  WHERE predicate = 'https://wendao.ai/ontology/healthcare#hasEncounter'
),
provider_links AS (
  SELECT DISTINCT target_id
  FROM ontology_relation
  WHERE predicate = 'https://wendao.ai/ontology/core#performs'
)
SELECT
  encounter.entity_id,
  'HEALTHCARE_ENCOUNTER_MISSING_CONTEXT' AS violation_type
FROM encounters encounter
LEFT JOIN patient_links patient
  ON encounter.entity_id = patient.target_id
LEFT JOIN provider_links provider
  ON encounter.entity_id = provider.target_id
WHERE patient.target_id IS NULL
   OR provider.target_id IS NULL
"
    .to_string()
}
