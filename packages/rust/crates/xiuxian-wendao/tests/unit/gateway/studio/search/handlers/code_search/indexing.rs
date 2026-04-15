use std::fs;
use std::sync::Arc;

use crate::gateway::studio::search::handlers::code_search::search::build_code_search_response;
use crate::gateway::studio::search::handlers::tests::{
    publish_repo_content_chunk_index, publish_repo_entity_index, sample_repo_analysis,
    test_studio_state,
};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};

#[tokio::test]
async fn build_code_search_response_skips_unsupported_repositories_when_searching_all_repos() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let valid_repo = temp.path().join("ValidPkg");
    fs::create_dir_all(valid_repo.join("src"))
        .unwrap_or_else(|error| panic!("create valid src: {error}"));
    fs::write(
        valid_repo.join("Project.toml"),
        "name = \"ValidPkg\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
    )
    .unwrap_or_else(|error| panic!("write project: {error}"));
    fs::write(
        valid_repo.join("src").join("ValidPkg.jl"),
        "module ValidPkg\nusing ModelingToolkit\nend\n",
    )
    .unwrap_or_else(|error| panic!("write valid source: {error}"));

    let invalid_repo = temp.path().join("DiffEqApproxFun.jl");
    fs::create_dir_all(invalid_repo.join("src"))
        .unwrap_or_else(|error| panic!("create invalid src: {error}"));
    fs::write(
        invalid_repo.join("src").join("DiffEqApproxFun.jl"),
        "module DiffEqApproxFun\nusing ApproxFun\nend\n",
    )
    .unwrap_or_else(|error| panic!("write invalid source: {error}"));

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![
            crate::gateway::studio::types::UiRepoProjectConfig {
                id: "valid".to_string(),
                root: Some(valid_repo.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["julia".to_string()],
            },
            crate::gateway::studio::types::UiRepoProjectConfig {
                id: "invalid".to_string(),
                root: Some(invalid_repo.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["julia".to_string()],
            },
        ],
    });
    studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: "valid".to_string(),
            analysis: Arc::new(crate::analyzers::RepositoryAnalysisOutput::default()),
        }));
    publish_repo_content_chunk_index(
        &studio,
        "valid",
        vec![RepoCodeDocument {
            path: "src/ValidPkg.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("module ValidPkg\nusing ModelingToolkit\nend\n"),
            size_bytes: 40,
            modified_unix_ms: 0,
        }],
    )
    .await;
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-21T00:00:00Z".to_string()),
        attempt_count: 1,
    });
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "invalid".to_string(),
        phase: RepoIndexPhase::Unsupported,
        queue_position: None,
        last_error: Some("missing Project.toml".to_string()),
        last_revision: None,
        updated_at: Some("2026-03-21T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let response = build_code_search_response(&studio, "ValidPkg".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| {
            panic!("all-repo code search should skip unsupported repositories: {error:?}")
        });

    assert_eq!(response.query, "ValidPkg");
    assert_eq!(response.selected_mode.as_deref(), Some("code_search"));
    assert!(response.partial);
    assert_eq!(response.skipped_repos, vec!["invalid".to_string()]);
    assert!(response.hits.iter().all(|hit| {
        hit.navigation_target
            .as_ref()
            .and_then(|target| target.project_name.as_deref())
            != Some("invalid")
    }));
}

#[tokio::test]
async fn build_code_search_response_returns_pending_payload_for_explicit_repo_without_snapshot() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "DifferentialEquations.jl".to_string(),
            root: Some(".".to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "DifferentialEquations.jl".to_string(),
        phase: RepoIndexPhase::Queued,
        queue_position: None,
        last_error: None,
        last_revision: None,
        updated_at: Some("2026-03-21T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let response = build_code_search_response(
        &studio,
        "using ModelingToolkit".to_string(),
        Some("DifferentialEquations.jl"),
        5,
    )
    .await
    .unwrap_or_else(|error| panic!("repo-specific pending search should not block: {error:?}"));

    assert!(response.hits.is_empty());
    assert!(response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("indexing"));
    assert_eq!(
        response.pending_repos,
        vec!["DifferentialEquations.jl".to_string()]
    );
    assert!(response.skipped_repos.is_empty());
}

#[tokio::test]
async fn build_code_search_response_infers_repo_seed_for_exact_repo_name_query() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![
            crate::gateway::studio::types::UiRepoProjectConfig {
                id: "SciMLBase.jl".to_string(),
                root: Some(".".to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["julia".to_string()],
            },
            crate::gateway::studio::types::UiRepoProjectConfig {
                id: "QueuedRepo.jl".to_string(),
                root: Some(".".to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["julia".to_string()],
            },
        ],
    });
    publish_repo_content_chunk_index(
        &studio,
        "SciMLBase.jl",
        vec![RepoCodeDocument {
            path: "src/SciMLBase.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("module SciMLBase\nend\n"),
            size_bytes: 19,
            modified_unix_ms: 0,
        }],
    )
    .await;
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "SciMLBase.jl".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-25T00:00:00Z".to_string()),
        attempt_count: 1,
    });
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "QueuedRepo.jl".to_string(),
        phase: RepoIndexPhase::Queued,
        queue_position: Some(1),
        last_error: None,
        last_revision: None,
        updated_at: Some("2026-03-25T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let response = build_code_search_response(&studio, "SciMLBase".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| {
            panic!("exact repo-seed query should route to one repo: {error:?}")
        });

    assert!(!response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("ready"));
    assert!(response.pending_repos.is_empty());
    assert!(response.skipped_repos.is_empty());
    assert_eq!(response.hit_count, 1);
    assert!(
        response
            .hits
            .iter()
            .all(|hit| hit.path == "src/SciMLBase.jl"),
        "expected exact repo-seed routing to avoid all-repo fanout: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, hit.score))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn build_code_search_response_uses_published_repo_tables_while_repo_refreshes() {
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
    publish_repo_content_chunk_index(
        &studio,
        "valid",
        vec![RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 67,
            modified_unix_ms: 0,
        }],
    )
    .await;
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Indexing,
        queue_position: None,
        last_error: None,
        last_revision: Some("def456".to_string()),
        updated_at: Some("2026-03-23T00:00:00Z".to_string()),
        attempt_count: 2,
    });

    let response = build_code_search_response(&studio, "reexport".to_string(), Some("valid"), 10)
        .await
        .unwrap_or_else(|error| {
            panic!("refreshing repo should still serve published hits: {error:?}")
        });

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("symbol")
                && hit.path == "src/BaseModelica.jl"),
        "expected published repo entity hit while repo refreshes: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    assert!(response.pending_repos.is_empty());
}

#[tokio::test]
async fn build_code_search_response_falls_back_to_repo_content_when_repo_entity_is_unpublished() {
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
    publish_repo_content_chunk_index(
        &studio,
        "valid",
        vec![RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 67,
            modified_unix_ms: 0,
        }],
    )
    .await;
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-26T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let response = build_code_search_response(&studio, "@reexport".to_string(), Some("valid"), 10)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "repo content fallback should succeed when repo entity is unpublished: {error:?}"
            )
        });

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("file")
                && hit.path == "src/BaseModelica.jl"),
        "expected repo content fallback hit: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
}
