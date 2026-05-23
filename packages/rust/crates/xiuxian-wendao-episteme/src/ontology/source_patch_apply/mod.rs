//! Explicit hash-gated source-patch application.

mod preview;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use preview::{
    EpistemeOntologySourcePatchApplyPreviewReport, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchApplyPreviewTarget,
    write_episteme_ontology_source_patch_apply_preview,
};

const SOURCE_PATCH_APPLY_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply.v1";
const SOURCE_PATCH_REVIEW_PACKET_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_review_packet.v1";
const SOURCE_PATCH_APPLY_PLAN_TSV: &str = "source_patch_apply_plan.tsv";
const SOURCE_PATCH_REVIEW_PACKET_JSON: &str = "source_patch_review_packet.json";
const SOURCE_PATCH_APPLY_ORG: &str = "source_patch_apply.org";
const SOURCE_PATCH_APPLY_JSON: &str = "source_patch_apply.json";
const APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH: &str = "propose_targeted_source_patch";
const OBJECT_INSTANCE_KIND: &str = "object_instance";
const INSTANCE_RELATION_KIND: &str = "instance_relation";
const WDSP_NS: &str = "https://wendao.ai/ontology/source-patch#";
const BEGIN_BLOCK: &str = "BEGIN WENDAO SOURCE PATCH";
const END_BLOCK: &str = "END WENDAO SOURCE PATCH";

/// Request for applying a reviewed source patch to ontology source files.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchApplyRequest {
    episteme_root: PathBuf,
    run_dir: PathBuf,
    expected_apply_plan_tsv_sha256: Option<String>,
    allow_source_mutation: bool,
}

impl EpistemeOntologySourcePatchApplyRequest {
    /// Create a source-patch apply request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_dir: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_dir: run_dir.into(),
            expected_apply_plan_tsv_sha256: None,
            allow_source_mutation: false,
        }
    }

    /// Require the operator-observed apply-plan TSV hash.
    #[must_use]
    pub fn with_expected_apply_plan_tsv_sha256(mut self, expected: impl Into<String>) -> Self {
        self.expected_apply_plan_tsv_sha256 = Some(expected.into());
        self
    }

    /// Explicitly enable source mutation.
    #[must_use]
    pub fn with_allow_source_mutation(mut self, allow: bool) -> Self {
        self.allow_source_mutation = allow;
        self
    }

    /// Episteme repository root containing ontology source files.
    #[must_use]
    pub fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    /// Source-patch run directory containing reviewed artifacts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Hash metadata for one source-patch target after application.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchAppliedTarget {
    /// Target RDF path relative to the Episteme `ontology/` directory.
    pub target_rdf_file: String,
    /// SHA-256 digest recorded in the review packet before mutation.
    pub before_rdf_sha256: String,
    /// SHA-256 digest after writing the source-patch block.
    pub after_rdf_sha256: String,
    /// Number of apply-plan rows written to this target.
    pub applied_row_count: usize,
}

/// Report emitted after source-patch application.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchApplyReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Episteme repository root used to resolve target RDF files.
    pub episteme_root: PathBuf,
    /// Source-patch run directory.
    pub run_dir: PathBuf,
    /// Source review-packet JSON path.
    pub source_patch_review_packet_json: PathBuf,
    /// Source apply-plan TSV path.
    pub source_patch_apply_plan_tsv: PathBuf,
    /// Generated apply receipt Org path.
    pub source_patch_apply_org: PathBuf,
    /// Generated apply receipt JSON path.
    pub source_patch_apply_json: PathBuf,
    /// Operator-provided expected apply-plan TSV hash.
    pub expected_apply_plan_tsv_sha256: String,
    /// Actual apply-plan TSV hash.
    pub apply_plan_tsv_sha256: String,
    /// Number of apply-plan rows applied.
    pub apply_plan_row_count: usize,
    /// Number of target RDF files mutated.
    pub target_rdf_file_count: usize,
    /// Per-target mutation receipts.
    pub applied_targets: Vec<EpistemeOntologySourcePatchAppliedTarget>,
    /// Whether this request authorized source mutation.
    pub source_mutation_allowed: bool,
    /// Whether the applied proposal block itself is ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePatchReviewPacketReceipt {
    schema_version: String,
    apply_plan_tsv_sha256: String,
    apply_plan_row_count: usize,
    object_apply_plan_count: usize,
    relation_apply_plan_count: usize,
    target_rdf_files: Vec<SourcePatchReviewPacketTarget>,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePatchReviewPacketTarget {
    target_rdf_file: String,
    target_rdf_sha256: String,
}

#[derive(Debug, Clone)]
struct SourcePatchApplyPlanRow {
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
    apply_action: String,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

pub(super) struct TargetWritePlan {
    pub(super) target_rdf_file: String,
    pub(super) path: PathBuf,
    pub(super) before_hash: String,
    pub(super) proposed_content: String,
    pub(super) proposal_block: String,
    pub(super) row_count: usize,
}

pub(super) struct ReviewedSourcePatchArtifacts {
    pub(super) source_patch_review_packet_json: PathBuf,
    pub(super) source_patch_apply_plan_tsv: PathBuf,
    pub(super) expected_apply_plan_tsv_sha256: String,
    pub(super) apply_plan_tsv_sha256: String,
    pub(super) apply_plan_row_count: usize,
    pub(super) write_plans: Vec<TargetWritePlan>,
}

/// Apply reviewed source-patch proposal resources to target ontology source files.
///
/// # Errors
///
/// Returns an error when reviewed artifacts are missing or inconsistent, the
/// operator did not explicitly allow source mutation, the expected apply-plan
/// hash does not match the review packet, target RDF hashes drifted, or the
/// target RDF/XML file cannot accept a bounded source-patch block.
pub fn apply_episteme_ontology_source_patch(
    request: &EpistemeOntologySourcePatchApplyRequest,
) -> Result<EpistemeOntologySourcePatchApplyReport> {
    if !request.allow_source_mutation {
        anyhow::bail!("source-patch apply requires explicit source mutation approval");
    }
    let expected_hash = request
        .expected_apply_plan_tsv_sha256
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| "source-patch apply requires an expected apply-plan TSV hash")?;
    let episteme_root = request.episteme_root();
    let run_dir = request.run_dir();
    let artifacts = reviewed_source_patch_artifacts(episteme_root, run_dir, expected_hash)?;

    let mut applied_targets = Vec::new();
    for plan in artifacts.write_plans {
        fs::write(plan.path.as_path(), plan.proposed_content.as_bytes())
            .with_context(|| format!("failed to write `{}`", plan.path.display()))?;
        let after_hash = sha256_file(plan.path.as_path())?;
        applied_targets.push(EpistemeOntologySourcePatchAppliedTarget {
            target_rdf_file: plan.target_rdf_file,
            before_rdf_sha256: plan.before_hash,
            after_rdf_sha256: after_hash,
            applied_row_count: plan.row_count,
        });
    }

    let output_org = run_dir.join(SOURCE_PATCH_APPLY_ORG);
    let output_json = run_dir.join(SOURCE_PATCH_APPLY_JSON);
    let report = EpistemeOntologySourcePatchApplyReport {
        schema_version: SOURCE_PATCH_APPLY_SCHEMA_VERSION,
        episteme_root: episteme_root.to_path_buf(),
        run_dir: run_dir.to_path_buf(),
        source_patch_review_packet_json: artifacts.source_patch_review_packet_json,
        source_patch_apply_plan_tsv: artifacts.source_patch_apply_plan_tsv,
        source_patch_apply_org: output_org,
        source_patch_apply_json: output_json,
        expected_apply_plan_tsv_sha256: artifacts.expected_apply_plan_tsv_sha256,
        apply_plan_tsv_sha256: artifacts.apply_plan_tsv_sha256,
        apply_plan_row_count: artifacts.apply_plan_row_count,
        target_rdf_file_count: applied_targets.len(),
        applied_targets,
        source_mutation_allowed: true,
        ontology_truth: false,
    };
    write_apply_org(report.source_patch_apply_org.as_path(), &report)?;
    write_json(report.source_patch_apply_json.as_path(), &report)?;
    Ok(report)
}

pub(super) fn reviewed_source_patch_artifacts(
    episteme_root: &Path,
    run_dir: &Path,
    expected_hash: &str,
) -> Result<ReviewedSourcePatchArtifacts> {
    let review_packet_json = run_dir.join(SOURCE_PATCH_REVIEW_PACKET_JSON);
    let apply_plan_tsv = run_dir.join(SOURCE_PATCH_APPLY_PLAN_TSV);
    let review_packet = read_review_packet(review_packet_json.as_path())?;
    let rows = read_apply_plan_rows(apply_plan_tsv.as_path())?;
    validate_review_packet(&review_packet, &rows)?;
    let actual_apply_plan_hash = sha256_file(apply_plan_tsv.as_path())?;
    if expected_hash != review_packet.apply_plan_tsv_sha256 {
        anyhow::bail!(
            "expected apply-plan TSV hash `{expected_hash}` does not match review packet hash `{}`",
            review_packet.apply_plan_tsv_sha256
        );
    }
    if actual_apply_plan_hash != review_packet.apply_plan_tsv_sha256 {
        anyhow::bail!(
            "current apply-plan TSV hash `{actual_apply_plan_hash}` does not match review packet hash `{}`",
            review_packet.apply_plan_tsv_sha256
        );
    }
    let write_plans = prepare_target_write_plans(episteme_root, &review_packet, &rows)?;
    Ok(ReviewedSourcePatchArtifacts {
        source_patch_review_packet_json: review_packet_json,
        source_patch_apply_plan_tsv: apply_plan_tsv,
        expected_apply_plan_tsv_sha256: expected_hash.to_string(),
        apply_plan_tsv_sha256: actual_apply_plan_hash,
        apply_plan_row_count: rows.len(),
        write_plans,
    })
}

fn read_review_packet(path: &Path) -> Result<SourcePatchReviewPacketReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse review-packet JSON `{}`", path.display()))
}

fn read_apply_plan_rows(path: &Path) -> Result<Vec<SourcePatchApplyPlanRow>> {
    let file = File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()
        .with_context(|| format!("failed to read `{}`", path.display()))?
        .with_context(|| format!("source-patch apply-plan TSV `{}` is empty", path.display()))?;
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
                "source-patch apply-plan TSV `{}` row {} has {} values for {} columns",
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
        rows.push(apply_plan_row(path, line_index + 2, &row)?);
    }
    Ok(rows)
}

fn apply_plan_row(
    path: &Path,
    row_number: usize,
    row: &BTreeMap<String, String>,
) -> Result<SourcePatchApplyPlanRow> {
    Ok(SourcePatchApplyPlanRow {
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
        apply_action: required(row, "apply_action", path, row_number)?,
        source_mutation_allowed: parse_bool(row, "source_mutation_allowed", path, row_number)?,
        ontology_truth: parse_bool(row, "ontology_truth", path, row_number)?,
    })
}

fn validate_review_packet(
    review_packet: &SourcePatchReviewPacketReceipt,
    rows: &[SourcePatchApplyPlanRow],
) -> Result<()> {
    if review_packet.schema_version != SOURCE_PATCH_REVIEW_PACKET_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch review packet has unsupported schemaVersion `{}`",
            review_packet.schema_version
        );
    }
    if review_packet.source_mutation_allowed {
        anyhow::bail!("source-patch review packet attempted to pre-authorize source mutation");
    }
    if review_packet.ontology_truth {
        anyhow::bail!("source-patch review packet attempted to mark ontology truth");
    }
    if review_packet.apply_plan_row_count != rows.len() {
        anyhow::bail!(
            "source-patch apply row count mismatch: review packet has {}, TSV has {}",
            review_packet.apply_plan_row_count,
            rows.len()
        );
    }
    let object_count = rows
        .iter()
        .filter(|row| row.record_kind == OBJECT_INSTANCE_KIND)
        .count();
    let relation_count = rows
        .iter()
        .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
        .count();
    if review_packet.object_apply_plan_count != object_count {
        anyhow::bail!(
            "source-patch object row count mismatch: review packet has {}, TSV has {object_count}",
            review_packet.object_apply_plan_count
        );
    }
    if review_packet.relation_apply_plan_count != relation_count {
        anyhow::bail!(
            "source-patch relation row count mismatch: review packet has {}, TSV has {relation_count}",
            review_packet.relation_apply_plan_count
        );
    }
    for row in rows {
        validate_row(row)?;
    }
    Ok(())
}

fn validate_row(row: &SourcePatchApplyPlanRow) -> Result<()> {
    if row.apply_action != APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH {
        anyhow::bail!(
            "source-patch row `{}` has unsupported apply_action `{}`",
            row.record_id,
            row.apply_action
        );
    }
    if row.source_mutation_allowed {
        anyhow::bail!(
            "source-patch row `{}` attempted to pre-authorize source mutation",
            row.record_id
        );
    }
    if row.ontology_truth {
        anyhow::bail!(
            "source-patch row `{}` attempted to mark ontology truth",
            row.record_id
        );
    }
    Ok(())
}

fn prepare_target_write_plans(
    episteme_root: &Path,
    review_packet: &SourcePatchReviewPacketReceipt,
    rows: &[SourcePatchApplyPlanRow],
) -> Result<Vec<TargetWritePlan>> {
    let targets_by_file = review_packet
        .target_rdf_files
        .iter()
        .map(|target| {
            (
                target.target_rdf_file.as_str(),
                target.target_rdf_sha256.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let row_targets = rows
        .iter()
        .map(|row| row.target_rdf_file.as_str())
        .collect::<BTreeSet<_>>();
    for target in &row_targets {
        if !targets_by_file.contains_key(target) {
            anyhow::bail!("source-patch row target `{target}` is absent from review packet");
        }
    }

    let mut write_plans = Vec::new();
    for target in row_targets {
        let path = resolve_target_rdf_file(episteme_root, target)?;
        let before_hash = sha256_file(path.as_path())?;
        let expected_hash = targets_by_file.get(target).with_context(|| {
            format!("source-patch target `{target}` is absent from review packet")
        })?;
        if before_hash != *expected_hash {
            anyhow::bail!(
                "source-patch target `{target}` hash drifted: review packet has {expected_hash}, current file has {before_hash}"
            );
        }
        let content = fs::read_to_string(path.as_path())
            .with_context(|| format!("failed to read `{}`", path.display()))?;
        let rows_for_target = rows
            .iter()
            .filter(|row| row.target_rdf_file == target)
            .collect::<Vec<_>>();
        let proposal_block = source_patch_block(target, expected_hash, &rows_for_target);
        let proposed_content = append_source_patch_block(
            content.as_str(),
            target,
            expected_hash,
            proposal_block.as_str(),
        )?;
        write_plans.push(TargetWritePlan {
            target_rdf_file: target.to_string(),
            path,
            before_hash,
            proposed_content,
            proposal_block,
            row_count: rows_for_target.len(),
        });
    }
    Ok(write_plans)
}

fn append_source_patch_block(
    content: &str,
    target_rdf_file: &str,
    target_hash: &str,
    proposal_block: &str,
) -> Result<String> {
    let marker = format!("{BEGIN_BLOCK} target={target_rdf_file} targetHash={target_hash}");
    if content.contains(marker.as_str()) {
        anyhow::bail!(
            "source-patch target `{target_rdf_file}` already contains block for hash `{target_hash}`"
        );
    }
    let Some(close_index) = content.rfind("</rdf:RDF>") else {
        anyhow::bail!("target RDF/XML is missing closing </rdf:RDF> element");
    };
    let mut output = String::with_capacity(content.len() + proposal_block.len());
    output.push_str(&content[..close_index]);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(proposal_block);
    output.push_str(&content[close_index..]);
    Ok(output)
}

fn source_patch_block(
    target_rdf_file: &str,
    target_hash: &str,
    rows: &[&SourcePatchApplyPlanRow],
) -> String {
    let mut output = String::new();
    output.push_str("  <!-- ");
    output.push_str(
        format!("{BEGIN_BLOCK} target={target_rdf_file} targetHash={target_hash}").as_str(),
    );
    output.push_str(" -->\n");
    for row in rows {
        output.push_str(render_rdf_xml_source_patch(row).as_str());
    }
    output.push_str("  <!-- ");
    output.push_str(END_BLOCK);
    output.push_str(" -->\n");
    output
}

fn render_rdf_xml_source_patch(row: &SourcePatchApplyPlanRow) -> String {
    let type_resource = match row.record_kind.as_str() {
        OBJECT_INSTANCE_KIND => format!("{WDSP_NS}ObjectInstanceSourcePatch"),
        INSTANCE_RELATION_KIND => format!("{WDSP_NS}InstanceRelationSourcePatch"),
        _ => format!("{WDSP_NS}UnknownSourcePatch"),
    };
    let mut output = String::new();
    output.push_str("  <rdf:Description rdf:about=\"");
    output.push_str(xml_escape_attribute(subject_for(row.record_id.as_str()).as_str()).as_str());
    output.push_str("\" xmlns:wdsp=\"");
    output.push_str(WDSP_NS);
    output.push_str("\">\n");
    output.push_str("    <rdf:type rdf:resource=\"");
    output.push_str(type_resource.as_str());
    output.push_str("\"/>\n");
    write_text_element(&mut output, "wdsp:recordId", row.record_id.as_str());
    write_text_element(&mut output, "wdsp:recordKind", row.record_kind.as_str());
    write_text_element(&mut output, "wdsp:domainId", row.domain_id.as_str());
    write_text_element(
        &mut output,
        "wdsp:targetRdfFile",
        row.target_rdf_file.as_str(),
    );
    write_text_element(&mut output, "wdsp:evidenceId", row.evidence_id.as_str());
    write_text_element(
        &mut output,
        "wdsp:reviewDecision",
        row.review_decision.as_str(),
    );
    write_text_element(
        &mut output,
        "wdsp:promotionDecision",
        row.promotion_decision.as_str(),
    );
    write_text_element(&mut output, "wdsp:reviewerId", row.reviewer_id.as_str());
    write_text_element(&mut output, "wdsp:applyAction", row.apply_action.as_str());
    write_bool_element(
        &mut output,
        "wdsp:sourceMutationAllowed",
        row.source_mutation_allowed,
    );
    write_bool_element(&mut output, "wdsp:ontologyTruth", row.ontology_truth);
    if row.record_kind == OBJECT_INSTANCE_KIND {
        write_text_element(&mut output, "rdfs:label", row.label.as_str());
        write_text_element(&mut output, "wdsp:objectType", row.object_type.as_str());
    }
    if row.record_kind == INSTANCE_RELATION_KIND {
        write_text_element(
            &mut output,
            "wdsp:sourceObjectId",
            row.source_object_id.as_str(),
        );
        write_text_element(&mut output, "wdsp:predicate", row.predicate.as_str());
        write_text_element(
            &mut output,
            "wdsp:targetObjectId",
            row.target_object_id.as_str(),
        );
    }
    output.push_str("  </rdf:Description>\n");
    output
}

fn write_text_element(output: &mut String, name: &str, value: &str) {
    output.push_str("    <");
    output.push_str(name);
    output.push('>');
    output.push_str(xml_escape_text(value).as_str());
    output.push_str("</");
    output.push_str(name);
    output.push_str(">\n");
}

fn write_bool_element(output: &mut String, name: &str, value: bool) {
    output.push_str("    <");
    output.push_str(name);
    output.push_str(" rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">");
    output.push_str(if value { "true" } else { "false" });
    output.push_str("</");
    output.push_str(name);
    output.push_str(">\n");
}

fn resolve_target_rdf_file(episteme_root: &Path, target_rdf_file: &str) -> Result<PathBuf> {
    if target_rdf_file.trim().is_empty() {
        anyhow::bail!("source-patch target RDF file is blank");
    }
    let relative = Path::new(target_rdf_file);
    if relative.is_absolute() || has_parent_dir(relative) {
        anyhow::bail!("source-patch target RDF file `{target_rdf_file}` is unsafe");
    }
    let root = episteme_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize `{}`", episteme_root.display()))?;
    let ontology_root = root.join("ontology");
    let path = ontology_root.join(relative);
    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize `{}`", path.display()))?;
    if !canonical.starts_with(ontology_root) {
        anyhow::bail!("source-patch target RDF file `{target_rdf_file}` escapes ontology root");
    }
    Ok(canonical)
}

fn has_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to hash `{}`", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read `{}`", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_digest(&hasher.finalize()))
}

pub(super) fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_digest(&hasher.finalize())
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn write_apply_org(path: &Path, report: &EpistemeOntologySourcePatchApplyReport) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Source Patch Apply Receipt")?;
    writeln!(file)?;
    writeln!(file, "* Source patch apply receipt")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_apply")?;
    writeln!(
        file,
        ":APPLY_PLAN_TSV_SHA256: {}",
        report.apply_plan_tsv_sha256
    )?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: true")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This receipt records an explicit hash-gated source-patch application."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| apply_plan_row_count | {} |",
        report.apply_plan_row_count
    )?;
    writeln!(
        file,
        "| target_rdf_file_count | {} |",
        report.target_rdf_file_count
    )?;
    writeln!(file, "| source_mutation_allowed | true |")?;
    writeln!(file, "| ontology_truth | false |")?;
    writeln!(file)?;
    writeln!(file, "** Applied Target RDF hashes")?;
    writeln!(
        file,
        "| target_rdf_file | before_rdf_sha256 | after_rdf_sha256 | applied_row_count |"
    )?;
    writeln!(file, "|-|-|-|-|")?;
    for target in &report.applied_targets {
        writeln!(
            file,
            "| {} | {} | {} | {} |",
            target.target_rdf_file,
            target.before_rdf_sha256,
            target.after_rdf_sha256,
            target.applied_row_count
        )?;
    }
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologySourcePatchApplyReport) -> Result<()> {
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

fn required(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<String> {
    row.get(name).cloned().with_context(|| {
        format!(
            "source-patch apply-plan TSV `{}` row {row_number} missing `{name}` column",
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
            "source-patch apply-plan TSV `{}` row {row_number} has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
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

fn subject_for(record_id: &str) -> String {
    format!(
        "urn:wendao:episteme:source-patch:{}",
        record_id
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>()
    )
}

fn xml_escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn xml_escape_attribute(value: &str) -> String {
    xml_escape_text(value).replace('"', "&quot;")
}
