use super::support::{assert_dispatch_request, blocking_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnEngineError, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskChoiceSpec,
    BpmnHumanTaskFormSpec, BpmnHumanTaskResourceRoleSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BusinessRuleTaskRequest, DmnDecisionRef,
    DmnEvaluationRequest, ManualTaskRequest, PendingHostWorkRequest, ProcessKey, ScriptTaskRequest,
    SendTaskRequest, ServiceTaskRequest, UserTaskRequest, advance_instance,
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
            process_id: "dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            activity_id: "task".to_string(),
            variables: json!({ "amount": 7 }),
            repeat: None,
            lane: None,
            form: None,
            assignment: None,
            claim: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_user_request_materializes_human_task_form() {
    let form = BpmnHumanTaskFormSpec::new("choice_input")
        .with_question_ref("currentQuestion")
        .with_choices_ref("currentChoices")
        .with_choice(BpmnHumanTaskChoiceSpec::new("approve").with_label("Approve"))
        .with_result_output("answer");
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", "dispatch", "digest_dispatch"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::UserTask).with_human_task_form(form.clone()),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg_dispatch", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch",
        BpmnInstanceInit::new("wf_dispatch", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = super::support::StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    assert!(matches!(
        blocked,
        qianji_bpmn_engine::BpmnAdvanceOutcome::BlockedOnHost(_)
    ));

    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    let PendingHostWorkRequest::User(request) = request else {
        panic!("expected user request");
    };
    assert_eq!(request.form, Some(form));
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_manual_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::ManualTask,
        PendingHostWorkRequest::Manual(ManualTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            process_id: "dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            activity_id: "task".to_string(),
            variables: json!({ "amount": 7 }),
            repeat: None,
            lane: None,
            form: None,
            assignment: None,
            claim: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_manual_request_materializes_human_task_assignment() {
    let assignment = BpmnHumanTaskAssignmentSpec::new()
        .with_human_performer(
            BpmnHumanTaskResourceRoleSpec::new()
                .with_name("reviewer")
                .with_assignment_expression("users.alice"),
        )
        .with_potential_owner(
            BpmnHumanTaskResourceRoleSpec::new()
                .with_name("team")
                .with_resource_ref("reviewers"),
        );
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", "dispatch", "digest_dispatch"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ManualTask)
                .with_human_task_assignment(assignment.clone()),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg_dispatch", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch",
        BpmnInstanceInit::new("wf_dispatch", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = super::support::StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    assert!(matches!(
        blocked,
        qianji_bpmn_engine::BpmnAdvanceOutcome::BlockedOnHost(_)
    ));

    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    let PendingHostWorkRequest::Manual(request) = request else {
        panic!("expected manual request");
    };
    assert_eq!(request.assignment, Some(assignment));
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
