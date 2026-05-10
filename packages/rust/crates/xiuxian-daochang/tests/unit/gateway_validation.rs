//! Test coverage for xiuxian-daochang behavior.

use axum::http::StatusCode;
use xiuxian_daochang::{MessageRequest, validate_message_request};

#[test]
fn validate_rejects_empty_session_id() {
    let body = MessageRequest {
        session_id: String::new(),
        message: "hi".to_string(),
    };
    let result = validate_message_request(&body);
    assert!(result.is_err());
    let Err(error) = result else {
        panic!("err");
    };
    assert_eq!(error.0, StatusCode::BAD_REQUEST);
}

#[test]
fn validate_rejects_empty_message() {
    let body = MessageRequest {
        session_id: "s1".to_string(),
        message: "  ".to_string(),
    };
    let result = validate_message_request(&body);
    assert!(result.is_err());
    let Err(error) = result else {
        panic!("err");
    };
    assert_eq!(error.0, StatusCode::BAD_REQUEST);
}

#[test]
fn validate_accepts_trimmed_values() {
    let body = MessageRequest {
        session_id: "  s1  ".to_string(),
        message: " hello ".to_string(),
    };
    let validated = match validate_message_request(&body) {
        Ok(values) => values,
        Err(error) => panic!("ok: {error:?}"),
    };
    assert_eq!(validated.session_id, "s1");
    assert_eq!(validated.message, "hello");
}
