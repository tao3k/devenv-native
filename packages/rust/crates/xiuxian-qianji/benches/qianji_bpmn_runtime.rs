//! Criterion benchmarks for Qianji BPMN runtime execution.

use std::sync::Arc;

use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use serde_json::json;
use xiuxian_qianji::{QianjiBpmnExecutionDriver, QianjiBpmnExecutionRequest, QianjiBpmnHostBridge};
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    ProcessKey,
};

const BPMN_RUNTIME_BENCH_ITERATIONS: u64 = 1;

fn bench_bpmn_runtime_driver(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|error| panic!("build benchmark runtime: {error}"));
    let package = Arc::new(minimal_bpmn_package());
    let host = QianjiBpmnHostBridge::default();

    let mut group = c.benchmark_group("qianji_bpmn_runtime");
    group.throughput(Throughput::Elements(BPMN_RUNTIME_BENCH_ITERATIONS));
    group.bench_function("driver_start_to_end", |bench| {
        bench.iter_batched(
            || {
                QianjiBpmnExecutionRequest::new(
                    "review",
                    "bench_runtime_start_to_end",
                    Some(json!({ "risk": "low" })),
                    17,
                )
            },
            |request| {
                let driver = QianjiBpmnExecutionDriver::new(Arc::clone(&package), None);
                let report = runtime
                    .block_on(driver.run_until_stable(&request, &host))
                    .unwrap_or_else(|error| panic!("run BPMN runtime benchmark: {error}"));
                assert_eq!(report.outcome, BpmnAdvanceOutcome::Completed);
                black_box(report)
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn minimal_bpmn_package() -> BpmnPackage {
    BpmnPackage::new(
        "pkg_runtime_bench",
        vec![BpmnProcessSpec::new(
            ProcessKey::new("pkg_runtime_bench", "review", "digest_runtime_bench"),
            vec![
                BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
                BpmnNodeSpec::new(1, "end", BpmnNodeKind::EndEvent),
            ],
            vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
            Vec::new(),
        )],
    )
}

criterion_group!(benches, bench_bpmn_runtime_driver);
criterion_main!(benches);
