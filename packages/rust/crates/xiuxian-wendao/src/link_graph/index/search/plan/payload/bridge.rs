use super::types::PlannedPayloadSearchRequest;
use crate::link_graph::LinkGraphPlannedSearchPayload;
use crate::link_graph::index::LinkGraphIndex;

impl LinkGraphIndex {
    pub(super) fn search_planned_payload_with_agentic_runtime_bridge_with_query_vector(
        &self,
        request: PlannedPayloadSearchRequest,
    ) -> LinkGraphPlannedSearchPayload {
        let fallback_index = self.clone();
        let fallback_request = request.clone();

        let worker_index = self.clone();
        let worker_request = request;
        let worker_name = "wendao-semantic-ignition-bridge".to_string();
        match std::thread::Builder::new()
            .name(worker_name)
            .spawn(move || {
                let request = worker_request;
                let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                else {
                    return worker_index.search_planned_payload_with_agentic_core_sync(request);
                };
                runtime
                    .block_on(worker_index.search_planned_payload_with_agentic_core_async(request))
            }) {
            Ok(handle) => match handle.join() {
                Ok(payload) => payload,
                Err(_) => {
                    fallback_index.search_planned_payload_with_agentic_core_sync(fallback_request)
                }
            },
            Err(_) => {
                fallback_index.search_planned_payload_with_agentic_core_sync(fallback_request)
            }
        }
    }
}
