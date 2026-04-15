use super::*;
use std::fs;

fn write_scenario_fixture(root: &Path, name: &str, category: &str) {
    write_scenario_fixture_with_id(root, name, name, category);
}

fn write_scenario_fixture_with_id(root: &Path, dir_name: &str, id: &str, category: &str) {
    let scenario_dir = root.join(dir_name);
    if let Err(error) = fs::create_dir_all(&scenario_dir) {
        panic!("scenario dir should be created: {error}");
    }
    if let Err(error) = fs::write(
        scenario_dir.join("scenario.toml"),
        format!(
            r#"[scenario]
id = "{id}"
name = "Fixture Scenario"
description = "Fixture"
category = "{category}"

[input]
type = "json"
"#
        ),
    ) {
        panic!("scenario.toml should be written: {error}");
    }
}

mod framework;
mod snapshot_presets;
mod snapshot_settings;
