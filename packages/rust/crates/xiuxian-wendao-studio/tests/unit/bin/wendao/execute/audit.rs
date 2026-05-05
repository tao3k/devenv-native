use super::{
    AuthoringTemplate, EpistemeTemplateManifest, collect_authoring_templates,
    render_authoring_template, validate_authoring_template,
};
use crate::bin_support::wendao::types::{Cli, Command};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

fn write_imported_template_registration(episteme: &Path, template_dir: &Path) {
    fs::write(
        episteme.join("episteme.toml"),
        r#"
[imports]
policy_manifests = ["policies/johnny_decimal/manifest.toml"]
"#,
    )
    .unwrap_or_else(|error| panic!("write manifest: {error}"));
    fs::write(
        template_dir.join("manifest.toml"),
        r#"
[[authoring_templates]]
id = "johnny-decimal.path-first-authoring-template"
framework = "johnny-decimal"
path = "authoring_template.toml"
"#,
    )
    .unwrap_or_else(|error| panic!("write manifest: {error}"));
}

#[test]
fn render_authoring_template_reads_registered_framework_template() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let episteme = temp.path().join("wendao-episteme");
    let template_dir = episteme.join("policies/johnny_decimal");
    fs::create_dir_all(&template_dir)
        .unwrap_or_else(|error| panic!("create template dir: {error}"));
    write_imported_template_registration(&episteme, &template_dir);
    fs::write(
        template_dir.join("authoring_template.toml"),
        r#"
id = "johnny-decimal.path-first-authoring-template"
framework = "johnny-decimal"
mode = "path_first"
coordinate_source = "path"
topology_source = "topology.toml"
authority = "template_guided_generation"
llm_role = "proposal_planner"
authority_rule = "topology.toml, existing path index, and deterministic audit diagnostics outrank LLM suggestions."

[paths]
category_directory_shape = "XX_category_slug"
note_file_shape = "XX.YY_semantic_name.md"
coordinate_shape = "XX.YY_semantic_name"
inbox_directory = "00_inbox"
inbox_policy = "Unassigned drafts may live under the inbox without a coordinate until reviewed."

[[diagnostics]]
code = "invalid_coordinate_shape"
severity = "error"
meaning = "A path-first note coordinate does not match XX.YY_semantic_name."
llm_action = "Propose a conforming filename without changing category authority."

[[llm_inputs]]
name = "topology_manifest"
description = "Project-local topology.toml category catalog."

[[llm_outputs]]
name = "reviewable_move_plan"
description = "A JSON proposal for create, move, or rename actions. It is not authority."

[[examples]]
kind = "valid_path"
path = "docs/10_wendao/10.03_audit_template_flow.md"

[[verification]]
phase = "post_generation"
command = "wendao lint markdown docs"
"#,
    )
    .unwrap_or_else(|error| panic!("write template: {error}"));

    let root_arg = temp.path().to_string_lossy().into_owned();
    let cli = Cli::parse_from([
        "wendao",
        "--root",
        root_arg.as_str(),
        "audit",
        "--template",
        "johnny-decimal",
    ]);
    let Command::Audit(args) = &cli.command else {
        panic!("expected audit command");
    };

    let rendered = render_authoring_template(&cli, args)
        .unwrap_or_else(|error| panic!("render template: {error}"));

    insta::assert_snapshot!(rendered, @r###"
Wendao audit template for LLM: johnny-decimal.path-first-authoring-template
framework: johnny-decimal
mode: path_first
coordinate_source: path
topology_source: topology.toml
authority: template_guided_generation
llm_role: proposal_planner

authority rule:
  topology.toml, existing path index, and deterministic audit diagnostics outrank LLM suggestions.

path contract:
  category_directory_shape: XX_category_slug
  note_file_shape: XX.YY_semantic_name.md
  coordinate_shape: XX.YY_semantic_name
  inbox_directory: 00_inbox
  inbox_policy: Unassigned drafts may live under the inbox without a coordinate until reviewed.

deterministic diagnostics:
error[invalid_coordinate_shape]: A path-first note coordinate does not match XX.YY_semantic_name.
  help: Propose a conforming filename without changing category authority.

llm inputs:
  topology_manifest: Project-local topology.toml category catalog.

llm outputs:
  reviewable_move_plan: A JSON proposal for create, move, or rename actions. It is not authority.

examples:
  valid_path: docs/10_wendao/10.03_audit_template_flow.md

verification:
  post_generation: wendao lint markdown docs
"###);
}

#[test]
fn render_authoring_template_accepts_direct_manifest_load_path() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let episteme = temp.path();
    let template_dir = episteme.join("policies/johnny_decimal");
    fs::create_dir_all(&template_dir)
        .unwrap_or_else(|error| panic!("create template dir: {error}"));
    write_imported_template_registration(episteme, &template_dir);
    fs::write(
        template_dir.join("authoring_template.toml"),
        r#"
id = "johnny-decimal.path-first-authoring-template"
framework = "johnny-decimal"
mode = "path_first"
coordinate_source = "path"
topology_source = "topology.toml"
authority = "template_guided_generation"
llm_role = "proposal_planner"
authority_rule = "topology.toml, existing path index, and deterministic audit diagnostics outrank LLM suggestions."

[paths]
category_directory_shape = "XX_category_slug"
note_file_shape = "XX.YY_semantic_name.md"
coordinate_shape = "XX.YY_semantic_name"
inbox_directory = "00_inbox"
inbox_policy = "Unassigned drafts may live under the inbox without a coordinate until reviewed."

[[diagnostics]]
code = "invalid_coordinate_shape"
severity = "error"
meaning = "A path-first note coordinate does not match XX.YY_semantic_name."
llm_action = "Propose a conforming filename without changing category authority."

[[llm_inputs]]
name = "topology_manifest"
description = "Project-local topology.toml category catalog."

[[llm_outputs]]
name = "reviewable_move_plan"
description = "A JSON proposal for create, move, or rename actions. It is not authority."

[[examples]]
kind = "valid_path"
path = "docs/10_wendao/10.03_audit_template_flow.md"

[[verification]]
phase = "post_generation"
command = "wendao lint markdown docs"
"#,
    )
    .unwrap_or_else(|error| panic!("write template: {error}"));

    let manifest_arg = episteme
        .join("episteme.toml")
        .to_string_lossy()
        .into_owned();
    let cli = Cli::parse_from([
        "wendao",
        "audit",
        "--load",
        manifest_arg.as_str(),
        "--template",
        "johnny-decimal.path-first-authoring-template",
    ]);
    let Command::Audit(args) = &cli.command else {
        panic!("expected audit command");
    };

    let rendered = render_authoring_template(&cli, args)
        .unwrap_or_else(|error| panic!("render template: {error}"));

    assert!(rendered.contains("coordinate_source: path"));
}

#[test]
fn render_authoring_template_rejects_incomplete_llm_contract() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let episteme = temp.path().join("wendao-episteme");
    let template_dir = episteme.join("policies/johnny_decimal");
    fs::create_dir_all(&template_dir)
        .unwrap_or_else(|error| panic!("create template dir: {error}"));
    write_imported_template_registration(&episteme, &template_dir);
    fs::write(
        template_dir.join("authoring_template.toml"),
        r#"
id = "johnny-decimal.path-first-authoring-template"
framework = "johnny-decimal"
mode = "path_first"
"#,
    )
    .unwrap_or_else(|error| panic!("write template: {error}"));

    let root_arg = temp.path().to_string_lossy().into_owned();
    let cli = Cli::parse_from([
        "wendao",
        "--root",
        root_arg.as_str(),
        "audit",
        "--template",
        "johnny-decimal",
    ]);
    let Command::Audit(args) = &cli.command else {
        panic!("expected audit command");
    };

    let error = match render_authoring_template(&cli, args) {
        Ok(rendered) => panic!("incomplete authoring template should fail, rendered: {rendered}"),
        Err(error) => error,
    };
    let error = format!("{error:#}");

    assert!(error.contains("missing required LLM contract fields"));
    assert!(error.contains("diagnostics"));
    assert!(error.contains("verification"));
}

#[test]
fn render_authoring_template_rejects_inline_root_template_registration() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let episteme = temp.path().join("wendao-episteme");
    fs::create_dir_all(&episteme).unwrap_or_else(|error| panic!("create episteme dir: {error}"));
    fs::write(
        episteme.join("episteme.toml"),
        r#"
[imports]
policy_manifests = ["policies/johnny_decimal/manifest.toml"]

[[authoring_templates]]
id = "johnny-decimal.path-first-authoring-template"
framework = "johnny-decimal"
path = "policies/johnny_decimal/authoring_template.toml"
"#,
    )
    .unwrap_or_else(|error| panic!("write manifest: {error}"));

    let root_arg = temp.path().to_string_lossy().into_owned();
    let cli = Cli::parse_from([
        "wendao",
        "--root",
        root_arg.as_str(),
        "audit",
        "--template",
        "johnny-decimal",
    ]);
    let Command::Audit(args) = &cli.command else {
        panic!("expected audit command");
    };

    let error = match render_authoring_template(&cli, args) {
        Ok(rendered) => panic!("inline authoring template should fail, rendered: {rendered}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("inline authoring_templates"));
}

#[test]
fn render_authoring_template_snapshots_all_episteme_theory_outputs() {
    let episteme_root = workspace_root().join("wendao-episteme");
    let manifest_path = episteme_root.join("episteme.toml");
    let manifest_text = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("read episteme manifest: {error}"));
    let manifest: EpistemeTemplateManifest = toml::from_str(&manifest_text)
        .unwrap_or_else(|error| panic!("parse episteme manifest: {error}"));
    let mut entries = collect_authoring_templates(&episteme_root, &manifest_path, manifest)
        .unwrap_or_else(|error| panic!("collect authoring templates: {error:#}"));
    entries.sort_by(|left, right| left.framework.cmp(&right.framework));

    let frameworks = entries
        .iter()
        .map(|entry| entry.framework.as_str())
        .collect::<Vec<_>>();
    insta::assert_debug_snapshot!(frameworks, @r###"
[
    "adr",
    "diataxis",
    "epistemic-sensemaking",
    "evergreen-notes",
    "folgezettel",
    "ibis",
    "johnny-decimal",
    "moc",
    "search-reasoning",
    "semantic-consistency",
    "structural-proprioception",
    "temporal-scaffolding",
]
"###);

    let load_arg = episteme_root.to_string_lossy().into_owned();
    let mut compact_outputs = String::new();
    for entry in entries {
        let template = parse_episteme_template(&episteme_root, entry.path.as_str());
        validate_authoring_template(&template).unwrap_or_else(|error| {
            panic!(
                "authoring template `{}` should be LLM-complete: {error:#}",
                entry.id
            )
        });

        let cli = Cli::parse_from([
            "wendao",
            "audit",
            "--load",
            load_arg.as_str(),
            "--template",
            entry.framework.as_str(),
        ]);
        let Command::Audit(args) = &cli.command else {
            panic!("expected audit command");
        };

        let rendered = render_authoring_template(&cli, args).unwrap_or_else(|error| {
            panic!(
                "render episteme template `{}` for framework `{}`: {error:#}",
                entry.id, entry.framework
            )
        });
        assert_rendered_template_is_complete(&entry.id, &entry.framework, &template, &rendered);

        compact_outputs.push_str("----- ");
        compact_outputs.push_str(entry.framework.as_str());
        compact_outputs.push_str(" -----\n");
        compact_outputs.push_str(&rendered);
        if !rendered.ends_with('\n') {
            compact_outputs.push('\n');
        }
    }

    insta::with_settings!({
        snapshot_path => concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/wendao/audit"),
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("episteme_authoring_templates_compact", compact_outputs);
    });
}

fn parse_episteme_template(episteme_root: &Path, relative_path: &str) -> AuthoringTemplate {
    let template_path = episteme_root.join(relative_path);
    let template_text = fs::read_to_string(&template_path)
        .unwrap_or_else(|error| panic!("read template `{}`: {error}", template_path.display()));
    toml::from_str(&template_text)
        .unwrap_or_else(|error| panic!("parse template `{}`: {error}", template_path.display()))
}

fn assert_rendered_template_is_complete(
    template_id: &str,
    framework: &str,
    template: &AuthoringTemplate,
    rendered: &str,
) {
    for required_section in [
        "authority rule:",
        "path contract:",
        "deterministic diagnostics:",
        "llm inputs:",
        "llm outputs:",
        "examples:",
        "verification:",
    ] {
        assert!(
            rendered.contains(required_section),
            "template `{template_id}` for `{framework}` omitted compact section `{required_section}`"
        );
    }
    assert!(
        rendered.contains(&format!("framework: {framework}")),
        "template `{template_id}` rendered the wrong framework"
    );
    assert!(
        rendered.contains(&format!("mode: {}", template.mode)),
        "template `{template_id}` omitted mode"
    );
    assert!(
        rendered.contains(&format!(
            "coordinate_source: {}",
            template.coordinate_source
        )),
        "template `{template_id}` omitted coordinate source"
    );
    assert!(
        rendered.contains(&format!("topology_source: {}", template.topology_source)),
        "template `{template_id}` omitted topology source"
    );
    assert!(
        rendered.contains(&format!("authority: {}", template.authority)),
        "template `{template_id}` omitted authority"
    );
    assert!(
        rendered.contains(&format!("llm_role: {}", template.llm_role)),
        "template `{template_id}` omitted LLM role"
    );

    for diagnostic in &template.diagnostics {
        assert!(
            rendered.contains(&format!("{}[{}]", diagnostic.severity, diagnostic.code)),
            "template `{template_id}` omitted diagnostic `{}`",
            diagnostic.code
        );
        assert!(
            rendered.contains(&format!("  help: {}", diagnostic.llm_action)),
            "template `{template_id}` omitted help for diagnostic `{}`",
            diagnostic.code
        );
    }
    for item in &template.llm_inputs {
        assert!(
            rendered.contains(&format!("  {}: {}", item.name, item.description)),
            "template `{template_id}` omitted LLM input `{}`",
            item.name
        );
    }
    for item in &template.llm_outputs {
        assert!(
            rendered.contains(&format!("  {}: {}", item.name, item.description)),
            "template `{template_id}` omitted LLM output `{}`",
            item.name
        );
    }
    for example in &template.examples {
        assert!(
            rendered.contains(&format!("  {}: {}", example.kind, example.path)),
            "template `{template_id}` omitted example `{}`",
            example.kind
        );
    }
    for step in &template.verification {
        assert!(
            rendered.contains(&format!("  {}: {}", step.phase, step.command)),
            "template `{template_id}` omitted verification phase `{}`",
            step.phase
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("resolve workspace root from CARGO_MANIFEST_DIR"))
        .to_path_buf()
}
