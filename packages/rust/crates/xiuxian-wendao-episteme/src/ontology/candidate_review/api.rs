//! Public candidate review entrypoint.

use anyhow::Result;

use super::{
    read::{read_candidate_evidence, read_candidate_objects, read_candidate_relations},
    review::build_review_rows,
    types::{
        EVIDENCE_TSV, EpistemeOntologyCandidateReviewReport,
        EpistemeOntologyCandidateReviewRequest, OBJECTS_TSV, QUALITY_REPORT_JSON, RELATIONS_TSV,
        REVIEW_ORG, REVIEW_SCHEMA_VERSION, REVIEW_TSV,
    },
    write::{write_json, write_review_org, write_review_tsv},
};

/// Review generated candidate TSVs and write deterministic quality artifacts.
///
/// # Errors
///
/// Returns an error when required TSV files are missing, malformed, or cannot
/// be read/written.
pub fn review_episteme_ontology_candidates(
    request: &EpistemeOntologyCandidateReviewRequest,
) -> Result<EpistemeOntologyCandidateReviewReport> {
    let run_dir = request.run_dir();
    let objects = read_candidate_objects(run_dir.join(OBJECTS_TSV).as_path())?;
    let relations = read_candidate_relations(run_dir.join(RELATIONS_TSV).as_path())?;
    let evidence = read_candidate_evidence(run_dir.join(EVIDENCE_TSV).as_path())?;
    let (review_rows, metrics) = build_review_rows(&objects, &relations, &evidence);
    let review_tsv = run_dir.join(REVIEW_TSV);
    let review_org = run_dir.join(REVIEW_ORG);
    let quality_report_json = run_dir.join(QUALITY_REPORT_JSON);
    write_review_tsv(review_tsv.as_path(), &review_rows)?;
    write_review_org(review_org.as_path(), &review_rows, &metrics)?;
    let report = EpistemeOntologyCandidateReviewReport {
        schema_version: REVIEW_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        candidate_review_tsv: review_tsv,
        candidate_review_org: review_org,
        quality_report_json,
        candidate_object_count: objects.len(),
        candidate_relation_count: relations.len(),
        candidate_evidence_count: evidence.len(),
        review_row_count: review_rows.len(),
        duplicate_candidate_id_count: metrics.duplicate_candidate_ids.len(),
        missing_relation_reference_count: metrics.missing_relation_reference_count,
        promotion_flag_violation_count: metrics.promotion_flag_violation_count,
        ontology_truth_violation_count: metrics.ontology_truth_violation_count,
        malformed_row_count: metrics.malformed_row_count,
        promotion_precondition_met_count: metrics.promotion_precondition_met_count,
        blocked_invalid_count: metrics.blocked_invalid_count,
        needs_evidence_count: metrics.needs_evidence_count,
        review_gate_passed: metrics.blocked_invalid_count == 0,
    };
    write_json(report.quality_report_json.as_path(), &report)?;
    Ok(report)
}
