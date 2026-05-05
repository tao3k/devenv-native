//! Criterion benchmarks for local `wendao get` projection performance.

use criterion::{BatchSize, Criterion, Throughput, black_box};
use tempfile::TempDir;
use xiuxian_wendao_client::perf_support::{
    GET_BENCH_DOC_COUNT, benchmark_local_page_index, write_local_get_benchmark_fixture,
};

fn build_fixture() -> TempDir {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
    write_local_get_benchmark_fixture(temp.path());
    temp
}

fn bench_local_page_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("wendao_client_get");
    group.throughput(Throughput::Elements(GET_BENCH_DOC_COUNT as u64));
    group.bench_function("local_page_index_512_docs", |bench| {
        bench.iter_batched(
            build_fixture,
            |fixture| {
                let summary = benchmark_local_page_index(fixture.path())
                    .unwrap_or_else(|error| panic!("benchmark local page index: {error}"));
                black_box(summary)
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    bench_local_page_index(&mut criterion);
    criterion.final_summary();
}
