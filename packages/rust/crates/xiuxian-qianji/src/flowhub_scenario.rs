//! Scenario ownership starts in `show`; check and detect stay secondary.

#[path = "flowhub_scenario_check.rs"]
mod check;
#[path = "flowhub_scenario_detect.rs"]
mod detect;
#[path = "flowhub_scenario_show/mod.rs"]
mod show;

pub use check::{
    FlowhubScenarioCheckReport, FlowhubScenarioDiagnostic, check_flowhub_scenario,
    render_flowhub_scenario_check_markdown,
};
pub use detect::looks_like_flowhub_scenario_dir;
pub use show::{
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview,
    render_flowhub_scenario_show, show_flowhub_scenario,
};
