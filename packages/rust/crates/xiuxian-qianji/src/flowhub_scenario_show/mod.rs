//! Flowhub scenario show seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "render.rs"]
mod render;

pub use api::{
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview,
    render_flowhub_scenario_show, show_flowhub_scenario,
};

#[cfg(test)]
pub(crate) use render::{
    render_scenario_flowchart_section_lines, render_scenario_hidden_aliases_section_lines,
    render_scenario_links_section_lines, render_scenario_surface_section_lines,
};

#[cfg(test)]
#[path = "../../tests/unit/flowhub/scenario_show.rs"]
mod tests;
