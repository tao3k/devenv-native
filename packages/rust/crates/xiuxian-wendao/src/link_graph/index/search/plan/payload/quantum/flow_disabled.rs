use crate::link_graph::LinkGraphPlannedSearchPayload;
use crate::link_graph::index::LinkGraphIndex;

impl LinkGraphIndex {
    pub(crate) async fn enrich_planned_payload_with_quantum_contexts(
        &self,
        _payload: &mut LinkGraphPlannedSearchPayload,
        _query_vector: &[f32],
    ) {
        std::future::ready(()).await;
    }
}
