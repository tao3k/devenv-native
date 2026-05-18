use std::{fs, path::Path};

use super::{EpistemeError, configured_episteme_corpus_root_env, source_contract_paths};

#[test]
fn selects_single_declared_source_contract() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_episteme_repo(temp.path(), single_domain_manifest())?;

    let paths = source_contract_paths(temp.path())?;

    assert_eq!(paths.domain_id(), "episteme://synthetic/source-contract");
    assert_eq!(
        paths.source_manifest_relative_path(),
        "ontology/SourceContract/corpus/source_manifest.toml"
    );
    assert_eq!(
        paths.mapping_ledger_relative_path(),
        "ontology/SourceContract/mappings/corpus_mapping.org"
    );
    assert_eq!(
        configured_episteme_corpus_root_env(temp.path())?,
        "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
    );
    Ok(())
}

#[test]
fn active_source_contract_selects_one_domain() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_episteme_repo(temp.path(), active_multi_domain_manifest())?;
    fs::create_dir_all(temp.path().join("ontology/Other/corpus"))?;
    fs::create_dir_all(temp.path().join("ontology/Other/mappings"))?;

    let paths = source_contract_paths(temp.path())?;

    assert_eq!(paths.domain_id(), "episteme://synthetic/source-contract");
    assert_eq!(
        paths.source_manifest_relative_path(),
        "ontology/SourceContract/corpus/source_manifest.toml"
    );
    Ok(())
}

#[test]
fn rejects_ambiguous_multiple_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    fs::create_dir_all(temp.path().join("ontology"))?;
    fs::write(
        temp.path().join("ontology/manifest.toml"),
        ambiguous_multi_domain_manifest(),
    )?;

    let error = source_contract_paths(temp.path()).expect_err("ambiguous manifest must fail");

    assert!(
        matches!(error, EpistemeError::InvalidEpistemeManifest(message) if message.contains("active_source_contract"))
    );
    Ok(())
}

#[test]
fn rejects_unsafe_source_manifest_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    fs::create_dir_all(temp.path().join("ontology"))?;
    fs::write(
        temp.path().join("ontology/manifest.toml"),
        unsafe_path_manifest(),
    )?;

    let error = source_contract_paths(temp.path()).expect_err("unsafe manifest must fail");

    assert!(
        matches!(error, EpistemeError::InvalidEpistemeManifest(message) if message.contains("safe paths relative to ontology"))
    );
    Ok(())
}

#[test]
fn rejects_selected_source_manifest_domain_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_episteme_repo_with_source_domain(
        temp.path(),
        single_domain_manifest(),
        "episteme://other",
    )?;

    let error = configured_episteme_corpus_root_env(temp.path())
        .expect_err("source manifest domain mismatch must fail");

    assert!(
        matches!(error, EpistemeError::InvalidEpistemeManifest(message) if message.contains("does not match selected manifest domain"))
    );
    Ok(())
}

fn write_episteme_repo(
    root: &Path,
    ontology_manifest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_episteme_repo_with_source_domain(
        root,
        ontology_manifest,
        "episteme://synthetic/source-contract",
    )
}

fn write_episteme_repo_with_source_domain(
    root: &Path,
    ontology_manifest: &str,
    source_domain: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("ontology/SourceContract/corpus"))?;
    fs::create_dir_all(root.join("ontology/SourceContract/mappings"))?;
    fs::write(root.join("ontology/manifest.toml"), ontology_manifest)?;
    fs::write(
        root.join("ontology/SourceContract/corpus/source_manifest.toml"),
        source_manifest(source_domain),
    )?;
    fs::write(
        root.join("ontology/SourceContract/mappings/corpus_mapping.org"),
        "",
    )?;
    Ok(())
}

fn single_domain_manifest() -> &'static str {
    r#"schema_version = 1

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]
"#
}

fn active_multi_domain_manifest() -> &'static str {
    r#"schema_version = 1

[active_source_contract]
domain_id = "episteme://synthetic/source-contract"
source_manifest = "SourceContract/corpus/source_manifest.toml"
mapping_ledger = "SourceContract/mappings/corpus_mapping.org"

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]

[[domains]]
id = "episteme://other/source-contract"
source_manifests = ["Other/corpus/source_manifest.toml"]
mapping_ledgers = ["Other/mappings/corpus_mapping.org"]
"#
}

fn ambiguous_multi_domain_manifest() -> &'static str {
    r#"schema_version = 1

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]

[[domains]]
id = "episteme://other/source-contract"
source_manifests = ["Other/corpus/source_manifest.toml"]
mapping_ledgers = ["Other/mappings/corpus_mapping.org"]
"#
}

fn unsafe_path_manifest() -> &'static str {
    r#"schema_version = 1

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["../outside/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]
"#
}

fn source_manifest(domain: &str) -> String {
    format!(
        r#"schema_version = 1
source_contract_id = "episteme_source_contract.corpus.v1"
domain = "{domain}"
primary_language = "zh-CN"
corpus_root_env = "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
files = "files.tsv"
extraction_queue = "extraction_queue.tsv"
copy_raw_files = false
raw_to_rdf_promotion_allowed = false

[routes]
document_text_evidence = ["org", "md"]
"#
    )
}
