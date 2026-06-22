use super::support::{EpistemeFixture, validate_episteme_source_contract};

#[test]
fn episteme_source_contract_manifest_active_selector_resolves_multi_domain_repo()
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
    fixture.write_multi_domain_manifest(true)?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;

    assert!(report.passed, "{:?}", report.errors);
    assert_eq!(
        report.corpus_root_env,
        "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
    );

    Ok(())
}

#[test]
fn episteme_source_contract_manifest_requires_active_selector_for_multi_domain_repo()
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
    fixture.write_multi_domain_manifest(false)?;

    let Err(error) =
        validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)
    else {
        return Err("multi-domain episteme without active selector should fail".into());
    };

    assert!(error.to_string().contains("active_source_contract"));

    Ok(())
}
