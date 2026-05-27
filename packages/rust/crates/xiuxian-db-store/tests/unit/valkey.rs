use std::error::Error;

use xiuxian_db_store::{
    ValkeyClient, ValkeyLeaseId, ValkeyLeaseScriptResult, ValkeyQueueEntryId, ValkeyQueueKeys,
    ValkeyStoreConfig, ValkeyStoreError, ValkeyStructuredClaimFilter, ValkeyStructuredClaimRequest,
    ValkeyStructuredQueue, ValkeyStructuredQueueLeaseRef, ValkeyWorkerId,
};

#[test]
fn valkey_config_rejects_blank_inputs() {
    assert!(matches!(
        ValkeyStoreConfig::new(" "),
        Err(ValkeyStoreError::BlankId { field: "redis_url" })
    ));
    assert!(matches!(
        ValkeyStoreConfig::new("redis://127.0.0.1:6379")
            .and_then(|config| config.with_namespace(" ")),
        Err(ValkeyStoreError::BlankId { field: "namespace" })
    ));
}

#[test]
fn valkey_config_keeps_namespace_as_structured_store_fact() -> Result<(), Box<dyn Error>> {
    let config =
        ValkeyStoreConfig::new("redis://127.0.0.1:6379")?.with_namespace("test:db-store")?;

    assert_eq!(config.redis_url(), "redis://127.0.0.1:6379");
    assert_eq!(config.namespace().as_str(), "test:db-store");
    Ok(())
}

#[test]
fn valkey_queue_keys_derive_payload_and_lease_keys_without_domain_parsing()
-> Result<(), Box<dyn Error>> {
    let keys = test_queue_keys()?;

    assert_eq!(keys.pending_key(), "test:queue:pending");
    assert_eq!(keys.lease_deadlines_key(), "test:queue:lease_deadlines");
    assert_eq!(
        keys.payload_key(&test_entry_id()?),
        "test:queue:payload:run-a|step-b|activity-c"
    );
    assert_eq!(
        keys.lease_key(&test_entry_id()?),
        "test:queue:lease:run-a|step-b|activity-c"
    );
    Ok(())
}

#[test]
fn valkey_queue_keys_reject_blank_key_parts() {
    assert!(matches!(
        ValkeyQueueKeys::new("", "lease-deadlines", "payload:", "lease:"),
        Err(ValkeyStoreError::BlankId {
            field: "pending_key"
        })
    ));
    assert!(matches!(
        ValkeyQueueKeys::new("pending", "lease-deadlines", "", "lease:"),
        Err(ValkeyStoreError::BlankId {
            field: "payload_prefix"
        })
    ));
}

#[tokio::test]
async fn valkey_queue_claim_rejects_zero_ttl_before_connecting() -> Result<(), Box<dyn Error>> {
    let queue = offline_queue()?;
    let filters = [ValkeyStructuredClaimFilter::new("run_id", "run-a")];
    let worker_id = ValkeyWorkerId::new("worker-a")?;
    let lease_id = ValkeyLeaseId::new("lease-a")?;

    let error = queue
        .claim(
            ValkeyStructuredClaimRequest::new(&worker_id, &lease_id, 100, 0).with_filters(&filters),
        )
        .await
        .err()
        .ok_or("zero TTL should fail before connecting")?;

    assert!(matches!(
        error,
        ValkeyStoreError::NonPositiveTtl {
            field: "lease_ttl_ms"
        }
    ));
    Ok(())
}

#[tokio::test]
async fn valkey_queue_renew_rejects_zero_ttl_before_connecting() -> Result<(), Box<dyn Error>> {
    let queue = offline_queue()?;

    let error = queue
        .renew(test_lease_ref()?, 100, 0)
        .await
        .err()
        .ok_or("zero TTL should fail before connecting")?;

    assert!(matches!(
        error,
        ValkeyStoreError::NonPositiveTtl {
            field: "lease_ttl_ms"
        }
    ));
    Ok(())
}

#[test]
fn valkey_lease_script_result_is_compact_and_explicit() {
    assert_eq!(
        ValkeyLeaseScriptResult::Applied,
        ValkeyLeaseScriptResult::Applied
    );
    assert_eq!(
        ValkeyLeaseScriptResult::NotApplied,
        ValkeyLeaseScriptResult::NotApplied
    );
}

fn offline_queue() -> Result<ValkeyStructuredQueue, Box<dyn Error>> {
    let config = ValkeyStoreConfig::new("redis://127.0.0.1:1")?.with_namespace("test:db-store")?;
    Ok(ValkeyStructuredQueue::new(
        ValkeyClient::new(config),
        test_queue_keys()?,
    ))
}

fn test_queue_keys() -> Result<ValkeyQueueKeys, ValkeyStoreError> {
    ValkeyQueueKeys::new(
        "test:queue:pending",
        "test:queue:lease_deadlines",
        "test:queue:payload:",
        "test:queue:lease:",
    )
}

fn test_lease_ref() -> Result<ValkeyStructuredQueueLeaseRef<'static>, Box<dyn Error>> {
    let lease = Box::leak(Box::new(ValkeyLeaseId::new("lease-a")?));
    let entry = Box::leak(Box::new(test_entry_id()?));
    let worker = Box::leak(Box::new(ValkeyWorkerId::new("worker-a")?));
    Ok(ValkeyStructuredQueueLeaseRef::new(lease, entry, worker))
}

fn test_entry_id() -> Result<ValkeyQueueEntryId, ValkeyStoreError> {
    ValkeyQueueEntryId::new("run-a|step-b|activity-c")
}
