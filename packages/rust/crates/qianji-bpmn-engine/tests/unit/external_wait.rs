use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEngineError, BpmnEventKind, BpmnEventSpec,
    BpmnHostBridge, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome, EventPollRequest,
    HostBridgeError, InstanceLifecycle, ManualTaskOutcome, ManualTaskRequest, ProcessKey,
    ScriptTaskOutcome, ScriptTaskRequest, SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome,
    ServiceTaskRequest, TokenRecord, UserTaskOutcome, UserTaskRequest, WaitKind, WaitRegistration,
    advance_instance, apply_event_poll_outcome, build_event_poll_request, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[test]
fn external_wait_builds_event_poll_request_from_blocked_instance() {
    let (_, instance) = waiting_instance();

    let request = build_event_poll_request(&instance).must("waiting instance should emit poll");

    assert_eq!(
        request,
        EventPollRequest {
            instance_id: "wf_wait".to_string(),
            gateway_node_index: None,
            waits: vec![WaitRegistration {
                process_id: Some("wait_boundary".to_string()),
                node_index: 1,
                blocking_node_index: None,
                kind: WaitKind::ExternalEvent,
                event_kind: Some(BpmnEventKind::Message),
                event_reference: Some("invoice_received".to_string()),
                event_name: Some("InvoiceReceived".to_string()),
                timer: None,
                correlation_key: Some("invoice:42".to_string()),
            }],
        }
    );
}

#[test]
fn external_wait_requires_wait_registration() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_wait",
        vec![waiting_process("wait_boundary")],
    ));
    let instance = create_instance(
        package,
        "wait_boundary",
        BpmnInstanceInit::new("wf_wait", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    let error =
        build_event_poll_request(&instance).must_err("poll request requires a wait boundary");

    assert_eq!(
        error,
        BpmnEngineError::MissingWaitRegistration {
            instance_id: "wf_wait".to_string(),
        }
    );
}

#[test]
fn external_wait_not_ready_outcome_preserves_wait_state() {
    let (package, mut instance) = waiting_instance();
    let outcome = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "ignored": true }),
        },
        88,
    )
    .must("not-ready poll should be handled");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 1);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.sequence, 3);
    assert_eq!(instance.updated_at_ms, 55);
    assert_eq!(instance.variables, json!({ "amount": 7 }));
}

#[tokio::test(flavor = "current_thread")]
async fn external_wait_ready_outcome_routes_to_next_node() {
    let (package, mut instance) = waiting_instance();
    let host = StubHost::new(144);

    let outcome = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "approved": true }),
        },
        99,
    )
    .must("ready poll should resume the instance");

    assert_eq!(outcome, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
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
    assert_eq!(instance.updated_at_ms, 99);
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("next advance should complete at the end event");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

fn waiting_instance() -> (Arc<BpmnPackage>, qianji_bpmn_engine::BpmnInstanceState) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_wait",
        vec![waiting_process("wait_boundary")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "wait_boundary",
        BpmnInstanceInit::new("wf_wait", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    instance.sequence = 3;
    instance.lifecycle = InstanceLifecycle::Waiting;
    instance.updated_at_ms = 55;
    instance.active_tokens.push(TokenRecord {
        token_id: 1,
        node_index: 1,
        incoming_edge_index: None,
        inclusive_join_hint: None,
    });
    instance.node_states[0].status = qianji_bpmn_engine::NodeRuntimeStatus::Completed;
    instance.node_states[1].status = qianji_bpmn_engine::NodeRuntimeStatus::Executing;
    instance.waits.push(WaitRegistration {
        process_id: Some("wait_boundary".to_string()),
        node_index: 1,
        blocking_node_index: None,
        kind: WaitKind::ExternalEvent,
        event_kind: Some(BpmnEventKind::Message),
        event_reference: Some("invoice_received".to_string()),
        event_name: Some("InvoiceReceived".to_string()),
        timer: None,
        correlation_key: Some("invoice:42".to_string()),
    });

    (package, instance)
}

fn waiting_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_wait", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "wait", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("invoice_received")
                .with_name("InvoiceReceived"),
        ],
    )
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
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        panic!("external wait tests should not dispatch send work");
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        panic!("external wait tests should not dispatch service work");
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError> {
        panic!("external wait tests should not dispatch script work");
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        panic!("external wait tests should not dispatch user work");
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        panic!("external wait tests should not dispatch manual work");
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        panic!("external wait tests should not dispatch business-rule work");
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        panic!("external wait tests should not poll through the host bridge");
    }

    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}
