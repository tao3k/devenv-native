use serde_json::json;
use xiuxian_qianhuan::EmbeddedManifestationTemplateCatalog;

use super::model::{
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview,
};

const SCENARIO_FLOWCHART_SECTION_TEMPLATE_NAME: &str = "flowhub_scenario_flowchart.md.j2";
const SCENARIO_FLOWCHART_SECTION_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_scenario_flowchart.md.j2");
const SCENARIO_SURFACE_SECTION_TEMPLATE_NAME: &str = "flowhub_scenario_surface.md.j2";
const SCENARIO_SURFACE_SECTION_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_scenario_surface.md.j2");
const SCENARIO_HIDDEN_ALIASES_TEMPLATE_NAME: &str = "flowhub_scenario_hidden_aliases.md.j2";
const SCENARIO_HIDDEN_ALIASES_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_scenario_hidden_aliases.md.j2");
const SCENARIO_LINKS_TEMPLATE_NAME: &str = "flowhub_scenario_links.md.j2";
const SCENARIO_LINKS_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/flowhub_scenario_links.md.j2");

static SCENARIO_TEMPLATE_CATALOG: EmbeddedManifestationTemplateCatalog =
    EmbeddedManifestationTemplateCatalog::new(
        "Flowhub scenario show template renderer",
        &[
            (
                SCENARIO_FLOWCHART_SECTION_TEMPLATE_NAME,
                SCENARIO_FLOWCHART_SECTION_TEMPLATE_SOURCE,
            ),
            (
                SCENARIO_SURFACE_SECTION_TEMPLATE_NAME,
                SCENARIO_SURFACE_SECTION_TEMPLATE_SOURCE,
            ),
            (
                SCENARIO_HIDDEN_ALIASES_TEMPLATE_NAME,
                SCENARIO_HIDDEN_ALIASES_TEMPLATE_SOURCE,
            ),
            (SCENARIO_LINKS_TEMPLATE_NAME, SCENARIO_LINKS_TEMPLATE_SOURCE),
        ],
    );

pub(crate) fn render_scenario_flowchart_section_lines(show: &FlowhubScenarioShow) -> Vec<String> {
    render_embedded_scenario_block(
        SCENARIO_FLOWCHART_SECTION_TEMPLATE_NAME,
        json!({
            "flowchart_preview": show.flowchart_preview.trim_end(),
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub scenario flowchart preview through qianhuan; falling back to inline format: {error}"
        );
        vec![
            "Status: preview".to_string(),
            "Preview:".to_string(),
            "```mermaid".to_string(),
            show.flowchart_preview.trim_end().to_string(),
            "```".to_string(),
        ]
    })
}

pub(crate) fn render_scenario_surface_section_lines(
    surface: &FlowhubScenarioSurfacePreview,
) -> Vec<String> {
    render_embedded_scenario_block(
        SCENARIO_SURFACE_SECTION_TEMPLATE_NAME,
        json!({
            "module_ref": surface.module_ref,
            "target_path": surface.target_path.display().to_string(),
            "source_manifest_path": surface.source_manifest_path.display().to_string(),
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub scenario surface preview through qianhuan; falling back to inline format: {error}"
        );
        vec![
            format!("Module: {}", surface.module_ref),
            format!("Target Path: {}", surface.target_path.display()),
            format!("Source Manifest: {}", surface.source_manifest_path.display()),
        ]
    })
}

pub(crate) fn render_scenario_hidden_aliases_section_lines(
    hidden_aliases: &[FlowhubScenarioHiddenAlias],
) -> Vec<String> {
    render_embedded_scenario_block(
        SCENARIO_HIDDEN_ALIASES_TEMPLATE_NAME,
        json!({
            "aliases_block": hidden_aliases
                .iter()
                .map(|hidden| format!("- {} -> {}", hidden.alias, hidden.module_ref))
                .collect::<Vec<_>>()
                .join("\n"),
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub scenario hidden aliases through qianhuan; falling back to inline format: {error}"
        );
        hidden_aliases
            .iter()
            .map(|hidden| format!("- {} -> {}", hidden.alias, hidden.module_ref))
            .collect()
    })
}

pub(crate) fn render_scenario_links_section_lines(links: &[String]) -> Vec<String> {
    render_embedded_scenario_block(
        SCENARIO_LINKS_TEMPLATE_NAME,
        json!({
            "links_block": links
                .iter()
                .map(|link| format!("- {link}"))
                .collect::<Vec<_>>()
                .join("\n"),
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub scenario links through qianhuan; falling back to inline format: {error}"
        );
        links.iter().map(|link| format!("- {link}")).collect()
    })
}

fn render_embedded_scenario_block(
    template_name: &str,
    payload: serde_json::Value,
) -> Result<Vec<String>, String> {
    SCENARIO_TEMPLATE_CATALOG.render_lines(template_name, payload)
}
