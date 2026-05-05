use std::fs;
use std::path::{Path, PathBuf};

use crate::zhenfa_router::native::semantic_check::episteme::{
    EpistemeLoadError, load_episteme_manifest,
};

mod boundaries;
mod load;
mod real;
mod validation;

fn write_file(root: &Path, relative_path: &str, content: &str) {
    let path = root.join(relative_path);
    let parent = path
        .parent()
        .unwrap_or_else(|| panic!("missing parent for {}", path.display()));
    fs::create_dir_all(parent)
        .unwrap_or_else(|error| panic!("create {}: {error}", parent.display()));
    fs::write(&path, content).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn write_minimal_episteme(root: &Path, policy_sql: &str) {
    write_file(root, "policies/johnny_decimal/validation.sql", policy_sql);
    write_file(
        root,
        "policies/johnny_decimal/diagnostic.toml",
        "id = \"jd.diagnostic\"\n",
    );
    write_file(
        root,
        "policies/johnny_decimal/manifest.toml",
        r#"
[[policy_queries]]
id = "johnny-decimal.anchor-id-validation"
framework = "johnny-decimal"
path = "validation.sql"
statement_mode = "select_only"

[[diagnostic_mappings]]
id = "johnny-decimal.anchor-id-diagnostic"
query = "johnny-decimal.anchor-id-validation"
path = "diagnostic.toml"
"#,
    );
    write_file(root, "prompts/anchor_v3_fixers/fix_jd_id.txt", "Fix ID.\n");
    write_file(
        root,
        "prompts/anchor_v3_fixers/manifest.toml",
        r#"
[defaults]
repair_tooling = "Project AnchoR v3"

[[repair_prompts]]
id = "johnny-decimal.fix-anchor-id"
path = "fix_jd_id.txt"
"#,
    );
    write_file(
        root,
        "policies/authorship/diagnostic.toml",
        "id = \"guard\"\n",
    );
    write_file(
        root,
        "policies/authorship/manifest.toml",
        r#"
[[repair_guards]]
id = "temporal-scaffolding.authorship-boundary"
path = "diagnostic.toml"
"#,
    );
    write_file(root, "sources/johnny_decimal/sources.toml", "[[source]]\n");
    write_file(
        root,
        "sources/johnny_decimal/evolution.skill.md",
        "# Skill\n\nRun the source comparison.\n",
    );
    write_file(
        root,
        "sources/manifest.toml",
        r#"
[[source_evolution_skill_surfaces]]
id = "johnny-decimal.source-evolution"
sources_path = "johnny_decimal/sources.toml"
skill_path = "johnny_decimal/evolution.skill.md"
"#,
    );

    write_file(
        root,
        "episteme.toml",
        r#"
schema_version = 1
name = "test-episteme"

[sql]
statement_mode = "select_only"
forbidden_operations = ["CREATE", "ALTER", "DROP", "INSERT", "UPDATE", "DELETE"]

[imports]
policy_manifests = [
  "policies/johnny_decimal/manifest.toml",
  "policies/authorship/manifest.toml",
]
repair_prompt_manifest = "prompts/anchor_v3_fixers/manifest.toml"
source_evolution_manifest = "sources/manifest.toml"
"#,
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap_or_else(|| panic!("failed to derive workspace root from CARGO_MANIFEST_DIR"))
        .to_path_buf()
}
