use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::gateway::studio::search::handlers::code_search::build_code_search_response;
use crate::gateway::studio::search::handlers::tests::{
    publish_repo_content_chunk_index, publish_repo_entity_index, sample_repo_analysis,
    test_studio_state,
};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
    RepoIndexStatusResponse,
};

struct AllRepoCodeSearchFixture {
    studio: crate::gateway::studio::router::StudioState,
    _temp: tempfile::TempDir,
}

#[tokio::test]
async fn build_code_search_response_skips_unsupported_repositories_when_searching_all_repos() {
    let fixture = build_all_repo_code_search_fixture().await;
    let studio = &fixture.studio;

    let response =
        build_code_search_response(studio, "using ModelingToolkit".to_string(), None, 10)
            .await
            .unwrap_or_else(|error| {
                panic!("all-repo code search should skip unsupported repositories: {error:?}")
            });

    assert_eq!(response.query, "using ModelingToolkit");
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

async fn build_all_repo_code_search_fixture() -> AllRepoCodeSearchFixture {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let repos = materialize_all_repo_code_search_repos(&temp);
    let studio = test_studio_state();
    configure_all_repo_code_search_projects(&studio, &repos);
    publish_all_repo_code_search_indexes(&studio).await;
    sync_all_repo_code_search_runtime(&studio).await;
    AllRepoCodeSearchFixture {
        studio,
        _temp: temp,
    }
}

struct AllRepoCodeSearchRepos {
    valid: PathBuf,
    invalid: PathBuf,
}

fn materialize_all_repo_code_search_repos(temp: &tempfile::TempDir) -> AllRepoCodeSearchRepos {
    let valid = temp.path().join("ValidPkg");
    write_julia_repo_fixture(
        valid.as_path(),
        "ValidPkg",
        "module ValidPkg\nusing ModelingToolkit\nend\n",
        true,
    );
    let invalid = temp.path().join("DiffEqApproxFun.jl");
    write_julia_repo_fixture(
        invalid.as_path(),
        "DiffEqApproxFun",
        "module DiffEqApproxFun\nusing ApproxFun\nend\n",
        false,
    );
    AllRepoCodeSearchRepos { valid, invalid }
}

fn write_julia_repo_fixture(
    repo_root: &Path,
    package_name: &str,
    source: &str,
    include_project_toml: bool,
) {
    fs::create_dir_all(repo_root.join("src"))
        .unwrap_or_else(|error| panic!("create repo src for {package_name}: {error}"));
    if include_project_toml {
        fs::write(
            repo_root.join("Project.toml"),
            format!("name = \"{package_name}\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n"),
        )
        .unwrap_or_else(|error| panic!("write project for {package_name}: {error}"));
    }
    fs::write(
        repo_root.join("src").join(format!("{package_name}.jl")),
        source,
    )
    .unwrap_or_else(|error| panic!("write source for {package_name}: {error}"));
}

fn configure_all_repo_code_search_projects(
    studio: &crate::gateway::studio::router::StudioState,
    repos: &AllRepoCodeSearchRepos,
) {
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![
            repo_project_config("valid", repos.valid.as_path()),
            repo_project_config("invalid", repos.invalid.as_path()),
        ],
    });
}

fn repo_project_config(
    repo_id: &str,
    root: &Path,
) -> crate::gateway::studio::types::UiRepoProjectConfig {
    crate::gateway::studio::types::UiRepoProjectConfig {
        id: repo_id.to_string(),
        root: Some(root.display().to_string()),
        url: None,
        git_ref: None,
        refresh: None,
        plugins: vec!["julia".to_string()],
    }
}

async fn publish_all_repo_code_search_indexes(
    studio: &crate::gateway::studio::router::StudioState,
) {
    studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: "valid".to_string(),
            analysis: Arc::new(crate::analyzers::RepositoryAnalysisOutput::default()),
        }));
    publish_repo_content_chunk_index(
        studio,
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
}

async fn sync_all_repo_code_search_runtime(studio: &crate::gateway::studio::router::StudioState) {
    let valid_status = repo_status("valid", RepoIndexPhase::Ready, None, Some("abc123"));
    let invalid_status = repo_status(
        "invalid",
        RepoIndexPhase::Unsupported,
        Some("missing Project.toml"),
        None,
    );
    studio.repo_index.set_status_for_test(valid_status.clone());
    studio
        .repo_index
        .set_status_for_test(invalid_status.clone());
    studio
        .search_plane
        .synchronize_repo_runtime_for_test(&RepoIndexStatusResponse {
            total: 2,
            active: 0,
            queued: 0,
            checking: 0,
            syncing: 0,
            indexing: 0,
            ready: 1,
            unsupported: 1,
            failed: 0,
            target_concurrency: 1,
            max_concurrency: 1,
            sync_concurrency_limit: 1,
            current_repo_id: None,
            active_repo_ids: Vec::new(),
            repos: vec![valid_status, invalid_status],
        })
        .await;
}

fn repo_status(
    repo_id: &str,
    phase: RepoIndexPhase,
    last_error: Option<&str>,
    last_revision: Option<&str>,
) -> RepoIndexEntryStatus {
    RepoIndexEntryStatus {
        repo_id: repo_id.to_string(),
        phase,
        queue_position: None,
        last_error: last_error.map(str::to_string),
        last_revision: last_revision.map(str::to_string),
        updated_at: Some("2026-03-21T00:00:00Z".to_string()),
        attempt_count: 1,
    }
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
