use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnScriptTaskSpec, BpmnTaskIoSpec,
    BpmnTaskOutputBinding, DmnDecisionRef, InstanceLifecycle, ProcessKey, create_instance,
};

pub(super) fn send_task_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            node_with_outputs(
                BpmnNodeSpec::new(1, "send_invoice_message", BpmnNodeKind::SendTask),
                &["sent", "message_ref"],
            ),
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

pub(super) fn generic_task_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            node_with_outputs(
                BpmnNodeSpec::new(1, "do_work", BpmnNodeKind::Task),
                &["completed"],
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

pub(super) fn script_task_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            node_with_outputs(
                BpmnNodeSpec::new(1, "evaluate_script", BpmnNodeKind::ScriptTask).with_script_task(
                    BpmnScriptTaskSpec::new(Some("feel"), Some("result = amount + tax")),
                ),
                &["computed"],
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

pub(super) fn business_rule_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            node_with_outputs(
                BpmnNodeSpec::new(1, "review", BpmnNodeKind::BusinessRuleTask)
                    .with_decision(DmnDecisionRef::new("loan-decision")),
                &["approved", "tier"],
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

pub(super) fn parallel_service_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_adapter", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "split", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            node_with_outputs(
                BpmnNodeSpec::new(2, "service_a", BpmnNodeKind::ServiceTask),
                &["branch_a"],
            ),
            node_with_outputs(
                BpmnNodeSpec::new(3, "service_b", BpmnNodeKind::ServiceTask),
                &["branch_b"],
            ),
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

fn node_with_outputs(node: BpmnNodeSpec, outputs: &[&str]) -> BpmnNodeSpec {
    let mut task_io = BpmnTaskIoSpec::new();
    for output in outputs {
        task_io = task_io.with_output(BpmnTaskOutputBinding::new(*output, *output));
    }
    node.with_task_io(task_io)
}

pub(super) fn waiting_instance() -> (
    Arc<BpmnPackage>,
    xiuxian_qianji_bpmn_engine::BpmnInstanceState,
) {
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
        .push(xiuxian_qianji_bpmn_engine::TokenRecord {
            token_id: 1,
            node_index: 1,
            incoming_edge_index: None,
            inclusive_join_hint: None,
        });
    instance.node_states[0].status = xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed;
    instance.node_states[1].status = xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Executing;
    instance
        .waits
        .push(xiuxian_qianji_bpmn_engine::WaitRegistration {
            process_id: Some("wait_boundary".to_string()),
            node_index: 1,
            blocking_node_index: None,
            kind: xiuxian_qianji_bpmn_engine::WaitKind::ExternalEvent,
            event_kind: Some(BpmnEventKind::Message),
            event_reference: Some("invoice_received".to_string()),
            event_name: Some("InvoiceReceived".to_string()),
            timer: None,
            condition_expression: None,
            deduplication_key: Some("invoice:42".to_string()),
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

pub(super) fn ok_of<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error:?}"),
    }
}

pub(super) fn err_of<T, E: std::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(_) => panic!("expected error result, got Ok value"),
        Err(error) => error,
    }
}
