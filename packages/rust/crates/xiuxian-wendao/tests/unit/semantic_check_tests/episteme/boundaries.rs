use super::{EpistemeLoadError, load_episteme_manifest, write_file, write_minimal_episteme};

#[test]
fn load_episteme_manifest_rejects_source_local_execution_config() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(temp.path(), "SELECT file_path FROM repo_content_chunk;\n");
    write_file(
        temp.path(),
        "sources/johnny_decimal/sources.toml",
        r#"
[execution]
compiler = "skillsc"

[[source]]
id = "johnny-decimal-official"
"#,
    );

    let error = match load_episteme_manifest(temp.path()) {
        Ok(report) => panic!("expected source registry execution error, got {report:?}"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        EpistemeLoadError::SourceRegistryInlineExecution { .. }
    ));
    assert!(error.to_string().contains("sources/manifest.toml defaults"));
}

#[test]
fn load_episteme_manifest_rejects_prompt_local_repair_tooling() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(temp.path(), "SELECT file_path FROM repo_content_chunk;\n");
    write_file(
        temp.path(),
        "prompts/anchor_v3_fixers/manifest.toml",
        r#"
[defaults]
repair_tooling = "Project AnchoR v3"

[[repair_prompts]]
id = "johnny-decimal.fix-anchor-id"
path = "fix_jd_id.txt"
repair_tooling = "Project AnchoR v3"
"#,
    );

    let error = match load_episteme_manifest(temp.path()) {
        Ok(report) => panic!("expected repair prompt tooling error, got {report:?}"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        EpistemeLoadError::RepairPromptInlineTooling { .. }
    ));
    assert!(error.to_string().contains("[defaults].repair_tooling"));
}
