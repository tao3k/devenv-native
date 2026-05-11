//! Saliency calculation for repository entities using structural topology.

use super::plugin::RepositoryAnalysisOutput;
use petgraph::graph::DiGraph;
use std::collections::HashMap;

/// Compute structural saliency scores for all symbols and modules in the analysis output.
/// Returns a map from entity ID to normalized saliency score (0.0 - 1.0).
pub fn compute_repository_saliency(analysis: &RepositoryAnalysisOutput) -> HashMap<String, f64> {
    let mut graph = DiGraph::<String, ()>::new();
    let mut nodes = HashMap::new();

    // 1. Collect all potential entities from records
    let mut entity_ids = Vec::new();
    for module in &analysis.modules {
        entity_ids.push(module.module_id.to_string());
    }
    for symbol in &analysis.symbols {
        entity_ids.push(symbol.symbol_id.to_string());
    }
    for example in &analysis.examples {
        entity_ids.push(example.example_id.to_string());
    }

    for id in entity_ids {
        nodes
            .entry(id.clone())
            .or_insert_with(|| graph.add_node(id));
    }

    // 2. Add edges from relations
    for relation in &analysis.relations {
        if let (Some(&source), Some(&target)) = (
            nodes.get(relation.source_id.as_str()),
            nodes.get(relation.target_id.as_str()),
        ) {
            // Weight can be adjusted based on RelationKind
            graph.add_edge(source, target, ());
        }
    }

    // 3. Compute simple degree-based saliency (Placeholder for PPR)
    // Core hub nodes (high in-degree) get higher scores.
    let mut scores = HashMap::new();
    let node_count = graph.node_count();
    if node_count == 0 {
        return scores;
    }

    for idx in graph.node_indices() {
        let id = graph[idx].clone();
        let in_degree = graph
            .edges_directed(idx, petgraph::Direction::Incoming)
            .count();
        let out_degree = graph
            .edges_directed(idx, petgraph::Direction::Outgoing)
            .count();

        // Saliency = normalized (in_degree * 2 + out_degree)
        // Hubs (like base types or common solvers) will have many incoming edges (Uses/Implements).
        let raw_score =
            (bounded_usize_to_f64(in_degree) * 2.0) + (bounded_usize_to_f64(out_degree) * 0.5);
        scores.insert(id, raw_score);
    }

    // 4. Normalize scores to 0.0 - 1.0
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
