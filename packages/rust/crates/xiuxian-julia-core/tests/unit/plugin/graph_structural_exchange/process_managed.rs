use crate::julia_plugin_test_support::common::ResultTestExt;

use super::{
    LIVE_SERVICE_STARTUP_TIMEOUT_SECS, RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV,
    assert_solver_demo_explicit_filter_rows, assert_solver_demo_explicit_rerank_rows,
    assert_solver_demo_multi_route_filter_rows, assert_solver_demo_multi_route_rerank_rows,
    await_live_step, ensure_process_managed_wendaosearch_solver_demo_service,
    graph_structural_explicit_filter_repository, graph_structural_explicit_rerank_repository,
    graph_structural_manifest_repository, prewarm_solver_demo_live_routes,
    process_managed_wendaosearch_solver_demo_base_url, process_managed_wendaosearch_test_enabled,
    wait_for_service_ready_with_attempts,
};

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
#[expect(clippy::large_futures, reason = "process-managed live proof is opt-in")]
async fn fetch_graph_structural_solver_demo_rows_for_repository_against_process_managed_wendaosearch_service()
 {
    if !process_managed_wendaosearch_test_enabled() {
        eprintln!(
            "skipping process-managed WendaoSearch live proof; set {RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV}=1"
        );
        return;
    }

    let _service = ensure_process_managed_wendaosearch_solver_demo_service()
        .await
        .or_panic("ensure process-managed WendaoSearch solver-demo Flight service");
    let base_url = process_managed_wendaosearch_solver_demo_base_url()
        .or_panic("resolve process-managed WendaoSearch solver-demo base URL");
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(&base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&base_url, 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for process-managed WendaoSearch solver-demo Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for process-managed WendaoSearch solver-demo Flight service: {error}")
    });

    prewarm_solver_demo_live_routes(&base_url, 2).await;

    assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository).await;
    assert_solver_demo_explicit_filter_rows(&explicit_filter_repository).await;
    assert_solver_demo_multi_route_rerank_rows(&manifest_repository).await;
    assert_solver_demo_multi_route_filter_rows(&manifest_repository).await;
}
