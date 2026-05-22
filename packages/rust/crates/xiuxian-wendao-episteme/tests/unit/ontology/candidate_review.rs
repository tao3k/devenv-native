use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyCandidateReviewRequest, review_episteme_ontology_candidates,
};

#[test]
fn ontology_candidate_review_writes_quality_gate_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_valid_candidate_run(temp.path())?;

    let request = EpistemeOntologyCandidateReviewRequest::new(temp.path());
    let report = review_episteme_ontology_candidates(&request)?;

    assert_eq!(report.candidate_object_count, 3);
    assert_eq!(report.candidate_relation_count, 2);
    assert_eq!(report.candidate_evidence_count, 1);
    assert_eq!(report.review_row_count, 6);
    assert_eq!(report.blocked_invalid_count, 0);
    assert!(report.review_gate_passed);
    assert!(report.promotion_precondition_met_count > 0);

    let review_tsv = fs::read_to_string(&report.candidate_review_tsv)?;
    assert!(review_tsv.contains("ready_for_review"));
    assert!(review_tsv.contains("mapping_ledger"));
    assert!(review_tsv.contains("extracted_text_hash"));
    assert!(!review_tsv.contains("raw private text"));

    let quality_report = fs::read_to_string(&report.quality_report_json)?;
    assert!(quality_report.contains("\"reviewGatePassed\": true"));
    Ok(())
}

#[test]
fn ontology_candidate_review_reports_invalid_rows() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_valid_candidate_run(temp.path())?;
    fs::write(
        temp.path().join("candidate_relations.tsv"),
        "candidate_id\trelation_kind\tsource_candidate_id\ttarget_candidate_id\tsource_file_id\tsource_queue_id\textraction_run_id\tevidence_sha256\treview_status\tpromotion_status\tontology_truth\nrelation.bad\tcandidate.link\tcandidate.source\tmissing.target\tfile.source\tqueue.source\tseed\tsha256:bad\treview_required\tblocked_pending_review\tfalse\n",
    )?;
    fs::write(
        temp.path().join("candidate_objects.tsv"),
        "candidate_id\tcandidate_kind\tstatus\tlabel\tsuggested_term_key\tsuggested_term_label\tsource_file_id\tsource_queue_id\tsource_path\tcategory\tlanguage\textraction_route\textraction_run_id\tsource_sha256\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\traw_to_rdf_promotion_allowed\tontology_truth\ncandidate.term\tontology_candidate.object_term\tcandidate\tPolicy Term\tpolicy.document\tPolicy Term\t\t\tmapping.org\tmapping\ten-US\tmapping_ledger\t\tsha256:source\tsha256:term\t0\treview_required\tblocked_pending_review\ttrue\tfalse\ncandidate.term\tontology_candidate.object_term\tcandidate\tPolicy Term Duplicate\tpolicy.document\tPolicy Term\t\t\tmapping.org\tmapping\ten-US\tmapping_ledger\t\tsha256:source\tsha256:term2\t0\treview_required\tblocked_pending_review\tfalse\tfalse\n",
    )?;

    let request = EpistemeOntologyCandidateReviewRequest::new(temp.path());
    let report = review_episteme_ontology_candidates(&request)?;

    assert!(!report.review_gate_passed);
    assert_eq!(report.duplicate_candidate_id_count, 1);
    assert_eq!(report.missing_relation_reference_count, 1);
    assert_eq!(report.promotion_flag_violation_count, 1);
    assert!(report.blocked_invalid_count >= 2);

    let review_tsv = fs::read_to_string(&report.candidate_review_tsv)?;
    assert!(review_tsv.contains("duplicate_candidate_id"));
    assert!(review_tsv.contains("missing_relation_reference"));
    assert!(review_tsv.contains("promotion_flag_violation"));
    Ok(())
}

fn write_valid_candidate_run(root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        root.join("candidate_objects.tsv"),
        "candidate_id\tcandidate_kind\tstatus\tlabel\tsuggested_term_key\tsuggested_term_label\tsource_file_id\tsource_queue_id\tsource_path\tcategory\tlanguage\textraction_route\textraction_run_id\tsource_sha256\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\traw_to_rdf_promotion_allowed\tontology_truth\ncandidate.term\tontology_candidate.object_term\tcandidate\tPolicy Term\tpolicy.document\tPolicy Term\t\t\tmapping.org\tmapping\ten-US\tmapping_ledger\t\tsha256:source\tsha256:term\t0\treview_required\tblocked_pending_review\tfalse\tfalse\ncandidate.source\tontology_candidate.source_artifact\tcandidate\tPolicy Source\tpolicy.document\tPolicy Term\tfile.source\tqueue.source\tdocs/policy.pdf\tpolicy\ten-US\tdocument_text_evidence\t\tsha256:source\tsha256:source\t0\treview_required\tblocked_pending_review\tfalse\tfalse\ncandidate.evidence\tontology_candidate.extraction_evidence\tcandidate\tPolicy Source evidence\tpolicy.document\tPolicy Term\tfile.source\tqueue.source\tdocs/policy.pdf\tpolicy\ten-US\tdocument_text_evidence\tseed\tsha256:source\tsha256:text\t16\treview_required\tblocked_pending_review\tfalse\tfalse\n",
    )?;
    fs::write(
        root.join("candidate_relations.tsv"),
        "candidate_id\trelation_kind\tsource_candidate_id\ttarget_candidate_id\tsource_file_id\tsource_queue_id\textraction_run_id\tevidence_sha256\treview_status\tpromotion_status\tontology_truth\nrelation.source.term\tontology_candidate.source_artifact.suggested_object_type\tcandidate.source\tcandidate.term\tfile.source\tqueue.source\t\tsha256:source\treview_required\tblocked_pending_review\tfalse\nrelation.evidence.source\tontology_candidate.extraction_evidence.supports_source_artifact\tcandidate.evidence\tcandidate.source\tfile.source\tqueue.source\tseed\tsha256:text\treview_required\tblocked_pending_review\tfalse\n",
    )?;
    fs::write(
        root.join("candidate_evidence.tsv"),
        "evidence_id\tevidence_kind\tsource_file_id\tsource_queue_id\tsource_path\tsource_sha256\textraction_run_id\tcache_output_path\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\tontology_truth\nevidence:candidate.evidence\tontology_candidate.extraction_cache\tfile.source\tqueue.source\tdocs/policy.pdf\tsha256:source\tseed\truns/extraction/seed/outputs/queue.source.json\tsha256:text\t16\treview_required\tblocked_pending_review\tfalse\n",
    )?;
    Ok(())
}
