//! Semantic preview artifact conversion for `WendaoGraph` ontology quality checks.

use std::collections::BTreeSet;
use std::path::Path;

use super::batch::{projection_state_batch, relation_batch, semantic_object_batch};
use super::read::{read_parquet_rows, read_projection_state_rows};
use super::types::{Column, Row, required_value};
use crate::integration_support::wendaograph::ontology_read_model::types::WendaoGraphOntologyReadModelQualityRequestBatches;

const SEMANTIC_OBJECTS_PARQUET: &str = "semantic_objects.parquet";
const SEMANTIC_RELATIONS_PARQUET: &str = "semantic_relations.parquet";
const SEMANTIC_PROJECTION_STATE_JSON: &str = "semantic_projection_state.json";
const RDF_SOURCE_SEMANTIC_OBJECTS_PARQUET: &str = "rdf_source_semantic_objects.parquet";
const RDF_SOURCE_SEMANTIC_RELATIONS_PARQUET: &str = "rdf_source_semantic_relations.parquet";
const RDF_SOURCE_SEMANTIC_PROJECTION_STATE_JSON: &str = "rdf_source_projection_state.json";

const SEMANTIC_OBJECT_COLUMNS: &[Column] = &[
    Column::string("id"),
    Column::string("kind"),
    Column::string("title"),
    Column::string("domain"),
    Column::string("evidence_id"),
    Column::string("evidence_status"),
    Column::string("target_rdf_file"),
    Column::string("review_decision"),
    Column::string("promotion_decision"),
    Column::string("reviewer_id"),
    Column::int64("relation_count"),
    Column::string("status"),
    Column::string("read_model_projection_staleness"),
];

const SEMANTIC_RELATION_COLUMNS: &[Column] = &[
    Column::string("id"),
    Column::string("kind"),
    Column::string("source"),
    Column::string("target"),
    Column::string("domain"),
    Column::string("evidence_id"),
    Column::string("evidence_status"),
    Column::string("target_rdf_file"),
    Column::string("review_decision"),
    Column::string("promotion_decision"),
    Column::string("reviewer_id"),
    Column::string("status"),
    Column::string("read_model_projection_staleness"),
];

/// Build `WendaoGraph` quality request tables from compiled Episteme semantic preview artifacts.
///
/// This converter accepts only generated Parquet read-model artifacts plus JSON
/// projection state. It does not read private corpus files, RDF source files,
/// `episteme.toml`, or `wendao.toml`.
///
/// # Errors
///
/// Returns an error when required artifact files are missing, Parquet/JSON
/// content is malformed, required columns are absent, numeric fields are
/// invalid, or a semantic relation points to an object ID absent from the
/// semantic object table.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_semantic_preview_artifacts(
    run_dir: impl AsRef<Path>,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    build_request_batches_from_artifacts(
        run_dir.as_ref(),
        SemanticArtifactPaths {
            objects_parquet: SEMANTIC_OBJECTS_PARQUET,
            relations_parquet: SEMANTIC_RELATIONS_PARQUET,
            projection_state_json: SEMANTIC_PROJECTION_STATE_JSON,
            artifact_label: "semantic preview",
        },
    )
}

/// Build `WendaoGraph` quality request tables from applied-RDF source read-model artifacts.
///
/// This converter accepts only generated Parquet read-model artifacts plus JSON
/// projection state. It does not read private corpus files, RDF source files,
/// `episteme.toml`, or `wendao.toml`.
///
/// # Errors
///
/// Returns an error when required artifact files are missing, Parquet/JSON
/// content is malformed, required columns are absent, numeric fields are
/// invalid, or a semantic relation points to an object ID absent from the RDF
/// source object table.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_rdf_source_artifacts(
    run_dir: impl AsRef<Path>,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    build_request_batches_from_artifacts(
        run_dir.as_ref(),
        SemanticArtifactPaths {
            objects_parquet: RDF_SOURCE_SEMANTIC_OBJECTS_PARQUET,
            relations_parquet: RDF_SOURCE_SEMANTIC_RELATIONS_PARQUET,
            projection_state_json: RDF_SOURCE_SEMANTIC_PROJECTION_STATE_JSON,
            artifact_label: "RDF source read-model",
        },
    )
}

#[derive(Debug, Clone, Copy)]
struct SemanticArtifactPaths {
    objects_parquet: &'static str,
    relations_parquet: &'static str,
    projection_state_json: &'static str,
    artifact_label: &'static str,
}

fn build_request_batches_from_artifacts(
    run_dir: &Path,
    artifacts: SemanticArtifactPaths,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    let object_rows = read_parquet_rows(
        &run_dir.join(artifacts.objects_parquet),
        artifacts.artifact_label,
        "semantic_objects",
        SEMANTIC_OBJECT_COLUMNS,
    )?;
    let relation_rows = read_parquet_rows(
        &run_dir.join(artifacts.relations_parquet),
        artifacts.artifact_label,
        "semantic_relations",
        SEMANTIC_RELATION_COLUMNS,
    )?;
    let projection_rows = read_projection_state_rows(
        &run_dir.join(artifacts.projection_state_json),
        artifacts.artifact_label,
    )?;

    validate_relation_endpoints(artifacts.artifact_label, &object_rows, &relation_rows)?;

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        semantic_object_batch(&object_rows, artifacts.artifact_label)?,
        relation_batch(&relation_rows, artifacts.artifact_label)?,
        projection_state_batch(&projection_rows, artifacts.artifact_label)?,
    ))
}

fn validate_relation_endpoints(
    artifact_label: &str,
    object_rows: &[Row],
    relation_rows: &[Row],
) -> Result<(), String> {
    let object_ids = object_rows
        .iter()
        .map(|row| required_value(row, "id", "semantic_objects", 0, artifact_label))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for (index, relation) in relation_rows.iter().enumerate() {
        let row_number = index + 2;
        let source = required_value(
            relation,
            "source",
            "semantic_relations",
            row_number,
            artifact_label,
        )?;
        let target = required_value(
            relation,
            "target",
            "semantic_relations",
            row_number,
            artifact_label,
        )?;
        if !object_ids.contains(source) {
            return Err(format!(
                "{artifact_label} `semantic_relations` row {row_number} references unknown source `{source}`"
            ));
        }
        if !object_ids.contains(target) {
            return Err(format!(
                "{artifact_label} `semantic_relations` row {row_number} references unknown target `{target}`"
            ));
        }
    }
    Ok(())
}
