use std::fs;

use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralIdfReasoningPacketRequest, EpistemeOntologyStructuralIdfRequest,
    write_episteme_ontology_structural_idf,
    write_episteme_ontology_structural_idf_reasoning_ledger_seed,
    write_episteme_ontology_structural_idf_reasoning_packet,
};

use super::fixtures::write_structural_idf_fixture;

#[test]
fn structural_idf_reasoning_ledger_seed_writes_object_and_relation_slots()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let reasoning_packet_json = write_reasoning_packet_fixture(temp.path())?;

    let request = EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest::new(
        &reasoning_packet_json,
        "ledger_seed",
    );
    let report = write_episteme_ontology_structural_idf_reasoning_ledger_seed(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.packet_row_count, 1);
    assert_eq!(report.object_seed_row_count, 1);
    assert_eq!(report.relation_seed_row_count, 1);
    assert_eq!(report.seed_row_count, 2);
    assert!(!report.execution.source_text_read);
    assert!(!report.execution.llm_executed);
    assert!(!report.safety.source_mutation_allowed);
    assert!(!report.safety.ontology_truth);
    let seed_tsv = fs::read_to_string(report.reasoning_ledger_seed_tsv)?;
    assert!(seed_tsv.contains("object_proposal_slot"));
    assert!(seed_tsv.contains("relation_proposal_slot"));
    assert!(seed_tsv.contains("blocked_until_review"));

    Ok(())
}

#[test]
fn structural_idf_reasoning_ledger_seed_rejects_packet_ontology_truth()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let reasoning_packet_json = write_reasoning_packet_fixture(temp.path())?;
    let mut packet_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&reasoning_packet_json)?)?;
    packet_json[0]["ontologyTruth"] = serde_json::json!(true);
    fs::write(
        &reasoning_packet_json,
        serde_json::to_string_pretty(&packet_json)?,
    )?;

    let request = EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest::new(
        &reasoning_packet_json,
        "ledger_seed",
    );
    let error = match write_episteme_ontology_structural_idf_reasoning_ledger_seed(
        &request,
        temp.path().join("runs/ontology-generation"),
    ) {
        Ok(report) => panic!("expected ontology truth error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("attempted to mark ontology truth")
    );
    Ok(())
}

fn write_reasoning_packet_fixture(
    temp_root: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let root = temp_root.join("episteme");
    let corpus_root = temp_root.join("corpus");
    write_structural_idf_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_idf(
        &EpistemeOntologyStructuralIdfRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let packet_report = write_episteme_ontology_structural_idf_reasoning_packet(
        &EpistemeOntologyStructuralIdfReasoningPacketRequest::new(
            &structural_report.structural_idf_json,
            "reasoning_packet",
        ),
        root.join("runs/ontology-generation"),
    )?;
    Ok(packet_report.reasoning_packet_json)
}
