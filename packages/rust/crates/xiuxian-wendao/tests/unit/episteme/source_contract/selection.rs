use std::fs;

use super::support::EpistemeFixture;
use xiuxian_wendao::episteme::{
    EpistemeEvidenceSelectionPlanRequest, EpistemeEvidenceSelectionValidationMode,
    write_episteme_evidence_selection_plan,
};

#[test]
fn episteme_evidence_selection_writes_org_tsv_and_receipt() -> Result<(), Box<dyn std::error::Error>>
{
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

    let report = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.b".to_string(), "episteme.file.a".to_string()],
        )
        .with_selection_reason("agent selected table and policy evidence"),
        fixture.episteme_root.join("runs/evidence-selection"),
    )?;

    assert_eq!(report.run_id, "selection_seed");
    assert_eq!(report.selected_count, 2);
    assert!(!report.extraction_executed);
    assert!(!report.raw_to_rdf_promotion_allowed);
    assert_eq!(
        report.validation_mode,
        EpistemeEvidenceSelectionValidationMode::MetadataOnly
    );
    assert!(report.selection_org_path.is_file());
    assert!(report.selection_tsv_path.is_file());
    assert!(report.receipt_path.is_file());

    let org = fs::read_to_string(&report.selection_org_path)?;
    assert!(org.contains(":WENDAO_KIND: episteme_evidence_selection"));
    assert!(org.contains(":ONTOLOGY_KIND: source_evidence_selection"));
    assert!(org.contains("episteme.file.b"));
    assert!(org.contains("agent selected table and policy evidence"));
    assert!(org.contains("extractor:image_ocr_evidence"));
    assert!(
        !org.contains("fixture content"),
        "selection ledger must not embed raw source corpus text"
    );

    let tsv = fs::read_to_string(&report.selection_tsv_path)?;
    assert!(tsv.starts_with(
        "selection_index\tfile_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\tselection_reason\tnext_action\n"
    ));
    assert!(tsv.contains("1\tepisteme.file.b\timages/b.jpg\tjpg\t"));
    assert!(tsv.contains("\timage_ocr_evidence\tagent selected table and policy evidence\textractor:image_ocr_evidence\n"));

    let receipt = fs::read_to_string(&report.receipt_path)?;
    let receipt: serde_json::Value = serde_json::from_str(&receipt)?;
    assert_eq!(receipt["runId"], "selection_seed");
    assert_eq!(receipt["selectedCount"], 2);
    assert_eq!(receipt["sourceFileCount"], 2);
    assert_eq!(receipt["rawToRdfPromotionAllowed"], false);
    assert_eq!(receipt["extractionExecuted"], false);
    assert_eq!(receipt["routeCounts"]["image_ocr_evidence"], 1);
    assert_eq!(receipt["selections"][0]["fileId"], "episteme.file.b");
    assert_eq!(receipt["selections"][1]["fileId"], "episteme.file.a");

    Ok(())
}

#[test]
fn episteme_evidence_selection_rejects_duplicate_file_ids() -> Result<(), Box<dyn std::error::Error>>
{
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

    let Err(error) = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.a".to_string(), "episteme.file.a".to_string()],
        ),
        fixture.episteme_root.join("runs/evidence-selection"),
    ) else {
        return Err("duplicate selected file ids must fail".into());
    };
    assert!(error.to_string().contains("duplicate selected file_id"));

    Ok(())
}

#[test]
fn episteme_evidence_selection_rejects_unknown_file_id() -> Result<(), Box<dyn std::error::Error>> {
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

    let Err(error) = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_seed",
            vec!["episteme.file.missing".to_string()],
        ),
        fixture.episteme_root.join("runs/evidence-selection"),
    ) else {
        return Err("unknown selected file id must fail".into());
    };
    assert!(error.to_string().contains("unknown selected file_id"));

    Ok(())
}

#[test]
fn episteme_evidence_selection_full_hash_rejects_hash_drift()
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

    let Err(error) = write_episteme_evidence_selection_plan(
        &EpistemeEvidenceSelectionPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "selection_full_hash_seed",
            vec!["episteme.file.a".to_string()],
        )
        .with_validation_mode(EpistemeEvidenceSelectionValidationMode::FullHash),
        fixture.episteme_root.join("runs/evidence-selection"),
    ) else {
        return Err("full-hash selection must reject same-size content drift".into());
    };
    assert!(error.to_string().contains("sha256 drift"));

    Ok(())
}
