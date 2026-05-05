//! Flowhub module, scenario, Mermaid, and materialize helpers.
//!
//! Start in `api`; the other modules are private owners and helpers.

mod anchor;
#[path = "../flowhub_anchored_show.rs"]
mod anchored_show;
#[path = "../flowhub_api.rs"]
mod api;
#[path = "../flowhub_check/mod.rs"]
mod check;
#[path = "../flowhub_discover.rs"]
mod discover;
mod flowchart;
#[path = "../flowhub_graph_show/mod.rs"]
mod graph_show;
#[path = "../flowhub_load.rs"]
mod load;
#[path = "../flowhub_materialize_anchored.rs"]
mod materialize_anchored;
#[path = "../flowhub_materialize_api.rs"]
mod materialize_api;
#[path = "materialize/copy.rs"]
mod materialize_copy;
#[path = "materialize/root.rs"]
mod materialize_root;
#[path = "materialize/safety.rs"]
mod materialize_safety;
#[path = "../flowhub_materialize_scenario.rs"]
mod materialize_scenario;
#[path = "../flowhub_mermaid_api.rs"]
mod mermaid_api;
#[path = "mermaid/model.rs"]
mod mermaid_model;
#[path = "mermaid/parse.rs"]
mod mermaid_parse;
#[path = "mermaid/topology.rs"]
mod mermaid_topology;
#[path = "mermaid/validate.rs"]
mod mermaid_validate;
#[path = "../flowhub_parse.rs"]
mod parse;
#[path = "../flowhub_resolve.rs"]
mod resolve;
#[path = "../flowhub_scenario.rs"]
mod scenario;
#[path = "scenario_ir/annotation_model.rs"]
mod scenario_ir_annotation_model;
#[path = "scenario_ir/annotation_node.rs"]
mod scenario_ir_annotation_node;
#[path = "scenario_ir/annotation_support.rs"]
mod scenario_ir_annotation_support;
#[path = "scenario_ir/annotations.rs"]
mod scenario_ir_annotations;
#[path = "../flowhub_scenario_ir_api.rs"]
mod scenario_ir_api;
#[path = "scenario_ir/compile.rs"]
mod scenario_ir_compile;
#[path = "scenario_ir/compile_legacy.rs"]
mod scenario_ir_compile_legacy;
#[path = "scenario_ir/compile_nodes.rs"]
mod scenario_ir_compile_nodes;
#[path = "scenario_ir/compile_workdir.rs"]
mod scenario_ir_compile_workdir;
#[path = "scenario_ir/model.rs"]
mod scenario_ir_model;
#[path = "../flowhub_show/mod.rs"]
mod show;
mod validate;

pub use self::api::{
    AnchoredMaterializedWorkdir, FlowhubCheckReport, FlowhubDiagnostic, FlowhubDirKind,
    FlowhubGraphShow, FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow,
    FlowhubScenarioCaseSummary, FlowhubScenarioCheckReport, FlowhubScenarioDiagnostic,
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview, FlowhubShow,
    MaterializedWorkdir, ResolvedFlowhubModule, check_flowhub, check_flowhub_scenario,
    classify_flowhub_dir, load_flowhub_module_manifest, load_flowhub_scenario_manifest,
    looks_like_flowhub_scenario_dir, materialize_flowhub_anchored_scenario,
    materialize_flowhub_anchored_scenario_at_node, materialize_flowhub_scenario_workdir,
    parse_flowhub_module_manifest, parse_flowhub_scenario_manifest,
    render_anchored_materialized_workdir, render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_scenario_check_markdown, render_flowhub_scenario_show, render_flowhub_show,
    resolve_flowhub_module_children, resolve_flowhub_scenario_modules, show_flowhub,
    show_flowhub_anchored_scenario, show_flowhub_graph, show_flowhub_scenario,
};
pub(crate) use self::api::{
    FlowhubGraphAnnotations, FlowhubScenarioIr, FlowhubScenarioNodeIr, MermaidFlowchart,
    MermaidNodeKind, analyze_mermaid_flowchart_topology, compile_flowhub_scenario_ir,
    derive_flowchart_aliases, normalize_graph_node_label, parse_flowhub_graph_annotations,
    parse_mermaid_flowchart, render_flowchart, resolve_flowhub_graph_name,
    scenario_graph_label_is_allowed, validate_mermaid_flowchart,
};
