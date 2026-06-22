use std::fs;

use super::load_episteme_runtime_config;

#[test]
fn episteme_runtime_config_resolves_relative_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let episteme_root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus-root");
    fs::create_dir_all(&episteme_root)?;
    fs::write(
        episteme_root.join("episteme.toml"),
        r#"schema_version = 1

[runtime]
corpus_root = "../corpus-root"
structure_run_root = "runs/structure"
evidence_selection_run_root = "runs/evidence-selection"
extraction_run_root = "runs/extraction"
ontology_generation_run_root = "runs/ontology-generation"
legacy_office_converter = "../tools/legacy-office-converter"
"#,
    )?;

    let Some(config) = load_episteme_runtime_config(&episteme_root)? else {
        return Err("expected episteme runtime config".into());
    };
    assert_eq!(config.corpus, Some(corpus_root));
    assert_eq!(
        config.structure_runs,
        Some(episteme_root.join("runs/structure"))
    );
    assert_eq!(
        config.evidence_selection_runs,
        Some(episteme_root.join("runs/evidence-selection"))
    );
    assert_eq!(
        config.extraction_runs,
        Some(episteme_root.join("runs/extraction"))
    );
    assert_eq!(
        config.ontology_generation_runs,
        Some(episteme_root.join("runs/ontology-generation"))
    );
    assert_eq!(
        config.legacy_office_converter,
        Some(temp.path().join("tools/legacy-office-converter"))
    );

    Ok(())
}

#[test]
fn episteme_runtime_config_is_optional() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;

    assert_eq!(load_episteme_runtime_config(temp.path())?, None);

    Ok(())
}
