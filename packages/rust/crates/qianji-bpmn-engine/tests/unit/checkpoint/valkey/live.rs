use super::super::*;
use super::support::TestValkey;
use qianji_bpmn_engine::{
    BPMN_CHECKPOINT_FORMAT_VERSION, BpmnEngineError, delete_checkpoint, delete_checkpoint_as_owner,
    load_checkpoint, release_checkpoint_lease, renew_checkpoint_lease, save_checkpoint,
    save_checkpoint_as_owner, try_acquire_checkpoint_lease,
};
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_live_round_trip_when_valkey_is_available() {
    let Some(valkey) = TestValkey::spawn_if_available()
        .await
        .must("valkey helper should decide availability cleanly")
    else {
        return;
    };
    let checkpoint = sample_checkpoint_with_sequence(1, json!({ "amount": 7 }));
    let newer = sample_checkpoint_with_sequence(2, json!({ "amount": 9 }));

    save_checkpoint(&checkpoint, valkey.url())
        .await
        .must("checkpoint should save to valkey");
    save_checkpoint(&newer, valkey.url())
        .await
        .must("newer checkpoint should replace older state");
    let loaded = load_checkpoint(checkpoint.state.instance_id.as_ref(), valkey.url())
        .await
        .must("checkpoint should load from valkey")
        .must("checkpoint should exist after save");

    assert_eq!(loaded.version, BPMN_CHECKPOINT_FORMAT_VERSION);
    assert_eq!(loaded.sequence, newer.sequence);
    assert_eq!(loaded.state, newer.state);
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_live_rejects_stale_sequences_when_valkey_is_available() {
    let Some(valkey) = TestValkey::spawn_if_available()
        .await
        .must("valkey helper should decide availability cleanly")
    else {
        return;
    };
    let current = sample_checkpoint_with_sequence(5, json!({ "amount": 11 }));
    let equal = sample_checkpoint_with_sequence(5, json!({ "amount": 13 }));
    let older = sample_checkpoint_with_sequence(4, json!({ "amount": 3 }));

    save_checkpoint(&current, valkey.url())
        .await
        .must("current checkpoint should save to valkey");

    let equal_error = save_checkpoint(&equal, valkey.url())
        .await
        .must_err("equal sequence should be rejected");
    assert_eq!(
        equal_error,
        BpmnEngineError::StaleCheckpointWrite {
            instance_id: "wf_checkpoint".to_string(),
            attempted_sequence: 5,
            stored_sequence: 5,
        }
    );

    let older_error = save_checkpoint(&older, valkey.url())
        .await
        .must_err("older sequence should be rejected");
    assert_eq!(
        older_error,
        BpmnEngineError::StaleCheckpointWrite {
            instance_id: "wf_checkpoint".to_string(),
            attempted_sequence: 4,
            stored_sequence: 5,
        }
    );

    let loaded = load_checkpoint("wf_checkpoint", valkey.url())
        .await
        .must("checkpoint should remain loadable after stale-write rejection")
        .must("checkpoint should still exist");
    assert_eq!(loaded.sequence, current.sequence);
    assert_eq!(loaded.state, current.state);
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_live_delete_removes_state_when_valkey_is_available() {
    let Some(valkey) = TestValkey::spawn_if_available()
        .await
        .must("valkey helper should decide availability cleanly")
    else {
        return;
    };
    let checkpoint = sample_checkpoint_with_sequence(3, json!({ "amount": 17 }));

    save_checkpoint(&checkpoint, valkey.url())
        .await
        .must("checkpoint should save before delete");
    delete_checkpoint(checkpoint.state.instance_id.as_ref(), valkey.url())
        .await
        .must("checkpoint delete should succeed");

    let loaded = load_checkpoint(checkpoint.state.instance_id.as_ref(), valkey.url())
        .await
        .must("checkpoint load should succeed after delete");
    assert!(loaded.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_lease_live_acquire_renew_release_when_valkey_is_available() {
    let Some(valkey) = TestValkey::spawn_if_available()
        .await
        .must("valkey helper should decide availability cleanly")
    else {
        return;
    };

    assert!(
        try_acquire_checkpoint_lease("wf_checkpoint", "owner-a", 30_000, valkey.url())
            .await
            .must("owner-a should acquire the lease")
    );
    assert!(
        !try_acquire_checkpoint_lease("wf_checkpoint", "owner-b", 30_000, valkey.url())
            .await
            .must("owner-b should lose the lease race")
    );
    assert!(
        renew_checkpoint_lease("wf_checkpoint", "owner-a", 30_000, valkey.url())
            .await
            .must("owner-a renewal should succeed")
    );
    assert!(
        !renew_checkpoint_lease("wf_checkpoint", "owner-b", 30_000, valkey.url())
            .await
            .must("owner-b renewal should fail")
    );
    assert!(
        !release_checkpoint_lease("wf_checkpoint", "owner-b", valkey.url())
            .await
            .must("owner-b release should fail")
    );
    assert!(
        release_checkpoint_lease("wf_checkpoint", "owner-a", valkey.url())
            .await
            .must("owner-a release should succeed")
    );
    assert!(
        try_acquire_checkpoint_lease("wf_checkpoint", "owner-b", 30_000, valkey.url())
            .await
            .must("owner-b should acquire the lease after release")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_live_owner_guarded_save_requires_lease_when_valkey_is_available() {
    let Some(valkey) = TestValkey::spawn_if_available()
        .await
        .must("valkey helper should decide availability cleanly")
    else {
        return;
    };
    let owner_a = "owner-a";
    let owner_b = "owner-b";
    let first = sample_checkpoint_with_sequence(1, json!({ "amount": 7 }));
    let stale = sample_checkpoint_with_sequence(1, json!({ "amount": 8 }));
    let next = sample_checkpoint_with_sequence(2, json!({ "amount": 9 }));

    let missing_lease = save_checkpoint_as_owner(&first, owner_a, valkey.url())
        .await
        .must_err("owner-guarded save should require a lease");
    assert_eq!(
        missing_lease,
        BpmnEngineError::CheckpointLeaseNotOwned {
            instance_id: "wf_checkpoint".to_string(),
        }
    );

    assert!(
        try_acquire_checkpoint_lease("wf_checkpoint", owner_a, 30_000, valkey.url())
            .await
            .must("owner-a should acquire the lease")
    );
    save_checkpoint_as_owner(&first, owner_a, valkey.url())
        .await
        .must("owner-a should save while holding the lease");

    let wrong_owner = save_checkpoint_as_owner(&next, owner_b, valkey.url())
        .await
        .must_err("non-owner save should be rejected");
    assert_eq!(
        wrong_owner,
        BpmnEngineError::CheckpointLeaseNotOwned {
            instance_id: "wf_checkpoint".to_string(),
        }
    );

    let stale_error = save_checkpoint_as_owner(&stale, owner_a, valkey.url())
        .await
        .must_err("stale sequence should still be rejected for the owner");
    assert_eq!(
        stale_error,
        BpmnEngineError::StaleCheckpointWrite {
            instance_id: "wf_checkpoint".to_string(),
            attempted_sequence: 1,
            stored_sequence: 1,
        }
    );

    save_checkpoint_as_owner(&next, owner_a, valkey.url())
        .await
        .must("newer sequence should save for the current owner");
    let loaded = load_checkpoint("wf_checkpoint", valkey.url())
        .await
        .must("owner-guarded save should remain readable")
        .must("checkpoint should still exist");
    assert_eq!(loaded.sequence, next.sequence);
    assert_eq!(loaded.state, next.state);
}

#[tokio::test(flavor = "current_thread")]
async fn checkpoint_live_owner_guarded_delete_requires_lease_when_valkey_is_available() {
    let Some(valkey) = TestValkey::spawn_if_available()
        .await
        .must("valkey helper should decide availability cleanly")
    else {
        return;
    };
    let owner_a = "owner-a";
    let owner_b = "owner-b";
    let checkpoint = sample_checkpoint_with_sequence(3, json!({ "amount": 7 }));

    save_checkpoint(&checkpoint, valkey.url())
        .await
        .must("checkpoint should save before delete ownership checks");

    let missing_lease = delete_checkpoint_as_owner("wf_checkpoint", owner_a, valkey.url())
        .await
        .must_err("owner-guarded delete should require a lease");
    assert_eq!(
        missing_lease,
        BpmnEngineError::CheckpointLeaseNotOwned {
            instance_id: "wf_checkpoint".to_string(),
        }
    );

    assert!(
        try_acquire_checkpoint_lease("wf_checkpoint", owner_a, 30_000, valkey.url())
            .await
            .must("owner-a should acquire the lease")
    );

    let wrong_owner = delete_checkpoint_as_owner("wf_checkpoint", owner_b, valkey.url())
        .await
        .must_err("owner-b should not delete without the lease");
    assert_eq!(
        wrong_owner,
        BpmnEngineError::CheckpointLeaseNotOwned {
            instance_id: "wf_checkpoint".to_string(),
        }
    );

    let still_present = load_checkpoint("wf_checkpoint", valkey.url())
        .await
        .must("checkpoint should remain after failed delete")
        .must("checkpoint should still exist");
    assert_eq!(still_present.sequence, checkpoint.sequence);
    assert_eq!(still_present.state, checkpoint.state);

    delete_checkpoint_as_owner("wf_checkpoint", owner_a, valkey.url())
        .await
        .must("owner-a should delete while holding the lease");

    let loaded = load_checkpoint("wf_checkpoint", valkey.url())
        .await
        .must("deleted checkpoint should load cleanly");
    assert!(loaded.is_none());
}
