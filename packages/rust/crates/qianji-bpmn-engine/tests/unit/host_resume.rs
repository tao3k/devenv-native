use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEngineError, BpmnHostBridge, BpmnInstanceInit,
    BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BusinessRuleTaskOutcome,
    BusinessRuleTaskRequest, DmnDecisionRef, DmnEvaluationResult, EventPollOutcome,
    EventPollRequest, HostBridgeError, InstanceLifecycle, ManualTaskOutcome, ManualTaskRequest,
    PendingHostWorkKind, PendingHostWorkResult, ProcessKey, ServiceTaskOutcome, ServiceTaskRequest,
    UserTaskOutcome, UserTaskRequest, advance_instance, apply_pending_host_work_result,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_resume_service_result_advances_and_then_completes() {
    assert_host_resume(
        BpmnNodeKind::ServiceTask,
        PendingHostWorkKind::Service,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "approved": true }),
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_user_result_advances_and_then_completes() {
    assert_host_resume(
        BpmnNodeKind::UserTask,
        PendingHostWorkKind::User,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "assignee": "ops" }),
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_manual_result_advances_and_then_completes() {
    assert_host_resume(
        BpmnNodeKind::ManualTask,
        PendingHostWorkKind::Manual,
        PendingHostWorkResult::Manual(ManualTaskOutcome {
            data: json!({ "reviewed": true }),
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_business_rule_result_advances_and_then_completes() {
    assert_host_resume(
        BpmnNodeKind::BusinessRuleTask,
        PendingHostWorkKind::BusinessRule,
        PendingHostWorkResult::BusinessRule(BusinessRuleTaskOutcome {
            evaluation: DmnEvaluationResult::new(
                "loan-decision",
                json!({ "approved": true, "tier": "gold" }),
                vec![std::sync::Arc::<str>::from("rule_1")],
            ),
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_requires_pending_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![blocking_process("resume", BpmnNodeKind::ServiceTask)],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "resume",
        BpmnInstanceInit::new("wf_resume", json!({}), 10),
    )
    .must("instance should be created");

    let error = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        1,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must_err("host completion without pending work should fail");

    assert_eq!(
        error,
        BpmnEngineError::MissingPendingHostWork {
            instance_id: "wf_resume".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_resume_rejects_kind_mismatch() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![blocking_process("resume", BpmnNodeKind::ServiceTask)],
    ));
    let mut instance =
        create_blocked_instance(Arc::clone(&package), "resume", &StubHost::new(55)).await;
    let token_id = instance.pending_host_work[0].token_id;

    let error = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must_err("host completion kind should match the pending work kind");

    assert_eq!(
        error,
        BpmnEngineError::HostResultKindMismatch {
            node_index: 1,
            expected: "service",
            actual: "user",
        }
    );
}

async fn assert_host_resume(
    node_kind: BpmnNodeKind,
    work_kind: PendingHostWorkKind,
    result: PendingHostWorkResult,
) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![blocking_process("resume", node_kind.clone())],
    ));
    let host = StubHost::new(55);
    let mut instance = create_blocked_instance(Arc::clone(&package), "resume", &host).await;
    let token_id = instance.pending_host_work[0].token_id;

    let outcome = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        result.clone(),
        100,
    )
    .must("host completion should resume the blocked instance");

    assert_eq!(outcome, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.sequence, 4);
    assert_eq!(instance.updated_at_ms, 100);
    assert_eq!(instance.variables, expected_variables(result.data()));

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("resumed instance should reach the end event");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.sequence, 5);
    assert_eq!(instance.updated_at_ms, 55);
    assert_eq!(
        work_kind,
        match node_kind {
            BpmnNodeKind::ServiceTask => PendingHostWorkKind::Service,
            BpmnNodeKind::UserTask => PendingHostWorkKind::User,
            BpmnNodeKind::ManualTask => PendingHostWorkKind::Manual,
            BpmnNodeKind::BusinessRuleTask => PendingHostWorkKind::BusinessRule,
            _ => unreachable!("helper only supports host-driven task kinds"),
        }
    );
}

async fn create_blocked_instance(
    package: Arc<BpmnPackage>,
    process_id: &str,
    host: &StubHost,
) -> qianji_bpmn_engine::BpmnInstanceState {
    let mut instance = create_instance(
        Arc::clone(&package),
        process_id,
        BpmnInstanceInit::new("wf_resume", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let blocked = advance_instance(package.as_ref(), &mut instance, host)
        .await
        .must("initial advance should block on host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.process.process_id.as_ref(), process_id);
    instance
}

fn blocking_process(process_id: &str, node_kind: BpmnNodeKind) -> BpmnProcessSpec {
    let task_node = match node_kind {
        BpmnNodeKind::BusinessRuleTask => {
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::BusinessRuleTask)
                .with_decision(DmnDecisionRef::new("loan-decision"))
        }
        _ => BpmnNodeSpec::new(1, "task", node_kind),
    };
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_resume", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            task_node,
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn expected_variables(output_data: &serde_json::Value) -> serde_json::Value {
    let mut variables = json!({ "amount": 7 });
    if let Some(obj) = output_data.as_object() {
        for (key, value) in obj {
            variables[key] = value.clone();
        }
    }
    variables
}

struct StubHost {
    now_ms: u64,
}

impl StubHost {
    fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[async_trait::async_trait]
impl BpmnHostBridge for StubHost {
    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        panic!("host resume tests should not dispatch through the host bridge");
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        panic!("host resume tests should not dispatch through the host bridge");
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        panic!("host resume tests should not dispatch through the host bridge");
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        panic!("host resume tests should not dispatch through the host bridge");
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        panic!("host resume tests should not poll external events");
    }

    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}
