use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, BusinessRuleTaskOutcome,
    DmnEvaluationResult, InstanceLifecycle, ScriptTaskOutcome, SendTaskOutcome, TaskOutcome,
    advance_instance, create_instance,
};

use super::support::{
    business_rule_process, generic_task_process, ok_of, script_task_process, send_task_process,
};
use crate::{QianjiBpmnHostBridge, resolve_pending_host_work};

#[tokio::test(flavor = "current_thread")]
async fn resolve_pending_host_work_completes_generic_task_through_bridge() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_adapter",
        vec![generic_task_process("generic")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "generic",
        BpmnInstanceInit::new("wf_task", json!({ "amount": 7 }), 10),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));
    let host = QianjiBpmnHostBridge::builder()
        .on_task(|request| async move {
            assert_eq!(request.activity_id.as_str(), "do_work");
            Ok(TaskOutcome {
                data: json!({ "completed": true }),
            })
        })
        .clock(|| 100)
        .build();

    let blocked = ok_of(
        advance_instance(package.as_ref(), &mut instance, &host).await,
        "initial advance should block on generic task host work",
    );
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let outcome = ok_of(
        resolve_pending_host_work(package.as_ref(), &mut instance, &host).await,
        "host bridge should resolve the pending generic task",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "amount": 7,
            "completed": true,
        })
    );
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
