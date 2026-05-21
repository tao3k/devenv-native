use std::io;

use xiuxian_qianji_control::{RunRecoverySnapshot, RunView};

use super::{ControlStateQueryView, serde_status};

#[derive(serde::Serialize)]
struct ControlStateQueryJson<'a> {
    event_count: usize,
    run_view: &'a RunView,
    recovery_snapshot: &'a RunRecoverySnapshot,
}

pub(crate) fn render_control_state_query_text(state: &ControlStateQueryView<'_>) -> String {
    let step_activity_count = state
        .run_view
        .steps
        .values()
        .map(|step| step.activities.len())
        .sum::<usize>();
    let step_timer_count = state
        .run_view
        .steps
        .values()
        .map(|step| step.timers.len())
        .sum::<usize>();
    let step_signal_count = state
        .run_view
        .steps
        .values()
        .map(|step| step.signals.len())
        .sum::<usize>();
    let summary = &state.recovery_snapshot.summary;

    format!(
        concat!(
            "# Qianji Control State\n\n",
            "- Run: `{}`\n",
            "- Status: `{}`\n",
            "- Observed at ms: `{}`\n",
            "- Events: `{}`\n",
            "- Steps: `{}`\n",
            "- Run activities: `{}`\n",
            "- Step activities: `{}`\n",
            "- Run timers: `{}`\n",
            "- Step timers: `{}`\n",
            "- Run signals: `{}`\n",
            "- Step signals: `{}`\n",
            "- Recovery actions: `{}`\n",
            "- Fireable timers: `{}`\n",
            "- Human approvals: `{}`\n",
            "- Total cost usd micros: `{}`\n"
        ),
        state.run_view.run_id.as_str(),
        serde_status(&state.run_view.status),
        state.recovery_snapshot.observed_at_ms,
        state.event_count,
        state.run_view.steps.len(),
        state.run_view.activities.len(),
        step_activity_count,
        state.run_view.timers.len(),
        step_timer_count,
        state.run_view.signals.len(),
        step_signal_count,
        summary.total_actions,
        summary.fireable_timers,
        summary.await_human_approvals,
        state.run_view.total_cost_usd_micros()
    )
}

pub(crate) fn render_control_state_query_json(
    state: &ControlStateQueryView<'_>,
) -> io::Result<String> {
    let json = ControlStateQueryJson {
        event_count: state.event_count,
        run_view: state.run_view,
        recovery_snapshot: state.recovery_snapshot,
    };
    serde_json::to_string_pretty(&json).map_err(io::Error::other)
}
