use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnInstanceInit, BpmnNodeKind, BpmnPackage,
    PendingHostWorkClaim, PendingHostWorkKind, PendingHostWorkRequest,
    PendingHumanTaskClaimRequest, PendingHumanTaskReleaseRequest, advance_instance,
    build_pending_host_work_request, claim_pending_human_task, create_instance,
    release_pending_human_task,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn human_task_claim_records_checkpointed_owner_metadata() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![super::linear_blocking_process(
            "claim_review",
            BpmnNodeKind::UserTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "claim_review",
        BpmnInstanceInit::new("wf_claim_review", json!({}), 10),
    )
    .must("instance should be created");

    let blocked = advance_instance(package.as_ref(), &mut instance, &super::StubHost::new(11))
        .await
        .must("user task should block on host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    let token_id = instance.pending_host_work[0].token_id;
    let initial_sequence = instance.sequence;

    let outcome = claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(token_id, "claim_review", "task", "alice", 99),
    )
    .must("human task claim should succeed");

    assert!(outcome.changed);
    assert_eq!(instance.sequence, initial_sequence + 1);
    assert_eq!(instance.updated_at_ms, 99);
    assert_eq!(
        outcome.pending_host_work.claim,
        Some(PendingHostWorkClaim {
            claimant: "alice".to_string(),
            claimed_at_ms: 99,
        })
    );

    let request = build_pending_host_work_request(&instance)
        .must("claimed human task should still materialize host request");
    let PendingHostWorkRequest::User(request) = request else {
        panic!("expected user task request");
    };
    assert_eq!(
        request.claim,
        Some(PendingHostWorkClaim {
            claimant: "alice".to_string(),
            claimed_at_ms: 99,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn human_task_claim_is_idempotent_for_same_claimant() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![super::linear_blocking_process(
            "claim_idempotent",
            BpmnNodeKind::ManualTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "claim_idempotent",
        BpmnInstanceInit::new("wf_claim_idempotent", json!({}), 10),
    )
    .must("instance should be created");
    advance_instance(package.as_ref(), &mut instance, &super::StubHost::new(11))
        .await
        .must("manual task should block on host work");
    let token_id = instance.pending_host_work[0].token_id;

    claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(token_id, "claim_idempotent", "task", "alice", 99),
    )
    .must("first claim should succeed");
    let claimed_sequence = instance.sequence;

    let outcome = claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(token_id, "claim_idempotent", "task", "alice", 120),
    )
    .must("same claimant should be idempotent");

    assert!(!outcome.changed);
    assert_eq!(instance.sequence, claimed_sequence);
    assert_eq!(
        outcome.pending_host_work.claim,
        Some(PendingHostWorkClaim {
            claimant: "alice".to_string(),
            claimed_at_ms: 99,
        })
    );

    let error = claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(token_id, "claim_idempotent", "task", "bob", 130),
    )
    .must_err("different claimant should be rejected");
    assert_eq!(
        error,
        BpmnEngineError::PendingHostWorkAlreadyClaimed {
            token_id,
            claimed_by: "alice".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn human_task_claim_rejects_non_human_pending_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![super::linear_blocking_process(
            "claim_service",
            BpmnNodeKind::ServiceTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "claim_service",
        BpmnInstanceInit::new("wf_claim_service", json!({}), 10),
    )
    .must("instance should be created");
    advance_instance(package.as_ref(), &mut instance, &super::StubHost::new(11))
        .await
        .must("service task should block on host work");
    let pending = instance.pending_host_work[0].clone();

    let error = claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(pending.token_id, "claim_service", "task", "alice", 99),
    )
    .must_err("service work is not human claimable");

    assert_eq!(
        error,
        BpmnEngineError::PendingHostWorkNotHumanTask {
            token_id: pending.token_id,
            node_index: pending.node_index,
            kind: "service".to_string(),
        }
    );
    assert_eq!(pending.kind, PendingHostWorkKind::Service);
}

#[tokio::test(flavor = "current_thread")]
async fn human_task_release_clears_checkpointed_owner_metadata() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![super::linear_blocking_process(
            "release_review",
            BpmnNodeKind::UserTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "release_review",
        BpmnInstanceInit::new("wf_release_review", json!({}), 10),
    )
    .must("instance should be created");
    advance_instance(package.as_ref(), &mut instance, &super::StubHost::new(11))
        .await
        .must("user task should block on host work");
    let token_id = instance.pending_host_work[0].token_id;

    claim_pending_human_task(
        &mut instance,
        PendingHumanTaskClaimRequest::new(token_id, "release_review", "task", "alice", 99),
    )
    .must("human task claim should succeed");
    let claimed_sequence = instance.sequence;

    let mismatch = release_pending_human_task(
        &mut instance,
        PendingHumanTaskReleaseRequest::new(token_id, "release_review", "task", "bob", 120),
    )
    .must_err("different claimant should not release human task claim");
    assert_eq!(
        mismatch,
        BpmnEngineError::PendingHostWorkClaimReleaseMismatch {
            token_id,
            claimed_by: "alice".to_string(),
            requested_by: "bob".to_string(),
        }
    );
    assert_eq!(instance.sequence, claimed_sequence);

    let outcome = release_pending_human_task(
        &mut instance,
        PendingHumanTaskReleaseRequest::new(token_id, "release_review", "task", "alice", 130),
    )
    .must("same claimant should release human task claim");

    assert!(outcome.changed);
    assert_eq!(instance.sequence, claimed_sequence + 1);
    assert_eq!(instance.updated_at_ms, 130);
    assert!(outcome.pending_host_work.claim.is_none());

    let request = build_pending_host_work_request(&instance)
        .must("released human task should still materialize host request");
    let PendingHostWorkRequest::User(request) = request else {
        panic!("expected user task request");
    };
    assert!(request.claim.is_none());

    let unclaimed = release_pending_human_task(
        &mut instance,
        PendingHumanTaskReleaseRequest::new(token_id, "release_review", "task", "alice", 140),
    )
    .must_err("unclaimed human task release should fail explicitly");
    assert_eq!(
        unclaimed,
        BpmnEngineError::PendingHostWorkNotClaimed { token_id }
    );
}
