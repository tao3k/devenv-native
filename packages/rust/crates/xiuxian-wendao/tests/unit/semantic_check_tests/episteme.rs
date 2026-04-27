use super::*;
use std::fs;
use std::path::{Path, PathBuf};

fn write_file(root: &Path, relative_path: &str, content: &str) {
    let path = root.join(relative_path);
    let parent = path
        .parent()
        .unwrap_or_else(|| panic!("missing parent for {}", path.display()));
    fs::create_dir_all(parent)
        .unwrap_or_else(|error| panic!("create {}: {error}", parent.display()));
    fs::write(&path, content).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn write_minimal_episteme(root: &Path, policy_sql: &str) {
    write_file(root, "policies/johnny_decimal/validation.sql", policy_sql);
    write_file(
        root,
        "policies/johnny_decimal/diagnostic.toml",
        "id = \"jd.diagnostic\"\n",
    );
    write_file(
        root,
        "policies/johnny_decimal/manifest.toml",
        r#"
[[policy_queries]]
id = "johnny-decimal.anchor-id-validation"
framework = "johnny-decimal"
path = "validation.sql"
statement_mode = "select_only"

[[diagnostic_mappings]]
id = "johnny-decimal.anchor-id-diagnostic"
query = "johnny-decimal.anchor-id-validation"
path = "diagnostic.toml"
"#,
    );
    write_file(root, "prompts/anchor_v3_fixers/fix_jd_id.txt", "Fix ID.\n");
    write_file(
        root,
        "prompts/anchor_v3_fixers/manifest.toml",
        r#"
[defaults]
repair_tooling = "Project AnchoR v3"

[[repair_prompts]]
id = "johnny-decimal.fix-anchor-id"
path = "fix_jd_id.txt"
"#,
    );
    write_file(
        root,
        "policies/authorship/diagnostic.toml",
        "id = \"guard\"\n",
    );
    write_file(
        root,
        "policies/authorship/manifest.toml",
        r#"
[[repair_guards]]
id = "temporal-scaffolding.authorship-boundary"
path = "diagnostic.toml"
"#,
    );
    write_file(root, "sources/johnny_decimal/sources.toml", "[[source]]\n");
    write_file(
        root,
        "sources/johnny_decimal/evolution.skill.md",
        "# Skill\n\nRun the source comparison.\n",
    );
    write_file(
        root,
        "sources/manifest.toml",
        r#"
[[source_evolution_skill_surfaces]]
id = "johnny-decimal.source-evolution"
sources_path = "johnny_decimal/sources.toml"
skill_path = "johnny_decimal/evolution.skill.md"
"#,
    );

    write_file(
        root,
        "episteme.toml",
        r#"
schema_version = 1
name = "test-episteme"

[sql]
statement_mode = "select_only"
forbidden_operations = ["CREATE", "ALTER", "DROP", "INSERT", "UPDATE", "DELETE"]

[imports]
policy_manifests = [
  "policies/johnny_decimal/manifest.toml",
  "policies/authorship/manifest.toml",
]
repair_prompt_manifest = "prompts/anchor_v3_fixers/manifest.toml"
source_evolution_manifest = "sources/manifest.toml"
"#,
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("failed to derive workspace root from CARGO_MANIFEST_DIR"))
        .to_path_buf()
}

#[test]
fn load_episteme_manifest_accepts_directory_input() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(
        temp.path(),
        "SELECT file_path, 'OK' AS violation_type FROM repo_content_chunk;\n",
    );

    let report = load_episteme_manifest(temp.path()).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(report.name.as_deref(), Some("test-episteme"));
    assert_eq!(report.schema_version, Some(1));
    assert_eq!(report.policy_query_count, 1);
    assert_eq!(report.diagnostic_mapping_count, 1);
    assert_eq!(report.repair_prompt_count, 1);
    assert_eq!(report.repair_guard_count, 1);
    assert_eq!(report.source_evolution_skill_count, 1);
    assert_eq!(
        report.policy_queries[0].id,
        "johnny-decimal.anchor-id-validation"
    );
    assert_eq!(report.policy_queries[0].statement_mode, "select_only");
}

#[test]
fn load_episteme_manifest_accepts_manifest_file_input() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(
        temp.path(),
        "WITH ids AS (SELECT file_path FROM repo_content_chunk) SELECT * FROM ids;\n",
    );
    let manifest_path = temp.path().join("episteme.toml");

    let report = load_episteme_manifest(&manifest_path).unwrap_or_else(|error| panic!("{error}"));

    assert!(report.manifest_path.ends_with("episteme.toml"));
    assert_eq!(report.policy_queries.len(), 1);
}

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
fn load_episteme_manifest_ignores_forbidden_words_in_comments_and_literals() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(
        temp.path(),
        "-- CREATE is documentation here, not an operation.\nSELECT 'DROP' AS note FROM repo_content_chunk;\n",
    );

    let report = load_episteme_manifest(temp.path()).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(report.policy_query_count, 1);
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

#[test]
#[ignore = "requires the real wendao-episteme submodule checkout"]
fn load_episteme_manifest_accepts_real_wendao_episteme_submodule() {
    let episteme_root = workspace_root().join("wendao-episteme");

    let report = load_episteme_manifest(&episteme_root)
        .unwrap_or_else(|error| panic!("load real episteme manifest: {error}"));

    assert_eq!(report.name.as_deref(), Some("wendao-episteme"));
    assert!(report.policy_query_count >= 1);
    assert!(report.diagnostic_mapping_count >= 1);
    assert!(report.repair_prompt_count >= 1);
    assert!(report.source_evolution_skill_count >= 1);
    assert!(
        report
            .policy_queries
            .iter()
            .all(|query| query.statement_mode == "select_only")
    );
    assert!(report.policy_queries.iter().any(|query| {
        query.id == "johnny-decimal.anchor-id-validation"
            && query.path == "policies/johnny_decimal/validation.sql"
    }));
}
