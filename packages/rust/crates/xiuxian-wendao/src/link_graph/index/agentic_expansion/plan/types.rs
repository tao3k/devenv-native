//! Planning data shared by agentic expansion helpers.

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(super) struct ExpansionCandidateDoc {
    pub(super) doc_id: String,
    pub(super) rank: f64,
    pub(super) saliency_signal: f64,
    pub(super) tags: HashSet<String>,
}
