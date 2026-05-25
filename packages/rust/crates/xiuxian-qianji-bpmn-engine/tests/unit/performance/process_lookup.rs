use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use xiuxian_qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    ProcessKey, create_instance,
};

#[test]
#[ignore = "performance probe"]
fn performance_probe_process_lookup_cache_compares_linear_scan_vs_index_access() {
    let process_count = 20_000_u32;
    let iterations = 200_000_u32;
    let target_process_id = format!("proc_{}", process_count - 1);
    let package = Arc::new(BpmnPackage::new(
        "pkg_perf_lookup",
        (0..process_count)
            .map(|index| start_end_process(&format!("proc_{index}")))
            .collect(),
    ));
    let state = create_instance(
        Arc::clone(&package),
        &target_process_id,
        BpmnInstanceInit::new("wf_perf_lookup", json!({}), 1),
    )
    .must("target process should exist");

    let linear_start = Instant::now();
    let mut linear_nodes = 0_usize;
    for _ in 0..iterations {
        let process = package
            .find_process(&target_process_id)
            .must("linear lookup should find the process");
        linear_nodes += process.nodes.len();
    }
    let linear_elapsed = linear_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed_nodes = 0_usize;
    for _ in 0..iterations {
        let process = &package.processes[state.process_index as usize];
        indexed_nodes += process.nodes.len();
    }
    let indexed_elapsed = indexed_start.elapsed();

    assert_eq!(linear_nodes, indexed_nodes);
    eprintln!(
        "performance_probe process_lookup processes={} iterations={} linear_ms={:.3} indexed_ms={:.3}",
        process_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
#[ignore = "performance probe"]
fn performance_probe_wait_process_lookup_compares_linear_vs_indexed() {
    let process_count = 20_000_u32;
    let iterations = 200_000_u32;
    let target_process_index = process_count - 1;
    let target_process_id = format!("proc_{target_process_index}");
    let package = BpmnPackage::new(
        "pkg_perf_wait_lookup",
        (0..process_count)
            .map(|index| start_end_process(&format!("proc_{index}")))
            .collect(),
    );

    let linear_start = Instant::now();
    let mut linear_nodes = 0_usize;
    for _ in 0..iterations {
        let process = package
            .find_process(&target_process_id)
            .must("linear wait lookup should find the process");
        linear_nodes += process.nodes.len();
    }
    let linear_elapsed = linear_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed_nodes = 0_usize;
    for _ in 0..iterations {
        let process =
            indexed_wait_process_lookup(&package, &target_process_id, target_process_index)
                .must("indexed wait lookup should find the process");
        indexed_nodes += process.nodes.len();
    }
    let indexed_elapsed = indexed_start.elapsed();

    let fallback = indexed_wait_process_lookup(&package, &target_process_id, 0)
        .must("stale wait process index should fall back to process id lookup");
    assert_eq!(fallback.key.process_id.as_ref(), target_process_id);
    assert_eq!(linear_nodes, indexed_nodes);
    std::hint::black_box((linear_nodes, indexed_nodes));
    eprintln!(
        "performance_probe wait_process_lookup processes={} iterations={} linear_ms={:.3} indexed_ms={:.3}",
        process_count,
        iterations,
        linear_elapsed.as_secs_f64() * 1000.0,
        indexed_elapsed.as_secs_f64() * 1000.0
    );
}

fn start_end_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_perf_lookup",
            process_id,
            format!("digest_{process_id}"),
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        Vec::new(),
    )
}

fn indexed_wait_process_lookup<'a>(
    package: &'a BpmnPackage,
    process_id: &str,
    process_index: u32,
) -> Option<&'a BpmnProcessSpec> {
    package
        .processes
        .get(process_index as usize)
        .filter(|process| process.key.process_id.as_ref() == process_id)
        .or_else(|| package.find_process(process_id))
}
