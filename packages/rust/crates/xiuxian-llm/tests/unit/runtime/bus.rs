use crate::model_runtime::{
    ExecutorId, ModelBus, ModelMetadata, ModelSlot, ModelSlotId, SlotState,
};

#[test]
fn model_bus_register_and_get() {
    let bus = ModelBus::new();

    let metadata = ModelMetadata {
        model_root: std::path::PathBuf::from("/test"),
        weights_size_bytes: 0,
        has_quantized_snapshot: false,
        model_kind: "test",
    };
    let slot = ModelSlot::vacant(
        ModelSlotId::new("test-model"),
        ExecutorId::new("test-executor"),
        metadata,
    );
    bus.register(slot);

    let retrieved = bus.get(&ModelSlotId::new("test-model"));
    assert!(retrieved.is_some());
}

#[test]
fn model_bus_snapshot() {
    let bus = ModelBus::new();

    let metadata = ModelMetadata {
        model_root: std::path::PathBuf::from("/test"),
        weights_size_bytes: 0,
        has_quantized_snapshot: false,
        model_kind: "test",
    };
    let slot = ModelSlot::vacant(
        ModelSlotId::new("snapshot-test"),
        ExecutorId::new("test-executor"),
        metadata,
    );
    bus.register(slot);

    let snapshot = bus.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].1, SlotState::Vacant);
}
