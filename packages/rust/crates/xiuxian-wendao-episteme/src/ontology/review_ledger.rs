//! Private and common review-ledger admission for ontology source contracts.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::{
    OrgOntologyAuthoringDocument, OrgOntologyAuthoringTable,
    compile_org_ontology_authoring_document,
};

use super::manifest::{
    EpistemeOntologyError, invalid_contract, read_to_string, resolve_ontology_artifact_path,
};

const OBJECT_INSTANCE_REVIEW_TABLE: &str = "object_instance_review";
const INSTANCE_RELATION_REVIEW_TABLE: &str = "instance_relation_review";
const APPROVED_PROMOTION_DECISION: &str = "approved";
const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

#[derive(Debug, Deserialize)]
struct ReviewLedgerToml {
    schema_version: u32,
    ledger_id: Option<String>,
    domain: Option<String>,
    ledger_org: Option<String>,
    ledger_org_sha256: Option<String>,
    source_mutation_allowed: Option<bool>,
    ontology_truth: Option<bool>,
    promotion_allowed: Option<bool>,
}

#[derive(Debug, Default)]
pub(super) struct ReviewLedgerSet {
    pub(super) object_rows: Vec<ObjectInstanceRow>,
    pub(super) relation_rows: Vec<InstanceRelationRow>,
}

#[derive(Debug)]
struct ReviewLedgerDocument {
    path: String,
    object_rows: Vec<ObjectInstanceRow>,
    relation_rows: Vec<InstanceRelationRow>,
}

#[derive(Debug, Clone)]
pub(super) struct ObjectInstanceRow {
    pub(super) domain_id: String,
    pub(super) object_id: String,
    pub(super) object_type: String,
    pub(super) label: String,
    pub(super) evidence_id: String,
    pub(super) review_decision: String,
    pub(super) promotion_decision: String,
    pub(super) reviewer_id: String,
}

#[derive(Debug, Clone)]
pub(super) struct InstanceRelationRow {
    pub(super) domain_id: String,
    pub(super) relation_id: String,
    pub(super) source_object_id: String,
    pub(super) predicate: String,
    pub(super) target_object_id: String,
    pub(super) evidence_id: String,
    pub(super) review_decision: String,
    pub(super) promotion_decision: String,
    pub(super) reviewer_id: String,
}

pub(super) fn validate_review_ledgers(
    episteme_root: &Path,
    paths: &[String],
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    let ledger_set = read_review_ledger_set(episteme_root, paths, field)?;
    validate_review_ledger_set(&ledger_set, field)
}

pub(super) fn read_review_ledger_set(
    episteme_root: &Path,
    paths: &[String],
    field: &str,
) -> Result<ReviewLedgerSet, EpistemeOntologyError> {
    let mut documents = Vec::new();
    for raw_path in paths {
        documents.push(read_review_ledger(episteme_root, raw_path, field)?);
    }

    let object_rows = documents
        .iter()
        .flat_map(|document| document.object_rows.iter().cloned())
        .collect::<Vec<_>>();
    let relation_rows = documents
        .iter()
        .flat_map(|document| document.relation_rows.iter().cloned())
        .collect::<Vec<_>>();
    let ledger_set = ReviewLedgerSet {
        object_rows,
        relation_rows,
    };
    validate_review_ledger_documents(&documents, field)?;
    Ok(ledger_set)
}

fn validate_review_ledger_documents(
    documents: &[ReviewLedgerDocument],
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    let mut object_ids = BTreeSet::new();
    for document in documents {
        for row in &document.object_rows {
            if !object_ids.insert(row.object_id.as_str()) {
                return Err(invalid_contract(format!(
                    "{field} `{}` contains duplicate object_id `{}`",
                    document.path, row.object_id
                )));
            }
        }
    }

    let mut relation_ids = BTreeSet::new();
    for document in documents {
        for row in &document.relation_rows {
            if !relation_ids.insert(row.relation_id.as_str()) {
                return Err(invalid_contract(format!(
                    "{field} `{}` contains duplicate relation_id `{}`",
                    document.path, row.relation_id
                )));
            }
            if !object_ids.contains(row.source_object_id.as_str()) {
                return Err(invalid_contract(format!(
                    "{field} `{}` relation `{}` references unknown source_object_id `{}`",
                    document.path, row.relation_id, row.source_object_id
                )));
            }
            if !object_ids.contains(row.target_object_id.as_str()) {
                return Err(invalid_contract(format!(
                    "{field} `{}` relation `{}` references unknown target_object_id `{}`",
                    document.path, row.relation_id, row.target_object_id
                )));
            }
        }
    }

    Ok(())
}

fn validate_review_ledger_set(
    ledger_set: &ReviewLedgerSet,
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    let mut object_ids = BTreeSet::new();
    for row in &ledger_set.object_rows {
        if !object_ids.insert(row.object_id.as_str()) {
            return Err(invalid_contract(format!(
                "{field} contains duplicate object_id `{}`",
                row.object_id
            )));
        }
    }

    let mut relation_ids = BTreeSet::new();
    for row in &ledger_set.relation_rows {
        if !relation_ids.insert(row.relation_id.as_str()) {
            return Err(invalid_contract(format!(
                "{field} contains duplicate relation_id `{}`",
                row.relation_id
            )));
        }
        if !object_ids.contains(row.source_object_id.as_str()) {
            return Err(invalid_contract(format!(
                "{field} relation `{}` references unknown source_object_id `{}`",
                row.relation_id, row.source_object_id
            )));
        }
        if !object_ids.contains(row.target_object_id.as_str()) {
            return Err(invalid_contract(format!(
                "{field} relation `{}` references unknown target_object_id `{}`",
                row.relation_id, row.target_object_id
            )));
        }
    }
    Ok(())
}

fn read_review_ledger(
    episteme_root: &Path,
    raw_path: &str,
    field: &str,
) -> Result<ReviewLedgerDocument, EpistemeOntologyError> {
    let ledger_path = resolve_ontology_artifact_path(episteme_root, raw_path, field)?;
    let raw_toml = read_to_string(&ledger_path)?;
    let metadata = toml::from_str::<ReviewLedgerToml>(&raw_toml).map_err(|source| {
        invalid_contract(format!(
            "{field} `{raw_path}` is not valid review-ledger TOML: {source}"
        ))
    })?;
    validate_review_ledger_metadata(raw_path, &metadata, field)?;

    let ledger_org = metadata.ledger_org.as_deref().unwrap_or_default();
    let org_path = resolve_review_ledger_org_path(&ledger_path, ledger_org, raw_path, field)?;
    let org_content = read_to_string(&org_path)?;
    if let Some(expected_hash) = metadata.ledger_org_sha256.as_deref() {
        validate_review_ledger_hash(raw_path, &org_content, expected_hash, field)?;
    }

    let compiled =
        compile_org_ontology_authoring_document(&org_content, org_path.display().to_string())
            .map_err(|source| {
                invalid_contract(format!(
                    "{field} `{raw_path}` Org ledger cannot be compiled: {source}"
                ))
            })?;
    let domain_id = metadata.domain.as_deref().unwrap_or_default();
    let (object_rows, relation_rows) = extract_review_rows(raw_path, domain_id, &compiled, field)?;
    if (!object_rows.is_empty() || !relation_rows.is_empty())
        && (metadata.source_mutation_allowed.unwrap_or(false)
            || metadata.ontology_truth.unwrap_or(false)
            || metadata.promotion_allowed.unwrap_or(false))
    {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` object/relation review ledgers must not allow source mutation, direct ontology truth, or direct promotion"
        )));
    }

    Ok(ReviewLedgerDocument {
        path: raw_path.to_string(),
        object_rows,
        relation_rows,
    })
}

fn validate_review_ledger_metadata(
    raw_path: &str,
    metadata: &ReviewLedgerToml,
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    if metadata.schema_version != 1 {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` has unsupported schema_version {}",
            metadata.schema_version
        )));
    }
    require_nonblank_metadata(raw_path, metadata.ledger_id.as_deref(), "ledger_id", field)?;
    require_nonblank_metadata(raw_path, metadata.domain.as_deref(), "domain", field)?;
    require_nonblank_metadata(
        raw_path,
        metadata.ledger_org.as_deref(),
        "ledger_org",
        field,
    )?;
    if metadata.source_mutation_allowed.unwrap_or(false) {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` must set source_mutation_allowed=false"
        )));
    }
    if metadata.ontology_truth.unwrap_or(false) {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` must set ontology_truth=false"
        )));
    }
    if metadata.promotion_allowed.unwrap_or(false) {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` must set promotion_allowed=false"
        )));
    }
    Ok(())
}

fn require_nonblank_metadata(
    raw_path: &str,
    value: Option<&str>,
    key: &str,
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    if value.map(str::trim).unwrap_or_default().is_empty() {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` must declare nonblank {key}"
        )));
    }
    Ok(())
}

fn resolve_review_ledger_org_path(
    ledger_path: &Path,
    raw_org: &str,
    raw_path: &str,
    field: &str,
) -> Result<PathBuf, EpistemeOntologyError> {
    let trimmed = raw_org.trim();
    if trimmed.is_empty() {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` ledger_org must not be blank"
        )));
    }
    let org_path = Path::new(trimmed);
    if org_path.is_absolute()
        || org_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` ledger_org must be a safe path relative to the review ledger TOML directory: {trimmed}"
        )));
    }
    let parent = ledger_path.parent().ok_or_else(|| {
        invalid_contract(format!(
            "{field} `{raw_path}` has no parent directory for ledger_org resolution"
        ))
    })?;
    let resolved = parent.join(org_path);
    if !resolved.is_file() {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` ledger_org does not exist or is not a file: {trimmed}"
        )));
    }
    Ok(resolved)
}

fn validate_review_ledger_hash(
    raw_path: &str,
    org_content: &str,
    expected_hash: &str,
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    let actual_hash = format!("sha256:{}", hex_sha256(org_content.as_bytes()));
    if expected_hash.trim() != actual_hash {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` ledger_org_sha256 mismatch: expected {}, actual {actual_hash}",
            expected_hash.trim()
        )));
    }
    Ok(())
}

fn extract_review_rows(
    raw_path: &str,
    domain_id: &str,
    document: &OrgOntologyAuthoringDocument,
    field: &str,
) -> Result<(Vec<ObjectInstanceRow>, Vec<InstanceRelationRow>), EpistemeOntologyError> {
    let mut objects = Vec::new();
    let mut relations = Vec::new();
    for section in &document.sections {
        for table in &section.tables {
            match table.kind.as_str() {
                OBJECT_INSTANCE_REVIEW_TABLE => {
                    for (row_index, row) in table.rows.iter().enumerate() {
                        objects.push(object_instance_row(
                            raw_path, domain_id, table, row, row_index, field,
                        )?);
                    }
                }
                INSTANCE_RELATION_REVIEW_TABLE => {
                    for (row_index, row) in table.rows.iter().enumerate() {
                        relations.push(instance_relation_row(
                            raw_path, domain_id, table, row, row_index, field,
                        )?);
                    }
                }
                _ => {}
            }
        }
    }
    Ok((objects, relations))
}

fn object_instance_row(
    raw_path: &str,
    domain_id: &str,
    table: &OrgOntologyAuthoringTable,
    row_map: &BTreeMap<String, String>,
    row_index: usize,
    field: &str,
) -> Result<ObjectInstanceRow, EpistemeOntologyError> {
    let row = ObjectInstanceRow {
        domain_id: domain_id.to_string(),
        object_id: required_row_value(raw_path, table, row_map, row_index, "object_id", field)?,
        object_type: required_row_value(raw_path, table, row_map, row_index, "object_type", field)?,
        label: required_row_value(raw_path, table, row_map, row_index, "label", field)?,
        evidence_id: required_row_value(raw_path, table, row_map, row_index, "evidence_id", field)?,
        review_decision: required_row_value(
            raw_path,
            table,
            row_map,
            row_index,
            "review_decision",
            field,
        )?,
        promotion_decision: required_row_value(
            raw_path,
            table,
            row_map,
            row_index,
            "promotion_decision",
            field,
        )?,
        reviewer_id: required_row_value(raw_path, table, row_map, row_index, "reviewer_id", field)?,
    };
    validate_approved_reviewer(
        raw_path,
        table,
        row_index,
        &row.promotion_decision,
        &row.reviewer_id,
        field,
    )?;
    Ok(row)
}

fn instance_relation_row(
    raw_path: &str,
    domain_id: &str,
    table: &OrgOntologyAuthoringTable,
    row_map: &BTreeMap<String, String>,
    row_index: usize,
    field: &str,
) -> Result<InstanceRelationRow, EpistemeOntologyError> {
    let row = InstanceRelationRow {
        domain_id: domain_id.to_string(),
        relation_id: required_row_value(raw_path, table, row_map, row_index, "relation_id", field)?,
        source_object_id: required_row_value(
            raw_path,
            table,
            row_map,
            row_index,
            "source_object_id",
            field,
        )?,
        target_object_id: required_row_value(
            raw_path,
            table,
            row_map,
            row_index,
            "target_object_id",
            field,
        )?,
        predicate: required_row_value(raw_path, table, row_map, row_index, "predicate", field)?,
        evidence_id: required_row_value(raw_path, table, row_map, row_index, "evidence_id", field)?,
        review_decision: required_row_value(
            raw_path,
            table,
            row_map,
            row_index,
            "review_decision",
            field,
        )?,
        promotion_decision: required_row_value(
            raw_path,
            table,
            row_map,
            row_index,
            "promotion_decision",
            field,
        )?,
        reviewer_id: required_row_value(raw_path, table, row_map, row_index, "reviewer_id", field)?,
    };
    validate_approved_reviewer(
        raw_path,
        table,
        row_index,
        &row.promotion_decision,
        &row.reviewer_id,
        field,
    )?;
    Ok(row)
}

fn required_row_value(
    raw_path: &str,
    table: &OrgOntologyAuthoringTable,
    row: &BTreeMap<String, String>,
    row_index: usize,
    key: &str,
    field: &str,
) -> Result<String, EpistemeOntologyError> {
    let value = row
        .iter()
        .find(|(candidate, _)| normalize_field(candidate) == key)
        .map(|(_, value)| value.trim().to_string())
        .unwrap_or_default();
    if value.is_empty() {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` table `{}` row {} must declare nonblank {key}",
            table.kind.as_str(),
            row_index + 1
        )));
    }
    Ok(value)
}

fn validate_approved_reviewer(
    raw_path: &str,
    table: &OrgOntologyAuthoringTable,
    row_index: usize,
    promotion_decision: &str,
    reviewer_id: &str,
    field: &str,
) -> Result<(), EpistemeOntologyError> {
    if normalize_field(promotion_decision) == APPROVED_PROMOTION_DECISION
        && reviewer_id.trim().is_empty()
    {
        return Err(invalid_contract(format!(
            "{field} `{raw_path}` table `{}` row {} approved promotions must declare reviewer_id",
            table.kind.as_str(),
            row_index + 1
        )));
    }
    Ok(())
}

fn normalize_field(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(HEX_LOWER[(byte >> 4) as usize] as char);
        output.push(HEX_LOWER[(byte & 0x0f) as usize] as char);
    }
    output
}
