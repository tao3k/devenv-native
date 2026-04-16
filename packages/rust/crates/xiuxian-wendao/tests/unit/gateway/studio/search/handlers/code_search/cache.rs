use crate::gateway::studio::search::handlers::code_search::build_code_search_cache_key;
use crate::gateway::studio::search::handlers::tests::test_studio_state_with_cache;

#[tokio::test]
async fn build_code_search_cache_key_is_stable_for_reordered_repo_config() {
    let studio = test_studio_state_with_cache();
    studio.seed_configured_owners_for_tests(
        crate::gateway::studio::types::UiConfig {
            projects: Vec::new(),
            repo_projects: vec![
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "alpha".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["julia".to_string()],
                },
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "beta".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["modelica".to_string()],
                },
            ],
        },
        false,
    );
    let left_key = build_code_search_cache_key(&studio, "solve", None, 10)
        .await
        .unwrap_or_else(|error| panic!("left code-search cache key: {error:?}"));

    studio.seed_configured_owners_for_tests(
        crate::gateway::studio::types::UiConfig {
            projects: Vec::new(),
            repo_projects: vec![
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "beta".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["modelica".to_string()],
                },
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "alpha".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["julia".to_string()],
                },
            ],
        },
        false,
    );
    let right_key = build_code_search_cache_key(&studio, "solve", None, 10)
        .await
        .unwrap_or_else(|error| panic!("right code-search cache key: {error:?}"));

    assert!(
        left_key.is_some(),
        "expected left code-search cache key to exist"
    );
    assert_eq!(left_key, right_key);
}
