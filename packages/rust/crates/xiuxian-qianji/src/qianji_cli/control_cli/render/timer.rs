use std::io;

use xiuxian_qianji_control::{TimerInventoryProjection, TimerView};

use super::{activity_scope_label, push_fmt, serde_status};

pub(crate) fn render_timer_view_text(timer: &TimerView) -> String {
    let scheduled_ms = timer
        .timer
        .as_ref()
        .map(|record| record.fire_at_ms.to_string());
    let completed_ms = timer.fired_at_ms.map(|fired_at| fired_at.to_string());

    format!(
        concat!(
            "# Qianji Control Timer\n\n",
            "- Timer: `{}`\n",
            "- Status: `{}`\n",
            "- Fire at ms: `{}`\n",
            "- Fired at ms: `{}`\n",
            "- Updated at ms: `{}`\n"
        ),
        timer.timer_id.as_str(),
        serde_status(&timer.status),
        scheduled_ms.as_deref().unwrap_or("<none>"),
        completed_ms.as_deref().unwrap_or("<none>"),
        timer.updated_at_ms
    )
}

pub(crate) fn render_timer_view_json(timer: &TimerView) -> io::Result<String> {
    serde_json::to_string_pretty(timer).map_err(io::Error::other)
}

pub(crate) fn render_timer_inventory_text(projection: &TimerInventoryProjection) -> String {
    let mut output = format!(
        concat!(
            "# Qianji Control Timers\n\n",
            "- Run: `{}`\n",
            "- Timers: total `{}`, pending `{}`, scheduled `{}`, fired `{}`\n"
        ),
        projection.run_id.as_str(),
        projection.summary.total,
        projection.summary.pending,
        projection.summary.scheduled,
        projection.summary.fired
    );

    if !projection.items.is_empty() {
        output.push_str("\n## Timers\n\n");
        for item in &projection.items {
            let scheduled_time = item.timer.timer.as_ref().map_or_else(
                || "<none>".to_string(),
                |timer| timer.fire_at_ms.to_string(),
            );
            let completion_time = item
                .timer
                .fired_at_ms
                .map_or_else(|| "<none>".to_string(), |fired_at| fired_at.to_string());
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` [{}] status `{}` fire_at_ms `{}` fired_at_ms `{}`\n",
                    item.timer.timer_id.as_str(),
                    activity_scope_label(&item.scope),
                    serde_status(&item.timer.status),
                    scheduled_time,
                    completion_time
                ),
            );
        }
    }

    output
}

pub(crate) fn render_timer_inventory_json(
    projection: &TimerInventoryProjection,
) -> io::Result<String> {
    serde_json::to_string_pretty(projection).map_err(io::Error::other)
}
