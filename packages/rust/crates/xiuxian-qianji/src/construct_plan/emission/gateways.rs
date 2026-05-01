use super::ids::flow_id;
use crate::construct_plan::WorkflowPlan;

pub(super) fn conditional_gateway_sources(plan: &WorkflowPlan) -> Vec<&str> {
    let mut sources = Vec::new();
    for edge in &plan.edges {
        if edge.condition.is_some() && !sources.contains(&edge.from.as_str()) {
            sources.push(edge.from.as_str());
        }
    }
    sources
}

pub(super) fn default_flow_for_source(
    plan: &WorkflowPlan,
    source: &str,
    gateway_source_count: usize,
) -> Option<String> {
    plan.edges
        .iter()
        .position(|edge| edge.from == source && edge.default)
        .map(|edge_index| flow_id(gateway_source_count + edge_index))
}
