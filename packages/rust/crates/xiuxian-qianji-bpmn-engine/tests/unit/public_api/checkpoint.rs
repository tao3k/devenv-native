use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BPMN_CHECKPOINT_FORMAT_VERSION, BpmnCheckpointEnvelope, BpmnEdgeSpec, BpmnEventKind,
    BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec,
    MultiInstanceCollectionKey, MultiInstanceCollectionKind, MultiInstanceCollectionSlot,
    MultiInstanceDataRuntimeState, MultiInstanceOutputCollectionState,
    ParallelMultiInstanceIterationState, ParallelMultiInstanceState, ProcessKey, create_instance,
    decode_checkpoint_json, encode_checkpoint_json, lease_key, state_key,
};

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
fn checkpoint_codec_omits_empty_runtime_collections() {
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
            BpmnInstanceInit::new("wf_compact", json!({ "amount": 7 }), 1_760_000_000_000),
        )
        .must("known process should create a scaffold instance"),
    );

    let encoded = encode_checkpoint_json(&checkpoint).must("checkpoint should encode");
    assert!(!encoded.contains(r#""active_tokens":[]"#));
    assert!(!encoded.contains(r#""trace":[]"#));
    assert!(!encoded.contains(r#""joins":[]"#));
    assert!(!encoded.contains(r#""waits":[]"#));
    assert!(!encoded.contains(r#""pending_host_work":[]"#));
    assert!(!encoded.contains(r#""suspend_reason":null"#));

    let decoded = decode_checkpoint_json(&encoded).must("compact checkpoint should decode");
    assert!(decoded.state.active_tokens.is_empty());
    assert!(decoded.state.trace.is_empty());
    assert!(decoded.state.joins.is_empty());
    assert!(decoded.state.waits.is_empty());
    assert!(decoded.state.pending_host_work.is_empty());
    assert!(decoded.state.suspend_reason.is_none());
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
        .remove("parallel_multi_instances");
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

#[test]
fn checkpoint_codec_round_trips_multi_instance_collection_keys() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg", "approve", "digest"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg", vec![process]));
    let mut state = create_instance(
        Arc::clone(&package),
        "approve",
        BpmnInstanceInit::new(
            "wf_multi_instance_checkpoint",
            json!({ "items": ["alpha"] }),
            10,
        ),
    )
    .must("known process should create a scaffold instance");
    state
        .parallel_multi_instances
        .push(ParallelMultiInstanceState {
            node_index: 1,
            total_iterations: 1,
            completed_iterations: 0,
            data_binding: Some(MultiInstanceDataRuntimeState {
                collection_kind: MultiInstanceCollectionKind::Array,
                input_data_item: Arc::<str>::from("item"),
                slots: vec![MultiInstanceCollectionSlot {
                    key: MultiInstanceCollectionKey::Index(0),
                    input: json!("alpha"),
                }],
                output: Some(MultiInstanceOutputCollectionState {
                    loop_data_output_ref: Arc::<str>::from("results"),
                    output_data_item: Arc::<str>::from("result"),
                    values: vec![None],
                }),
            }),
            active_iterations: vec![ParallelMultiInstanceIterationState {
                token_id: 11,
                iteration_index: 0,
            }],
        });

    let checkpoint = BpmnCheckpointEnvelope::from_state(state);
    let encoded = encode_checkpoint_json(&checkpoint)
        .must("multi-instance checkpoint should encode collection keys");

    assert!(encoded.contains(r#""kind":"index""#));
    assert!(encoded.contains(r#""value":0"#));
    let decoded = decode_checkpoint_json(&encoded)
        .must("multi-instance checkpoint should decode collection keys");
    assert_eq!(
        decoded.state.parallel_multi_instances[0]
            .data_binding
            .as_ref()
            .must("data binding should round-trip")
            .slots[0]
            .key,
        MultiInstanceCollectionKey::Index(0)
    );
}
