use std::path::PathBuf;

use std::time::Instant;

use super::{
    Args, STDIO_SESSION_RESPONSE_KIND, parse_args, parse_stdio_session_request, run,
    stdio_session_response,
};

#[test]
fn parse_args_accepts_persistent_warm_samples() {
    let Ok(args) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--search-root",
            ".",
            "--flight-base-url",
            "http://127.0.0.1:50052",
            "--persistent-warm-samples",
            "3",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("parse persistent warm samples");
    };

    assert_eq!(args.persistent_warm_samples, Some(3));
    assert_eq!(args.flight_timeout_seconds, 30);
    assert!(!args.serve_stdio);
}

#[test]
fn parse_args_rejects_zero_persistent_warm_samples() {
    let Err(error) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--search-root",
            ".",
            "--persistent-warm-samples",
            "0",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("zero samples must fail");
    };

    assert_eq!(error, "--persistent-warm-samples must be greater than zero");
}

#[test]
fn parse_args_accepts_stdio_session_without_intent() {
    let Ok(args) = parse_args(
        [
            "--search-root",
            ".",
            "--flight-base-url",
            "http://127.0.0.1:50052",
            "--serve-stdio",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("stdio session should not require a startup intent");
    };

    assert!(args.serve_stdio);
    assert!(args.intent.is_none());
}

#[test]
fn parse_args_defaults_search_root_to_current_dir() {
    let Ok(args) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--flight-base-url",
            "http://127.0.0.1:50052",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("search root should default to current dir");
    };

    assert!(args.search_root.is_absolute());
    assert_eq!(args.flight_repo, None);
}

#[test]
fn parse_args_rejects_stdio_session_combined_with_stabilization_report() {
    let Err(error) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--search-root",
            ".",
            "--flight-base-url",
            "http://127.0.0.1:50052",
            "--persistent-warm-samples",
            "2",
            "--serve-stdio",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("stdio session cannot also request stabilization report");
    };

    assert_eq!(
        error,
        "--serve-stdio cannot be combined with --persistent-warm-samples"
    );
}

#[test]
fn parse_stdio_session_request_trims_intent_and_keeps_request_id() {
    let Ok(request) = parse_stdio_session_request(
        r#"{"requestId":"req-1","intent":"  find ownership evidence  "}"#,
    ) else {
        panic!("parse stdio request");
    };

    assert_eq!(request.request_id.as_deref(), Some("req-1"));
    assert_eq!(request.intent, "find ownership evidence");
}

#[test]
fn parse_stdio_session_request_rejects_blank_intent() {
    let Err(error) = parse_stdio_session_request(r#"{"requestId":"req-1","intent":"   "}"#) else {
        panic!("blank stdio intent must fail");
    };

    assert_eq!(
        error,
        "SearchStrategyFlow stdio request intent must not be blank"
    );
}

#[test]
fn stdio_session_response_embeds_trace_json() {
    let response = stdio_session_response(
        Some("req-1"),
        Instant::now(),
        Ok(r#"{"validation":{"requiredEvidenceCovered":true}}"#.to_owned()),
    );

    assert_eq!(response["kind"], STDIO_SESSION_RESPONSE_KIND);
    assert_eq!(response["requestId"], "req-1");
    assert_eq!(response["ok"], true);
    assert_eq!(
        response["trace"]["validation"]["requiredEvidenceCovered"],
        true
    );
}

#[tokio::test]
async fn persistent_warm_samples_require_flight_config_before_launch() {
    let args = Args {
        intent: Some("Find ownership and validation evidence".to_owned()),
        search_root: PathBuf::from("."),
        flight_base_url: None,
        flight_repo: None,
        flight_timeout_seconds: 30,
        persistent_warm_samples: Some(1),
        serve_stdio: false,
    };

    let Err(error) = run(args).await else {
        panic!("persistent mode must require Flight config");
    };

    assert_eq!(
        error,
        "--persistent-warm-samples requires --flight-base-url"
    );
}
