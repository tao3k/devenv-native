use super::{
    LIVE_SERVICE_STARTUP_TIMEOUT_SECS, assert_demo_multi_route_rerank_rows, await_live_step,
    graph_structural_explicit_rerank_repository, graph_structural_manifest_repository,
    local_wendaosearch_package_available, reserve_real_service_port,
    spawn_real_wendaosearch_demo_multi_route_service, wait_for_service_ready_with_attempts,
};

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
async fn fetch_graph_structural_demo_rerank_rows_for_repository_against_real_wendaosearch_multi_route_service()
 {
    if !local_wendaosearch_package_available() {
        eprintln!(
            "skipping real WendaoSearch demo multi-route service test; set WENDAOSEARCH_PACKAGE_DIR"
        );
        return;
    }

    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaosearch_demo_multi_route_service(port);
    let explicit_repository = graph_structural_explicit_rerank_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&format!("http://127.0.0.1:{port}"), 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch multi-route Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for real WendaoSearch multi-route Flight service: {error}")
    });

    assert_demo_multi_route_rerank_rows(&explicit_repository).await;
    assert_demo_multi_route_rerank_rows(&manifest_repository).await;
    service.kill();
}
