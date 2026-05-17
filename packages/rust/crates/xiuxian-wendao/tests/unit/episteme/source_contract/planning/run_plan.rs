use std::fs;

use crate::episteme::source_contract::support::EpistemeFixture;
use xiuxian_wendao::episteme::{
    EpistemeRunPlanRequest, plan_episteme_extraction_run, validate_episteme_source_contract,
    write_episteme_extraction_run_plan,
};

#[test]
fn episteme_source_contract_validates_and_plans_seed_run() -> Result<(), Box<dyn std::error::Error>>
{
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

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;
    assert!(report.passed, "{:?}", report.errors);
    assert_eq!(report.files_tsv_rows, 2);
    assert_eq!(report.extraction_queue_rows, 2);
    assert_eq!(report.mapping_ledger_sections, 1);
    assert_eq!(report.mapping_ledger_reasoning_property_records, 1);
    assert!(!report.raw_to_rdf_promotion_allowed);

    let receipt = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "source_contract_seed",
        )
        .with_route("document_text_evidence")
        .with_limit(1),
    )?;
    assert_eq!(receipt.selected_count, 1);
    assert!(!receipt.extraction_executed);
    assert_eq!(receipt.tasks[0].queue_id, "episteme.extract.a");
    assert_eq!(
        receipt.tasks[0].planned_output_path,
        "outputs/episteme.extract.a.json"
    );
    assert_eq!(receipt.route_counts.get("document_text_evidence"), Some(&1));

    Ok(())
}

#[test]
fn episteme_source_contract_writes_deterministic_run_plan() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let request = EpistemeRunPlanRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "source_contract_seed",
    )
    .with_route("document_text_evidence")
    .with_limit(1);
    let run_root = fixture.episteme_root.join("runs/extraction");

    let report = write_episteme_extraction_run_plan(&request, &run_root)?;
    assert_eq!(report.selected_count, 1);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert!(report.outputs_dir.is_dir());
    assert_eq!(report.run_dir, run_root.join("source_contract_seed"));

    let tasks = fs::read_to_string(&report.tasks_path)?;
    assert!(tasks.starts_with(
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\tsource_sha256\tplanned_output_path\toutput_contract\tstatus\n"
    ));
    assert!(tasks.contains(
        "episteme.extract.a\tepisteme.file.a\tdocs/a.docx\tsynthetic_policy_category\tzh-CN\tdocument_text_evidence\t10"
    ));
    assert!(
        tasks.contains("\toutputs/episteme.extract.a.json\tcache_only_no_rdf_promotion\tplanned\n")
    );

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["run_id"], "source_contract_seed");
    assert_eq!(receipt["selected_count"], 1);
    assert_eq!(receipt["extraction_executed"], false);
    assert_eq!(receipt["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(receipt["tasks"][0]["queue_id"], "episteme.extract.a");

    fs::write(&report.tasks_path, "stale")?;
    let rewritten = write_episteme_extraction_run_plan(&request, &run_root)?;
    assert_ne!(fs::read_to_string(rewritten.tasks_path)?, "stale");

    Ok(())
}

#[test]
fn episteme_source_contract_planner_rejects_unsafe_run_id() -> Result<(), Box<dyn std::error::Error>>
{
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.docx",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;

    let Err(error) = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(&fixture.episteme_root, &fixture.corpus_root, "../bad")
            .with_route("document_text_evidence")
            .with_limit(1),
    ) else {
        return Err("unsafe run id should fail".into());
    };
    assert!(error.to_string().contains("invalid run id"));

    Ok(())
}
