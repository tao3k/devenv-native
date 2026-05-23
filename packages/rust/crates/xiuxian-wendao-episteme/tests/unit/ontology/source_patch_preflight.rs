use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologySourcePatchPreflightRequest, write_episteme_ontology_source_patch_preflight,
};

use super::fixtures::{
    replace_manifest_fragment, write_object_relation_review_ledgers,
    write_private_extension_fixture,
};

#[test]
fn ontology_source_patch_preflight_writes_pending_only_noop_receipt()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "pending_review", "pending_review")?;

    let run_dir = temp.path().join("runs/source_patch_preflight");
    let request = EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir);
    let report = write_episteme_ontology_source_patch_preflight(&request)?;

    assert_eq!(report.object_review_row_count, 2);
    assert_eq!(report.relation_review_row_count, 1);
    assert_eq!(report.approved_object_count, 0);
    assert_eq!(report.approved_relation_count, 0);
    assert_eq!(report.preflight_row_count, 0);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let preflight_tsv = fs::read_to_string(&report.source_patch_preflight_tsv)?;
    assert!(preflight_tsv.contains("preflight_action"));
    assert_eq!(preflight_tsv.lines().count(), 1);

    let preflight_json = fs::read_to_string(&report.source_patch_preflight_json)?;
    assert!(preflight_json.contains("\"sourceMutationAllowed\": false"));
    assert!(preflight_json.contains("\"ontologyTruth\": false"));
    Ok(())
}

#[test]
fn ontology_source_patch_preflight_writes_approved_object_and_relation_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;

    let run_dir = temp.path().join("runs/source_patch_preflight");
    let request = EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir);
    let report = write_episteme_ontology_source_patch_preflight(&request)?;

    assert_eq!(report.approved_object_count, 2);
    assert_eq!(report.approved_relation_count, 1);
    assert_eq!(report.preflight_row_count, 3);

    let preflight_tsv = fs::read_to_string(&report.source_patch_preflight_tsv)?;
    assert!(preflight_tsv.contains("propose_source_patch"));
    assert!(preflight_tsv.contains("domain_id"));
    assert!(preflight_tsv.contains("episteme://private/synthetic/10_Private"));
    assert!(preflight_tsv.contains("10_Private/ontology.rdf"));
    assert!(preflight_tsv.contains("object_instance"));
    assert!(preflight_tsv.contains("instance_relation"));
    assert!(preflight_tsv.contains("reviewer.one"));
    assert!(preflight_tsv.contains("\tfalse\tfalse"));
    Ok(())
}

#[test]
fn ontology_source_patch_preflight_rejects_approved_relation_without_approved_endpoint()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "pending_review", "approved")?;

    let run_dir = temp.path().join("runs/source_patch_preflight");
    let request = EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir);
    let Err(error) = write_episteme_ontology_source_patch_preflight(&request) else {
        return Err("approved relation without approved endpoint should be rejected".into());
    };

    assert!(
        error
            .to_string()
            .contains("without an approved object-instance review row")
    );
    assert!(!run_dir.join("source_patch_preflight.tsv").exists());
    assert!(!run_dir.join("source_patch_preflight.org").exists());
    assert!(!run_dir.join("source_patch_preflight.json").exists());
    Ok(())
}

#[test]
fn ontology_source_patch_preflight_rejects_approved_rows_without_single_rdf_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_review_ledgers(temp.path(), "approved", "approved")?;
    replace_manifest_fragment(
        temp.path(),
        r#"rdf_files = ["10_Private/ontology.rdf"]"#,
        "rdf_files = []",
    )?;

    let run_dir = temp.path().join("runs/source_patch_preflight");
    let request = EpistemeOntologySourcePatchPreflightRequest::new(temp.path(), &run_dir);
    let Err(error) = write_episteme_ontology_source_patch_preflight(&request) else {
        return Err("approved rows without a single RDF target should be rejected".into());
    };

    assert!(
        error
            .to_string()
            .contains("must declare exactly one RDF source target")
    );
    assert!(!run_dir.join("source_patch_preflight.tsv").exists());
    Ok(())
}
