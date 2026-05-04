//! Criterion benchmarks for Wendao Flight/gRPC transport helpers.

use criterion::{Criterion, black_box};
use xiuxian_wendao_server::transport::{
    SEARCH_SYMBOLS_ROUTE, WENDAO_SCHEMA_VERSION_HEADER, flight_descriptor_path,
    normalize_flight_route,
};

fn bench_flight_descriptor_path(criterion: &mut Criterion) {
    criterion.bench_function("transport_flight_descriptor_path", |bencher| {
        bencher.iter(|| black_box(flight_descriptor_path(black_box(SEARCH_SYMBOLS_ROUTE))));
    });
}

fn bench_normalize_flight_route(criterion: &mut Criterion) {
    criterion.bench_function("transport_normalize_flight_route", |bencher| {
        bencher.iter(|| black_box(normalize_flight_route(black_box("/search/symbols/"))));
    });
}

fn bench_schema_header_access(criterion: &mut Criterion) {
    criterion.bench_function("transport_schema_header_access", |bencher| {
        bencher.iter(|| black_box(WENDAO_SCHEMA_VERSION_HEADER));
    });
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    bench_flight_descriptor_path(&mut criterion);
    bench_normalize_flight_route(&mut criterion);
    bench_schema_header_access(&mut criterion);
    criterion.final_summary();
}
