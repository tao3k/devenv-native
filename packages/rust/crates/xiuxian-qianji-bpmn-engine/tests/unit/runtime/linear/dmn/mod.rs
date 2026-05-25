use crate::runtime::{
    StubHost, dmn_fixture_business_knowledge_models, dmn_fixture_decision_services,
    dmn_fixture_definitions, dmn_fixture_input_data,
};
use crate::test_support::MustExt as _;
use serde_json::Value;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEngineError, BpmnInstanceInit, BpmnInstanceState,
    BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, DmnDecisionRef, InstanceLifecycle,
    NodeRuntimeStatus, ProcessKey, advance_instance, create_instance,
};

mod core;
mod datetime;
mod duration;

fn local_business_rule_process(
    process_id: &str,
    decision_id: &str,
    source_id: &str,
) -> BpmnProcessSpec {
    let digest = format!("digest_{process_id}");
    let decision = DmnDecisionRef::new(decision_id).with_source_id(source_id);
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, &digest),
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
    )
}

async fn assert_local_business_rule_task(
    process_id: &str,
    workflow_id: &str,
    decision_id: &str,
    source_id: &str,
    input: Value,
    expected_variables: Value,
    updated_at_ms: u64,
) {
    let (outcome, instance) = run_local_business_rule_task(
        process_id,
        workflow_id,
        decision_id,
        source_id,
        input,
        updated_at_ms,
    )
    .await
    .must("business rule task should execute locally when the decision is registered");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.variables, expected_variables);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.sequence, 4);
    assert_eq!(instance.updated_at_ms, updated_at_ms);
}

async fn run_local_business_rule_task(
    process_id: &str,
    workflow_id: &str,
    decision_id: &str,
    source_id: &str,
    input: Value,
    updated_at_ms: u64,
) -> std::result::Result<(BpmnAdvanceOutcome, BpmnInstanceState), BpmnEngineError> {
    let process = local_business_rule_process(process_id, decision_id, source_id);
    let package = Arc::new(
        BpmnPackage::new("pkg_runtime", vec![process])
            .with_dmn_decisions(dmn_fixture_definitions(source_id))
            .with_dmn_input_data(dmn_fixture_input_data(source_id))
            .with_dmn_business_knowledge_models(dmn_fixture_business_knowledge_models(source_id))
            .with_dmn_decision_services(dmn_fixture_decision_services(source_id)),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        process_id,
        BpmnInstanceInit::new(workflow_id, input, 10),
    )?;

    let outcome = advance_instance(
        package.as_ref(),
        &mut instance,
        &StubHost::new(updated_at_ms),
    )
    .await?;

    Ok((outcome, instance))
}
