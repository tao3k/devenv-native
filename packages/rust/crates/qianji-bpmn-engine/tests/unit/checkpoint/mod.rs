use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnProcessSpec, ProcessKey, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[cfg(feature = "sqlite")]
mod sql;
#[cfg(feature = "valkey")]
mod valkey;

fn sample_checkpoint() -> BpmnCheckpointEnvelope {
    sample_checkpoint_with_sequence(0, json!({ "amount": 7 }))
}

fn sample_checkpoint_with_sequence(
    sequence: u64,
    variables: serde_json::Value,
) -> BpmnCheckpointEnvelope {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_checkpoint", "approve", "digest_checkpoint"),
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
    let package = Arc::new(BpmnPackage::new("pkg_checkpoint", vec![process]));
    let state = create_instance(
        Arc::clone(&package),
        "approve",
        BpmnInstanceInit::new("wf_checkpoint", variables, 1_760_000_000_000),
    )
    .must("known process should create an instance");
    let mut state = state;
    state.sequence = sequence;
    BpmnCheckpointEnvelope::from_state(state)
}
