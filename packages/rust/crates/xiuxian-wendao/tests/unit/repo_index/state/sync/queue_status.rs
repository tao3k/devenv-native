use super::support::{
    Duration, PathBuf, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexTaskPriority,
    RepoIntelligenceError, RepositoryAnalysisOutput, SearchPlaneService, await_analysis_completion,
    fingerprint, new_coordinator, repo, timestamp_now,
};

#[test]
fn record_repo_status_advances_attempt_count_without_lock_reentrancy() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "ADTypes.jl".to_string(),
        phase: RepoIndexPhase::Indexing,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some(timestamp_now()),
        attempt_count: 2,
    });

    coordinator.record_repo_status(
        "ADTypes.jl",
        RepoIndexPhase::Ready,
        Some("abc123".to_string()),
        None,
    );

    let status = coordinator.status_response(Some("ADTypes.jl"));
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos.first().map(|item| item.attempt_count), Some(3));
}

#[test]
fn interactive_enqueue_promotes_pending_repository_to_front() {
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    let first_repo = repo("ADTypes.jl", "./ADTypes.jl");
    let second_repo = repo("DifferentialEquations.jl", "./DifferentialEquations.jl");
    let first_fingerprint = fingerprint(&first_repo);
    let second_fingerprint = fingerprint(&second_repo);

    assert!(coordinator.enqueue_repository(
        first_repo,
        false,
        true,
        first_fingerprint,
        RepoIndexTaskPriority::Background,
    ));
    assert!(coordinator.enqueue_repository(
        second_repo.clone(),
        false,
        true,
        second_fingerprint.clone(),
        RepoIndexTaskPriority::Background,
    ));
    assert!(coordinator.enqueue_repository(
        second_repo,
        false,
        false,
        second_fingerprint,
        RepoIndexTaskPriority::Interactive,
    ));

    let pending = coordinator.pending_repo_ids_for_test();
    assert_eq!(
        pending,
        vec![
            "DifferentialEquations.jl".to_string(),
            "ADTypes.jl".to_string()
        ]
    );

    let status = coordinator.status_response(None);
    assert_eq!(
        status
            .repos
            .iter()
            .find(|repo| repo.repo_id == "DifferentialEquations.jl")
            .and_then(|repo| repo.queue_position),
        Some(1)
    );
    assert_eq!(
        status
            .repos
            .iter()
            .find(|repo| repo.repo_id == "ADTypes.jl")
            .and_then(|repo| repo.queue_position),
        Some(2)
    );
}

#[tokio::test]
async fn await_analysis_completion_returns_timeout_error_for_stuck_analysis() {
    let task = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(25));
        Ok(RepositoryAnalysisOutput::default())
    });

    let Err(error) = await_analysis_completion("stuck", task, Duration::from_millis(1)).await
    else {
        panic!("slow analysis should time out");
    };

    match error {
        RepoIntelligenceError::AnalysisFailed { message } => {
            assert!(message.contains("repo `stuck` indexing timed out"));
        }
        other => panic!("expected analysis timeout failure, got {other:?}"),
    }
}
