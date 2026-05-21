use std::error::Error;

use xiuxian_qianji_control::{
    ControlError, HotStateStore, LeaseId, RunId, RunnableStep, StepId, StepLease,
    ValkeyHotStateConfig, ValkeyHotStateStore, WorkerHeartbeat, WorkerId, WorkerRef,
};

#[test]
fn valkey_config_uses_stable_key_namespace() -> Result<(), Box<dyn Error>> {
    let config =
        ValkeyHotStateConfig::new("redis://127.0.0.1:6379")?.with_namespace("test:qianji")?;
    let run_id = RunId::new("run-a")?;
    let step_id = StepId::new("step-b")?;
    let worker_id = WorkerId::new("worker-c")?;

    assert_eq!(config.pending_queue_key(), "test:qianji:pending");
    assert_eq!(config.lease_deadlines_key(), "test:qianji:lease_deadlines");
    assert_eq!(
        config.step_payload_key(&run_id, &step_id),
        "test:qianji:step:run-a|step-b"
    );
    assert_eq!(
        config.lease_key(&run_id, &step_id),
        "test:qianji:lease:run-a|step-b"
    );
    assert_eq!(
        config.heartbeat_key(&worker_id),
        "test:qianji:heartbeat:worker-c"
    );
    Ok(())
}

#[test]
fn valkey_config_rejects_blank_runtime_inputs() {
    assert!(matches!(
        ValkeyHotStateConfig::new(" "),
        Err(ControlError::BlankId { field: "redis_url" })
    ));
    assert!(matches!(
        ValkeyHotStateConfig::new("redis://127.0.0.1:6379")
            .and_then(|config| config.with_namespace(" ")),
        Err(ControlError::BlankId {
            field: "valkey_key_namespace"
        })
    ));
}

#[test]
fn valkey_store_construction_does_not_connect_until_used() -> Result<(), Box<dyn Error>> {
    let config = ValkeyHotStateConfig::new("redis://127.0.0.1:1")?.with_namespace("test:qianji")?;
    let store = ValkeyHotStateStore::new(config);

    let debug_name = std::any::type_name_of_val(&store);
    assert!(debug_name.contains("ValkeyHotStateStore"));
    Ok(())
}

#[test]
fn valkey_hot_state_payload_contract_round_trips_as_json() -> Result<(), Box<dyn Error>> {
    let step = RunnableStep {
        run_id: RunId::new("run-payload")?,
        step_id: StepId::new("step-payload")?,
        priority: 7,
        not_before_ms: 42,
        metadata: serde_json::json!({"scope": "unit"}),
    };
    let heartbeat = WorkerHeartbeat {
        worker_id: WorkerId::new("worker-payload")?,
        observed_at_ms: 100,
        expires_at_ms: 200,
        metadata: serde_json::json!({"capacity": 2}),
    };

    let step_payload = serde_json::to_string(&step)?;
    let heartbeat_payload = serde_json::to_string(&heartbeat)?;

    assert_eq!(serde_json::from_str::<RunnableStep>(&step_payload)?, step);
    assert_eq!(
        serde_json::from_str::<WorkerHeartbeat>(&heartbeat_payload)?,
        heartbeat
    );
    Ok(())
}

#[tokio::test]
async fn valkey_hot_state_rejects_zero_lease_ttl_before_connecting() -> Result<(), Box<dyn Error>> {
    let store = ValkeyHotStateStore::new(ValkeyHotStateConfig::new("redis://127.0.0.1:1")?);
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-zero-ttl")?,
        capabilities: Vec::new(),
        metadata: serde_json::json!({}),
    };
    let Err(error) = store.acquire_lease(worker, 100, 0).await else {
        panic!("expected zero lease ttl to fail before connecting");
    };

    assert!(matches!(
        error,
        ControlError::Storage {
            operation: "validate_valkey_lease_ttl",
            ..
        }
    ));
    Ok(())
}

#[tokio::test]
async fn valkey_hot_state_rejects_expired_heartbeat_before_connecting() -> Result<(), Box<dyn Error>>
{
    let store = ValkeyHotStateStore::new(ValkeyHotStateConfig::new("redis://127.0.0.1:1")?);
    let heartbeat = WorkerHeartbeat {
        worker_id: WorkerId::new("worker-expired-heartbeat")?,
        observed_at_ms: 100,
        expires_at_ms: 100,
        metadata: serde_json::json!({}),
    };
    let error = match store.heartbeat(heartbeat).await {
        Ok(()) => panic!("expected expired heartbeat to fail before connecting"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        ControlError::Storage {
            operation: "validate_valkey_worker_heartbeat_ttl",
            ..
        }
    ));
    Ok(())
}

#[tokio::test]
async fn valkey_hot_state_rejects_zero_renew_ttl_before_connecting() -> Result<(), Box<dyn Error>> {
    let store = ValkeyHotStateStore::new(ValkeyHotStateConfig::new("redis://127.0.0.1:1")?);
    let lease = StepLease {
        lease_id: LeaseId::new("lease-zero-renew")?,
        run_id: RunId::new("run-zero-renew")?,
        step_id: StepId::new("step-zero-renew")?,
        worker_id: WorkerId::new("worker-zero-renew")?,
        acquired_at_ms: 10,
        expires_at_ms: 20,
    };
    let Err(error) = store.renew_lease(&lease, 20, 0).await else {
        panic!("expected zero renew ttl to fail before connecting");
    };

    assert!(matches!(
        error,
        ControlError::Storage {
            operation: "validate_valkey_lease_ttl",
            ..
        }
    ));
    Ok(())
}
