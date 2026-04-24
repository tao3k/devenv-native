use std::fs;
use std::path::Path;

use tempfile::{Builder, tempdir};

use super::{SkillWorkflowType, load_skill_manifest_from_path, resolve_skill_authority};

fn ok_or_panic<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn some_or_panic<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        panic!("create parent directories: {error}");
    }
    if let Err(error) = fs::write(path, content) {
        panic!("write file: {error}");
    }
}

#[test]
fn load_manifest_uses_defaults_and_overrides() {
    let dir = ok_or_panic(tempdir(), "tempdir");
    let path = dir.path().join("sample.toml");
    write_file(
        &path,
        r#"
name = "Sample Tool"
description = "Sample description"
binding_id = "sample-native"
tool_contract = { category = "filesystem" }
workflow_type = { type = "workflow" }
qianhuan_background = { background = "wendao://background" }
flow_definition = { uri = "flow://definition" }
annotations = { read_only = true, destructive = false, idempotent = true, open_world = false }
"#,
    );

    let manifest = ok_or_panic(load_skill_manifest_from_path(&path), "manifest");

    assert_eq!(manifest.manifest_id, "sample");
    assert_eq!(manifest.tool_name, "Sample Tool");
    assert_eq!(manifest.description, "Sample description");
    assert_eq!(manifest.binding_id, "sample-native");
    assert_eq!(manifest.workflow_type, SkillWorkflowType::QianjiFlow);
    assert_eq!(
        manifest.qianhuan_background.as_deref(),
        Some("wendao://background")
    );
    assert_eq!(
        manifest.flow_definition.as_deref(),
        Some("flow://definition")
    );
    assert_eq!(
        manifest.metadata,
        serde_json::json!({ "category": "filesystem" })
    );
    assert!(manifest.annotations.read_only);
    assert!(!manifest.annotations.destructive);
    assert!(manifest.annotations.is_idempotent());
    assert!(!manifest.annotations.is_open_world());
    assert_eq!(manifest.source_path, path);
}

#[test]
fn load_manifest_rejects_invalid_description() {
    let dir = ok_or_panic(tempdir(), "tempdir");
    let path = dir.path().join("sample.toml");
    write_file(
        &path,
        r#"
description = "invalid"
"#,
    );

    let Err(error) = load_skill_manifest_from_path(&path) else {
        panic!("expected failure");
    };
    assert!(error.to_string().contains("invalid description"));
}

#[test]
fn workflow_type_parser_recognizes_known_aliases() {
    assert_eq!(
        SkillWorkflowType::from_raw(None),
        SkillWorkflowType::QianjiFlow
    );
    assert_eq!(
        SkillWorkflowType::from_raw(Some("flow")),
        SkillWorkflowType::QianjiFlow
    );
    assert_eq!(
        SkillWorkflowType::from_raw(Some("native")),
        SkillWorkflowType::Native
    );
}

#[test]
fn resolve_authority_collects_authorized_ghost_and_unauthorized_manifests() {
    let dir = ok_or_panic(
        Builder::new().prefix("skill-manifest").tempdir_in("."),
        "tempdir",
    );
    let root = dir.path();
    let root_rel = Path::new(some_or_panic(root.file_name(), "tempdir name"));

    let alpha_root = root.join("alpha");
    write_file(
        &alpha_root.join("SKILL.md"),
        r"
[manifest](references/qianji.toml)
[ghost](references/missing/qianji.toml)
",
    );
    write_file(
        &alpha_root.join("references/qianji.toml"),
        r#"
manifest_id = "alpha-manifest"
name = "Alpha Tool"
"#,
    );

    let beta_root = root.join("beta");
    write_file(
        &beta_root.join("SKILL.md"),
        r"
beta skill without explicit manifest links
",
    );
    write_file(
        &beta_root.join("references/qianji.toml"),
        r#"
manifest_id = "beta-manifest"
name = "Beta Tool"
"#,
    );

    let outcome = ok_or_panic(resolve_skill_authority(root_rel), "authority outcome");

    assert_eq!(
        outcome.report.authorized_manifests,
        vec!["wendao://skills/alpha/references/qianji.toml"]
    );
    assert_eq!(
        outcome.report.ghost_links,
        vec!["wendao://skills/alpha/references/missing/qianji.toml"]
    );
    assert_eq!(
        outcome.report.unauthorized_manifests,
        vec!["wendao://skills/beta/references/qianji.toml"]
    );
    assert_eq!(outcome.authorized.len(), 1);
    assert_eq!(outcome.authorized[0].tool_name, "Alpha Tool");
    assert_eq!(
        outcome.authorized[0].workflow_type,
        SkillWorkflowType::QianjiFlow
    );
    assert!(
        outcome.authorized[0]
            .source_path
            .ends_with("alpha/references/qianji.toml")
    );
}

#[test]
fn resolve_authority_collects_manifest_intents_from_mixed_markdown_references() {
    let dir = ok_or_panic(
        Builder::new()
            .prefix("skill-manifest-wikilinks")
            .tempdir_in("."),
        "tempdir",
    );
    let root = dir.path();
    let root_rel = Path::new(some_or_panic(root.file_name(), "tempdir name"));

    let alpha_root = root.join("alpha");
    write_file(
        &alpha_root.join("SKILL.md"),
        r"
[Manifest](references/qianji.toml#flow)
[[references/qianji.toml|Manifest]]
[[references/missing/qianji.toml#draft|Ghost]]
[[#local-heading]]
",
    );
    write_file(
        &alpha_root.join("references/qianji.toml"),
        r#"
manifest_id = "alpha-manifest"
name = "Alpha Tool"
"#,
    );

    let outcome = ok_or_panic(resolve_skill_authority(root_rel), "authority outcome");

    assert_eq!(
        outcome.report.authorized_manifests,
        vec!["wendao://skills/alpha/references/qianji.toml"]
    );
    assert_eq!(
        outcome.report.ghost_links,
        vec!["wendao://skills/alpha/references/missing/qianji.toml"]
    );
    assert!(outcome.report.unauthorized_manifests.is_empty());
}

#[test]
fn resolve_authority_ignores_nested_skill_docs_when_building_root_catalog() {
    let dir = ok_or_panic(
        Builder::new()
            .prefix("skill-manifest-nested-skill-docs")
            .tempdir_in("."),
        "tempdir",
    );
    let root = dir.path();
    let root_rel = Path::new(some_or_panic(root.file_name(), "tempdir name"));

    let alpha_root = root.join("alpha");
    write_file(
        &alpha_root.join("SKILL.md"),
        "[Manifest](references/qianji.toml)\n",
    );
    write_file(
        &alpha_root.join("references/qianji.toml"),
        r#"
manifest_id = "alpha-manifest"
name = "Alpha Tool"
"#,
    );
    write_file(
        &alpha_root.join("nested").join("SKILL.md"),
        "[Nested](references/nested/qianji.toml)\n",
    );
    write_file(
        &alpha_root
            .join("nested")
            .join("references/nested/qianji.toml"),
        r#"
manifest_id = "nested-manifest"
name = "Nested Tool"
"#,
    );

    let outcome = ok_or_panic(resolve_skill_authority(root_rel), "authority outcome");

    assert_eq!(
        outcome.report.authorized_manifests,
        vec!["wendao://skills/alpha/references/qianji.toml"]
    );
    assert!(outcome.report.ghost_links.is_empty());
    assert!(outcome.report.unauthorized_manifests.is_empty());
}
