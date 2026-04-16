use super::*;

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
