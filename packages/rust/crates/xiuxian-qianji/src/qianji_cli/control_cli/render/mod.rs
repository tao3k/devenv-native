//! Renders `qianji control` views into operator text or JSON.

mod activity;
mod decision;
mod history;
#[cfg(any(feature = "valkey", test))]
mod hot_state;
mod lease;
mod recovery;
mod signal;
mod state;
mod state_context;
mod summary;
mod text_support;
mod timer;
mod view;

pub(super) use activity::{
    render_activity_queue_projection_json, render_activity_queue_projection_text,
    render_activity_view_json, render_activity_view_text, render_cost_inventory_json,
    render_cost_inventory_text,
};
pub(super) use decision::{render_agent_decision_json, render_agent_decision_text};
pub(super) use history::{render_control_history_json, render_control_history_text};
#[cfg(any(feature = "valkey", test))]
pub(super) use hot_state::{render_hot_state_snapshot_json, render_hot_state_snapshot_text};
pub(super) use lease::{
    render_step_lease_json, render_step_lease_text, render_step_leases_json,
    render_step_leases_text,
};
#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(super) use recovery::{render_recovery_loop_json, render_recovery_loop_text};
pub(super) use recovery::{render_recovery_snapshot_json, render_recovery_snapshot_text};
pub(super) use signal::{
    render_signal_append_json, render_signal_append_text, render_signal_inventory_json,
    render_signal_inventory_text,
};
pub(super) use state::{render_control_state_query_json, render_control_state_query_text};
pub(super) use state_context::ControlStateQueryView;
pub(super) use summary::{render_operator_summary_json, render_operator_summary_text};
pub(super) use text_support::{activity_scope_label, push_fmt, serde_status};
pub(super) use timer::{
    render_timer_inventory_json, render_timer_inventory_text, render_timer_view_json,
    render_timer_view_text,
};
pub(super) use view::{
    render_run_view_json, render_run_view_text, render_step_view_json, render_step_view_text,
};

#[cfg(test)]
#[path = "../../../../tests/unit/bin/qianji/control_cli/render.rs"]
mod tests;
