use std::future::pending;

use crate::search::SearchCorpusKind;
use crate::search::service::core::QueuedRepoMaintenanceTask;
use crate::search::service::core::maintenance::REPO_MAINTENANCE_SHUTDOWN_MESSAGE;
use crate::search::service::core::maintenance::tests::{make_prewarm_task, make_service};
use tokio::sync::oneshot;
use tokio::time::{Duration, timeout};
use xiuxian_db_store::VectorStoreError;

#[tokio::test]
async fn stop_repo_maintenance_clears_waiters_and_aborts_worker() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = make_service(&temp_dir, "xiuxian:test:repo-maintenance-stop");
    let task = make_prewarm_task(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        "repo_entity_alpha",
        &["path"],
    );
    let task_key = task.task_key();
    let (sender, receiver) = oneshot::channel();
    let worker_handle = tokio::spawn(async {
        pending::<()>().await;
    });
    {
        let mut runtime = service
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.in_flight.insert(task_key.clone());
        runtime.waiters.insert(task_key.clone(), vec![sender]);
        runtime.queue.push_back(QueuedRepoMaintenanceTask {
            task,
            enqueue_sequence: 0,
        });
        runtime.worker_running = true;
        runtime.worker_handle = Some(worker_handle);
        runtime.active_task = Some(task_key.clone());
    }

    service.stop_repo_maintenance();

    let waiter_result = timeout(Duration::from_secs(1), receiver)
        .await
        .unwrap_or_else(|error| panic!("waiter timeout: {error}"))
        .unwrap_or_else(|error| panic!("waiter canceled: {error}"));
    assert_eq!(
        waiter_result,
        Err(REPO_MAINTENANCE_SHUTDOWN_MESSAGE.to_string())
    );
    let runtime = service
        .repo_maintenance
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(runtime.shutdown_requested);
    assert!(runtime.in_flight.is_empty());
    assert!(runtime.waiters.is_empty());
    assert!(runtime.queue.is_empty());
    assert!(!runtime.worker_running);
    assert!(runtime.worker_handle.is_none());
    assert!(runtime.active_task.is_none());
}

#[tokio::test]
async fn prewarm_repo_table_rejects_new_tasks_after_shutdown() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = make_service(&temp_dir, "xiuxian:test:repo-maintenance-shutdown-rejects");

    service.stop_repo_maintenance();

    let Err(error) = service
        .prewarm_repo_table(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            "repo_entity_alpha",
            &["path"],
        )
        .await
    else {
        panic!("shutdown should reject repo maintenance prewarm");
    };
    assert!(matches!(
        error,
        VectorStoreError::General(message) if message == REPO_MAINTENANCE_SHUTDOWN_MESSAGE
    ));
}
