use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::{
    GatewayState, SearchQuery, StudioState, UiConfig, UiRepoProjectConfig, json,
    load_intent_search_response_with_metadata, make_state_with_docs,
};
use crate::analyzers::{
    RegisteredRepository, RepositoryAnalysisOutput, RepositoryPluginConfig,
    RepositoryRefreshPolicy, analyze_registered_repository_with_registry,
    bootstrap_builtin_registry,
};
use crate::gateway::studio::search::handlers::tests::linked_parser_summary::{
    ensure_linked_modelica_parser_summary_service, ensure_linked_parser_summary_service,
};
use crate::gateway::studio::test_support::{
    assert_studio_json_snapshot, commit_all, init_git_repository, search_response_snapshot,
};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};
use serial_test::serial;

#[tokio::test]
#[serial(julia_live)]
async fn search_intent_routes_code_search_to_plain_julia_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_parser_summary_service()?;
    let fixture = make_state_with_docs(Vec::new());
    let repo_dir = create_sample_julia_repo(fixture.temp_dir.path(), "SearchJulia")?;
    let repository = RegisteredRepository {
        id: "julia-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let registry = bootstrap_builtin_registry()?;
    let analysis = analyze_registered_repository_with_registry(
        &repository,
        fixture.temp_dir.path(),
        &registry,
    )?;

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: fixture.state.studio.configured_projects(),
            repo_projects: vec![UiRepoProjectConfig {
                id: repository.id.clone(),
                root: Some(repo_dir.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["julia".to_string()],
            }],
        });
    publish_repository_snapshot(
        &fixture.state,
        &repository.id,
        analysis,
        vec![repo_code_document(
            &repo_dir,
            repo_dir.join("src/SearchJulia.jl"),
            "julia",
        )?],
    )
    .await;

    let (response, _metadata) = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("solve".to_string()),
            limit: Some(10),
            intent: Some("code_search".to_string()),
            repo: Some(repository.id.clone()),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("Julia code-search intent response: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("code_search"));
    assert!(
        response.hits.iter().any(
            |hit| hit.doc_type.as_deref() == Some("symbol") && hit.path == "src/SearchJulia.jl"
        ),
        "expected Julia symbol hit in code-search intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(julia_live)]
async fn search_intent_routes_code_search_to_plain_modelica_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let fixture = make_state_with_docs(Vec::new());
    let repo_dir = create_sample_modelica_repo(fixture.temp_dir.path(), "SearchModelica")?;
    let repository = RegisteredRepository {
        id: "modelica-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    let registry = bootstrap_builtin_registry()?;
    let analysis = analyze_registered_repository_with_registry(
        &repository,
        fixture.temp_dir.path(),
        &registry,
    )?;

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: fixture.state.studio.configured_projects(),
            repo_projects: vec![UiRepoProjectConfig {
                id: repository.id.clone(),
                root: Some(repo_dir.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["modelica".to_string()],
            }],
        });
    publish_repository_snapshot(
        &fixture.state,
        &repository.id,
        analysis,
        vec![
            repo_code_document(&repo_dir, repo_dir.join("package.mo"), "modelica")?,
            repo_code_document(&repo_dir, repo_dir.join("Controllers/PI.mo"), "modelica")?,
        ],
    )
    .await;

    let (response, _metadata) = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("PI".to_string()),
            limit: Some(10),
            intent: Some("code_search".to_string()),
            repo: Some(repository.id.clone()),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("Modelica code-search intent response: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("code_search"));
    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("symbol") && hit.path == "Controllers/PI.mo"
        }),
        "expected Modelica symbol hit in code-search intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(rust_ast_grep)]
async fn search_intent_routes_code_search_to_search_only_rust_repo_from_plain_seed()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = make_state_with_docs(Vec::new());
    let repo_dir = create_sample_rust_repo(fixture.temp_dir.path(), "SearchRust")?;

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: fixture.state.studio.configured_projects(),
            repo_projects: vec![UiRepoProjectConfig {
                id: "lance".to_string(),
                root: Some(repo_dir.display().to_string()),
                url: Some("https://github.com/lance-format/lance".to_string()),
                git_ref: None,
                refresh: None,
                plugins: vec!["ast-grep".to_string()],
            }],
        });

    let (response, _metadata) = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("lance".to_string()),
            limit: Some(10),
            intent: Some("code_search".to_string()),
            repo: None,
        },
    )
    .await
    .unwrap_or_else(|error| {
        panic!("search-only plain-seed code-search intent response: {error:?}")
    });

    assert_eq!(response.selected_mode.as_deref(), Some("code_search"));
    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.tags.iter().any(|tag| tag == "lang:rust")
        }),
        "expected search-only plain-seed code-search intent to return Rust hit: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(rust_ast_grep)]
async fn search_intent_routes_code_search_to_search_only_rust_and_toml_hits()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = make_state_with_docs(Vec::new());
    let repo_dir = create_sample_rust_repo(fixture.temp_dir.path(), "SearchRust")?;

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: fixture.state.studio.configured_projects(),
            repo_projects: vec![UiRepoProjectConfig {
                id: "lance".to_string(),
                root: Some(repo_dir.display().to_string()),
                url: Some("https://github.com/lance-format/lance".to_string()),
                git_ref: None,
                refresh: None,
                plugins: vec!["ast-grep".to_string()],
            }],
        });

    let (rust_response, _metadata) = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("lance lang:rust".to_string()),
            limit: Some(10),
            intent: Some("code_search".to_string()),
            repo: None,
        },
    )
    .await
    .unwrap_or_else(|error| {
        panic!("search-only rust-filter code-search intent response: {error:?}")
    });

    assert!(
        rust_response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.tags.iter().any(|tag| tag == "lang:rust")
        }),
        "expected Rust-filtered code-search intent to return src/lib.rs: {:?}",
        rust_response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );

    let (toml_response, _metadata) = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("lance lang:toml".to_string()),
            limit: Some(10),
            intent: Some("code_search".to_string()),
            repo: None,
        },
    )
    .await
    .unwrap_or_else(|error| {
        panic!("search-only toml-filter code-search intent response: {error:?}")
    });

    assert!(
        toml_response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "Cargo.toml"
                && hit.tags.iter().any(|tag| tag == "lang:toml")
        }),
        "expected TOML-filtered code-search intent to return Cargo.toml: {:?}",
        toml_response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(rust_ast_grep)]
async fn search_intent_snapshots_search_only_ast_grep_query_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = make_state_with_docs(Vec::new());
    let repo_dir = create_sample_rust_repo(fixture.temp_dir.path(), "SearchRust")?;

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: fixture.state.studio.configured_projects(),
            repo_projects: vec![UiRepoProjectConfig {
                id: "lance".to_string(),
                root: Some(repo_dir.display().to_string()),
                url: Some("https://github.com/lance-format/lance".to_string()),
                git_ref: None,
                refresh: None,
                plugins: vec!["ast-grep".to_string()],
            }],
        });

    let plain_seed = load_code_search_intent_response(fixture.state.studio.as_ref(), "lance").await;
    let rust_filter =
        load_code_search_intent_response(fixture.state.studio.as_ref(), "lance lang:rust").await;
    let toml_filter =
        load_code_search_intent_response(fixture.state.studio.as_ref(), "lance lang:toml").await;
    let placeholder_rust = load_code_search_intent_response(
        fixture.state.studio.as_ref(),
        "lance ast:\"$PATTERN\" lang:rust",
    )
    .await;
    let placeholder_toml = load_code_search_intent_response(
        fixture.state.studio.as_ref(),
        "lance ast:\"$PATTERN\" lang:toml",
    )
    .await;

    assert_studio_json_snapshot(
        "search_code_search_intent_search_only_ast_grep_query_payloads",
        json!({
            "plainSeed": search_response_snapshot(&plain_seed),
            "rustFilter": search_response_snapshot(&rust_filter),
            "tomlFilter": search_response_snapshot(&toml_filter),
            "placeholderRust": search_response_snapshot(&placeholder_rust),
            "placeholderToml": search_response_snapshot(&placeholder_toml),
        }),
    );
    Ok(())
}

async fn load_code_search_intent_response(
    studio: &StudioState,
    query: &str,
) -> crate::gateway::studio::types::SearchResponse {
    let (response, _metadata) = load_intent_search_response_with_metadata(
        studio,
        SearchQuery {
            q: Some(query.to_string()),
            limit: Some(10),
            intent: Some("code_search".to_string()),
            repo: None,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("code-search intent response for `{query}`: {error:?}"));
    response
}

async fn publish_repository_snapshot(
    state: &Arc<GatewayState>,
    repo_id: &str,
    analysis: RepositoryAnalysisOutput,
    documents: Vec<RepoCodeDocument>,
) {
    let analysis = Arc::new(analysis);
    state
        .studio
        .search_plane
        .publish_repo_entities_with_revision(repo_id, analysis.as_ref(), &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo entities for `{repo_id}`: {error}"));
    state
        .studio
        .search_plane
        .publish_repo_content_chunks_with_revision(repo_id, &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks for `{repo_id}`: {error}"));
    state
        .studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: repo_id.to_string(),
            analysis: Arc::clone(&analysis),
        }));
    state
        .studio
        .repo_index
        .set_status_for_test(RepoIndexEntryStatus {
            repo_id: repo_id.to_string(),
            phase: RepoIndexPhase::Ready,
            queue_position: None,
            last_error: None,
            last_revision: Some("fixture".to_string()),
            updated_at: Some("2026-04-09T00:00:00Z".to_string()),
            attempt_count: 1,
        });
}

fn repo_code_document(
    repo_root: &Path,
    file_path: impl AsRef<Path>,
    language: &str,
) -> Result<RepoCodeDocument, Box<dyn std::error::Error>> {
    let file_path = file_path.as_ref();
    let contents = fs::read_to_string(file_path)?;
    let relative_path = file_path
        .strip_prefix(repo_root)?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(RepoCodeDocument {
        path: relative_path,
        language: Some(language.to_string()),
        size_bytes: u64::try_from(contents.len()).unwrap_or(u64::MAX),
        modified_unix_ms: 0,
        contents: Arc::<str>::from(contents),
    })
}

fn create_sample_julia_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            "name = \"{package_name}\"\nuuid = \"12345678-1234-1234-1234-123456789abc\"\nversion = \"0.1.0\"\n"
        ),
    )?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!(
            r"module {package_name}

export solve, Problem

struct Problem
    x::Int
end

solve(problem::Problem) = problem.x

end
"
        ),
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn create_sample_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers"))?;
    fs::write(
        repo_dir.join("package.mo"),
        format!(
            r"within ;
package {package_name}
end {package_name};
"
        ),
    )?;
    fs::write(
        repo_dir.join("Controllers/PI.mo"),
        format!(
            r"within {package_name}.Controllers;
model PI
  parameter Real k = 1;
  parameter Real Ti = 0.1;
end PI;
"
        ),
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn create_sample_rust_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            package_name.to_ascii_lowercase()
        ),
    )?;
    fs::write(
        repo_dir.join("src/lib.rs"),
        r"pub struct Dataset;

fn scan_rows(dataset: &Dataset) {
    let _ = dataset;
}
",
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn initialize_git_fixture(repo_dir: &Path) {
    init_git_repository(repo_dir);
    commit_all(repo_dir, "seed fixture");
}
