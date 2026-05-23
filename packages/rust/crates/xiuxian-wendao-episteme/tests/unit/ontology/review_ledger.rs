use std::fs;

use xiuxian_wendao_episteme::{EpistemeOntologyError, validate_ontology_contract};

use super::fixtures::{replace_manifest_fragment, write_private_extension_fixture};

#[test]
fn ontology_contract_validation_accepts_object_and_relation_review_ledgers()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_ledgers(temp.path(), &ReviewFixtureOptions::default())?;

    validate_ontology_contract(temp.path())?;

    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_relation_review_unknown_endpoint()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_ledgers(
        temp.path(),
        &ReviewFixtureOptions {
            relation_target_object_id: "obj.missing".to_string(),
            ..ReviewFixtureOptions::default()
        },
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("relation endpoint without object review should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("unknown target_object_id"));
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_review_ledger_hash_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_ledgers(
        temp.path(),
        &ReviewFixtureOptions {
            object_org_hash_line: "ledger_org_sha256 = \"sha256:0000\"\n".to_string(),
            ..ReviewFixtureOptions::default()
        },
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("review-ledger hash mismatch should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("ledger_org_sha256 mismatch"));
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_object_review_source_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_private_extension_fixture(temp.path())?;
    write_object_relation_ledgers(
        temp.path(),
        &ReviewFixtureOptions {
            object_source_mutation_allowed: true,
            ..ReviewFixtureOptions::default()
        },
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("object review source mutation should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("source_mutation_allowed=false"));
    Ok(())
}

#[derive(Debug, Clone)]
struct ReviewFixtureOptions {
    relation_target_object_id: String,
    object_org_hash_line: String,
    object_source_mutation_allowed: bool,
}

impl Default for ReviewFixtureOptions {
    fn default() -> Self {
        Self {
            relation_target_object_id: "obj.service".to_string(),
            object_org_hash_line: String::new(),
            object_source_mutation_allowed: false,
        }
    }
}

fn write_object_relation_ledgers(
    root: &std::path::Path,
    options: &ReviewFixtureOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    replace_manifest_fragment(
        root,
        r#"review_ledgers = ["10_Private/review_ledgers/review.toml"]"#,
        r#"review_ledgers = ["10_Private/review_ledgers/object.toml", "10_Private/review_ledgers/relation.toml"]"#,
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/object.toml"),
        format!(
            r#"schema_version = 1
ledger_id = "synthetic.object_review.v1"
domain = "episteme://private/synthetic/10_Private"
ledger_org = "object.org"
source_mutation_allowed = {}
ontology_truth = false
promotion_allowed = false
{}"#,
            options.object_source_mutation_allowed, options.object_org_hash_line
        ),
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/object.org"),
        r"#+TITLE: Object Review

* Object review
:PROPERTIES:
:ID: 22222222-2222-4222-8222-222222222222
:AUTHORING_KIND: dataset_mapping
:LIFECYCLE_STATE: review
:END:

| object_id | object_type | label | evidence_id | review_decision | promotion_decision | reviewer_id |
| --- | --- | --- | --- | --- | --- | --- |
| obj.policy | type.policy | Policy | evidence.one | accepted_evidence_candidate | pending_review | reviewer.one |
| obj.service | type.service | Service | evidence.one | accepted_evidence_candidate | pending_review | reviewer.one |
",
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/relation.toml"),
        r#"schema_version = 1
ledger_id = "synthetic.relation_review.v1"
domain = "episteme://private/synthetic/10_Private"
ledger_org = "relation.org"
source_mutation_allowed = false
ontology_truth = false
promotion_allowed = false
"#,
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/relation.org"),
        format!(
            r"#+TITLE: Relation Review

* Relation review
:PROPERTIES:
:ID: 33333333-3333-4333-8333-333333333333
:AUTHORING_KIND: dataset_mapping
:LIFECYCLE_STATE: review
:END:

| relation_id | source_object_id | predicate | target_object_id | evidence_id | review_decision | promotion_decision | reviewer_id |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rel.policy.defines.service | obj.policy | pred.defines | {} | evidence.one | accepted_evidence_candidate | pending_review | reviewer.one |
",
            options.relation_target_object_id
        ),
    )?;
    Ok(())
}
