use std::{fmt::Write as _, io};

pub(crate) fn render_step_lease_text(lease: &xiuxian_qianji_control::StepLease) -> String {
    format!(
        concat!(
            "# Qianji Control Lease\n\n",
            "- Run: `{}`\n",
            "- Step: `{}`\n",
            "- Lease: `{}`\n",
            "- Worker: `{}`\n",
            "- Acquired at ms: `{}`\n",
            "- Expires at ms: `{}`\n"
        ),
        lease.run_id.as_str(),
        lease.step_id.as_str(),
        lease.lease_id.as_str(),
        lease.worker_id.as_str(),
        lease.acquired_at_ms,
        lease.expires_at_ms
    )
}

pub(crate) fn render_step_lease_json(
    lease: &xiuxian_qianji_control::StepLease,
) -> io::Result<String> {
    serde_json::to_string_pretty(lease).map_err(io::Error::other)
}

pub(crate) fn render_step_leases_text(leases: &[xiuxian_qianji_control::StepLease]) -> String {
    let mut rendered = format!(
        "# Qianji Control Leases\n\nActive leases: `{}`\n",
        leases.len()
    );
    for lease in leases {
        let _ = writeln!(
            rendered,
            "\n- Step: `{}` | Lease: `{}` | Worker: `{}` | Acquired at ms: `{}` | Expires at ms: `{}`",
            lease.step_id.as_str(),
            lease.lease_id.as_str(),
            lease.worker_id.as_str(),
            lease.acquired_at_ms,
            lease.expires_at_ms
        );
    }
    rendered
}

pub(crate) fn render_step_leases_json(
    leases: &[xiuxian_qianji_control::StepLease],
) -> io::Result<String> {
    serde_json::to_string_pretty(leases).map_err(io::Error::other)
}
