use serde_json::json;

use crate::agent::{
    DownstreamAdmissionRuntimeSnapshot, MemoryRecallMetricsSnapshot, MemoryRuntimeStatusSnapshot,
    SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot, SessionContextMode,
    SessionContextSnapshotInfo, SessionContextWindowInfo, SessionMemoryRecallSnapshot,
    SessionRecallFeedbackDirection,
};

use super::{format_optional_f32, format_optional_usize};

pub(crate) fn format_session_feedback(
    direction: SessionRecallFeedbackDirection,
    previous_bias: f32,
    updated_bias: f32,
) -> String {
    let direction_label = match direction {
        SessionRecallFeedbackDirection::Up => "up",
        SessionRecallFeedbackDirection::Down => "down",
    };
    format!(
        "Session recall feedback updated.\ndirection={direction_label}\nprevious_bias={previous_bias:.3}\nupdated_bias={updated_bias:.3}"
    )
}

pub(crate) fn format_session_feedback_json(
    direction: SessionRecallFeedbackDirection,
    previous_bias: f32,
    updated_bias: f32,
) -> String {
    let direction_label = match direction {
        SessionRecallFeedbackDirection::Up => "up",
        SessionRecallFeedbackDirection::Down => "down",
    };
    json!({
        "kind": "session_feedback",
        "applied": true,
        "direction": direction_label,
        "previous_bias": previous_bias,
        "updated_bias": updated_bias,
    })
    .to_string()
}

pub(crate) fn format_session_feedback_unavailable_json() -> String {
    json!({
        "kind": "session_feedback",
        "applied": false,
        "reason": "memory_disabled",
        "message": "Session recall feedback is unavailable because memory is disabled.",
    })
    .to_string()
}

pub(crate) fn format_command_error_json(command: &str, error: &str) -> String {
    json!({
        "kind": "command_error",
        "command": command,
        "status": "error",
        "error": error,
    })
    .to_string()
}

pub(crate) fn format_session_context_snapshot(
    session_id: &str,
    partition_key: &str,
    partition_mode: &str,
    active: SessionContextWindowInfo,
    snapshot: Option<SessionContextSnapshotInfo>,
    admission: DownstreamAdmissionRuntimeSnapshot,
) -> String {
    let mut lines = vec![
        "============================================================".to_string(),
        "session-context dashboard".to_string(),
        "============================================================".to_string(),
        "Overview:".to_string(),
        format!("  logical_session_id={session_id}"),
        format!("  partition_key={partition_key}"),
        format!("  partition_mode={partition_mode}"),
        format!("  mode={}", format_context_mode(active.mode)),
        "------------------------------------------------------------".to_string(),
        "Active:".to_string(),
        format!("  messages={}", active.messages),
        format!("  summary_segments={}", active.summary_segments),
    ];
    if let Some(window_turns) = active.window_turns {
        lines.push(format!("  window_turns={window_turns}"));
    }
    if let Some(window_slots) = active.window_slots {
        lines.push(format!("  window_slots={window_slots}"));
    }
    if let Some(total_tool_calls) = active.total_tool_calls {
        lines.push(format!("  window_tool_calls={total_tool_calls}"));
    }
    lines.push("------------------------------------------------------------".to_string());
    lines.push("Saved Snapshot:".to_string());
    match snapshot {
        Some(info) => {
            lines.push("  status=available".to_string());
            lines.push(format!("  saved_messages={}", info.messages));
            lines.push(format!(
                "  saved_summary_segments={}",
                info.summary_segments
            ));
            if let Some(saved_at_unix_ms) = info.saved_at_unix_ms {
                lines.push(format!("  saved_at_unix_ms={saved_at_unix_ms}"));
            }
            if let Some(saved_age_secs) = info.saved_age_secs {
                lines.push(format!("  saved_age_secs={saved_age_secs}"));
            }
            lines.push("  restore_hint=/resume".to_string());
        }
        None => lines.push("  status=none".to_string()),
    }
    lines.push("------------------------------------------------------------".to_string());
    lines.push("Admission:".to_string());
    lines.extend(format_downstream_admission_status_lines(admission));
    lines.push("============================================================".to_string());
    lines.join("\n")
}

pub(crate) fn format_session_context_snapshot_json(
    session_id: &str,
    partition_key: &str,
    partition_mode: &str,
    active: SessionContextWindowInfo,
    snapshot: Option<SessionContextSnapshotInfo>,
    admission: DownstreamAdmissionRuntimeSnapshot,
) -> String {
    let snapshot_json = match snapshot {
        Some(info) => json!({
            "status": "available",
            "saved_messages": info.messages,
            "saved_summary_segments": info.summary_segments,
            "saved_at_unix_ms": info.saved_at_unix_ms,
            "saved_age_secs": info.saved_age_secs,
            "restore_hint": "/resume",
        }),
        None => json!({
            "status": "none",
        }),
    };

    json!({
        "kind": "session_context",
        "logical_session_id": session_id,
        "partition_key": partition_key,
        "partition_mode": partition_mode,
        "mode": format_context_mode(active.mode),
        "active": {
            "messages": active.messages,
            "summary_segments": active.summary_segments,
            "window_turns": active.window_turns,
            "window_slots": active.window_slots,
            "window_tool_calls": active.total_tool_calls,
        },
        "saved_snapshot": snapshot_json,
        "admission": format_downstream_admission_status_json(&admission),
    })
    .to_string()
}

pub(crate) fn format_context_budget_snapshot(snapshot: &SessionContextBudgetSnapshot) -> String {
    let (largest_drop, largest_trunc) = compute_largest_bottlenecks(snapshot);
    let mut lines = vec![
        "============================================================".to_string(),
        "session-budget dashboard".to_string(),
        "============================================================".to_string(),
        "Overview:".to_string(),
        format!("  captured_at_unix_ms={}", snapshot.created_at_unix_ms),
        format!("  strategy={}", snapshot.strategy.as_str()),
        format!(
            "  budget={} reserve={} effective={}",
            snapshot.budget_tokens, snapshot.reserve_tokens, snapshot.effective_budget_tokens
        ),
        format!(
            "  messages={} -> {} (dropped={})",
            snapshot.pre_messages, snapshot.post_messages, snapshot.dropped_messages
        ),
        format!(
            "  tokens={} -> {} (dropped={})",
            snapshot.pre_tokens, snapshot.post_tokens, snapshot.dropped_tokens
        ),
        "------------------------------------------------------------".to_string(),
        "Classes:".to_string(),
        "  class           in_msg  kept  drop  trunc  in_tok  kept   drop   trunc".to_string(),
    ];
    lines.extend(format_context_budget_class_row(
        "non_system",
        &snapshot.non_system,
    ));
    lines.extend(format_context_budget_class_row(
        "regular_system",
        &snapshot.regular_system,
    ));
    lines.extend(format_context_budget_class_row(
        "summary_system",
        &snapshot.summary_system,
    ));
    lines.extend([
        "------------------------------------------------------------".to_string(),
        "Bottlenecks:".to_string(),
        format!(
            "  largest_dropped_tokens={} ({})",
            largest_drop.0, largest_drop.1
        ),
        format!(
            "  largest_truncated_tokens={} ({})",
            largest_trunc.0, largest_trunc.1
        ),
        "============================================================".to_string(),
    ]);
    lines.join("\n")
}

pub(crate) fn format_context_budget_snapshot_json(
    snapshot: &SessionContextBudgetSnapshot,
) -> String {
    let (largest_drop, largest_trunc) = compute_largest_bottlenecks(snapshot);
    json!({
        "kind": "session_budget",
        "available": true,
        "captured_at_unix_ms": snapshot.created_at_unix_ms,
        "strategy": snapshot.strategy.as_str(),
        "budget_tokens": snapshot.budget_tokens,
        "reserve_tokens": snapshot.reserve_tokens,
        "effective_budget_tokens": snapshot.effective_budget_tokens,
        "messages": {
            "pre": snapshot.pre_messages,
            "post": snapshot.post_messages,
            "dropped": snapshot.dropped_messages,
        },
        "tokens": {
            "pre": snapshot.pre_tokens,
            "post": snapshot.post_tokens,
            "dropped": snapshot.dropped_tokens,
        },
        "classes": {
            "non_system": format_context_budget_class_json(&snapshot.non_system),
            "regular_system": format_context_budget_class_json(&snapshot.regular_system),
            "summary_system": format_context_budget_class_json(&snapshot.summary_system),
        },
        "bottlenecks": {
            "largest_dropped_tokens": {"class": largest_drop.0, "tokens": largest_drop.1},
            "largest_truncated_tokens": {"class": largest_trunc.0, "tokens": largest_trunc.1},
        },
    })
    .to_string()
}

pub(crate) fn format_context_budget_not_found_json() -> String {
    json!({
        "kind": "session_budget",
        "available": false,
        "status": "not_found",
        "hint": "Run at least one normal turn first (non-command message).",
    })
    .to_string()
}

pub(crate) fn format_memory_recall_snapshot(
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

pub(crate) fn format_memory_recall_snapshot_json(
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

pub(crate) fn format_memory_recall_not_found(
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

pub(crate) fn format_memory_recall_not_found_json(
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

pub(crate) fn format_memory_recall_compact_snapshot(
    snapshot: SessionMemoryRecallSnapshot,
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    [
        "## Session Memory".to_string(),
        format!("- Session scope: `{session_scope}`"),
        String::new(),
        "### Trigger - Decision".to_string(),
        format!(
            "- `decision={}` `query_tokens={}` `pipeline_ms={}`",
            snapshot.decision.as_str(),
            snapshot.query_tokens,
            snapshot.pipeline_duration_ms
        ),
        format!(
            "- `feedback_bias={:.3}` `embedding_source={}`",
            snapshot.recall_feedback_bias, snapshot.embedding_source
        ),
        String::new(),
        "### Recall Result".to_string(),
        format!(
            "- `injected={}` / `selected={}` / `total={}`",
            snapshot.recalled_injected, snapshot.recalled_selected, snapshot.recalled_total
        ),
        format!(
            "- `context_chars={}` `best_score={}` `weakest_score={}`",
            snapshot.context_chars_injected,
            format_optional_f32(snapshot.best_score),
            format_optional_f32(snapshot.weakest_score)
        ),
        String::new(),
        "### Adaptive Metrics".to_string(),
        format_memory_runtime_status_compact(runtime_status),
        format_downstream_admission_status_compact(&admission_status),
        "Tip: run `/session memory json` for full payload.".to_string(),
    ]
    .join("\n")
}

pub(crate) fn format_memory_recall_compact_not_found(
    runtime_status: &MemoryRuntimeStatusSnapshot,
    admission_status: DownstreamAdmissionRuntimeSnapshot,
    session_scope: &str,
) -> String {
    [
        "## Session Memory".to_string(),
        "No memory recall snapshot found for this session yet.".to_string(),
        format!("- Session scope: `{session_scope}`"),
        String::new(),
        "### Persistence".to_string(),
        format_memory_runtime_status_compact(runtime_status),
        format_downstream_admission_status_compact(&admission_status),
        "Use `/session memory json` for full payload.".to_string(),
    ]
    .join("\n")
}

fn format_context_mode(mode: SessionContextMode) -> &'static str {
    match mode {
        SessionContextMode::Bounded => "bounded",
        SessionContextMode::Unbounded => "unbounded",
    }
}

fn compute_largest_bottlenecks(
    snapshot: &SessionContextBudgetSnapshot,
) -> ((&'static str, usize), (&'static str, usize)) {
    let classes = [
        ("non_system", snapshot.non_system),
        ("regular_system", snapshot.regular_system),
        ("summary_system", snapshot.summary_system),
    ];
    let mut largest_drop = ("none", 0usize);
    let mut largest_trunc = ("none", 0usize);
    for (name, class) in classes {
        if class.dropped_tokens > largest_drop.1 {
            largest_drop = (name, class.dropped_tokens);
        }
        if class.truncated_tokens > largest_trunc.1 {
            largest_trunc = (name, class.truncated_tokens);
        }
    }
    (largest_drop, largest_trunc)
}

fn format_context_budget_class_json(
    stats: &SessionContextBudgetClassSnapshot,
) -> serde_json::Value {
    json!({
        "input_messages": stats.input_messages,
        "kept_messages": stats.kept_messages,
        "dropped_messages": stats.dropped_messages,
        "truncated_messages": stats.truncated_messages,
        "input_tokens": stats.input_tokens,
        "kept_tokens": stats.kept_tokens,
        "dropped_tokens": stats.dropped_tokens,
        "truncated_tokens": stats.truncated_tokens,
    })
}

fn format_context_budget_class_row(
    label: &str,
    stats: &SessionContextBudgetClassSnapshot,
) -> Vec<String> {
    vec![format!(
        "  {label:<14} {in_msg:>6} {kept:>5} {drop:>5} {trunc:>6} {in_tok:>7} {kept_tok:>6} {drop_tok:>6} {trunc_tok:>7}",
        in_msg = stats.input_messages,
        kept = stats.kept_messages,
        drop = stats.dropped_messages,
        trunc = stats.truncated_messages,
        in_tok = stats.input_tokens,
        kept_tok = stats.kept_tokens,
        drop_tok = stats.dropped_tokens,
        trunc_tok = stats.truncated_tokens,
    )]
}

fn is_backend_ready(
    enabled: bool,
    active_backend_present: bool,
    startup_load_status: &str,
) -> bool {
    enabled && active_backend_present && startup_load_status == "loaded"
}

fn format_optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "-".to_string(), format_yes_no)
}

fn format_optional_str(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_string(), ToString::to_string)
}

fn format_optional_string(value: Option<String>) -> String {
    value.unwrap_or_else(|| "-".to_string())
}

fn format_yes_no(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

fn format_memory_runtime_status_lines(status: MemoryRuntimeStatusSnapshot) -> Vec<String> {
    let backend_ready = is_backend_ready(
        status.enabled,
        status.active_backend.is_some(),
        status.startup_load_status,
    );
    vec![
        format!("- `memory_enabled={}`", format_yes_no(status.enabled)),
        format!(
            "- `configured_backend={}` / `active_backend={}`",
            format_optional_string(status.configured_backend),
            format_optional_str(status.active_backend)
        ),
        format!(
            "- `strict_startup={}` / `startup_load_status={}` / `backend_ready={}`",
            format_optional_bool(status.strict_startup),
            status.startup_load_status,
            format_yes_no(backend_ready)
        ),
        format!(
            "- `store_path={}` / `table_name={}`",
            format_optional_string(status.store_path),
            format_optional_string(status.table_name)
        ),
        format!(
            "- `episodes_total={}` / `q_values_total={}`",
            status
                .episodes_total
                .map_or_else(|| "-".to_string(), |value| value.to_string()),
            status
                .q_values_total
                .map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
        format!(
            "- `gate_promote_threshold={}` / `gate_obsolete_threshold={}`",
            format_optional_f32(status.gate_promote_threshold),
            format_optional_f32(status.gate_obsolete_threshold)
        ),
        format!(
            "- `gate_promote_min_usage={}` / `gate_obsolete_min_usage={}`",
            format_optional_usize(status.gate_promote_min_usage.map(|value| value as usize)),
            format_optional_usize(status.gate_obsolete_min_usage.map(|value| value as usize))
        ),
    ]
}

fn format_memory_runtime_status_json(status: &MemoryRuntimeStatusSnapshot) -> serde_json::Value {
    let backend_ready = is_backend_ready(
        status.enabled,
        status.active_backend.is_some(),
        status.startup_load_status,
    );
    json!({
        "memory_enabled": status.enabled,
        "configured_backend": status.configured_backend,
        "active_backend": status.active_backend,
        "strict_startup": status.strict_startup,
        "startup_load_status": status.startup_load_status,
        "backend_ready": backend_ready,
        "store_path": status.store_path,
        "table_name": status.table_name,
        "gate_promote_threshold": status.gate_promote_threshold,
        "gate_obsolete_threshold": status.gate_obsolete_threshold,
        "gate_promote_min_usage": status.gate_promote_min_usage,
        "gate_obsolete_min_usage": status.gate_obsolete_min_usage,
        "gate_promote_failure_rate_ceiling": status.gate_promote_failure_rate_ceiling,
        "gate_obsolete_failure_rate_floor": status.gate_obsolete_failure_rate_floor,
        "gate_promote_min_ttl_score": status.gate_promote_min_ttl_score,
        "gate_obsolete_max_ttl_score": status.gate_obsolete_max_ttl_score,
        "episodes_total": status.episodes_total,
        "q_values_total": status.q_values_total,
    })
}

fn format_downstream_admission_status_lines(
    status: DownstreamAdmissionRuntimeSnapshot,
) -> Vec<String> {
    vec![
        format!("- `enabled={}`", format_yes_no(status.enabled)),
        format!(
            "- `llm_reject_threshold_pct={}` / `embedding_reject_threshold_pct={}`",
            status.llm_reject_threshold_pct, status.embedding_reject_threshold_pct
        ),
        format!(
            "- `total={}` / `admitted={}` / `rejected={}` / `reject_rate_pct={}`",
            status.metrics.total,
            status.metrics.admitted,
            status.metrics.rejected,
            status.metrics.reject_rate_pct
        ),
        format!(
            "- `rejected_llm_saturated={}` / `rejected_embedding_saturated={}`",
            status.metrics.rejected_llm_saturated, status.metrics.rejected_embedding_saturated
        ),
    ]
}

fn format_downstream_admission_status_json(
    status: &DownstreamAdmissionRuntimeSnapshot,
) -> serde_json::Value {
    json!({
        "enabled": status.enabled,
        "llm_reject_threshold_pct": status.llm_reject_threshold_pct,
        "embedding_reject_threshold_pct": status.embedding_reject_threshold_pct,
        "metrics": {
            "total": status.metrics.total,
            "admitted": status.metrics.admitted,
            "rejected": status.metrics.rejected,
            "rejected_llm_saturated": status.metrics.rejected_llm_saturated,
            "rejected_embedding_saturated": status.metrics.rejected_embedding_saturated,
            "reject_rate_pct": status.metrics.reject_rate_pct,
        },
    })
}

fn format_memory_recall_metrics_lines(metrics: MemoryRecallMetricsSnapshot) -> Vec<String> {
    vec![
        format!("- `planned_total={}`", metrics.planned_total),
        format!(
            "- `completed_total={}` / `injected={}` / `skipped={}`",
            metrics.completed_total, metrics.injected_total, metrics.skipped_total
        ),
        format!(
            "- `selected_total={}` / `injected_items_total={}`",
            metrics.selected_total, metrics.injected_items_total
        ),
        format!(
            "- `context_chars_injected_total={}`",
            metrics.context_chars_injected_total
        ),
        format!(
            "- `avg_pipeline_duration_ms={:.2}` / `total_pipeline_duration_ms={}`",
            metrics.avg_pipeline_duration_ms, metrics.pipeline_duration_ms_total
        ),
        format!(
            "- `injected_rate={:.3}` / `avg_selected_per_completed={:.3}` / `avg_injected_per_injected={:.3}`",
            metrics.injected_rate,
            metrics.avg_selected_per_completed,
            metrics.avg_injected_per_injected
        ),
        format!(
            "- `embedding_success_total={}` / `embedding_timeout_total={}` / `embedding_cooldown_reject_total={}` / `embedding_unavailable_total={}`",
            metrics.embedding_success_total,
            metrics.embedding_timeout_total,
            metrics.embedding_cooldown_reject_total,
            metrics.embedding_unavailable_total
        ),
    ]
}

fn format_memory_recall_metrics_json(metrics: MemoryRecallMetricsSnapshot) -> serde_json::Value {
    json!({
        "captured_at_unix_ms": metrics.captured_at_unix_ms,
        "planned_total": metrics.planned_total,
        "injected_total": metrics.injected_total,
        "skipped_total": metrics.skipped_total,
        "completed_total": metrics.completed_total,
        "selected_total": metrics.selected_total,
        "injected_items_total": metrics.injected_items_total,
        "context_chars_injected_total": metrics.context_chars_injected_total,
        "pipeline_duration_ms_total": metrics.pipeline_duration_ms_total,
        "avg_pipeline_duration_ms": metrics.avg_pipeline_duration_ms,
        "avg_selected_per_completed": metrics.avg_selected_per_completed,
        "avg_injected_per_injected": metrics.avg_injected_per_injected,
        "injected_rate": metrics.injected_rate,
        "embedding_success_total": metrics.embedding_success_total,
        "embedding_timeout_total": metrics.embedding_timeout_total,
        "embedding_cooldown_reject_total": metrics.embedding_cooldown_reject_total,
        "embedding_unavailable_total": metrics.embedding_unavailable_total,
        "latency_buckets_ms": {
            "le_10ms": metrics.latency_buckets.le_10ms,
            "le_25ms": metrics.latency_buckets.le_25ms,
            "le_50ms": metrics.latency_buckets.le_50ms,
            "le_100ms": metrics.latency_buckets.le_100ms,
            "le_250ms": metrics.latency_buckets.le_250ms,
            "le_500ms": metrics.latency_buckets.le_500ms,
            "gt_500ms": metrics.latency_buckets.gt_500ms,
        },
    })
}

fn format_memory_runtime_status_compact(status: &MemoryRuntimeStatusSnapshot) -> String {
    let backend_ready = is_backend_ready(
        status.enabled,
        status.active_backend.is_some(),
        status.startup_load_status,
    );
    format!(
        "- `memory_enabled={}` `backend_ready={}` `startup_load_status={}`\n- `promote(threshold={},min_usage={},failure_ceiling={},min_ttl={})` `obsolete(threshold={},min_usage={},failure_floor={},max_ttl={})`",
        format_yes_no(status.enabled),
        format_yes_no(backend_ready),
        status.startup_load_status,
        format_optional_f32(status.gate_promote_threshold),
        format_optional_usize(status.gate_promote_min_usage.map(|value| value as usize)),
        format_optional_f32(status.gate_promote_failure_rate_ceiling),
        format_optional_f32(status.gate_promote_min_ttl_score),
        format_optional_f32(status.gate_obsolete_threshold),
        format_optional_usize(status.gate_obsolete_min_usage.map(|value| value as usize)),
        format_optional_f32(status.gate_obsolete_failure_rate_floor),
        format_optional_f32(status.gate_obsolete_max_ttl_score),
    )
}

fn format_downstream_admission_status_compact(
    status: &DownstreamAdmissionRuntimeSnapshot,
) -> String {
    format!(
        "- `admission(enabled={},total={},admitted={},rejected={},reject_rate_pct={})`",
        format_yes_no(status.enabled),
        status.metrics.total,
        status.metrics.admitted,
        status.metrics.rejected,
        status.metrics.reject_rate_pct
    )
}
