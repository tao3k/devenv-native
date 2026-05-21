//! Flowhub scenario preview API.
//!
//! This module loads a scenario manifest, resolves Flowhub modules, and
//! renders the bounded work-surface preview exposed by `qianji show`.

use std::path::Path;

use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};

use crate::flowhub::{
    ResolvedFlowhubModule, derive_flowchart_aliases, load_flowhub_scenario_manifest,
    render_flowchart, resolve_flowhub_scenario_modules,
};

use super::model::{
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview,
};
use super::render::{
    render_scenario_flowchart_section_lines, render_scenario_hidden_aliases_section_lines,
    render_scenario_links_section_lines, render_scenario_surface_section_lines,
};

/// Build a first-order work-surface preview from a Flowhub scenario directory.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the scenario manifest cannot be
/// loaded or Flowhub modules cannot be resolved.
pub fn show_flowhub_scenario(
    flowhub_root: impl AsRef<Path>,
    scenario_dir: impl AsRef<Path>,
) -> Result<FlowhubScenarioShow, QianjiError> {
    let flowhub_root = flowhub_root.as_ref();
    let scenario_dir = scenario_dir.as_ref();
    let manifest_path = scenario_dir.join("qianji.toml");
    let manifest = load_flowhub_scenario_manifest(&manifest_path)?;
    let resolved_modules = resolve_flowhub_scenario_modules(flowhub_root, &manifest)?;

    let ScenarioModulePartition {
        surfaces,
        hidden_aliases,
        visible_aliases,
    } = partition_scenario_modules(scenario_dir, &resolved_modules);

    if surfaces.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub scenario `{}` does not expose any leaf nodes that can anchor a bounded work surface",
            manifest.planning.name
        )));
    }

    let flowchart_aliases = derive_flowchart_aliases(&manifest, &visible_aliases);
    let flowchart_preview = render_flowchart(&manifest, &visible_aliases, &flowchart_aliases);
    let links = build_scenario_links(&manifest.template.link);

    Ok(FlowhubScenarioShow {
        plan_name: manifest.planning.name,
        scenario_dir: scenario_dir.to_path_buf(),
        flowhub_root: flowhub_root.to_path_buf(),
        flowchart_preview,
        surfaces,
        hidden_aliases,
        links,
    })
}

/// Render a scenario-derived work-surface preview into markdown.
#[must_use]
pub fn render_flowhub_scenario_show(show: &FlowhubScenarioShow) -> String {
    let mut sections = vec![MarkdownShowSection {
        title: "flowchart.mmd".into(),
        lines: render_scenario_flowchart_section_lines(show),
    }];

    for surface in &show.surfaces {
        sections.push(MarkdownShowSection {
            title: surface.alias.as_str().into(),
            lines: render_scenario_surface_section_lines(surface),
        });
    }

    if !show.hidden_aliases.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Hidden Composite Aliases".into(),
            lines: render_scenario_hidden_aliases_section_lines(&show.hidden_aliases),
        });
    }

    if !show.links.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Links".into(),
            lines: render_scenario_links_section_lines(&show.links),
        });
    }

    render_show_surface(
        "Scenario Work Surface Preview",
        &[
            format!("Scenario: {}", show.plan_name),
            format!("Location: {}", show.scenario_dir.display()),
            format!("Flowhub: {}", show.flowhub_root.display()),
        ],
        &sections,
    )
}

fn display_link_ref(reference: &crate::contracts::TemplateLinkRef) -> String {
    match (&reference.alias, &reference.symbol) {
        (Some(alias), symbol) => format!("{alias}::{symbol}"),
        (None, symbol) => symbol.clone(),
    }
}

#[derive(Default)]
struct ScenarioModulePartition {
    surfaces: Vec<FlowhubScenarioSurfacePreview>,
    hidden_aliases: Vec<FlowhubScenarioHiddenAlias>,
    visible_aliases: Vec<String>,
}

fn partition_scenario_modules(
    scenario_dir: &Path,
    resolved_modules: &[ResolvedFlowhubModule],
) -> ScenarioModulePartition {
    resolved_modules.iter().fold(
        ScenarioModulePartition::default(),
        |mut partition, module| {
            if module.manifest.template.is_some() {
                partition.hidden_aliases.push(hidden_scenario_alias(module));
            } else {
                partition.visible_aliases.push(module.alias.clone());
                partition
                    .surfaces
                    .push(surface_preview_for_module(scenario_dir, module));
            }
            partition
        },
    )
}

fn hidden_scenario_alias(module: &ResolvedFlowhubModule) -> FlowhubScenarioHiddenAlias {
    FlowhubScenarioHiddenAlias {
        alias: module.alias.clone(),
        module_ref: module.module_ref.clone(),
    }
}

fn surface_preview_for_module(
    scenario_dir: &Path,
    module: &ResolvedFlowhubModule,
) -> FlowhubScenarioSurfacePreview {
    FlowhubScenarioSurfacePreview {
        alias: module.alias.clone(),
        module_ref: module.module_ref.clone(),
        target_path: scenario_dir.join(&module.alias),
        source_manifest_path: module.manifest_path.clone(),
    }
}

fn build_scenario_links(links: &[crate::contracts::TemplateLinkSpec]) -> Vec<String> {
    links
        .iter()
        .map(|link| {
            format!(
                "{} -> {}",
                display_link_ref(&link.from),
                display_link_ref(&link.to)
            )
        })
        .collect()
}
