use super::{
    RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV, WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS_ENV,
    WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES_ENV, live_perf_env_usize,
    measure_solver_demo_live_perf_profile, print_live_perf_summary,
};

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
#[expect(
    clippy::large_futures,
    reason = "live perf proof is opt-in and route-complete"
)]
async fn graph_structural_live_perf_against_real_wendaosearch_solver_demo_multi_route_service() {
    if std::env::var_os(RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoSearch graph-structural live perf profile; set {RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV}=1"
        );
        return;
    }

    let run_count = live_perf_env_usize(WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS_ENV, 1);
    let warm_sample_count =
        live_perf_env_usize(WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES_ENV, 3);
    let mut measurements = Vec::new();
    for run_index in 0..run_count {
        measurements.extend(
            measure_solver_demo_live_perf_profile(
                "cold",
                false,
                None,
                false,
                run_index,
                warm_sample_count,
            )
            .await,
        );
        measurements.extend(
            measure_solver_demo_live_perf_profile(
                "prewarmed",
                true,
                Some("none"),
                false,
                run_index,
                warm_sample_count,
            )
            .await,
        );
        measurements.extend(
            measure_solver_demo_live_perf_profile(
                "prewarmed-flight-probe",
                true,
                Some("none"),
                true,
                run_index,
                warm_sample_count,
            )
            .await,
        );
    }
    print_live_perf_summary(&measurements);
}
