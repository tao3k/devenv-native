use serde_json::json;

pub(super) fn parse_json_from_text(raw: &str) -> Option<serde_json::Value> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }

    let strip_fence = |candidate: &str| -> String {
        let without_open = candidate
            .strip_prefix("```json")
            .or_else(|| candidate.strip_prefix("```JSON"))
            .or_else(|| candidate.strip_prefix("```"))
            .unwrap_or(candidate)
            .trim()
            .to_string();
        without_open
            .strip_suffix("```")
            .unwrap_or(&without_open)
            .trim()
            .to_string()
    };

    let mut candidates = vec![strip_fence(text)];
    let fence_stripped = candidates[0].clone();

    let list_start = fence_stripped.find('[');
    let list_end = fence_stripped.rfind(']');
    if let (Some(start), Some(end)) = (list_start, list_end)
        && end > start
    {
        candidates.push(fence_stripped[start..=end].to_string());
    }

    let obj_start = fence_stripped.find('{');
    let obj_end = fence_stripped.rfind('}');
    if let (Some(start), Some(end)) = (obj_start, obj_end)
        && end > start
    {
        candidates.push(fence_stripped[start..=end].to_string());
    }

    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&candidate) {
            return Some(value);
        }
    }
    None
}

pub(super) fn build_repo_tree_fallback_plan(context: &serde_json::Value) -> serde_json::Value {
    let repo_tree = context
        .get("repo_tree")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let mut paths = repo_tree
        .lines()
        .filter_map(repo_tree_top_level_path)
        .take(12)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        paths.push(".".to_string());
    }
    json!([
        {
            "shard_id": "repository-overview",
            "paths": paths,
        }
    ])
}

fn repo_tree_top_level_path(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("./") || trimmed.matches('/').count() > 1 {
        return None;
    }
    let path = trimmed.trim_start_matches("./").trim();
    (!path.is_empty()).then(|| path.to_string())
}

pub(super) fn context_non_empty_string(context: &serde_json::Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn resolve_model_for_request(
    context: &serde_json::Value,
    default_model: &str,
) -> String {
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
