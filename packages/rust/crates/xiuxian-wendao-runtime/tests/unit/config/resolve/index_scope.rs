use super::resolve_link_graph_index_runtime_with_settings;
use crate::config::test_support;
use std::fs;

#[test]
fn resolve_index_runtime_filters_hidden_excludes_and_uses_auto_candidates()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    fs::create_dir_all(temp.path().join("src"))?;
    fs::create_dir_all(temp.path().join("docs"))?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[link_graph]
include_dirs_auto = true
include_dirs_auto_candidates = ["src", "docs", "missing"]
exclude_dirs = [".git", "target", "target"]
"#,
    )?;

    let settings = test_support::load_test_settings_from_path(&config_path)?;
    let runtime = resolve_link_graph_index_runtime_with_settings(temp.path(), &settings);
    assert_eq!(
        runtime.include_dirs,
        vec!["src".to_string(), "docs".to_string()]
    );
    assert_eq!(runtime.exclude_dirs, vec!["target".to_string()]);

    Ok(())
}

#[test]
fn resolve_index_runtime_accepts_explicit_link_graph_source_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    fs::create_dir_all(temp.path().join("docs"))?;
    fs::create_dir_all(temp.path().join("semantic"))?;
    fs::create_dir_all(
        temp.path()
            .join("packages/rust/crates/xiuxian-wendao/src/link_graph"),
    )?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[link_graph]
include_dirs = [
  "docs",
  "semantic",
  "packages/rust/crates/xiuxian-wendao",
  "packages/rust/crates/xiuxian-wendao/src/link_graph",
]
exclude_dirs = ["target"]
"#,
    )?;

    let settings = test_support::load_test_settings_from_path(&config_path)?;
    let runtime = resolve_link_graph_index_runtime_with_settings(temp.path(), &settings);
    assert_eq!(
        runtime.include_dirs,
        vec![
            "docs".to_string(),
            "semantic".to_string(),
            "packages/rust/crates/xiuxian-wendao".to_string(),
            "packages/rust/crates/xiuxian-wendao/src/link_graph".to_string(),
        ]
    );
    assert_eq!(runtime.exclude_dirs, vec!["target".to_string()]);

    Ok(())
}
