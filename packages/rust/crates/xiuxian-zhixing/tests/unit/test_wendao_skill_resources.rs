//! Embedded skill/resource registry tests for Zhixing-Wendao integration.

use xiuxian_wendao::enhancer::WendaoResourceRegistry;
use xiuxian_wendao::skill_runtime::{
    ZHIXING_SKILL_DOC_PATH, build_embedded_wendao_registry, embedded_resource_text,
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
use xiuxian_zhixing::RESOURCES;

#[test]
fn embedded_skill_markdown_is_available() {
    let Some(markdown) = embedded_skill_markdown() else {
        panic!("expected embedded zhixing skill markdown at {ZHIXING_SKILL_DOC_PATH}");
    };
    assert!(markdown.contains("name: agenda-management"));
    assert!(markdown.contains("Skill Manifest: Agenda Management"));
}

#[test]
fn wendao_registry_extracts_skill_md_from_zhixing_resource_image()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = WendaoResourceRegistry::build_from_embedded(&RESOURCES)?;
    let Some(skill_file) = registry.file(ZHIXING_SKILL_DOC_PATH) else {
        panic!("expected SKILL.md entry in zhixing embedded resource image");
    };
    assert_eq!(skill_file.path(), ZHIXING_SKILL_DOC_PATH);
    Ok(())
}

#[test]
fn embedded_registry_builds_and_skill_file_links_match_index_api()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = build_embedded_wendao_registry()?;
    let Some(skill_file) = registry.file(ZHIXING_SKILL_DOC_PATH) else {
        panic!("expected registry file entry for {ZHIXING_SKILL_DOC_PATH}");
    };
    assert_eq!(
        skill_file.links_by_id(),
        &embedded_skill_links_index()?,
        "file-level links_by_id should match convenience API"
    );
    Ok(())
}

#[test]
fn embedded_skill_links_for_unknown_id_returns_empty()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let unknown_links = embedded_skill_links_for_id("does_not_exist")?;
    assert!(unknown_links.is_empty());
    Ok(())
}

#[test]
fn embedded_skill_links_for_qianji_flow_reference_type_returns_semantic_uri()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let links = embedded_skill_links_for_reference_type("qianji-flow")?;
    assert_eq!(
        links,
        vec!["wendao://skills/agenda-management/references/agenda_flow.toml".to_string()]
    );
    Ok(())
}

#[test]
fn embedded_resource_text_resolves_linked_templates() {
    let Some(content) =
        embedded_resource_text("./zhixing/skills/agenda-management/references/draft_agenda.j2")
    else {
        panic!("expected embedded template content for linked draft agenda");
    };
    assert!(content.contains("agenda_draft"));
}

#[test]
fn embedded_registry_tracks_wendao_uri_links_for_skill_bus()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = build_embedded_wendao_registry()?;
    let Some(skill_file) = registry.file("zhixing/skills/agenda-management/SKILL.md") else {
        panic!("expected semantic skill descriptor in embedded registry");
    };
    assert_eq!(
        skill_file.links_for_id("draft_agenda.j2"),
        Some(&["wendao://skills/agenda-management/references/draft_agenda.j2".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("critique_agenda.j2"),
        Some(&["wendao://skills/agenda-management/references/critique_agenda.j2".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("final_agenda.j2"),
        Some(&["wendao://skills/agenda-management/references/final_agenda.j2".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("steward"),
        Some(&["wendao://skills/agenda-management/references/steward.md".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("teacher"),
        Some(&["wendao://skills/agenda-management/references/teacher.md".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("rules"),
        Some(&["wendao://skills/agenda-management/references/rules.md".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("agenda_classifier"),
        Some(
            &["wendao://skills/agenda-management/references/prompts/classifier.md".to_string()][..]
        )
    );
    assert_eq!(
        skill_file.links_for_id("agenda_validation_genesis_rules"),
        Some(
            &["wendao://skills/agenda-management/references/prompts/agenda_validation_genesis_rules.md".to_string()][..]
        )
    );
    assert_eq!(
        skill_file.links_for_id("agenda_flow"),
        Some(&["wendao://skills/agenda-management/references/agenda_flow.toml".to_string()][..])
    );
    Ok(())
}

#[test]
fn embedded_resource_text_from_wendao_uri_resolves_template_payload() {
    let Some(content) = embedded_resource_text_from_wendao_uri(
        "wendao://skills/agenda-management/references/draft_agenda.j2",
    ) else {
        panic!("expected semantic URI to resolve embedded draft agenda template");
    };
    assert!(content.contains("<agenda_draft>"));
}

#[test]
fn embedded_resource_text_from_wendao_uri_resolves_persona_payload() {
    let Some(content) = embedded_resource_text_from_wendao_uri(
        "wendao://skills/agenda-management/references/steward.md",
    ) else {
        panic!("expected semantic URI to resolve embedded steward persona");
    };
    assert!(content.contains("Professional Identity: The Clockwork Guardian"));
}

#[test]
fn embedded_resource_text_from_wendao_uri_resolves_qianji_flow_payload() {
    let Some(content) = embedded_resource_text_from_wendao_uri(
        "wendao://skills/agenda-management/references/agenda_flow.toml",
    ) else {
        panic!("expected semantic URI to resolve embedded agenda flow");
    };
    assert!(content.contains("Self_Healing_Agenda_Governance_Flow"));
    assert!(content.contains("Professor_Audit"));
}

#[test]
fn embedded_resource_text_from_wendao_uri_resolves_teacher_output_protocol() {
    let Some(content) = embedded_resource_text_from_wendao_uri(
        "wendao://skills/agenda-management/references/teacher.md",
    ) else {
        panic!("expected semantic URI to resolve embedded teacher persona");
    };
    assert!(
        content.contains("<professor_audit>") && content.contains("score (0.0-1.0)"),
        "teacher persona should require XML critique output with score and critique tags"
    );
}
