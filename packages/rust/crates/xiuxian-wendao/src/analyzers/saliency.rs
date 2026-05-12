//! Saliency calculation for repository entities using structural topology.

use super::plugin::RepositoryAnalysisOutput;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Compute structural saliency scores for all symbols and modules in the analysis output.
/// Returns a map from entity ID to normalized saliency score (0.0 - 1.0).
pub fn compute_repository_saliency(analysis: &RepositoryAnalysisOutput) -> HashMap<String, f64> {
    let mut graph = DiGraph::<String, ()>::new();
    let mut nodes = HashMap::new();

    add_analysis_entities(analysis, &mut graph, &mut nodes);
    add_relation_edges(analysis, &mut graph, &nodes);
    normalize_scores(degree_saliency_scores(&graph))
}

fn add_analysis_entities(
    analysis: &RepositoryAnalysisOutput,
    graph: &mut DiGraph<String, ()>,
    nodes: &mut HashMap<String, NodeIndex>,
) {
    repository_entity_ids(analysis).for_each(|id| {
        nodes
            .entry(id.clone())
            .or_insert_with(|| graph.add_node(id));
    });
}

fn repository_entity_ids(analysis: &RepositoryAnalysisOutput) -> impl Iterator<Item = String> + '_ {
    analysis
        .modules
        .iter()
        .map(|module| module.module_id.to_string())
        .chain(
            analysis
                .symbols
                .iter()
                .map(|symbol| symbol.symbol_id.to_string()),
        )
        .chain(
            analysis
                .examples
                .iter()
                .map(|example| example.example_id.to_string()),
        )
}

fn add_relation_edges(
    analysis: &RepositoryAnalysisOutput,
    graph: &mut DiGraph<String, ()>,
    nodes: &HashMap<String, NodeIndex>,
) {
    analysis
        .relations
        .iter()
        .filter_map(|relation| {
            nodes
                .get(relation.source_id.as_str())
                .zip(nodes.get(relation.target_id.as_str()))
        })
        .for_each(|(&source, &target)| {
            graph.add_edge(source, target, ());
        });
}

fn degree_saliency_scores(graph: &DiGraph<String, ()>) -> HashMap<String, f64> {
    graph
        .node_indices()
        .map(|idx| (graph[idx].clone(), degree_saliency_score(graph, idx)))
        .collect()
}

fn degree_saliency_score(graph: &DiGraph<String, ()>, idx: NodeIndex) -> f64 {
    let in_degree = graph
        .edges_directed(idx, petgraph::Direction::Incoming)
        .count();
    let out_degree = graph
        .edges_directed(idx, petgraph::Direction::Outgoing)
        .count();
    (bounded_usize_to_f64(in_degree) * 2.0) + (bounded_usize_to_f64(out_degree) * 0.5)
}

fn normalize_scores(mut scores: HashMap<String, f64>) -> HashMap<String, f64> {
    let max_score = scores.values().copied().fold(0.0, f64::max);
    if max_score > 0.0 {
        for score in scores.values_mut() {
            *score /= max_score;
        }
    }

    scores
}

fn bounded_usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}

#[cfg(test)]
#[path = "../../tests/unit/analyzers/saliency.rs"]
mod tests;
