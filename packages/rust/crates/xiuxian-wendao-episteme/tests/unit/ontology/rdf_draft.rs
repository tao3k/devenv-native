use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyRdfDraftExportRequest, export_episteme_ontology_rdf_draft,
};

#[test]
fn ontology_rdf_draft_export_writes_review_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_reviewed_candidate_run(temp.path(), true)?;

    let request = EpistemeOntologyRdfDraftExportRequest::new(temp.path());
    let report = export_episteme_ontology_rdf_draft(&request)?;

    assert_eq!(report.candidate_object_count, 3);
    assert_eq!(report.candidate_relation_count, 2);
    assert_eq!(report.candidate_evidence_count, 1);
    assert_eq!(report.review_row_count, 6);
    assert_eq!(report.draft_resource_count, 6);
    assert!(report.draft_statement_count > report.draft_resource_count);
    assert!(report.review_gate_passed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert!(!report.ontology_truth);

    let rdf = fs::read_to_string(&report.rdf_draft_ttl)?;
    assert!(rdf.contains("wdp:OntologyCandidate"));
    assert!(rdf.contains("wdp:OntologyCandidateRelation"));
    assert!(rdf.contains("wdp:OntologyCandidateEvidence"));
    assert!(rdf.contains("wdp:candidateId \"candidate.term\""));
    assert!(rdf.contains("rdfs:label \"Policy Term\""));
    assert!(rdf.contains("wdp:proposalStatus \"draft_pending_review\""));
    assert!(rdf.contains("wdp:ontologyTruth \"false\"^^xsd:boolean"));
    assert!(!rdf.contains("raw private text"));

    let proposal_org = fs::read_to_string(&report.promotion_proposal_org)?;
    assert!(proposal_org.contains(":WENDAO_KIND: ontology_promotion_proposal"));
    assert!(proposal_org.contains("| draft_resource_count | 6 |"));

    let proposal_json = fs::read_to_string(&report.promotion_proposal_json)?;
    assert!(proposal_json.contains("\"ontologyTruth\": false"));
    assert!(proposal_json.contains("\"rawToRdfPromotionAllowed\": false"));
    Ok(())
}

#[test]
fn ontology_rdf_draft_export_requires_passed_review_gate() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    write_reviewed_candidate_run(temp.path(), false)?;

    let request = EpistemeOntologyRdfDraftExportRequest::new(temp.path());
    let Err(error) = export_episteme_ontology_rdf_draft(&request) else {
        return Err("failed review gate must block RDF draft export".into());
    };

    assert!(error.to_string().contains("reviewGatePassed=true"));
    assert!(!temp.path().join("rdf_draft.ttl").exists());
    assert!(!temp.path().join("promotion_proposal.org").exists());
    assert!(!temp.path().join("promotion_proposal.json").exists());
    Ok(())
}

fn write_reviewed_candidate_run(
    root: &std::path::Path,
    review_gate_passed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
    fs::write(
        root.join("candidate_review.tsv"),
        "record_id\trecord_kind\treview_decision\tquality_score\tevidence_strength\tissue_codes\tpromotion_precondition_met\tsource_file_id\tsource_queue_id\textraction_run_id\tsuggested_term_key\tlabel\ncandidate.term\tontology_candidate.object_term\tready_for_review\t80\tmapping_ledger\t\ttrue\t\t\t\tpolicy.document\tPolicy Term\ncandidate.source\tontology_candidate.source_artifact\tready_for_review\t80\tsource_metadata\t\ttrue\tfile.source\tqueue.source\t\tpolicy.document\tPolicy Source\ncandidate.evidence\tontology_candidate.extraction_evidence\tready_for_review\t90\textracted_text_hash\t\ttrue\tfile.source\tqueue.source\tseed\t\tPolicy Source evidence\nrelation.source.term\tontology_candidate.source_artifact.suggested_object_type\tready_for_review\t75\thash_provenance\t\ttrue\tfile.source\tqueue.source\t\t\t\nrelation.evidence.source\tontology_candidate.extraction_evidence.supports_source_artifact\tready_for_review\t75\thash_provenance\t\ttrue\tfile.source\tqueue.source\tseed\t\t\nevidence:candidate.evidence\tontology_candidate.extraction_cache\tready_for_review\t85\textracted_text_hash\t\ttrue\tfile.source\tqueue.source\tseed\t\t\n",
    )?;
    fs::write(
        root.join("quality_report.json"),
        format!(
            "{{\n  \"schemaVersion\": \"xiuxian_wendao.episteme_ontology_candidate_review.v1\",\n  \"runDir\": \"{}\",\n  \"candidateReviewTsv\": \"{}\",\n  \"qualityReportJson\": \"{}\",\n  \"candidateObjectCount\": 3,\n  \"candidateRelationCount\": 2,\n  \"candidateEvidenceCount\": 1,\n  \"reviewRowCount\": 6,\n  \"duplicateCandidateIdCount\": 0,\n  \"missingRelationReferenceCount\": 0,\n  \"promotionFlagViolationCount\": 0,\n  \"ontologyTruthViolationCount\": 0,\n  \"malformedRowCount\": 0,\n  \"promotionPreconditionMetCount\": 6,\n  \"blockedInvalidCount\": 0,\n  \"needsEvidenceCount\": 0,\n  \"reviewGatePassed\": {}\n}}\n",
            root.display(),
            root.join("candidate_review.tsv").display(),
            root.join("quality_report.json").display(),
            review_gate_passed
        ),
    )?;
    Ok(())
}
