use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnHostBridge,
    BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnScriptTaskSpec,
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, DmnDecisionRef, EventPollOutcome,
    EventPollRequest, HostBridgeError, InstanceLifecycle, ManualTaskOutcome, ManualTaskRequest,
    PendingHostWorkKind, PendingHostWorkResult, ProcessKey, ScriptTaskOutcome, ScriptTaskRequest,
    SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome,
    UserTaskRequest, advance_instance, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

pub(super) async fn assert_host_resume(
    node_kind: BpmnNodeKind,
    work_kind: PendingHostWorkKind,
    result: PendingHostWorkResult,
) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![blocking_process("resume", &node_kind)],
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
            BpmnNodeKind::SendTask => PendingHostWorkKind::Send,
            BpmnNodeKind::ServiceTask => PendingHostWorkKind::Service,
            BpmnNodeKind::ScriptTask => PendingHostWorkKind::Script,
            BpmnNodeKind::UserTask => PendingHostWorkKind::User,
            BpmnNodeKind::ManualTask => PendingHostWorkKind::Manual,
            BpmnNodeKind::BusinessRuleTask => PendingHostWorkKind::BusinessRule,
            _ => unreachable!("helper only supports host-driven task kinds"),
        }
    );
}

pub(super) async fn create_blocked_instance(
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

pub(super) fn blocking_process(process_id: &str, node_kind: &BpmnNodeKind) -> BpmnProcessSpec {
    let task_node = match node_kind {
        BpmnNodeKind::BusinessRuleTask => {
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::BusinessRuleTask)
                .with_decision(DmnDecisionRef::new("loan-decision"))
        }
        BpmnNodeKind::ScriptTask => {
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ScriptTask).with_script_task(
                BpmnScriptTaskSpec::new(Some("feel"), Some("result = amount + tax")),
            )
        }
        _ => BpmnNodeSpec::new(1, "task", node_kind.clone()),
    };
    let events = match node_kind {
        BpmnNodeKind::SendTask => vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("invoice_dispatched")
                .with_name("InvoiceDispatched"),
        ],
        _ => Vec::new(),
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
        events,
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

pub(super) struct StubHost {
    now_ms: u64,
}

impl StubHost {
    pub(super) fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[async_trait::async_trait]
impl BpmnHostBridge for StubHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        panic!("host resume tests should not execute send work");
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        panic!("host resume tests should not dispatch through the host bridge");
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError> {
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
