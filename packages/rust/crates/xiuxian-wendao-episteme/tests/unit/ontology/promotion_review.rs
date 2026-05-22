use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyPromotionReviewPacketRequest, write_episteme_ontology_promotion_review_packet,
};

#[test]
fn ontology_promotion_review_packet_writes_pending_review_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_clean_draft_run(temp.path(), false)?;

    let request = EpistemeOntologyPromotionReviewPacketRequest::new(temp.path());
    let report = write_episteme_ontology_promotion_review_packet(&request)?;

    assert_eq!(report.review_row_count, 3);
    assert_eq!(report.promotion_review_row_count, 3);
    assert_eq!(report.pending_review_count, 3);
    assert!(report.review_gate_passed);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let review_tsv = fs::read_to_string(&report.promotion_review_tsv)?;
    assert!(review_tsv.contains("promotion_decision"));
    assert!(review_tsv.contains("pending_review"));
    assert!(review_tsv.contains("relation.source.term"));
    assert!(review_tsv.contains("candidate.source"));
    assert!(review_tsv.contains("candidate.term"));
    assert!(review_tsv.contains("source_mutation_allowed"));
    assert!(review_tsv.contains("\tfalse\tfalse\t"));
    assert!(!review_tsv.contains("raw private text"));

    let review_org = fs::read_to_string(&report.promotion_review_org)?;
    assert!(review_org.contains(":WENDAO_KIND: ontology_promotion_review_packet"));
    assert!(review_org.contains("| pending_review_count | 3 |"));

    let review_json = fs::read_to_string(&report.promotion_review_json)?;
    assert!(review_json.contains("\"sourceMutationAllowed\": false"));
    assert!(review_json.contains("\"ontologyTruth\": false"));
    Ok(())
}

#[test]
fn ontology_promotion_review_packet_blocks_unsafe_proposal()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_clean_draft_run(temp.path(), true)?;

    let request = EpistemeOntologyPromotionReviewPacketRequest::new(temp.path());
    let Err(error) = write_episteme_ontology_promotion_review_packet(&request) else {
        return Err("unsafe proposal must block promotion review packet generation".into());
    };

    assert!(error.to_string().contains("ontologyTruth=false"));
    assert!(!temp.path().join("promotion_review.tsv").exists());
    assert!(!temp.path().join("promotion_review.org").exists());
    assert!(!temp.path().join("promotion_review.json").exists());
    Ok(())
}

fn write_clean_draft_run(
    root: &std::path::Path,
    unsafe_ontology_truth: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(root.join("rdf_draft.ttl"), "@prefix draft: <urn:test#> .\n")?;
    fs::write(
        root.join("candidate_relations.tsv"),
        "candidate_id\trelation_kind\tsource_candidate_id\ttarget_candidate_id\tsource_file_id\tsource_queue_id\textraction_run_id\tevidence_sha256\treview_status\tpromotion_status\tontology_truth\nrelation.source.term\tontology_candidate.source_artifact.suggested_object_type\tcandidate.source\tcandidate.term\tfile.source\tqueue.source\t\tsha256:source\treview_required\tblocked_pending_review\tfalse\n",
    )?;
    fs::write(
        root.join("candidate_review.tsv"),
        "record_id\trecord_kind\treview_decision\tquality_score\tevidence_strength\tissue_codes\tpromotion_precondition_met\tsource_file_id\tsource_queue_id\textraction_run_id\tsuggested_term_key\tlabel\ncandidate.term\tontology_candidate.object_term\tready_for_review\t80\tmapping_ledger\t\ttrue\t\t\t\tpolicy.document\tPolicy Term\ncandidate.source\tontology_candidate.source_artifact\tready_for_review\t80\tsource_metadata\t\ttrue\tfile.source\tqueue.source\t\tpolicy.document\tPolicy Source\nrelation.source.term\tontology_candidate.source_artifact.suggested_object_type\tready_for_review\t75\thash_provenance\t\ttrue\tfile.source\tqueue.source\t\t\t\n",
    )?;
    fs::write(
        root.join("promotion_proposal.json"),
        format!(
            "{{\n  \"schemaVersion\": \"xiuxian_wendao.episteme_ontology_rdf_draft_export.v1\",\n  \"runDir\": \"{}\",\n  \"rdfDraftTtl\": \"rdf_draft.ttl\",\n  \"promotionProposalOrg\": \"promotion_proposal.org\",\n  \"promotionProposalJson\": \"promotion_proposal.json\",\n  \"candidateObjectCount\": 2,\n  \"candidateRelationCount\": 1,\n  \"candidateEvidenceCount\": 0,\n  \"reviewRowCount\": 3,\n  \"draftResourceCount\": 3,\n  \"draftStatementCount\": 42,\n  \"reviewGatePassed\": true,\n  \"rawToRdfPromotionAllowed\": false,\n  \"ontologyTruth\": {}\n}}\n",
            root.display(),
            unsafe_ontology_truth
        ),
    )?;
    Ok(())
}
