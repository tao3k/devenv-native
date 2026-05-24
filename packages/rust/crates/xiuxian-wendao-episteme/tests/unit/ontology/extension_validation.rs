use std::{fs, path::Path};

use sha2::{Digest, Sha256};
use xiuxian_wendao_episteme::{
    EpistemeExtensionValidationMode, EpistemeExtensionValidationRequest,
    validate_episteme_extension_contract,
};

#[test]
fn extension_pack_validation_accepts_extension_source_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let corpus = tempfile::tempdir()?;
    write_extension_fixture(temp.path(), corpus.path())?;

    let report = validate_episteme_extension_contract(
        &EpistemeExtensionValidationRequest::new(temp.path())
            .with_corpus_root(corpus.path())
            .with_validation_mode(EpistemeExtensionValidationMode::FullHash),
    )?;

    assert_eq!(report.domains, 1);
    assert_eq!(report.rdf_files, 1);
    assert_eq!(report.object_model_contracts, 1);
    assert_eq!(report.source_manifests, 1);
    assert_eq!(report.source_files, 1);
    assert_eq!(report.extraction_queue_rows, 1);
    assert_eq!(report.rdf_classes, 2);
    assert_eq!(report.rdf_object_properties, 1);
    assert_eq!(report.object_types, 2);
    assert_eq!(report.property_types, 4);
    assert_eq!(report.link_types, 1);
    assert_eq!(report.action_types, 1);
    assert_eq!(report.query_types, 1);
    Ok(())
}

#[test]
fn extension_pack_validation_uses_episteme_toml_corpus_root()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let corpus = tempfile::tempdir()?;
    write_extension_fixture(temp.path(), corpus.path())?;

    let report = validate_episteme_extension_contract(
        &EpistemeExtensionValidationRequest::new(temp.path())
            .with_validation_mode(EpistemeExtensionValidationMode::FullHash),
    )?;

    assert_eq!(report.source_files, 1);
    assert_eq!(report.extraction_queue_rows, 1);
    Ok(())
}

#[test]
fn extension_pack_validation_rejects_missing_primary_language_label()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let corpus = tempfile::tempdir()?;
    write_extension_fixture(temp.path(), corpus.path())?;
    replace_file_fragment(
        &temp.path().join("ontology/10_Extension/object_model.toml"),
        r#"display_name = "政策文件""#,
        r#"display_name = "Policy Document""#,
    )?;

    let error = match validate_episteme_extension_contract(
        &EpistemeExtensionValidationRequest::new(temp.path()).with_corpus_root(corpus.path()),
    ) {
        Ok(report) => {
            return Err(format!(
                "English-only display_name should fail a zh-CN extension pack; got {report:#?}"
            )
            .into());
        }
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("object_type.display_name must contain a Chinese label")
    );
    Ok(())
}

#[test]
fn extension_pack_validation_rejects_unknown_rdf_class() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let corpus = tempfile::tempdir()?;
    write_extension_fixture(temp.path(), corpus.path())?;
    replace_file_fragment(
        &temp.path().join("ontology/10_Extension/object_model.toml"),
        "https://private.example.test/ontology/ltc#PolicyDocument",
        "https://private.example.test/ontology/ltc#MissingClass",
    )?;

    let error = match validate_episteme_extension_contract(
        &EpistemeExtensionValidationRequest::new(temp.path()).with_corpus_root(corpus.path()),
    ) {
        Ok(report) => {
            return Err(format!(
                "unknown RDF class should fail extension validation; got {report:#?}"
            )
            .into());
        }
        Err(error) => error,
    };

    assert!(error.to_string().contains("references unknown RDF class"));
    Ok(())
}

#[test]
fn extension_pack_validation_rejects_hash_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let corpus = tempfile::tempdir()?;
    write_extension_fixture(temp.path(), corpus.path())?;
    fs::write(corpus.path().join("policy/source.txt"), "changed\n")?;

    let error = match validate_episteme_extension_contract(
        &EpistemeExtensionValidationRequest::new(temp.path())
            .with_corpus_root(corpus.path())
            .with_validation_mode(EpistemeExtensionValidationMode::FullHash),
    ) {
        Ok(report) => {
            return Err(
                format!("full hash validation should reject drift; got {report:#?}").into(),
            );
        }
        Err(error) => error,
    };

    assert!(
        error.to_string().contains("byte_size drift") || error.to_string().contains("sha256 drift")
    );
    Ok(())
}

#[test]
#[ignore = "requires WENDAO_EPISTEME_EXTENSION_ROOT and a configured extension-pack corpus"]
fn extension_pack_validation_accepts_configured_real_extension_pack()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(root) = std::env::var_os("WENDAO_EPISTEME_EXTENSION_ROOT") else {
        panic!("WENDAO_EPISTEME_EXTENSION_ROOT is required");
    };
    let validation_mode = match std::env::var("WENDAO_EPISTEME_EXTENSION_VALIDATION_MODE")
        .unwrap_or_else(|_| "metadata-only".to_owned())
        .as_str()
    {
        "metadata-only" => EpistemeExtensionValidationMode::MetadataOnly,
        "full-hash" => EpistemeExtensionValidationMode::FullHash,
        value => panic!("unsupported WENDAO_EPISTEME_EXTENSION_VALIDATION_MODE `{value}`"),
    };

    let report = validate_episteme_extension_contract(
        &EpistemeExtensionValidationRequest::new(root).with_validation_mode(validation_mode),
    )?;

    assert!(report.domains > 0);
    assert!(report.source_files > 0);
    assert!(report.object_types > 0);
    eprintln!("{report:#?}");
    Ok(())
}

fn write_extension_fixture(
    root: &Path,
    corpus_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "ontology/10_Extension/corpus",
        "ontology/10_Extension/mappings",
        "ontology/10_Extension/review_ledgers",
    ] {
        fs::create_dir_all(root.join(path))?;
    }
    fs::create_dir_all(corpus_root.join("policy"))?;
    let source = corpus_root.join("policy/source.txt");
    fs::write(&source, "Extension source evidence\n")?;
    let bytes = fs::read(&source)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    fs::write(root.join("episteme.toml"), episteme_config(corpus_root))?;
    fs::write(root.join("ontology/manifest.toml"), manifest())?;
    fs::write(
        root.join("ontology/10_Extension/ontology.rdf"),
        ontology_rdf(),
    )?;
    fs::write(
        root.join("ontology/10_Extension/object_model.toml"),
        OBJECT_MODEL,
    )?;
    fs::write(
        root.join("ontology/10_Extension/mappings/corpus_mapping.org"),
        "#+TITLE: Mapping\n",
    )?;
    fs::write(
        root.join("ontology/10_Extension/corpus/source_manifest.toml"),
        source_manifest(),
    )?;
    fs::write(
        root.join("ontology/10_Extension/corpus/files.tsv"),
        format!(
            "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\n\
             synthetic.file.policy\tpolicy/source.txt\ttxt\t{}\t{}\tpolicy\tzh-CN\tdocument_text_evidence\n",
            bytes.len(),
            sha256
        ),
    )?;
    fs::write(
        root.join("ontology/10_Extension/corpus/extraction_queue.tsv"),
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\n\
         synthetic.extract.policy\tsynthetic.file.policy\tpolicy/source.txt\tpolicy\tzh-CN\tdocument_text_evidence\t10\tcache_only_no_rdf_promotion\tpending\n",
    )?;
    Ok(())
}

fn episteme_config(corpus_root: &Path) -> String {
    let corpus_root = corpus_root.to_string_lossy().replace('"', "\\\"");
    format!(
        r#"schema_version = 1

[runtime]
corpus_root = "{corpus_root}"
"#
    )
}

fn replace_file_fragment(
    path: &Path,
    needle: &str,
    replacement: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    assert!(content.contains(needle));
    fs::write(path, content.replace(needle, replacement))?;
    Ok(())
}

fn manifest() -> &'static str {
    r#"schema_version = 1
name = "synthetic-extension-episteme"
primary_language = "zh-CN"
artifact_mode = "extension_source_contract"
mutation_allowed = false

[boundaries]
owner = "synthetic-extension-episteme"
common_domain_owner = "wendao-episteme"
runtime_compilation_owner = "xiuxian-wendao"
raw_corpus_policy = "external_evidence_root_only"
raw_to_rdf_promotion_allowed = false

[extends]
common_manifest = "episteme://30_Healthcare"
common_ontology_iri = "https://wendao.ai/ontology/healthcare"

[[domains]]
id = "episteme://synthetic-extension/10_Extension"
category = "10"
layer = "Extension-L2"
name_zh = "合成私有领域"
name_en = "Synthetic Extension Domain"
rdf_files = ["10_Extension/ontology.rdf"]
object_model_contracts = ["10_Extension/object_model.toml"]
source_manifests = ["10_Extension/corpus/source_manifest.toml"]
mapping_ledgers = ["10_Extension/mappings/corpus_mapping.org"]
"#
}

fn source_manifest() -> &'static str {
    r#"schema_version = 1
source_contract_id = "synthetic.extension.source.v1"
domain = "episteme://synthetic-extension/10_Extension"
primary_language = "zh-CN"
corpus_root_env = "SYNTHETIC_CORPUS_ROOT"
files = "files.tsv"
extraction_queue = "extraction_queue.tsv"
copy_raw_files = false
raw_to_rdf_promotion_allowed = false
ignored_names = []

[routes]
document_text_evidence = ["txt"]
"#
}

fn ontology_rdf() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
  xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Class rdf:about="https://private.example.test/ontology/ltc#PolicyDocument">
    <rdfs:label xml:lang="zh-CN">政策文件</rdfs:label>
  </owl:Class>
  <owl:Class rdf:about="https://private.example.test/ontology/ltc#PilotCity">
    <rdfs:label xml:lang="zh-CN">试点城市</rdfs:label>
  </owl:Class>
  <owl:ObjectProperty rdf:about="https://private.example.test/ontology/ltc#appliesToCity">
    <rdfs:label xml:lang="zh-CN">政策适用城市</rdfs:label>
    <rdfs:domain rdf:resource="https://private.example.test/ontology/ltc#PolicyDocument"/>
    <rdfs:range rdf:resource="https://private.example.test/ontology/ltc#PilotCity"/>
  </owl:ObjectProperty>
</rdf:RDF>
"#
}

const OBJECT_MODEL: &str = r#"schema_version = 1
ontology = "synthetic-extension-episteme"
compatibility = "semantic_api_compatibility"
object_model_compatibility = "foundry_style_object_model_v1"

[boundaries]
artifact_mode = "extension_source_contract"
mutation_allowed = false
runtime_compilation_owner = "xiuxian-wendao"
sdk_generation_owner = "xiuxian-wendao"
rdf_source_authority = true
object_model_source_authority = true
runtime_object_mutation_allowed = false

[[object_types]]
domain = "episteme://synthetic-extension/10_Extension"
api_name = "PolicyDocument"
display_name = "政策文件"
plural_display_name = "政策文件"
status = "active"
rdf_class = "https://private.example.test/ontology/ltc#PolicyDocument"
primary_key = ["policyDocumentId"]
display_name_property = "policyTitle"
title_property = "policyTitle"
interfaces = ["EvidenceBackedEntity"]
visibility = "private"

[[object_types]]
domain = "episteme://synthetic-extension/10_Extension"
api_name = "PilotCity"
display_name = "试点城市"
plural_display_name = "试点城市"
status = "active"
rdf_class = "https://private.example.test/ontology/ltc#PilotCity"
primary_key = ["pilotCityId"]
display_name_property = "cityName"
title_property = "cityName"
interfaces = ["EvidenceBackedEntity"]
visibility = "private"

[[property_types]]
domain = "episteme://synthetic-extension/10_Extension"
object_type = "PolicyDocument"
api_name = "policyDocumentId"
display_name = "政策文件 ID"
value_type = "string"
required = true
indexed = true
search_policy = "exact"

[[property_types]]
domain = "episteme://synthetic-extension/10_Extension"
object_type = "PolicyDocument"
api_name = "policyTitle"
display_name = "政策标题"
value_type = "string"
required = true
indexed = true
search_policy = "full_text"

[[property_types]]
domain = "episteme://synthetic-extension/10_Extension"
object_type = "PilotCity"
api_name = "pilotCityId"
display_name = "城市 ID"
value_type = "string"
required = true
indexed = true
search_policy = "exact"

[[property_types]]
domain = "episteme://synthetic-extension/10_Extension"
object_type = "PilotCity"
api_name = "cityName"
display_name = "城市名称"
value_type = "string"
required = true
indexed = true
search_policy = "full_text"

[[link_types]]
domain = "episteme://synthetic-extension/10_Extension"
api_name = "PolicyDocument.appliesToCities"
display_name = "政策适用城市"
status = "active"
rdf_property = "https://private.example.test/ontology/ltc#appliesToCity"
from_object_type = "PolicyDocument"
to_object_type = "PilotCity"
cardinality = "one_to_many"
from_api_name = "appliesToCities"
to_api_name = "policies"
inverse_api_name = "PilotCity.policies"
foreign_key_property = "policyDocumentId"

[[action_types]]
domain = "episteme://synthetic-extension/10_Extension"
api_name = "proposePolicyCityApplicability"
display_name = "提议政策适用城市"
status = "active"
affected_object_types = ["PolicyDocument", "PilotCity"]
requires_evidence = true
validation_rules = []
parameters = ["policyDocumentId", "pilotCityId", "evidenceId"]
operations = [
  "create_object:PolicyDocument",
  "create_object:PilotCity",
  "create_link:PolicyDocument.appliesToCities",
]
tool_description = "根据审查过的证据，提议政策文件与适用城市之间的候选关系。"

[[query_types]]
domain = "episteme://synthetic-extension/10_Extension"
api_name = "policyDocumentsForCity"
parameters = ["pilotCityId"]
returns = "PolicyDocument"
returns_kind = "object_set"
object_set_recipe = "CityPolicyDocuments"

[[interface_types]]
api_name = "EvidenceBackedEntity"
implemented_by = ["PolicyDocument", "PilotCity"]

[[object_set_recipes]]
domain = "episteme://synthetic-extension/10_Extension"
api_name = "CityPolicyDocuments"
kind = "link"
base_object_type = "PilotCity"
link_type = "PolicyDocument.appliesToCities"
target_object_type = "PolicyDocument"
allowed_methods = ["filter", "aggregate", "load"]
"#;
