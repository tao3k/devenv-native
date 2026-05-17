use std::fs;

use super::support::{EpistemeFixture, SYNTHETIC_MAPPING_LEDGER};
use xiuxian_wendao::episteme::{
    EpistemeReadModelRequest, EpistemeRunPlanRequest,
    materialize_episteme_read_model_seed_with_validation_hash_cache, plan_episteme_extraction_run,
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

#[test]
fn episteme_extraction_plan_shape_validation_rejects_queue_mismatch()
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
    fs::write(
        fixture
            .episteme_root
            .join("ontology/SourceContract/corpus/extraction_queue.tsv"),
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\n",
    )?;

    let Err(error) = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "queue_shape_plan_seed",
        )
        .with_limit(1),
    ) else {
        return Err("queue mismatch should fail shape-only planning".into());
    };
    assert!(
        error
            .to_string()
            .contains("extraction_queue.tsv missing file_id")
    );

    Ok(())
}

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
