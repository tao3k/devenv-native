use crate::runtime::{StubHost, exclusive_branch_process, exclusive_numeric_branch_process};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnInstanceInit, BpmnPackage, advance_instance,
    create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_exclusive_gateway_first_matching_condition_wins() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![exclusive_branch_process("exclusive_branch_left")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "exclusive_branch_left",
        BpmnInstanceInit::new(
            "wf_exclusive_branch_left",
            json!({ "approved": true, "vip": true }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(77))
        .await
        .must("exclusive branching should follow the first matching condition");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.active_tokens.len(), 0);
    assert_eq!(
        instance.node_states[2].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[3].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[4].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_exclusive_gateway_uses_default_when_conditions_do_not_match() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![exclusive_branch_process("exclusive_branch_default")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "exclusive_branch_default",
        BpmnInstanceInit::new(
            "wf_exclusive_branch_default",
            json!({ "approved": false, "vip": false }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(77))
        .await
        .must("exclusive branching should fall back to the default flow");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.node_states[4].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_exclusive_gateway_reports_unresolved_condition_variables() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![exclusive_branch_process("exclusive_branch_missing_var")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "exclusive_branch_missing_var",
        BpmnInstanceInit::new("wf_exclusive_branch_missing_var", json!({}), 10),
    )
    .must("instance should be created");

    let error = advance_instance(package.as_ref(), &mut instance, &StubHost::new(77))
        .await
        .must_err("missing condition variables should stay explicit");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: ("exclusive_branch_missing_var".to_string()).into(),
            node_id: ("decision".to_string()).into(),
            detail: "unresolved_condition_variable",
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_exclusive_gateway_numeric_condition_wins_first_matching_branch() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![exclusive_numeric_branch_process("exclusive_branch_numeric")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "exclusive_branch_numeric",
        BpmnInstanceInit::new(
            "wf_exclusive_branch_numeric",
            json!({ "amount": 120, "risk": 9 }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(77))
        .await
        .must("exclusive branching should follow the first matching numeric condition");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.node_states[2].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[3].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[4].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}
