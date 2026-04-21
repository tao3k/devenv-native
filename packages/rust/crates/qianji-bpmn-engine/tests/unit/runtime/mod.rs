use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind, BpmnHostBridge,
    BpmnMultiInstanceDataBindingSpec, BpmnNodeKind, BpmnNodeSpec, BpmnParallelMultiInstanceSpec,
    BpmnProcessSpec, BpmnRepeatSpec, BpmnSequentialMultiInstanceSpec, BpmnStandardLoopSpec,
    BpmnTimerKind, BpmnTimerSpec, BusinessRuleTaskOutcome, BusinessRuleTaskRequest,
    DmnDecisionDefinition, DmnDecisionRef, DmnSourceFile, EventPollOutcome, EventPollRequest,
    HostBridgeError, ManualTaskOutcome, ManualTaskRequest, ProcessKey, SendTaskOutcome,
    SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome, UserTaskRequest,
    parse_dmn_decision,
};
use serde_json::json;

mod boundary;
mod call_activity;
mod frontier;
mod gateway;
mod linear;
mod looped;
mod multi_instance;
mod wait;

fn start_end_process() -> BpmnProcessSpec {
    start_end_process_with_id("complete")
}

fn start_end_process_with_id(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "end", BpmnNodeKind::EndEvent),
        ],
        vec![BpmnEdgeSpec::new(0, 1, None::<&str>)],
        Vec::new(),
    )
}

fn linear_blocking_process(process_id: &str, node_kind: BpmnNodeKind) -> BpmnProcessSpec {
    let events = match node_kind {
        BpmnNodeKind::SendTask => vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("invoice_dispatched")
                .with_name("InvoiceDispatched"),
        ],
        _ => Vec::new(),
    };
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", node_kind),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        events,
    )
}

fn parallel_join_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "left", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(3, "right", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(4, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(5, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")),
            BpmnEdgeSpec::new(1, 3, Some("right")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
        ],
        Vec::new(),
    )
}

fn parallel_host_block_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "pass_through", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(4, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(5, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("service")),
            BpmnEdgeSpec::new(1, 3, Some("pass")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
        ],
        Vec::new(),
    )
}

fn parallel_dual_host_block_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "left_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "right_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(4, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(5, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")),
            BpmnEdgeSpec::new(1, 3, Some("right")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
        ],
        Vec::new(),
    )
}

fn parallel_host_and_wait_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "wait_message", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(4, "service_end", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(5, "wait_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("service")),
            BpmnEdgeSpec::new(1, 3, Some("wait")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 5, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(3, BpmnEventKind::Message)
                .with_reference_id("parallel_wait_message")
                .with_name("ParallelWaitMessage"),
        ],
    )
}

fn parallel_join_same_edge_duplicate_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "left_duplicate", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(3, "left_merge", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(4, "wait_right", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(5, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(6, "post_join_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(7, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")),
            BpmnEdgeSpec::new(1, 4, Some("right_wait")),
            BpmnEdgeSpec::new(2, 3, Some("left_dup_a")),
            BpmnEdgeSpec::new(2, 3, Some("left_dup_b")),
            BpmnEdgeSpec::new(3, 5, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
            BpmnEdgeSpec::new(5, 6, None::<&str>),
            BpmnEdgeSpec::new(6, 7, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(4, BpmnEventKind::Message)
                .with_reference_id("peer_arrived")
                .with_name("PeerArrived"),
        ],
    )
}

fn exclusive_branch_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "decision", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive)
                .with_default_outgoing_edge(3),
            BpmnNodeSpec::new(2, "end_left", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(3, "end_right", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(4, "end_default", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")).with_condition_expression("approved"),
            BpmnEdgeSpec::new(1, 3, Some("right")).with_condition_expression("vip"),
            BpmnEdgeSpec::new(1, 4, Some("default")),
        ],
        Vec::new(),
    )
}

fn inclusive_branch_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "decision", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Inclusive)
                .with_default_outgoing_edge(3)
                .with_inclusive_join_node(5),
            BpmnNodeSpec::new(2, "left_pass", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(3, "right_pass", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(4, "default_pass", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(5, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Inclusive),
            BpmnNodeSpec::new(6, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")).with_condition_expression("approved"),
            BpmnEdgeSpec::new(1, 3, Some("right")).with_condition_expression("vip"),
            BpmnEdgeSpec::new(1, 4, Some("default")),
            BpmnEdgeSpec::new(2, 5, None::<&str>),
            BpmnEdgeSpec::new(3, 5, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
            BpmnEdgeSpec::new(5, 6, None::<&str>),
        ],
        Vec::new(),
    )
}

fn inclusive_host_block_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "decision", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Inclusive)
                .with_inclusive_join_node(4),
            BpmnNodeSpec::new(2, "left_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "right_pass", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Exclusive),
            BpmnNodeSpec::new(4, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Inclusive),
            BpmnNodeSpec::new(5, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")).with_condition_expression("approved"),
            BpmnEdgeSpec::new(1, 3, Some("right")).with_condition_expression("vip"),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
        ],
        Vec::new(),
    )
}

fn event_based_gateway_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "wait_race", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::EventBased),
            BpmnNodeSpec::new(2, "wait_invoice", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(3, "wait_timeout", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(4, "invoice_end", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(5, "timeout_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("message")),
            BpmnEdgeSpec::new(1, 3, Some("timer")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 5, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(2, BpmnEventKind::Message)
                .with_reference_id("invoice_received")
                .with_name("InvoiceReceived"),
            BpmnEventSpec::new(3, BpmnEventKind::Timer)
                .with_name("RaceTimeout")
                .with_timer(BpmnTimerSpec::new(BpmnTimerKind::Duration, "PT5M")),
        ],
    )
}

fn intermediate_message_wait_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "wait_message", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("payment_received")
                .with_name("PaymentReceived"),
        ],
    )
}

fn receive_task_wait_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "wait_message", BpmnNodeKind::ReceiveTask),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("payment_received")
                .with_name("PaymentReceived"),
        ],
    )
}

fn intermediate_timer_wait_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "wait_timer", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(1, BpmnEventKind::Timer)
                .with_name("WaitForTimeout")
                .with_timer(BpmnTimerSpec::new(BpmnTimerKind::Duration, "PT5M")),
        ],
    )
}

fn standard_loop_service_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::ServiceTask).with_repeat(
                BpmnRepeatSpec::StandardLoop(
                    BpmnStandardLoopSpec::new(true, Some(3)).with_loop_condition("not done"),
                ),
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

fn standard_loop_business_rule_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::BusinessRuleTask)
                .with_decision(
                    DmnDecisionRef::new("loan-decision")
                        .with_source_id("simple-unique-eligibility.dmn"),
                )
                .with_repeat(BpmnRepeatSpec::StandardLoop(BpmnStandardLoopSpec::new(
                    true,
                    Some(3),
                ))),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn sequential_multi_instance_process(
    process_id: &str,
    node_kind: BpmnNodeKind,
    loop_cardinality: u32,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", node_kind).with_repeat(
                BpmnRepeatSpec::SequentialMultiInstance(BpmnSequentialMultiInstanceSpec::new(
                    loop_cardinality,
                )),
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

fn sequential_multi_instance_process_with_completion_condition(
    process_id: &str,
    node_kind: BpmnNodeKind,
    loop_cardinality: u32,
    completion_condition: &str,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", node_kind).with_repeat(
                BpmnRepeatSpec::SequentialMultiInstance(
                    BpmnSequentialMultiInstanceSpec::new(loop_cardinality)
                        .with_completion_condition(completion_condition),
                ),
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

fn sequential_multi_instance_business_rule_process(
    process_id: &str,
    loop_cardinality: u32,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::BusinessRuleTask)
                .with_decision(
                    DmnDecisionRef::new("loan-decision")
                        .with_source_id("simple-unique-eligibility.dmn"),
                )
                .with_repeat(BpmnRepeatSpec::SequentialMultiInstance(
                    BpmnSequentialMultiInstanceSpec::new(loop_cardinality),
                )),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn parallel_multi_instance_process(
    process_id: &str,
    node_kind: BpmnNodeKind,
    loop_cardinality: u32,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", node_kind).with_repeat(
                BpmnRepeatSpec::ParallelMultiInstance(BpmnParallelMultiInstanceSpec::new(
                    loop_cardinality,
                )),
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

fn parallel_multi_instance_process_with_completion_condition(
    process_id: &str,
    node_kind: BpmnNodeKind,
    loop_cardinality: u32,
    completion_condition: &str,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", node_kind).with_repeat(
                BpmnRepeatSpec::ParallelMultiInstance(
                    BpmnParallelMultiInstanceSpec::new(loop_cardinality)
                        .with_completion_condition(completion_condition),
                ),
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

fn parallel_multi_instance_business_rule_process(
    process_id: &str,
    loop_cardinality: u32,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::BusinessRuleTask)
                .with_decision(
                    DmnDecisionRef::new("loan-decision")
                        .with_source_id("simple-unique-eligibility.dmn"),
                )
                .with_repeat(BpmnRepeatSpec::ParallelMultiInstance(
                    BpmnParallelMultiInstanceSpec::new(loop_cardinality),
                )),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn sequential_multi_instance_data_binding_process(
    process_id: &str,
    node_kind: BpmnNodeKind,
) -> BpmnProcessSpec {
    let binding =
        BpmnMultiInstanceDataBindingSpec::new("items", "item").with_output("results", "result");
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", node_kind).with_repeat(
                BpmnRepeatSpec::SequentialMultiInstance(
                    BpmnSequentialMultiInstanceSpec::from_data_binding(binding),
                ),
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

fn parallel_multi_instance_data_binding_process(
    process_id: &str,
    node_kind: BpmnNodeKind,
) -> BpmnProcessSpec {
    let binding = BpmnMultiInstanceDataBindingSpec::new("assignments", "item")
        .with_output("results", "result");
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", node_kind).with_repeat(
                BpmnRepeatSpec::ParallelMultiInstance(
                    BpmnParallelMultiInstanceSpec::from_data_binding(binding),
                ),
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

fn sequential_multi_instance_data_binding_business_rule_process(
    process_id: &str,
) -> BpmnProcessSpec {
    let binding =
        BpmnMultiInstanceDataBindingSpec::new("tiers", "tier").with_output("decisions", "approval");
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::BusinessRuleTask)
                .with_decision(
                    DmnDecisionRef::new("loan-decision")
                        .with_source_id("simple-unique-eligibility.dmn"),
                )
                .with_repeat(BpmnRepeatSpec::SequentialMultiInstance(
                    BpmnSequentialMultiInstanceSpec::from_data_binding(binding),
                )),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn boundary_timer_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::UserTask),
            BpmnNodeSpec::new(2, "review_timeout", BpmnNodeKind::BoundaryEvent)
                .with_boundary_attachment(1, true),
            BpmnNodeSpec::new(3, "approved_end", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(4, "timeout_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 3, None::<&str>),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(2, BpmnEventKind::Timer)
                .with_name("ReviewTimeout")
                .with_timer(BpmnTimerSpec::new(BpmnTimerKind::Duration, "PT30M")),
        ],
    )
}

fn call_activity_main_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "invoke_child", BpmnNodeKind::SubProcess)
                .with_called_process("child_process"),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

fn call_activity_child_process() -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", "child_process", "digest_child_process"),
        vec![
            BpmnNodeSpec::new(0, "child_start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "child_review", BpmnNodeKind::UserTask),
            BpmnNodeSpec::new(2, "child_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

struct StubHost {
    now_ms: u64,
}

impl StubHost {
    fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[async_trait::async_trait]
impl BpmnHostBridge for StubHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        Ok(SendTaskOutcome { data: json!({}) })
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        panic!("runtime kernel should not dispatch host work in the blocking slice");
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        panic!("runtime kernel should not dispatch host work in the blocking slice");
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        panic!("runtime kernel should not dispatch host work in the blocking slice");
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        panic!("runtime kernel should not dispatch host work in the blocking slice");
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        panic!("runtime kernel should not poll external events in the blocking slice");
    }

    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}

fn dmn_fixture_definition(name: &str) -> DmnDecisionDefinition {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents = std::fs::read_to_string(path).must("fixture should be readable");
    parse_dmn_decision(&DmnSourceFile::new(name, contents)).must("bounded DMN fixture should parse")
}
