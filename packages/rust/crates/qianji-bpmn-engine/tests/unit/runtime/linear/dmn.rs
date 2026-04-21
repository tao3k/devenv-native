use super::super::{StubHost, dmn_fixture_definition};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage,
    BpmnProcessSpec, DmnDecisionRef, InstanceLifecycle, ProcessKey, advance_instance,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_dmn_decision_locally() {
    let decision =
        DmnDecisionRef::new("loan-decision").with_source_id("simple-unique-eligibility.dmn");
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "dmn_local", "digest_dmn_local"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "decision", BpmnNodeKind::BusinessRuleTask)
                .with_decision(decision),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(
        BpmnPackage::new("pkg_runtime", vec![process]).with_dmn_decisions(vec![
            dmn_fixture_definition("simple-unique-eligibility.dmn"),
        ]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "dmn_local",
        BpmnInstanceInit::new("wf_dmn_local", json!({ "tier": "gold" }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(70))
        .await
        .must("business rule task should execute locally when the decision is registered");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "tier": "gold", "approval": "approve" })
    );
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.sequence, 4);
    assert_eq!(instance.updated_at_ms, 70);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_offset_dmn_decision_locally() {
    let decision = DmnDecisionRef::new("release-window-offset")
        .with_source_id("datetime-comparison-release-window-offset.dmn");
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "dmn_local_offset", "digest_dmn_local_offset"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "decision", BpmnNodeKind::BusinessRuleTask)
                .with_decision(decision),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(
        BpmnPackage::new("pkg_runtime", vec![process]).with_dmn_decisions(vec![
            dmn_fixture_definition("datetime-comparison-release-window-offset.dmn"),
        ]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "dmn_local_offset",
        BpmnInstanceInit::new(
            "wf_dmn_local_offset",
            json!({ "release_timestamp": "2026-04-21T09:00:00+09:00" }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(71))
        .await
        .must("business rule task should execute locally for offset-aware DMN decisions");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(
        instance.variables,
        json!({
            "release_timestamp": "2026-04-21T09:00:00+09:00",
            "phase": "post-day-one-offset"
        })
    );
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.sequence, 4);
    assert_eq!(instance.updated_at_ms, 71);
}
