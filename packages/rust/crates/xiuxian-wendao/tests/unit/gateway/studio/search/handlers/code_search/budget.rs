use std::sync::Arc;
use std::time::Duration;

use crate::gateway::studio::search::handlers::code_search::search::build_code_search_response_with_budget;
use crate::gateway::studio::search::handlers::tests::{
    publish_repo_entity_index, sample_repo_analysis, test_studio_state,
};
use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};

#[tokio::test]
async fn build_code_search_response_marks_partial_when_repo_wide_budget_expires() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
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

    let permit_count = studio
        .search_plane
        .repo_search_read_permits
        .available_permits();
    let held = Arc::clone(&studio.search_plane.repo_search_read_permits)
        .acquire_many_owned(u32::try_from(permit_count).unwrap_or(u32::MAX))
        .await
        .unwrap_or_else(|error| panic!("hold repo search permits: {error}"));

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
