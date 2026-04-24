//! Embedded Forge skill resource registry tests.

use xiuxian_wendao::enhancer::WendaoResourceRegistry;
use xiuxian_wendao::skill_runtime::{
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
};
use xiuxian_zhixing::RESOURCES;

const FORGE_SKILL_DOC_PATH: &str = "zhixing/skills/forge-evolution/SKILL.md";
const FORGE_FLOW_URI: &str = "wendao://skills/forge-evolution/references/soul_forge_flow.toml";

#[test]
fn forge_skill_is_registered_in_embedded_resource_image()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let registry = WendaoResourceRegistry::build_from_embedded(&RESOURCES)?;
    let skill_file = registry
        .file(FORGE_SKILL_DOC_PATH)
        .ok_or_else(|| std::io::Error::other("expected forge-evolution SKILL.md registry entry"))?;

    assert_eq!(
        skill_file.links_for_id("soul_forge_flow"),
        Some(&[FORGE_FLOW_URI.to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("grand_auditor"),
        Some(&["wendao://skills/forge-evolution/references/grand_auditor.md".to_string()][..])
    );
    assert_eq!(
        skill_file.links_for_id("forge_guard"),
        Some(&["wendao://skills/forge-evolution/references/forge_guard.md".to_string()][..])
    );
    Ok(())
}

#[test]
fn forge_flow_uri_resolves_from_embedded_wendao_vfs() {
    let Some(content) = embedded_resource_text_from_wendao_uri(FORGE_FLOW_URI) else {
        panic!("expected semantic URI to resolve embedded forge flow");
    };
    assert!(content.contains("Evolution_Trinity_Soul_Forge_Flow"));
    assert!(content.contains("Grand_Auditor"));
    assert!(content.contains("Soul_Forger"));
}

#[test]
fn agenda_fallback_api_stays_stable_for_backward_compatibility()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    // Keep legacy helper behavior stable while adding new skills.
    let links = embedded_skill_links_for_id("agenda_flow")?;
    assert_eq!(
        links,
        vec!["wendao://skills/agenda-management/references/agenda_flow.toml".to_string()]
    );
    Ok(())
}
