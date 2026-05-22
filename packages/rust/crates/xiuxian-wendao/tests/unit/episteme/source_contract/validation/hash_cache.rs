use super::support::{
    EpistemeFixture, EpistemeReadModelRequest, fs,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
    validate_episteme_source_contract, validate_episteme_source_contract_with_hash_cache,
};

#[test]
fn episteme_source_contract_validation_hash_cache_reuses_unchanged_files()
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
    let cache_path = fixture.episteme_root.join("cache/hash-cache.json");

    let (first_report, first_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(first_report.passed, "{:?}", first_report.errors);
    assert_eq!(first_cache.hash_cache_hits, 0);
    assert_eq!(first_cache.hash_cache_misses, 2);
    assert_eq!(first_cache.entries_written, 2);

    let (second_report, second_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(second_report.passed, "{:?}", second_report.errors);
    assert_eq!(second_cache.entries_loaded, 2);
    assert_eq!(second_cache.hash_cache_hits, 2);
    assert_eq!(second_cache.hash_cache_misses, 0);
    assert_eq!(second_cache.entries_written, 2);

    let (materialization, materialization_cache) =
        materialize_episteme_read_model_seed_with_validation_hash_cache(
            &EpistemeReadModelRequest::new(&fixture.episteme_root, &fixture.corpus_root),
            &cache_path,
        )?;
    assert_eq!(materialization.row_counts(), [4, 2, 1]);
    assert_eq!(materialization_cache.hash_cache_hits, 2);
    assert_eq!(materialization_cache.hash_cache_misses, 0);

    Ok(())
}

#[test]
fn episteme_source_contract_validation_hash_cache_rejects_drift()
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
    let cache_path = fixture.episteme_root.join("cache/hash-cache.json");

    let (first_report, first_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(first_report.passed, "{:?}", first_report.errors);
    assert_eq!(first_cache.hash_cache_misses, 1);

    fs::write(fixture.corpus_root.join("docs/a.docx"), "changed")?;
    let (second_report, second_cache) = validate_episteme_source_contract_with_hash_cache(
        &fixture.episteme_root,
        &fixture.corpus_root,
        &cache_path,
    )?;
    assert!(!second_report.passed);
    assert!(
        second_report
            .errors
            .iter()
            .any(|error| error.contains("sha256 drift"))
    );
    assert_eq!(second_cache.hash_cache_hits, 0);
    assert_eq!(second_cache.hash_cache_misses, 1);

    Ok(())
}

#[test]
fn episteme_source_contract_validation_reports_hash_drift() -> Result<(), Box<dyn std::error::Error>>
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
    fs::write(fixture.corpus_root.join("docs/a.docx"), "changed")?;

    let report = validate_episteme_source_contract(&fixture.episteme_root, &fixture.corpus_root)?;
    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.contains("sha256 drift"))
    );

    Ok(())
}
