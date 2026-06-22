use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityType, ControlError, HotStateStore, IdempotencyKey, LeaseId, RunId,
    RunnableActivityTask, RunnableStep, StepId, StepLease, TaskQueue, ValkeyHotStateConfig,
    ValkeyHotStateStore, WorkerActivityTask, WorkerHeartbeat, WorkerId, WorkerRef,
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
    assert_eq!(
        config.activity_pending_queue_key(),
        "test:qianji:activity_pending"
    );
    assert_eq!(
        config.activity_lease_deadlines_key(),
        "test:qianji:activity_lease_deadlines"
    );
    assert_eq!(
        config.activity_payload_key(&run_id, Some(&step_id), &ActivityId::new("activity-d")?),
        "test:qianji:activity:run-a|step-b|activity-d"
    );
    assert_eq!(
        config.activity_lease_key(&run_id, None, &ActivityId::new("activity-e")?),
        "test:qianji:activity_lease:run-a||activity-e"
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
    let activity_task = RunnableActivityTask {
        task: WorkerActivityTask {
            run_id: RunId::new("run-activity-payload")?,
            step_id: Some(StepId::new("step-activity-payload")?),
            activity_id: ActivityId::new("activity-payload")?,
            activity_type: ActivityType::new("llm.plan")?,
            task_queue: TaskQueue::new("llm.openai")?,
            next_attempt: 1,
            scheduled_at_ms: 10,
            input_ref: None,
            idempotency_key: IdempotencyKey::new("run-activity-payload/activity/1")?,
            retry_policy: None,
            timeout_ms: Some(30_000),
            metadata: serde_json::Value::Null,
        },
        priority: 3,
        not_before_ms: 12,
        metadata: serde_json::json!({"scope": "activity"}),
    };
    let activity_task_payload = serde_json::to_string(&activity_task)?;

    assert_eq!(serde_json::from_str::<RunnableStep>(&step_payload)?, step);
    assert_eq!(
        serde_json::from_str::<RunnableActivityTask>(&activity_task_payload)?,
        activity_task
    );
    assert_eq!(
        serde_json::from_str::<WorkerHeartbeat>(&heartbeat_payload)?,
        heartbeat
    );
    Ok(())
}

#[tokio::test]
async fn valkey_hot_state_rejects_zero_activity_task_lease_ttl_before_connecting()
-> Result<(), Box<dyn Error>> {
    let store = ValkeyHotStateStore::new(ValkeyHotStateConfig::new("redis://127.0.0.1:1")?);
    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-zero-activity-ttl")?,
        capabilities: Vec::new(),
        metadata: serde_json::json!({}),
    };
    let queue = TaskQueue::new("llm.openai")?;
    let Err(error) = store
        .claim_activity_task(worker, Some(&queue), 100, 0)
        .await
    else {
        panic!("expected zero activity task lease ttl to fail before connecting");
    };

    assert!(matches!(
        error,
        ControlError::Storage {
            operation: "validate_valkey_activity_task_lease_ttl",
            ..
        }
    ));
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
