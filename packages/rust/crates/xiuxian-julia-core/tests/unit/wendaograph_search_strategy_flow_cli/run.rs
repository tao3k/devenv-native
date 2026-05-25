use std::path::PathBuf;

use super::{Args, run};

#[tokio::test]
async fn persistent_warm_samples_require_flight_config_before_launch() {
    let args = Args {
        intent: Some("Find ownership and validation evidence".to_owned()),
        search_root: PathBuf::from("."),
        flight_base_url: None,
        flight_repo: None,
        flight_timeout_seconds: 30,
        strategy_flow_service_base_url: None,
        strategy_flow_service_timeout_seconds: 30,
        persistent_warm_samples: Some(1),
        serve_stdio: false,
        query_understanding_arrow_ipc_path: None,
        branch_judgements_arrow_ipc_path: None,
    };

    let Err(error) = run(args).await else {
        panic!("persistent mode must require Flight config");
    };

    assert_eq!(
        error,
        "--persistent-warm-samples requires --flight-base-url"
    );
}
