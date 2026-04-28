use super::*;

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
fn load_episteme_manifest_ignores_forbidden_words_in_comments_and_literals() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_minimal_episteme(
        temp.path(),
        "-- CREATE is documentation here, not an operation.\nSELECT 'DROP' AS note FROM repo_content_chunk;\n",
    );

    let report = load_episteme_manifest(temp.path()).unwrap_or_else(|error| panic!("{error}"));

    assert_eq!(report.policy_query_count, 1);
}
