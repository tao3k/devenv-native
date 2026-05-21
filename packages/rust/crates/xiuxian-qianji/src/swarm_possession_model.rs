//! Swarm possession model surface for `xiuxian-qianji`.

use serde::{Deserialize, Serialize};

use super::possession_clock::current_unix_millis;
use crate::contracts::QianjiOutput;

/// One remote node-execution request published by a source cluster.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct RemoteNodeRequest {
    /// Unique request id.
    pub request_id: String,
    /// Shared workflow session id.
    pub session_id: String,
    /// Target node id to execute remotely.
    pub node_id: String,
    /// Target role class that can execute this node.
    pub role_class: String,
    /// Requester cluster identifier.
    pub requester_cluster_id: String,
    /// Requester agent identifier.
    pub requester_agent_id: String,
    /// Serialized context snapshot at delegation point.
    pub context: serde_json::Value,
    /// Request timestamp in unix milliseconds.
    pub created_ms: u64,
}

impl RemoteNodeRequest {
    /// Creates a request with generated id and timestamp.
    #[must_use]
    /// Positional boundary: this compatibility API keeps the established public call shape.
    pub fn new(
        session_id: impl Into<String>,
        node_id: impl Into<String>,
        role_class: impl Into<String>,
        requester_cluster_id: impl Into<String>,
        requester_agent_id: impl Into<String>,
        context: serde_json::Value,
    ) -> Self {
        let created_ms = current_unix_millis();
        let random: u64 = rand::random();
        let request_id = format!("remote_possession_{created_ms}_{random:x}");
        Self {
            request_id,
            session_id: session_id.into(),
            node_id: node_id.into(),
            role_class: role_class.into(),
            requester_cluster_id: requester_cluster_id.into(),
            requester_agent_id: requester_agent_id.into(),
            context,
            created_ms,
        }
    }
}

/// One remote execution response returned by a responder cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct RemoteNodeResponse {
    /// Request id to correlate with caller.
    pub request_id: String,
    /// Session id carried from request.
    pub session_id: String,
    /// Target node id that was executed.
    pub node_id: String,
    /// Responder cluster id.
    pub responder_cluster_id: String,
    /// Responder agent id.
    pub responder_agent_id: String,
    /// Whether execution was successful.
    pub ok: bool,
    /// Output payload when successful.
    pub output: Option<QianjiOutput>,
    /// Error message when failed.
    pub error: Option<String>,
    /// Response timestamp in unix milliseconds.
    pub finished_ms: u64,
}

impl RemoteNodeResponse {
    /// Constructs a successful response.
    #[must_use]
    pub fn success(
        request: &RemoteNodeRequest,
        responder_cluster_id: impl Into<String>,
        responder_agent_id: impl Into<String>,
        output: QianjiOutput,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            node_id: request.node_id.clone(),
            responder_cluster_id: responder_cluster_id.into(),
            responder_agent_id: responder_agent_id.into(),
            ok: true,
            output: Some(output),
            error: None,
            finished_ms: current_unix_millis(),
        }
    }

    /// Constructs a failed response.
    #[must_use]
    pub fn failure(
        request: &RemoteNodeRequest,
        responder_cluster_id: impl Into<String>,
        responder_agent_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            node_id: request.node_id.clone(),
            responder_cluster_id: responder_cluster_id.into(),
            responder_agent_id: responder_agent_id.into(),
            ok: false,
            output: None,
            error: Some(error.into()),
            finished_ms: current_unix_millis(),
        }
    }
}
