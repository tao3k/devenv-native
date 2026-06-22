//! Caller-safe error mapping coverage for Zhenfa errors.

use xiuxian_zhenfa::{ZhenfaError, ZhenfaTransmuterError, ZhenfaXmlLiteTagName};

#[test]
fn zhenfa_error_exposes_caller_safe_summary() {
    let not_found = ZhenfaError::not_found("wendao.search");
    assert_eq!(
        not_found.caller_safe_message(),
        "requested tool is unavailable in the current runtime"
    );

    let invalid = ZhenfaError::invalid_arguments("bad payload");
    assert_eq!(
        invalid.caller_safe_message(),
        "tool arguments are invalid; adjust parameters and retry"
    );

    let execution = ZhenfaError::execution("i/o timeout");
    assert_eq!(
        execution.caller_safe_message(),
        "tool execution failed in the current environment; retry with a simpler request"
    );
}

#[test]
fn transmuter_error_exposes_caller_safe_summary() {
    let malformed = ZhenfaTransmuterError::UnclosedTag {
        tag: ZhenfaXmlLiteTagName::from("score"),
    };
    assert_eq!(
        malformed.caller_safe_message(),
        "content has malformed XML-Lite structure; ensure all tags are balanced"
    );

    let control_chars = ZhenfaTransmuterError::NullByteDetected;
    assert_eq!(
        control_chars.caller_safe_message(),
        "content contains unsupported control characters; clean the payload and retry"
    );
}
