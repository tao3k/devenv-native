use include_dir::{Dir, include_dir};

use super::scan::{is_markdown_file, normalize_registry_key, semantic_skill_name_from_descriptor};
use super::semantic::semantic_lift_target;
use super::types::WendaoResourceRegistry;

fn embedded_fixture() -> &'static Dir<'static> {
    static FIXTURE: Dir<'_> =
        include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/embedded-registry/wendao-uri");
    &FIXTURE
}

#[test]
fn registry_helpers_normalize_registry_paths() {
    assert!(is_markdown_file("skill.md"));
    assert!(is_markdown_file("note.markdown"));
    assert!(!is_markdown_file("note.txt"));
    assert_eq!(normalize_registry_key("./foo\\bar.md"), "foo/bar.md");
}

#[test]
fn registry_helpers_lift_skill_reference_targets() {
    let lifted = semantic_lift_target(
        "assets/skills/agenda-management/references/steward.md",
        "assets/skills/agenda-management/SKILL.md",
        Some("agenda-management"),
    );
    assert_eq!(
        lifted,
        "wendao://skills/agenda-management/references/steward.md"
    );
}

#[test]
fn registry_helpers_extract_skill_semantic_name_from_skill_md() {
    let markdown = "---\nname: agenda-management\nmetadata:\n  version: \"1.0.0\"\n---\n# Skill\n";
    let name =
        semantic_skill_name_from_descriptor("assets/skills/agenda-management/SKILL.md", markdown);
    assert_eq!(name.as_deref(), Some("agenda-management"));
}

#[test]
fn registry_helpers_extract_skill_semantic_name_from_kind_marked_doc() {
    let markdown = concat!(
        "---\n",
        "kind: SKILL.md\n",
        "name: planner\n",
        "metadata:\n",
        "  version: \"1.0.0\"\n",
        "---\n",
        "# Planner\n",
    );
    let name = semantic_skill_name_from_descriptor("docs/planner.md", markdown);
    assert_eq!(name.as_deref(), Some("planner"));
}

#[test]
fn embedded_registry_builds_from_fixture() {
    let registry = WendaoResourceRegistry::build_from_embedded(embedded_fixture())
        .unwrap_or_else(|error| panic!("embedded registry should build: {error}"));
    assert!(registry.files_len() > 0);
    assert!(
        registry
            .file("zhixing/skills/agenda-management/SKILL.md")
            .is_some()
    );
}
