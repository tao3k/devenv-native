use super::support::{
    EpistemeFixture, EpistemeRunPlanRequest, SYNTHETIC_MAPPING_LEDGER, fs,
    plan_episteme_extraction_run, validate_episteme_source_contract,
};

#[test]
fn episteme_source_contract_validation_rejects_invalid_mapping_ledger_properties()
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
    fixture.write_contract()?;
    fs::write(
        fixture.mapping_ledger_path(),
        SYNTHETIC_MAPPING_LEDGER.replace(
            "16b4038b-2c91-4f70-b38a-e0152629752d",
            "episteme.mapping.invalid",
        ),
    )?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;

    assert!(!report.passed);
    assert_eq!(report.mapping_ledger_sections, 0);
    assert_eq!(report.mapping_ledger_reasoning_property_records, 0);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("mapping ledger") && error.contains("UUID"))
    );

    let Err(error) = plan_episteme_extraction_run(&EpistemeRunPlanRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "source_contract_seed",
    )) else {
        return Err("invalid mapping ledger should prevent run planning".into());
    };
    assert!(error.to_string().contains("mapping ledger"));

    Ok(())
}
