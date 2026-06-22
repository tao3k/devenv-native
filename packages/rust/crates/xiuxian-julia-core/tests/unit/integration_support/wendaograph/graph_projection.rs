use super::{
    wendaograph_profile_graph_projection, wendaograph_readiness_graph_projection,
    wendaograph_schedule_graph_projection,
};
use serde::Serialize;
use std::fmt::Debug;
use xiuxian_graph_core::CompactMermaidGraph;
use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, JuliaAcceleratorDiagnostics, JuliaAcceleratorState,
    JuliaAcceleratorStateInput, JuliaComputeTaskShape, JuliaProfileSchedulingFacts,
    JuliaReadinessEvidence, JuliaRuntimeStats, JuliaTaskComplexityClass, LaneCapability,
    ManifestReadinessState, WarmupState,
};

use crate::polyglot::{
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, wendao_graph_gnn_reasoning_schedule_plan,
};

#[test]
fn readiness_projection_uses_graph_core_nodes_for_wendaograph_profile_facts() {
    let readiness = readiness_evidence();

    let projection = wendaograph_readiness_graph_projection(&readiness);

    let validation = projection.validate();
    assert!(
        validation.is_ok(),
        "invalid graph projection: {validation:?}"
    );
    let profile_id = typed_node_id("profile", WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);
    assert!(projection.nodes().iter().any(|node| {
        node.id().as_str() == profile_id
            && node.label() == format!("profile: {WENDAO_GRAPH_GNN_REASONING_PROFILE_ID}")
    }));
    assert!(projection.nodes().iter().any(|node| {
        node.id().as_str() == "accelerator_metal"
            && node
                .label()
                .contains("accelerator: metal loaded=true functional=true")
    }));
    assert!(projection.edges().iter().any(|edge| {
        edge.source().as_str() == profile_id
            && edge.target().as_str() == "readiness_ready"
            && edge.label() == Some("reports")
    }));
    let diagram = CompactMermaidGraph::new().render(&projection);
    assert!(diagram.is_ok(), "invalid Mermaid graph: {diagram:?}");
}

#[test]
fn schedule_projection_preserves_action_reason_and_batchability_edges() {
    let plan = wendao_graph_gnn_reasoning_schedule_plan(
        JuliaComputeTaskShape::new()
            .with_rows(64)
            .with_graph_size(256, 1024)
            .with_feature_columns(8)
            .with_batchability_key("gnn:graph-page-window")
            .with_complexity(JuliaTaskComplexityClass::Heavy),
        JuliaProfileSchedulingFacts {
            max_in_flight: Some(4),
            runtime_stats: JuliaRuntimeStats::new()
                .with_warmup(WarmupState::Ready)
                .with_benchmark(BenchmarkState::WithinThreshold)
                .with_latency_ms(Some(18), Some(33)),
            fallback_available: true,
            deadline_ms: None,
            target_latency_ms: Some(50),
        },
    );

    let projection = wendaograph_schedule_graph_projection(&plan);

    let validation = projection.validate();
    assert!(
        validation.is_ok(),
        "invalid schedule projection: {validation:?}"
    );
    let schedule_id = typed_node_id("schedule", plan.profile_id.as_str());
    let action_id = typed_node_id("action", &enum_token(&plan.action));
    let reason_id = typed_node_id("reason", &enum_token(&plan.reason));
    assert!(projection.nodes().iter().any(|node| {
        node.id().as_str() == "batchability_gnn_graph_page_window"
            && node.label() == "batchability: gnn:graph-page-window"
    }));
    assert!(projection.edges().iter().any(|edge| {
        edge.source().as_str() == schedule_id
            && edge.target().as_str() == action_id
            && edge.label() == Some("action")
    }));
    assert!(projection.edges().iter().any(|edge| {
        edge.source().as_str() == schedule_id
            && edge.target().as_str() == reason_id
            && edge.label() == Some("reason")
    }));
    let diagram = CompactMermaidGraph::new().render(&projection);
    assert!(diagram.is_ok(), "invalid Mermaid graph: {diagram:?}");
}

#[test]
fn profile_projection_links_readiness_to_schedule_plan() {
    let readiness = readiness_evidence();
    let plan = wendao_graph_gnn_reasoning_schedule_plan(
        JuliaComputeTaskShape::new().with_complexity(JuliaTaskComplexityClass::Heavy),
        JuliaProfileSchedulingFacts::new(
            JuliaRuntimeStats::new()
                .with_warmup(WarmupState::Ready)
                .with_benchmark(BenchmarkState::WithinThreshold),
        ),
    );

    let projection = wendaograph_profile_graph_projection(&readiness, Some(&plan));

    let validation = projection.validate();
    assert!(
        validation.is_ok(),
        "invalid profile projection: {validation:?}"
    );
    let profile_id = typed_node_id("profile", WENDAO_GRAPH_GNN_REASONING_PROFILE_ID);
    let schedule_id = typed_node_id("schedule", plan.profile_id.as_str());
    assert!(projection.edges().iter().any(|edge| {
        edge.source().as_str() == profile_id
            && edge.target().as_str() == schedule_id
            && edge.label() == Some("planned_by")
    }));
    let diagram = CompactMermaidGraph::new().render(&projection);
    assert!(diagram.is_ok(), "invalid Mermaid graph: {diagram:?}");
}

fn readiness_evidence() -> JuliaReadinessEvidence {
    JuliaReadinessEvidence::new(
        LaneCapability::GraphEvidenceCompute,
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
    )
    .with_schema_version("wendaograph-gnn-v1")
    .with_route_validation(ContractValidationState::Valid)
    .with_schema_validation(ContractValidationState::Valid)
    .with_manifest_readiness(ManifestReadinessState::Ready)
    .with_warmup(WarmupState::Ready)
    .with_benchmark(BenchmarkState::WithinThreshold)
    .with_admission_window(Some(4), 1, 0)
    .with_fallback_available(true)
    .with_accelerator_diagnostics([JuliaAcceleratorDiagnostics::new(
        "metal",
        JuliaAcceleratorState::new(JuliaAcceleratorStateInput {
            loaded: true,
            functional: true,
        }),
    )
    .with_observed_output_count(Some(8))])
}

fn enum_token<T>(value: &T) -> String
where
    T: Debug + Serialize,
{
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| format!("{value:?}"))
}

fn typed_node_id(kind: &str, value: &str) -> String {
    format!("{}_{}", safe_id(kind), safe_id(value))
}

fn safe_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}
