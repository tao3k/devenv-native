use super::parse_args;

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
    assert!(args.strategy_flow_service_base_url.is_none());
    assert_eq!(args.strategy_flow_service_timeout_seconds, 30);
    assert!(args.query_understanding_arrow_ipc_path.is_none());
    assert!(args.branch_judgements_arrow_ipc_path.is_none());
    assert!(!args.serve_stdio);
}

#[test]
fn parse_args_accepts_strategy_flow_service_base_url() {
    let Ok(args) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--flight-base-url",
            "http://127.0.0.1:50052",
            "--strategy-flow-service-base-url",
            "http://127.0.0.1:8815",
            "--strategy-flow-service-timeout-seconds",
            "45",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("parse strategy flow service URL");
    };

    assert_eq!(
        args.strategy_flow_service_base_url.as_deref(),
        Some("http://127.0.0.1:8815")
    );
    assert_eq!(args.strategy_flow_service_timeout_seconds, 45);
    assert_eq!(
        args.flight_base_url.as_deref(),
        Some("http://127.0.0.1:50052")
    );
}

#[test]
fn parse_args_accepts_query_understanding_arrow_ipc_path() {
    let Ok(args) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--query-understanding-arrow-ipc",
            "/tmp/query-understanding.arrow",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("parse query-understanding Arrow IPC path");
    };

    assert_eq!(
        args.query_understanding_arrow_ipc_path.as_deref(),
        Some(std::path::Path::new("/tmp/query-understanding.arrow"))
    );
}

#[test]
fn parse_args_accepts_branch_judgements_arrow_ipc_path() {
    let Ok(args) = parse_args(
        [
            "--intent",
            "Find ownership and validation evidence",
            "--branch-judgements-arrow-ipc",
            "/tmp/branch-judgements.arrow",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("parse branch judgements Arrow IPC path");
    };

    assert_eq!(
        args.branch_judgements_arrow_ipc_path.as_deref(),
        Some(std::path::Path::new("/tmp/branch-judgements.arrow"))
    );
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
        ["--search-root", ".", "--serve-stdio"]
            .into_iter()
            .map(str::to_owned),
    ) else {
        panic!("stdio session should not require a startup intent or Flight");
    };

    assert!(args.serve_stdio);
    assert!(args.intent.is_none());
    assert!(args.flight_base_url.is_none());
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
fn parse_args_rejects_stdio_session_combined_with_strategy_flow_service() {
    let Err(error) = parse_args(
        [
            "--search-root",
            ".",
            "--strategy-flow-service-base-url",
            "http://127.0.0.1:8815",
            "--serve-stdio",
        ]
        .into_iter()
        .map(str::to_owned),
    ) else {
        panic!("stdio session cannot also request production service mode");
    };

    assert_eq!(
        error,
        "--serve-stdio cannot be combined with --strategy-flow-service-base-url"
    );
}
