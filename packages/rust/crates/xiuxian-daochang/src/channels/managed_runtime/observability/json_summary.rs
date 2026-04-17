use serde_json::{Map, Value};

type JsonObject = Map<String, Value>;

#[derive(Debug)]
pub(crate) struct JsonReplySummary {
    pub(crate) kind: Option<String>,
    pub(crate) available: Option<bool>,
    pub(crate) status: Option<String>,
    pub(crate) audit_error: Option<String>,
    pub(crate) found: Option<bool>,
    pub(crate) decision: Option<String>,
    pub(crate) session_scope: Option<String>,
    pub(crate) logical_session_id: Option<String>,
    pub(crate) partition_key: Option<String>,
    pub(crate) partition_mode: Option<String>,
    pub(crate) context_mode: Option<String>,
    pub(crate) saved_snapshot_status: Option<String>,
    pub(crate) runtime_backend_ready: Option<bool>,
    pub(crate) runtime_active_backend: Option<String>,
    pub(crate) runtime_startup_load_status: Option<String>,
    pub(crate) admission_enabled: Option<bool>,
    pub(crate) admission_llm_reject_threshold_pct: Option<u64>,
    pub(crate) admission_embedding_reject_threshold_pct: Option<u64>,
    pub(crate) admission_metrics_total: Option<u64>,
    pub(crate) admission_metrics_admitted: Option<u64>,
    pub(crate) admission_metrics_rejected: Option<u64>,
    pub(crate) admission_metrics_rejected_llm_saturated: Option<u64>,
    pub(crate) admission_metrics_rejected_embedding_saturated: Option<u64>,
    pub(crate) admission_metrics_reject_rate_pct: Option<u64>,
    pub(crate) result_recalled_injected: Option<u64>,
    pub(crate) query_tokens: Option<u64>,
    pub(crate) override_admin_count: Option<usize>,
    pub(crate) keys: usize,
}

pub(crate) fn summarize_json_reply(message: &str) -> Option<JsonReplySummary> {
    let value: Value = serde_json::from_str(message).ok()?;
    let object = value.as_object()?;
    Some(build_summary(object))
}

fn build_summary(object: &JsonObject) -> JsonReplySummary {
    let saved_snapshot = nested_object(object, "saved_snapshot");
    let runtime = nested_object(object, "runtime");
    let admission = nested_object(object, "admission");
    let admission_metrics =
        admission.and_then(|admission_obj| nested_object(admission_obj, "metrics"));
    let result = nested_object(object, "result");
    JsonReplySummary {
        kind: string_field(object, "kind"),
        available: bool_field(object, "available"),
        status: string_field(object, "status"),
        audit_error: extract_audit_error(object),
        found: bool_field(object, "found"),
        decision: string_field(object, "decision"),
        session_scope: string_field(object, "session_scope"),
        logical_session_id: string_field(object, "logical_session_id"),
        partition_key: string_field(object, "partition_key"),
        partition_mode: string_field(object, "partition_mode"),
        context_mode: string_field(object, "mode"),
        saved_snapshot_status: nested_string_field(saved_snapshot, "status"),
        runtime_backend_ready: nested_bool_field(runtime, "backend_ready"),
        runtime_active_backend: nested_string_field(runtime, "active_backend"),
        runtime_startup_load_status: nested_string_field(runtime, "startup_load_status"),
        admission_enabled: nested_bool_field(admission, "enabled"),
        admission_llm_reject_threshold_pct: nested_u64_field(admission, "llm_reject_threshold_pct"),
        admission_embedding_reject_threshold_pct: nested_u64_field(
            admission,
            "embedding_reject_threshold_pct",
        ),
        admission_metrics_total: nested_u64_field(admission_metrics, "total"),
        admission_metrics_admitted: nested_u64_field(admission_metrics, "admitted"),
        admission_metrics_rejected: nested_u64_field(admission_metrics, "rejected"),
        admission_metrics_rejected_llm_saturated: nested_u64_field(
            admission_metrics,
            "rejected_llm_saturated",
        ),
        admission_metrics_rejected_embedding_saturated: nested_u64_field(
            admission_metrics,
            "rejected_embedding_saturated",
        ),
        admission_metrics_reject_rate_pct: nested_u64_field(admission_metrics, "reject_rate_pct"),
        result_recalled_injected: nested_u64_field(result, "recalled_injected"),
        query_tokens: u64_field(object, "query_tokens"),
        override_admin_count: extract_override_admin_count(object),
        keys: object.len(),
    }
}

fn nested_object<'a>(object: &'a JsonObject, key: &str) -> Option<&'a JsonObject> {
    object.get(key).and_then(Value::as_object)
}

fn string_field(object: &JsonObject, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn bool_field(object: &JsonObject, key: &str) -> Option<bool> {
    object.get(key).and_then(Value::as_bool)
}

fn u64_field(object: &JsonObject, key: &str) -> Option<u64> {
    object.get(key).and_then(Value::as_u64)
}

fn nested_string_field(object: Option<&JsonObject>, key: &str) -> Option<String> {
    object.and_then(|nested| string_field(nested, key))
}

fn nested_bool_field(object: Option<&JsonObject>, key: &str) -> Option<bool> {
    object.and_then(|nested| bool_field(nested, key))
}

fn nested_u64_field(object: Option<&JsonObject>, key: &str) -> Option<u64> {
    object.and_then(|nested| u64_field(nested, key))
}

fn extract_override_admin_count(object: &JsonObject) -> Option<usize> {
    match object.get("override_admin_users") {
        Some(Value::Array(entries)) => Some(entries.len()),
        Some(Value::Null) => Some(0),
        _ => None,
    }
}

fn extract_audit_error(object: &JsonObject) -> Option<String> {
    if let Some(single) = string_field(object, "audit_error") {
        return Some(single);
    }
    match object.get("audit_errors") {
        Some(Value::Array(entries)) => entries
            .iter()
            .find_map(Value::as_str)
            .map(ToString::to_string),
        _ => None,
    }
}

pub(crate) fn optional_bool_token(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    }
}

pub(crate) fn optional_u64_token(value: Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub(crate) fn optional_usize_token(value: Option<usize>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}
