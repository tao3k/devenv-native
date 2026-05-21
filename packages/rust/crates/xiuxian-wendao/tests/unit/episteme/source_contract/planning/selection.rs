use std::fs;

use crate::episteme::source_contract::support::EpistemeFixture;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceSelectionPlanRequest, EpistemeRunPlanRequest,
    read_episteme_evidence_selection_file_ids, write_episteme_evidence_selection_plan,
    write_episteme_extraction_run_plan,
};

#[test]
fn episteme_selection_driven_extraction_plan_uses_selected_file_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let selection = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.b".to_string(), "episteme.file.a".to_string()],
        ),
        fixture.episteme_root.join("runs/evidence-selection"),
    )?;
    let selected_file_ids =
        read_episteme_evidence_selection_file_ids(&selection.selection_tsv_path)?;

    let report = write_episteme_extraction_run_plan(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selected_extraction_seed",
        )
        .with_selected_file_ids(selected_file_ids),
        fixture.episteme_root.join("runs/extraction"),
    )?;

    assert_eq!(report.selected_count, 2);
    let tasks = fs::read_to_string(&report.tasks_path)?;
    assert!(tasks.contains("episteme.extract.b\tepisteme.file.b\timages/b.jpg"));
    assert!(tasks.contains("episteme.extract.a\tepisteme.file.a\tdocs/a.txt"));

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(
        receipt["selected_file_ids"],
        serde_json::json!(["episteme.file.b", "episteme.file.a"])
    );
    assert_eq!(receipt["tasks"][0]["file_id"], "episteme.file.b");
    assert_eq!(receipt["tasks"][1]["file_id"], "episteme.file.a");
    assert_eq!(receipt["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(receipt["extraction_executed"], false);
    assert_eq!(receipt["validation_mode"], "contract_shape_only");

    Ok(())
}

#[test]
fn episteme_selection_driven_extraction_plan_rejects_unplannable_selected_id()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let Err(error) = write_episteme_extraction_run_plan(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selected_extraction_seed",
        )
        .with_route("document_text_evidence")
        .with_selected_file_ids(vec!["episteme.file.b".to_string()]),
        fixture.episteme_root.join("runs/extraction"),
    ) else {
        return Err("route-filtered selection must not silently drop selected ids".into());
    };
    assert!(
        error
            .to_string()
            .contains("selected file_id has no plannable pending queue row")
    );

    Ok(())
}

#[test]
fn episteme_selection_driven_extraction_plan_rejects_selection_over_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let Err(error) = write_episteme_extraction_run_plan(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selected_extraction_seed",
        )
        .with_limit(1)
        .with_selected_file_ids(vec![
            "episteme.file.a".to_string(),
            "episteme.file.b".to_string(),
        ]),
        fixture.episteme_root.join("runs/extraction"),
    ) else {
        return Err("selection larger than run-plan limit must fail".into());
    };
    assert!(error.to_string().contains("run-plan limit is 1"));

    Ok(())
}
