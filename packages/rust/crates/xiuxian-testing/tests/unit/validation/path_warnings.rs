use super::*;

#[test]
fn test_validate_crate_path_warnings_flags_repeated_namespace_segments_in_tests_tree() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "tests/unit/gateway/studio/router/config/router/mod.rs",
        "mod bootstrap;\n",
    );
    write_file(
        temp.path(),
        "tests/unit/gateway/studio/router/config/router/capabilities.rs",
        "fn smoke() {}\n",
    );

    let warnings = validate_crate_path_warnings(temp.path());
    assert_eq!(warnings.len(), 1, "expected one warning: {warnings:?}");
    assert_eq!(warnings[0].repeated_namespaces, vec!["router".to_string()]);
    assert!(
        warnings[0]
            .path
            .to_string_lossy()
            .ends_with("tests/unit/gateway/studio/router/config/router"),
        "unexpected path: {:?}",
        warnings[0].path
    );
}

#[test]
fn test_validate_crate_path_warnings_flags_repeated_namespace_segments_in_src_tree() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "src/gateway/studio/router/config/router/mod.rs",
        "mod bootstrap;\n",
    );

    let warnings = validate_crate_path_warnings(temp.path());
    assert_eq!(warnings.len(), 1, "expected one warning: {warnings:?}");
    assert_eq!(warnings[0].repeated_namespaces, vec!["router".to_string()]);
    assert!(
        warnings[0]
            .path
            .to_string_lossy()
            .ends_with("src/gateway/studio/router/config/router"),
        "unexpected path: {:?}",
        warnings[0].path
    );
}

#[test]
fn test_validate_crate_path_warnings_accepts_unique_namespace_segments() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "src/gateway/studio/router/config/bootstrap/mod.rs",
        "mod support;\n",
    );
    write_file(
        temp.path(),
        "tests/unit/gateway/studio/router/config/bootstrap/mod.rs",
        "mod support;\n",
    );

    let warnings = validate_crate_path_warnings(temp.path());
    assert!(
        warnings.is_empty(),
        "expected no warnings for unique path segments: {warnings:?}"
    );
}

#[test]
fn test_format_path_structure_warning_report_with_warnings() {
    let warnings = vec![PathStructureWarning {
        path: PathBuf::from("src/gateway/studio/router/config/router"),
        repeated_namespaces: vec!["router".to_string()],
        suggestion: "Prefer one stable owner namespace per branch.".to_string(),
    }];

    let report = format_path_structure_warning_report(&warnings);
    assert!(report.contains("Found 1 test path-structure warning"));
    assert!(report.contains("repeated namespace segments: `router`"));
}
