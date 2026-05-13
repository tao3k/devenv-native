use std::path::PathBuf;
use std::time::Duration;

use super::RepoTaskJoinResult;
use crate::analyzers::PluginRegistry;
use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::repo_index::state::task::{
    RepoIndexTask, RepoIndexTaskPriority, RepoTaskFeedback, RepoTaskOutcome,
};
use crate::repo_index::state::tests::remote_repo;
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::SearchPlaneService;

#[test]
fn handle_task_result_releases_repo_after_panicked_worker() {
    let coordinator = RepoIndexCoordinator::new(
        PathBuf::from("."),
        std::sync::Arc::new(PluginRegistry::new()),
        SearchPlaneService::new(PathBuf::from(".")),
    );
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "alpha/repo".to_string(),
        phase: RepoIndexPhase::Syncing,
        queue_position: None,
        last_error: None,
        last_revision: None,
        updated_at: None,
        attempt_count: 1,
    });
    coordinator.mark_active_for_test("alpha/repo");

    coordinator.handle_task_result(Ok(RepoTaskJoinResult::Panicked {
        repo_id: "alpha/repo".to_string(),
    }));

    let status = coordinator.status_response(None);
    assert_eq!(status.active_repo_ids, Vec::<String>::new());
    assert_eq!(status.failed, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Failed);
    assert!(
        status.repos[0]
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("panicked"))
    );
}

#[test]
fn handle_task_result_uses_control_elapsed_for_adaptive_concurrency() {
    let coordinator = RepoIndexCoordinator::new(
        PathBuf::from("."),
        std::sync::Arc::new(PluginRegistry::new()),
        SearchPlaneService::new(PathBuf::from(".")),
    );
    coordinator.mark_active_for_test("alpha/repo");
    coordinator
        .pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_back(RepoIndexTask {
            repository: remote_repo("beta/repo", "https://example.com/beta/repo.git"),
            refresh: false,
            fingerprint: "fingerprint-beta".to_string(),
            priority: RepoIndexTaskPriority::Background,
            retry_count: 0,
        });
    {
        let mut controller = coordinator
            .concurrency
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        controller.current_limit = 3;
        controller.max_limit = 6;
        controller.reference_limit = 3;
        controller.ema_elapsed_ms = Some(100.0);
        controller.baseline_elapsed_ms = Some(100.0);
        controller.previous_efficiency = Some(3.0 / 100.0);
    }

    coordinator.handle_task_result(Ok(RepoTaskJoinResult::Completed(Box::new(
        RepoTaskFeedback {
            repo_id: "alpha/repo".to_string(),
            control_elapsed: Duration::from_millis(110),
            outcome: RepoTaskOutcome::Success {
                revision: Some("rev-1".to_string()),
            },
        },
    ))));

    let controller = coordinator
        .concurrency
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(controller.current_limit, 3);
    assert_eq!(controller.success_streak, 1);
}

#[test]
fn handle_task_result_does_not_penalize_unsupported_repo_failures() {
    let coordinator = RepoIndexCoordinator::new(
        PathBuf::from("."),
        std::sync::Arc::new(PluginRegistry::new()),
        SearchPlaneService::new(PathBuf::from(".")),
    );
    coordinator.mark_active_for_test("alpha/repo");
    coordinator
        .pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_back(RepoIndexTask {
            repository: remote_repo("beta/repo", "https://example.com/beta/repo.git"),
            refresh: false,
            fingerprint: "fingerprint-beta".to_string(),
            priority: RepoIndexTaskPriority::Background,
            retry_count: 0,
        });
    {
        let mut controller = coordinator
            .concurrency
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        controller.current_limit = 3;
        controller.max_limit = 6;
        controller.reference_limit = 3;
        controller.previous_efficiency = Some(3.0 / 100.0);
    }

    coordinator.handle_task_result(Ok(RepoTaskJoinResult::Completed(Box::new(
        RepoTaskFeedback {
            repo_id: "alpha/repo".to_string(),
            control_elapsed: Duration::from_millis(110),
            outcome: RepoTaskOutcome::Failure {
                revision: None,
                error: crate::analyzers::RepoIntelligenceError::UnsupportedRepositoryLayout {
                    repo_id: "alpha/repo".to_string().into(),
                    message: "missing Project.toml".to_string(),
                },
            },
        },
    ))));

    let controller = coordinator
        .concurrency
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(controller.current_limit, 3);

    let status = coordinator.status_response(Some("alpha/repo"));
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Unsupported);
}
