use super::*;

#[test]
fn snapshot_policy_builds_metadata_rich_settings() {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    write_scenario_fixture(temp.path(), "001_policy", "policy_case");
    let scenario = match Scenario::load(temp.path().join("001_policy")) {
        Ok(scenario) => scenario,
        Err(error) => panic!("scenario should load: {error}"),
    };
    let snapshot_path = temp.path().join("snapshots");
    let policy = ScenarioSnapshotPolicy::recommended();

    let settings = policy.settings_for(&snapshot_path, &scenario);

    assert!(settings.sort_maps());
    assert!(!settings.prepend_module_to_snapshot());
    assert_eq!(settings.snapshot_path(), snapshot_path.as_path());
    assert_eq!(
        settings.description(),
        Some("Scenario 001_policy [policy_case]: Fixture Scenario")
    );
    assert_eq!(
        settings.input_file(),
        Some(scenario.dir.join("scenario.toml").as_path())
    );
    assert!(settings.has_info());
}

#[test]
fn snapshot_policy_portable_ci_omits_input_file_from_settings() {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    write_scenario_fixture(temp.path(), "001_portable", "portable_case");
    let scenario = match Scenario::load(temp.path().join("001_portable")) {
        Ok(scenario) => scenario,
        Err(error) => panic!("scenario should load: {error}"),
    };
    let snapshot_path = temp.path().join("snapshots");
    let policy = ScenarioSnapshotPolicy::portable_ci();

    let settings = policy.settings_for(&snapshot_path, &scenario);

    assert_eq!(settings.input_file(), None);
    assert!(settings.has_info());
    assert_eq!(
        settings.description(),
        Some("Scenario 001_portable [portable_case]: Fixture Scenario")
    );
}

#[test]
fn snapshot_policy_clears_disabled_metadata_from_parent_settings() {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    write_scenario_fixture(temp.path(), "001_clean", "clean_case");
    let scenario = match Scenario::load(temp.path().join("001_clean")) {
        Ok(scenario) => scenario,
        Err(error) => panic!("scenario should load: {error}"),
    };
    let snapshot_path = temp.path().join("snapshots");
    let policy = ScenarioSnapshotPolicy::portable_ci();
    let mut parent = insta::Settings::new();
    parent.set_description("parent description");
    parent.set_input_file(temp.path().join("parent.toml"));
    parent.set_info(&serde_json::json!({ "parent": true }));

    parent.bind(|| {
        let settings = policy.settings_for(&snapshot_path, &scenario);

        assert_eq!(settings.input_file(), None);
        assert_eq!(
            settings.description(),
            Some("Scenario 001_clean [clean_case]: Fixture Scenario")
        );
        assert!(settings.has_info());
    });
}

#[test]
fn snapshot_policy_new_removes_parent_metadata_when_disabled() {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    write_scenario_fixture(temp.path(), "001_minimal", "minimal_case");
    let scenario = match Scenario::load(temp.path().join("001_minimal")) {
        Ok(scenario) => scenario,
        Err(error) => panic!("scenario should load: {error}"),
    };
    let snapshot_path = temp.path().join("snapshots");
    let policy = ScenarioSnapshotPolicy::new();
    let mut parent = insta::Settings::new();
    parent.set_description("parent description");
    parent.set_input_file(temp.path().join("parent.toml"));
    parent.set_info(&serde_json::json!({ "parent": true }));

    parent.bind(|| {
        let settings = policy.settings_for(&snapshot_path, &scenario);

        assert_eq!(settings.description(), None);
        assert_eq!(settings.input_file(), None);
        assert!(!settings.has_info());
    });
}

#[test]
fn normalize_path_redaction_stabilizes_workspace_and_temp_prefixes() {
    let Some(workspace_root) = workspace_root() else {
        panic!("workspace root should be detected");
    };
    let workspace_path = workspace_root
        .join("packages")
        .join("rust")
        .join("crates")
        .join("xiuxian-testing")
        .to_string_lossy()
        .replace('/', "\\");
    let temp_path = std::env::temp_dir()
        .join("xiuxian-testing")
        .join("fixture.json")
        .to_string_lossy()
        .to_string();
    let mut settings = insta::Settings::new();
    ScenarioSnapshotRedaction::normalize_path(".workspace").apply(&mut settings);
    ScenarioSnapshotRedaction::normalize_path(".temp").apply(&mut settings);

    settings.bind(|| {
        insta::assert_json_snapshot!(
            serde_json::json!({
                "workspace": workspace_path,
                "temp": temp_path,
                "relative": "docs/alpha.md",
            }),
            @r#"
            {
              "relative": "docs/alpha.md",
              "temp": "[temp]/xiuxian-testing/fixture.json",
              "workspace": "[workspace]/packages/rust/crates/xiuxian-testing"
            }
            "#
        );
    });
}
