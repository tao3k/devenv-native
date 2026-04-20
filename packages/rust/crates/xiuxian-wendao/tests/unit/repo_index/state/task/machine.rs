use std::time::Duration;

use super::{
    default_repo_index_analysis_timeout_for_parallelism,
    default_repo_index_sync_concurrency_for_parallelism,
    default_repo_index_sync_requeue_attempts_for_parallelism,
    default_repo_index_sync_timeout_for_parallelism,
};

#[test]
fn repo_index_sync_concurrency_scales_with_parallelism() {
    assert_eq!(default_repo_index_sync_concurrency_for_parallelism(1), 1);
    assert_eq!(default_repo_index_sync_concurrency_for_parallelism(2), 2);
    assert_eq!(default_repo_index_sync_concurrency_for_parallelism(4), 3);
    assert_eq!(default_repo_index_sync_concurrency_for_parallelism(12), 8);
}

#[test]
fn repo_index_sync_concurrency_caps_on_large_hosts() {
    assert_eq!(default_repo_index_sync_concurrency_for_parallelism(24), 16);
    assert_eq!(default_repo_index_sync_concurrency_for_parallelism(64), 16);
}

#[test]
fn repo_index_sync_timeout_tracks_machine_tuned_sync_budget() {
    assert_eq!(
        default_repo_index_sync_timeout_for_parallelism(1),
        Duration::from_secs(90)
    );
    assert_eq!(
        default_repo_index_sync_timeout_for_parallelism(12),
        Duration::from_mins(2)
    );
    assert_eq!(
        default_repo_index_sync_timeout_for_parallelism(24),
        Duration::from_secs(200)
    );
}

#[test]
fn repo_index_analysis_timeout_scales_with_parallelism() {
    assert_eq!(
        default_repo_index_analysis_timeout_for_parallelism(1),
        Duration::from_secs(45)
    );
    assert_eq!(
        default_repo_index_analysis_timeout_for_parallelism(12),
        Duration::from_mins(1)
    );
    assert_eq!(
        default_repo_index_analysis_timeout_for_parallelism(32),
        Duration::from_mins(2)
    );
}

#[test]
fn repo_index_sync_requeue_attempts_scale_with_parallelism() {
    assert_eq!(
        default_repo_index_sync_requeue_attempts_for_parallelism(1),
        1
    );
    assert_eq!(
        default_repo_index_sync_requeue_attempts_for_parallelism(8),
        1
    );
    assert_eq!(
        default_repo_index_sync_requeue_attempts_for_parallelism(12),
        2
    );
    assert_eq!(
        default_repo_index_sync_requeue_attempts_for_parallelism(24),
        3
    );
    assert_eq!(
        default_repo_index_sync_requeue_attempts_for_parallelism(64),
        3
    );
}
