use std::path::PathBuf;

use super::*;

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

#[test]
fn embedded_skill_markdown_loads_agenda_management_manifest() {
    let markdown = some_or_panic(
        embedded_skill_markdown(),
        "agenda-management skill markdown",
    );
    assert!(markdown.contains("name: agenda-management"));
    assert!(markdown.contains("Skill Manifest: Agenda Management"));
}

#[test]
fn embedded_forge_evolution_skill_markdown_loads_manifest() {
    let markdown = some_or_panic(
        embedded_resource_text("zhixing/skills/forge-evolution/SKILL.md"),
        "forge-evolution skill markdown",
    );
    assert!(markdown.contains("name: forge-evolution"));
    assert!(markdown.contains("Skill Manifest: Forge Evolution"));
}

#[test]
fn embedded_resource_text_normalizes_relative_paths() {
    let normalized = some_or_panic(
        embedded_resource_text("zhixing/templates/daily_agenda.md"),
        "daily agenda template",
    );
    let relative = some_or_panic(
        embedded_resource_text("./zhixing/templates/daily_agenda.md"),
        "relative daily agenda template",
    );

    assert_eq!(normalized, relative);
    assert!(normalized.contains("# Daily Agenda"));
}

#[test]
fn embedded_resource_text_from_wendao_uri_reads_skill_reference() {
    let uri = "wendao://skills/agenda-management/references/steward.md#persona";
    let content = some_or_panic(
        embedded_resource_text_from_wendao_uri(uri),
        "steward reference",
    );

    assert!(content.contains("Professional Identity: The Clockwork Guardian"));
}

#[test]
fn embedded_semantic_mounts_include_agenda_management_references() {
    let mounts = embedded_semantic_reference_mounts();
    let paths = some_or_panic(mounts.get("agenda-management"), "agenda-management mount");

    assert_eq!(
        paths,
        &vec![PathBuf::from("zhixing/skills/agenda-management/references")]
    );
}

#[test]
fn embedded_registry_and_links_keep_agenda_management_anchors() {
    let registry = ok_or_panic(build_embedded_wendao_registry(), "embedded registry");
    let skill_file = some_or_panic(
        registry.file(ZHIXING_SKILL_DOC_PATH),
        "agenda-management skill file",
    );

    let steward_uri = "wendao://skills/agenda-management/references/steward.md".to_string();
    let teacher_uri = "wendao://skills/agenda-management/references/teacher.md".to_string();

    let steward_links = some_or_panic(skill_file.links_for_id("steward"), "steward links");
    assert_eq!(steward_links, std::slice::from_ref(&steward_uri));

    let links_index = ok_or_panic(embedded_skill_links_index(), "links index");
    assert_eq!(
        links_index.get("steward").map(Vec::as_slice),
        Some(steward_links)
    );

    let persona_links = ok_or_panic(
        embedded_skill_links_for_reference_type("persona"),
        "persona links",
    );
    assert_eq!(
        persona_links,
        vec![steward_uri.clone(), teacher_uri.clone()]
    );

    let links_for_id = ok_or_panic(embedded_skill_links_for_id("steward"), "links for id");
    assert_eq!(links_for_id, vec![steward_uri]);
}

#[test]
fn embedded_discovery_finds_agenda_management_targets() {
    let steward_uri = "wendao://skills/agenda-management/references/steward.md".to_string();
    let teacher_uri = "wendao://skills/agenda-management/references/teacher.md".to_string();

    let discovered_by_type = ok_or_panic(
        embedded_discover_canonical_uris("reference_type:persona"),
        "persona discovery",
    );
    assert!(discovered_by_type.contains(&steward_uri));
    assert!(discovered_by_type.contains(&teacher_uri));

    let discovered_by_id = ok_or_panic(
        embedded_discover_canonical_uris("id:steward"),
        "steward discovery",
    );
    assert!(discovered_by_id.contains(&steward_uri));
}
