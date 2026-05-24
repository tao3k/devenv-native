use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyPromotionApplyPlanRequest, EpistemeOntologyPromotionReviewPacketRequest,
    EpistemeOntologyQianjiReviewCandidateImportRequest, EpistemeOntologyRdfDraftExportRequest,
    export_episteme_ontology_rdf_draft, import_episteme_ontology_qianji_review_candidates,
    write_episteme_ontology_promotion_apply_plan, write_episteme_ontology_promotion_review_packet,
};
use xiuxian_wendao_parsers::compile_org_ontology_authoring_document;

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

#[test]
fn qianji_review_candidate_import_rejects_missing_canonical_review()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    fs::write(
        &artifact,
        r#"{
  "schema": "qianji.openai_compatible_llm_response.v1",
  "content": "{}"
}"#,
    )?;

    let error = match import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    ) {
        Ok(report) => panic!("expected missing review error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("has no episteme_review"));
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
fn qianji_review_candidate_import_rejects_zero_candidate_review_without_blockers()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    let mut json: serde_json::Value =
        serde_json::from_str(qianji_zero_candidate_review_artifact())?;
    json["episteme_review"]["blockers"] = serde_json::json!([]);
    fs::write(&artifact, serde_json::to_string_pretty(&json)?)?;

    let error = match import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    ) {
        Ok(report) => panic!("expected missing blocker error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("has no candidatePatches and no blockers")
    );
    Ok(())
}

#[test]
fn qianji_review_candidate_import_rejects_patch_group_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    let mut json: serde_json::Value = serde_json::from_str(qianji_review_artifact())?;
    json["episteme_review"]["candidatePatches"][0]["targetLedgerFieldGroup"] =
        serde_json::json!("relation_proposal");
    fs::write(&artifact, serde_json::to_string_pretty(&json)?)?;

    let error = match import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    ) {
        Ok(report) => panic!("expected group mismatch error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("patch targetLedgerFieldGroup does not match")
    );
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

#[test]
fn qianji_review_candidate_import_rejects_unsupported_patch_kind()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let artifact = temp.path().join("qianji_review.json");
    let mut json: serde_json::Value = serde_json::from_str(qianji_review_artifact())?;
    json["episteme_review"]["candidatePatches"][0]["patchKind"] =
        serde_json::json!("ledger_candidate");
    fs::write(&artifact, serde_json::to_string_pretty(&json)?)?;

    let error = match import_episteme_ontology_qianji_review_candidates(
        &EpistemeOntologyQianjiReviewCandidateImportRequest::new(temp.path())
            .with_review_artifact(&artifact),
    ) {
        Ok(report) => panic!("expected unsupported patch error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported patch kind"));
    Ok(())
}

fn qianji_zero_candidate_review_artifact() -> &'static str {
    r#"{
  "schema": "qianji.openai_compatible_llm_response.v1",
  "model": "deepseek/deepseek-v4-pro",
  "activity_id": "activity.episteme_ontology_reasoning_fill.test",
  "content": "{}",
  "episteme_review": {
    "schema": "xiuxian.wendao.episteme.reasoning_fill_review.v1",
    "status": "review_only",
    "fillItemId": "structural_facts.reasoning_fill_plan.test",
    "targetLedgerFieldGroup": "object_proposal",
    "reviewSummary": "Evidence is not enough for a safe ObjectType proposal.",
    "candidatePatchCount": 0,
    "candidatePatches": [],
    "blockers": [
      "Evidence describes procedural service items rather than object type boundaries",
      "No stable property set is explicit in the source evidence"
    ],
    "rdfMutation": false
  }
}"#
}

fn qianji_review_artifact() -> &'static str {
    r#"{
  "schema": "qianji.openai_compatible_llm_response.v1",
  "model": "deepseek/deepseek-v4-pro",
  "activity_id": "activity.episteme_ontology_reasoning_fill.test",
  "content": "{}",
  "episteme_review": {
    "schema": "xiuxian.wendao.episteme.reasoning_fill_review.v1",
    "status": "review_only",
    "fillItemId": "structural_facts.reasoning_fill_plan.test",
    "targetLedgerFieldGroup": "object_proposal",
    "reviewSummary": "Evidence supports one object candidate.",
    "candidatePatchCount": 1,
    "candidatePatches": [
      {
        "patchKind": "object_model_object_type_candidate",
        "fillItemId": "structural_facts.reasoning_fill_plan.test",
        "targetLedgerFieldGroup": "object_proposal",
        "objectType": {
          "domain": "episteme://medical-extension/ltc",
          "apiName": "LtcServiceCatalog",
          "displayName": "Shanghai LTC Service Catalog",
          "pluralDisplayName": "Shanghai LTC Service Catalogs",
          "status": "preview",
          "rdfClass": "https://wendao.ai/private/medical/ltc#ServiceCatalog",
          "primaryKey": ["sourceId"],
          "displayNameProperty": "name",
          "titleProperty": "name",
          "interfaces": [],
          "visibility": "private"
        },
        "sourceEvidence": [
          {
            "fileId": "ltc.file.policy.001",
            "relativePath": "policy/source.doc",
            "quote": "raw private quote body",
            "reason": "supports this candidate"
          }
        ],
        "confidence": "high",
        "reviewNotes": "Review-only candidate."
      }
    ],
    "blockers": [],
    "rdfMutation": false
  },
  "provider_response": {}
}"#
}
