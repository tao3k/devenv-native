use std::{fs, path::Path};

use serde_json::Value;
use sha2::{Digest, Sha256};
use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchApplyPlanRequest, EpistemeOntologySourcePatchApplyRequest,
    EpistemeOntologySourcePatchDraftRequest, EpistemeOntologySourcePatchPreflightRequest,
    EpistemeOntologySourcePatchRdfReadModelRequest, EpistemeOntologySourcePatchReviewPacketRequest,
    apply_episteme_ontology_source_patch, export_episteme_ontology_source_patch_draft,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_rdf_read_model,
    write_episteme_ontology_source_patch_review_packet,
};

use super::fixtures::{
    write_extension_source_contract_fixture, write_object_relation_review_ledgers,
};

#[test]
fn ontology_source_patch_rdf_read_model_reads_applied_rdf_source()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_applied_patch_fixture(temp.path())?;

    let report = write_episteme_ontology_source_patch_rdf_read_model(
        &EpistemeOntologySourcePatchRdfReadModelRequest::new(temp.path(), &run_dir),
    )?;

    assert_eq!(report.rdf_source_row_count, 3);
    assert_eq!(report.semantic_object_count, 2);
    assert_eq!(report.semantic_relation_count, 1);
    assert_eq!(report.semantic_evidence_count, 3);
    assert_eq!(report.semantic_projection_state_count, 1);
    assert_eq!(report.target_rdf_file_count, 1);
    assert!(report.projection_quality_passed);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let objects: Value = serde_json::from_str(&fs::read_to_string(&report.semantic_objects_json)?)?;
    let relations: Value =
        serde_json::from_str(&fs::read_to_string(&report.semantic_relations_json)?)?;
    let projection_state: Value =
        serde_json::from_str(&fs::read_to_string(&report.semantic_projection_state_json)?)?;
    assert_eq!(json_array_len(&objects)?, 2);
    assert_eq!(json_array_len(&relations)?, 1);
    assert_eq!(json_array_len(&projection_state)?, 1);
    assert!(fs::read_to_string(&report.semantic_objects_tsv)?.contains("obj.policy"));
    assert!(fs::read_to_string(&report.semantic_relations_tsv)?.contains("pred.defines"));
    assert!(fs::read_to_string(&report.semantic_evidence_tsv)?.contains("type.policy"));
    Ok(())
}

#[test]
fn ontology_source_patch_rdf_read_model_rejects_target_hash_drift()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_applied_patch_fixture(temp.path())?;
    let target = temp.path().join("ontology/10_Extension/ontology.rdf");
    fs::write(
        &target,
        fs::read_to_string(&target)?.replace("</rdf:RDF>", "  <!-- drift -->\n</rdf:RDF>"),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_rdf_read_model(
        &EpistemeOntologySourcePatchRdfReadModelRequest::new(temp.path(), &run_dir),
    ) else {
        return Err("RDF source read-model should reject target hash drift".into());
    };

    assert!(error.to_string().contains("hash drifted"));
    assert!(!run_dir.join("rdf_source_read_model.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_rdf_read_model_rejects_record_kind_type_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_applied_patch_fixture(temp.path())?;
    let target = temp.path().join("ontology/10_Extension/ontology.rdf");
    let content = fs::read_to_string(&target)?;
    let mutated = content.replacen(
        "<wdsp:recordKind>object_instance</wdsp:recordKind>",
        "<wdsp:recordKind>instance_relation</wdsp:recordKind>",
        1,
    );
    fs::write(&target, mutated.as_str())?;
    rewrite_receipt_target_hash(
        run_dir.join("source_patch_apply.json").as_path(),
        mutated.as_bytes(),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_rdf_read_model(
        &EpistemeOntologySourcePatchRdfReadModelRequest::new(temp.path(), &run_dir),
    ) else {
        return Err("RDF source read-model should reject recordKind/rdf:type drift".into());
    };

    let rendered = format!("{error:#}");
    assert!(rendered.contains("recordKind"));
    assert!(rendered.contains("rdf:type expects"));
    assert!(!run_dir.join("rdf_source_read_model.json").exists());
    Ok(())
}

fn rewrite_receipt_target_hash(
    receipt_path: &Path,
    target_content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut receipt: Value = serde_json::from_str(&fs::read_to_string(receipt_path)?)?;
    let after_hash = format!("{:x}", Sha256::digest(target_content));
    let Some(targets) = receipt
        .get_mut("appliedTargets")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Err("source_patch_apply.json missing appliedTargets".into());
    };
    let Some(first_target) = targets.first_mut() else {
        return Err("source_patch_apply.json has no appliedTargets".into());
    };
    first_target["afterRdfSha256"] = Value::String(after_hash);
    fs::write(receipt_path, serde_json::to_string_pretty(&receipt)?)?;
    Ok(())
}

#[test]
fn ontology_source_patch_rdf_read_model_rejects_unapplied_record_kind_type_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_applied_patch_fixture(temp.path())?;
    let target = temp.path().join("ontology/10_Extension/ontology.rdf");
    let content = fs::read_to_string(&target)?;
    fs::write(
        &target,
        content.replacen(
            "<wdsp:recordKind>object_instance</wdsp:recordKind>",
            "<wdsp:recordKind>instance_relation</wdsp:recordKind>",
            1,
        ),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_rdf_read_model(
        &EpistemeOntologySourcePatchRdfReadModelRequest::new(temp.path(), &run_dir),
    ) else {
        return Err("RDF source read-model should reject recordKind/rdf:type drift".into());
    };

    let rendered = error.to_string();
    assert!(rendered.contains("hash drifted"));
    assert!(!run_dir.join("rdf_source_read_model.json").exists());
    Ok(())
}

fn write_applied_patch_fixture(
    root: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    write_extension_source_contract_fixture(root)?;
    fs::write(
        root.join("ontology/10_Extension/ontology.rdf"),
        "<rdf:RDF\n  xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n  xmlns:rdfs=\"http://www.w3.org/2000/01/rdf-schema#\">\n  <rdf:Description rdf:about=\"urn:synthetic\"/>\n</rdf:RDF>\n",
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
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(root, &run_dir),
    )?;
    apply_episteme_ontology_source_patch(
        &EpistemeOntologySourcePatchApplyRequest::new(root, &run_dir)
            .with_expected_apply_plan_tsv_sha256(review_packet.apply_plan_tsv_sha256)
            .with_allow_source_mutation(true),
    )?;
    Ok(run_dir)
}

fn json_array_len(value: &Value) -> Result<usize, Box<dyn std::error::Error>> {
    value
        .as_array()
        .map(Vec::len)
        .ok_or_else(|| "expected JSON array".into())
}
