use super::*;

#[test]
fn validate_crate_test_policy_with_workspace_config_applies_overrides() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        "tests/coactivation_multihop_diffusion.rs",
        "#[test]\nfn smoke() {}\n",
    );
    write_fixture_file(
        temp.path(),
        "tests/bench/throughput.rs",
        "#[test]\nfn smoke() {}\n",
    );
    write_fixture_file(
        temp.path(),
        TEST_POLICY_CONFIG_FILE,
        r#"
[tests]
allowed_root_files = [
  { name = "coactivation_multihop_diffusion.rs", explanation = "Legacy root harness pending structured migration." },
]
allowed_directories = [
  { name = "bench", explanation = "Legacy benchmark directory pending performance harness migration." },
]
"#,
    );

    let report = validate_crate_test_policy_with_workspace_config(temp.path())
        .unwrap_or_else(|error| panic!("workspace-config validation should pass: {error}"));
    assert!(report.is_clean(), "expected clean report, got {report:?}");
}

#[test]
fn validate_crate_test_policy_with_workspace_config_rejects_invalid_toml() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        TEST_POLICY_CONFIG_FILE,
        r#"
[tests
allowed_root_files = ["coactivation_multihop_diffusion.rs"]
"#,
    );

    let Err(error) = validate_crate_test_policy_with_workspace_config(temp.path()) else {
        panic!("invalid toml should fail");
    };
    assert!(error.contains(TEST_POLICY_CONFIG_FILE));
}

#[test]
fn validate_crate_test_policy_with_workspace_config_rejects_missing_explanation() {
    let temp = create_temp_crate();
    write_fixture_file(
        temp.path(),
        TEST_POLICY_CONFIG_FILE,
        r#"
[tests]
allowed_root_files = [
  { name = "coactivation_multihop_diffusion.rs" },
]
"#,
    );

    let Err(error) = validate_crate_test_policy_with_workspace_config(temp.path()) else {
        panic!("missing explanation should fail");
    };
    assert!(error.contains("allowed_root_files"));
    assert!(error.contains("coactivation_multihop_diffusion.rs"));
    assert!(error.contains("explanation"));
}

#[test]
fn assert_crate_tests_structure_with_workspace_config_ignores_external_layer() {
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
        "tests/coactivation_weighted_propagation.rs",
        "#[test]\nfn smoke() {}\n",
    );
    write_fixture_file(
        temp.path(),
        TEST_POLICY_CONFIG_FILE,
        r#"
[tests]
allowed_root_files = [
  { name = "coactivation_weighted_propagation.rs", explanation = "Legacy root test harness kept temporarily at tests root." },
]
"#,
    );

    assert_crate_tests_structure_with_workspace_config(temp.path());
}

#[test]
fn assert_crate_test_policy_with_workspace_config_rejects_inline_test_blocks() {
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
        TEST_POLICY_CONFIG_FILE,
        r"
[tests]
allowed_root_files = []
allowed_directories = []
",
    );

    let Err(panic) = std::panic::catch_unwind(|| {
        assert_crate_test_policy_with_workspace_config(temp.path());
    }) else {
        panic!("full crate test policy should reject inline cfg(test) blocks");
    };

    let message = if let Some(message) = panic.downcast_ref::<String>() {
        message.as_str()
    } else if let Some(message) = panic.downcast_ref::<&str>() {
        message
    } else {
        panic!("panic payload should be a string message");
    };

    assert!(message.contains("External Test Policy"));
    assert!(message.contains("Inline cfg(test) module"));
    assert!(message.contains("../tests/unit/foo.rs"));
}
