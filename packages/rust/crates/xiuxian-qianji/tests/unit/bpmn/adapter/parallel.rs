use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Barrier;
use tokio::time::{Duration, timeout};
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, InstanceLifecycle, ServiceTaskOutcome,
    advance_instance, create_instance,
};

use super::support::{ok_of, parallel_service_process};
use crate::{QianjiBpmnHostBridge, resolve_pending_host_work};

#[tokio::test(flavor = "current_thread")]
async fn resolve_pending_host_work_dispatches_parallel_service_batch_concurrently() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_adapter",
        vec![parallel_service_process("parallel_review")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_review",
        BpmnInstanceInit::new("wf_parallel", json!({ "seed": 1 }), 10),
    )
    .unwrap_or_else(|error| panic!("instance should be created: {error:?}"));
    let entered = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(2));
    let host = QianjiBpmnHostBridge::builder()
        .on_service_task({
            let entered = Arc::clone(&entered);
            let barrier = Arc::clone(&barrier);
            move |request| {
                let entered = Arc::clone(&entered);
                let barrier = Arc::clone(&barrier);
                async move {
                    entered.fetch_add(1, Ordering::SeqCst);
                    barrier.wait().await;
                    Ok(ServiceTaskOutcome {
                        data: match request.node_index {
                            2 => json!({ "branch_a": true }),
                            3 => json!({ "branch_b": true }),
                            other => panic!("unexpected service node {other}"),
                        },
                    })
                }
            }
        })
        .clock(|| 200)
        .build();

    let blocked = ok_of(
        advance_instance(package.as_ref(), &mut instance, &host).await,
        "initial advance should block on both service tasks",
    );
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let outcome = ok_of(
        ok_of(
            timeout(
                Duration::from_secs(1),
                resolve_pending_host_work(package.as_ref(), &mut instance, &host),
            )
            .await,
            "parallel dispatch should not deadlock",
        ),
        "host bridge should resolve both pending service tasks",
    );

    assert_eq!(entered.load(Ordering::SeqCst), 2);
    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "seed": 1,
            "branch_a": true,
            "branch_b": true,
        })
    );
}
