#![cfg(feature = "provider-litellm")]

use litellm_rs::core::types::chat::ChatMessage as LiteChatMessage;
use litellm_rs::core::types::message::{
    MessageContent as LiteMessageContent, MessageRole as LiteMessageRole,
};

use xiuxian_llm::llm::providers::split_anthropic_system_messages;

#[test]
fn split_anthropic_system_messages_promotes_developer_role_to_system_prompt() {
    let messages = vec![
        LiteChatMessage {
            role: LiteMessageRole::Developer,
            content: Some(LiteMessageContent::Text(
                "Prefer terse structured answers.".to_string(),
            )),
            thinking: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
            audio: None,
        },
        LiteChatMessage {
            role: LiteMessageRole::System,
            content: Some(LiteMessageContent::Text(
                "Honor the active project policy.".to_string(),
            )),
            thinking: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
            audio: None,
        },
        LiteChatMessage {
            role: LiteMessageRole::User,
            content: Some(LiteMessageContent::Text("hello".to_string())),
            thinking: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
            audio: None,
        },
    ];

    let (system, others) = split_anthropic_system_messages(&messages);

    assert_eq!(
        system.as_deref(),
        Some("Prefer terse structured answers.\nHonor the active project policy.")
    );
    assert_eq!(others.len(), 1);
    assert!(matches!(others[0].role, LiteMessageRole::User));
}
