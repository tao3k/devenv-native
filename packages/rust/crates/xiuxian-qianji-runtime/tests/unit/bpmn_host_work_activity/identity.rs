use xiuxian_qianji_bpmn_engine::PendingHostWorkKind;
use xiuxian_qianji_runtime::{
    find_matching_bpmn_host_work, pending_bpmn_host_work_matches_identity,
};

use super::support::{host_work_identity, pending_work};

#[test]
fn bpmn_host_work_identity_matches_exact_pending_work() {
    let work = pending_work(PendingHostWorkKind::Service);
    let identity = host_work_identity(PendingHostWorkKind::Service);

    assert!(pending_bpmn_host_work_matches_identity(&work, &identity));
    assert_eq!(
        find_matching_bpmn_host_work(std::slice::from_ref(&work), &identity)
            .map(|matched| matched.token_id),
        Some(9)
    );
}

#[test]
fn bpmn_host_work_identity_rejects_kind_mismatch() {
    let work = pending_work(PendingHostWorkKind::User);
    let identity = host_work_identity(PendingHostWorkKind::Service);

    assert!(!pending_bpmn_host_work_matches_identity(&work, &identity));
    assert!(find_matching_bpmn_host_work(std::slice::from_ref(&work), &identity).is_none());
}

#[test]
fn bpmn_host_work_identity_rejects_missing_bpmn_identity() {
    let mut work = pending_work(PendingHostWorkKind::Service);
    work.activity_id = None;
    let identity = host_work_identity(PendingHostWorkKind::Service);

    assert!(!pending_bpmn_host_work_matches_identity(&work, &identity));
    assert!(find_matching_bpmn_host_work(std::slice::from_ref(&work), &identity).is_none());
}
