//! `link_graph::models::records::diagnostics` owns Wendao models records diagnostics behavior.

use crate::link_graph::models::query::LinkGraphPprSubgraphMode;
use serde::{Deserialize, Serialize};

/// Debug/observability diagnostics for related PPR retrieval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinkGraphRelatedPprDiagnostics {
    /// Effective restart probability for PPR walk.
    pub alpha: f64,
    /// Effective max iteration cap.
    pub max_iter: usize,
    /// Effective convergence tolerance.
    pub tol: f64,
    /// Iterations actually executed.
    pub iteration_count: usize,
    /// Final L1 residual at convergence stop.
    pub final_residual: f64,
    /// Candidate count in bounded horizon (excluding seed notes).
    pub candidate_count: usize,
    /// Candidate cap applied by runtime guard.
    pub candidate_cap: usize,
    /// Whether bounded horizon candidates were trimmed by candidate cap guard.
    pub candidate_capped: bool,
    /// Graph node count used by the PPR computation.
    pub graph_node_count: usize,
    /// Number of subgraph kernels executed before score fusion.
    pub subgraph_count: usize,
    /// Largest partition node count used by the subgraph kernels.
    pub partition_max_node_count: usize,
    /// Smallest partition node count used by the subgraph kernels.
    pub partition_min_node_count: usize,
    /// Average partition node count used by the subgraph kernels.
    pub partition_avg_node_count: f64,
    /// End-to-end related PPR compute duration in milliseconds.
    pub total_duration_ms: f64,
    /// Subgraph partition build duration in milliseconds.
    pub partition_duration_ms: f64,
    /// PPR kernel execution duration in milliseconds.
    pub kernel_duration_ms: f64,
    /// Score fusion duration in milliseconds.
    pub fusion_duration_ms: f64,
    /// Effective subgraph mode used by runtime.
    pub subgraph_mode: LinkGraphPprSubgraphMode,
    /// Whether computation restricted to bounded horizon subgraph.
    pub horizon_restricted: bool,
    /// Runtime wall-clock budget applied to one related PPR computation.
    pub time_budget_ms: f64,
    /// Whether runtime budget/guard truncated kernel execution.
    pub timed_out: bool,
}
