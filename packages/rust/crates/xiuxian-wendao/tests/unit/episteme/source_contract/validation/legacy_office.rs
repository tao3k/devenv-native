use super::support::{
    EpistemeFixture, EpistemeRunPlanRequest, add_doc_to_docling_route,
    plan_episteme_extraction_run, validate_episteme_source_contract,
};

#[test]
fn episteme_source_contract_validation_accepts_legacy_office_route()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.doc",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "legacy_office_document_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fixture.add_legacy_office_route()?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;

    assert!(report.passed, "{:?}", report.errors);
    Ok(())
}

#[test]
fn episteme_source_contract_validation_rejects_legacy_office_docling_route()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.doc",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    add_doc_to_docling_route(&fixture)?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;

    assert!(!report.passed);
    assert!(report.errors.iter().any(|error| {
        error.contains("legacy Office extension doc")
            && error.contains("legacy_office_document_evidence")
    }));
    let Err(error) = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "legacy_docling_route_seed",
        )
        .with_limit(1),
    ) else {
        return Err("shape-only run planning should reject legacy Office Docling route".into());
    };
    assert!(error.to_string().contains("legacy Office extension doc"));
    Ok(())
}
