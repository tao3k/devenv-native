//! `RDF` draft export pipeline for reviewed ontology candidates.

use anyhow::Result;

use super::{
    EpistemeOntologyRdfDraftExportReport, EpistemeOntologyRdfDraftExportRequest,
    input::read_draft_inputs,
    model::{
        ONTOLOGY_TRUTH, PROMOTION_PROPOSAL_JSON, PROMOTION_PROPOSAL_ORG,
        RAW_TO_RDF_PROMOTION_ALLOWED, RDF_DRAFT_SCHEMA_VERSION, RDF_DRAFT_TTL,
    },
    render::render_rdf_draft,
    validation::validate_review_gate,
    writer::{write_json, write_promotion_proposal_org, write_string},
};

/// Export review-gated RDF draft and promotion proposal artifacts.
///
/// # Errors
///
/// Returns an error when the review gate report is missing, failed, inconsistent
/// with candidate artifacts, or when draft artifacts cannot be written.
pub fn export_episteme_ontology_rdf_draft(
    request: &EpistemeOntologyRdfDraftExportRequest,
) -> Result<EpistemeOntologyRdfDraftExportReport> {
    let run_dir = request.run_dir();
    let inputs = read_draft_inputs(run_dir)?;
    validate_review_gate(&inputs)?;

    let render = render_rdf_draft(&inputs)?;
    let rdf_draft_ttl = run_dir.join(RDF_DRAFT_TTL);
    let promotion_proposal_org = run_dir.join(PROMOTION_PROPOSAL_ORG);
    let promotion_proposal_json = run_dir.join(PROMOTION_PROPOSAL_JSON);
    write_string(rdf_draft_ttl.as_path(), render.ttl.as_str())?;

    let report = EpistemeOntologyRdfDraftExportReport {
        schema_version: RDF_DRAFT_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        rdf_draft_ttl,
        promotion_proposal_org,
        promotion_proposal_json,
        candidate_object_count: inputs.objects.len(),
        candidate_relation_count: inputs.relations.len(),
        candidate_evidence_count: inputs.evidence.len(),
        review_row_count: inputs.reviews_by_id.len(),
        draft_resource_count: render.resource_count,
        draft_statement_count: render.statement_count,
        review_gate_passed: inputs.quality.review_gate_passed,
        raw_to_rdf_promotion_allowed: RAW_TO_RDF_PROMOTION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    };
    write_promotion_proposal_org(report.promotion_proposal_org.as_path(), &report)?;
    write_json(report.promotion_proposal_json.as_path(), &report)?;
    Ok(report)
}
