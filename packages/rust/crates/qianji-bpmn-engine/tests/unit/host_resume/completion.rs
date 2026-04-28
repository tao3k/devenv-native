use super::support::assert_host_resume;
use qianji_bpmn_engine::{
    BpmnHumanTaskLifecycleEventKind, BpmnNodeKind, BpmnPackage, BusinessRuleTaskOutcome,
    DmnEvaluationResult, ManualTaskOutcome, PendingHostWorkKind, PendingHostWorkResult,
    PendingHumanTaskClaimRequest, ScriptTaskOutcome, SendTaskOutcome, ServiceTaskOutcome,
    UserTaskOutcome, apply_pending_host_work_result, claim_pending_human_task,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_resume_send_result_advances_and_then_completes() {
    assert_host_resume(
        BpmnNodeKind::SendTask,
        PendingHostWorkKind::Send,
        PendingHostWorkResult::Send(SendTaskOutcome {
            data: json!({ "sent": true }),
        }),
    )
    .await;
}

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
async fn host_resume_script_result_advances_and_then_completes() {
    assert_host_resume(
        BpmnNodeKind::ScriptTask,
        PendingHostWorkKind::Script,
        PendingHostWorkResult::Script(ScriptTaskOutcome {
            data: json!({ "computed": 17 }),
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
async fn host_resume_claimed_user_result_records_claimant_on_completed_event() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_resume",
        vec![super::support::blocking_process(
            "claimed_user_resume",
            &BpmnNodeKind::UserTask,
        )],
    ));
    let host = super::support::StubHost::new(55);
    let mut instance =
        super::support::create_blocked_instance(Arc::clone(&package), "claimed_user_resume", &host)
            .await;
    let token_id = instance.pending_host_work[0].token_id;

    claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(token_id, "claimed_user_resume", "task", "alice", 90),
    )
    .unwrap_or_else(|error| panic!("human task claim should succeed: {error:?}"));

    apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "assignee": "ops" }),
        }),
        100,
    )
    .unwrap_or_else(|error| panic!("claimed user task completion should succeed: {error:?}"));

    assert_eq!(
        instance
            .human_task_events
            .iter()
            .map(|event| event.kind.clone())
            .collect::<Vec<_>>(),
        vec![
            BpmnHumanTaskLifecycleEventKind::Created,
            BpmnHumanTaskLifecycleEventKind::Claimed,
            BpmnHumanTaskLifecycleEventKind::Completed,
        ]
    );
    let completed = instance
        .human_task_events
        .last()
        .unwrap_or_else(|| panic!("completed event should exist"));
    assert_eq!(completed.occurred_at_ms, 100);
    assert_eq!(completed.claimant.as_deref(), Some("alice"));
    assert_eq!(completed.work_kind, PendingHostWorkKind::User);
}
