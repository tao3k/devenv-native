use super::*;

#[test]
fn test_framework_new() {
    let framework = ScenarioFramework::new();
    assert!(framework.find_runner("nonexistent").is_none());
    assert!(framework.snapshot_policy().sort_maps());
    assert!(!framework.snapshot_policy().include_description());
    assert!(!framework.snapshot_policy().include_info());
}

#[test]
fn test_scenario_config_default() {
    let config = RunnerConfig::default();
    assert!(config.build_page_index.is_none());
    assert!(config.collect_links.is_none());
}

#[test]
fn test_scenarios_root_returns_crate_local() {
    let root = scenarios_root();
    assert!(
        root.ends_with("tests/scenarios"),
        "scenarios_root should end with tests/scenarios: {root:?}"
    );
}

#[test]
fn run_all_at_fails_when_scenario_ids_collide() {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    write_scenario_fixture_with_id(temp.path(), "001_first", "001_collision", "collision_case");
    write_scenario_fixture_with_id(temp.path(), "002_second", "001_collision", "collision_case");

    let framework = ScenarioFramework::new();
    let Err(error) = framework.run_all_at(temp.path()) else {
        panic!("duplicate scenario ids should fail closed");
    };
    let message = error.to_string();

    assert!(
        message.contains("Duplicate scenario id '001_collision'"),
        "unexpected error: {error}"
    );
    assert!(
        message.contains("001_first") && message.contains("002_second"),
        "duplicate path context should be present: {error}"
    );
}

#[test]
fn test_discover_scenarios_returns_local() {
    let scenarios = discover_scenarios();
    for scenario in &scenarios {
        assert!(
            scenario.to_string_lossy().contains("tests/scenarios"),
            "Scenario should be in crate-local path: {scenario:?}"
        );
    }
}

#[test]
fn run_all_at_fails_when_scenario_has_no_registered_runner() {
    let temp = match tempfile::tempdir() {
        Ok(temp) => temp,
        Err(error) => panic!("tempdir should be created: {error}"),
    };
    write_scenario_fixture(temp.path(), "001_missing_runner", "missing_runner");

    let framework = ScenarioFramework::new();
    let Err(error) = framework.run_all_at(temp.path()) else {
        panic!("missing runner should fail closed");
    };

    assert!(
        error
            .to_string()
            .contains("No runner registered for scenario category 'missing_runner'"),
        "unexpected error: {error}"
    );
}

#[test]
fn framework_can_replace_snapshot_policy() {
    let mut framework = ScenarioFramework::new();
    let mut policy = ScenarioSnapshotPolicy::recommended();
    policy.set_sort_maps(false);
    framework.set_snapshot_policy(policy.clone());

    assert_eq!(framework.snapshot_policy(), &policy);
    assert!(!framework.snapshot_policy().sort_maps());
}
