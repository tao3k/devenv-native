use std::fs;

use super::{EpistemeLoadError, load_episteme_manifest, write_file, write_minimal_episteme};

#[test]
fn load_episteme_manifest_rejects_forbidden_sql_operations() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(
        temp.path(),
        "CREATE TABLE leaked_schema AS SELECT * FROM repo_content_chunk;\n",
    );

    let error = match load_episteme_manifest(temp.path()) {
        Ok(report) => panic!("expected forbidden SQL error, got {report:?}"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        EpistemeLoadError::ForbiddenSqlOperation { .. }
    ));
    assert!(error.to_string().contains("CREATE"));
}

#[test]
fn load_episteme_manifest_rejects_unknown_diagnostic_query_reference() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(temp.path(), "SELECT file_path FROM repo_content_chunk;\n");
    let manifest_path = temp.path().join("policies/johnny_decimal/manifest.toml");
    let mut manifest =
        fs::read_to_string(&manifest_path).unwrap_or_else(|error| panic!("read manifest: {error}"));
    manifest = manifest.replace(
        "query = \"johnny-decimal.anchor-id-validation\"",
        "query = \"missing.policy\"",
    );
    fs::write(&manifest_path, manifest).unwrap_or_else(|error| panic!("write manifest: {error}"));

    let error = match load_episteme_manifest(temp.path()) {
        Ok(report) => panic!("expected unknown diagnostic query error, got {report:?}"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        EpistemeLoadError::UnknownDiagnosticQuery { .. }
    ));
}

#[test]
fn load_episteme_manifest_rejects_inline_root_policy_registration() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_file(
        temp.path(),
        "episteme.toml",
        r#"
schema_version = 1

[imports]
policy_manifests = ["policies/johnny_decimal/manifest.toml"]

[[policy_queries]]
id = "johnny-decimal.anchor-id-validation"
path = "policies/johnny_decimal/validation.sql"
"#,
    );

    let error = match load_episteme_manifest(temp.path()) {
        Ok(report) => panic!("expected inline manifest section error, got {report:?}"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        EpistemeLoadError::InlineManifestSection { .. }
    ));
    assert!(error.to_string().contains("inline policy_queries"));
}
