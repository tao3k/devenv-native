#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::too_many_lines,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::assigning_clones
)]

//! HTTP gateway integration tests: validation (400), routing, response shape.
//! Uses a minimal Agent (no external tools) so no external services are required.

use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;
use xiuxian_daochang::{Agent, AgentConfig, router};

fn minimal_agent_config() -> AgentConfig {
    AgentConfig {
        inference_url: "https://api.openai.com/v1/chat/completions".to_string(),
        model: "gpt-4o-mini".to_string(),
        api_key: None,
        max_tool_rounds: 1,
        ..AgentConfig::default()
    }
}

#[tokio::test]
async fn gateway_returns_400_for_empty_session_id() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, None);

    let response = app
        .oneshot(
            Request::post("/message")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"session_id":"","message":"hi"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_returns_400_for_empty_message() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, None);

    let response = app
        .oneshot(
            Request::post("/message")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"session_id":"s1","message":"   "}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_returns_404_for_unknown_route() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, None);

    let response = app
        .oneshot(Request::get("/unknown").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gateway_embed_returns_400_for_empty_text() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, None);

    let response = app
        .oneshot(
            Request::post("/embed")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"text":"   "}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_embed_batch_returns_400_for_empty_texts() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, None);

    let response = app
        .oneshot(
            Request::post("/embed/batch")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"texts":[]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_openai_embeddings_returns_400_for_invalid_input_type() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, None);

    let response = app
        .oneshot(
            Request::post("/v1/embeddings")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"input":123}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_health_returns_structured_summary_without_tools() {
    let config = minimal_agent_config();
    let agent = Agent::from_config(config).await.expect("agent");
    let app = router(agent, 300, Some(4));

    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).expect("json body");

    assert_eq!(
        payload.get("status").and_then(Value::as_str),
        Some("healthy")
    );
    assert_eq!(
        payload.get("turn_timeout_secs").and_then(Value::as_u64),
        Some(300)
    );
    assert_eq!(
        payload.get("max_concurrent_turns").and_then(Value::as_u64),
        Some(4)
    );
    assert_eq!(
        payload.get("in_flight_turns").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("tools")
            .and_then(|tools| tools.get("enabled"))
            .and_then(Value::as_bool),
        Some(false)
    );
    let tools_list_cache = payload
        .get("tools")
        .and_then(|tools| tools.get("tools_list_cache"));
    assert!(
        tools_list_cache.is_none_or(Value::is_null),
        "tools_list_cache should be omitted or null when external tools are disabled"
    );
}
