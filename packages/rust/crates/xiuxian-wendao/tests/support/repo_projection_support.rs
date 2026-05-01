use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::linked_parser_summary::linked_parser_summary_base_url;

pub type TestResultPath = Result<PathBuf, Box<dyn std::error::Error>>;

pub fn assert_repo_json_snapshot(name: &str, value: impl Serialize) {
    insta::with_settings!({
        snapshot_path => "../snapshots/repo_intelligence",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}

fn linked_julia_parser_summary_plugin_toml() -> Result<String, Box<dyn std::error::Error>> {
    let base_url = linked_parser_summary_base_url()?;
    Ok(format!(
        r#"{{ id = "julia", parser_summary_transport = {{ base_url = "{base_url}", file_summary = {{ schema_version = "v3" }}, root_summary = {{ schema_version = "v3" }} }} }}"#
    ))
}

pub fn write_repo_config(base: &Path, repo_dir: &Path, repo_id: &str) -> TestResultPath {
    let config_path = base.join(format!("{repo_id}.wendao.toml"));
    let plugin = linked_julia_parser_summary_plugin_toml()?;
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.{repo_id}]
root = "{}"
plugins = [{plugin}]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(config_path)
}
