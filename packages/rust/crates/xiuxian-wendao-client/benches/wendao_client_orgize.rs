//! Criterion benchmarks for Org agent read-model performance.

use criterion::{BatchSize, Criterion, Throughput, black_box};
use tempfile::TempDir;
use xiuxian_wendao_client::orgize_perf_support::{
    ORGIZE_AGENT_BENCH_TASK_COUNT, benchmark_agent_org_cached_active_query,
    benchmark_agent_org_read_model, write_agent_org_benchmark_fixture,
};

const CACHED_ACTIVE_LIMIT: usize = 20;

fn build_fixture() -> TempDir {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
    let _org_dir = write_agent_org_benchmark_fixture(temp.path());
    temp
}

fn build_cached_fixture() -> TempDir {
    let fixture = build_fixture();
    let summary = benchmark_agent_org_read_model(fixture.path())
        .unwrap_or_else(|error| panic!("prime agent Org read-model benchmark cache: {error}"));
    assert_eq!(summary.cached_rows, ORGIZE_AGENT_BENCH_TASK_COUNT);
    fixture
}

fn bench_agent_org_read_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("wendao_client_orgize");
    group.throughput(Throughput::Elements(ORGIZE_AGENT_BENCH_TASK_COUNT as u64));
    group.bench_function("agent_read_model_refresh_query_1024_tasks", |bench| {
        bench.iter_batched(
            build_fixture,
            |fixture| {
                let summary = benchmark_agent_org_read_model(fixture.path())
                    .unwrap_or_else(|error| panic!("benchmark agent Org read model: {error}"));
                assert_eq!(summary.cached_rows, ORGIZE_AGENT_BENCH_TASK_COUNT);
                black_box(summary)
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("cached_active_recovery_limit_20", |bench| {
        bench.iter_batched(
            build_cached_fixture,
            |fixture| {
                let summary =
                    benchmark_agent_org_cached_active_query(fixture.path(), CACHED_ACTIVE_LIMIT)
                        .unwrap_or_else(|error| {
                            panic!("benchmark cached active recovery query: {error}")
                        });
                assert_eq!(summary.shown_rows, CACHED_ACTIVE_LIMIT);
                black_box(summary)
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    bench_agent_org_read_model(&mut criterion);
    criterion.final_summary();
}
