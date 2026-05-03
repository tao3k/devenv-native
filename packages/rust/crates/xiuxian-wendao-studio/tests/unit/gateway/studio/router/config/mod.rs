use std::fs;

use crate::studio::router::{
    load_ui_config_from_wendao_toml, studio_wendao_overlay_toml_path, studio_wendao_toml_path,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn load_ui_config_from_wendao_toml_accepts_inline_repo_plugin_config() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("wendao.toml"),
        r#"[link_graph.projects.sample]
root = "."
plugins = [
  "julia",
  { id = "julia", flight_transport = { base_url = "http://127.0.0.1:8815" } }
]
"#,
    )?;

    let Some(config) = load_ui_config_from_wendao_toml(temp.path()) else {
        panic!("ui config should load");
    };
    assert_eq!(config.repo_projects.len(), 1);
    assert_eq!(config.repo_projects[0].id, "sample");
    assert_eq!(config.repo_projects[0].plugins, vec!["julia".to_string()]);
    Ok(())
}

#[test]
fn load_ui_config_from_wendao_toml_prefers_overlay_importing_base() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        studio_wendao_toml_path(temp.path()),
        r#"[link_graph.projects.kernel]
root = "."
dirs = ["docs"]
"#,
    )?;
    fs::write(
        studio_wendao_overlay_toml_path(temp.path()),
        r#"imports = ["wendao.toml"]

[link_graph.projects.kernel]
root = "."
dirs = ["docs", "src"]
"#,
    )?;

    let Some(config) = load_ui_config_from_wendao_toml(temp.path()) else {
        panic!("ui config should load from the persisted base config");
    };
    assert_eq!(config.projects.len(), 1);
    assert_eq!(config.projects[0].name, "kernel");
    assert_eq!(
        config.projects[0].dirs,
        vec!["docs".to_string(), "src".to_string()]
    );
    Ok(())
}
