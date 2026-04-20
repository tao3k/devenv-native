use crate::{BpmnOrchestrationError, QianjiBpmnSchedulerLeaseConfig, SchedulerAgentIdentity};

#[test]
fn scheduler_lease_config_derives_owner_token_from_agent_id() {
    let identity = SchedulerAgentIdentity::new(
        Some(" worker-alpha ".to_string()),
        Some("Manager".to_string()),
    );

    let lease = match QianjiBpmnSchedulerLeaseConfig::from_scheduler_identity(&identity, 45_000) {
        Ok(lease) => lease,
        Err(error) => panic!("agent-backed identity should derive lease config: {error}"),
    };

    assert_eq!(lease.owner_token(), "bpmn-scheduler:worker-alpha");
    assert_eq!(lease.lease_ttl_ms(), 45_000);
}

#[test]
fn scheduler_lease_config_rejects_identity_without_agent_id() {
    let identity = SchedulerAgentIdentity::new(None, Some("manager".to_string()));

    let error = match QianjiBpmnSchedulerLeaseConfig::from_scheduler_identity(&identity, 45_000) {
        Ok(lease) => panic!("role-only identity should be rejected, got: {lease:?}"),
        Err(error) => error,
    };

    match error {
        BpmnOrchestrationError::CheckpointLeaseAgentIdRequired => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
