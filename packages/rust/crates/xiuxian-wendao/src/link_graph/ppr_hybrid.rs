//! Advanced Hybrid PPR Kernel for Wendao.
//! Implements `HippoRAG` 2 mixed directed graph (P-E topology).

use petgraph::Direction;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::{EdgeRef, NodeIndexable};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;

/// Types of nodes in the `HippoRAG` 2 mixed graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Atomic knowledge entity (Extracted from `OpenIE` triples).
    Entity,
    /// Contextual passage node (Contains text blocks).
    Passage,
}

/// The state of a node within the PPR iteration.
#[derive(Debug, Clone)]
pub struct NodeData {
    /// Unique node identifier.
    pub id: String,
    /// Node semantic type in the mixed graph.
    pub node_type: NodeType,
    /// Current rank value during / after PPR iteration.
    pub rank: f64,
    /// Saliency prior from Hebbian learning.
    pub saliency: f64,
}

/// `HippoRAG` 2 hybrid PPR implementation.
pub struct HybridPprKernel {
    /// Directed weighted graph storage.
    pub graph: StableGraph<NodeData, f32>,
    /// Node id to graph index lookup.
    pub id_to_idx: HashMap<String, petgraph::prelude::NodeIndex>,
}

impl Default for HybridPprKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridPprKernel {
    /// Create an empty hybrid PPR kernel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            id_to_idx: HashMap::new(),
        }
    }

    /// Adds a node if not exists.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn add_node(&mut self, id: &str, node_type: NodeType, saliency: f64) {
        if !self.id_to_idx.contains_key(id) {
            let idx = self.graph.add_node(NodeData {
                id: id.to_string(),
                node_type,
                rank: 0.0,
                saliency,
            });
            self.id_to_idx.insert(id.to_string(), idx);
        }
    }

    /// Adds a weighted edge.
    pub fn add_edge(&mut self, from: &str, to: &str, weight: f32) {
        if let (Some(&f), Some(&t)) = (self.id_to_idx.get(from), self.id_to_idx.get(to)) {
            self.graph.add_edge(f, t, weight);
        }
    }

    /// Run non-uniform PPR with parallel computation and early stopping.
    pub fn run(
        &mut self,
        seeds: &HashMap<String, f64>,
        alpha: f64,
        iterations: usize,
        tol: Option<f64>,
    ) {
        let tolerance = tol.unwrap_or(1e-6);
        let node_count = self.graph.node_bound();
        if node_count == 0 {
            return;
        }

        self.initialize_seed_ranks(seeds);
        let out_weights = self.out_weight_sums(node_count);
        let indices = self.node_indices();
        self.run_power_iterations(seeds, alpha, iterations, tolerance, &out_weights, &indices);
    }

    fn initialize_seed_ranks(&mut self, seeds: &HashMap<String, f64>) {
        for (id, &val) in seeds {
            if let Some(&idx) = self.id_to_idx.get(id) {
                self.graph[idx].rank = val;
            }
        }
    }

    fn out_weight_sums(&self, node_count: usize) -> Vec<f64> {
        let mut out_weights = vec![0.0; node_count];
        for idx in self.graph.node_indices() {
            let total: f32 = self.graph.edges(idx).map(|edge| *edge.weight()).sum();
            out_weights[idx.index()] = f64::from(total);
        }
        out_weights
    }

    fn node_indices(&self) -> Vec<NodeIndex> {
        self.graph.node_indices().collect()
    }

    fn run_power_iterations(
        &mut self,
        seeds: &HashMap<String, f64>,
        alpha: f64,
        iterations: usize,
        tolerance: f64,
        out_weights: &[f64],
        indices: &[NodeIndex],
    ) {
        for _ in 0..iterations {
            let new_ranks = self.next_rank_batch(seeds, alpha, out_weights, indices);
            let diff = self.apply_rank_updates(new_ranks);
            if diff < tolerance {
                break;
            }
        }
    }

    fn next_rank_batch(
        &self,
        seeds: &HashMap<String, f64>,
        alpha: f64,
        out_weights: &[f64],
        indices: &[NodeIndex],
    ) -> Vec<(NodeIndex, f64)> {
        indices
            .par_iter()
            .map(|&idx| (idx, self.next_rank_for_node(idx, seeds, alpha, out_weights)))
            .collect()
    }

    fn next_rank_for_node(
        &self,
        idx: NodeIndex,
        seeds: &HashMap<String, f64>,
        alpha: f64,
        out_weights: &[f64],
    ) -> f64 {
        let incoming_sum = self.incoming_rank_sum(idx, out_weights);
        let teleport_prob = self.teleport_probability(idx, seeds);
        (1.0 - alpha) * incoming_sum + alpha * teleport_prob
    }

    fn incoming_rank_sum(&self, idx: NodeIndex, out_weights: &[f64]) -> f64 {
        self.graph
            .edges_directed(idx, Direction::Incoming)
            .filter_map(|edge| {
                self.incoming_edge_contribution(edge.source(), *edge.weight(), out_weights)
            })
            .sum()
    }

    fn incoming_edge_contribution(
        &self,
        source: NodeIndex,
        weight: f32,
        out_weights: &[f64],
    ) -> Option<f64> {
        let out_weight = out_weights[source.index()];
        (out_weight > 0.0).then(|| self.graph[source].rank * (f64::from(weight) / out_weight))
    }

    fn teleport_probability(&self, idx: NodeIndex, seeds: &HashMap<String, f64>) -> f64 {
        let seed_prob = seeds.get(&self.graph[idx].id).copied().unwrap_or(0.0);
        let current_saliency = self.graph[idx].saliency;
        (seed_prob + current_saliency / 10.0).min(1.0)
    }

    fn apply_rank_updates(&mut self, new_ranks: Vec<(NodeIndex, f64)>) -> f64 {
        new_ranks
            .into_iter()
            .map(|(idx, new_rank)| self.apply_rank_update(idx, new_rank))
            .sum()
    }

    fn apply_rank_update(&mut self, idx: NodeIndex, new_rank: f64) -> f64 {
        let old_rank = self.graph[idx].rank;
        self.graph[idx].rank = new_rank;
        (new_rank - old_rank).abs()
    }

    /// Extract top-K nodes.
    #[must_use]
    pub fn top_k(&self, k: usize) -> Vec<(String, f64)> {
        let mut results: Vec<_> = self
            .graph
            .node_indices()
            .map(|idx| (self.graph[idx].id.clone(), self.graph[idx].rank))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }
}
