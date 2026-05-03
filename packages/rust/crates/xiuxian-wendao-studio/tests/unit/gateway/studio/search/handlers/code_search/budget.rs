use std::time::Duration;

use crate::studio::search::handlers::code_search::build_code_search_response_with_budget;
use crate::studio::search::handlers::tests::{
    publish_repo_entity_index, sample_repo_analysis, test_studio_state,
};
use xiuxian_wendao::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};

#[tokio::test]
async fn build_code_search_response_marks_partial_when_repo_wide_budget_expires() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![xiuxian_wendao::search::contracts::UiRepoProjectConfig {
            id: "valid".to_string(),
            root: Some(".".to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    publish_repo_entity_index(&studio, "valid", &sample_repo_analysis("valid")).await;
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-25T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let permit_count = studio.search_plane.available_repo_search_read_permits();
    let mut held = Vec::with_capacity(permit_count);
    for _ in 0..permit_count {
        held.push(
            studio
                .search_plane
                .acquire_repo_search_read_permit()
                .await
                .unwrap_or_else(|error| panic!("hold repo search permit: {error}")),
        );
    }

    let response = build_code_search_response_with_budget(
        &studio,
        "reexport".to_string(),
        None,
        10,
        Some(Duration::from_millis(1)),
    )
    .await
    .unwrap_or_else(|error| panic!("repo-wide timeout should return partial response: {error:?}"));

    drop(held);

    assert!(response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("partial"));
    assert_eq!(response.hit_count, 0);
    assert!(response.pending_repos.is_empty());
    assert!(response.skipped_repos.is_empty());
}
