use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchDraftRequest, EpistemeOntologySourcePatchPreflightRequest,
    export_episteme_ontology_source_patch_draft, write_episteme_ontology_source_patch_preflight,
};

use super::fixtures::{write_object_relation_review_ledgers, write_private_extension_fixture};

#[test]
fn ontology_source_patch_draft_writes_empty_receipt_for_pending_preflight()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "pending_review", "pending_review")?;
    let run_dir = temp.path().join("runs/source_patch_preflight");
    write_episteme_ontology_source_patch_preflight(
        &EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir),
    )?;

    let report = export_episteme_ontology_source_patch_draft(
        &EpistemeOntologySourcePatchDraftRequest::new(&run_dir),
    )?;

    assert_eq!(report.preflight_row_count, 0);
    assert_eq!(report.object_patch_count, 0);
    assert_eq!(report.relation_patch_count, 0);
    assert_eq!(report.draft_resource_count, 0);
    assert_eq!(report.draft_statement_count, 0);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let draft = fs::read_to_string(&report.source_patch_draft_ttl)?;
    assert!(draft.contains("https://wendao.ai/ontology/source-patch/"));
    assert!(!draft.contains("ObjectInstanceSourcePatch"));
    Ok(())
}

#[test]
fn ontology_source_patch_draft_writes_object_and_relation_resources()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    let run_dir = temp.path().join("runs/source_patch_preflight");
    write_episteme_ontology_source_patch_preflight(
        &EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir),
    )?;

    let report = export_episteme_ontology_source_patch_draft(
        &EpistemeOntologySourcePatchDraftRequest::new(&run_dir),
    )?;

    assert_eq!(report.preflight_row_count, 3);
    assert_eq!(report.object_patch_count, 2);
    assert_eq!(report.relation_patch_count, 1);
    assert_eq!(report.draft_resource_count, 3);
    assert!(report.draft_statement_count > report.draft_resource_count);

    let draft = fs::read_to_string(&report.source_patch_draft_ttl)?;
    assert!(draft.contains("wdsp:ObjectInstanceSourcePatch"));
    assert!(draft.contains("wdsp:InstanceRelationSourcePatch"));
    assert!(draft.contains("wdsp:recordId \"obj.policy\""));
    assert!(draft.contains("wdsp:domainId \"episteme://private/synthetic/10_Private\""));
    assert!(draft.contains("wdsp:targetRdfFile \"10_Private/ontology.rdf\""));
    assert!(draft.contains("rdfs:label \"Policy\""));
    assert!(draft.contains("wdsp:sourceObjectId \"obj.policy\""));
    assert!(draft.contains("wdsp:targetObjectId \"obj.service\""));
    assert!(draft.contains("wdsp:sourceMutationAllowed \"false\"^^xsd:boolean"));
    assert!(draft.contains("wdsp:ontologyTruth \"false\"^^xsd:boolean"));

    let receipt = fs::read_to_string(&report.source_patch_draft_json)?;
    assert!(receipt.contains("\"sourceMutationAllowed\": false"));
    assert!(receipt.contains("\"ontologyTruth\": false"));
    Ok(())
}

#[test]
fn ontology_source_patch_draft_rejects_preflight_count_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    let run_dir = temp.path().join("runs/source_patch_preflight");
    write_episteme_ontology_source_patch_preflight(
        &EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir),
    )?;
    let receipt_path = run_dir.join("source_patch_preflight.json");
    let receipt = fs::read_to_string(&receipt_path)?;
    fs::write(
        &receipt_path,
        receipt.replace("\"preflightRowCount\": 3", "\"preflightRowCount\": 99"),
    )?;

    let Err(error) = export_episteme_ontology_source_patch_draft(
        &EpistemeOntologySourcePatchDraftRequest::new(&run_dir),
    ) else {
        return Err("preflight row count mismatch should fail".into());
    };

    assert!(error.to_string().contains("row count mismatch"));
    assert!(!run_dir.join("source_patch_draft.ttl").exists());
    assert!(!run_dir.join("source_patch_draft.org").exists());
    assert!(!run_dir.join("source_patch_draft.json").exists());
    Ok(())
}
