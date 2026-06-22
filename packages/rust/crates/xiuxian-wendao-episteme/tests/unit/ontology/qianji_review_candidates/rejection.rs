use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyQianjiReviewCandidateImportRequest,
    import_episteme_ontology_qianji_review_candidates,
};

use super::fixtures::{qianji_review_artifact, qianji_zero_candidate_review_artifact};

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
