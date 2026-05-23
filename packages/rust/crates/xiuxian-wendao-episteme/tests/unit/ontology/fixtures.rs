use std::{fs, path::Path};

use sha2::{Digest, Sha256};

pub(super) fn write_valid_ontology_fixture(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "ontology/00_Core",
        "ontology/10_Domain/rules",
        "ontology/10_Domain/policies",
        "ontology/10_Domain/mappings",
        "ontology/examples/local_project",
    ] {
        fs::create_dir_all(root.join(path))?;
    }

    fs::write(root.join("ontology/00_Core/ontology.rdf"), "<rdf:RDF />")?;
    fs::write(root.join("ontology/10_Domain/ontology.rdf"), "<rdf:RDF />")?;
    fs::write(
        root.join("ontology/10_Domain/rules/01_rule.sql"),
        "SELECT 1;",
    )?;
    fs::write(
        root.join("ontology/10_Domain/policies/policy.md"),
        "# Policy\n",
    )?;
    fs::write(
        root.join("ontology/10_Domain/mappings/mapping.toml"),
        "schema_version = 1\n",
    )?;
    fs::write(
        root.join("ontology/examples/local_project/ontology.toml"),
        "name = \"example\"\n",
    )?;
    fs::write(
        root.join("ontology/api_surface.toml"),
        "object_types = []\n",
    )?;
    fs::write(root.join("ontology/manifest.toml"), valid_manifest())?;
    Ok(())
}

pub(super) fn write_private_extension_fixture(
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "ontology/10_Private",
        "ontology/10_Private/corpus",
        "ontology/10_Private/mappings",
        "ontology/10_Private/review_ledgers",
    ] {
        fs::create_dir_all(root.join(path))?;
    }

    fs::write(root.join("ontology/10_Private/ontology.rdf"), "<rdf:RDF />")?;
    fs::write(
        root.join("ontology/10_Private/corpus/source_manifest.toml"),
        "schema_version = 1\n",
    )?;
    fs::write(
        root.join("ontology/10_Private/mappings/corpus_mapping.org"),
        "#+TITLE: Mapping\n",
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/review.toml"),
        r#"schema_version = 1
ledger_id = "synthetic.review.v1"
domain = "episteme://private/synthetic/10_Private"
ledger_org = "review.org"
promotion_allowed = false
source_mutation_allowed = false
ontology_truth = false
"#,
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/review.org"),
        r"#+TITLE: Synthetic Review Ledger

* Review
:PROPERTIES:
:ID: 11111111-1111-4111-8111-111111111111
:AUTHORING_KIND: corpus_mapping
:LIFECYCLE_STATE: review
:END:

| field | value |
| --- | --- |
| status | candidate |
",
    )?;
    fs::write(root.join("ontology/manifest.toml"), private_manifest())?;
    Ok(())
}

pub(super) fn write_structural_idf_fixture(
    root: &Path,
    corpus_root: &Path,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_private_extension_fixture(root)?;
    let source_dir = root.join("ontology/10_Private/corpus");
    let source_path = corpus_root.join("policy/source.txt");
    let source_parent = source_path.parent().ok_or("source path missing parent")?;
    fs::create_dir_all(source_parent)?;
    fs::write(&source_path, "Private source evidence\n")?;
    let bytes = fs::read(&source_path)?;
    let good_hash = format!("{:x}", Sha256::digest(&bytes));
    let hash = if mode == "bad_hash" {
        "0000000000000000000000000000000000000000000000000000000000000000"
    } else {
        good_hash.as_str()
    };

    fs::write(
        source_dir.join("source_manifest.toml"),
        r#"schema_version = 1
source_contract_id = "synthetic.private.source.v1"
domain = "episteme://private/synthetic/10_Private"
primary_language = "zh-CN"
corpus_root_env = "SYNTHETIC_CORPUS_ROOT"
files = "files.tsv"
extraction_queue = "extraction_queue.tsv"
copy_raw_files = false
raw_to_rdf_promotion_allowed = false
ignored_names = []

[routes]
document_text_evidence = ["txt"]
"#,
    )?;
    let duplicate_row = if mode == "duplicate_file_id" {
        format!(
            "\nsynthetic.file.one\tpolicy/source-copy.txt\ttxt\t{}\t{}\tpolicy\tzh-CN\tdocument_text_evidence",
            bytes.len(),
            hash
        )
    } else {
        String::new()
    };
    if mode == "duplicate_file_id" {
        fs::write(
            corpus_root.join("policy/source-copy.txt"),
            "Private source evidence\n",
        )?;
    }
    fs::write(
        source_dir.join("files.tsv"),
        format!(
            "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\nsynthetic.file.one\tpolicy/source.txt\ttxt\t{}\t{}\tpolicy\tzh-CN\tdocument_text_evidence{}",
            bytes.len(),
            hash,
            duplicate_row
        ),
    )?;
    fs::write(
        source_dir.join("extraction_queue.tsv"),
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\n",
    )?;
    Ok(())
}

pub(super) fn replace_manifest_fragment(
    root: &Path,
    needle: &str,
    replacement: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = root.join("ontology/manifest.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    assert!(
        manifest.contains(needle),
        "manifest fixture missing replacement needle: {needle}"
    );
    fs::write(manifest_path, manifest.replace(needle, replacement))?;
    Ok(())
}

pub(super) fn write_object_relation_review_ledgers(
    root: &Path,
    object_decision: &str,
    relation_decision: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    replace_manifest_fragment(
        root,
        r#"review_ledgers = ["10_Private/review_ledgers/review.toml"]"#,
        r#"review_ledgers = ["10_Private/review_ledgers/object.toml", "10_Private/review_ledgers/relation.toml"]"#,
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/object.toml"),
        "schema_version = 1\n\
         ledger_id = \"synthetic.object_review.v1\"\n\
         domain = \"episteme://private/synthetic/10_Private\"\n\
         ledger_org = \"object.org\"\n\
         source_mutation_allowed = false\n\
         ontology_truth = false\n\
         promotion_allowed = false\n",
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/object.org"),
        format!(
            "#+TITLE: Object Review\n\
             \n\
             * Object review\n\
             :PROPERTIES:\n\
             :ID: 44444444-4444-4444-8444-444444444444\n\
             :AUTHORING_KIND: dataset_mapping\n\
             :LIFECYCLE_STATE: review\n\
             :END:\n\
             \n\
             | object_id | object_type | label | evidence_id | review_decision | promotion_decision | reviewer_id |\n\
             | obj.policy | type.policy | Policy | evidence.one | accepted_evidence_candidate | {object_decision} | reviewer.one |\n\
             | obj.service | type.service | Service | evidence.one | accepted_evidence_candidate | {object_decision} | reviewer.one |\n",
        ),
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/relation.toml"),
        "schema_version = 1\n\
         ledger_id = \"synthetic.relation_review.v1\"\n\
         domain = \"episteme://private/synthetic/10_Private\"\n\
         ledger_org = \"relation.org\"\n\
         source_mutation_allowed = false\n\
         ontology_truth = false\n\
         promotion_allowed = false\n",
    )?;
    fs::write(
        root.join("ontology/10_Private/review_ledgers/relation.org"),
        format!(
            "#+TITLE: Relation Review\n\
             \n\
             * Relation review\n\
             :PROPERTIES:\n\
             :ID: 55555555-5555-4555-8555-555555555555\n\
             :AUTHORING_KIND: dataset_mapping\n\
             :LIFECYCLE_STATE: review\n\
             :END:\n\
             \n\
             | relation_id | source_object_id | predicate | target_object_id | evidence_id | review_decision | promotion_decision | reviewer_id |\n\
             | rel.policy.defines.service | obj.policy | pred.defines | obj.service | evidence.one | accepted_evidence_candidate | {relation_decision} | reviewer.one |\n",
        ),
    )?;
    Ok(())
}

fn valid_manifest() -> &'static str {
    r#"schema_version = 1
name = "synthetic-ontology"

[boundaries]
owner = "wendao-episteme"
artifact_mode = "source_contract"
runtime_compilation_owner = "xiuxian-wendao"
sql_execution_owner = "xiuxian-wendao"
mutation_allowed = false

[[domains]]
id = "episteme://synthetic/domain-one"
category = "00"
layer = "L0"
name = "Synthetic Domain One"
rdf_files = ["00_Core/ontology.rdf"]
rules = []

[[domains]]
id = "episteme://synthetic/domain-two"
category = "10"
layer = "L1"
name = "Synthetic Domain Two"
rdf_files = ["10_Domain/ontology.rdf"]
rules = ["10_Domain/rules/01_rule.sql"]
policies = ["10_Domain/policies/policy.md"]
dataset_mappings = ["10_Domain/mappings/mapping.toml"]

[extension_contract]
example = "examples/local_project/ontology.toml"
extends_field = "ontology.metadata.extends"
namespace_field = "ontology.metadata.namespace"
allowed_sections = ["ontology.metadata", "entity"]
rule_mount = "mount_rules_for_extends_only"

[api_surface]
file = "api_surface.toml"
compatibility = "semantic_api_compatibility"
reference_nouns = ["Ontology", "OntologyObject"]
"#
}

fn private_manifest() -> &'static str {
    r#"schema_version = 1
name = "synthetic-private-episteme"
primary_language = "zh-CN"
artifact_mode = "private_source_contract"
mutation_allowed = false

[boundaries]
owner = "synthetic-private-episteme"
common_domain_owner = "wendao-episteme"
runtime_compilation_owner = "xiuxian-wendao"
raw_corpus_policy = "external_evidence_root_only"
raw_to_rdf_promotion_allowed = false

[extends]
common_manifest = "episteme://synthetic/healthcare"
common_ontology_iri = "https://wendao.ai/ontology/healthcare"

[[domains]]
id = "episteme://private/synthetic/10_Private"
category = "10"
layer = "Private-L2"
name_zh = "Private Domain"
name_en = "Private Domain"
rdf_files = ["10_Private/ontology.rdf"]
source_manifests = ["10_Private/corpus/source_manifest.toml"]
mapping_ledgers = ["10_Private/mappings/corpus_mapping.org"]
review_ledgers = ["10_Private/review_ledgers/review.toml"]
"#
}
