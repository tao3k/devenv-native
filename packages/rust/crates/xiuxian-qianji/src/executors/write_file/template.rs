use crate::scheduler_preflight::resolve_semantic_content;
use serde_json::Value;

pub(super) fn render_template(raw: &str, context: &Value) -> Result<String, String> {
    let semantic = resolve_semantic_content(raw, context)?;
    Ok(interpolate_braced_placeholders(semantic.as_str(), context))
}

fn interpolate_braced_placeholders(raw: &str, context: &Value) -> String {
    let mut rendered = String::with_capacity(raw.len());
    let mut remaining = raw;

    while let Some(open_index) = remaining.find("{{") {
        rendered.push_str(&remaining[..open_index]);
        let after_open = &remaining[open_index + 2..];
        let Some(close_index) = after_open.find("}}") else {
            rendered.push_str(&remaining[open_index..]);
            return rendered;
        };

        let token = after_open[..close_index].trim();
        if token.is_empty() {
            rendered.push_str("{{}}");
        } else if let Some(value) = lookup_context_value(context, token) {
            rendered.push_str(context_value_to_text(value).as_str());
        } else {
            rendered.push_str("{{");
            rendered.push_str(token);
            rendered.push_str("}}");
        }

        remaining = &after_open[close_index + 2..];
    }

    rendered.push_str(remaining);
    rendered
}

fn lookup_context_value<'a>(context: &'a Value, key_path: &str) -> Option<&'a Value> {
    let mut current = context;
    for segment in key_path.split('.') {
        let key = segment.trim();
        if key.is_empty() {
            continue;
        }
        current = current.get(key)?;
    }
    Some(current)
}

fn context_value_to_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}
