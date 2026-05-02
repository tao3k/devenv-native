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

        // 1. Initialize ranks from seeds
        for (id, &val) in seeds {
            if let Some(&idx) = self.id_to_idx.get(id) {
                self.graph[idx].rank = val;
            }
        }

        // Pre-compute out-weight sums for all nodes (for fast O(1) division during gather)
        let mut out_weights = vec![0.0; node_count];
        for idx in self.graph.node_indices() {
            let total: f32 = self.graph.edges(idx).map(|e| *e.weight()).sum();
            out_weights[idx.index()] = f64::from(total);
        }

        // Collect indices for parallel iteration
        let indices: Vec<NodeIndex> = self.graph.node_indices().collect();

        // 2. Power iteration
        for _ in 0..iterations {
            // Gather phase (Parallel)
            let new_ranks: Vec<(NodeIndex, f64)> = indices
                .par_iter()
                .map(|&v| {
                    let mut incoming_sum = 0.0;
                    for edge in self.graph.edges_directed(v, Direction::Incoming) {
                        let u = edge.source();
                        let w = f64::from(*edge.weight());
                        let out_w = out_weights[u.index()];
                        if out_w > 0.0 {
                            incoming_sum += self.graph[u].rank * (w / out_w);
                        }
                    }

                    let seed_prob = seeds.get(&self.graph[v].id).copied().unwrap_or(0.0);
                    let current_saliency = self.graph[v].saliency;
                    let teleport_prob = (seed_prob + current_saliency / 10.0).min(1.0);

                    let next_rank = (1.0 - alpha) * incoming_sum + alpha * teleport_prob;
                    (v, next_rank)
                })
                .collect();

            // Convergence check & Apply updates
            let mut diff = 0.0;
            for (idx, new_rank) in new_ranks {
                let old_rank = self.graph[idx].rank;
                diff += (new_rank - old_rank).abs();
                self.graph[idx].rank = new_rank;
            }

            if diff < tolerance {
                break; // Early stopping
            }
        }
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
