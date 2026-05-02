//! Contract tests for Flowhub module/scenario manifest parsing and validation.

use std::fs;
use std::path::PathBuf;

#[path = "../support/workspace.rs"]
mod workspace;
use tempfile::TempDir;
use xiuxian_qianji::contracts::{FlowhubGraphTopology, TemplateLinkRef, TemplateUseSpec};
use xiuxian_qianji::{
    load_flowhub_module_manifest, load_flowhub_scenario_manifest, parse_flowhub_module_manifest,
    parse_flowhub_scenario_manifest, resolve_flowhub_module_children,
    resolve_flowhub_scenario_modules,
};

fn repo_root() -> PathBuf {
    workspace::workspace_root()
}

fn flowhub_root() -> PathBuf {
    repo_root().join("qianji-flowhub")
}

fn real_flowhub_fixture_available() -> bool {
    flowhub_root().join("qianji.toml").is_file()
}

fn flowhub_module_manifest(module_ref: &str) -> String {
    let path = flowhub_root().join(module_ref).join("qianji.toml");
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "should read Flowhub module manifest {}: {error}",
            path.display()
        )
    })
}

fn scenario_fixture_path(name: &str) -> PathBuf {
    repo_root().join(format!(
        "packages/rust/crates/xiuxian-qianji/tests/fixtures/flowhub/{name}/qianji.toml"
    ))
}

fn write_temp_scenario_manifest(temp_dir: &TempDir, content: &str) -> PathBuf {
    let manifest_path = temp_dir.path().join("qianji.toml");
    fs::write(&manifest_path, content)
        .unwrap_or_else(|error| panic!("should write scenario manifest: {error}"));
    manifest_path
}

#[test]
fn flowhub_rust_module_manifest_parses_as_leaf_node() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest = parse_flowhub_module_manifest(&flowhub_module_manifest("rust"))
        .unwrap_or_else(|error| panic!("rust module manifest should parse: {error}"));

    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.module.name, "rust");
    assert_eq!(manifest.exports.entry, "task.rust-start");
    assert_eq!(manifest.exports.ready, "task.constraints-ready");
    assert!(manifest.contract.is_none());
    assert!(manifest.graph.is_empty());
    assert!(manifest.validation.is_empty());
    assert!(manifest.template.is_none());
}

#[test]
fn template_use_spec_parses_hierarchical_module_refs() {
    let spec: TemplateUseSpec = "rust as rust"
        .parse()
        .unwrap_or_else(|error| panic!("template.use grammar should parse: {error}"));

    assert_eq!(spec.module_ref, "rust");
    assert_eq!(spec.alias, "rust");
    assert_eq!(spec.to_string(), "rust as rust");
}

#[test]
fn template_link_ref_parses_aliased_reference() {
    let link_ref: TemplateLinkRef = "blueprint::task.blueprint-ready"
        .parse()
        .unwrap_or_else(|error| panic!("template.link reference should parse: {error}"));

    assert_eq!(link_ref.alias.as_deref(), Some("blueprint"));
    assert_eq!(link_ref.symbol, "task.blueprint-ready");
    assert_eq!(link_ref.to_string(), "blueprint::task.blueprint-ready");
}

#[test]
fn template_link_ref_parses_local_reference() {
    let link_ref: TemplateLinkRef = "task.constraints-ready"
        .parse()
        .unwrap_or_else(|error| panic!("local template.link reference should parse: {error}"));

    assert_eq!(link_ref.alias, None);
    assert_eq!(link_ref.symbol, "task.constraints-ready");
    assert_eq!(link_ref.to_string(), "task.constraints-ready");
}

#[test]
fn load_flowhub_module_manifest_reads_real_leaf_file() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest = load_flowhub_module_manifest(flowhub_root().join("blueprint/qianji.toml"))
        .unwrap_or_else(|error| panic!("module manifest file should load: {error}"));

    assert_eq!(manifest.module.name, "blueprint");
    assert_eq!(manifest.exports.ready, "task.blueprint-ready");
    assert!(manifest.contract.is_none());
    assert!(manifest.graph.is_empty());
    assert!(manifest.template.is_none());
    assert!(manifest.validation.is_empty());
}

#[test]
fn load_flowhub_scenario_manifest_reads_fixture_file() {
    let manifest =
        load_flowhub_scenario_manifest(scenario_fixture_path("coding_rust_blueprint_plan"))
            .unwrap_or_else(|error| panic!("scenario manifest file should load: {error}"));

    assert_eq!(manifest.planning.name, "coding-rust-blueprint-plan-demo");
    assert_eq!(manifest.template.use_entries.len(), 4);
    assert_eq!(manifest.template.link.len(), 3);
}

#[test]
fn resolve_flowhub_scenario_modules_matches_hierarchical_demo() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest =
        load_flowhub_scenario_manifest(scenario_fixture_path("coding_rust_blueprint_plan"))
            .unwrap_or_else(|error| panic!("scenario manifest file should load: {error}"));

    let resolved = resolve_flowhub_scenario_modules(flowhub_root(), &manifest)
        .unwrap_or_else(|error| panic!("scenario modules should resolve: {error}"));

    assert_eq!(resolved.len(), 4);
    assert_eq!(resolved[0].alias, "coding");
    assert_eq!(resolved[0].module_ref, "coding");
    assert_eq!(resolved[1].alias, "rust");
    assert_eq!(resolved[1].module_ref, "rust");
    assert_eq!(resolved[1].module_name, "rust");
    assert_eq!(resolved[2].module_ref, "blueprint");
    assert_eq!(resolved[2].manifest.exports.entry, "task.blueprint-start");
    assert_eq!(resolved[3].module_ref, "plan");
    assert_eq!(resolved[3].manifest.exports.ready, "task.plan-ready");
}

#[test]
fn resolve_flowhub_module_children_returns_empty_for_leaf_rust() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let rust_manifest = load_flowhub_module_manifest(flowhub_root().join("rust/qianji.toml"))
        .unwrap_or_else(|error| panic!("rust module manifest should load: {error}"));
    let rust_module = xiuxian_qianji::ResolvedFlowhubModule {
        alias: "rust".to_string(),
        module_ref: "rust".to_string(),
        module_name: rust_manifest.module.name.clone(),
        module_dir: flowhub_root().join("rust"),
        manifest_path: flowhub_root().join("rust/qianji.toml"),
        manifest: rust_manifest,
    };

    let resolved = resolve_flowhub_module_children(&rust_module)
        .unwrap_or_else(|error| panic!("leaf rust module should not resolve children: {error}"));

    assert!(resolved.is_empty());
}

#[test]
fn flowhub_scenario_manifest_rejects_invalid_template_use_grammar() {
    let error = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "broken-scenario"

[template]
use = ["blueprint"]
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("invalid template.use item should fail"));

    let message = error.to_string();
    assert!(message.contains("template.use"));
    assert!(message.contains("<module-ref> as <alias>"));
}

#[test]
fn flowhub_scenario_manifest_rejects_module_ref_traversal() {
    let error = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "path-like-module"

[template]
use = ["../blueprint as blueprint"]
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("traversal refs should fail"));

    let message = error.to_string();
    assert!(message.contains("template.use"));
    assert!(message.contains("invalid path segment"));
}

#[test]
fn flowhub_scenario_manifest_rejects_empty_use_entries() {
    let error = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "empty-use"

[template]
use = []
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("empty template.use should fail"));

    assert!(
        error
            .to_string()
            .contains("requires at least one `template.use` entry")
    );
}

#[test]
fn flowhub_scenario_manifest_rejects_duplicate_aliases() {
    let error = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "duplicate-aliases"

[template]
use = [
  "coding as coding",
  "rust as rust",
  "blueprint as rust",
]
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("duplicate aliases should fail"));

    assert!(
        error
            .to_string()
            .contains("duplicate template.use alias `rust`")
    );
}

#[test]
fn flowhub_scenario_manifest_rejects_unknown_link_alias() {
    let error = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "unknown-alias"

[template]
use = [
  "blueprint as blueprint",
]

[[template.link]]
from = "blueprint::task.blueprint-ready"
to = "plan::task.plan-start"
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("unknown link alias should fail"));

    assert!(
        error
            .to_string()
            .contains("unknown template.link alias `plan`")
    );
}

#[test]
fn flowhub_scenario_manifest_rejects_local_link_refs() {
    let error = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "local-link-ref"

[template]
use = [
  "coding as coding",
  "rust as rust",
  "blueprint as blueprint",
]

[[template.link]]
from = "task.constraints-ready"
to = "blueprint::task.blueprint-start"
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("local scenario link refs should fail"));

    assert!(error.to_string().contains("must use `<alias>::<symbol>`"));
}

#[test]
fn resolve_flowhub_scenario_modules_rejects_missing_modules() {
    let manifest = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "missing-module"

[template]
use = ["coding/missing as missing"]
"#,
    )
    .unwrap_or_else(|error| panic!("scenario manifest should parse before resolve: {error}"));

    let error = resolve_flowhub_scenario_modules(flowhub_root(), &manifest)
        .err()
        .unwrap_or_else(|| panic!("missing module resolution should fail"));

    assert!(error.to_string().contains("coding/missing"));
}

#[test]
fn resolve_flowhub_scenario_modules_rejects_manifest_name_mismatch() {
    let temp_dir = TempDir::new().unwrap_or_else(|error| panic!("temp dir should exist: {error}"));
    let module_dir = temp_dir.path().join("rust");
    fs::create_dir_all(&module_dir)
        .unwrap_or_else(|error| panic!("module dir should be created: {error}"));
    fs::write(
        module_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "wrong-name"

[exports]
entry = "task.rust-start"
ready = "task.constraints-ready"
"#,
    )
    .unwrap_or_else(|error| panic!("module manifest should be written: {error}"));

    let manifest = parse_flowhub_scenario_manifest(
        r#"
version = 1

[planning]
name = "name-mismatch"

[template]
use = ["rust as rust"]
"#,
    )
    .unwrap_or_else(|error| panic!("scenario manifest should parse before resolve: {error}"));

    let error = resolve_flowhub_scenario_modules(temp_dir.path(), &manifest)
        .err()
        .unwrap_or_else(|| panic!("module name mismatch should fail"));

    assert!(
        error
            .to_string()
            .contains("mismatched `module.name = \"wrong-name\"`")
    );
}

#[test]
fn load_flowhub_scenario_manifest_reads_written_file() {
    let temp_dir = TempDir::new().unwrap_or_else(|error| panic!("temp dir should exist: {error}"));
    let manifest_path = write_temp_scenario_manifest(
        &temp_dir,
        r#"
version = 1

[planning]
name = "fixture-roundtrip"

[template]
use = [
  "coding as coding",
  "rust as rust",
  "blueprint as blueprint",
]

[[template.link]]
from = "coding::task.coding-ready"
to = "rust::task.rust-start"

[[template.link]]
from = "rust::task.constraints-ready"
to = "blueprint::task.blueprint-start"
"#,
    );

    let manifest = load_flowhub_scenario_manifest(&manifest_path)
        .unwrap_or_else(|error| panic!("scenario manifest file should load: {error}"));

    assert_eq!(manifest.template.use_entries[0].module_ref, "coding");
    assert_eq!(
        manifest.template.link[0].from.alias.as_deref(),
        Some("coding")
    );
}

mod graph_contracts;
