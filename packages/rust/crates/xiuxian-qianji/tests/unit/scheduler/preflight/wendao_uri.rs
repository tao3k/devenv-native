use super::resolve_wendao_uri_text;
use super::{LocalSkillRuntimeResolver, normalize_relative_path};
use crate::scheduler_preflight::mounts::{RuntimeWendaoMount, with_runtime_wendao_mounts};
use include_dir::{Dir, include_dir};
use std::fs;

static AGENDA_OVERRIDE_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/agenda_override/resources");

#[test]
fn normalize_relative_path_trims_prefix_and_separators() {
    assert_eq!(
        normalize_relative_path(" ./references\\\\qianji.toml "),
        "references/qianji.toml".to_string()
    );
}

#[test]
fn local_skill_runtime_resolver_reads_reference_from_skill_root() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| {
        panic!("temp skill root should be created: {error}");
    });
    let skill_dir = temp_dir.path().join("skills").join("local-skill");
    fs::create_dir_all(skill_dir.join("references")).unwrap_or_else(|error| {
        panic!("skill references directory should be created: {error}");
    });
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: local-skill\ntitle: Local Skill\n---\n# Local Skill\n",
    )
    .unwrap_or_else(|error| {
        panic!("skill descriptor should be written: {error}");
    });
    fs::write(
        skill_dir.join("references").join("qianji.toml"),
        "node = []",
    )
    .unwrap_or_else(|error| {
        panic!("skill reference should be written: {error}");
    });

    let resolver = LocalSkillRuntimeResolver::from_roots(&[temp_dir.path().to_path_buf()]);
    let content = resolver
        .read_utf8("wendao://skills/local-skill/references/qianji.toml")
        .unwrap_or_else(|error| panic!("local skill reference should resolve: {error}"));

    assert_eq!(content, "node = []");
}

#[tokio::test]
async fn runtime_mounts_are_task_local_to_bootcamp_scope() {
    const AGENDA_FLOW_URI: &str = "wendao://skills/agenda-management/references/agenda_flow.toml";

    let default_reference = resolve_wendao_uri_text(AGENDA_FLOW_URI)
        .unwrap_or_else(|| panic!("default embedded agenda flow should resolve"));
    assert!(default_reference.contains("Student_Ambition"));

    let mounted_reference = with_runtime_wendao_mounts(
        vec![RuntimeWendaoMount {
            semantic_name: "agenda-management",
            references_dir: "skills/agenda-management/references",
            dir: &AGENDA_OVERRIDE_RESOURCES,
        }],
        async {
            resolve_wendao_uri_text(AGENDA_FLOW_URI)
                .unwrap_or_else(|| panic!("mounted agenda flow should resolve"))
        },
    )
    .await;
    assert!(mounted_reference.contains("Agenda_Override_Mount_Test"));
    assert!(!mounted_reference.contains("Student_Ambition"));

    let default_after_scope = resolve_wendao_uri_text(AGENDA_FLOW_URI)
        .unwrap_or_else(|| panic!("default embedded agenda flow should still resolve"));
    assert!(default_after_scope.contains("Student_Ambition"));
}
