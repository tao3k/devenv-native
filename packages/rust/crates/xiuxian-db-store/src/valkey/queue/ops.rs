use crate::valkey::{
    ValkeyClient,
    error::{ValkeyStoreError, validate_positive_ttl},
    queue::{
        ValkeyLeaseId, ValkeyLeaseScriptResult, ValkeyQueueEntryId, ValkeyQueueKeys,
        ValkeyStructuredClaimRequest, ValkeyStructuredQueueEntry, ValkeyStructuredQueueLease,
        ValkeyStructuredQueueLeaseRef, ValkeyWorkerId,
    },
};

use super::model::{lease_not_owned_error, priority_score};
use super::scripts::{
    CLAIM_QUEUE_LUA, RECLAIM_EXPIRED_LEASE_LUA, RELEASE_LEASE_LUA, RENEW_LEASE_LUA,
};

/// Structured Valkey queue.
#[derive(Clone)]
pub struct ValkeyStructuredQueue {
    client: ValkeyClient,
    keys: ValkeyQueueKeys,
}

impl ValkeyStructuredQueue {
    /// Creates a queue from a shared client and key set.
    #[must_use]
    pub fn new(client: ValkeyClient, keys: ValkeyQueueKeys) -> Self {
        Self { client, keys }
    }

    /// Borrows queue keys.
    #[must_use]
    pub const fn keys(&self) -> &ValkeyQueueKeys {
        &self.keys
    }

    /// Enqueues a structured payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn enqueue(
        &self,
        entry: ValkeyStructuredQueueEntry<'_>,
    ) -> Result<(), ValkeyStoreError> {
        let payload_key = self.keys.payload_key(entry.entry_id());
        let lease_key = self.keys.lease_key(entry.entry_id());
        let priority_score = priority_score(entry.priority());
        let _: i64 = self
            .client
            .run_command("valkey_structured_queue_enqueue", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&payload_key)
                    .arg("payload")
                    .arg(entry.payload())
                    .arg("priority_score")
                    .arg(priority_score.to_string())
                    .arg("not_before_ms")
                    .arg(entry.not_before_ms().to_string());
                for (field, value) in entry.fields() {
                    command.arg(*field).arg(*value);
                }
                command
            })
            .await?;
        let lease_exists: bool = self
            .client
            .run_command("valkey_structured_queue_lease_exists", || {
                let mut command = redis::cmd("EXISTS");
                command.arg(&lease_key);
                command
            })
            .await?;
        if lease_exists {
            return Ok(());
        }
        let _: i64 = self
            .client
            .run_command("valkey_structured_queue_zadd", || {
                let mut command = redis::cmd("ZADD");
                command
                    .arg(self.keys.pending_key())
                    .arg(priority_score)
                    .arg(entry.entry_id().as_str());
                command
            })
            .await?;
        Ok(())
    }

    /// Claims a structured payload using explicit field filters.
    ///
    /// # Errors
    ///
    /// Returns an error when TTL validation or the Valkey command fails.
    pub async fn claim(
        &self,
        request: ValkeyStructuredClaimRequest<'_>,
    ) -> Result<Option<ValkeyStructuredQueueLease>, ValkeyStoreError> {
        validate_positive_ttl("lease_ttl_ms", request.lease_ttl_ms())?;
        let expires_at_ms = request.now_ms().saturating_add(request.lease_ttl_ms());
        let payload: Option<Vec<String>> = self
            .client
            .run_command("valkey_structured_queue_claim", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(CLAIM_QUEUE_LUA)
                    .arg(2)
                    .arg(self.keys.pending_key())
                    .arg(self.keys.lease_deadlines_key())
                    .arg(&self.keys.payload_prefix)
                    .arg(&self.keys.lease_prefix)
                    .arg(request.now_ms().to_string())
                    .arg(request.lease_id().as_str())
                    .arg(request.worker_id().as_str())
                    .arg(request.now_ms().to_string())
                    .arg(expires_at_ms.to_string())
                    .arg(request.lease_ttl_ms().to_string())
                    .arg(request.filters().len().to_string());
                for filter in request.filters() {
                    command.arg(filter.field()).arg(filter.value());
                }
                command
            })
            .await?;
        let Some(mut payload) = payload else {
            return Ok(None);
        };
        if payload.len() != 2 {
            return Err(ValkeyStoreError::Storage {
                operation: "decode_valkey_structured_queue_claim",
                message: format!("expected 2 return values, received {}", payload.len()),
            });
        }
        let entry_id = ValkeyQueueEntryId::new(payload.remove(0))?;
        let payload = payload.remove(0);
        Ok(Some(ValkeyStructuredQueueLease::new(
            request.lease_id().clone(),
            entry_id,
            payload,
            request.worker_id().clone(),
            request.now_ms(),
            expires_at_ms,
        )))
    }

    /// Renews a lease.
    ///
    /// # Errors
    ///
    /// Returns an error when TTL validation or the Valkey command fails.
    pub async fn renew(
        &self,
        lease: ValkeyStructuredQueueLeaseRef<'_>,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> Result<ValkeyLeaseScriptResult, ValkeyStoreError> {
        validate_positive_ttl("lease_ttl_ms", lease_ttl_ms)?;
        let expires_at_ms = now_ms.saturating_add(lease_ttl_ms);
        let lease_key = self.keys.lease_key(lease.entry_id());
        let result: i64 = self
            .client
            .run_command("valkey_structured_queue_renew", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RENEW_LEASE_LUA)
                    .arg(2)
                    .arg(&lease_key)
                    .arg(self.keys.lease_deadlines_key())
                    .arg(lease.lease_id().as_str())
                    .arg(lease.worker_id().as_str())
                    .arg(expires_at_ms.to_string())
                    .arg(lease_ttl_ms.to_string())
                    .arg(lease.entry_id().as_str());
                command
            })
            .await?;
        lease_script_result(result, lease)
    }

    /// Releases a lease.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn release(
        &self,
        lease: ValkeyStructuredQueueLeaseRef<'_>,
    ) -> Result<ValkeyLeaseScriptResult, ValkeyStoreError> {
        let lease_key = self.keys.lease_key(lease.entry_id());
        let result: i64 = self
            .client
            .run_command("valkey_structured_queue_release", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RELEASE_LEASE_LUA)
                    .arg(2)
                    .arg(&lease_key)
                    .arg(self.keys.lease_deadlines_key())
                    .arg(lease.lease_id().as_str())
                    .arg(lease.worker_id().as_str())
                    .arg(lease.entry_id().as_str());
                command
            })
            .await?;
        lease_script_result(result, lease)
    }

    /// Reclaims an expired lease into the pending queue.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn reclaim_expired(
        &self,
        lease: ValkeyStructuredQueueLeaseRef<'_>,
        now_ms: u64,
    ) -> Result<ValkeyLeaseScriptResult, ValkeyStoreError> {
        let lease_key = self.keys.lease_key(lease.entry_id());
        let payload_key = self.keys.payload_key(lease.entry_id());
        let result: i64 = self
            .client
            .run_command("valkey_structured_queue_reclaim_expired", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RECLAIM_EXPIRED_LEASE_LUA)
                    .arg(4)
                    .arg(self.keys.pending_key())
                    .arg(&lease_key)
                    .arg(self.keys.lease_deadlines_key())
                    .arg(&payload_key)
                    .arg(lease.lease_id().as_str())
                    .arg(lease.worker_id().as_str())
                    .arg(lease.entry_id().as_str())
                    .arg(now_ms.to_string());
                command
            })
            .await?;
        lease_script_result(result, lease)
    }

    /// Loads pending entry ids.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn pending_entries(&self) -> Result<Vec<ValkeyQueueEntryId>, ValkeyStoreError> {
        self.load_entries("valkey_structured_queue_pending_entries", |keys| {
            keys.pending_key()
        })
        .await
    }

    /// Loads lease deadline entry ids.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn lease_entries(&self) -> Result<Vec<ValkeyQueueEntryId>, ValkeyStoreError> {
        self.load_entries("valkey_structured_queue_lease_entries", |keys| {
            keys.lease_deadlines_key()
        })
        .await
    }

    async fn load_entries(
        &self,
        operation: &'static str,
        key: impl Fn(&ValkeyQueueKeys) -> &str,
    ) -> Result<Vec<ValkeyQueueEntryId>, ValkeyStoreError> {
        let entries: Vec<String> = self
            .client
            .run_command(operation, || {
                let mut command = redis::cmd("ZRANGE");
                command.arg(key(&self.keys)).arg(0).arg(-1);
                command
            })
            .await?;
        entries
            .into_iter()
            .map(ValkeyQueueEntryId::new)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Loads one payload by entry id.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn load_payload(
        &self,
        entry_id: &ValkeyQueueEntryId,
    ) -> Result<Option<String>, ValkeyStoreError> {
        let payload_key = self.keys.payload_key(entry_id);
        self.client
            .run_command("valkey_structured_queue_payload", || {
                let mut command = redis::cmd("HGET");
                command.arg(&payload_key).arg("payload");
                command
            })
            .await
    }

    /// Loads one lease hash by entry id.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn load_lease_hash(
        &self,
        entry_id: &ValkeyQueueEntryId,
    ) -> Result<Vec<(String, String)>, ValkeyStoreError> {
        let lease_key = self.keys.lease_key(entry_id);
        self.client
            .run_command("valkey_structured_queue_lease_hash", || {
                let mut command = redis::cmd("HGETALL");
                command.arg(&lease_key);
                command
            })
            .await
    }
}

fn lease_script_result(
    result: i64,
    lease: ValkeyStructuredQueueLeaseRef<'_>,
) -> Result<ValkeyLeaseScriptResult, ValkeyStoreError> {
    match result {
        1 => Ok(ValkeyLeaseScriptResult::Applied),
        0 => Ok(ValkeyLeaseScriptResult::NotApplied),
        -1 => Err(lease_not_owned_error(
            ValkeyLeaseId::new(lease.lease_id().as_str().to_owned())?,
            ValkeyWorkerId::new(lease.worker_id().as_str().to_owned())?,
        )),
        result => Err(ValkeyStoreError::UnexpectedLeaseScriptResult { result }),
    }
}
