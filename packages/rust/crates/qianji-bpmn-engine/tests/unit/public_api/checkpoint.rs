use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BPMN_CHECKPOINT_FORMAT_VERSION, BpmnCheckpointEnvelope, BpmnEdgeSpec, BpmnEventKind,
    BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    ProcessKey, create_instance, decode_checkpoint_json, encode_checkpoint_json, lease_key,
    state_key,
};
use serde_json::json;
use std::sync::Arc;

#[test]
fn checkpoint_keys_follow_v1_convention() {
    assert_eq!(state_key("wf_123"), "xq:bpmn:ckpt:wf_123:state");
    assert_eq!(lease_key("wf_123"), "xq:bpmn:ckpt:wf_123:lease");
}

#[test]
fn instance_creation_and_checkpoint_codec_round_trip() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg", "approve", "digest"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("done")),
        ],
        vec![BpmnEventSpec::new(0, BpmnEventKind::Message).with_name("start_message")],
    );
    let package = Arc::new(BpmnPackage::new("pkg", vec![process]));
    let state = create_instance(
        Arc::clone(&package),
        "approve",
        BpmnInstanceInit::new("wf_123", json!({ "amount": 7 }), 1_760_000_000_000),
    )
    .must("known process should create a scaffold instance");
    assert_eq!(state.process.process_id.as_ref(), "approve");
    assert_eq!(state.process_index, 0);
    assert_eq!(state.node_states.len(), 3);
    let checkpoint = BpmnCheckpointEnvelope::from_state(state.clone());
    assert_eq!(checkpoint.version, BPMN_CHECKPOINT_FORMAT_VERSION);
    let encoded = encode_checkpoint_json(&checkpoint).must("checkpoint should encode");
    let decoded = decode_checkpoint_json(&encoded).must("checkpoint should decode");
    assert_eq!(
        decoded.state.instance_id.as_ref(),
        state.instance_id.as_ref()
    );
    assert_eq!(decoded.state.process_index, 0);
}

#[test]
fn checkpoint_codec_decodes_without_process_index_field() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg", "approve", "digest"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("done")),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg", vec![process]));
    let checkpoint = BpmnCheckpointEnvelope::from_state(
        create_instance(
            Arc::clone(&package),
            "approve",
            BpmnInstanceInit::new("wf_legacy", json!({ "amount": 7 }), 1_760_000_000_000),
        )
        .must("known process should create a scaffold instance"),
    );
    let encoded = encode_checkpoint_json(&checkpoint).must("checkpoint should encode");
    let mut legacy_json: serde_json::Value =
        serde_json::from_str(&encoded).must("encoded checkpoint should be valid JSON");
    legacy_json["state"]
        .as_object_mut()
        .must("state payload should be an object")
        .remove("process_index");
    legacy_json["state"]
        .as_object_mut()
        .must("state payload should be an object")
        .remove("call_stack");
    legacy_json["state"]
        .as_object_mut()
        .must("state payload should be an object")
        .remove("standard_loops");
    legacy_json["state"]
        .as_object_mut()
        .must("state payload should be an object")
        .remove("sequential_multi_instances");
    legacy_json["state"]
        .as_object_mut()
        .must("state payload should be an object")
        .remove("event_competition");

    let decoded = decode_checkpoint_json(
        &serde_json::to_string(&legacy_json).must("legacy checkpoint JSON should re-encode"),
    )
    .must("checkpoint should decode without process_index");

    assert_eq!(decoded.state.instance_id.as_ref(), "wf_legacy");
    assert_eq!(decoded.state.process.process_id.as_ref(), "approve");
    assert_eq!(decoded.state.process_index, 0);
}
