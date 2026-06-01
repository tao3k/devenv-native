//! Public Qianji review candidate import entrypoint.

use std::fs;

use anyhow::{Context, Result, bail};

use super::{
    build::append_review_candidates,
    read::read_review_artifact,
    types::{
        EVIDENCE_TSV, EpistemeOntologyQianjiReviewCandidateImportReport,
        EpistemeOntologyQianjiReviewCandidateImportRequest, IMPORT_REPORT_JSON,
        IMPORT_SCHEMA_VERSION, OBJECTS_TSV, QianjiReviewCandidateImportBuild, RELATIONS_TSV,
    },
    write::{write_evidence_tsv, write_json, write_objects_tsv, write_relations_tsv},
};
use crate::ontology::candidate_review::{
    EpistemeOntologyCandidateReviewRequest, review_episteme_ontology_candidates,
};

/// Import Qianji review artifacts into candidate rows and run the review gate.
///
/// # Errors
///
/// Returns an error when a review artifact is missing, malformed, attempts RDF
/// mutation, contains unsupported patch kinds, or generated review artifacts
/// cannot be written.
pub fn import_episteme_ontology_qianji_review_candidates(
    request: &EpistemeOntologyQianjiReviewCandidateImportRequest,
) -> Result<EpistemeOntologyQianjiReviewCandidateImportReport> {
    import_episteme_ontology_qianji_review_candidates_impl(request)
}

fn import_episteme_ontology_qianji_review_candidates_impl(
    request: &EpistemeOntologyQianjiReviewCandidateImportRequest,
) -> Result<EpistemeOntologyQianjiReviewCandidateImportReport> {
    if request.review_artifacts.is_empty() {
        bail!("Qianji review candidate import requires at least one review artifact");
    }
    let build = request.review_artifacts.iter().try_fold(
        QianjiReviewCandidateImportBuild::default(),
        |mut build, artifact_path| {
            let review = read_review_artifact(artifact_path.as_path())?;
            build.review_blocker_count += review.blockers.len();
            if review.candidate_patch_count == 0 {
                build.zero_candidate_review_count += 1;
            }
            append_review_candidates(
                &review,
                artifact_path.as_path(),
                &mut build.objects,
                &mut build.relations,
                &mut build.evidence,
            )?;
            Ok::<_, anyhow::Error>(build)
        },
    )?;

    fs::create_dir_all(request.run_dir()).with_context(|| {
        format!(
            "failed to create Qianji review candidate run dir `{}`",
            request.run_dir().display()
        )
    })?;
    let objects_tsv = request.run_dir().join(OBJECTS_TSV);
    let relations_tsv = request.run_dir().join(RELATIONS_TSV);
    let evidence_tsv = request.run_dir().join(EVIDENCE_TSV);
    let import_report_json = request.run_dir().join(IMPORT_REPORT_JSON);
    write_objects_tsv(objects_tsv.as_path(), &build.objects)?;
    write_relations_tsv(relations_tsv.as_path(), &build.relations)?;
    write_evidence_tsv(evidence_tsv.as_path(), &build.evidence)?;
    let candidate_review = review_episteme_ontology_candidates(
        &EpistemeOntologyCandidateReviewRequest::new(request.run_dir()),
    )?;
    let report = EpistemeOntologyQianjiReviewCandidateImportReport {
        schema_version: IMPORT_SCHEMA_VERSION,
        run_dir: request.run_dir().to_path_buf(),
        qianji_review_artifacts: request.review_artifacts.clone(),
        candidate_objects_tsv: objects_tsv,
        candidate_relations_tsv: relations_tsv,
        candidate_evidence_tsv: evidence_tsv,
        import_report_json,
        candidate_object_count: build.objects.len(),
        candidate_relation_count: build.relations.len(),
        candidate_evidence_count: build.evidence.len(),
        zero_candidate_review_count: build.zero_candidate_review_count,
        review_blocker_count: build.review_blocker_count,
        candidate_review,
        ontology_truth: false,
        raw_to_rdf_promotion_allowed: false,
    };
    write_json(report.import_report_json.as_path(), &report)?;
    Ok(report)
}
