use super::support::{assert_dispatch_request, blocking_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, BusinessRuleTaskRequest,
    DmnDecisionRef, DmnEvaluationRequest, ManualTaskRequest, PendingHostWorkRequest,
    ScriptTaskRequest, SendTaskRequest, ServiceTaskRequest, UserTaskRequest,
    build_pending_host_work_request, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_send_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::SendTask,
        PendingHostWorkRequest::Send(SendTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            message_reference: "invoice_dispatched".to_string(),
            message_name: Some("InvoiceDispatched".to_string()),
            variables: json!({ "amount": 7 }),
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_service_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::ServiceTask,
        PendingHostWorkRequest::Service(ServiceTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_script_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::ScriptTask,
        PendingHostWorkRequest::Script(ScriptTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            script_format: Some("feel".to_string()),
            script_body: Some("result = amount + tax".to_string()),
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_user_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::UserTask,
        PendingHostWorkRequest::User(UserTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_manual_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::ManualTask,
        PendingHostWorkRequest::Manual(ManualTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_business_rule_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::BusinessRuleTask,
        PendingHostWorkRequest::BusinessRule(BusinessRuleTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            evaluation: DmnEvaluationRequest::new(
                DmnDecisionRef::new("loan-decision"),
                json!({ "amount": 7 }),
            ),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_requires_pending_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_process("dispatch", &BpmnNodeKind::ServiceTask)],
    ));
    let instance = create_instance(
        Arc::clone(&package),
        "dispatch",
        BpmnInstanceInit::new("wf_dispatch", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    let error = build_pending_host_work_request(&instance)
        .must_err("request materialization requires pending host work");

    assert_eq!(
        error,
        BpmnEngineError::MissingPendingHostWork {
            instance_id: "wf_dispatch".to_string(),
        }
    );
}
