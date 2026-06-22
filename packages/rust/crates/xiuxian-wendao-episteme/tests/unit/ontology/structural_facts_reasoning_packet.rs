use std::fs;

use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralFactsReasoningPacketRequest, EpistemeOntologyStructuralFactsRequest,
    EpistemeOntologyStructuralFactsValidationMode, write_episteme_ontology_structural_facts,
    write_episteme_ontology_structural_facts_reasoning_packet,
};

use super::fixtures::write_structural_facts_fixture;

#[test]
fn structural_facts_reasoning_packet_writes_org_tsv_and_json()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_facts(
        &EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed")
            .with_validation_mode(EpistemeOntologyStructuralFactsValidationMode::FullHash),
        root.join("runs/structure"),
    )?;

    let request = EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
        &structural_report.structural_facts_json,
        "reasoning_seed",
    );
    let report = write_episteme_ontology_structural_facts_reasoning_packet(
        &request,
        root.join("runs/reasoning"),
    )?;

    assert_eq!(report.packet_row_count, 1);
    assert_eq!(report.selected_document_count, 1);
    assert_eq!(report.skipped_by_filter_count, 0);
    assert_eq!(report.skipped_by_limit_count, 0);
    assert!(!report.execution.source_text_read);
    assert!(!report.execution.llm_executed);
    assert!(!report.safety.source_mutation_allowed);
    assert!(!report.safety.ontology_truth);
    assert!(report.reasoning_packet_org.is_file());
    assert!(report.reasoning_packet_tsv.is_file());
    assert!(report.reasoning_packet_json.is_file());
    let packet_tsv = fs::read_to_string(report.reasoning_packet_tsv)?;
    assert!(packet_tsv.contains("document_text_ontology_proposal"));
    assert!(packet_tsv.contains("synthetic.file.one"));

    Ok(())
}

#[test]
fn structural_facts_reasoning_packet_classifies_service_catalog_targets()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_facts(
        &EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let mut structural_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        &structural_report.structural_facts_json,
    )?)?;
    structural_json["documents"][0]["relativePath"] =
        serde_json::json!("policy/长期护理服务项目目录.xlsx");
    structural_json["documents"][0]["category"] = serde_json::json!("service_catalog");
    fs::write(
        &structural_report.structural_facts_json,
        serde_json::to_string_pretty(&structural_json)?,
    )?;

    let request = EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
        &structural_report.structural_facts_json,
        "reasoning_seed",
    );
    let report = write_episteme_ontology_structural_facts_reasoning_packet(
        &request,
        root.join("runs/reasoning"),
    )?;

    let packet_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.reasoning_packet_json)?)?;
    assert_eq!(
        packet_json[0]["evidenceTargetIntent"],
        "service_catalog_extraction"
    );
    assert_eq!(
        packet_json[0]["evidenceStructureHint"],
        "document_root:service_catalog"
    );
    let packet_tsv = fs::read_to_string(report.reasoning_packet_tsv)?;
    assert!(packet_tsv.contains("service_catalog_extraction"));

    Ok(())
}

#[test]
fn structural_facts_reasoning_packet_prefers_policy_city_over_service_table()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_facts(
        &EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let mut structural_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        &structural_report.structural_facts_json,
    )?)?;
    structural_json["documents"][0]["relativePath"] =
        serde_json::json!("policy/第一批长护险试点城市比较（15城）.xls");
    structural_json["documents"][0]["category"] = serde_json::json!("全国长护险（专题）");
    fs::write(
        &structural_report.structural_facts_json,
        serde_json::to_string_pretty(&structural_json)?,
    )?;

    let request = EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
        &structural_report.structural_facts_json,
        "reasoning_seed",
    );
    let report = write_episteme_ontology_structural_facts_reasoning_packet(
        &request,
        root.join("runs/reasoning"),
    )?;

    let packet_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.reasoning_packet_json)?)?;
    assert_eq!(
        packet_json[0]["evidenceTargetIntent"],
        "policy_city_relation"
    );
    assert_eq!(
        packet_json[0]["evidenceStructureHint"],
        "document_root:policy_city_relation"
    );

    Ok(())
}

#[test]
fn structural_facts_reasoning_packet_rejects_missing_document_root_anchor()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_facts(
        &EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let mut structural_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        &structural_report.structural_facts_json,
    )?)?;
    structural_json["anchors"] = serde_json::json!([]);
    fs::write(
        &structural_report.structural_facts_json,
        serde_json::to_string_pretty(&structural_json)?,
    )?;

    let request = EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
        &structural_report.structural_facts_json,
        "reasoning_seed",
    );
    let error = match write_episteme_ontology_structural_facts_reasoning_packet(
        &request,
        root.join("runs/reasoning"),
    ) {
        Ok(report) => panic!("expected missing anchor error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("has no document_root anchor"));
    Ok(())
}
