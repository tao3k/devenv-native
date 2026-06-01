//! Source-patch preflight receipts for approved ontology review-ledger rows.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::{
    manifest::{EpistemeOntologyDomain, read_ontology_manifest, validate_ontology_contract},
    review_ledger::{
        InstanceRelationRow, ObjectInstanceRow, ReviewLedgerSet, read_review_ledger_set,
    },
};

const SOURCE_PATCH_PREFLIGHT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_preflight.v1";
const SOURCE_PATCH_PREFLIGHT_TSV: &str = "source_patch_preflight.tsv";
const SOURCE_PATCH_PREFLIGHT_ORG: &str = "source_patch_preflight.org";
const SOURCE_PATCH_PREFLIGHT_JSON: &str = "source_patch_preflight.json";
const PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH: &str = "propose_source_patch";
const APPROVED_PROMOTION_DECISION: &str = "approved";
const SOURCE_MUTATION_ALLOWED: bool = false;
const ONTOLOGY_TRUTH: bool = false;

/// Request for writing a source-patch preflight receipt from review ledgers.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchPreflightRequest {
    episteme_root: PathBuf,
    run_dir: PathBuf,
}

impl EpistemeOntologySourcePatchPreflightRequest {
    /// Create a source-patch preflight request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_dir: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_dir: run_dir.into(),
        }
    }

    /// Episteme repository root containing `ontology/manifest.toml`.
    #[must_use]
    pub fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    /// Output run directory for preflight artifacts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after source-patch preflight generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchPreflightReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Episteme repository root that was inspected.
    pub episteme_root: PathBuf,
    /// Output run directory.
    pub run_dir: PathBuf,
    /// Generated preflight TSV path.
    pub source_patch_preflight_tsv: PathBuf,
    /// Generated preflight Org path.
    pub source_patch_preflight_org: PathBuf,
    /// Generated preflight JSON path.
    pub source_patch_preflight_json: PathBuf,
    /// Number of object-instance review rows read.
    pub object_review_row_count: usize,
    /// Number of relation review rows read.
    pub relation_review_row_count: usize,
    /// Number of explicitly approved object-instance rows.
    pub approved_object_count: usize,
    /// Number of explicitly approved relation rows.
    pub approved_relation_count: usize,
    /// Number of source-patch preflight rows written.
    pub preflight_row_count: usize,
    /// Whether this preflight authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug)]
struct SourcePatchPreflightRow {
    record_id: String,
    record_kind: &'static str,
    domain_id: String,
    target_rdf_file: String,
    label: String,
    object_type: String,
    source_object_id: String,
    predicate: String,
    target_object_id: String,
    evidence_id: String,
    review_decision: String,
    promotion_decision: String,
    reviewer_id: String,
    preflight_action: &'static str,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

/// Write a non-mutating source-patch preflight receipt from approved review rows.
///
/// # Errors
///
/// Returns an error when the Episteme ontology contract is invalid, declared
/// review ledgers cannot be compiled, or an approved relation references an
/// endpoint that is not also explicitly approved as an object instance.
pub fn write_episteme_ontology_source_patch_preflight(
    request: &EpistemeOntologySourcePatchPreflightRequest,
) -> Result<EpistemeOntologySourcePatchPreflightReport> {
    validate_ontology_contract(request.episteme_root())
        .context("source patch preflight requires a valid ontology source contract")?;
    let manifest = read_ontology_manifest(request.episteme_root())
        .context("failed to read ontology manifest for source patch preflight")?;
    let ledger_set = manifest.domains.iter().try_fold(
        ReviewLedgerSet::default(),
        |mut ledger_set, domain| {
            let domain_set = read_review_ledger_set(
                request.episteme_root(),
                &domain.review_ledgers,
                "review_ledgers",
            )?;
            ledger_set.object_rows.extend(domain_set.object_rows);
            ledger_set.relation_rows.extend(domain_set.relation_rows);
            Ok::<_, anyhow::Error>(ledger_set)
        },
    )?;
    let source_patch_targets = source_patch_targets(&manifest.domains);
    let rows = build_preflight_rows(&ledger_set, &source_patch_targets)?;
    let source_patch_preflight_tsv = request.run_dir().join(SOURCE_PATCH_PREFLIGHT_TSV);
    let source_patch_preflight_org = request.run_dir().join(SOURCE_PATCH_PREFLIGHT_ORG);
    let source_patch_preflight_json = request.run_dir().join(SOURCE_PATCH_PREFLIGHT_JSON);
    write_preflight_tsv(source_patch_preflight_tsv.as_path(), &rows)?;

    let report = EpistemeOntologySourcePatchPreflightReport {
        schema_version: SOURCE_PATCH_PREFLIGHT_SCHEMA_VERSION,
        episteme_root: request.episteme_root().to_path_buf(),
        run_dir: request.run_dir().to_path_buf(),
        source_patch_preflight_tsv,
        source_patch_preflight_org,
        source_patch_preflight_json,
        object_review_row_count: ledger_set.object_rows.len(),
        relation_review_row_count: ledger_set.relation_rows.len(),
        approved_object_count: ledger_set
            .object_rows
            .iter()
            .filter(|row| is_approved(row.promotion_decision.as_str()))
            .count(),
        approved_relation_count: ledger_set
            .relation_rows
            .iter()
            .filter(|row| is_approved(row.promotion_decision.as_str()))
            .count(),
        preflight_row_count: rows.len(),
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    };
    write_preflight_org(report.source_patch_preflight_org.as_path(), &report)?;
    write_json(report.source_patch_preflight_json.as_path(), &report)?;
    Ok(report)
}

fn build_preflight_rows(
    ledger_set: &ReviewLedgerSet,
    source_patch_targets: &BTreeMap<String, String>,
) -> Result<Vec<SourcePatchPreflightRow>> {
    let approved_object_ids = ledger_set
        .object_rows
        .iter()
        .filter(|row| is_approved(row.promotion_decision.as_str()))
        .map(|row| row.object_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut rows = ledger_set
        .object_rows
        .iter()
        .filter(|row| is_approved(row.promotion_decision.as_str()))
        .map(|row| object_preflight_row(row, source_patch_targets))
        .collect::<Result<Vec<_>>>()?;
    for relation in ledger_set
        .relation_rows
        .iter()
        .filter(|row| is_approved(row.promotion_decision.as_str()))
    {
        if !approved_object_ids.contains(relation.source_object_id.as_str()) {
            anyhow::bail!(
                "approved relation `{}` references source_object_id `{}` without an approved object-instance review row",
                relation.relation_id,
                relation.source_object_id
            );
        }
        if !approved_object_ids.contains(relation.target_object_id.as_str()) {
            anyhow::bail!(
                "approved relation `{}` references target_object_id `{}` without an approved object-instance review row",
                relation.relation_id,
                relation.target_object_id
            );
        }
        rows.push(relation_preflight_row(relation, source_patch_targets)?);
    }
    Ok(rows)
}

fn object_preflight_row(
    row: &ObjectInstanceRow,
    source_patch_targets: &BTreeMap<String, String>,
) -> Result<SourcePatchPreflightRow> {
    let target_rdf_file = source_patch_target(row.domain_id.as_str(), source_patch_targets)?;
    Ok(SourcePatchPreflightRow {
        record_id: row.object_id.clone(),
        record_kind: "object_instance",
        domain_id: row.domain_id.clone(),
        target_rdf_file: target_rdf_file.to_string(),
        label: row.label.clone(),
        object_type: row.object_type.clone(),
        source_object_id: String::new(),
        predicate: String::new(),
        target_object_id: String::new(),
        evidence_id: row.evidence_id.clone(),
        review_decision: row.review_decision.clone(),
        promotion_decision: row.promotion_decision.clone(),
        reviewer_id: row.reviewer_id.clone(),
        preflight_action: PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH,
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    })
}

fn relation_preflight_row(
    row: &InstanceRelationRow,
    source_patch_targets: &BTreeMap<String, String>,
) -> Result<SourcePatchPreflightRow> {
    let target_rdf_file = source_patch_target(row.domain_id.as_str(), source_patch_targets)?;
    Ok(SourcePatchPreflightRow {
        record_id: row.relation_id.clone(),
        record_kind: "instance_relation",
        domain_id: row.domain_id.clone(),
        target_rdf_file: target_rdf_file.to_string(),
        label: String::new(),
        object_type: String::new(),
        source_object_id: row.source_object_id.clone(),
        predicate: row.predicate.clone(),
        target_object_id: row.target_object_id.clone(),
        evidence_id: row.evidence_id.clone(),
        review_decision: row.review_decision.clone(),
        promotion_decision: row.promotion_decision.clone(),
        reviewer_id: row.reviewer_id.clone(),
        preflight_action: PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH,
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    })
}

fn source_patch_targets(domains: &[EpistemeOntologyDomain]) -> BTreeMap<String, String> {
    domains
        .iter()
        .filter_map(|domain| {
            let [target] = domain.rdf_files.as_slice() else {
                return None;
            };
            Some((domain.id.clone(), target.clone()))
        })
        .collect()
}

fn source_patch_target<'a>(
    domain_id: &str,
    source_patch_targets: &'a BTreeMap<String, String>,
) -> Result<&'a str> {
    source_patch_targets
        .get(domain_id)
        .map(String::as_str)
        .with_context(|| {
            format!(
                "approved source-patch row domain `{domain_id}` must declare exactly one RDF source target"
            )
        })
}

fn write_preflight_tsv(path: &Path, rows: &[SourcePatchPreflightRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "record_id\trecord_kind\tdomain_id\ttarget_rdf_file\tlabel\tobject_type\tsource_object_id\tpredicate\ttarget_object_id\tevidence_id\treview_decision\tpromotion_decision\treviewer_id\tpreflight_action\tsource_mutation_allowed\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.record_id),
            row.record_kind,
            escape_tsv(&row.domain_id),
            escape_tsv(&row.target_rdf_file),
            escape_tsv(&row.label),
            escape_tsv(&row.object_type),
            escape_tsv(&row.source_object_id),
            escape_tsv(&row.predicate),
            escape_tsv(&row.target_object_id),
            escape_tsv(&row.evidence_id),
            escape_tsv(&row.review_decision),
            escape_tsv(&row.promotion_decision),
            escape_tsv(&row.reviewer_id),
            row.preflight_action,
            row.source_mutation_allowed,
            row.ontology_truth
        )?;
    }
    Ok(())
}

fn write_preflight_org(
    path: &Path,
    report: &EpistemeOntologySourcePatchPreflightReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Source Patch Preflight")?;
    writeln!(file)?;
    writeln!(file, "* Source patch preflight")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_preflight")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This preflight verifies approved review-ledger rows before any source RDF patch is generated."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| object_review_row_count | {} |",
        report.object_review_row_count
    )?;
    writeln!(
        file,
        "| relation_review_row_count | {} |",
        report.relation_review_row_count
    )?;
    writeln!(
        file,
        "| approved_object_count | {} |",
        report.approved_object_count
    )?;
    writeln!(
        file,
        "| approved_relation_count | {} |",
        report.approved_relation_count
    )?;
    writeln!(
        file,
        "| preflight_row_count | {} |",
        report.preflight_row_count
    )?;
    writeln!(
        file,
        "| source_mutation_allowed | {} |",
        report.source_mutation_allowed
    )?;
    writeln!(file, "| ontology_truth | {} |", report.ontology_truth)?;
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologySourcePatchPreflightReport) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, report)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn is_approved(value: &str) -> bool {
    normalize(value) == APPROVED_PROMOTION_DECISION
}

fn normalize(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
