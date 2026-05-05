use std::path::PathBuf;
use std::sync::Arc;

use crate::model_runtime::{
    ExecutorId, ModelExecutor, ModelMetadata, ModelSlot, ModelSlotId, NoopExecutor, SlotState,
};

#[test]
fn slot_state_transitions() {
    let metadata = ModelMetadata {
        model_root: PathBuf::from("/nonexistent"),
        weights_size_bytes: 1024,
        has_quantized_snapshot: false,
        model_kind: "test",
    };
    let slot = ModelSlot::vacant(
        ModelSlotId::new("test"),
        ExecutorId::new("test-executor"),
        metadata,
    );

    assert_eq!(slot.state(), SlotState::Vacant);

    let prev = slot.hibernate();
    assert_eq!(prev, SlotState::Vacant);
    assert_eq!(slot.state(), SlotState::Hibernated);

    let executor = Arc::new(Box::new(NoopExecutor::new("test-executor")) as Box<dyn ModelExecutor>);
    let prev = slot.activate(Arc::clone(&executor));
    assert_eq!(prev, SlotState::Hibernated);
    assert_eq!(slot.state(), SlotState::Active);

    let prev = slot.evict();
    assert_eq!(prev, SlotState::Active);
    assert_eq!(slot.state(), SlotState::Hibernated);

    let prev = slot.vacate();
    assert_eq!(prev, SlotState::Hibernated);
    assert_eq!(slot.state(), SlotState::Vacant);
}
