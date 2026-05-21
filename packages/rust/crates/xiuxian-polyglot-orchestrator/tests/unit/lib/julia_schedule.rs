use crate::{
    BenchmarkState, ContractValidationState, JuliaComputeTaskShape, JuliaRuntimeStats,
    JuliaScheduleAction, JuliaScheduleReason, JuliaSchedulingInput, JuliaTaskComplexityClass,
    LaneCapability, ManifestReadinessState, WarmupState,
};

#[test]
fn warm_healthy_julia_dispatches_heavy_graph_search() {
    let plan = base_input(
        LaneCapability::GraphSearchCompute,
        "wendaosearch_structural_rerank",
        heavy_graph_shape(),
    )
    .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Dispatch);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaAdvantage);
    assert!(plan.confidence_score > 0);
    assert!(plan.selected_batch_size > 1);
}

#[test]
fn cold_julia_queues_when_deadline_allows_waiting() {
    let readiness = readiness(LaneCapability::MemoryProfileCompute, "memory_gate_score")
        .with_warmup(WarmupState::Cold);
    let stats = JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Cold)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_latency_ms(Some(20), Some(80));

    let plan = JuliaSchedulingInput::new(readiness, memory_shape(), stats)
        .with_fallback_available(true)
        .with_deadline_ms(Some(2_000))
        .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Queue);
    assert_eq!(plan.reason, JuliaScheduleReason::JuliaWarming);
}

#[test]
fn cold_julia_falls_back_when_deadline_is_tight() {
    let readiness = readiness(LaneCapability::MemoryProfileCompute, "episodic_recall")
        .with_warmup(WarmupState::Cold);
    let stats = JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Cold)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_latency_ms(Some(20), Some(120));

    let plan = JuliaSchedulingInput::new(readiness, memory_shape(), stats)
        .with_fallback_available(true)
        .with_deadline_ms(Some(50))
        .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Fallback);
    assert_eq!(plan.reason, JuliaScheduleReason::DeadlineTooTight);
}

#[test]
fn invalid_schema_rejects_without_fallback() {
    let readiness = readiness(
        LaneCapability::GraphSearchCompute,
        "wendaosearch_legacy_rerank",
    )
    .with_schema_validation(ContractValidationState::Invalid);

    let plan = JuliaSchedulingInput::new(
        readiness,
        heavy_graph_shape(),
        stable_stats().with_benchmark(BenchmarkState::WithinThreshold),
    )
    .with_fallback_available(false)
    .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Reject);
    assert_eq!(plan.reason, JuliaScheduleReason::ContractInvalid);
}

#[test]
fn benchmark_failure_falls_back_instead_of_dispatching() {
    let readiness = readiness(
        LaneCapability::GraphSearchCompute,
        "wendaosearch_constraint_filter",
    )
    .with_benchmark(BenchmarkState::Failed);
    let stats = stable_stats().with_benchmark(BenchmarkState::Failed);

    let plan = JuliaSchedulingInput::new(readiness, heavy_graph_shape(), stats)
        .with_fallback_available(true)
        .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Fallback);
    assert_eq!(plan.reason, JuliaScheduleReason::BenchmarkFailed);
}

#[test]
fn high_queue_pressure_queues_when_deadline_allows() {
    let readiness = readiness(
        LaneCapability::GraphEvidenceCompute,
        "wendao_graph_link_evidence",
    )
    .with_admission_window(Some(4), 2, 4);
    let stats = stable_stats().with_queue(4, 2);

    let plan = JuliaSchedulingInput::new(readiness, heavy_graph_shape(), stats)
        .with_fallback_available(true)
        .with_deadline_ms(Some(2_000))
        .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Queue);
    assert_eq!(plan.reason, JuliaScheduleReason::QueuePressure);
}

#[test]
fn high_queue_pressure_falls_back_when_deadline_is_tight() {
    let readiness = readiness(
        LaneCapability::GraphEvidenceCompute,
        "wendao_graph_link_evidence",
    )
    .with_admission_window(Some(4), 2, 4);
    let stats = stable_stats()
        .with_queue(4, 2)
        .with_latency_ms(Some(30), Some(140));

    let plan = JuliaSchedulingInput::new(readiness, heavy_graph_shape(), stats)
        .with_fallback_available(true)
        .with_deadline_ms(Some(100))
        .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Fallback);
    assert_eq!(plan.reason, JuliaScheduleReason::DeadlineTooTight);
}

#[test]
fn larger_graph_shapes_increase_julia_preference() {
    let small = base_input(
        LaneCapability::GraphEvidenceCompute,
        "wendao_graph_link_evidence",
        JuliaComputeTaskShape::new()
            .with_graph_size(12, 20)
            .with_complexity(JuliaTaskComplexityClass::Simple),
    )
    .with_fallback_available(true)
    .plan();
    let large = base_input(
        LaneCapability::GraphEvidenceCompute,
        "wendao_graph_link_evidence",
        heavy_graph_shape(),
    )
    .with_fallback_available(true)
    .plan();

    assert!(large.benefit_score > small.benefit_score);
    assert_eq!(small.action, JuliaScheduleAction::Fallback);
    assert_eq!(large.action, JuliaScheduleAction::Dispatch);
}

#[test]
fn transfer_cost_can_suppress_julia_dispatch() {
    let plan = base_input(
        LaneCapability::MemoryProfileCompute,
        "memory_calibration",
        memory_shape().with_byte_size(64 * 1024 * 1024),
    )
    .with_fallback_available(true)
    .plan();

    assert_eq!(plan.action, JuliaScheduleAction::Fallback);
    assert_eq!(plan.reason, JuliaScheduleReason::CostExceedsBenefit);
    assert!(plan.cost_score > plan.benefit_score);
}

#[test]
fn error_rate_and_p95_latency_penalize_confidence() {
    let stable = base_input(
        LaneCapability::GraphSearchCompute,
        "wendaosearch_structural_rerank",
        heavy_graph_shape(),
    )
    .plan();
    let unstable = JuliaSchedulingInput::new(
        readiness(
            LaneCapability::GraphSearchCompute,
            "wendaosearch_structural_rerank",
        ),
        heavy_graph_shape(),
        stable_stats()
            .with_error_rate_basis_points(800)
            .with_latency_ms(Some(100), Some(800)),
    )
    .with_target_latency_ms(Some(150))
    .plan();

    assert!(unstable.confidence_score < stable.confidence_score);
}

#[test]
fn batch_compatible_tasks_produce_larger_batch_size() {
    let unbatched = base_input(
        LaneCapability::GraphSearchCompute,
        "wendaosearch_structural_rerank",
        heavy_graph_shape_without_batch_key().with_rows(32),
    )
    .plan();
    let batched = base_input(
        LaneCapability::GraphSearchCompute,
        "wendaosearch_structural_rerank",
        heavy_graph_shape()
            .with_rows(32)
            .with_batchability_key("wendaosearch:rerank:v1"),
    )
    .plan();

    assert_eq!(unbatched.selected_batch_size, 1);
    assert!(batched.selected_batch_size > unbatched.selected_batch_size);
    assert_eq!(
        batched.batchability_key.as_deref(),
        Some("wendaosearch:rerank:v1")
    );
}

fn base_input(
    capability: LaneCapability,
    profile_id: &str,
    shape: JuliaComputeTaskShape,
) -> JuliaSchedulingInput {
    JuliaSchedulingInput::new(readiness(capability, profile_id), shape, stable_stats())
        .with_target_latency_ms(Some(250))
}

fn readiness(capability: LaneCapability, profile_id: &str) -> crate::JuliaReadinessEvidence {
    crate::JuliaReadinessEvidence::new(capability, profile_id)
        .with_schema_version("v1")
        .with_route_validation(ContractValidationState::Valid)
        .with_schema_validation(ContractValidationState::Valid)
        .with_manifest_readiness(ManifestReadinessState::Ready)
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_admission_window(Some(8), 0, 0)
        .with_fallback_available(true)
}

fn stable_stats() -> JuliaRuntimeStats {
    JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::WithinThreshold)
        .with_latency_ms(Some(30), Some(90))
}

fn heavy_graph_shape() -> JuliaComputeTaskShape {
    heavy_graph_shape_without_batch_key().with_batchability_key("graph:v1")
}

fn heavy_graph_shape_without_batch_key() -> JuliaComputeTaskShape {
    JuliaComputeTaskShape::new()
        .with_rows(24)
        .with_graph_size(1_500, 12_000)
        .with_feature_columns(18)
        .with_byte_size(2 * 1024 * 1024)
        .with_complexity(JuliaTaskComplexityClass::Heavy)
}

fn memory_shape() -> JuliaComputeTaskShape {
    JuliaComputeTaskShape::new()
        .with_rows(1)
        .with_feature_columns(6)
        .with_byte_size(64 * 1024)
        .with_complexity(JuliaTaskComplexityClass::Simple)
}
