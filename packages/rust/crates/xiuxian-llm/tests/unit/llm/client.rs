use super::{
    extract_litellm_chat_stream_chunk_text, openai_like_chat_completions_endpoint, trimmed_api_key,
};
use litellm_rs::core::types::responses::{ChatChunk, ChatDelta, ChatStreamChoice};

#[test]
fn stream_chunk_text_ignores_reasoning_only_deltas() {
    let chunk = ChatChunk {
        id: "chunk_1".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1,
        model: "mimo-v2-pro".to_string(),
        choices: vec![
            ChatStreamChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: None,
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                    audio: None,
                },
                finish_reason: None,
                logprobs: None,
            },
            ChatStreamChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: Some("<score>0.95</score>".to_string()),
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                    audio: None,
                },
                finish_reason: None,
                logprobs: None,
            },
        ],
        usage: None,
        system_fingerprint: None,
    };

    assert_eq!(
        extract_litellm_chat_stream_chunk_text(&chunk),
        "<score>0.95</score>"
    );
}

#[test]
fn chat_completions_endpoint_accepts_base_or_full_path() {
    assert_eq!(
        openai_like_chat_completions_endpoint("https://provider.example/v1"),
        "https://provider.example/v1/chat/completions"
    );
    assert_eq!(
        openai_like_chat_completions_endpoint("https://provider.example/v1/chat/completions"),
        "https://provider.example/v1/chat/completions"
    );
}

#[test]
fn trimmed_api_key_skips_empty_values() {
    assert_eq!(trimmed_api_key("  test-key  "), Some("test-key"));
    assert_eq!(trimmed_api_key("   "), None);
}
