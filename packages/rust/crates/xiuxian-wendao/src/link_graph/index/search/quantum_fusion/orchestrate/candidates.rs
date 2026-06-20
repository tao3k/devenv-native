use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::index::search::quantum_fusion::anchor_batch::QuantumAnchorBatchView;
use crate::link_graph::index::search::quantum_fusion::scored_context::quantum_contexts_from_scored_batch;
use crate::link_graph::index::search::quantum_fusion::scoring::{
    BatchQuantumScorer, topology_score_from_ranked,
};
use crate::link_graph::index::search::quantum_fusion::topology_expansion::{
    collect_related_clusters, quantum_related_limit_for_doc, resolve_quantum_related_limits,
};
use crate::link_graph::models::{QuantumAnchorHit, QuantumContext, QuantumFusionOptions};
use arrow::record_batch::RecordBatch;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(crate) struct QuantumContextCandidate {
    pub(crate) anchor_id: String,
    pub(crate) semantic_path: Vec<String>,
    pub(crate) related_clusters: Vec<String>,
    pub(crate) vector_score: f64,
    pub(crate) topology_score: f64,
}

impl LinkGraphIndex {
    /// Build traceable quantum-fusion contexts from Arrow anchor batches.
    ///
    /// # Errors
    ///
    /// Returns [`QuantumContextBuildError`] when the anchor batch cannot be
    /// decoded or the Arrow-native scorer cannot produce the fused saliency
    /// output column.
    pub fn quantum_contexts_from_anchor_batch(
        &self,
        batch: &RecordBatch,
        id_col: &str,
        score_col: &str,
        options: &QuantumFusionOptions,
    ) -> Result<Vec<QuantumContext>, crate::QuantumContextBuildError> {
        let options = options.normalized();
        let view = QuantumAnchorBatchView::new(batch, id_col, score_col)?;
        let resolved = self.resolve_quantum_anchors(&view);
        if resolved.is_empty() {
            return Ok(Vec::new());
        }
        let candidates = self.expand_quantum_context_candidates(&resolved, &options);
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let topology_scores = candidates
            .iter()
            .map(|candidate| (candidate.batch_anchor_id.clone(), candidate.topology_score))
            .collect::<HashMap<_, _>>();
        let scorer = BatchQuantumScorer::new(&options);
        let scored_batch = scorer
            .score_batch(view.batch(), &topology_scores, id_col, score_col)
            .map_err(crate::QuantumContextBuildError::ScoreScoringBatch)?;

        quantum_contexts_from_scored_batch(candidates, &scored_batch)
    }

    /// Build traceable quantum-fusion contexts from precomputed semantic anchors.
    ///
    /// # Errors
    ///
    /// Returns [`QuantumContextBuildError`] when the prepared scoring batch
    /// cannot be constructed or the Arrow-native scorer cannot produce the
    /// fused saliency output column.
    pub fn quantum_contexts_from_anchors(
        &self,
        anchors: &[QuantumAnchorHit],
        options: &QuantumFusionOptions,
    ) -> Result<Vec<QuantumContext>, crate::QuantumContextBuildError> {
        let options = options.normalized();
        let candidates = self.quantum_context_candidates(anchors, &options);
        let saliency_scores = crate::link_graph::index::search::quantum_fusion::orchestrate::scoring::score_quantum_context_candidates(&candidates, &options)?;

        let mut contexts = candidates
            .into_iter()
            .zip(saliency_scores)
            .map(|(candidate, saliency_score)| {
                let anchor_id = candidate.anchor_id;
                let doc_id = anchor_id
                    .split_once('#')
                    .map_or(anchor_id.as_str(), |(doc_id, _)| doc_id)
                    .to_string();
                let path = self
                    .get_doc(doc_id.as_str())
                    .map_or_else(|| doc_id.clone(), |doc| doc.path.clone());
                let trace_label =
                    QuantumContext::trace_label_from_semantic_path(&candidate.semantic_path);
                QuantumContext {
                    anchor_id,
                    doc_id,
                    path,
                    semantic_path: candidate.semantic_path,
                    trace_label,
                    related_clusters: candidate.related_clusters,
                    saliency_score,
                    vector_score: candidate.vector_score,
                    topology_score: candidate.topology_score,
                }
            })
            .collect::<Vec<_>>();

        contexts.sort_by(|left, right| {
            right
                .saliency_score
                .partial_cmp(&left.saliency_score)
                .unwrap_or(Ordering::Equal)
                .then(left.anchor_id.cmp(&right.anchor_id))
        });
        Ok(contexts)
    }

    pub(crate) fn quantum_context_candidates(
        &self,
        anchors: &[QuantumAnchorHit],
        options: &QuantumFusionOptions,
    ) -> Vec<QuantumContextCandidate> {
        let related_limits = resolve_quantum_related_limits(
            &anchors
                .iter()
                .filter_map(|anchor| self.quantum_anchor_doc_id(anchor.anchor_id.as_str()))
                .collect::<Vec<_>>(),
            options.related_limit,
        );
        let mut candidates = Vec::new();

        for anchor in anchors {
            let anchor_id = anchor.anchor_id.trim();
            if anchor_id.is_empty() {
                continue;
            }
            let Some(seed_doc_id) = self.quantum_anchor_doc_id(anchor_id) else {
                continue;
            };
            let semantic_path = self
                .extract_lineage(anchor_id)
                .unwrap_or_else(|| vec![anchor_id.to_string()]);

            let effective_related_limit = quantum_related_limit_for_doc(
                seed_doc_id.as_str(),
                &related_limits,
                options.related_limit,
            );
            let ranked = self.quantum_related_ranked_doc_ids(
                seed_doc_id.as_str(),
                anchor.vector_score,
                options,
            );
            let related_clusters = collect_related_clusters(&ranked, effective_related_limit);
            let topology_score = topology_score_from_ranked(&ranked, effective_related_limit);

            candidates.push(QuantumContextCandidate {
                anchor_id: anchor_id.to_string(),
                semantic_path,
                related_clusters,
                vector_score: anchor.vector_score.clamp(0.0, 1.0),
                topology_score,
            });
        }

        candidates
    }

    pub(crate) fn quantum_related_ranked_doc_ids(
        &self,
        seed_doc_id: &str,
        _vector_score: f64,
        options: &QuantumFusionOptions,
    ) -> Vec<(String, usize, f64)> {
        let seeds = HashSet::from([seed_doc_id.to_string()]);
        self.related_ppr_ranked_doc_ids(&seeds, options.max_distance, options.ppr.as_ref())
    }
}
