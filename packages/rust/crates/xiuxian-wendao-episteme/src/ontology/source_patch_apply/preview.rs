//! Non-mutating preview for reviewed source-patch application.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::{BEGIN_BLOCK, END_BLOCK, WDSP_NS, reviewed_source_patch_artifacts, sha256_bytes};

const SOURCE_PATCH_APPLY_PREVIEW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply_preview.v1";
const SOURCE_PATCH_APPLY_PREVIEW_ORG: &str = "source_patch_apply_preview.org";
const SOURCE_PATCH_APPLY_PREVIEW_JSON: &str = "source_patch_apply_preview.json";
const SOURCE_PATCH_APPLY_PREVIEW_BLOCKS_DIR: &str = "source_patch_apply_preview_blocks";
const SOURCE_PATCH_APPLY_PREVIEW_PROPOSED_DIR: &str = "source_patch_apply_preview_proposed";

/// Request for previewing a reviewed source patch without mutating source files.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchApplyPreviewRequest {
    episteme_root: PathBuf,
    run_dir: PathBuf,
    expected_apply_plan_tsv_sha256: String,
}

impl EpistemeOntologySourcePatchApplyPreviewRequest {
    /// Create a source-patch apply-preview request.
    #[must_use]
    pub fn new(
        episteme_root: impl Into<PathBuf>,
        run_dir: impl Into<PathBuf>,
        expected_apply_plan_tsv_sha256: impl Into<String>,
    ) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_dir: run_dir.into(),
            expected_apply_plan_tsv_sha256: expected_apply_plan_tsv_sha256.into(),
        }
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

/// Preview metadata for one target RDF source file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchApplyPreviewTarget {
    /// Target RDF path relative to the Episteme `ontology/` directory.
    pub target_rdf_file: String,
    /// Current target RDF SHA-256 digest.
    pub before_rdf_sha256: String,
    /// Proposed target RDF SHA-256 digest after applying the preview block.
    pub proposed_after_rdf_sha256: String,
    /// SHA-256 digest of the bounded preview block.
    pub preview_block_sha256: String,
    /// Preview block artifact path.
    pub preview_block_path: PathBuf,
    /// Complete proposed target RDF artifact path.
    pub proposed_rdf_path: PathBuf,
    /// Whether the complete proposed target RDF passed preview admission.
    pub proposed_rdf_admission_passed: bool,
    /// Deterministic preview admission checks applied to the complete proposed RDF.
    pub proposed_rdf_admission_checks: Vec<&'static str>,
    /// Number of apply-plan rows represented in this preview block.
    pub preview_row_count: usize,
}

/// Report emitted after source-patch apply preview.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchApplyPreviewReport {
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
    /// Generated preview receipt Org path.
    pub source_patch_apply_preview_org: PathBuf,
    /// Generated preview receipt JSON path.
    pub source_patch_apply_preview_json: PathBuf,
    /// Operator-provided expected apply-plan TSV hash.
    pub expected_apply_plan_tsv_sha256: String,
    /// Actual apply-plan TSV hash.
    pub apply_plan_tsv_sha256: String,
    /// Number of apply-plan rows previewed.
    pub apply_plan_row_count: usize,
    /// Number of target RDF files previewed.
    pub target_rdf_file_count: usize,
    /// Per-target preview metadata.
    pub preview_targets: Vec<EpistemeOntologySourcePatchApplyPreviewTarget>,
    /// Whether this preview authorized source mutation.
    pub source_mutation_allowed: bool,
    /// Whether this preview marks rows as ontology truth.
    pub ontology_truth: bool,
}

/// Write a non-mutating source-patch apply preview.
///
/// # Errors
///
/// Returns an error when reviewed artifacts are missing or inconsistent, the
/// expected apply-plan hash does not match the review packet, target RDF hashes
/// drifted, preview block artifacts cannot be written, or the target RDF/XML
/// file cannot accept the bounded source-patch block.
pub fn write_episteme_ontology_source_patch_apply_preview(
    request: &EpistemeOntologySourcePatchApplyPreviewRequest,
) -> Result<EpistemeOntologySourcePatchApplyPreviewReport> {
    let expected_hash = request.expected_apply_plan_tsv_sha256.trim();
    if expected_hash.is_empty() {
        anyhow::bail!("source-patch apply preview requires an expected apply-plan TSV hash");
    }
    let artifacts =
        reviewed_source_patch_artifacts(request.episteme_root(), request.run_dir(), expected_hash)?;
    let blocks_dir = request
        .run_dir()
        .join(SOURCE_PATCH_APPLY_PREVIEW_BLOCKS_DIR);
    let proposed_dir = request
        .run_dir()
        .join(SOURCE_PATCH_APPLY_PREVIEW_PROPOSED_DIR);
    fs::create_dir_all(blocks_dir.as_path())
        .with_context(|| format!("failed to create `{}`", blocks_dir.display()))?;
    fs::create_dir_all(proposed_dir.as_path())
        .with_context(|| format!("failed to create `{}`", proposed_dir.display()))?;

    let mut preview_targets = Vec::new();
    for plan in artifacts.write_plans {
        let block_path = blocks_dir.join(preview_block_filename(plan.target_rdf_file.as_str()));
        let proposed_path = proposed_dir.join(proposed_rdf_filename(plan.target_rdf_file.as_str()));
        let admission_checks = proposed_rdf_admission_checks(&plan)?;
        write_string(block_path.as_path(), plan.proposal_block.as_str())?;
        write_string(proposed_path.as_path(), plan.proposed_content.as_str())?;
        preview_targets.push(EpistemeOntologySourcePatchApplyPreviewTarget {
            target_rdf_file: plan.target_rdf_file,
            before_rdf_sha256: plan.before_hash,
            proposed_after_rdf_sha256: sha256_bytes(plan.proposed_content.as_bytes()),
            preview_block_sha256: sha256_bytes(plan.proposal_block.as_bytes()),
            preview_block_path: block_path,
            proposed_rdf_path: proposed_path,
            proposed_rdf_admission_passed: true,
            proposed_rdf_admission_checks: admission_checks,
            preview_row_count: plan.row_count,
        });
    }

    let output_org = request.run_dir().join(SOURCE_PATCH_APPLY_PREVIEW_ORG);
    let output_json = request.run_dir().join(SOURCE_PATCH_APPLY_PREVIEW_JSON);
    let report = EpistemeOntologySourcePatchApplyPreviewReport {
        schema_version: SOURCE_PATCH_APPLY_PREVIEW_SCHEMA_VERSION,
        episteme_root: request.episteme_root().to_path_buf(),
        run_dir: request.run_dir().to_path_buf(),
        source_patch_review_packet_json: artifacts.source_patch_review_packet_json,
        source_patch_apply_plan_tsv: artifacts.source_patch_apply_plan_tsv,
        source_patch_apply_preview_org: output_org,
        source_patch_apply_preview_json: output_json,
        expected_apply_plan_tsv_sha256: artifacts.expected_apply_plan_tsv_sha256,
        apply_plan_tsv_sha256: artifacts.apply_plan_tsv_sha256,
        apply_plan_row_count: artifacts.apply_plan_row_count,
        target_rdf_file_count: preview_targets.len(),
        preview_targets,
        source_mutation_allowed: false,
        ontology_truth: false,
    };
    write_preview_org(report.source_patch_apply_preview_org.as_path(), &report)?;
    write_json(report.source_patch_apply_preview_json.as_path(), &report)?;
    Ok(report)
}

fn preview_block_filename(target_rdf_file: &str) -> String {
    format!(
        "{}.source-patch-preview.rdf.xml",
        safe_target_filename(target_rdf_file)
    )
}

fn proposed_rdf_filename(target_rdf_file: &str) -> String {
    format!(
        "{}.source-patch-proposed.rdf.xml",
        safe_target_filename(target_rdf_file)
    )
}

fn safe_target_filename(target_rdf_file: &str) -> String {
    target_rdf_file
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
}

fn proposed_rdf_admission_checks(plan: &super::TargetWritePlan) -> Result<Vec<&'static str>> {
    if !plan.proposed_content.contains("<rdf:RDF") {
        anyhow::bail!(
            "source-patch preview target `{}` proposed RDF is missing <rdf:RDF",
            plan.target_rdf_file
        );
    }
    if plan.proposed_content.matches("</rdf:RDF>").count() != 1 {
        anyhow::bail!(
            "source-patch preview target `{}` proposed RDF must contain exactly one closing </rdf:RDF>",
            plan.target_rdf_file
        );
    }
    if plan.proposed_content.matches(BEGIN_BLOCK).count() != 1
        || plan.proposed_content.matches(END_BLOCK).count() != 1
    {
        anyhow::bail!(
            "source-patch preview target `{}` proposed RDF must contain one bounded source-patch block",
            plan.target_rdf_file
        );
    }
    if !plan.proposed_content.contains(WDSP_NS) {
        anyhow::bail!(
            "source-patch preview target `{}` proposed RDF is missing source-patch namespace",
            plan.target_rdf_file
        );
    }
    if contains_true_xml_bool_element(&plan.proposed_content, "wdsp:sourceMutationAllowed")
        || contains_true_xml_bool_element(&plan.proposed_content, "wdsp:ontologyTruth")
    {
        anyhow::bail!(
            "source-patch preview target `{}` proposed RDF attempted to mark mutation or ontology truth",
            plan.target_rdf_file
        );
    }
    Ok(vec![
        "rdf_root_present",
        "single_rdf_close",
        "single_bounded_source_patch_block",
        "source_patch_namespace_present",
        "no_mutation_or_truth_escalation",
    ])
}

fn contains_true_xml_bool_element(content: &str, element_name: &str) -> bool {
    let open_prefix = format!("<{element_name}");
    let close = format!("</{element_name}>");
    let mut search_from = 0;
    while let Some(relative_start) = content[search_from..].find(open_prefix.as_str()) {
        let element_start = search_from + relative_start;
        let Some(relative_open_end) = content[element_start..].find('>') else {
            return false;
        };
        let value_start = element_start + relative_open_end + 1;
        let Some(relative_close_start) = content[value_start..].find(close.as_str()) else {
            return false;
        };
        let value = &content[value_start..value_start + relative_close_start];
        if value.trim().eq_ignore_ascii_case("true") {
            return true;
        }
        search_from = value_start + relative_close_start + close.len();
    }
    false
}

fn write_preview_org(
    path: &Path,
    report: &EpistemeOntologySourcePatchApplyPreviewReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Source Patch Apply Preview")?;
    writeln!(file)?;
    writeln!(file, "* Source patch apply preview")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_apply_preview")?;
    writeln!(
        file,
        ":APPLY_PLAN_TSV_SHA256: {}",
        report.apply_plan_tsv_sha256
    )?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This preview renders the exact source-patch block without mutating target RDF files."
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
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    writeln!(file)?;
    writeln!(file, "** Preview Target RDF hashes")?;
    writeln!(
        file,
        "| target_rdf_file | before_rdf_sha256 | proposed_after_rdf_sha256 | preview_block_sha256 | preview_row_count | proposed_rdf_admission_passed | preview_block_path | proposed_rdf_path |"
    )?;
    writeln!(file, "|-|-|-|-|-|-|-|-|")?;
    for target in &report.preview_targets {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            target.target_rdf_file,
            target.before_rdf_sha256,
            target.proposed_after_rdf_sha256,
            target.preview_block_sha256,
            target.preview_row_count,
            target.proposed_rdf_admission_passed,
            target.preview_block_path.display(),
            target.proposed_rdf_path.display()
        )?;
    }
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologySourcePatchApplyPreviewReport) -> Result<()> {
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
