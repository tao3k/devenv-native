use std::fs;

use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralFactsReasoningFillPlanRequest,
    EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralFactsReasoningPacketRequest, EpistemeOntologyStructuralFactsRequest,
    write_episteme_ontology_structural_facts,
    write_episteme_ontology_structural_facts_reasoning_fill_plan,
    write_episteme_ontology_structural_facts_reasoning_ledger_seed,
    write_episteme_ontology_structural_facts_reasoning_packet,
};

use super::fixtures::write_structural_facts_fixture;

#[test]
fn structural_facts_reasoning_fill_plan_writes_workflow_items()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let ledger_seed_json = write_reasoning_ledger_seed_fixture(temp.path())?;

    let request = EpistemeOntologyStructuralFactsReasoningFillPlanRequest::new(
        &ledger_seed_json,
        "fill_plan",
    );
    let report = write_episteme_ontology_structural_facts_reasoning_fill_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.seed_row_count, 2);
    assert_eq!(report.object_fill_item_count, 1);
    assert_eq!(report.relation_fill_item_count, 1);
    assert_eq!(report.fill_item_count, 2);
    assert!(!report.execution.source_text_read);
    assert!(!report.execution.llm_executed);
    assert!(!report.execution.workflow_executed);
    assert!(!report.safety.source_mutation_allowed);
    assert!(!report.safety.rdf_mutation_allowed);
    assert!(!report.safety.ontology_truth);
    let fill_plan_tsv = fs::read_to_string(report.reasoning_fill_plan_tsv)?;
    assert!(fill_plan_tsv.contains("episteme_ontology_reasoning_fill"));
    assert!(fill_plan_tsv.contains("read_targeted_evidence_then_fill_org_proposal"));
    assert!(fill_plan_tsv.contains("object_proposal"));
    assert!(fill_plan_tsv.contains("relation_proposal"));

    Ok(())
}

#[test]
fn structural_facts_reasoning_fill_plan_preserves_service_catalog_targeting()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let ledger_seed_json = write_reasoning_ledger_seed_fixture(temp.path())?;
    let mut seed_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&ledger_seed_json)?)?;
    seed_json[0]["seedKind"] = serde_json::json!("service_catalog_review_slot");
    seed_json[0]["evidenceTargetIntent"] = serde_json::json!("service_catalog_extraction");
    seed_json[0]["evidenceStructureHint"] = serde_json::json!("document_root:service_catalog");
    seed_json
        .as_array_mut()
        .ok_or("seed json must be an array")?
        .truncate(1);
    fs::write(&ledger_seed_json, serde_json::to_string_pretty(&seed_json)?)?;

    let request = EpistemeOntologyStructuralFactsReasoningFillPlanRequest::new(
        &ledger_seed_json,
        "fill_plan",
    );
    let report = write_episteme_ontology_structural_facts_reasoning_fill_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.object_fill_item_count, 0);
    assert_eq!(report.relation_fill_item_count, 0);
    assert_eq!(report.service_catalog_fill_item_count, 1);
    assert_eq!(report.object_instance_fill_item_count, 0);
    assert_eq!(report.fill_item_count, 1);
    let fill_plan_tsv = fs::read_to_string(report.reasoning_fill_plan_tsv)?;
    assert!(fill_plan_tsv.contains("service_catalog_review"));
    assert!(fill_plan_tsv.contains("service_catalog_extraction"));
    assert!(!fill_plan_tsv.contains("object_proposal\t"));

    Ok(())
}

#[test]
fn structural_facts_reasoning_fill_plan_rejects_mutating_seed_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let ledger_seed_json = write_reasoning_ledger_seed_fixture(temp.path())?;
    let mut seed_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&ledger_seed_json)?)?;
    seed_json[0]["sourceMutationAllowed"] = serde_json::json!(true);
    fs::write(&ledger_seed_json, serde_json::to_string_pretty(&seed_json)?)?;

    let request = EpistemeOntologyStructuralFactsReasoningFillPlanRequest::new(
        &ledger_seed_json,
        "fill_plan",
    );
    let error = match write_episteme_ontology_structural_facts_reasoning_fill_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    ) {
        Ok(report) => panic!("expected source mutation error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("attempted to allow source mutation")
    );
    Ok(())
}

fn write_reasoning_ledger_seed_fixture(
    temp_root: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let root = temp_root.join("episteme");
    let corpus_root = temp_root.join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_facts(
        &EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let packet_report = write_episteme_ontology_structural_facts_reasoning_packet(
        &EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
            &structural_report.structural_facts_json,
            "reasoning_packet",
        ),
        root.join("runs/ontology-generation"),
    )?;
    let ledger_seed_report = write_episteme_ontology_structural_facts_reasoning_ledger_seed(
        &EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest::new(
            &packet_report.reasoning_packet_json,
            "reasoning_ledger_seed",
        ),
        root.join("runs/ontology-generation"),
    )?;
    Ok(ledger_seed_report.reasoning_ledger_seed_json)
}
