use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchApplyPlanRequest, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchApplyRequest, EpistemeOntologySourcePatchDraftRequest,
    EpistemeOntologySourcePatchPreflightRequest, EpistemeOntologySourcePatchReviewPacketRequest,
    apply_episteme_ontology_source_patch, export_episteme_ontology_source_patch_draft,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_apply_preview,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_review_packet,
};

use super::fixtures::{write_object_relation_review_ledgers, write_private_extension_fixture};

#[test]
fn ontology_source_patch_apply_rejects_without_explicit_mutation_flag()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;
    let target = temp.path().join("ontology/10_Private/ontology.rdf");
    let before = fs::read_to_string(&target)?;

    let Err(error) = apply_episteme_ontology_source_patch(
        &EpistemeOntologySourcePatchApplyRequest::new(temp.path(), &run_dir)
            .with_expected_apply_plan_tsv_sha256(review_packet.apply_plan_tsv_sha256),
    ) else {
        return Err("source patch apply should require explicit mutation approval".into());
    };

    assert!(
        error
            .to_string()
            .contains("requires explicit source mutation approval")
    );
    assert_eq!(fs::read_to_string(&target)?, before);
    assert!(!run_dir.join("source_patch_apply.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_apply_rejects_expected_hash_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;

    let Err(error) = apply_episteme_ontology_source_patch(
        &EpistemeOntologySourcePatchApplyRequest::new(temp.path(), &run_dir)
            .with_expected_apply_plan_tsv_sha256("bad-hash")
            .with_allow_source_mutation(true),
    ) else {
        return Err("hash mismatch should fail source patch apply".into());
    };

    assert!(
        error
            .to_string()
            .contains("does not match review packet hash")
    );
    assert!(!run_dir.join("source_patch_apply.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_apply_rejects_target_hash_drift() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;
    let target = temp.path().join("ontology/10_Private/ontology.rdf");
    fs::write(&target, "<rdf:RDF>\n  <!-- drift -->\n</rdf:RDF>\n")?;

    let Err(error) = apply_episteme_ontology_source_patch(
        &EpistemeOntologySourcePatchApplyRequest::new(temp.path(), &run_dir)
            .with_expected_apply_plan_tsv_sha256(review_packet.apply_plan_tsv_sha256)
            .with_allow_source_mutation(true),
    ) else {
        return Err("target hash drift should fail source patch apply".into());
    };

    assert!(error.to_string().contains("hash drifted"));
    assert!(!run_dir.join("source_patch_apply.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_apply_writes_bounded_rdf_xml_block()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;

    let report = apply_episteme_ontology_source_patch(
        &EpistemeOntologySourcePatchApplyRequest::new(temp.path(), &run_dir)
            .with_expected_apply_plan_tsv_sha256(review_packet.apply_plan_tsv_sha256)
            .with_allow_source_mutation(true),
    )?;

    assert_eq!(report.apply_plan_row_count, 3);
    assert_eq!(report.target_rdf_file_count, 1);
    assert_eq!(
        report.applied_targets[0].target_rdf_file,
        "10_Private/ontology.rdf"
    );
    assert_eq!(report.applied_targets[0].applied_row_count, 3);
    assert_ne!(
        report.applied_targets[0].before_rdf_sha256,
        report.applied_targets[0].after_rdf_sha256
    );
    assert!(report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let target = fs::read_to_string(temp.path().join("ontology/10_Private/ontology.rdf"))?;
    assert!(target.contains("BEGIN WENDAO SOURCE PATCH"));
    assert!(target.contains("source-patch#ObjectInstanceSourcePatch"));
    assert!(target.contains("<wdsp:recordId>obj.policy</wdsp:recordId>"));
    assert!(target.contains("<wdsp:recordId>rel.policy.defines.service</wdsp:recordId>"));
    assert!(target.contains("<wdsp:sourceMutationAllowed rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">false</wdsp:sourceMutationAllowed>"));
    assert!(target.contains("<wdsp:ontologyTruth rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">false</wdsp:ontologyTruth>"));
    assert!(!target.contains("<wdsp:sourceMutationAllowed rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">true</wdsp:sourceMutationAllowed>"));
    assert!(target.contains("</rdf:RDF>"));

    let receipt = fs::read_to_string(&report.source_patch_apply_json)?;
    assert!(receipt.contains("sourceMutationAllowed"));
    assert!(receipt.contains("afterRdfSha256"));
    Ok(())
}

#[test]
fn ontology_source_patch_apply_preview_writes_blocks_without_mutating_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;
    let target = temp.path().join("ontology/10_Private/ontology.rdf");
    let before = fs::read_to_string(&target)?;

    let report = write_episteme_ontology_source_patch_apply_preview(
        &EpistemeOntologySourcePatchApplyPreviewRequest::new(
            temp.path(),
            &run_dir,
            review_packet.apply_plan_tsv_sha256,
        ),
    )?;

    assert_eq!(fs::read_to_string(&target)?, before);
    assert_eq!(report.apply_plan_row_count, 3);
    assert_eq!(report.target_rdf_file_count, 1);
    assert_eq!(
        report.preview_targets[0].target_rdf_file,
        "10_Private/ontology.rdf"
    );
    assert_eq!(report.preview_targets[0].preview_row_count, 3);
    assert_ne!(
        report.preview_targets[0].before_rdf_sha256,
        report.preview_targets[0].proposed_after_rdf_sha256
    );
    assert_eq!(report.preview_targets[0].preview_block_sha256.len(), 64);
    assert_ne!(report.preview_targets[0].proposed_rdf_path, target);
    assert!(report.preview_targets[0].proposed_rdf_admission_passed);
    assert!(
        report.preview_targets[0]
            .proposed_rdf_admission_checks
            .contains(&"single_bounded_source_patch_block")
    );
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let block = fs::read_to_string(&report.preview_targets[0].preview_block_path)?;
    assert!(block.contains("BEGIN WENDAO SOURCE PATCH"));
    assert!(block.contains("<wdsp:recordId>obj.policy</wdsp:recordId>"));
    assert!(block.contains("<wdsp:recordId>rel.policy.defines.service</wdsp:recordId>"));
    let proposed = fs::read_to_string(&report.preview_targets[0].proposed_rdf_path)?;
    assert!(proposed.contains("<rdf:Description rdf:about=\"urn:synthetic\"/>"));
    assert!(proposed.contains("BEGIN WENDAO SOURCE PATCH"));
    assert!(proposed.contains("<wdsp:recordId>obj.policy</wdsp:recordId>"));
    assert!(proposed.contains("<wdsp:sourceMutationAllowed rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">false</wdsp:sourceMutationAllowed>"));
    assert!(!proposed.contains("<wdsp:sourceMutationAllowed rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">true</wdsp:sourceMutationAllowed>"));
    assert!(proposed.contains("</rdf:RDF>"));
    let receipt = fs::read_to_string(&report.source_patch_apply_preview_json)?;
    assert!(receipt.contains("proposedAfterRdfSha256"));
    assert!(receipt.contains("previewBlockPath"));
    assert!(receipt.contains("proposedRdfPath"));
    assert!(receipt.contains("proposedRdfAdmissionPassed"));
    assert!(receipt.contains("no_mutation_or_truth_escalation"));
    Ok(())
}

#[test]
fn ontology_source_patch_apply_preview_rejects_truthy_mutation_marker_in_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    let target = temp.path().join("ontology/10_Private/ontology.rdf");
    fs::write(
        &target,
        "<rdf:RDF>\n  <rdf:Description rdf:about=\"urn:synthetic\">\n    <wdsp:sourceMutationAllowed xmlns:wdsp=\"https://wendao.ai/ontology/source-patch#\" rdf:datatype=\"http://www.w3.org/2001/XMLSchema#boolean\">true</wdsp:sourceMutationAllowed>\n  </rdf:Description>\n</rdf:RDF>\n",
    )?;
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_apply_preview(
        &EpistemeOntologySourcePatchApplyPreviewRequest::new(
            temp.path(),
            &run_dir,
            review_packet.apply_plan_tsv_sha256,
        ),
    ) else {
        return Err("truthy source mutation marker should fail preview admission".into());
    };

    assert!(
        error
            .to_string()
            .contains("attempted to mark mutation or ontology truth")
    );
    assert!(!run_dir.join("source_patch_apply_preview.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_apply_preview_rejects_expected_hash_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_patch_fixture(temp.path())?;
    write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(temp.path(), &run_dir),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_apply_preview(
        &EpistemeOntologySourcePatchApplyPreviewRequest::new(temp.path(), &run_dir, "bad-hash"),
    ) else {
        return Err("preview hash mismatch should fail".into());
    };

    assert!(
        error
            .to_string()
            .contains("does not match review packet hash")
    );
    assert!(!run_dir.join("source_patch_apply_preview.json").exists());
    Ok(())
}

fn write_reviewed_patch_fixture(
    root: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    write_private_extension_fixture(root)?;
    fs::write(
        root.join("ontology/10_Private/ontology.rdf"),
        "<rdf:RDF>\n  <rdf:Description rdf:about=\"urn:synthetic\"/>\n</rdf:RDF>\n",
    )?;
    write_object_relation_review_ledgers(root, "approved", "approved")?;
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
