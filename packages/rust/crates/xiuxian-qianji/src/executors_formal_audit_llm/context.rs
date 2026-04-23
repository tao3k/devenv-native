use serde_json::Value;

pub(super) fn context_non_empty_string(context: &Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn resolve_model_for_request(context: &Value, default_model: &str) -> String {
    if let Some(explicit_override) = context_non_empty_string(context, "llm_model") {
        return explicit_override;
    }
    let default_trimmed = default_model.trim();
    if !default_trimmed.is_empty() {
        return default_trimmed.to_string();
    }
    if let Some(fallback) = context_non_empty_string(context, "llm_model_fallback") {
        return fallback;
    }
    default_trimmed.to_string()
}
