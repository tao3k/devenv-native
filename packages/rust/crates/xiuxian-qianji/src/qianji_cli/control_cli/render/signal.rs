use std::io;

use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord, SignalInventoryProjection};

use super::{activity_scope_label, push_fmt};

pub(crate) fn render_signal_append_text(record: &ControlEventRecord) -> String {
    let (signal_name, metadata) = match &record.event.kind {
        ControlEventKind::SignalReceived { signal } => (
            signal.signal_name.as_str(),
            serde_json::to_string(&signal.metadata).unwrap_or_else(|_| "null".to_string()),
        ),
        _ => ("<unknown>", "null".to_string()),
    };
    let scope = record
        .event
        .step_id
        .as_ref()
        .map_or("run", |step_id| step_id.as_str());

    format!(
        concat!(
            "# Qianji Control Signal\n\n",
            "- Event sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Scope: `{}`\n",
            "- Signal: `{}`\n",
            "- Received at ms: `{}`\n",
            "- Payload metadata: `{}`\n"
        ),
        record.sequence,
        record.event.run_id.as_str(),
        scope,
        signal_name,
        record.event.occurred_at_ms,
        metadata
    )
}

pub(crate) fn render_signal_append_json(record: &ControlEventRecord) -> io::Result<String> {
    serde_json::to_string_pretty(record).map_err(io::Error::other)
}

pub(crate) fn render_signal_inventory_text(projection: &SignalInventoryProjection) -> String {
    let mut output = format!(
        concat!(
            "# Qianji Control Signals\n\n",
            "- Run: `{}`\n",
            "- Signals: total `{}`, run `{}`, step `{}`\n"
        ),
        projection.run_id.as_str(),
        projection.summary.total,
        projection.summary.run_scoped,
        projection.summary.step_scoped
    );

    if !projection.items.is_empty() {
        output.push_str("\n## Signals\n\n");
        for item in &projection.items {
            push_fmt(
                &mut output,
                format_args!(
                    "- seq `{}` [{}] signal `{}` received_at_ms `{}`\n",
                    item.sequence,
                    activity_scope_label(&item.scope),
                    item.signal.signal_name.as_str(),
                    item.received_at_ms
                ),
            );
        }
    }

    output
}

pub(crate) fn render_signal_inventory_json(
    projection: &SignalInventoryProjection,
) -> io::Result<String> {
    serde_json::to_string_pretty(projection).map_err(io::Error::other)
}
