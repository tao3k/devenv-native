use serde_json::json;

use crate::agent::{
    DownstreamAdmissionRuntimeSnapshot, MemoryRecallMetricsSnapshot, MemoryRuntimeStatusSnapshot,
};

use crate::channels::discord::runtime::managed::replies::memory::metrics::format_memory_recall_metrics_json;
use crate::channels::discord::runtime::managed::replies::memory::runtime_status::{
    format_downstream_admission_status_json, format_downstream_admission_status_lines,
    format_memory_runtime_status_json, format_memory_runtime_status_lines,
};

pub(in super::super::super::super) fn format_memory_recall_not_found(
    runtime_status: MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    let mut lines = vec![
        "## Session Memory".to_string(),
        "No memory recall snapshot found for this session yet.".to_string(),
        format!("- Session scope: `{session_scope}`"),
        String::new(),
        "### Persistence".to_string(),
    ];
    lines.extend(format_memory_runtime_status_lines(runtime_status));
    lines.extend([String::new(), "### Admission".to_string()]);
    lines.extend(format_downstream_admission_status_lines(admission_status));
    lines.extend([
        String::new(),
        "### Next Step".to_string(),
        "- Send at least one normal turn first (non-command message).".to_string(),
        "- Then run `/session memory` again.".to_string(),
    ]);
    lines.join("\n")
}

pub(in super::super::super::super) fn format_memory_recall_not_found_json(
    metrics: MemoryRecallMetricsSnapshot,
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    json!({
        "kind": "session_memory",
        "available": false,
        "session_scope": session_scope,
        "status": "not_found",
        "hint": "Run at least one normal turn first (non-command message).",
        "runtime": format_memory_runtime_status_json(runtime_status),
        "admission": format_downstream_admission_status_json(&admission_status),
        "metrics": format_memory_recall_metrics_json(metrics),
    })
    .to_string()
}
