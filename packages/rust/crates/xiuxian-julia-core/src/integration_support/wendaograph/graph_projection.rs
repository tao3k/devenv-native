//! Graph-core projections for Julia-owned `WendaoGraph.jl` control facts.

use std::collections::BTreeSet;
use std::fmt::Debug;

use serde::Serialize;
use xiuxian_graph_core::{GraphEdge, GraphNode, GraphNodeId, GraphProjection};
use xiuxian_polyglot_orchestrator::{
    JuliaAcceleratorDiagnostics, JuliaReadinessEvidence, JuliaSchedulePlan,
};

/// Projects `WendaoGraph.jl` readiness evidence into the shared graph model.
///
/// The returned projection is descriptive only. It does not schedule Julia
/// work, open Flight routes, probe a host, or mutate admission state.
#[must_use]
pub fn wendaograph_readiness_graph_projection(
    readiness: &JuliaReadinessEvidence,
) -> GraphProjection {
    let mut builder = ProjectionBuilder::default();
    let profile = builder.node(
        profile_node_id(readiness.profile_id.as_str()),
        format!("profile: {}", readiness.profile_id),
    );
    let capability = builder.node(
        typed_node_id("capability", &enum_token(&readiness.capability)),
        format!("capability: {}", enum_token(&readiness.capability)),
    );
    let readiness_state = builder.node(
        typed_node_id("readiness", &enum_token(&readiness.readiness_state())),
        format!("readiness: {}", enum_token(&readiness.readiness_state())),
    );
    let pressure = builder.node(
        typed_node_id("pressure", &enum_token(&readiness.pressure_level())),
        format!("pressure: {}", enum_token(&readiness.pressure_level())),
    );
    let fallback = builder.node(
        typed_node_id("fallback", &readiness.fallback_available.to_string()),
        format!("fallback: {}", readiness.fallback_available),
    );
    let admission = builder.node(
        typed_node_id("admission", readiness.profile_id.as_str()),
        format!(
            "admission: active={} queue={} max={}",
            readiness.active_in_flight,
            readiness.queue_depth,
            readiness
                .max_in_flight
                .map_or_else(|| "unknown".to_string(), |value| value.to_string())
        ),
    );

    builder.edge(&profile, &capability, "owns");
    builder.edge(&profile, &readiness_state, "reports");
    builder.edge(&profile, &pressure, "reports");
    builder.edge(&profile, &fallback, "fallback");
    builder.edge(&profile, &admission, "admission");

    for diagnostic in &readiness.accelerator_diagnostics {
        add_accelerator(&mut builder, &profile, diagnostic);
    }

    builder.finish()
}

/// Projects a Julia schedule plan into the shared graph model.
///
/// The schedule graph preserves the Rust-owned scheduling decision while
/// keeping the graph primitive generic.
#[must_use]
pub fn wendaograph_schedule_graph_projection(plan: &JuliaSchedulePlan) -> GraphProjection {
    let mut builder = ProjectionBuilder::default();
    add_schedule_plan(&mut builder, plan);
    builder.finish()
}

/// Projects readiness evidence and an optional schedule plan into one graph.
///
/// Use this when an agent or operator needs a compact reasoning-tree view for
/// one `WendaoGraph.jl` profile.
#[must_use]
pub fn wendaograph_profile_graph_projection(
    readiness: &JuliaReadinessEvidence,
    schedule_plan: Option<&JuliaSchedulePlan>,
) -> GraphProjection {
    let mut builder = ProjectionBuilder::default();
    let profile = add_readiness(&mut builder, readiness);
    if let Some(plan) = schedule_plan {
        let schedule = add_schedule_plan(&mut builder, plan);
        builder.edge(&profile, &schedule, "planned_by");
    }
    builder.finish()
}

fn add_readiness(
    builder: &mut ProjectionBuilder,
    readiness: &JuliaReadinessEvidence,
) -> GraphNodeId {
    let profile = builder.node(
        profile_node_id(readiness.profile_id.as_str()),
        format!("profile: {}", readiness.profile_id),
    );
    let capability = builder.node(
        typed_node_id("capability", &enum_token(&readiness.capability)),
        format!("capability: {}", enum_token(&readiness.capability)),
    );
    let readiness_state = builder.node(
        typed_node_id("readiness", &enum_token(&readiness.readiness_state())),
        format!("readiness: {}", enum_token(&readiness.readiness_state())),
    );
    let pressure = builder.node(
        typed_node_id("pressure", &enum_token(&readiness.pressure_level())),
        format!("pressure: {}", enum_token(&readiness.pressure_level())),
    );
    let fallback = builder.node(
        typed_node_id("fallback", &readiness.fallback_available.to_string()),
        format!("fallback: {}", readiness.fallback_available),
    );
    let admission = builder.node(
        typed_node_id("admission", readiness.profile_id.as_str()),
        format!(
            "admission: active={} queue={} max={}",
            readiness.active_in_flight,
            readiness.queue_depth,
            readiness
                .max_in_flight
                .map_or_else(|| "unknown".to_string(), |value| value.to_string())
        ),
    );

    builder.edge(&profile, &capability, "owns");
    builder.edge(&profile, &readiness_state, "reports");
    builder.edge(&profile, &pressure, "reports");
    builder.edge(&profile, &fallback, "fallback");
    builder.edge(&profile, &admission, "admission");

    for diagnostic in &readiness.accelerator_diagnostics {
        add_accelerator(builder, &profile, diagnostic);
    }

    profile
}

fn add_schedule_plan(builder: &mut ProjectionBuilder, plan: &JuliaSchedulePlan) -> GraphNodeId {
    let profile = builder.node(
        profile_node_id(plan.profile_id.as_str()),
        format!("profile: {}", plan.profile_id.as_str()),
    );
    let schedule = builder.node(
        typed_node_id("schedule", plan.profile_id.as_str()),
        format!(
            "schedule: batch={} confidence={}",
            plan.selected_batch_size, plan.confidence_score
        ),
    );
    let action = builder.node(
        typed_node_id("action", &enum_token(&plan.action)),
        format!("action: {}", enum_token(&plan.action)),
    );
    let reason = builder.node(
        typed_node_id("reason", &enum_token(&plan.reason)),
        format!("reason: {}", enum_token(&plan.reason)),
    );
    let readiness = builder.node(
        typed_node_id("readiness", &enum_token(&plan.readiness)),
        format!("readiness: {}", enum_token(&plan.readiness)),
    );
    let pressure = builder.node(
        typed_node_id("pressure", &enum_token(&plan.pressure)),
        format!("pressure: {}", enum_token(&plan.pressure)),
    );
    let latency = builder.node(
        typed_node_id("latency_ms", &plan.predicted_latency_ms.get().to_string()),
        format!("latency_ms: {}", plan.predicted_latency_ms.get()),
    );

    builder.edge(&profile, &schedule, "plans");
    builder.edge(&schedule, &action, "action");
    builder.edge(&schedule, &reason, "reason");
    builder.edge(&schedule, &readiness, "uses");
    builder.edge(&schedule, &pressure, "uses");
    builder.edge(&schedule, &latency, "predicts");

    if let Some(batchability_key) = &plan.batchability_key {
        let batchability = builder.node(
            typed_node_id("batchability", batchability_key.as_str()),
            format!("batchability: {}", batchability_key.as_str()),
        );
        builder.edge(&schedule, &batchability, "batches");
    }

    schedule
}

fn add_accelerator(
    builder: &mut ProjectionBuilder,
    profile: &GraphNodeId,
    diagnostic: &JuliaAcceleratorDiagnostics,
) {
    let accelerator = builder.node(
        typed_node_id("accelerator", diagnostic.backend.as_str()),
        format!(
            "accelerator: {} loaded={} functional={}",
            diagnostic.backend, diagnostic.loaded, diagnostic.functional
        ),
    );
    builder.edge(profile, &accelerator, "accelerator");
}

fn profile_node_id(profile_id: &str) -> String {
    typed_node_id("profile", profile_id)
}

fn typed_node_id(kind: &str, value: &str) -> String {
    format!("{}_{}", safe_id(kind), safe_id(value))
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

#[derive(Default)]
struct ProjectionBuilder {
    projection: GraphProjection,
    node_ids: BTreeSet<String>,
}

impl ProjectionBuilder {
    fn node(&mut self, id: impl Into<String>, label: impl Into<String>) -> GraphNodeId {
        let id = id.into();
        if self.node_ids.insert(id.clone()) {
            self.projection
                .push_node(GraphNode::new(id.clone(), label.into()));
        }
        GraphNodeId::new(id)
    }

    fn edge(&mut self, source: &GraphNodeId, target: &GraphNodeId, label: impl Into<String>) {
        self.projection
            .push_edge(GraphEdge::new(source.clone(), target.clone()).with_label(label));
    }

    fn finish(self) -> GraphProjection {
        debug_assert!(self.projection.validate().is_ok());
        self.projection
    }
}
