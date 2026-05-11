use std::time::{Duration, Instant};

use super::await_repository_sync_completion;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::{RepoSourceKind, RepoSyncResult};

#[tokio::test]
async fn await_repository_sync_completion_returns_without_waiting_for_full_timeout() {
    let task = tokio::task::spawn_blocking(|| {
        Ok::<RepoSyncResult, RepoIntelligenceError>(RepoSyncResult {
            repo_id: "alpha/repo".to_string().into(),
            source_kind: RepoSourceKind::ManagedRemote,
            ..RepoSyncResult::default()
        })
    });
    let started_at = Instant::now();
    let result = await_repository_sync_completion("alpha/repo", task, Duration::from_secs(1)).await;
    let elapsed = started_at.elapsed();

    assert!(result.is_ok(), "fast sync worker should succeed");
    assert!(
        elapsed < Duration::from_millis(250),
        "successful sync completion should not wait on the timeout budget, elapsed={elapsed:?}"
    );
}

#[tokio::test]
async fn await_repository_sync_completion_reports_timeout_for_stuck_worker() {
    let task = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(50));
        Ok::<RepoSyncResult, RepoIntelligenceError>(RepoSyncResult {
            repo_id: "alpha/repo".to_string().into(),
            source_kind: RepoSourceKind::ManagedRemote,
            ..RepoSyncResult::default()
        })
    });

    let error =
        match await_repository_sync_completion("alpha/repo", task, Duration::from_millis(10)).await
        {
            Ok(result) => panic!("expected timeout failure, got {result:?}"),
            Err(error) => error,
        };

    match error {
        RepoIntelligenceError::AnalysisFailed { message } => {
            assert!(message.contains("alpha/repo"));
            assert!(message.contains("timed out"));
        }
        other => panic!("expected sync timeout failure, got {other:?}"),
    }
}
