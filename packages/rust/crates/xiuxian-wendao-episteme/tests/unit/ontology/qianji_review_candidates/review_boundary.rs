use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyPromotionApplyPlanRequest, EpistemeOntologyPromotionReviewPacketRequest,
    EpistemeOntologyQianjiReviewCandidateImportRequest, EpistemeOntologyRdfDraftExportRequest,
    export_episteme_ontology_rdf_draft, import_episteme_ontology_qianji_review_candidates,
    write_episteme_ontology_promotion_apply_plan, write_episteme_ontology_promotion_review_packet,
};

use super::fixtures::qianji_review_artifact;

#[test]
fn qianji_review_candidate_import_reaches_non_mutating_promotion_review_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    fs::write(&artifact, qianji_review_artifact())?;

    let import_report = import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    )?;
    assert!(import_report.candidate_review.review_gate_passed);

    let rdf_report = export_episteme_ontology_rdf_draft(
        &EpistemeOntologyRdfDraftExportRequest::new(temp.path()),
    )?;
    assert_eq!(rdf_report.candidate_object_count, 1);
    assert_eq!(rdf_report.candidate_evidence_count, 1);
    assert_eq!(rdf_report.review_row_count, 2);
    assert_eq!(rdf_report.draft_resource_count, 2);
    assert!(rdf_report.draft_statement_count > 0);
    assert!(rdf_report.review_gate_passed);
    assert!(!rdf_report.raw_to_rdf_promotion_allowed);
    assert!(!rdf_report.ontology_truth);

    let rdf_draft = fs::read_to_string(&rdf_report.rdf_draft_ttl)?;
    assert!(rdf_draft.contains("wdp:proposalStatus \"draft_pending_review\""));
    assert!(rdf_draft.contains("wdp:ontologyTruth \"false\"^^xsd:boolean"));
    assert!(!rdf_draft.contains("raw private quote body"));

    let promotion_review = write_episteme_ontology_promotion_review_packet(
        &EpistemeOntologyPromotionReviewPacketRequest::new(temp.path()),
    )?;
    assert_eq!(promotion_review.review_row_count, 2);
    assert_eq!(promotion_review.promotion_review_row_count, 2);
    assert_eq!(promotion_review.pending_review_count, 2);
    assert!(promotion_review.review_gate_passed);
    assert!(!promotion_review.source_mutation_allowed);
    assert!(!promotion_review.ontology_truth);

    let apply_plan = write_episteme_ontology_promotion_apply_plan(
        &EpistemeOntologyPromotionApplyPlanRequest::new(temp.path()),
    )?;
    assert_eq!(apply_plan.promotion_review_row_count, 2);
    assert_eq!(apply_plan.pending_review_count, 2);
    assert_eq!(apply_plan.approved_count, 0);
    assert_eq!(apply_plan.apply_plan_row_count, 0);
    assert!(!apply_plan.source_mutation_allowed);
    assert!(!apply_plan.ontology_truth);
    Ok(())
}
