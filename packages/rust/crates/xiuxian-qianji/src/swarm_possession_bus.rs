use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;

use super::possession_model::{RemoteNodeRequest, RemoteNodeResponse};

#[path = "swarm/possession/bus/connection.rs"]
mod connection;
#[path = "swarm/possession/bus/keys.rs"]
mod keys;
#[path = "swarm/possession/bus/request.rs"]
mod request;
#[path = "swarm/possession/bus/response.rs"]
mod response;

/// Valkey transport for remote possession request and response orchestration.
pub struct RemotePossessionBus {
    pub(super) redis_url: String,
    pub(super) connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    pub(super) reconnect_lock: Arc<Mutex<()>>,
}

impl RemotePossessionBus {
    /// Creates a new possession bus from Valkey URL.
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Submits a remote request and enqueues it for target role workers.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or Valkey commands fail.
    pub async fn submit_request(
        &self,
        request: &RemoteNodeRequest,
        ttl_seconds: u64,
    ) -> anyhow::Result<()> {
        self.submit_request_impl(request, ttl_seconds).await
    }

    /// Claims one pending request from a role queue.
    ///
    /// Returns `Ok(None)` when no request arrives in `block_timeout`.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or request payload is malformed.
    pub async fn claim_next_for_role(
        &self,
        role_class: &str,
        claimer_id: &str,
        block_timeout: Duration,
    ) -> anyhow::Result<Option<RemoteNodeRequest>> {
        self.claim_next_for_role_impl(role_class, claimer_id, block_timeout)
            .await
    }

    /// Convenience helper: submit request and wait for one response.
    ///
    /// # Errors
    ///
    /// Returns an error when submit or wait operations fail.
    pub async fn request_and_wait(
        &self,
        request: &RemoteNodeRequest,
        ttl_seconds: u64,
        max_wait: Duration,
    ) -> anyhow::Result<Option<RemoteNodeResponse>> {
        self.request_and_wait_impl(request, ttl_seconds, max_wait)
            .await
    }

    /// Publishes one response for a previously submitted request.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or Valkey commands fail.
    pub async fn submit_response(
        &self,
        response: &RemoteNodeResponse,
        ttl_seconds: u64,
    ) -> anyhow::Result<()> {
        self.submit_response_impl(response, ttl_seconds).await
    }

    /// Waits for response of one request.
    ///
    /// Returns `Ok(None)` on timeout.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey or pubsub operations fail.
    pub async fn wait_response(
        &self,
        request_id: &str,
        max_wait: Duration,
    ) -> anyhow::Result<Option<RemoteNodeResponse>> {
        self.wait_response_impl(request_id, max_wait).await
    }
}
