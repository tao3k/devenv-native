//! Non-mutating RDF draft export for approved source-patch preflight rows.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs::{self, File},
    io::{BufRead, BufReader, Write as _},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SOURCE_PATCH_DRAFT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_draft.v1";
const SOURCE_PATCH_PREFLIGHT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_preflight.v1";
const SOURCE_PATCH_PREFLIGHT_TSV: &str = "source_patch_preflight.tsv";
const SOURCE_PATCH_PREFLIGHT_JSON: &str = "source_patch_preflight.json";
const SOURCE_PATCH_DRAFT_TTL: &str = "source_patch_draft.ttl";
const SOURCE_PATCH_DRAFT_ORG: &str = "source_patch_draft.org";
const SOURCE_PATCH_DRAFT_JSON: &str = "source_patch_draft.json";
const PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH: &str = "propose_source_patch";
const APPROVED_PROMOTION_DECISION: &str = "approved";
const OBJECT_INSTANCE_KIND: &str = "object_instance";
const INSTANCE_RELATION_KIND: &str = "instance_relation";
const SOURCE_MUTATION_ALLOWED: bool = false;
const ONTOLOGY_TRUTH: bool = false;
const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";
const RDF_PREFIXES: &str = concat!(
    "@prefix wdsp: <https://wendao.ai/ontology/source-patch/> .\n",
    "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n",
    "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n"
);

/// Request for exporting a non-mutating RDF draft from preflight rows.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchDraftRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologySourcePatchDraftRequest {
    /// Create a source-patch draft request from a source-patch preflight run.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Source-patch preflight run directory containing the preflight receipt.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after source-patch draft export.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchDraftReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Source-patch preflight run directory.
    pub run_dir: PathBuf,
    /// Source preflight TSV path.
    pub source_patch_preflight_tsv: PathBuf,
    /// Generated source-patch RDF draft path.
    pub source_patch_draft_ttl: PathBuf,
    /// Generated source-patch draft Org receipt path.
    pub source_patch_draft_org: PathBuf,
    /// Generated source-patch draft JSON path.
    pub source_patch_draft_json: PathBuf,
    /// Number of preflight rows read.
    pub preflight_row_count: usize,
    /// Number of object-instance patch rows written.
    pub object_patch_count: usize,
    /// Number of instance-relation patch rows written.
    pub relation_patch_count: usize,
    /// Number of RDF resources written.
    pub draft_resource_count: usize,
    /// Number of RDF statements written.
    pub draft_statement_count: usize,
    /// Whether this draft authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePatchPreflightReceipt {
    schema_version: String,
    preflight_row_count: usize,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct SourcePatchPreflightRecord {
    record_id: String,
    record_kind: String,
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
    preflight_action: String,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

struct RenderedResource {
    text: String,
    statement_count: usize,
}

/// Export a non-mutating RDF source-patch draft from approved preflight rows.
///
/// # Errors
///
/// Returns an error when the preflight receipt is missing, inconsistent with
/// the TSV rows, attempts to authorize mutation or ontology truth, contains
/// non-approved rows, or has relation endpoints absent from object rows.
pub fn export_episteme_ontology_source_patch_draft(
    request: &EpistemeOntologySourcePatchDraftRequest,
) -> Result<EpistemeOntologySourcePatchDraftReport> {
    let run_dir = request.run_dir();
    let source_patch_preflight_tsv = run_dir.join(SOURCE_PATCH_PREFLIGHT_TSV);
    let receipt = read_preflight_receipt(run_dir.join(SOURCE_PATCH_PREFLIGHT_JSON).as_path())?;
    let rows = read_preflight_rows(source_patch_preflight_tsv.as_path())?;
    validate_preflight_receipt(&receipt, rows.len())?;
    validate_preflight_rows(&rows)?;

    let render = render_source_patch_draft(&rows);
    let source_patch_draft_ttl = run_dir.join(SOURCE_PATCH_DRAFT_TTL);
    let source_patch_draft_org = run_dir.join(SOURCE_PATCH_DRAFT_ORG);
    let source_patch_draft_json = run_dir.join(SOURCE_PATCH_DRAFT_JSON);
    write_string(source_patch_draft_ttl.as_path(), render.0.as_str())?;

    let report = EpistemeOntologySourcePatchDraftReport {
        schema_version: SOURCE_PATCH_DRAFT_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        source_patch_preflight_tsv,
        source_patch_draft_ttl,
        source_patch_draft_org,
        source_patch_draft_json,
        preflight_row_count: rows.len(),
        object_patch_count: rows
            .iter()
            .filter(|row| row.record_kind == OBJECT_INSTANCE_KIND)
            .count(),
        relation_patch_count: rows
            .iter()
            .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
            .count(),
        draft_resource_count: render.1,
        draft_statement_count: render.2,
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    };
    write_draft_org(report.source_patch_draft_org.as_path(), &report)?;
    write_json(report.source_patch_draft_json.as_path(), &report)?;
    Ok(report)
}

fn read_preflight_receipt(path: &Path) -> Result<SourcePatchPreflightReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse source-patch preflight JSON `{}`",
            path.display()
        )
    })
}

fn validate_preflight_receipt(
    receipt: &SourcePatchPreflightReceipt,
    row_count: usize,
) -> Result<()> {
    if receipt.schema_version != SOURCE_PATCH_PREFLIGHT_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch preflight receipt has unsupported schemaVersion `{}`",
            receipt.schema_version
        );
    }
    if receipt.source_mutation_allowed {
        anyhow::bail!("source-patch preflight receipt attempted to authorize source mutation");
    }
    if receipt.ontology_truth {
        anyhow::bail!("source-patch preflight receipt attempted to mark ontology truth");
    }
    if receipt.preflight_row_count != row_count {
        anyhow::bail!(
            "source-patch preflight row count mismatch: receipt has {}, TSV has {row_count}",
            receipt.preflight_row_count
        );
    }
    Ok(())
}

fn read_preflight_rows(path: &Path) -> Result<Vec<SourcePatchPreflightRecord>> {
    let file = File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()
        .with_context(|| format!("failed to read `{}`", path.display()))?
        .with_context(|| format!("source-patch preflight TSV `{}` is empty", path.display()))?;
    let columns = header.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line = line.with_context(|| format!("failed to read `{}`", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let values = line.split('\t').map(unescape_tsv).collect::<Vec<_>>();
        if values.len() != columns.len() {
            anyhow::bail!(
                "source-patch preflight TSV `{}` row {} has {} values for {} columns",
                path.display(),
                line_index + 2,
                values.len(),
                columns.len()
            );
        }
        let row = columns
            .iter()
            .cloned()
            .zip(values)
            .collect::<BTreeMap<_, _>>();
        rows.push(preflight_record(path, line_index + 2, &row)?);
    }
    Ok(rows)
}

fn preflight_record(
    path: &Path,
    row_number: usize,
    row: &BTreeMap<String, String>,
) -> Result<SourcePatchPreflightRecord> {
    Ok(SourcePatchPreflightRecord {
        record_id: required(row, "record_id", path, row_number)?,
        record_kind: required(row, "record_kind", path, row_number)?,
        domain_id: required(row, "domain_id", path, row_number)?,
        target_rdf_file: required(row, "target_rdf_file", path, row_number)?,
        label: optional(row, "label"),
        object_type: optional(row, "object_type"),
        source_object_id: optional(row, "source_object_id"),
        predicate: optional(row, "predicate"),
        target_object_id: optional(row, "target_object_id"),
        evidence_id: required(row, "evidence_id", path, row_number)?,
        review_decision: required(row, "review_decision", path, row_number)?,
        promotion_decision: required(row, "promotion_decision", path, row_number)?,
        reviewer_id: required(row, "reviewer_id", path, row_number)?,
        preflight_action: required(row, "preflight_action", path, row_number)?,
        source_mutation_allowed: parse_bool(row, "source_mutation_allowed", path, row_number)?,
        ontology_truth: parse_bool(row, "ontology_truth", path, row_number)?,
    })
}

fn validate_preflight_rows(rows: &[SourcePatchPreflightRecord]) -> Result<()> {
    let mut object_ids = BTreeSet::new();
    for row in rows {
        if row.preflight_action != PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH {
            anyhow::bail!(
                "source-patch preflight row `{}` has unsupported preflight_action `{}`",
                row.record_id,
                row.preflight_action
            );
        }
        if normalize(row.promotion_decision.as_str()) != APPROVED_PROMOTION_DECISION {
            anyhow::bail!(
                "source-patch preflight row `{}` is not explicitly approved",
                row.record_id
            );
        }
        if row.source_mutation_allowed {
            anyhow::bail!(
                "source-patch preflight row `{}` attempted to authorize source mutation",
                row.record_id
            );
        }
        if row.ontology_truth {
            anyhow::bail!(
                "source-patch preflight row `{}` attempted to mark ontology truth",
                row.record_id
            );
        }
        require_nonblank(row.domain_id.as_str(), row.record_id.as_str(), "domain_id")?;
        require_nonblank(
            row.target_rdf_file.as_str(),
            row.record_id.as_str(),
            "target_rdf_file",
        )?;
        match row.record_kind.as_str() {
            OBJECT_INSTANCE_KIND => {
                require_nonblank(
                    row.object_type.as_str(),
                    row.record_id.as_str(),
                    "object_type",
                )?;
                require_nonblank(row.label.as_str(), row.record_id.as_str(), "label")?;
                if !object_ids.insert(row.record_id.as_str()) {
                    anyhow::bail!(
                        "source-patch preflight contains duplicate object record `{}`",
                        row.record_id
                    );
                }
            }
            INSTANCE_RELATION_KIND => {
                require_nonblank(
                    row.source_object_id.as_str(),
                    row.record_id.as_str(),
                    "source_object_id",
                )?;
                require_nonblank(row.predicate.as_str(), row.record_id.as_str(), "predicate")?;
                require_nonblank(
                    row.target_object_id.as_str(),
                    row.record_id.as_str(),
                    "target_object_id",
                )?;
            }
            _ => anyhow::bail!(
                "source-patch preflight row `{}` has unsupported record_kind `{}`",
                row.record_id,
                row.record_kind
            ),
        }
    }

    for row in rows
        .iter()
        .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
    {
        if !object_ids.contains(row.source_object_id.as_str()) {
            anyhow::bail!(
                "source-patch relation `{}` references source_object_id `{}` without an object patch row",
                row.record_id,
                row.source_object_id
            );
        }
        if !object_ids.contains(row.target_object_id.as_str()) {
            anyhow::bail!(
                "source-patch relation `{}` references target_object_id `{}` without an object patch row",
                row.record_id,
                row.target_object_id
            );
        }
    }
    Ok(())
}

fn render_source_patch_draft(rows: &[SourcePatchPreflightRecord]) -> (String, usize, usize) {
    let resources = rows
        .iter()
        .map(render_preflight_resource)
        .collect::<Vec<_>>();
    let resource_count = resources.len();
    let statement_count = resources
        .iter()
        .map(|resource| resource.statement_count)
        .sum();
    let ttl = resources
        .into_iter()
        .fold(RDF_PREFIXES.to_string(), |mut output, resource| {
            output.push_str(resource.text.as_str());
            output
        });
    (ttl, resource_count, statement_count)
}

fn render_preflight_resource(row: &SourcePatchPreflightRecord) -> RenderedResource {
    let mut statements = vec![
        (
            "a",
            match row.record_kind.as_str() {
                OBJECT_INSTANCE_KIND => "wdsp:ObjectInstanceSourcePatch".to_string(),
                INSTANCE_RELATION_KIND => "wdsp:InstanceRelationSourcePatch".to_string(),
                _ => "wdsp:UnknownSourcePatch".to_string(),
            },
        ),
        ("wdsp:recordId", quoted_literal(row.record_id.as_str())),
        ("wdsp:recordKind", quoted_literal(row.record_kind.as_str())),
        ("wdsp:domainId", quoted_literal(row.domain_id.as_str())),
        (
            "wdsp:targetRdfFile",
            quoted_literal(row.target_rdf_file.as_str()),
        ),
        ("wdsp:evidenceId", quoted_literal(row.evidence_id.as_str())),
        (
            "wdsp:reviewDecision",
            quoted_literal(row.review_decision.as_str()),
        ),
        (
            "wdsp:promotionDecision",
            quoted_literal(row.promotion_decision.as_str()),
        ),
        ("wdsp:reviewerId", quoted_literal(row.reviewer_id.as_str())),
        (
            "wdsp:preflightAction",
            quoted_literal(row.preflight_action.as_str()),
        ),
        (
            "wdsp:sourceMutationAllowed",
            typed_bool_literal(row.source_mutation_allowed),
        ),
        ("wdsp:ontologyTruth", typed_bool_literal(row.ontology_truth)),
    ];
    if row.record_kind == OBJECT_INSTANCE_KIND {
        statements.push(("rdfs:label", quoted_literal(row.label.as_str())));
        statements.push(("wdsp:objectType", quoted_literal(row.object_type.as_str())));
    }
    if row.record_kind == INSTANCE_RELATION_KIND {
        statements.push((
            "wdsp:sourceObjectId",
            quoted_literal(row.source_object_id.as_str()),
        ));
        statements.push(("wdsp:predicate", quoted_literal(row.predicate.as_str())));
        statements.push((
            "wdsp:targetObjectId",
            quoted_literal(row.target_object_id.as_str()),
        ));
    }
    render_resource(subject_for(row.record_id.as_str()).as_str(), &statements)
}

fn render_resource(subject: &str, statements: &[(&'static str, String)]) -> RenderedResource {
    let mut text = String::new();
    let last_index = statements.len().saturating_sub(1);
    for (index, (predicate, value)) in statements.iter().enumerate() {
        if index == 0 {
            let _ = write!(text, "{subject} {predicate} {value}");
        } else {
            let _ = write!(text, " ;\n    {predicate} {value}");
        }
        if index == last_index {
            text.push_str(" .\n\n");
        }
    }
    RenderedResource {
        text,
        statement_count: statements.len(),
    }
}

fn write_draft_org(path: &Path, report: &EpistemeOntologySourcePatchDraftReport) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Source Patch Draft")?;
    writeln!(file)?;
    writeln!(file, "* Source patch draft")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_draft")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This draft renders approved source-patch preflight rows as reviewable RDF proposal resources. It does not mutate ontology source files."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| preflight_row_count | {} |",
        report.preflight_row_count
    )?;
    writeln!(
        file,
        "| object_patch_count | {} |",
        report.object_patch_count
    )?;
    writeln!(
        file,
        "| relation_patch_count | {} |",
        report.relation_patch_count
    )?;
    writeln!(
        file,
        "| draft_resource_count | {} |",
        report.draft_resource_count
    )?;
    writeln!(
        file,
        "| draft_statement_count | {} |",
        report.draft_statement_count
    )?;
    writeln!(
        file,
        "| source_mutation_allowed | {} |",
        report.source_mutation_allowed
    )?;
    writeln!(file, "| ontology_truth | {} |", report.ontology_truth)?;
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologySourcePatchDraftReport) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, report)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn write_string(path: &Path, content: &str) -> Result<()> {
    let mut file = create_file(path)?;
    file.write_all(content.as_bytes())
        .with_context(|| format!("failed to write `{}`", path.display()))
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn required(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<String> {
    row.get(name).cloned().with_context(|| {
        format!(
            "source-patch preflight TSV `{}` row {row_number} missing `{name}` column",
            path.display()
        )
    })
}

fn optional(row: &BTreeMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
}

fn parse_bool(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<bool> {
    let value = required(row, name, path, row_number)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "source-patch preflight TSV `{}` row {row_number} has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn require_nonblank(value: &str, record_id: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("source-patch preflight row `{record_id}` must declare nonblank {field}");
    }
    Ok(())
}

fn subject_for(record_id: &str) -> String {
    format!(
        "<urn:wendao:episteme:source-patch:{}>",
        hex_sha256(record_id.as_bytes())
    )
}

fn quoted_literal(value: &str) -> String {
    format!("\"{}\"", escape_rdf_literal(value))
}

fn typed_bool_literal(value: bool) -> String {
    format!("\"{value}\"^^xsd:boolean")
}

fn normalize(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}

fn unescape_tsv(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('t') => output.push('\t'),
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('\\') | None => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn escape_rdf_literal(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
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
