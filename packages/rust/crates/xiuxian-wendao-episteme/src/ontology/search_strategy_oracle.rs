//! `SearchStrategyFlow` oracle projection from Episteme review ledgers.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Serialize;

use super::{
    manifest::read_ontology_manifest,
    review_ledger::{
        InstanceRelationRow, ObjectInstanceRow, read_review_ledger_set, validate_review_ledger_set,
    },
};

const ORACLE_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_search_strategy_oracle.v1";
const REPORT_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_search_strategy_oracle_report.v1";
const PROJECTION_SOURCE: &str = "xiuxian-wendao-episteme.review-ledger-oracle";
const CASES_JSON: &str = "search_strategy_oracle_cases.json";
const CANDIDATES_JSON: &str = "search_strategy_oracle_candidates.json";
const REPORT_JSON: &str = "search_strategy_oracle_report.json";
const APPROVED_PROMOTION_DECISION: &str = "approved";

/// Request for compiling `SearchStrategyFlow` oracle facts from an Episteme root.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeSearchStrategyOracleRequest {
    episteme_root: PathBuf,
    run_id: String,
}

impl EpistemeSearchStrategyOracleRequest {
    /// Create a search-oracle projection request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_id: run_id.into(),
        }
    }

    /// Episteme repository root.
    #[must_use]
    pub fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    /// Safe run id used for output directories.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }
}

/// Report emitted after writing `SearchStrategyFlow` oracle artifacts.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeSearchStrategyOracleReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Run id used for artifact paths.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Search oracle cases JSON path.
    pub cases_json: PathBuf,
    /// Search oracle candidates JSON path.
    pub candidates_json: PathBuf,
    /// Report JSON path.
    pub report_json: PathBuf,
    /// Number of compiled cases.
    pub case_count: usize,
    /// Number of compiled candidate rows.
    pub candidate_count: usize,
    /// Number of expected selected ids across cases.
    pub expected_selected_count: usize,
    /// Number of expected rejected ids across cases.
    pub expected_rejected_count: usize,
    /// Number of review ledger paths consumed from the manifest.
    pub review_ledger_count: usize,
    /// Required evidence labels carried by every domain case.
    pub required_evidence_labels: Vec<String>,
    /// Raw/private source rows are not ontology truth.
    pub ontology_truth: bool,
    /// Projection never mutates source files.
    pub source_mutation_allowed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchStrategyOracleCaseFile {
    schema_version: &'static str,
    run_id: String,
    projection_source: &'static str,
    source_mutation_allowed: bool,
    ontology_truth: bool,
    cases: Vec<SearchStrategyOracleCaseRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchStrategyOracleCaseRow {
    case_id: String,
    domain_id: String,
    intent: String,
    expected_selected_candidate_ids: Vec<String>,
    expected_rejected_candidate_ids: Vec<String>,
    required_evidence_labels: Vec<String>,
    source_review_ledgers: Vec<String>,
    promotion_status: String,
    ontology_truth: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchStrategyOracleCandidateFile {
    schema_version: &'static str,
    run_id: String,
    projection_source: &'static str,
    source_mutation_allowed: bool,
    ontology_truth: bool,
    candidates: Vec<SearchStrategyOracleCandidateRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchStrategyOracleCandidateRow {
    candidate_id: String,
    revision_id: String,
    domain_id: String,
    candidate_kind: String,
    label: String,
    object_type: String,
    predicate: String,
    source_object_id: String,
    target_object_id: String,
    evidence_id: String,
    review_decision: String,
    promotion_decision: String,
    reviewer_id: String,
    expected_action: String,
    action: String,
    blocked: bool,
    route_role: String,
    required_evidence: String,
    final_score: f64,
    context_cost: usize,
    ontology_truth: bool,
}

/// Compile `SearchStrategyFlow` oracle facts from manifest-declared Org review ledgers.
///
/// # Errors
///
/// Returns an error when the Episteme manifest or review ledgers are invalid,
/// no review rows are available, or output artifacts cannot be written.
pub fn write_episteme_search_strategy_oracle(
    request: &EpistemeSearchStrategyOracleRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeSearchStrategyOracleReport> {
    write_episteme_search_strategy_oracle_impl(request, run_root)
}

fn write_episteme_search_strategy_oracle_impl(
    request: &EpistemeSearchStrategyOracleRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeSearchStrategyOracleReport> {
    validate_run_id(request.run_id())?;
    let manifest = read_ontology_manifest(request.episteme_root())
        .context("failed to read ontology manifest for search oracle projection")?;
    let review_ledger_paths = manifest
        .domains
        .iter()
        .flat_map(|domain| domain.review_ledgers.iter().cloned())
        .collect::<Vec<_>>();
    if review_ledger_paths.is_empty() {
        bail!("search oracle projection requires at least one review ledger");
    }
    let ledger_set = read_review_ledger_set(
        request.episteme_root(),
        review_ledger_paths.as_slice(),
        "review_ledgers",
    )
    .context("failed to read review ledgers for search oracle projection")?;
    validate_review_ledger_set(&ledger_set, "review_ledgers")
        .context("failed to validate review ledgers for search oracle projection")?;
    if ledger_set.object_rows.is_empty() && ledger_set.relation_rows.is_empty() {
        bail!("search oracle projection requires object or relation review rows");
    }

    let candidates = candidate_rows(&ledger_set.object_rows, &ledger_set.relation_rows);
    let cases = case_rows(&review_ledger_paths, &candidates);
    if cases.is_empty() {
        bail!("search oracle projection did not produce any cases");
    }

    let run_dir = run_root.as_ref().join(request.run_id());
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create `{}`", run_dir.display()))?;
    let cases_json = run_dir.join(CASES_JSON);
    let candidates_json = run_dir.join(CANDIDATES_JSON);
    let report_json = run_dir.join(REPORT_JSON);
    let case_file = SearchStrategyOracleCaseFile {
        schema_version: ORACLE_SCHEMA_VERSION,
        run_id: request.run_id().to_string(),
        projection_source: PROJECTION_SOURCE,
        source_mutation_allowed: false,
        ontology_truth: false,
        cases: cases.clone(),
    };
    let candidate_file = SearchStrategyOracleCandidateFile {
        schema_version: ORACLE_SCHEMA_VERSION,
        run_id: request.run_id().to_string(),
        projection_source: PROJECTION_SOURCE,
        source_mutation_allowed: false,
        ontology_truth: false,
        candidates: candidates.clone(),
    };
    write_json(cases_json.as_path(), &case_file)?;
    write_json(candidates_json.as_path(), &candidate_file)?;

    let required_evidence_labels = required_evidence_labels();
    let report = EpistemeSearchStrategyOracleReport {
        schema_version: REPORT_SCHEMA_VERSION,
        run_id: request.run_id().to_string(),
        run_dir,
        cases_json,
        candidates_json,
        report_json,
        case_count: cases.len(),
        candidate_count: candidates.len(),
        expected_selected_count: cases
            .iter()
            .map(|case| case.expected_selected_candidate_ids.len())
            .sum(),
        expected_rejected_count: cases
            .iter()
            .map(|case| case.expected_rejected_candidate_ids.len())
            .sum(),
        review_ledger_count: review_ledger_paths.len(),
        required_evidence_labels,
        ontology_truth: false,
        source_mutation_allowed: false,
    };
    write_json(report.report_json.as_path(), &report)?;
    Ok(report)
}

fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.trim().is_empty() {
        bail!("search oracle run_id must not be blank");
    }
    if !run_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        bail!("search oracle run_id must be ASCII alphanumeric plus `_` or `-`: {run_id}");
    }
    Ok(())
}

fn candidate_rows(
    objects: &[ObjectInstanceRow],
    relations: &[InstanceRelationRow],
) -> Vec<SearchStrategyOracleCandidateRow> {
    let mut rows = objects
        .iter()
        .map(object_candidate_row)
        .chain(relations.iter().map(relation_candidate_row))
        .collect::<Vec<_>>();
    let domains = rows
        .iter()
        .map(|row| row.domain_id.clone())
        .collect::<BTreeSet<_>>();
    for domain_id in domains {
        append_route_support_candidates(domain_id.as_str(), &mut rows);
    }
    rows.sort_by(|left, right| {
        left.domain_id
            .cmp(&right.domain_id)
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });
    rows
}

fn object_candidate_row(row: &ObjectInstanceRow) -> SearchStrategyOracleCandidateRow {
    SearchStrategyOracleCandidateRow {
        candidate_id: row.object_id.clone(),
        revision_id: "episteme-review-ledger".to_string(),
        domain_id: row.domain_id.clone(),
        candidate_kind: "object_instance".to_string(),
        label: row.label.clone(),
        object_type: row.object_type.clone(),
        predicate: String::new(),
        source_object_id: String::new(),
        target_object_id: String::new(),
        evidence_id: row.evidence_id.clone(),
        review_decision: row.review_decision.clone(),
        promotion_decision: row.promotion_decision.clone(),
        reviewer_id: row.reviewer_id.clone(),
        expected_action: expected_action(&row.promotion_decision).to_string(),
        action: action(&row.promotion_decision).to_string(),
        blocked: !is_approved(&row.promotion_decision),
        route_role: "authority".to_string(),
        required_evidence: "ownership_boundary".to_string(),
        final_score: if is_approved(&row.promotion_decision) {
            0.82
        } else {
            0.64
        },
        context_cost: 80,
        ontology_truth: false,
    }
}

fn relation_candidate_row(row: &InstanceRelationRow) -> SearchStrategyOracleCandidateRow {
    SearchStrategyOracleCandidateRow {
        candidate_id: row.relation_id.clone(),
        revision_id: "episteme-review-ledger".to_string(),
        domain_id: row.domain_id.clone(),
        candidate_kind: "instance_relation".to_string(),
        label: row.relation_id.clone(),
        object_type: String::new(),
        predicate: row.predicate.clone(),
        source_object_id: row.source_object_id.clone(),
        target_object_id: row.target_object_id.clone(),
        evidence_id: row.evidence_id.clone(),
        review_decision: row.review_decision.clone(),
        promotion_decision: row.promotion_decision.clone(),
        reviewer_id: row.reviewer_id.clone(),
        expected_action: expected_action(&row.promotion_decision).to_string(),
        action: action(&row.promotion_decision).to_string(),
        blocked: !is_approved(&row.promotion_decision),
        route_role: "link_graph".to_string(),
        required_evidence: "relation_path".to_string(),
        final_score: if is_approved(&row.promotion_decision) {
            0.84
        } else {
            0.99
        },
        context_cost: 80,
        ontology_truth: false,
    }
}

fn append_route_support_candidates(
    domain_id: &str,
    rows: &mut Vec<SearchStrategyOracleCandidateRow>,
) {
    let domain_suffix = stable_case_suffix(domain_id);
    for (required_evidence, route_role, label, score) in [
        (
            "ownership_boundary",
            "authority",
            "Episteme ownership and Org review authority",
            0.96,
        ),
        (
            "validation_path",
            "validation",
            "Episteme review-ledger validation path",
            0.95,
        ),
        (
            "relation_path",
            "link_graph",
            "Episteme reviewed relation evidence path",
            0.94,
        ),
        (
            "page_index_seed",
            "page_index",
            "Episteme source-ledger page and span seed",
            0.93,
        ),
    ] {
        rows.push(SearchStrategyOracleCandidateRow {
            candidate_id: format!("episteme.oracle.{domain_suffix}.{required_evidence}"),
            revision_id: "episteme-search-oracle-route-support".to_string(),
            domain_id: domain_id.to_string(),
            candidate_kind: "oracle_route_support".to_string(),
            label: label.to_string(),
            object_type: String::new(),
            predicate: String::new(),
            source_object_id: String::new(),
            target_object_id: String::new(),
            evidence_id: format!("episteme.oracle.evidence.{domain_suffix}.{required_evidence}"),
            review_decision: "derived_from_review_ledger".to_string(),
            promotion_decision: "approved".to_string(),
            reviewer_id: "system.episteme.search_oracle".to_string(),
            expected_action: "select".to_string(),
            action: "keep".to_string(),
            blocked: false,
            route_role: route_role.to_string(),
            required_evidence: required_evidence.to_string(),
            final_score: score,
            context_cost: 60,
            ontology_truth: false,
        });
    }
}

fn case_rows(
    review_ledger_paths: &[String],
    candidates: &[SearchStrategyOracleCandidateRow],
) -> Vec<SearchStrategyOracleCaseRow> {
    let mut by_domain = BTreeMap::<String, Vec<&SearchStrategyOracleCandidateRow>>::new();
    for candidate in candidates {
        by_domain
            .entry(candidate.domain_id.clone())
            .or_default()
            .push(candidate);
    }
    by_domain
        .into_iter()
        .map(|(domain_id, rows)| {
            let selected = rows
                .iter()
                .filter(|row| !row.blocked)
                .map(|row| row.candidate_id.clone())
                .collect::<Vec<_>>();
            let rejected = rows
                .iter()
                .filter(|row| row.blocked)
                .map(|row| row.candidate_id.clone())
                .collect::<Vec<_>>();
            SearchStrategyOracleCaseRow {
                case_id: format!("episteme-domain-{}", stable_case_suffix(&domain_id)),
                intent: format!(
                    "Find reviewed ontology evidence for {domain_id} while preserving approved promotions and rejecting pending or blocked candidates."
                ),
                domain_id,
                expected_selected_candidate_ids: selected,
                expected_rejected_candidate_ids: rejected,
                required_evidence_labels: required_evidence_labels(),
                source_review_ledgers: review_ledger_paths.to_vec(),
                promotion_status: "review-ledger-derived".to_string(),
                ontology_truth: false,
            }
        })
        .collect()
}

fn stable_case_suffix(domain_id: &str) -> String {
    domain_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn required_evidence_labels() -> Vec<String> {
    [
        "ownership_boundary",
        "validation_path",
        "relation_path",
        "page_index_seed",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn expected_action(promotion_decision: &str) -> &'static str {
    if is_approved(promotion_decision) {
        "select"
    } else {
        "reject"
    }
}

fn action(promotion_decision: &str) -> &'static str {
    if is_approved(promotion_decision) {
        "keep"
    } else {
        "prune"
    }
}

fn is_approved(promotion_decision: &str) -> bool {
    normalize_token(promotion_decision) == APPROVED_PROMOTION_DECISION
}

fn normalize_token(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    let mut file =
        File::create(path).with_context(|| format!("failed to create `{}`", path.display()))?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file).with_context(|| format!("failed to finish `{}`", path.display()))
}
