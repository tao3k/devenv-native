//! `Episteme` ontology candidate generation pipeline.

use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::{
    EpistemeOntologyCandidateGenerationReport, EpistemeOntologyCandidateGenerationRequest,
    identifiers::validate_run_id,
    io::write_json,
    model::{
        CANDIDATE_GENERATION_SCHEMA, CandidateGenerationInputs, CandidateGenerationOutputPaths,
        CandidateGenerationReceipt, ONTOLOGY_TRUTH, RAW_TO_RDF_PROMOTION_ALLOWED,
    },
    read_model::write_candidate_read_model,
    rows::build_candidate_rows,
    writing::{
        write_candidate_evidence_tsv, write_candidate_objects_tsv, write_candidate_relations_tsv,
        write_review_ledger_org,
    },
};

/// Generate review-gated ontology candidates from source-contract facts and
/// optional extraction cache evidence.
///
/// # Errors
///
/// Returns an error when the Episteme source contract cannot be resolved, the
/// run id is unsafe, extraction cache outputs are malformed, or output files
/// cannot be written.
pub fn generate_episteme_ontology_candidates(
    request: &EpistemeOntologyCandidateGenerationRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyCandidateGenerationReport> {
    validate_run_id(request.run_id.as_str())?;
    let inputs = CandidateGenerationInputs::load(request)?;
    let candidates = build_candidate_rows(&inputs);
    let paths = CandidateGenerationOutputPaths::new(run_root.as_ref(), request.run_id.as_str());
    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;

    write_candidate_objects_tsv(paths.objects_tsv.as_path(), &candidates.objects)?;
    write_candidate_relations_tsv(paths.relations_tsv.as_path(), &candidates.relations)?;
    write_candidate_evidence_tsv(paths.evidence_tsv.as_path(), &candidates.evidence)?;
    write_candidate_read_model(&paths, &candidates)?;
    write_review_ledger_org(paths.review_ledger_org.as_path(), &inputs, &candidates)?;
    write_json(
        paths.receipt_json.as_path(),
        &receipt_for(request, &inputs, &candidates),
    )?;

    Ok(EpistemeOntologyCandidateGenerationReport {
        schema_version: CANDIDATE_GENERATION_SCHEMA,
        run_id: request.run_id.clone(),
        run_dir: paths.run_dir,
        candidate_objects_tsv: paths.objects_tsv,
        candidate_relations_tsv: paths.relations_tsv,
        candidate_evidence_tsv: paths.evidence_tsv,
        candidate_objects_parquet: paths.objects_parquet,
        candidate_relations_parquet: paths.relations_parquet,
        candidate_evidence_parquet: paths.evidence_parquet,
        review_ledger_org: paths.review_ledger_org,
        receipt_json: paths.receipt_json,
        domain: inputs.domain,
        source_file_count: inputs.files.len(),
        mapping_term_count: inputs.mapping_terms.len(),
        extraction_evidence_count: inputs.cache_evidence.len(),
        candidate_object_count: candidates.objects.len(),
        candidate_relation_count: candidates.relations.len(),
        candidate_evidence_count: candidates.evidence.len(),
        raw_to_rdf_promotion_allowed: RAW_TO_RDF_PROMOTION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    })
}

fn receipt_for(
    request: &EpistemeOntologyCandidateGenerationRequest,
    inputs: &CandidateGenerationInputs,
    candidates: &super::model::CandidateRows,
) -> CandidateGenerationReceipt {
    CandidateGenerationReceipt {
        schema_version: CANDIDATE_GENERATION_SCHEMA,
        run_id: request.run_id.clone(),
        domain: inputs.domain.clone(),
        source_revision: inputs.source_revision.clone(),
        extraction_run_ids: request.extraction_run_ids.clone(),
        source_file_count: inputs.files.len(),
        mapping_term_count: inputs.mapping_terms.len(),
        extraction_evidence_count: inputs.cache_evidence.len(),
        candidate_object_count: candidates.objects.len(),
        candidate_relation_count: candidates.relations.len(),
        candidate_evidence_count: candidates.evidence.len(),
        raw_to_rdf_promotion_allowed: RAW_TO_RDF_PROMOTION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}
