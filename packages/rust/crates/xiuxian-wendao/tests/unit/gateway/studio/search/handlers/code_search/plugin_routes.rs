use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::analyzers::{
    RegisteredRepository, RepositoryAnalysisOutput, RepositoryPluginConfig,
    RepositoryRefreshPolicy, analyze_registered_repository_with_registry,
    bootstrap_builtin_registry,
};
use crate::gateway::studio::search::handlers::code_search::search::build_code_search_response;
use crate::gateway::studio::search::handlers::tests::linked_parser_summary::{
    ensure_linked_julia_parser_summary_service, ensure_linked_modelica_parser_summary_service,
};
use crate::gateway::studio::search::handlers::tests::test_studio_state;
use crate::gateway::studio::test_support::{
    assert_studio_json_snapshot, commit_all, init_git_repository, search_response_snapshot,
};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};

#[tokio::test]
#[serial_test::serial(julia_live)]
async fn build_code_search_response_returns_hits_for_plain_julia_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "SearchJulia")?;
    let repository = RegisteredRepository {
        id: "julia-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let registry = bootstrap_builtin_registry()?;
    let analysis =
        analyze_registered_repository_with_registry(&repository, temp.path(), &registry)?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: repository.id.clone(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    publish_repository_snapshot(
        &studio,
        &repository.id,
        analysis,
        vec![repo_code_document(
            &repo_dir,
            repo_dir.join("src/SearchJulia.jl"),
            "julia",
        )?],
    )
    .await;

    let response = build_code_search_response(&studio, "solve".to_string(), Some("julia-live"), 10)
        .await
        .unwrap_or_else(|error| panic!("Julia code search response: {error:?}"));

    assert!(
        response.hits.iter().any(
            |hit| hit.doc_type.as_deref() == Some("symbol") && hit.path == "src/SearchJulia.jl"
        ),
        "expected Julia symbol hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(julia_live)]
async fn build_code_search_response_returns_hits_for_plain_modelica_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "SearchModelica")?;
    let repository = RegisteredRepository {
        id: "modelica-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    let registry = bootstrap_builtin_registry()?;
    let analysis =
        analyze_registered_repository_with_registry(&repository, temp.path(), &registry)?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: repository.id.clone(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["modelica".to_string()],
        }],
    });
    publish_repository_snapshot(
        &studio,
        &repository.id,
        analysis,
        vec![
            repo_code_document(&repo_dir, repo_dir.join("package.mo"), "modelica")?,
            repo_code_document(&repo_dir, repo_dir.join("Controllers/PI.mo"), "modelica")?,
        ],
    )
    .await;

    let response = build_code_search_response(&studio, "PI".to_string(), Some("modelica-live"), 10)
        .await
        .unwrap_or_else(|error| panic!("Modelica code search response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("symbol") && hit.path == "Controllers/PI.mo"),
        "expected Modelica symbol hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_supports_repo_scoped_ast_grep_without_published_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:rust ast:\"fn $NAME($$$ARGS) { $$$BODY }\"".to_string(),
        Some("rust-live"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep code search response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected ast-grep hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_supports_repo_scoped_generic_ast_analysis_without_pattern()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response =
        build_code_search_response(&studio, "scan lang:rust".to_string(), Some("rust-live"), 10)
            .await
            .unwrap_or_else(|error| panic!("Rust ast-grep analysis response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.best_section.as_deref() == Some("fn scan_rows(dataset: &Dataset) {")
        }),
        "expected generic ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_treats_placeholder_ast_pattern_as_generic_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:rust ast:\"$PATTERN\"".to_string(),
        Some("rust-live"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep placeholder response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected placeholder ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_treats_search_only_repo_seed_as_generic_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(&studio, "lance".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("search-only repo seed response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.best_section.as_deref() == Some("fn scan_rows(dataset: &Dataset) {")
        }),
        "expected search-only repo seed to trigger generic ast analysis: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_returns_rust_hits_for_search_only_repo_seed_with_lang_filter()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(&studio, "lance lang:rust".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("search-only repo rust response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "src/lib.rs"
                && hit.tags.iter().any(|tag| tag == "lang:rust")
        }),
        "expected search-only repo rust query to return src/lib.rs: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_returns_toml_hits_for_search_only_repo_seed_with_lang_filter()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(&studio, "lance lang:toml".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("search-only repo toml response: {error:?}"));

    assert!(
        response.hits.iter().any(|hit| {
            hit.doc_type.as_deref() == Some("ast_match")
                && hit.path == "Cargo.toml"
                && hit.tags.iter().any(|tag| tag == "lang:toml")
        }),
        "expected search-only repo toml query to return Cargo.toml: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_snapshots_search_only_ast_grep_query_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "lance".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let plain_seed = load_code_search_response(&studio, "lance").await;
    let rust_filter = load_code_search_response(&studio, "lance lang:rust").await;
    let toml_filter = load_code_search_response(&studio, "lance lang:toml").await;
    let placeholder_rust =
        load_code_search_response(&studio, "lance ast:\"$PATTERN\" lang:rust").await;
    let placeholder_toml =
        load_code_search_response(&studio, "lance ast:\"$PATTERN\" lang:toml").await;

    assert_studio_json_snapshot(
        "search_code_search_response_search_only_ast_grep_query_payloads",
        serde_json::json!({
            "plainSeed": search_response_snapshot(&plain_seed),
            "rustFilter": search_response_snapshot(&rust_filter),
            "tomlFilter": search_response_snapshot(&toml_filter),
            "placeholderRust": search_response_snapshot(&placeholder_rust),
            "placeholderToml": search_response_snapshot(&placeholder_toml),
        }),
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_treats_repo_alias_search_term_as_scope_for_placeholder_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "lancd".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lance lang:rust ast:\"$PATTERN\"".to_string(),
        Some("lancd"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep alias placeholder response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected alias-scoped placeholder ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_infers_repo_scope_from_repository_url_alias()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "lancd".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lance lang:rust ast:\"$PATTERN\"".to_string(),
        None,
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("Rust ast-grep inferred alias response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "src/lib.rs"),
        "expected inferred alias ast-grep analysis hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.best_section, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_supports_repo_scoped_ast_grep_for_html_without_published_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_html_repo(temp.path(), "SearchHtml")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "html-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:html ast:\"<title>$TEXT</title>\"".to_string(),
        Some("html-live"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("HTML ast-grep code search response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("ast_match") && hit.path == "index.html"),
        "expected HTML ast-grep hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_excludes_language_owned_by_non_ast_plugin_from_generic_ast_grep()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_toml_repo(temp.path(), "OwnedToml")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "toml-owned".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["toml".to_string()],
        }],
    });

    let response = build_code_search_response(
        &studio,
        "lang:toml ast:\"name = $VALUE\"".to_string(),
        Some("toml-owned"),
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("owned-language ast-grep code search response: {error:?}"));

    assert!(
        response.hits.is_empty(),
        "generic ast-grep should skip files owned by a dedicated plugin window: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type, &hit.tags))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial_test::serial(rust_ast_grep)]
async fn build_code_search_response_rejects_ast_grep_without_explicit_repository_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_rust_repo(temp.path(), "SearchRust")?;

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "rust-live".to_string(),
            root: Some(repo_dir.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let error = match build_code_search_response(
        &studio,
        "ast:\"fn $NAME($$$ARGS) { $$$BODY }\"".to_string(),
        None,
        10,
    )
    .await
    {
        Ok(response) => panic!("repo-scopeless ast-grep query should fail: {response:?}"),
        Err(error) => error,
    };

    assert_eq!(error.code(), "MISSING_REPOSITORY");
    assert_eq!(
        error.error.message,
        "ast-grep code search requires repo:<id> or an explicit repository hint"
    );
    Ok(())
}

async fn publish_repository_snapshot(
    studio: &crate::gateway::studio::router::StudioState,
    repo_id: &str,
    analysis: RepositoryAnalysisOutput,
    documents: Vec<RepoCodeDocument>,
) {
    let analysis = Arc::new(analysis);
    studio
        .search_plane
        .publish_repo_entities_with_revision(repo_id, analysis.as_ref(), &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo entities for `{repo_id}`: {error}"));
    studio
        .search_plane
        .publish_repo_content_chunks_with_revision(repo_id, &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks for `{repo_id}`: {error}"));
    studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: repo_id.to_string(),
            analysis: Arc::clone(&analysis),
        }));
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
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

function solve(problem::Problem)
    problem.x
end
end
"
        ),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

fn create_sample_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers"))?;
    fs::write(repo_dir.join("package.order"), "Controllers\n")?;
    fs::write(
        repo_dir.join("package.mo"),
        format!("within;\npackage {package_name}\nend {package_name};\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("package.mo"),
        format!("within {package_name};\npackage Controllers\nend Controllers;\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("PI.mo"),
        format!("within {package_name}.Controllers;\nmodel PI\n  parameter Real k = 1;\nend PI;\n"),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
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
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

fn create_sample_toml_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(&repo_dir)?;
    fs::write(
        repo_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            package_name.to_ascii_lowercase()
        ),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

async fn load_code_search_response(
    studio: &crate::gateway::studio::router::StudioState,
    query: &str,
) -> crate::gateway::studio::types::SearchResponse {
    build_code_search_response(studio, query.to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("code search response for `{query}`: {error:?}"))
}

fn create_sample_html_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(&repo_dir)?;
    fs::write(
        repo_dir.join("index.html"),
        format!(
            "<!doctype html>\n<html>\n  <head>\n    <title>{package_name}</title>\n  </head>\n  <body>\n    <main><section>search fixture</section></main>\n  </body>\n</html>\n"
        ),
    )?;
    initialize_git_fixture(&repo_dir, package_name);
    Ok(repo_dir)
}

fn initialize_git_fixture(repo_dir: &Path, package_name: &str) {
    init_git_repository(repo_dir);
    let remote_url = format!(
        "https://example.invalid/xiuxian-wendao/{}.git",
        package_name.to_ascii_lowercase()
    );
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["remote", "add", "origin", remote_url.as_str()])
        .output()
        .unwrap_or_else(|error| panic!("add git remote: {error}"));
    assert!(
        output.status.success(),
        "add git remote failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    commit_all(repo_dir, "initial import");
}
