use std::{fs, path::Path};

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
        "schema_version = 1\n",
    )?;
    fs::write(root.join("ontology/manifest.toml"), private_manifest())?;
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
