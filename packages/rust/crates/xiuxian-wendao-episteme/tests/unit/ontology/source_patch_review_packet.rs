use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchApplyPlanRequest, EpistemeOntologySourcePatchDraftRequest,
    EpistemeOntologySourcePatchPreflightRequest, EpistemeOntologySourcePatchReviewPacketRequest,
    export_episteme_ontology_source_patch_draft, write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_review_packet,
};

use super::fixtures::{
    write_extension_source_contract_fixture, write_object_relation_review_ledgers,
};

#[test]
fn ontology_source_patch_review_packet_writes_empty_packet_for_pending_apply_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_extension_source_contract_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "pending_review", "pending_review")?;
    let run_dir = write_preflight_draft_and_apply_plan(temp.path())?;

    let report = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;

    assert_eq!(report.apply_plan_row_count, 0);
    assert_eq!(report.target_rdf_file_count, 0);
    assert_eq!(report.target_rdf_files.len(), 0);
    assert_eq!(report.apply_plan_tsv_sha256.len(), 64);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let packet = fs::read_to_string(&report.source_patch_review_packet_org)?;
    assert!(packet.contains("ontology_source_patch_review_packet"));
    assert!(packet.contains("| target_rdf_file_count | 0 |"));
    Ok(())
}

#[test]
fn ontology_source_patch_review_packet_hashes_target_rdf_for_approved_apply_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_extension_source_contract_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    let run_dir = write_preflight_draft_and_apply_plan(temp.path())?;

    let report = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;

    assert_eq!(report.apply_plan_row_count, 3);
    assert_eq!(report.object_apply_plan_count, 2);
    assert_eq!(report.relation_apply_plan_count, 1);
    assert_eq!(report.target_rdf_file_count, 1);
    assert_eq!(
        report.target_rdf_files[0].target_rdf_file,
        "10_Extension/ontology.rdf"
    );
    assert_eq!(report.target_rdf_files[0].target_rdf_sha256.len(), 64);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let packet_json = fs::read_to_string(&report.source_patch_review_packet_json)?;
    assert!(packet_json.contains("10_Extension/ontology.rdf"));
    assert!(packet_json.contains("targetRdfSha256"));
    Ok(())
}

#[test]
fn ontology_source_patch_review_packet_rejects_missing_target_rdf()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_extension_source_contract_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    let run_dir = write_preflight_draft_and_apply_plan(temp.path())?;
    fs::remove_file(temp.path().join("ontology/10_Extension/ontology.rdf"))?;

    let Err(error) = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    ) else {
        return Err("missing target RDF should fail review-packet generation".into());
    };

    assert!(error.to_string().contains("failed to canonicalize"));
    assert!(!run_dir.join("source_patch_review_packet.org").exists());
    assert!(!run_dir.join("source_patch_review_packet.json").exists());
    Ok(())
}

fn write_preflight_draft_and_apply_plan(
    root: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let run_dir = root.join("runs/source_patch_preflight");
    write_episteme_ontology_source_patch_preflight(
        &EpistemeOntologySourcePatchPreflightRequest::new(root, &run_dir),
    )?;
    export_episteme_ontology_source_patch_draft(&EpistemeOntologySourcePatchDraftRequest::new(
        &run_dir,
    ))?;
    write_episteme_ontology_source_patch_apply_plan(
        &EpistemeOntologySourcePatchApplyPlanRequest::new(&run_dir),
    )?;
    Ok(run_dir)
}
