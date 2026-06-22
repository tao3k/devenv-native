use super::anchor_batch::{QuantumAnchorBatchRow, QuantumAnchorBatchView};
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::models::QuantumContext;

#[derive(Debug, Clone)]
pub(super) struct ResolvedQuantumAnchor {
    pub(super) batch_row: usize,
    pub(super) batch_anchor_id: String,
    pub(super) anchor_id: String,
    pub(super) doc_id: String,
    pub(super) path: String,
    pub(super) semantic_path: Vec<String>,
    pub(super) trace_label: Option<String>,
    pub(super) vector_score: f64,
}

impl LinkGraphIndex {
    pub(super) fn resolve_quantum_anchors(
        &self,
        batch: &QuantumAnchorBatchView<'_>,
    ) -> Vec<ResolvedQuantumAnchor> {
        batch
            .rows()
            .filter_map(|row| self.resolve_quantum_anchor_row(row))
            .collect()
    }

    fn resolve_quantum_anchor_row(
        &self,
        row: QuantumAnchorBatchRow<'_>,
    ) -> Option<ResolvedQuantumAnchor> {
        let anchor_id = row.anchor_id.trim();
        if anchor_id.is_empty() {
            return None;
        }
        let doc_id = self.quantum_anchor_doc_id(anchor_id)?;
        let path = self
            .get_doc(doc_id.as_str())
            .map_or_else(|| doc_id.clone(), |doc| doc.path.clone());
        let semantic_path = self
            .extract_lineage(anchor_id)
            .unwrap_or_else(|| vec![anchor_id.to_string()]);
        let trace_label = QuantumContext::trace_label_from_semantic_path(&semantic_path);

        Some(ResolvedQuantumAnchor {
            batch_row: row.row,
            batch_anchor_id: row.anchor_id.to_string(),
            anchor_id: anchor_id.to_string(),
            doc_id,
            path,
            semantic_path,
            trace_label,
            vector_score: row.vector_score.clamp(0.0, 1.0),
        })
    }

    pub(super) fn quantum_anchor_doc_id(&self, anchor_id: &str) -> Option<String> {
        let trimmed = anchor_id.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some((doc_id, _)) = trimmed.split_once('#')
            && self.docs_by_id.contains_key(doc_id)
        {
            return Some(doc_id.to_string());
        }
        self.resolve_doc_id_pub(trimmed).map(str::to_string)
    }
}
