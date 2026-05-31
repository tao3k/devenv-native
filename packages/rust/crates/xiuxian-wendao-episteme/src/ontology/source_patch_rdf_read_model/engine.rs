//! Semantic read-model generation engine from applied source-patch RDF.

use std::{
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use serde_json::from_str;
use sha2::{Digest, Sha256};

use super::parse::read_source_patch_rows_from_rdf;
use super::projection::{compile_semantic_projection, projection_quality_issues};
use super::types::{
    EpistemeOntologySourcePatchRdfReadModelReport, EpistemeOntologySourcePatchRdfReadModelRequest,
    RDF_SOURCE_PROJECTION_STATE_JSON, RDF_SOURCE_READ_MODEL_JSON, RDF_SOURCE_READ_MODEL_ORG,
    RDF_SOURCE_SEMANTIC_EVIDENCE_JSON, RDF_SOURCE_SEMANTIC_EVIDENCE_TSV,
    RDF_SOURCE_SEMANTIC_OBJECTS_JSON, RDF_SOURCE_SEMANTIC_OBJECTS_TSV,
    RDF_SOURCE_SEMANTIC_RELATIONS_JSON, RDF_SOURCE_SEMANTIC_RELATIONS_TSV, SOURCE_PATCH_APPLY_JSON,
    SourcePatchApplyReceipt,
};
use super::write::{
    write_evidence_tsv, write_json, write_objects_tsv, write_read_model_org, write_relations_tsv,
};

const RDF_READ_MODEL_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_rdf_read_model.v1";
const SOURCE_PATCH_APPLY_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply.v1";

/// Compile applied source-patch RDF records into semantic read-model rows.
///
/// # Errors
///
/// Returns an error when the source-patch apply receipt is missing or stale,
/// target RDF files drift from their apply receipt hashes, source-patch RDF
/// rows are malformed, or deterministic projection quality checks fail.
pub fn write_episteme_ontology_source_patch_rdf_read_model(
    request: &EpistemeOntologySourcePatchRdfReadModelRequest,
) -> Result<EpistemeOntologySourcePatchRdfReadModelReport> {
    write_episteme_ontology_source_patch_rdf_read_model_impl(request)
}

fn write_episteme_ontology_source_patch_rdf_read_model_impl(
    request: &EpistemeOntologySourcePatchRdfReadModelRequest,
) -> Result<EpistemeOntologySourcePatchRdfReadModelReport> {
    let episteme_root = request.episteme_root();
    let run_dir = request.run_dir();
    let apply_json = run_dir.join(SOURCE_PATCH_APPLY_JSON);
    let receipt = read_apply_receipt(apply_json.as_path())?;
    validate_apply_receipt(&receipt)?;

    let mut rows = Vec::new();
    for target in &receipt.applied_targets {
        let target_path = resolve_target_rdf_file(episteme_root, target.target_rdf_file.as_str())?;
        let current_hash = sha256_file(target_path.as_path())?;
        if current_hash != target.after_rdf_sha256 {
            anyhow::bail!(
                "source-patch RDF target `{}` hash drifted: apply receipt has {}, current file has {current_hash}",
                target.target_rdf_file,
                target.after_rdf_sha256
            );
        }
        let target_rows = read_source_patch_rows_from_rdf(
            target_path.as_path(),
            target.target_rdf_file.as_str(),
        )?;
        if target_rows.len() != target.applied_row_count {
            anyhow::bail!(
                "source-patch RDF target `{}` row count mismatch: apply receipt has {}, RDF has {}",
                target.target_rdf_file,
                target.applied_row_count,
                target_rows.len()
            );
        }
        rows.extend(target_rows);
    }
    if rows.len() != receipt.apply_plan_row_count {
        anyhow::bail!(
            "source-patch RDF row count mismatch: apply receipt has {}, RDF has {}",
            receipt.apply_plan_row_count,
            rows.len()
        );
    }

    let projection = compile_semantic_projection(&rows)?;
    let quality_issues = projection_quality_issues(&projection);
    if !quality_issues.is_empty() {
        anyhow::bail!(
            "RDF source read-model quality checks failed: {}",
            quality_issues.join("; ")
        );
    }

    let semantic_objects_tsv = run_dir.join(RDF_SOURCE_SEMANTIC_OBJECTS_TSV);
    let semantic_objects_json = run_dir.join(RDF_SOURCE_SEMANTIC_OBJECTS_JSON);
    let semantic_relations_tsv = run_dir.join(RDF_SOURCE_SEMANTIC_RELATIONS_TSV);
    let semantic_relations_json = run_dir.join(RDF_SOURCE_SEMANTIC_RELATIONS_JSON);
    let semantic_evidence_tsv = run_dir.join(RDF_SOURCE_SEMANTIC_EVIDENCE_TSV);
    let semantic_evidence_json = run_dir.join(RDF_SOURCE_SEMANTIC_EVIDENCE_JSON);
    let semantic_projection_state_json = run_dir.join(RDF_SOURCE_PROJECTION_STATE_JSON);
    let rdf_source_read_model_org = run_dir.join(RDF_SOURCE_READ_MODEL_ORG);
    let rdf_source_read_model_json = run_dir.join(RDF_SOURCE_READ_MODEL_JSON);

    write_objects_tsv(semantic_objects_tsv.as_path(), &projection.objects)?;
    write_json(semantic_objects_json.as_path(), &projection.objects)?;
    write_relations_tsv(semantic_relations_tsv.as_path(), &projection.relations)?;
    write_json(semantic_relations_json.as_path(), &projection.relations)?;
    write_evidence_tsv(semantic_evidence_tsv.as_path(), &projection.evidence)?;
    write_json(semantic_evidence_json.as_path(), &projection.evidence)?;
    write_json(
        semantic_projection_state_json.as_path(),
        &projection.projection_state,
    )?;

    let report = EpistemeOntologySourcePatchRdfReadModelReport {
        schema_version: RDF_READ_MODEL_SCHEMA_VERSION,
        episteme_root: episteme_root.to_path_buf(),
        run_dir: run_dir.to_path_buf(),
        source_patch_apply_json: apply_json,
        semantic_objects_tsv,
        semantic_objects_json,
        semantic_relations_tsv,
        semantic_relations_json,
        semantic_evidence_tsv,
        semantic_evidence_json,
        semantic_projection_state_json,
        rdf_source_read_model_org,
        rdf_source_read_model_json,
        rdf_source_row_count: rows.len(),
        semantic_object_count: projection.objects.len(),
        semantic_relation_count: projection.relations.len(),
        semantic_evidence_count: projection.evidence.len(),
        semantic_projection_state_count: projection.projection_state.len(),
        target_rdf_file_count: receipt.applied_targets.len(),
        projection_quality_passed: true,
        quality_issues,
        source_mutation_allowed: false,
        ontology_truth: false,
    };
    write_read_model_org(report.rdf_source_read_model_org.as_path(), &report)?;
    write_json(report.rdf_source_read_model_json.as_path(), &report)?;
    Ok(report)
}

fn read_apply_receipt(path: &Path) -> Result<SourcePatchApplyReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut receipt: SourcePatchApplyReceipt = from_str(content.as_str()).with_context(|| {
        format!(
            "failed to parse source-patch apply JSON `{}`",
            path.display()
        )
    })?;
    receipt.source_patch_apply_json = path.to_path_buf();
    Ok(receipt)
}

fn validate_apply_receipt(receipt: &SourcePatchApplyReceipt) -> Result<()> {
    if receipt.schema_version != SOURCE_PATCH_APPLY_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch apply receipt has unsupported schemaVersion `{}`",
            receipt.schema_version
        );
    }
    if !receipt.source_mutation_allowed {
        anyhow::bail!("source-patch apply receipt did not record source mutation approval");
    }
    if receipt.ontology_truth {
        anyhow::bail!("source-patch apply receipt attempted to mark ontology truth");
    }
    if receipt.target_rdf_file_count != receipt.applied_targets.len() {
        anyhow::bail!(
            "source-patch apply receipt target count mismatch: receipt has {}, target list has {}",
            receipt.target_rdf_file_count,
            receipt.applied_targets.len()
        );
    }
    let target_row_count = receipt
        .applied_targets
        .iter()
        .map(|target| target.applied_row_count)
        .sum::<usize>();
    if target_row_count != receipt.apply_plan_row_count {
        anyhow::bail!(
            "source-patch apply receipt row count mismatch: receipt has {}, targets sum to {target_row_count}",
            receipt.apply_plan_row_count
        );
    }
    Ok(())
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
    Ok(format!("{:x}", hasher.finalize()))
}
