use std::time::Instant;

use crate::repo_index::perf_support::RepoBootstrapBenchmarkFixture;

#[test]
fn repo_bootstrap_benchmark_fixture_bootstraps_10k_without_snapshot_file() {
    let fixture = RepoBootstrapBenchmarkFixture::synthetic(10_000);
    assert!(!fixture.snapshot_file_exists());

    let start = Instant::now();
    let status_count = fixture.bootstrap_status_count();
    let elapsed = start.elapsed();

    assert_eq!(status_count, fixture.repo_count());
    eprintln!(
        "repo bootstrap benchmark: repos={} statuses={} elapsed_ms={:.3}",
        fixture.repo_count(),
        status_count,
        elapsed.as_secs_f64() * 1_000.0
    );
}
