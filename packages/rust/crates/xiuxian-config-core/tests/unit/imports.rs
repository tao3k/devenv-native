//! Tests for recursive TOML imports and namespace-aware merge behavior.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_config_core::{
    ConfigCascadeSpec, ConfigCoreError, load_toml_value_with_imports,
    load_toml_value_with_imports_and_paths, resolve_and_merge_toml_with_paths,
};

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("create parent {}: {error}", parent.display()));
    }
    std::fs::write(path, content)
        .unwrap_or_else(|error| panic!("write fixture {}: {error}", path.display()));
}

fn temp_workspace() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("create temp workspace: {error}"));
    let root = temp.path().to_path_buf();
    std::fs::create_dir_all(root.join(".config/xiuxian-artisan-workshop"))
        .unwrap_or_else(|error| panic!("create .config/xiuxian-artisan-workshop: {error}"));
    (temp, root)
}

fn string_at<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

#[test]
fn load_toml_value_with_imports_preserves_import_order() {
    let (_temp, root) = temp_workspace();
    let config_path = root.join("config.toml");

    write_text(
        root.join("base.toml").as_path(),
        r#"
[llm]
default_model = "base"
"#,
    );
    write_text(
        root.join("override.toml").as_path(),
        r#"
[llm]
default_model = "override"
"#,
    );
    write_text(
        config_path.as_path(),
        r#"
imports = ["base.toml", "override.toml"]

[llm]
default_provider = "embedded"
"#,
    );

    let value = load_toml_value_with_imports(config_path.as_path())
        .unwrap_or_else(|error| panic!("load imported config: {error}"));

    assert_eq!(
        string_at(&value, &["llm", "default_model"]),
        Some("override")
    );
    assert_eq!(
        string_at(&value, &["llm", "default_provider"]),
        Some("embedded")
    );
}

#[test]
fn load_toml_value_with_imports_rejects_invalid_import_shapes() {
    let (_temp, root) = temp_workspace();
    let config_path = root.join("config.toml");

    write_text(
        config_path.as_path(),
        r#"
imports = "base.toml"
"#,
    );

    let error = match load_toml_value_with_imports(config_path.as_path()) {
        Ok(value) => panic!("expected invalid imports error, got {value:?}"),
        Err(error) => error,
    };

    match error {
        ConfigCoreError::InvalidImports { path, message } => {
            assert_eq!(path, config_path.display().to_string());
            assert!(message.contains("array"));
        }
        other => panic!("expected InvalidImports, got {other}"),
    }
}

#[test]
fn load_toml_value_with_imports_detects_cycles() {
    let (_temp, root) = temp_workspace();

    write_text(
        root.join("a.toml").as_path(),
        r#"
imports = ["b.toml"]

[a]
name = "alpha"
"#,
    );
    write_text(
        root.join("b.toml").as_path(),
        r#"
imports = ["a.toml"]

[b]
name = "beta"
"#,
    );

    let error = match load_toml_value_with_imports(root.join("a.toml").as_path()) {
        Ok(value) => panic!("expected import cycle error, got {value:?}"),
        Err(error) => error,
    };

    match error {
        ConfigCoreError::ImportCycle { chain } => {
            assert!(chain.contains("a.toml"));
            assert!(chain.contains("b.toml"));
            assert!(chain.contains("->"));
        }
        other => panic!("expected ImportCycle, got {other}"),
    }
}

#[test]
fn embedded_imports_are_resolved_with_source_path() {
    let (_temp, root) = temp_workspace();
    let embedded_source =
        root.join("packages/rust/crates/xiuxian-wendao/resources/config/wendao.toml");

    write_text(
        embedded_source
            .parent()
            .unwrap_or_else(|| panic!("embedded source should have a parent directory"))
            .join("shared.toml")
            .as_path(),
        r#"
[retrieval]
candidate_multiplier = 4

[retrieval.semantic_ignition]
backend = "openai-compatible"
"#,
    );

    let spec = ConfigCascadeSpec::new(
        "link_graph",
        r#"
[link_graph]
imports = ["shared.toml"]

[link_graph.retrieval]
mode = "hybrid"
"#,
        "wendao.toml",
    )
    .with_embedded_source_path(
        embedded_source
            .to_str()
            .unwrap_or_else(|| panic!("embedded source path must be valid UTF-8")),
    );

    let merged = resolve_and_merge_toml_with_paths(
        spec,
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve embedded imports: {error}"));

    assert_eq!(
        string_at(&merged, &["link_graph", "retrieval", "mode"]),
        Some("hybrid")
    );
    assert_eq!(
        merged
            .get("link_graph")
            .and_then(|value| value.get("retrieval"))
            .and_then(|value| value.get("candidate_multiplier"))
            .and_then(toml::Value::as_integer),
        Some(4)
    );
    assert_eq!(
        string_at(
            &merged,
            &["link_graph", "retrieval", "semantic_ignition", "backend"]
        ),
        Some("openai-compatible")
    );
}

#[test]
fn load_toml_value_with_imports_and_paths_expands_prj_root_variables() {
    let (_temp, root) = temp_workspace();
    let config_path = root.join("config.toml");

    write_text(
        root.join("imports/shared.toml").as_path(),
        r#"
[llm]
default_model = "root-import"
"#,
    );
    write_text(
        config_path.as_path(),
        r#"
imports = ["$PRJ_ROOT/imports/shared.toml"]

[llm]
default_provider = "embedded"
"#,
    );

    let merged = load_toml_value_with_imports_and_paths(
        config_path.as_path(),
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve imported config with paths: {error}"));

    assert_eq!(
        string_at(&merged, &["llm", "default_model"]),
        Some("root-import")
    );
    assert_eq!(
        string_at(&merged, &["llm", "default_provider"]),
        Some("embedded")
    );
}

#[test]
fn load_toml_value_with_imports_and_paths_expands_prj_skills_dir_default() {
    let (_temp, root) = temp_workspace();
    let config_path = root.join("config.toml");

    write_text(
        root.join("skills/shared.toml").as_path(),
        r#"
[skill]
name = "runtime-skill"
"#,
    );
    write_text(
        config_path.as_path(),
        r#"
imports = ["$PRJ_SKILLS_DIR/shared.toml"]
"#,
    );

    let merged = load_toml_value_with_imports_and_paths(
        config_path.as_path(),
        Some(root.as_path()),
        Some(root.join(".config").as_path()),
    )
    .unwrap_or_else(|error| panic!("resolve imported skill config with paths: {error}"));

    assert_eq!(
        string_at(&merged, &["skill", "name"]),
        Some("runtime-skill")
    );
}
