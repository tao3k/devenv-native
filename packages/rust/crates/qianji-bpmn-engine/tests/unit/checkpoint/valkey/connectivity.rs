use super::super::*;
use qianji_bpmn_engine::{
    BpmnEngineError, load_checkpoint, save_checkpoint, try_acquire_checkpoint_lease,
};

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_save_invalid_url_returns_storage_error() {
    let checkpoint = sample_checkpoint();
    let error = save_checkpoint(&checkpoint, "not-a-valid-valkey-url")
        .await
        .must_err("invalid valkey url should fail explicitly");

    match error {
        BpmnEngineError::CheckpointStorage { operation, .. } => {
            assert_eq!(operation, "save_checkpoint_connect");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_load_invalid_url_returns_storage_error() {
    let error = load_checkpoint("wf_checkpoint", "not-a-valid-valkey-url")
        .await
        .must_err("invalid valkey url should fail explicitly");

    match error {
        BpmnEngineError::CheckpointStorage { operation, .. } => {
            assert_eq!(operation, "load_checkpoint_connect");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_lease_zero_ttl_is_rejected() {
    let error =
        try_acquire_checkpoint_lease("wf_checkpoint", "owner-a", 0, "redis://127.0.0.1:6379/0")
            .await
            .must_err("zero lease ttl should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::InvalidCheckpointLeaseTtl { ttl_ms: 0 }
    );
}
