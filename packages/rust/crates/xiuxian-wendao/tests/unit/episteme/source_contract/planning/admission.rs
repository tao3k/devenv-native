use std::fs;

use crate::episteme::source_contract::support::EpistemeFixture;
use xiuxian_wendao::episteme::{
    EpistemeRunPlanRequest, plan_episteme_extraction_run, validate_episteme_source_contract,
    write_episteme_extraction_run_plan,
};

#[test]
fn episteme_source_contract_plans_image_ocr_admission_without_execution()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
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

    let request = EpistemeRunPlanRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "image_ocr_admission_seed",
    )
    .with_route("image_ocr_evidence")
    .with_limit(4);
    let receipt = plan_episteme_extraction_run(&request)?;

    assert_eq!(receipt.selected_count, 1);
    assert_eq!(receipt.route_counts.get("image_ocr_evidence"), Some(&1));
    assert!(!receipt.extraction_executed);
    assert!(!receipt.raw_to_rdf_promotion_allowed);

    let task = &receipt.tasks[0];
    assert_eq!(task.queue_id, "episteme.extract.b");
    assert_eq!(task.file_id, "episteme.file.b");
    assert_eq!(task.relative_path, "images/b.jpg");
    assert_eq!(task.extraction_route, "image_ocr_evidence");
    assert_eq!(task.output_contract, "cache_only_no_rdf_promotion");
    assert_eq!(task.status, "planned");
    assert_eq!(task.planned_output_path, "outputs/episteme.extract.b.json");

    let report = write_episteme_extraction_run_plan(
        &request,
        fixture.episteme_root.join("runs/extraction"),
    )?;
    let tasks = fs::read_to_string(&report.tasks_path)?;
    assert!(tasks.contains(
        "episteme.extract.b\tepisteme.file.b\timages/b.jpg\tsynthetic_case_category\tzh-CN\timage_ocr_evidence\t45"
    ));
    assert!(
        tasks.contains("\toutputs/episteme.extract.b.json\tcache_only_no_rdf_promotion\tplanned\n")
    );

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["route"], "image_ocr_evidence");
    assert_eq!(receipt["route_counts"]["image_ocr_evidence"], 1);
    assert_eq!(
        receipt["tasks"][0]["extraction_route"],
        "image_ocr_evidence"
    );
    assert_eq!(receipt["tasks"][0]["status"], "planned");
    assert_eq!(receipt["extraction_executed"], false);
    assert_eq!(receipt["raw_to_rdf_promotion_allowed"], false);

    Ok(())
}

#[test]
fn episteme_extraction_plan_uses_shape_validation_without_hashing()
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
    fixture.write_contract()?;

    let source_path = fixture.corpus_root.join("docs/a.txt");
    let original = fs::read_to_string(&source_path)?;
    fs::write(&source_path, "x".repeat(original.len()))?;

    let receipt = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "shape_only_plan_seed",
        )
        .with_limit(1),
    )?;

    assert_eq!(receipt.selected_count, 1);
    assert_eq!(receipt.validation_mode, "contract_shape_only");

    let full_validation =
        validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;
    assert!(!full_validation.passed);
    assert!(
        full_validation
            .errors
            .iter()
            .any(|error| error.contains("sha256 drift"))
    );

    Ok(())
}
