//! Swarm possession error map surface for `xiuxian-qianji`.

use crate::contracts::{FlowInstruction, QianjiOutput};

use super::possession_clock::current_unix_millis;
use super::possession_model::{RemoteNodeRequest, RemoteNodeResponse};

/// Converts any mechanism error into a failed remote response.
#[must_use]
/// Identifier boundary: this public compatibility seam accepts externally owned ids.
pub fn map_execution_error_to_response(
    request: &RemoteNodeRequest,
    responder_cluster_id: &str,
    responder_agent_id: &str,
    error: &str,
) -> RemoteNodeResponse {
    RemoteNodeResponse {
        request_id: request.request_id.clone(),
        session_id: request.session_id.clone(),
        node_id: request.node_id.clone(),
        responder_cluster_id: responder_cluster_id.to_string(),
        responder_agent_id: responder_agent_id.to_string(),
        ok: false,
        output: Some(QianjiOutput {
            data: serde_json::json!({
                "remote_possession_error": error,
            }),
            instruction: FlowInstruction::Abort(error.to_string()),
        }),
        error: Some(error.to_string()),
        finished_ms: current_unix_millis(),
    }
}
