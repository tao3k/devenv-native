use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyQianjiReviewCandidateImportRequest,
    import_episteme_ontology_qianji_review_candidates,
};
use xiuxian_wendao_parsers::compile_org_ontology_authoring_document;

use super::fixtures::{qianji_review_artifact, qianji_zero_candidate_review_artifact};

#[test]
fn qianji_review_candidate_import_writes_reviewed_candidate_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    fs::write(&artifact, qianji_review_artifact())?;

    let report = import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    )?;

    assert_eq!(report.candidate_object_count, 1);
    assert_eq!(report.candidate_relation_count, 0);
    assert_eq!(report.candidate_evidence_count, 1);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert!(!report.ontology_truth);
    assert!(report.candidate_review.review_gate_passed);
    assert_eq!(report.candidate_review.review_row_count, 2);

    let objects = fs::read_to_string(&report.candidate_objects_tsv)?;
    assert!(objects.contains("ontology_candidate.qianji_object_patch"));
    assert!(objects.contains("Shanghai LTC Service Catalog"));
    assert!(objects.contains("https://wendao.ai/private/medical/ltc#ServiceCatalog"));
    assert!(!objects.contains("raw private quote body"));

    let evidence = fs::read_to_string(&report.candidate_evidence_tsv)?;
    assert!(evidence.contains("ontology_candidate.qianji_review_evidence"));
    assert!(evidence.contains("ltc.file.policy.001"));
    assert!(!evidence.contains("raw private quote body"));

    let review_org = fs::read_to_string(&report.candidate_review.candidate_review_org)?;
    let compiled = compile_org_ontology_authoring_document(
        &review_org,
        report
            .candidate_review
            .candidate_review_org
            .display()
            .to_string(),
    )?;
    assert!(
        compiled
            .sections
            .iter()
            .flat_map(|section| section.tables.iter())
            .any(|table| table.kind == "candidate_review" && table.rows.len() == 2)
    );
    Ok(())
}

#[test]
fn qianji_review_candidate_import_accepts_zero_candidate_review_with_blockers()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    fs::write(&artifact, qianji_zero_candidate_review_artifact())?;

    let report = import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    )?;

    assert_eq!(report.candidate_object_count, 0);
    assert_eq!(report.candidate_relation_count, 0);
    assert_eq!(report.candidate_evidence_count, 0);
    assert_eq!(report.zero_candidate_review_count, 1);
    assert_eq!(report.review_blocker_count, 2);
    assert!(report.candidate_review.review_gate_passed);
    assert_eq!(report.candidate_review.review_row_count, 0);

    let objects = fs::read_to_string(&report.candidate_objects_tsv)?;
    assert!(objects.starts_with("candidate_id\tcandidate_kind\tstatus"));
    assert!(!objects.contains("ontology_candidate.qianji_object_patch"));

    let import_report = fs::read_to_string(&report.import_report_json)?;
    assert!(import_report.contains("\"zeroCandidateReviewCount\": 1"));
    assert!(import_report.contains("\"reviewBlockerCount\": 2"));
    Ok(())
}

#[test]
fn qianji_review_candidate_import_accepts_relation_patch_as_review_only_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    let mut json: serde_json::Value = serde_json::from_str(qianji_review_artifact())?;
    json["episteme_review"]["candidatePatches"][0]["patchKind"] =
        serde_json::json!("object_model_link_type_candidate");
    json["episteme_review"]["candidatePatches"][0]["targetLedgerFieldGroup"] =
        serde_json::json!("relation_proposal");
    json["episteme_review"]["candidatePatches"][0]["objectType"] = serde_json::Value::Null;
    json["episteme_review"]["candidatePatches"][0]["linkType"] = serde_json::json!({
        "apiName": "policyDefinesServiceCatalog",
        "displayName": "Policy defines service catalog",
        "rdfProperty": "https://wendao.ai/private/medical/ltc#policyDefinesServiceCatalog",
        "fromObjectType": "LtcPolicyDocument",
        "toObjectType": "LtcServiceCatalog"
    });
    json["episteme_review"]["candidatePatches"][0]["endpointObjectTypes"] = serde_json::json!([
        {
            "apiName": "LtcPolicyDocument",
            "displayName": "Shanghai LTC Policy",
            "rdfClass": "https://wendao.ai/private/medical/ltc#PolicyDocument"
        },
        {
            "apiName": "LtcServiceCatalog",
            "displayName": "Shanghai LTC Service Catalog",
            "rdfClass": "https://wendao.ai/private/medical/ltc#ServiceCatalog"
        }
    ]);
    json["episteme_review"]["targetLedgerFieldGroup"] = serde_json::json!("relation_proposal");
    fs::write(&artifact, serde_json::to_string_pretty(&json)?)?;

    let report = import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    )?;

    assert_eq!(report.candidate_object_count, 2);
    assert_eq!(report.candidate_relation_count, 1);
    assert_eq!(report.candidate_evidence_count, 1);
    assert!(report.candidate_review.review_gate_passed);
    assert_eq!(report.candidate_review.review_row_count, 4);

    let objects = fs::read_to_string(&report.candidate_objects_tsv)?;
    assert!(objects.contains("Shanghai LTC Policy"));
    assert!(objects.contains("Shanghai LTC Service Catalog"));
    assert!(!objects.contains("raw private quote body"));

    let relations = fs::read_to_string(&report.candidate_relations_tsv)?;
    assert!(
        relations.contains("https://wendao.ai/private/medical/ltc#policyDefinesServiceCatalog")
    );
    assert!(relations.contains("qianji.relation."));
    assert!(!relations.contains("raw private quote body"));
    Ok(())
}
