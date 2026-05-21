use std::fs;

use super::support::EpistemeFixture;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceByteSizeStatus, EpistemeEvidenceReadRequest,
    EpistemeEvidenceReadValidationMode, EpistemeEvidenceSha256Status,
    EpistemeEvidenceSourceAvailability, read_episteme_evidence,
};

#[test]
fn episteme_evidence_read_returns_bounded_text_preview() -> Result<(), Box<dyn std::error::Error>> {
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

    let report = read_episteme_evidence(
        &EpistemeEvidenceReadRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "episteme.file.a",
        )
        .with_max_preview_bytes(12),
    )?;

    assert_eq!(report.source.file_id, "episteme.file.a");
    assert_eq!(report.source.relative_path, "docs/a.txt");
    assert_eq!(report.source.extraction_route, "document_text_evidence");
    assert_eq!(report.preview_kind, "plain-text");
    assert_eq!(
        report.source_availability,
        EpistemeEvidenceSourceAvailability::Available
    );
    assert_eq!(
        report.byte_size_status,
        EpistemeEvidenceByteSizeStatus::Matches
    );
    assert_eq!(
        report.sha256_status,
        EpistemeEvidenceSha256Status::NotChecked
    );
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    let Some(preview) = report.text_preview else {
        return Err("expected text preview".into());
    };
    assert_eq!(preview.text, "fixture cont");
    assert_eq!(preview.byte_count, 12);
    assert!(preview.truncated);

    Ok(())
}

#[test]
fn episteme_evidence_read_binary_source_returns_reference_without_preview()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "images/b.jpg",
        "episteme.file.b",
        "episteme.extract.b",
        "synthetic_case_category",
        "image_ocr_evidence",
        45,
    )?;
    fixture.write_contract()?;

    let report = read_episteme_evidence(&EpistemeEvidenceReadRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "episteme.file.b",
    ))?;

    assert_eq!(report.source.file_id, "episteme.file.b");
    assert_eq!(report.source.relative_path, "images/b.jpg");
    assert_eq!(report.preview_kind, "unsupported-binary");
    assert!(report.text_preview.is_none());
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);

    Ok(())
}

#[test]
fn episteme_evidence_read_full_hash_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>> {
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

    let Err(error) = read_episteme_evidence(
        &EpistemeEvidenceReadRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "episteme.file.a",
        )
        .with_validation_mode(EpistemeEvidenceReadValidationMode::FullHash),
    ) else {
        return Err("full-hash evidence read must reject same-size content drift".into());
    };
    assert!(error.to_string().contains("sha256 drift"));

    Ok(())
}

#[test]
fn episteme_evidence_read_rejects_unknown_file_id() -> Result<(), Box<dyn std::error::Error>> {
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

    let Err(error) = read_episteme_evidence(&EpistemeEvidenceReadRequest::new(
        &fixture.episteme_root,
        &fixture.corpus_root,
        "episteme.file.missing",
    )) else {
        return Err("unknown file id must fail".into());
    };
    assert!(
        error
            .to_string()
            .contains("unknown source-contract file_id")
    );

    Ok(())
}
