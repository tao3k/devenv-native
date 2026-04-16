use super::*;
use std::fs;

#[test]
fn test_is_allowed_root_file() {
    assert!(is_allowed_root_file("mod.rs", None));
    assert!(is_allowed_root_file("unit_test.rs", None));
    assert!(is_allowed_root_file("integration_test.rs", None));
    assert!(is_allowed_root_file("performance_test.rs", None));
    assert!(is_allowed_root_file("scenarios_test.rs", None));
    assert!(is_allowed_root_file("xiuxian-testing-gate.rs", None));
    assert!(!is_allowed_root_file("my_test.rs", None));
    assert!(!is_allowed_root_file("test_entity.rs", None));
    assert!(!is_allowed_root_file("entity_unit.rs", None));
}

#[test]
fn test_is_allowed_root_file_with_policy_override() {
    let policy = TestsStructurePolicy {
        allowed_directories: Vec::new(),
        allowed_root_files: vec!["quantum_fusion_saliency_window.rs".to_string()],
    };
    assert!(is_allowed_root_file(
        "quantum_fusion_saliency_window.rs",
        Some(&policy)
    ));
}

#[test]
fn test_is_unit_test_file() {
    assert!(is_unit_test_file("entity_unit.rs"));
    assert!(is_unit_test_file("unit_storage.rs"));
    assert!(!is_unit_test_file("entity.rs"));
    assert!(!is_unit_test_file("test_entity.rs"));
}

#[test]
fn test_is_integration_test_file() {
    assert!(is_integration_test_file(
        "dependency_indexer_integration.rs"
    ));
    assert!(!is_integration_test_file("entity_unit.rs"));
}

#[test]
fn test_performance_directory_is_allowed_by_default() {
    assert!(is_allowed_directory("performance", None));
}

#[test]
fn test_validate_nonexistent_directory() {
    let violations = validate_tests_structure(Path::new("/nonexistent/path/tests"));
    assert!(violations.is_empty());
}

#[test]
fn test_validate_tests_structure_flags_unclassified_root_rs_file() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    let tests_dir = temp.path().join("tests");
    fs::create_dir_all(&tests_dir)
        .unwrap_or_else(|error| panic!("tests dir should exist: {error}"));
    fs::write(
        tests_dir.join("quantum_fusion_saliency_window.rs"),
        "#[test]\nfn smoke() {}\n",
    )
    .unwrap_or_else(|error| panic!("test fixture should exist: {error}"));

    let violations = validate_tests_structure(&tests_dir);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind, ViolationKind::ScatteredTestFile);
}

#[test]
fn test_validate_tests_structure_with_policy_allows_root_file_and_directory() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    let tests_dir = temp.path().join("tests");
    fs::create_dir_all(tests_dir.join("bench"))
        .unwrap_or_else(|error| panic!("custom directory should exist: {error}"));
    fs::write(
        tests_dir.join("coactivation_multihop_diffusion.rs"),
        "#[test]\nfn smoke() {}\n",
    )
    .unwrap_or_else(|error| panic!("test fixture should exist: {error}"));

    let policy = TestsStructurePolicy {
        allowed_directories: vec!["bench".to_string()],
        allowed_root_files: vec!["coactivation_multihop_diffusion.rs".to_string()],
    };
    let violations = validate_tests_structure_with_policy(&tests_dir, Some(&policy));
    assert!(
        violations.is_empty(),
        "expected no violations: {violations:?}"
    );
}

#[test]
fn test_format_violation_report_empty() {
    let report = format_violation_report(&[]);
    assert!(report.contains("No violations"));
}

#[test]
fn test_format_violation_report_with_violations() {
    let violations = vec![StructureViolation {
        path: PathBuf::from("tests/test_entity.rs"),
        kind: ViolationKind::TestPrefixInRoot,
        suggestion: "Move to tests/unit/entity.rs".to_string(),
    }];
    let report = format_violation_report(&violations);
    assert!(report.contains("Found 1"));
    assert!(report.contains("test_entity.rs"));
    assert!(report.contains("Move to tests/unit/entity.rs"));
}
