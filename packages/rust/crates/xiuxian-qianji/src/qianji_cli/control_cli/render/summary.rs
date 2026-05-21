use std::io;

use xiuxian_qianji_control::RunOperatorSummary;

use super::serde_status;

pub(crate) fn render_operator_summary_text(summary: &RunOperatorSummary) -> String {
    format!(
        concat!(
            "# Qianji Control Summary\n\n",
            "- Run: `{}`\n",
            "- Status: `{}`\n",
            "- Observed at ms: `{}`\n",
            "- Updated at ms: `{}`\n",
            "- Events: `{}`\n",
            "- Steps: `{}`\n",
            "- Active leases: `{}`\n",
            "- Activities: total `{}`, scheduled `{}`, in-flight `{}`, completed `{}`, failed `{}`\n",
            "- Timers: total `{}`, pending `{}`, scheduled `{}`, fired `{}`\n",
            "- Signals: total `{}`, run `{}`, step `{}`\n",
            "- Cost usd micros: `{}`\n",
            "- Tokens: `{}`\n",
            "- Recovery actions: `{}`\n",
            "- Reclaim expired leases: `{}`\n",
            "- Fireable timers: `{}`\n",
            "- Human approvals: `{}`\n"
        ),
        summary.run_id.as_str(),
        serde_status(&summary.status),
        summary.observed_at_ms,
        summary.updated_at_ms,
        summary.event_count,
        summary.steps,
        summary.active_leases,
        summary.activities.total,
        summary.activities.scheduled,
        summary.activities.in_flight,
        summary.activities.completed,
        summary.activities.failed,
        summary.timers.total,
        summary.timers.pending,
        summary.timers.scheduled,
        summary.timers.fired,
        summary.signals.total,
        summary.signals.run_scoped,
        summary.signals.step_scoped,
        summary.costs.cost_usd_micros,
        summary.costs.total_tokens,
        summary.recovery.total_actions,
        summary.recovery.reclaim_expired_leases,
        summary.recovery.fireable_timers,
        summary.recovery.await_human_approvals
    )
}

pub(crate) fn render_operator_summary_json(summary: &RunOperatorSummary) -> io::Result<String> {
    serde_json::to_string_pretty(summary).map_err(io::Error::other)
}
