//! BPMN LLM completion shaping for strict task output bindings.

use serde_json::Value;

pub(super) fn shape_llm_content_for_bpmn_outputs(
    content: &str,
    output_bindings: &[String],
) -> Value {
    match parse_llm_json_content(content) {
        Some(value) => shape_json_for_bpmn_outputs(value, content, output_bindings),
        None => shape_text_for_bpmn_outputs(content, output_bindings),
    }
}

fn parse_llm_json_content(content: &str) -> Option<Value> {
    serde_json::from_str(content)
        .ok()
        .or_else(|| fenced_json_body(content).and_then(|body| serde_json::from_str(body).ok()))
}

fn fenced_json_body(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    let rest = trimmed.strip_prefix("```")?;
    let body_start = rest.find('\n')? + 1;
    let body = &rest[body_start..];
    let body_end = body.rfind("```")?;
    let candidate = body[..body_end].trim();
    (!candidate.is_empty()).then_some(candidate)
}

fn shape_json_for_bpmn_outputs(
    value: Value,
    raw_content: &str,
    output_bindings: &[String],
) -> Value {
    if output_bindings.is_empty() {
        return value;
    }
    if let Value::Object(map) = &value
        && output_bindings
            .iter()
            .all(|name| map.contains_key(name.as_str()))
    {
        return value;
    }
    if output_bindings.len() == 1 {
        let output_name = &output_bindings[0];
        if let Some(content) = value.get("content").and_then(Value::as_str) {
            return serde_json::json!({ output_name: content });
        }
        return serde_json::json!({ output_name: raw_content });
    }
    value
}

fn shape_text_for_bpmn_outputs(content: &str, output_bindings: &[String]) -> Value {
    if output_bindings.len() == 1 {
        let output_name = &output_bindings[0];
        return serde_json::json!({ output_name: content });
    }
    serde_json::json!({ "content": content })
}
