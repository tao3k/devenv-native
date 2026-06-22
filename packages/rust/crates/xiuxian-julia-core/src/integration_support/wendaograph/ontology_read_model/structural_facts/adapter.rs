//! Structural-facts adapter for `WendaoGraph` ontology read-model quality.

use std::path::Path;

use super::{
    batch::{projection_state_batch, relation_batch, semantic_object_batch},
    convert::{
        object_rows_to_semantic, relation_counts, relation_rows_to_semantic, require_pre_truth,
        validate_relation_endpoints,
    },
    read::{read_projection_state_rows, read_structural_rows},
};
use crate::integration_support::wendaograph::ontology_read_model::types::WendaoGraphOntologyReadModelQualityRequestBatches;

const STRUCTURAL_FACTS_OBJECTS_PARQUET: &str = "structural_facts_read_model_objects.parquet";
const STRUCTURAL_FACTS_RELATIONS_PARQUET: &str = "structural_facts_read_model_relations.parquet";
const STRUCTURAL_FACTS_PROJECTION_STATE_JSON: &str =
    "structural_facts_read_model_projection_state.json";

const STRUCTURAL_FACTS_OBJECT_COLUMNS: &[&str] = &[
    "id",
    "kind",
    "title",
    "read_model_projection_staleness",
    "domain_id",
    "source_contract_id",
    "ontology_truth",
    "status",
];

const STRUCTURAL_FACTS_RELATION_COLUMNS: &[&str] = &[
    "id",
    "kind",
    "source",
    "target",
    "read_model_projection_staleness",
    "domain_id",
    "source_contract_id",
    "ontology_truth",
    "status",
];

/// Build `WendaoGraph` quality request tables from Episteme structural-facts artifacts.
///
/// This converter accepts only generated Parquet read-model artifacts plus JSON
/// projection state. It does not read private corpus files, RDF source files,
/// `episteme.toml`, or `wendao.toml`.
///
/// # Errors
///
/// Returns an error when required artifact files are missing, Parquet/JSON
/// content is malformed, required columns are absent, structural facts attempt
/// to mark ontology truth, numeric fields are invalid, or a relation points to
/// an object ID absent from the structural-facts object table.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_structural_facts_artifacts(
    run_dir: impl AsRef<Path>,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    let run_dir = run_dir.as_ref();
    let structural_object_rows = read_structural_rows(
        &run_dir.join(STRUCTURAL_FACTS_OBJECTS_PARQUET),
        "structural_facts_read_model_objects",
        STRUCTURAL_FACTS_OBJECT_COLUMNS,
    )?;
    let structural_relation_rows = read_structural_rows(
        &run_dir.join(STRUCTURAL_FACTS_RELATIONS_PARQUET),
        "structural_facts_read_model_relations",
        STRUCTURAL_FACTS_RELATION_COLUMNS,
    )?;
    let projection_rows =
        read_projection_state_rows(&run_dir.join(STRUCTURAL_FACTS_PROJECTION_STATE_JSON))?;

    require_pre_truth(
        "structural_facts_read_model_objects",
        &structural_object_rows,
    )?;
    require_pre_truth(
        "structural_facts_read_model_relations",
        &structural_relation_rows,
    )?;

    let relation_counts = relation_counts(&structural_relation_rows)?;
    let object_rows = object_rows_to_semantic(&structural_object_rows, &relation_counts)?;
    let relation_rows = relation_rows_to_semantic(&structural_relation_rows)?;
    validate_relation_endpoints(&object_rows, &relation_rows)?;

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        semantic_object_batch(&object_rows)?,
        relation_batch(&relation_rows)?,
        projection_state_batch(&projection_rows)?,
    ))
}
