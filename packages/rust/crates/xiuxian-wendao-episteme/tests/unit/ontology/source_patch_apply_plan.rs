use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchApplyPlanRequest, EpistemeOntologySourcePatchDraftRequest,
    EpistemeOntologySourcePatchPreflightRequest, export_episteme_ontology_source_patch_draft,
    write_episteme_ontology_source_patch_apply_plan,
    write_episteme_ontology_source_patch_preflight,
};

use super::fixtures::{write_object_relation_review_ledgers, write_private_extension_fixture};

#[test]
fn ontology_source_patch_apply_plan_writes_empty_plan_for_pending_draft()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "pending_review", "pending_review")?;
    let run_dir = write_preflight_and_draft(temp.path())?;

    let report = write_episteme_ontology_source_patch_apply_plan(
        &EpistemeOntologySourcePatchApplyPlanRequest::new(&run_dir),
    )?;

    assert_eq!(report.preflight_row_count, 0);
    assert_eq!(report.draft_resource_count, 0);
    assert_eq!(report.apply_plan_row_count, 0);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let plan = fs::read_to_string(&report.source_patch_apply_plan_tsv)?;
    assert!(plan.contains("target_rdf_file"));
    assert_eq!(plan.lines().count(), 1);
    Ok(())
}

#[test]
fn ontology_source_patch_apply_plan_writes_targeted_rows_for_approved_draft()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    let run_dir = write_preflight_and_draft(temp.path())?;

    let report = write_episteme_ontology_source_patch_apply_plan(
        &EpistemeOntologySourcePatchApplyPlanRequest::new(&run_dir),
    )?;

    assert_eq!(report.preflight_row_count, 3);
    assert_eq!(report.draft_resource_count, 3);
    assert_eq!(report.object_apply_plan_count, 2);
    assert_eq!(report.relation_apply_plan_count, 1);
    assert_eq!(report.apply_plan_row_count, 3);

    let plan = fs::read_to_string(&report.source_patch_apply_plan_tsv)?;
    assert!(plan.contains("propose_targeted_source_patch"));
    assert!(plan.contains("episteme://private/synthetic/10_Private"));
    assert!(plan.contains("10_Private/ontology.rdf"));
    assert!(plan.contains("\tfalse\tfalse"));
    Ok(())
}

#[test]
fn ontology_source_patch_apply_plan_rejects_draft_resource_count_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    let run_dir = write_preflight_and_draft(temp.path())?;
    let draft_path = run_dir.join("source_patch_draft.json");
    let draft = fs::read_to_string(&draft_path)?;
    fs::write(
        &draft_path,
        draft.replace("\"draftResourceCount\": 3", "\"draftResourceCount\": 99"),
    )?;

    let Err(error) = write_episteme_ontology_source_patch_apply_plan(
        &EpistemeOntologySourcePatchApplyPlanRequest::new(&run_dir),
    ) else {
        return Err("draft resource mismatch should fail apply-plan generation".into());
    };

    assert!(error.to_string().contains("resource count mismatch"));
    assert!(!run_dir.join("source_patch_apply_plan.tsv").exists());
    assert!(!run_dir.join("source_patch_apply_plan.org").exists());
    assert!(!run_dir.join("source_patch_apply_plan.json").exists());
    Ok(())
}

fn write_preflight_and_draft(
    root: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let run_dir = root.join("runs/source_patch_preflight");
    write_episteme_ontology_source_patch_preflight(
        &EpistemeOntologySourcePatchPreflightRequest::new(root, &run_dir),
    )?;
    export_episteme_ontology_source_patch_draft(&EpistemeOntologySourcePatchDraftRequest::new(
        &run_dir,
    ))?;
    Ok(run_dir)
}
