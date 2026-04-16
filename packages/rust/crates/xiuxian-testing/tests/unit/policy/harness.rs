use super::*;

#[test]
fn validate_crate_test_policy_harness_reports_missing_target_gate_mounts() {
    let temp = create_temp_crate();
    write_manifest(
        temp.path(),
        r#"

[[test]]
name = "runtime_config"
path = "tests/integration/runtime_config.rs"
"#,
    );
    write_fixture_file(
        temp.path(),
        "tests/integration/runtime_config.rs",
        "#[test]\nfn smoke() {}\n",
    );

    let report = validate_crate_test_policy_harness(temp.path())
        .unwrap_or_else(|error| panic!("harness validation should succeed: {error}"));
    assert!(report.policy_report.is_clean(), "{report:?}");
    assert_eq!(report.target_gate_violations.len(), 1);
    assert_eq!(
        report.target_gate_violations[0].target_file,
        PathBuf::from("tests/integration/runtime_config.rs")
    );

    let formatted = format_crate_test_policy_harness_report(&report);
    assert!(formatted.contains("Test Target Gate Policy"));
    assert!(formatted.contains("crate_test_policy_harness!"));
}

#[test]
fn validate_crate_test_policy_harness_accepts_macro_mounted_targets() {
    let temp = create_temp_crate();
    write_manifest(
        temp.path(),
        r#"

[[test]]
name = "runtime_config"
path = "tests/integration/runtime_config.rs"
"#,
    );
    write_fixture_file(
        temp.path(),
        "tests/integration/runtime_config.rs",
        r"
xiuxian_testing::crate_test_policy_harness!();

#[test]
fn smoke() {}
",
    );

    let report = validate_crate_test_policy_harness(temp.path())
        .unwrap_or_else(|error| panic!("harness validation should succeed: {error}"));
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn validate_crate_test_policy_harness_accepts_legacy_explicit_gate_target() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        "tests/xiuxian-testing-gate.rs",
        r#"
use std::path::Path;

use xiuxian_testing::assert_crate_test_policy_with_workspace_config;

#[test]
fn enforce_gate() {
    assert_crate_test_policy_with_workspace_config(Path::new(env!("CARGO_MANIFEST_DIR")));
}
"#,
    );

    let report = validate_crate_test_policy_harness(temp.path())
        .unwrap_or_else(|error| panic!("harness validation should succeed: {error}"));
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn validate_crate_test_policy_harness_reports_missing_source_gate_mounts() {
    let temp = create_temp_crate();
    write_fixture_file(temp.path(), "src/lib.rs", "mod foo;\n");
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
        "use super::*;\n#[test]\nfn helper_exists() { helper(); }\n",
    );

    let report = validate_crate_test_policy_harness(temp.path())
        .unwrap_or_else(|error| panic!("harness validation should succeed: {error}"));
    assert!(report.policy_report.is_clean(), "{report:?}");
    assert_eq!(report.target_gate_violations.len(), 0);
    assert_eq!(report.source_gate_violations.len(), 1);
    assert_eq!(
        report.source_gate_violations[0].source_file,
        PathBuf::from("src/lib.rs")
    );

    let formatted = format_crate_test_policy_harness_report(&report);
    assert!(formatted.contains("Source Test Gate Policy"));
    assert!(formatted.contains("crate_test_policy_source_harness!"));
}

#[test]
fn validate_crate_test_policy_harness_accepts_source_harness_macro() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        "src/lib.rs",
        r#"
xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

mod foo;
"#,
    );
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
        "use super::*;\n#[test]\nfn helper_exists() { helper(); }\n",
    );
    write_fixture_file(
        temp.path(),
        "tests/unit/lib_policy.rs",
        "xiuxian_testing::crate_test_policy_harness!();\n",
    );

    let report = validate_crate_test_policy_harness(temp.path())
        .unwrap_or_else(|error| panic!("harness validation should succeed: {error}"));
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn validate_crate_test_policy_harness_collects_path_structure_warnings() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        "src/gateway/studio/router/config/router/mod.rs",
        "mod bootstrap;\n",
    );

    let report = validate_crate_test_policy_harness(temp.path())
        .unwrap_or_else(|error| panic!("harness validation should succeed: {error}"));
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.path_structure_warnings.len(), 1);
    assert_eq!(
        report.path_structure_warnings[0].repeated_namespaces,
        vec!["router".to_string()]
    );

    let formatted = format_crate_test_policy_harness_report(&report);
    assert!(formatted.contains("Crate Path Structure Warnings"));
    assert!(formatted.contains("repeated namespace segments: `router`"));
}
