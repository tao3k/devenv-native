use std::env;
use std::io;
use std::process::{Command, Stdio};

use super::fixtures::{branch_judgement_arrow_ipc, fixture_candidate_batch};
use crate::integration_support::service_runtime::{
    JuliaServiceGuard, reserve_service_port, wait_for_service_ready_with_attempts,
};
use crate::integration_support::wendaograph::wendaograph_julia_project;
use crate::integration_support::{
    SearchStrategyFlowServiceFlightBindingOptions, SearchStrategyFlowServiceRequestOptions,
    roundtrip_search_strategy_flow_frontier_with_service,
    roundtrip_search_strategy_flow_frontier_with_service_request,
};

const RUN_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_TEST";

#[tokio::test]
async fn search_strategy_flow_service_live_loopback_uses_real_wendaograph_arrow_flight()
-> io::Result<()> {
    if env::var_os(RUN_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_ENV).is_none() {
        eprintln!(
            "skipping live WendaoGraph SearchStrategyFlow service loopback; set {RUN_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_ENV}=1"
        );
        return Ok(());
    }

    let candidate_batch = fixture_candidate_batch();
    let project = wendaograph_julia_project().map_err(io::Error::other)?;
    let runner = project
        .join("scripts")
        .join("run_search_strategy_flow_service.jl");
    if !runner.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing WendaoGraph SearchStrategyFlow runner `{}`",
                runner.display()
            ),
        ));
    }

    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut guard = JuliaServiceGuard::new(
        Command::new("julia")
            .arg(format!("--project={}", project.display()))
            .arg(&runner)
            .arg("--host=127.0.0.1")
            .arg(format!("--port={port}"))
            .arg("--flow-id=search-strategy-flow-rust-live")
            .arg("--max-active-requests=4")
            .arg("--request-capacity=4")
            .arg("--response-capacity=4")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?,
    );

    wait_for_service_ready_with_attempts(&base_url, 300)
        .await
        .map_err(io::Error::other)?;
    let options = SearchStrategyFlowServiceFlightBindingOptions::new(base_url)
        .map_err(io::Error::other)?
        .with_max_in_flight_requests(1);
    let roundtrip =
        roundtrip_search_strategy_flow_frontier_with_service(&candidate_batch, options.clone())
            .await
            .map_err(io::Error::other)?;

    assert_eq!(roundtrip.flight_route, "/wendao/graph/search_strategy_flow");
    assert_eq!(roundtrip.rows.len(), 1);
    assert_eq!(roundtrip.rows[0].flow_id, "search-strategy-flow-rust-live");
    assert_eq!(
        roundtrip.rows[0].candidate_id,
        "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary"
    );
    assert!(roundtrip.rows[0].selected);

    let judged_request_options = SearchStrategyFlowServiceRequestOptions::default()
        .with_branch_judgements_arrow_ipc_stream(branch_judgement_arrow_ipc(
            "search-strategy-flow-rust-live",
            "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
        ));
    let judged_roundtrip = roundtrip_search_strategy_flow_frontier_with_service_request(
        &candidate_batch,
        judged_request_options,
        options,
    )
    .await
    .map_err(io::Error::other)?;

    assert_eq!(judged_roundtrip.rows.len(), 1);
    assert!(!judged_roundtrip.rows[0].selected);

    guard.kill();
    Ok(())
}
