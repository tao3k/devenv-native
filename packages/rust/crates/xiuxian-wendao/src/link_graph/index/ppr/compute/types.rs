//! Related PPR telemetry shared by compute helpers.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(super) struct RelatedPprKernelTelemetry {
    pub(super) fused_scores_by_doc_id: HashMap<String, f64>,
    pub(super) iteration_count: usize,
    pub(super) final_residual: f64,
    pub(super) subgraph_count: usize,
    pub(super) partition_sizes: Vec<usize>,
    pub(super) partition_duration_ms: f64,
    pub(super) kernel_duration_ms: f64,
    pub(super) fusion_duration_ms: f64,
    pub(super) timed_out: bool,
}
