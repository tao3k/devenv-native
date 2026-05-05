//! Flowhub show seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "discover.rs"]
mod discover;
#[path = "render/mod.rs"]
mod render;

pub use api::{
    FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow,
    FlowhubScenarioCaseSummary, FlowhubShow, render_flowhub_show, show_flowhub,
};

#[cfg(test)]
pub(crate) use render::{
    render_flowhub_module_contract_section_lines, render_flowhub_root_module_section_lines,
    render_scenario_case_section_lines, render_scenario_case_summary_block,
};

#[cfg(test)]
#[path = "../../tests/unit/flowhub/show.rs"]
mod tests;
