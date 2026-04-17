use super::extract_litellm_chat_stream_chunk_text;
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
