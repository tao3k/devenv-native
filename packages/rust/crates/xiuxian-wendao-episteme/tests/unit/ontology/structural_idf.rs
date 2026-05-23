use std::fs;

use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralIdfRequest, EpistemeOntologyStructuralIdfValidationMode,
    write_episteme_ontology_structural_idf,
};

use super::fixtures::write_structural_idf_fixture;

#[test]
fn structural_idf_writes_documents_anchors_and_relations() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_idf_fixture(&root, &corpus_root, "expected")?;

    let request = EpistemeOntologyStructuralIdfRequest::new(&root, &corpus_root, "structural_seed")
        .with_validation_mode(EpistemeOntologyStructuralIdfValidationMode::FullHash);
    let report = write_episteme_ontology_structural_idf(&request, root.join("runs/structure"))?;

    assert_eq!(report.file_count, 1);
    assert_eq!(report.document_count, 1);
    assert!(report.anchor_count >= 2);
    assert!(report.relation_count >= 1);
    assert!(!report.safety.extraction_executed);
    assert!(!report.safety.source_mutation_allowed);
    assert!(!report.safety.ontology_truth);
    assert!(report.full_hash_checked);
    assert!(report.structural_idf_json.is_file());
    assert!(report.structural_idf_org.is_file());
    assert!(fs::read_to_string(report.documents_tsv)?.contains("idf.document.synthetic.file.one"));

    Ok(())
}

#[test]
fn structural_idf_rejects_sha256_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_idf_fixture(&root, &corpus_root, "bad_hash")?;

    let request = EpistemeOntologyStructuralIdfRequest::new(&root, &corpus_root, "structural_seed")
        .with_validation_mode(EpistemeOntologyStructuralIdfValidationMode::FullHash);
    let error = match write_episteme_ontology_structural_idf(&request, root.join("runs/structure"))
    {
        Ok(report) => panic!("expected sha256 drift error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("sha256 drift"));
    Ok(())
}

#[test]
fn structural_idf_rejects_duplicate_file_ids() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_idf_fixture(&root, &corpus_root, "duplicate_file_id")?;

    let request = EpistemeOntologyStructuralIdfRequest::new(&root, &corpus_root, "structural_seed");
    let error = match write_episteme_ontology_structural_idf(&request, root.join("runs/structure"))
    {
        Ok(report) => panic!("expected duplicate file id error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("duplicate file_id"));
    Ok(())
}
