use std::fs;

use super::support::EpistemeFixture;
use xiuxian_wendao::episteme::{
    EpistemeStructureTocRequest, EpistemeStructureTocValidationMode, write_episteme_structure_toc,
};

#[test]
fn episteme_structure_toc_writes_org_ledger_without_raw_text()
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

    let report = write_episteme_structure_toc(
        &EpistemeStructureTocRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "structure_seed",
        ),
        fixture.episteme_root.join("runs/structure"),
    )?;

    assert_eq!(report.run_id, "structure_seed");
    assert_eq!(report.file_count, 2);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert_eq!(
        report.validation_mode,
        EpistemeStructureTocValidationMode::MetadataOnly
    );
    assert!(report.toc_org_path.is_file());
    assert!(report.receipt_path.is_file());

    let toc = fs::read_to_string(&report.toc_org_path)?;
    assert!(toc.contains(":WENDAO_KIND: episteme_structure_toc"));
    assert!(toc.contains(":ONTOLOGY_KIND: source_structure_toc"));
    assert!(toc.contains("** Extraction route summary"));
    assert!(toc.contains("| document_text_evidence | 1 |"));
    assert!(toc.contains("| image_ocr_evidence | 1 |"));
    assert!(toc.contains("** Category summary"));
    assert!(toc.contains("| synthetic_policy_category | 1 |"));
    assert!(toc.contains("| synthetic_case_category | 1 |"));
    assert!(toc.contains("episteme.file.a"));
    assert!(toc.contains("docs/a.docx"));
    assert!(toc.contains("document_text_evidence"));
    assert!(
        !toc.contains("fixture content"),
        "TOC ledger must not embed raw source corpus text"
    );

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["runId"], "structure_seed");
    assert_eq!(receipt["fileCount"], 2);
    assert_eq!(
        receipt["routeCounts"]["document_text_evidence"],
        serde_json::json!(1)
    );
    assert_eq!(receipt["rawToRdfPromotionAllowed"], false);
    assert_eq!(receipt["validationMode"], "metadata-only");

    Ok(())
}

#[test]
fn episteme_structure_toc_full_hash_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>> {
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

    let source_path = fixture.corpus_root.join("docs/a.docx");
    let original = fs::read_to_string(&source_path)?;
    fs::write(&source_path, "x".repeat(original.len()))?;

    let metadata_report = write_episteme_structure_toc(
        &EpistemeStructureTocRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "structure_metadata_seed",
        ),
        fixture.episteme_root.join("runs/structure"),
    )?;
    assert_eq!(
        metadata_report.validation_mode,
        EpistemeStructureTocValidationMode::MetadataOnly
    );

    let Err(error) = write_episteme_structure_toc(
        &EpistemeStructureTocRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "structure_full_hash_seed",
        )
        .with_validation_mode(EpistemeStructureTocValidationMode::FullHash),
        fixture.episteme_root.join("runs/structure"),
    ) else {
        return Err("full-hash mode must reject same-size content drift".into());
    };
    assert!(error.to_string().contains("sha256 drift"));

    Ok(())
}
