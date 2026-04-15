use super::*;

#[test]
fn validate_crate_test_policy_returns_clean_report_for_valid_crate() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        "src/foo.rs",
        r#"
fn helper() {}

#[cfg(test)]
#[path = "../tests/unit/foo.rs"]
mod tests;
"#,
    );
    write_fixture_file(
        temp.path(),
        "tests/unit/foo.rs",
        r"
use super::*;

#[test]
fn helper_exists() {
    helper();
}
",
    );

    let report = validate_crate_test_policy(temp.path());
    assert!(report.is_clean(), "expected clean report, got {report:?}");
}

#[test]
fn validate_crate_test_policy_collects_both_policy_layers() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        "src/foo.rs",
        r"
#[cfg(test)]
mod tests {
    #[test]
    fn inline_policy_violation() {}
}
",
    );
    write_fixture_file(
        temp.path(),
        "tests/test_foo.rs",
        "#[test]\nfn scattered() {}\n",
    );

    let report = validate_crate_test_policy(temp.path());
    assert_eq!(report.external_test_issues.len(), 1);
    assert_eq!(report.structure_violations.len(), 1);

    let formatted = format_crate_test_policy_report(&report);
    assert!(formatted.contains("External Test Policy"));
    assert!(formatted.contains("Test Structure Policy"));
}
