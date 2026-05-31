use std::{collections::BTreeSet, fs};

use serde_json::Value;
use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeSearchStrategyOracleRequest, write_episteme_search_strategy_oracle,
};

use super::fixtures::{
    write_extension_source_contract_fixture, write_object_relation_review_ledgers,
};

#[test]
fn search_strategy_oracle_compiles_from_org_review_ledgers()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    write_extension_source_contract_fixture(&root)?;
    write_object_relation_review_ledgers(&root, "approved", "pending_review")?;

    let report = write_episteme_search_strategy_oracle(
        &EpistemeSearchStrategyOracleRequest::new(&root, "oracle_seed"),
        root.join("runs/search"),
    )?;

    assert_eq!(report.case_count, 1);
    assert_eq!(report.candidate_count, 7);
    assert_eq!(report.expected_selected_count, 6);
    assert_eq!(report.expected_rejected_count, 1);
    assert_eq!(report.review_ledger_count, 2);
    assert!(!report.ontology_truth);
    assert!(!report.source_mutation_allowed);
    assert!(report.cases_json.is_file());
    assert!(report.candidates_json.is_file());
    assert!(report.report_json.is_file());

    let cases: Value = serde_json::from_str(&fs::read_to_string(&report.cases_json)?)?;
    assert_eq!(
        cases["projectionSource"],
        "xiuxian-wendao-episteme.review-ledger-oracle"
    );
    assert_eq!(cases["sourceMutationAllowed"], false);
    assert_eq!(cases["ontologyTruth"], false);
    let case = &cases["cases"][0];
    assert_eq!(
        case["caseId"],
        "oracle_seed.episteme---medical-episteme-extension"
    );
    assert_eq!(case["ontologyTruth"], false);
    assert!(
        case["expectedSelectedCandidateIds"]
            .as_array()
            .ok_or("selected ids missing")?
            .iter()
            .any(|id| id == "obj.policy")
    );
    assert!(
        case["expectedRejectedCandidateIds"]
            .as_array()
            .ok_or("rejected ids missing")?
            .iter()
            .any(|id| id == "rel.policy.defines.service")
    );
    assert!(
        case["requiredEvidenceLabels"]
            .as_array()
            .ok_or("required evidence missing")?
            .iter()
            .any(|id| id == "ownership_boundary")
    );

    let candidates_json = fs::read_to_string(&report.candidates_json)?;
    let candidates: Value = serde_json::from_str(&candidates_json)?;
    assert_eq!(
        candidates["projectionSource"],
        "xiuxian-wendao-episteme.review-ledger-oracle"
    );
    assert_eq!(candidates["sourceMutationAllowed"], false);
    assert_eq!(candidates["ontologyTruth"], false);
    assert!(candidates_json.contains("\"candidateKind\": \"object_instance\""));
    assert!(candidates_json.contains("\"candidateKind\": \"instance_relation\""));
    assert!(candidates_json.contains("\"candidateKind\": \"oracle_route_support\""));
    assert!(candidates_json.contains("\"expectedAction\": \"select\""));
    assert!(candidates_json.contains("\"expectedAction\": \"reject\""));
    assert!(candidates_json.contains("\"routeRole\": \"validation\""));
    assert!(candidates_json.contains("\"requiredEvidence\": \"page_index_seed\""));

    let candidate_ids = candidates["candidates"]
        .as_array()
        .ok_or("candidates missing")?
        .iter()
        .filter_map(|candidate| candidate["candidateId"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        candidate_ids.len(),
        candidates["candidates"]
            .as_array()
            .ok_or("candidates missing")?
            .len()
    );
    Ok(())
}

#[test]
fn search_strategy_oracle_rejects_manifest_without_review_ledgers()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    write_extension_source_contract_fixture(&root)?;
    let manifest_path = root.join("ontology/manifest.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    fs::write(
        &manifest_path,
        manifest.replace(
            r#"review_ledgers = ["10_Extension/review_ledgers/review.toml"]"#,
            "review_ledgers = []",
        ),
    )?;

    let error = match write_episteme_search_strategy_oracle(
        &EpistemeSearchStrategyOracleRequest::new(&root, "oracle_seed"),
        root.join("runs/search"),
    ) {
        Ok(report) => panic!("expected missing ledger error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("requires at least one review ledger")
    );
    Ok(())
}

#[test]
fn search_strategy_oracle_rejects_unsafe_run_id() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    write_extension_source_contract_fixture(&root)?;

    let error = match write_episteme_search_strategy_oracle(
        &EpistemeSearchStrategyOracleRequest::new(&root, "../bad"),
        root.join("runs/search"),
    ) {
        Ok(report) => panic!("expected unsafe run id error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("run_id must be ASCII"));
    Ok(())
}
