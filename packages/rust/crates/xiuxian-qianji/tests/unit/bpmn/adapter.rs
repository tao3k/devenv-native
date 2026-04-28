use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind,
    BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnScriptTaskSpec,
    BusinessRuleTaskOutcome, DmnDecisionRef, DmnEvaluationResult, EventPollOutcome,
    HostBridgeError, InstanceLifecycle, PendingHostWorkRequest, ProcessKey, ScriptTaskOutcome,
    SendTaskOutcome, ServiceTaskOutcome, UserTaskRequest, advance_instance, create_instance,
};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Barrier;
use tokio::time::{Duration, timeout};

use crate::{
    BpmnAdapterError, QianjiBpmnHostBridge, dispatch_pending_host_work_request,
    resolve_pending_host_work, resolve_waiting_external_event,
};

#[tokio::test(flavor = "current_thread")]
async fn default_bridge_keeps_unsupported_host_operations_explicit() {
    let host = QianjiBpmnHostBridge::default();
    let error = err_of(
        dispatch_pending_host_work_request(
            &host,
            PendingHostWorkRequest::User(UserTaskRequest {
                instance_id: "wf_user".to_string(),
                process_id: "review".to_string(),
                token_id: 7,
                node_index: 3,
                activity_id: "Task_Review".to_string(),
                variables: json!({ "approved": false }),
                repeat: None,
                lane: None,
                form: None,
                assignment: None,
                claim: None,
            }),
        )
        .await,
    );

    match error {
        BpmnAdapterError::Host(HostBridgeError::UnsupportedOperation { operation }) => {
            assert_eq!(operation, "dispatch_user_task");
        }
        other => panic!("expected explicit unsupported host error, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_pending_host_work_completes_business_rule_task_through_bridge() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_adapter",
        vec![business_rule_process("review")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "review",
        BpmnInstanceInit::new("wf_business_rule", json!({ "risk": "high" }), 10),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));
    let host = QianjiBpmnHostBridge::builder()
        .on_business_rule_task(|request| async move {
            Ok(BusinessRuleTaskOutcome {
                evaluation: DmnEvaluationResult::new(
                    request.evaluation.decision.decision_id.as_ref(),
                    json!({ "approved": false, "tier": "manual_review" }),
                    vec![std::sync::Arc::<str>::from("rule_host")],
                ),
            })
        })
        .clock(|| 100)
        .build();

    let blocked = ok_of(
        advance_instance(package.as_ref(), &mut instance, &host).await,
        "initial advance should block on business-rule host work",
    );
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let outcome = ok_of(
        resolve_pending_host_work(package.as_ref(), &mut instance, &host).await,
        "host bridge should resolve the pending business-rule task",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "risk": "high",
            "approved": false,
            "tier": "manual_review",
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_pending_host_work_completes_send_task_through_bridge() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_adapter",
        vec![send_task_process("send_invoice")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "send_invoice",
        BpmnInstanceInit::new("wf_send", json!({ "amount": 7 }), 10),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));
    let host = QianjiBpmnHostBridge::builder()
        .on_send_task(|request| async move {
            assert_eq!(request.message_reference, "invoice_dispatched");
            assert_eq!(request.message_name.as_deref(), Some("InvoiceDispatched"));
            Ok(SendTaskOutcome {
                data: json!({
                    "sent": true,
                    "message_ref": request.message_reference,
                }),
            })
        })
        .clock(|| 100)
        .build();

    let blocked = ok_of(
        advance_instance(package.as_ref(), &mut instance, &host).await,
        "initial advance should block on send-task host work",
    );
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let outcome = ok_of(
        resolve_pending_host_work(package.as_ref(), &mut instance, &host).await,
        "host bridge should resolve the pending send task",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "amount": 7,
            "sent": true,
            "message_ref": "invoice_dispatched",
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_pending_host_work_completes_script_task_through_bridge() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_adapter",
        vec![script_task_process("script_eval")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "script_eval",
        BpmnInstanceInit::new("wf_script", json!({ "amount": 7, "tax": 10 }), 10),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));
    let host = QianjiBpmnHostBridge::builder()
        .on_script_task(|request| async move {
            assert_eq!(request.script_format.as_deref(), Some("feel"));
            assert_eq!(
                request.script_body.as_deref(),
                Some("result = amount + tax")
            );
            Ok(ScriptTaskOutcome {
                data: json!({ "computed": 17 }),
            })
        })
        .clock(|| 100)
        .build();

    let blocked = ok_of(
        advance_instance(package.as_ref(), &mut instance, &host).await,
        "initial advance should block on script-task host work",
    );
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let outcome = ok_of(
        resolve_pending_host_work(package.as_ref(), &mut instance, &host).await,
        "host bridge should resolve the pending script task",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "amount": 7,
            "tax": 10,
            "computed": 17,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_pending_host_work_dispatches_parallel_service_batch_concurrently() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_adapter",
        vec![parallel_service_process("parallel_review")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_review",
        BpmnInstanceInit::new("wf_parallel", json!({ "seed": 1 }), 10),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));
    let entered = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(2));
    let host = QianjiBpmnHostBridge::builder()
        .on_service_task({
            let entered = Arc::clone(&entered);
            let barrier = Arc::clone(&barrier);
            move |request| {
                let entered = Arc::clone(&entered);
                let barrier = Arc::clone(&barrier);
                async move {
                    entered.fetch_add(1, Ordering::SeqCst);
                    barrier.wait().await;
                    Ok(ServiceTaskOutcome {
                        data: match request.node_index {
                            2 => json!({ "branch_a": true }),
                            3 => json!({ "branch_b": true }),
                            other => panic!("unexpected service node {other}"),
                        },
                    })
                }
            }
        })
        .clock(|| 200)
        .build();

    let blocked = ok_of(
        advance_instance(package.as_ref(), &mut instance, &host).await,
        "initial advance should block on both service tasks",
    );
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let outcome = ok_of(
        ok_of(
            timeout(
                Duration::from_secs(1),
                resolve_pending_host_work(package.as_ref(), &mut instance, &host),
            )
            .await,
            "parallel dispatch should not deadlock",
        ),
        "host bridge should resolve both pending service tasks",
    );

    assert_eq!(entered.load(Ordering::SeqCst), 2);
    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "seed": 1,
            "branch_a": true,
            "branch_b": true,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_waiting_external_event_preserves_waiting_when_poll_is_unsupported() {
    let (package, mut instance) = waiting_instance();
    let host = QianjiBpmnHostBridge::default();

    let outcome = ok_of(
        resolve_waiting_external_event(package.as_ref(), &mut instance, &host).await,
        "unsupported event polling should preserve the waiting state",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 1);
    assert_eq!(instance.variables, json!({ "amount": 7 }));
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_waiting_external_event_applies_ready_outcome_through_bridge() {
    let (package, mut instance) = waiting_instance();
    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|_request| async move {
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: None,
                data: json!({ "approved": true }),
            })
        })
        .clock(|| 144)
        .build();

    let resumed = ok_of(
        resolve_waiting_external_event(package.as_ref(), &mut instance, &host).await,
        "ready event polling should resume the waiting instance",
    );

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.variables,
        json!({
            "amount": 7,
            "approved": true,
        })
    );
}

fn send_task_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "send_invoice_message", BpmnNodeKind::SendTask),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("invoice_dispatched")
                .with_name("InvoiceDispatched"),
        ],
    )
}

fn script_task_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "evaluate_script", BpmnNodeKind::ScriptTask).with_script_task(
                BpmnScriptTaskSpec::new(Some("feel"), Some("result = amount + tax")),
            ),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn business_rule_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::BusinessRuleTask)
                .with_decision(DmnDecisionRef::new("loan-decision")),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn parallel_service_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "split", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "service_a", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "service_b", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(4, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(5, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
            BpmnEdgeSpec::new(1, 3, None::<&str>),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
        ],
        Vec::new(),
    )
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
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));

    instance.sequence = 3;
    instance.lifecycle = InstanceLifecycle::Waiting;
    instance.updated_at_ms = 55;
    instance
        .active_tokens
        .push(qianji_bpmn_engine::TokenRecord {
            token_id: 1,
            node_index: 1,
            incoming_edge_index: None,
            inclusive_join_hint: None,
        });
    instance.node_states[0].status = qianji_bpmn_engine::NodeRuntimeStatus::Completed;
    instance.node_states[1].status = qianji_bpmn_engine::NodeRuntimeStatus::Executing;
    instance.waits.push(qianji_bpmn_engine::WaitRegistration {
        process_id: Some("wait_boundary".to_string()),
        node_index: 1,
        blocking_node_index: None,
        kind: qianji_bpmn_engine::WaitKind::ExternalEvent,
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

fn ok_of<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error:?}"),
    }
}

fn err_of<T, E: std::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(_) => panic!("expected error result, got Ok value"),
        Err(error) => error,
    }
}
