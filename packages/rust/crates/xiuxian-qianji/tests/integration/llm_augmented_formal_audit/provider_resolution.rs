use super::*;

#[tokio::test]
async fn llm_augmented_audit_resolves_claude_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "claude-3-opus-20240229", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_resolves_anthropic_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "anthropic-claude-v1", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_resolves_gemini_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "gemini-pro", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_resolves_codex_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "gpt-4", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}
