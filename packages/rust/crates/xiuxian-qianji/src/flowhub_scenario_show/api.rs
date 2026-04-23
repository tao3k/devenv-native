use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::markdown::{MarkdownShowSection, render_show_surface};

use crate::flowhub::{
    derive_flowchart_aliases, load_flowhub_scenario_manifest, render_flowchart,
    resolve_flowhub_scenario_modules,
};

use super::render::{
    render_scenario_flowchart_section_lines, render_scenario_hidden_aliases_section_lines,
    render_scenario_links_section_lines, render_scenario_surface_section_lines,
};

/// One visible surface preview derived from a scenario alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioSurfacePreview {
    /// Alias that will become a top-level bounded work-surface directory.
    pub alias: String,
    /// Resolved Flowhub module reference for this alias.
    pub module_ref: String,
    /// Conceptual target path inside the future work surface.
    pub target_path: PathBuf,
    /// Source node manifest inside Flowhub.
    pub source_manifest_path: PathBuf,
}

/// One hidden composite alias that participates in the scenario graph but does
/// not materialize into a top-level bounded surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioHiddenAlias {
    /// Alias declared by the scenario manifest.
    pub alias: String,
    /// Resolved Flowhub module reference.
    pub module_ref: String,
}

/// First-order preview of the bounded work surface implied by a scenario root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioShow {
    /// Stable scenario/plan name.
    pub plan_name: String,
    /// Scenario root directory.
    pub scenario_dir: PathBuf,
    /// Resolved Flowhub root used for module lookups.
    pub flowhub_root: PathBuf,
    /// Derived preview of the materialized root flowchart.
    pub flowchart_preview: String,
    /// Ordered visible leaf surfaces that will materialize.
    pub surfaces: Vec<FlowhubScenarioSurfacePreview>,
    /// Ordered composite aliases hidden behind the top-level bounded surface.
    pub hidden_aliases: Vec<FlowhubScenarioHiddenAlias>,
    /// Declared scenario links rendered as stable references.
    pub links: Vec<String>,
}

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

    let mut surfaces = Vec::new();
    let mut hidden_aliases = Vec::new();
    let mut visible_aliases = Vec::new();
    for module in &resolved_modules {
        if module.manifest.template.is_some() {
            hidden_aliases.push(FlowhubScenarioHiddenAlias {
                alias: module.alias.clone(),
                module_ref: module.module_ref.clone(),
            });
            continue;
        }

        visible_aliases.push(module.alias.clone());
        surfaces.push(FlowhubScenarioSurfacePreview {
            alias: module.alias.clone(),
            module_ref: module.module_ref.clone(),
            target_path: scenario_dir.join(&module.alias),
            source_manifest_path: module.manifest_path.clone(),
        });
    }

    if surfaces.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub scenario `{}` does not expose any leaf nodes that can anchor a bounded work surface",
            manifest.planning.name
        )));
    }

    let flowchart_aliases = derive_flowchart_aliases(&manifest, &visible_aliases);
    let flowchart_preview = render_flowchart(&manifest, &visible_aliases, &flowchart_aliases);
    let links = manifest
        .template
        .link
        .iter()
        .map(|link| {
            format!(
                "{} -> {}",
                display_link_ref(&link.from),
                display_link_ref(&link.to)
            )
        })
        .collect::<Vec<_>>();

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
