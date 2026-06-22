use super::{
    LIVE_SERVICE_STARTUP_TIMEOUT_SECS, assert_solver_demo_explicit_filter_rows,
    assert_solver_demo_explicit_rerank_rows, assert_solver_demo_multi_route_filter_rows,
    assert_solver_demo_multi_route_rerank_rows, await_live_step,
    graph_structural_explicit_filter_repository, graph_structural_explicit_rerank_repository,
    graph_structural_manifest_repository, reserve_real_service_port,
    solver_demo_multi_route_base_url_for_port, solver_demo_wendaosearch_service_available,
    spawn_real_wendaosearch_solver_demo_multi_route_service, wait_for_service_ready_with_attempts,
};

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
async fn fetch_graph_structural_solver_demo_rows_for_repository_via_manifest_discovery_against_real_wendaosearch_multi_route_service()
 {
    if !solver_demo_wendaosearch_service_available() {
        eprintln!(
            "skipping real WendaoSearch solver-demo multi-route service test; set WENDAOSEARCH_SOLVER_DEMO_BASE_URL or WENDAOSEARCH_PACKAGE_DIR"
        );
        return;
    }

    let port = reserve_real_service_port();
    let base_url = solver_demo_multi_route_base_url_for_port(port);
    let mut service = spawn_real_wendaosearch_solver_demo_multi_route_service(port);
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(&base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&base_url, 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch solver-demo multi-route Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for real WendaoSearch solver-demo multi-route Flight service: {error}")
    });

    assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository).await;
    assert_solver_demo_explicit_filter_rows(&explicit_filter_repository).await;
    assert_solver_demo_multi_route_rerank_rows(&manifest_repository).await;
    assert_solver_demo_multi_route_filter_rows(&manifest_repository).await;
    service.kill();
}
