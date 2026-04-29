use super::Value;

pub(in crate::lint::bpmn::condition_contract) fn append_to_option(
    target: &mut Option<String>,
    text: &str,
) {
    target.get_or_insert_with(String::new).push_str(text);
}

pub(in crate::lint::bpmn::condition_contract) fn choice_values_from_assignment(
    text: &str,
) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(text.trim()) else {
        return Vec::new();
    };
    let Value::Array(items) = value else {
        return Vec::new();
    };
    items
        .into_iter()
        .filter_map(|item| match item {
            Value::String(value) => Some(value),
            Value::Object(mut object) => object
                .remove("value")
                .and_then(|value| value.as_str().map(ToOwned::to_owned)),
            _ => None,
        })
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}
