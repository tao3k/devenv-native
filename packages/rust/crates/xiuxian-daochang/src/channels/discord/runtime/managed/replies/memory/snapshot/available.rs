use serde_json::json;

use crate::agent::{
    DownstreamAdmissionRuntimeSnapshot, MemoryRecallMetricsSnapshot, MemoryRuntimeStatusSnapshot,
    SessionMemoryRecallSnapshot,
};

use crate::channels::discord::runtime::managed::replies::memory::metrics::{
    format_memory_recall_metrics_json, format_memory_recall_metrics_lines,
};
use crate::channels::discord::runtime::managed::replies::memory::runtime_status::{
    format_downstream_admission_status_json, format_downstream_admission_status_lines,
    format_memory_runtime_status_json, format_memory_runtime_status_lines,
};
use crate::channels::discord::runtime::managed::replies::session_reply_format::{
    format_optional_f32, format_optional_usize,
};

pub(in super::super::super::super) fn format_memory_recall_snapshot(
    snapshot: SessionMemoryRecallSnapshot,
    metrics: MemoryRecallMetricsSnapshot,
    runtime_status: MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    let mut lines = vec![
        "## Session Memory".to_string(),
        format!("Captured at unix ms: `{}`", snapshot.created_at_unix_ms),
        format!("- Session scope: `{session_scope}`"),
        String::new(),
        "### Trigger".to_string(),
        format!("- Decision: `{}`", snapshot.decision.as_str()),
        format!("- Query tokens: `{}`", snapshot.query_tokens),
        format!(
            "- Recall feedback bias: `{:.3}`",
            snapshot.recall_feedback_bias
        ),
        format!("- Embedding source: `{}`", snapshot.embedding_source),
        format!(
            "- Pipeline duration: `{} ms`",
            snapshot.pipeline_duration_ms
        ),
        String::new(),
        "### Persistence".to_string(),
    ];
    lines.extend(format_memory_runtime_status_lines(runtime_status));
    lines.extend([String::new(), "### Admission".to_string()]);
    lines.extend(format_downstream_admission_status_lines(admission_status));
    lines.extend([
        String::new(),
        "### Recall Plan".to_string(),
        format!("- `k1={}` / `k2={}`", snapshot.k1, snapshot.k2),
        format!("- `lambda={:.3}`", snapshot.lambda),
        format!("- `min_score={:.3}`", snapshot.min_score),
        format!("- `max_context_chars={}`", snapshot.max_context_chars),
        String::new(),
        "### Context Pressure".to_string(),
        format!("- `budget_pressure={:.3}`", snapshot.budget_pressure),
        format!("- `window_pressure={:.3}`", snapshot.window_pressure),
        format!(
            "- `effective_budget_tokens={}`",
            format_optional_usize(snapshot.effective_budget_tokens)
        ),
        format!(
            "- `active_turns_estimate={}`",
            snapshot.active_turns_estimate
        ),
        format!(
            "- `summary_segment_count={}`",
            snapshot.summary_segment_count
        ),
        String::new(),
        "### Recall Result".to_string(),
        format!("- `recalled_total={}`", snapshot.recalled_total),
        format!("- `recalled_selected={}`", snapshot.recalled_selected),
        format!("- `recalled_injected={}`", snapshot.recalled_injected),
        format!(
            "- `context_chars_injected={}`",
            snapshot.context_chars_injected
        ),
        format!(
            "- `best_score={}`",
            format_optional_f32(snapshot.best_score)
        ),
        format!(
            "- `weakest_score={}`",
            format_optional_f32(snapshot.weakest_score)
        ),
        String::new(),
        "### Process Metrics".to_string(),
    ]);
    lines.extend(format_memory_recall_metrics_lines(metrics));
    lines.join("\n")
}

pub(in super::super::super::super) fn format_memory_recall_snapshot_json(
    snapshot: SessionMemoryRecallSnapshot,
    metrics: MemoryRecallMetricsSnapshot,
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    json!({
        "kind": "session_memory",
        "available": true,
        "session_scope": session_scope,
        "captured_at_unix_ms": snapshot.created_at_unix_ms,
        "decision": snapshot.decision.as_str(),
        "query_tokens": snapshot.query_tokens,
        "recall_feedback_bias": snapshot.recall_feedback_bias,
        "embedding_source": snapshot.embedding_source,
        "pipeline_duration_ms": snapshot.pipeline_duration_ms,
        "plan": {
            "k1": snapshot.k1,
            "k2": snapshot.k2,
            "lambda": snapshot.lambda,
            "min_score": snapshot.min_score,
            "max_context_chars": snapshot.max_context_chars,
        },
        "context_pressure": {
            "budget_pressure": snapshot.budget_pressure,
            "window_pressure": snapshot.window_pressure,
            "effective_budget_tokens": snapshot.effective_budget_tokens,
            "active_turns_estimate": snapshot.active_turns_estimate,
            "summary_segment_count": snapshot.summary_segment_count,
        },
        "result": {
            "recalled_total": snapshot.recalled_total,
            "recalled_selected": snapshot.recalled_selected,
            "recalled_injected": snapshot.recalled_injected,
            "context_chars_injected": snapshot.context_chars_injected,
            "best_score": snapshot.best_score,
            "weakest_score": snapshot.weakest_score,
        },
        "runtime": format_memory_runtime_status_json(runtime_status),
        "admission": format_downstream_admission_status_json(&admission_status),
        "metrics": format_memory_recall_metrics_json(metrics),
    })
    .to_string()
}
