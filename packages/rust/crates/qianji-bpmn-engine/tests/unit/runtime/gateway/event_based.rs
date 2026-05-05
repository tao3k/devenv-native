use crate::runtime::{StubHost, event_based_gateway_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind,
    BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, EventPollOutcome,
    EventPollRequest, ProcessKey, WaitKind, advance_instance, apply_event_poll_outcome,
    build_event_poll_request, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_event_based_gateway_registers_competing_waits() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![event_based_gateway_process("event_race")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "event_race",
        BpmnInstanceInit::new("wf_event_race", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(91))
        .await
        .must("event-based gateway should arm competing waits");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), 2);
    assert_eq!(
        instance.event_competition,
        Some(qianji_bpmn_engine::EventCompetitionState {
            gateway_node_index: 1,
            wait_node_indices: vec![2, 3],
        })
    );
    assert_eq!(instance.active_tokens.len(), 2);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.active_tokens[1].node_index, 3);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );

    let poll_request =
        build_event_poll_request(&instance).must("competing waits should materialize a poll");
    assert_eq!(
        poll_request,
        EventPollRequest {
            instance_id: "wf_event_race".to_string(),
            gateway_node_index: Some(1),
            waits: vec![
                qianji_bpmn_engine::WaitRegistration {
                    process_id: Some("event_race".to_string()),
                    node_index: 2,
                    blocking_node_index: None,
                    kind: WaitKind::ExternalEvent,
                    event_kind: Some(BpmnEventKind::Message),
                    event_reference: Some("invoice_received".to_string()),
                    event_name: Some("InvoiceReceived".to_string()),
                    timer: None,
                    condition_expression: None,
                    deduplication_key: Some("invoice_received".to_string()),
                },
                qianji_bpmn_engine::WaitRegistration {
                    process_id: Some("event_race".to_string()),
                    node_index: 3,
                    blocking_node_index: None,
                    kind: WaitKind::Timer,
                    event_kind: Some(BpmnEventKind::Timer),
                    event_reference: None,
                    event_name: Some("RaceTimeout".to_string()),
                    timer: Some(qianji_bpmn_engine::BpmnTimerSpec::new(
                        qianji_bpmn_engine::BpmnTimerKind::Duration,
                        "PT5M",
                    )),
                    condition_expression: None,
                    deduplication_key: None,
                },
            ],
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_event_based_gateway_message_winner_cancels_siblings() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![event_based_gateway_process("event_race_winner")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "event_race_winner",
        BpmnInstanceInit::new("wf_event_race_winner", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(133);

    let armed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("event-based gateway should arm competing waits");
    assert_eq!(armed, BpmnAdvanceOutcome::WaitingExternalEvent);

    let resume = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: Some(2),
            data: json!({ "approved": true }),
        },
        144,
    )
    .must("winner should resume the message branch");

    assert_eq!(resume, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert!(instance.event_competition.is_none());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("winner branch should complete normally");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_event_based_gateway_conditional_candidate_wins_after_poll_data() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![conditional_event_based_gateway_process(
            "event_race_conditional",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "event_race_conditional",
        BpmnInstanceInit::new(
            "wf_event_race_conditional",
            json!({ "approved": false }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(144);

    let armed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("event-based gateway should arm competing waits");
    assert_eq!(armed, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), 2);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Message));
    assert_eq!(
        instance.waits[1].event_kind,
        Some(BpmnEventKind::Conditional)
    );
    assert_eq!(instance.waits[1].kind, WaitKind::Conditional);
    assert_eq!(
        instance.waits[1].condition_expression.as_deref(),
        Some("approved")
    );

    let still_waiting = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "approved": false }),
        },
        155,
    )
    .must("false conditional competition data should keep waiting");
    assert_eq!(still_waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), 2);
    assert!(instance.event_competition.is_some());

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "approved": true }),
        },
        166,
    )
    .must("true conditional competition data should select the conditional branch");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert!(instance.event_competition.is_none());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 5);
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[5].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.variables, json!({ "approved": true }));

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("conditional winner branch should complete");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_event_based_gateway_wide_competition_winner_cancels_all_siblings() {
    let wait_count = 6_u32;
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![wide_event_based_gateway_process(
            "event_race_wide",
            wait_count,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "event_race_wide",
        BpmnInstanceInit::new("wf_event_race_wide", json!({ "amount": 9 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(177);

    let armed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("wide event-based gateway should arm competing waits");
    assert_eq!(armed, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), wait_count as usize);
    assert_eq!(instance.active_tokens.len(), wait_count as usize);

    let winning_wait_node_index = 2 + wait_count - 1;
    let winning_end_node_index = 2 + wait_count + (wait_count - 1);
    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: Some(winning_wait_node_index),
            data: json!({ "approved": true, "winner": winning_wait_node_index }),
        },
        188,
    )
    .must("winner should resume the selected branch");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert!(instance.event_competition.is_none());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, winning_end_node_index);
    for wait_node_index in 2..2 + wait_count {
        let expected_status = if wait_node_index == winning_wait_node_index {
            qianji_bpmn_engine::NodeRuntimeStatus::Completed
        } else {
            qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
        };
        assert_eq!(
            instance.node_states[wait_node_index as usize].status,
            expected_status
        );
    }
    assert_eq!(
        instance.node_states[winning_end_node_index as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("winning wide event branch should complete");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
}

fn conditional_event_based_gateway_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "wait_race", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::EventBased),
            BpmnNodeSpec::new(2, "wait_invoice", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(3, "wait_approval", BpmnNodeKind::IntermediateCatchEvent),
            BpmnNodeSpec::new(4, "invoice_end", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(5, "approval_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("message")),
            BpmnEdgeSpec::new(1, 3, Some("condition")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 5, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(2, BpmnEventKind::Message)
                .with_reference_id("invoice_received")
                .with_name("InvoiceReceived"),
            BpmnEventSpec::new(3, BpmnEventKind::Conditional).with_condition_expression("approved"),
        ],
    )
}

fn wide_event_based_gateway_process(process_id: &str, wait_count: u32) -> BpmnProcessSpec {
    let mut nodes = vec![
        BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
        BpmnNodeSpec::new(1, "wait_race", BpmnNodeKind::Gateway)
            .with_gateway_kind(BpmnGatewayKind::EventBased),
    ];
    let mut edges = vec![BpmnEdgeSpec::new(0, 1, None::<&str>)];
    let mut events = Vec::with_capacity(wait_count as usize);

    for wait_offset in 0..wait_count {
        let wait_node_index = 2 + wait_offset;
        let end_node_index = 2 + wait_count + wait_offset;
        let label = format!("wait_{wait_offset}");
        let reference_id = format!("event_{wait_offset}");
        let event_name = format!("Event{wait_offset}");

        nodes.push(BpmnNodeSpec::new(
            wait_node_index,
            label.clone(),
            BpmnNodeKind::IntermediateCatchEvent,
        ));
        nodes.push(BpmnNodeSpec::new(
            end_node_index,
            format!("end_{wait_offset}"),
            BpmnNodeKind::EndEvent,
        ));
        edges.push(BpmnEdgeSpec::new(1, wait_node_index, Some(label.clone())));
        edges.push(BpmnEdgeSpec::new(
            wait_node_index,
            end_node_index,
            None::<&str>,
        ));
        events.push(
            BpmnEventSpec::new(wait_node_index, BpmnEventKind::Message)
                .with_reference_id(reference_id)
                .with_name(event_name),
        );
    }

    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        nodes,
        edges,
        events,
    )
}
