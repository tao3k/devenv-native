use super::*;
use std::fmt::Write;
use std::fs;

fn write_file(root: &Path, relative_path: &str, content: &str) {
    let path = root.join(relative_path);
    let Some(parent) = path.parent() else {
        panic!("fixture path should have parent: {path:?}");
    };
    fs::create_dir_all(parent).unwrap_or_else(|error| panic!("parent should exist: {error}"));
    fs::write(path, content).unwrap_or_else(|error| panic!("fixture should be written: {error}"));
}

fn make_unit_test_fixture(test_count: usize, helper_lines: usize) -> String {
    let mut content = String::from("use super::*;\n\n");
    for index in 0..helper_lines {
        let _ = writeln!(content, "const LINE_{index}: usize = {index};");
    }
    content.push('\n');
    for index in 0..test_count {
        let _ = writeln!(
            content,
            "#[test]\nfn case_{index}() {{\n    assert_eq!(LINE_0, 0);\n}}\n"
        );
    }
    content
}

fn make_integration_test_fixture(test_count: usize, helper_lines: usize) -> String {
    let mut content = String::from("use super::*;\n\n");
    for index in 0..helper_lines {
        let _ = writeln!(content, "const CASE_LINE_{index}: usize = {index};");
    }
    content.push('\n');
    for index in 0..test_count {
        let _ = writeln!(
            content,
            "#[test]\nfn contract_case_{index}() {{\n    assert_eq!(CASE_LINE_0, 0);\n}}\n"
        );
    }
    content
}

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

#[test]
fn test_validate_crate_tests_flags_bloated_unit_test_leaf() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "tests/unit/policy.rs",
        &make_unit_test_fixture(8, 240),
    );

    let violations = validate_crate_tests(temp.path());
    assert_eq!(
        violations.len(),
        1,
        "expected one violation: {violations:?}"
    );
    assert_eq!(violations[0].kind, ViolationKind::BloatedUnitTestFile);
    assert!(
        violations[0]
            .path
            .to_string_lossy()
            .ends_with("tests/unit/policy.rs"),
        "unexpected path: {:?}",
        violations[0].path
    );
    assert!(violations[0].suggestion.contains("testing-gate harness"));
}

#[test]
fn test_validate_crate_tests_accepts_focused_unit_test_leaf() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "tests/unit/policy.rs",
        &make_unit_test_fixture(4, 24),
    );

    let violations = validate_crate_tests(temp.path());
    assert!(
        violations.is_empty(),
        "expected no violations for focused unit file: {violations:?}"
    );
}

#[test]
fn test_validate_crate_tests_flags_bloated_integration_test_leaf() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "tests/integration/contracts_modularity.rs",
        &make_integration_test_fixture(12, 390),
    );

    let violations = validate_crate_tests(temp.path());
    assert_eq!(
        violations.len(),
        1,
        "expected one violation: {violations:?}"
    );
    assert_eq!(
        violations[0].kind,
        ViolationKind::BloatedIntegrationTestFile
    );
    assert!(
        violations[0]
            .path
            .to_string_lossy()
            .ends_with("tests/integration/contracts_modularity.rs"),
        "unexpected path: {:?}",
        violations[0].path
    );
    assert!(violations[0].suggestion.contains("testing-gate harness"));
}

#[test]
fn test_validate_crate_tests_accepts_focused_integration_test_leaf() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path(),
        "tests/integration/contracts_runner.rs",
        &make_integration_test_fixture(5, 40),
    );

    let violations = validate_crate_tests(temp.path());
    assert!(
        violations.is_empty(),
        "expected no violations for focused integration file: {violations:?}"
    );
}
