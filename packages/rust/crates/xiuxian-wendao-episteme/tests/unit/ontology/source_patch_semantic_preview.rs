use std::{fs, path::Path};

use serde_json::Value;
use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchApplyPlanRequest, EpistemeOntologySourcePatchApplyPreviewRequest,
    EpistemeOntologySourcePatchDraftRequest, EpistemeOntologySourcePatchPreflightRequest,
    EpistemeOntologySourcePatchReviewPacketRequest,
    EpistemeOntologySourcePatchSemanticPreviewRequest, export_episteme_ontology_source_patch_draft,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_apply_preview,
    write_episteme_ontology_source_patch_preflight,
    write_episteme_ontology_source_patch_review_packet,
    write_episteme_ontology_source_patch_semantic_preview,
};

use super::fixtures::{
    write_extension_source_contract_fixture, write_object_relation_review_ledgers,
};

#[test]
fn ontology_source_patch_semantic_preview_writes_graph_ready_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_preview_fixture(temp.path())?;

    let report = write_episteme_ontology_source_patch_semantic_preview(
        &EpistemeOntologySourcePatchSemanticPreviewRequest::new(&run_dir),
    )?;

    assert_eq!(report.apply_plan_row_count, 3);
    assert_eq!(report.semantic_object_count, 2);
    assert_eq!(report.semantic_relation_count, 1);
    assert_eq!(report.semantic_evidence_count, 3);
    assert_eq!(report.semantic_projection_state_count, 1);
    assert!(report.projection_quality_passed);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);
    assert!(report.semantic_objects_tsv.exists());
    assert!(report.semantic_relations_tsv.exists());
    assert!(report.semantic_evidence_tsv.exists());
    assert!(report.semantic_projection_state_json.exists());

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
fn ontology_source_patch_semantic_preview_rejects_failed_preview_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_preview_fixture(temp.path())?;
    let preview_json = run_dir.join("source_patch_apply_preview.json");
    let preview = fs::read_to_string(&preview_json)?;
    fs::write(
        &preview_json,
        preview.replace(
            "\"proposedRdfAdmissionPassed\": true",
            "\"proposedRdfAdmissionPassed\": false",
        ),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_semantic_preview(
        &EpistemeOntologySourcePatchSemanticPreviewRequest::new(&run_dir),
    ) else {
        return Err("semantic preview should reject failed preview admission".into());
    };

    assert!(
        error
            .to_string()
            .contains("did not pass proposed RDF admission")
    );
    assert!(!run_dir.join("semantic_read_model_preview.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_semantic_preview_rejects_stale_apply_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let run_dir = write_reviewed_preview_fixture(temp.path())?;
    let apply_plan = run_dir.join("source_patch_apply_plan.tsv");
    let content = fs::read_to_string(&apply_plan)?;
    fs::write(
        &apply_plan,
        content.replace("obj.policy", "obj.policy.changed"),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_semantic_preview(
        &EpistemeOntologySourcePatchSemanticPreviewRequest::new(&run_dir),
    ) else {
        return Err("semantic preview should reject stale apply-plan TSV".into());
    };

    assert!(error.to_string().contains("hash mismatch"));
    assert!(!run_dir.join("semantic_read_model_preview.json").exists());
    Ok(())
}

fn write_reviewed_preview_fixture(
    root: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    write_extension_source_contract_fixture(root)?;
    fs::write(
        root.join("ontology/10_Extension/ontology.rdf"),
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
    let review_packet = write_episteme_ontology_source_patch_review_packet(
        &EpistemeOntologySourcePatchReviewPacketRequest::new(root, &run_dir),
    )?;
    write_episteme_ontology_source_patch_apply_preview(
        &EpistemeOntologySourcePatchApplyPreviewRequest::new(
            root,
            &run_dir,
            review_packet.apply_plan_tsv_sha256,
        ),
    )?;
    Ok(run_dir)
}

fn json_array_len(value: &Value) -> Result<usize, Box<dyn std::error::Error>> {
    value
        .as_array()
        .map(Vec::len)
        .ok_or_else(|| "expected JSON array".into())
}
