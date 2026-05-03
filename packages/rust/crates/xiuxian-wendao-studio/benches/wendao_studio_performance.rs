//! Criterion benchmarks for Studio gateway performance trend analysis.

use std::sync::Arc;

use criterion::{BatchSize, Criterion, black_box};
use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao_studio::studio::{GatewayState, StudioState, studio_router};

fn bench_studio_state_bootstrap(criterion: &mut Criterion) {
    criterion.bench_function("studio_state_bootstrap", |bencher| {
        bencher.iter(|| black_box(StudioState::new()));
    });
}

fn bench_studio_router_creation(criterion: &mut Criterion) {
    criterion.bench_function("studio_router_creation", |bencher| {
        bencher.iter_batched(
            || {
                Arc::new(GatewayState::new(
                    None,
                    None,
                    Arc::new(PluginRegistry::new()),
                ))
            },
            |state| black_box(studio_router(state)),
            BatchSize::SmallInput,
        );
    });
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    bench_studio_state_bootstrap(&mut criterion);
    bench_studio_router_creation(&mut criterion);
    criterion.final_summary();
}
