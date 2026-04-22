use super::support::assert_host_resume;
use qianji_bpmn_engine::{
    BpmnNodeKind, BusinessRuleTaskOutcome, DmnEvaluationResult, ManualTaskOutcome,
    PendingHostWorkKind, PendingHostWorkResult, ScriptTaskOutcome, SendTaskOutcome,
    ServiceTaskOutcome, UserTaskOutcome,
};
use serde_json::json;

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
