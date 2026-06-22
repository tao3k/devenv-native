use std::collections::{BTreeMap, BTreeSet};

use super::types::{Row, required_value};

const STRUCTURAL_FACTS_RDF_SEED_FILE: &str = "structural_facts_rdf_seed.ttl";
const STRUCTURAL_FACTS_REVIEW_DECISION: &str = "structural_seed";
const STRUCTURAL_FACTS_PROMOTION_DECISION: &str = "blocked_structure_only";
const STRUCTURAL_FACTS_REVIEWER_ID: &str = "xiuxian-wendao-episteme";

pub(super) fn require_pre_truth(table_name: &str, rows: &[Row]) -> Result<(), String> {
    for (index, row) in rows.iter().enumerate() {
        let value = required_value(row, "ontology_truth", table_name, index + 2)?;
        if !matches!(value.trim(), "false" | "0") {
            return Err(format!(
                "structural facts `{table_name}` row {} attempted to mark ontology truth",
                index + 2
            ));
        }
    }
    Ok(())
}

pub(super) fn relation_counts(relation_rows: &[Row]) -> Result<BTreeMap<String, i64>, String> {
    relation_rows
        .iter()
        .enumerate()
        .try_fold(BTreeMap::new(), |mut counts, (index, relation)| {
            let row_number = index + 2;
            let source = required_value(
                relation,
                "source",
                "structural_facts_read_model_relations",
                row_number,
            )?;
            let target = required_value(
                relation,
                "target",
                "structural_facts_read_model_relations",
                row_number,
            )?;
            *counts.entry(source.to_owned()).or_insert(0) += 1;
            *counts.entry(target.to_owned()).or_insert(0) += 1;
            Ok(counts)
        })
}

pub(super) fn object_rows_to_semantic(
    structural_rows: &[Row],
    relation_counts: &BTreeMap<String, i64>,
) -> Result<Vec<Row>, String> {
    structural_rows
        .iter()
        .enumerate()
        .map(|(index, row)| object_row_to_semantic(row, index + 2, relation_counts))
        .collect()
}

pub(super) fn relation_rows_to_semantic(structural_rows: &[Row]) -> Result<Vec<Row>, String> {
    structural_rows
        .iter()
        .enumerate()
        .map(|(index, row)| relation_row_to_semantic(row, index + 2))
        .collect()
}

pub(super) fn validate_relation_endpoints(
    object_rows: &[Row],
    relation_rows: &[Row],
) -> Result<(), String> {
    let object_ids = object_rows
        .iter()
        .map(|row| required_value(row, "id", "semantic_objects", 0))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for (index, relation) in relation_rows.iter().enumerate() {
        let row_number = index + 2;
        let source = required_value(relation, "source", "semantic_relations", row_number)?;
        let target = required_value(relation, "target", "semantic_relations", row_number)?;
        if !object_ids.contains(source) {
            return Err(format!(
                "structural facts `semantic_relations` row {row_number} references unknown source `{source}`"
            ));
        }
        if !object_ids.contains(target) {
            return Err(format!(
                "structural facts `semantic_relations` row {row_number} references unknown target `{target}`"
            ));
        }
    }
    Ok(())
}

fn object_row_to_semantic(
    row: &Row,
    row_number: usize,
    relation_counts: &BTreeMap<String, i64>,
) -> Result<Row, String> {
    let id = object_value(row, "id", row_number)?;
    let status = object_value(row, "status", row_number)?;
    let evidence_id = object_evidence_id(row, row_number)?;
    let relation_count = relation_counts.get(id).copied().unwrap_or_default();

    Ok(BTreeMap::from([
        ("id".to_string(), id.to_owned()),
        (
            "kind".to_string(),
            object_value(row, "kind", row_number)?.to_owned(),
        ),
        (
            "title".to_string(),
            object_value(row, "title", row_number)?.to_owned(),
        ),
        (
            "domain".to_string(),
            object_value(row, "domain_id", row_number)?.to_owned(),
        ),
        ("evidence_id".to_string(), evidence_id),
        ("evidence_status".to_string(), status.to_owned()),
        (
            "target_rdf_file".to_string(),
            STRUCTURAL_FACTS_RDF_SEED_FILE.to_string(),
        ),
        (
            "review_decision".to_string(),
            STRUCTURAL_FACTS_REVIEW_DECISION.to_string(),
        ),
        (
            "promotion_decision".to_string(),
            STRUCTURAL_FACTS_PROMOTION_DECISION.to_string(),
        ),
        (
            "reviewer_id".to_string(),
            STRUCTURAL_FACTS_REVIEWER_ID.to_string(),
        ),
        ("relation_count".to_string(), relation_count.to_string()),
        ("status".to_string(), status.to_owned()),
        (
            "read_model_projection_staleness".to_string(),
            object_value(row, "read_model_projection_staleness", row_number)?.to_owned(),
        ),
    ]))
}

fn relation_row_to_semantic(row: &Row, row_number: usize) -> Result<Row, String> {
    let status = relation_value(row, "status", row_number)?;
    Ok(BTreeMap::from([
        (
            "id".to_string(),
            relation_value(row, "id", row_number)?.to_owned(),
        ),
        (
            "kind".to_string(),
            relation_value(row, "kind", row_number)?.to_owned(),
        ),
        (
            "source".to_string(),
            relation_value(row, "source", row_number)?.to_owned(),
        ),
        (
            "target".to_string(),
            relation_value(row, "target", row_number)?.to_owned(),
        ),
        (
            "domain".to_string(),
            relation_value(row, "domain_id", row_number)?.to_owned(),
        ),
        (
            "evidence_id".to_string(),
            relation_value(row, "source_contract_id", row_number)?.to_owned(),
        ),
        ("evidence_status".to_string(), status.to_owned()),
        (
            "target_rdf_file".to_string(),
            STRUCTURAL_FACTS_RDF_SEED_FILE.to_string(),
        ),
        (
            "review_decision".to_string(),
            STRUCTURAL_FACTS_REVIEW_DECISION.to_string(),
        ),
        (
            "promotion_decision".to_string(),
            STRUCTURAL_FACTS_PROMOTION_DECISION.to_string(),
        ),
        (
            "reviewer_id".to_string(),
            STRUCTURAL_FACTS_REVIEWER_ID.to_string(),
        ),
        ("status".to_string(), status.to_owned()),
        (
            "read_model_projection_staleness".to_string(),
            relation_value(row, "read_model_projection_staleness", row_number)?.to_owned(),
        ),
    ]))
}

fn object_evidence_id(row: &Row, row_number: usize) -> Result<String, String> {
    let source_content_hash = row
        .get("source_content_hash")
        .map(String::as_str)
        .unwrap_or_default()
        .trim();
    if source_content_hash.is_empty() {
        return Ok(object_value(row, "source_contract_id", row_number)?.to_owned());
    }
    Ok(format!("sha256:{source_content_hash}"))
}

fn object_value<'a>(row: &'a Row, column_name: &str, row_number: usize) -> Result<&'a str, String> {
    required_value(
        row,
        column_name,
        "structural_facts_read_model_objects",
        row_number,
    )
}

fn relation_value<'a>(
    row: &'a Row,
    column_name: &str,
    row_number: usize,
) -> Result<&'a str, String> {
    required_value(
        row,
        column_name,
        "structural_facts_read_model_relations",
        row_number,
    )
}
