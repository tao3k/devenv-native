//! Bounded-workdir ownership starts in `api`; runtime-state and validation stay private.

#[path = "../workdir_advance.rs"]
mod advance;
#[path = "../workdir_api.rs"]
mod api;
#[path = "../workdir_check/mod.rs"]
mod check;
#[path = "../workdir_detect.rs"]
mod detect;
#[path = "../workdir_load.rs"]
mod load;
#[path = "../workdir_parse.rs"]
mod parse;
#[path = "../workdir_query.rs"]
mod query;
#[path = "runtime_state.rs"]
mod runtime_state;
#[path = "semantic_scope.rs"]
mod semantic_scope;
#[path = "../workdir_show.rs"]
mod show;
#[path = "validate.rs"]
mod validate;

pub use api::{
    WorkdirAdvance, WorkdirCheckFollowUpQuery, WorkdirCheckReport, WorkdirDiagnostic,
    WorkdirMarkdownSurface, WorkdirSemanticProjectionPolicySummary,
    WorkdirSemanticScopeGuardStatus, WorkdirSemanticScopeGuardTrace,
    WorkdirSemanticScopeObjectSummary, WorkdirSemanticSqlGuardSummary, WorkdirShow,
    WorkdirVisibleSurface, WorkdirVisibleSurfaceKind, advance_workdir_step,
    build_workdir_check_follow_up_query, check_workdir, load_workdir_manifest,
    looks_like_workdir_dir, parse_workdir_manifest, query_workdir_check_follow_up_payload,
    query_workdir_markdown_payload, render_workdir_advance, render_workdir_check_markdown,
    render_workdir_semantic_scope_guard_trace, render_workdir_show, show_workdir,
    trace_workdir_semantic_scope_bundle, trace_workdir_semantic_scope_bundle_with_evidence,
    trace_workdir_semantic_scope_bundle_with_sql_guard_evidence, trace_workdir_semantic_scope_json,
};
pub(crate) use api::{
    WorkdirAllowedNextIssue, WorkdirCurrentNodeIssue, WorkdirRuntimeNode, WorkdirRuntimeState,
    expected_next_labels, load_workdir_runtime_state, resolve_runtime_node,
    validate_workdir_manifest,
};
