use std::io;

use xiuxian_qianji_control::HotStateSnapshot;

use super::push_fmt;

#[cfg(any(feature = "valkey", test))]
pub(crate) fn render_hot_state_snapshot_text(snapshot: &HotStateSnapshot) -> String {
    let mut output = format!(
        concat!(
            "# Qianji Control Hot State\n\n",
            "- Observed at ms: `{}`\n",
            "- Pending steps: `{}`\n",
            "- Leased steps: `{}`\n",
            "- Active leases: `{}`\n",
            "- Expired leases: `{}`\n",
            "- Worker heartbeats: `{}`\n",
            "- Live worker heartbeats: `{}`\n"
        ),
        snapshot.observed_at_ms,
        snapshot.pending_steps.len(),
        snapshot.leased_steps.len(),
        snapshot.active_lease_count(),
        snapshot.expired_lease_count(),
        snapshot.worker_heartbeats.len(),
        snapshot.live_heartbeat_count()
    );

    if !snapshot.pending_steps.is_empty() {
        output.push_str("\n## Pending Steps\n\n");
        for step in &snapshot.pending_steps {
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` step `{}` priority `{}` not-before `{}`\n",
                    step.run_id.as_str(),
                    step.step_id.as_str(),
                    step.priority,
                    step.not_before_ms
                ),
            );
        }
    }

    if !snapshot.leased_steps.is_empty() {
        output.push_str("\n## Leased Steps\n\n");
        for leased in &snapshot.leased_steps {
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` step `{}` lease `{}` worker `{}` expires `{}`\n",
                    leased.lease.run_id.as_str(),
                    leased.lease.step_id.as_str(),
                    leased.lease.lease_id.as_str(),
                    leased.lease.worker_id.as_str(),
                    leased.lease.expires_at_ms
                ),
            );
        }
    }

    if !snapshot.worker_heartbeats.is_empty() {
        output.push_str("\n## Worker Heartbeats\n\n");
        for heartbeat in &snapshot.worker_heartbeats {
            push_fmt(
                &mut output,
                format_args!(
                    "- `{}` observed `{}` expires `{}`\n",
                    heartbeat.worker_id.as_str(),
                    heartbeat.observed_at_ms,
                    heartbeat.expires_at_ms
                ),
            );
        }
    }

    output
}

#[cfg(any(feature = "valkey", test))]
pub(crate) fn render_hot_state_snapshot_json(snapshot: &HotStateSnapshot) -> io::Result<String> {
    serde_json::to_string_pretty(snapshot).map_err(io::Error::other)
}
