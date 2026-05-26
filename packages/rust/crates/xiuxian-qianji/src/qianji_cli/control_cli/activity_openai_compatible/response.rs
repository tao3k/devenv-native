use serde_json::Value;

pub(super) fn openai_message_content(provider_response: &Value) -> Option<&str> {
    provider_response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .filter(|content| !content.trim().is_empty())
}

pub(super) fn retryable_contract_invalid(message: &str, provider_response: &Value) -> bool {
    openai_finish_reason(provider_response).is_some_and(|reason| reason == "length")
        || message.contains("EOF while parsing")
}

fn openai_finish_reason(provider_response: &Value) -> Option<&str> {
    provider_response
        .pointer("/choices/0/finish_reason")
        .or_else(|| provider_response.pointer("/choices/0/native_finish_reason"))
        .and_then(Value::as_str)
}

pub(super) fn body_preview(body: &str) -> String {
    body.chars().take(4096).collect()
}
