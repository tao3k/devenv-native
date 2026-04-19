mod anchor;
mod anchored_show;
mod check;
mod discover;
mod flowchart;
mod graph_show;
mod load;
mod materialize;
pub(crate) mod mermaid;
mod parse;
mod resolve;
mod scenario;
pub(crate) mod scenario_ir;
mod show;
mod validate;

pub use anchored_show::show_flowhub_anchored_scenario;
pub use check::{
    FlowhubCheckReport, FlowhubDiagnostic, check_flowhub, render_flowhub_check_markdown,
};
pub use discover::{FlowhubDirKind, classify_flowhub_dir};
pub(crate) use flowchart::{derive_flowchart_aliases, render_flowchart};
pub use graph_show::{
    FlowhubGraphEdgeSummary, FlowhubGraphNodeSummary, FlowhubGraphShow, render_flowhub_graph_show,
    show_flowhub_graph,
};
pub use load::{load_flowhub_module_manifest, load_flowhub_scenario_manifest};
pub use materialize::{
    AnchoredMaterializedWorkdir, materialize_flowhub_anchored_scenario,
    materialize_flowhub_anchored_scenario_at_node, render_anchored_materialized_workdir,
};
pub use materialize::{MaterializedWorkdir, materialize_flowhub_scenario_workdir};
pub use parse::{parse_flowhub_module_manifest, parse_flowhub_scenario_manifest};
pub use resolve::{
    ResolvedFlowhubModule, resolve_flowhub_module_children, resolve_flowhub_scenario_modules,
};
pub use scenario::{
    FlowhubScenarioCheckReport, FlowhubScenarioDiagnostic, FlowhubScenarioHiddenAlias,
    FlowhubScenarioShow, FlowhubScenarioSurfacePreview, check_flowhub_scenario,
    looks_like_flowhub_scenario_dir, render_flowhub_scenario_check_markdown,
    render_flowhub_scenario_show, show_flowhub_scenario,
};
pub use show::{
    FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow,
    FlowhubScenarioCaseSummary, FlowhubShow, render_flowhub_show, show_flowhub,
};
