#![cfg(feature = "zhenfa-router")]

use crate as xiuxian_wendao;

use std::collections::BTreeMap;
use std::fs;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

use xiuxian_git_repo::{SyncMode, discover_checkout_metadata};
use xiuxian_wendao::analyzers::cache::{
    build_repository_analysis_cache_key, store_cached_repository_analysis,
};
use xiuxian_wendao::analyzers::resolve_registered_repository_source;
use xiuxian_wendao::analyzers::{
    DocRecord, DocsProjectedGapReportQuery, ExampleRecord, ProjectedPageIndexNode,
    ProjectionPageKind, RefineEntityDocRequest, RegisteredRepository,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, RepoSymbolKind,
    RepositoryAnalysisOutput, RepositoryPluginConfig, RepositoryRecord, RepositoryRefreshPolicy,
    SymbolRecord, analyze_registered_repository_with_registry, build_projected_pages,
    docs_projected_gap_report_from_config, load_repo_intelligence_config,
    repo_projected_page_index_trees_from_config, repo_projected_pages_from_config,
};
use xiuxian_wendao::analyzers::{ModuleRecord, RelationKind, RelationRecord};
use xiuxian_wendao::gateway::studio::symbol_index::SymbolIndexCoordinator;
use xiuxian_wendao::gateway::studio::test_support::{
    add_git_remote, assert_studio_json_snapshot, commit_all, init_git_repository,
};
use xiuxian_wendao::gateway::studio::types::{UiConfig, UiRepoProjectConfig};
use xiuxian_wendao::gateway::studio::{GatewayState, StudioState, studio_router};
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::repo_index::{RepoIndexCoordinator, RepoIndexRequest};
use xiuxian_wendao::search::{SearchPlaneService, publish_repo_entities};
use xiuxian_wendao_julia::{
    integration_support::{
        JuliaExampleServiceGuard, spawn_wendaosearch_modelica_parser_summary_service,
    },
    set_linked_modelica_parser_summary_base_url_for_tests,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type LocalProjectMetadata = (Option<String>, Option<String>, Option<String>);

struct LinkedModelicaParserSummaryService {
    _guard: Mutex<JuliaExampleServiceGuard>,
}

static LINKED_MODELICA_PARSER_SUMMARY_SERVICE: OnceLock<
    Result<LinkedModelicaParserSummaryService, String>,
> = OnceLock::new();

async fn request_json(
    router: axum::Router,
    uri: &str,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload = serde_json::from_slice(&body)?;
    Ok((status, payload))
}

async fn request_json_post<T: serde::Serialize>(
    router: axum::Router,
    uri: &str,
    payload: &T,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(payload)?))?,
        )
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload = serde_json::from_slice(&body)?;
    Ok((status, payload))
}

mod gap_reports;
mod planner;

fn page_matches_needle(page: &serde_json::Map<String, Value>, needle: &str) -> bool {
    let title = page
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let page_id = page
        .get("page_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    title.contains(needle) || page_id.contains(needle)
}

fn node_matches_needle(node: &serde_json::Map<String, Value>, needle: &str) -> bool {
    let node_title = node
        .get("node_title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let page_title = node
        .get("page_title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    node_title.contains(needle) || page_title.contains(needle)
}

#[tokio::test]
async fn repo_overview_endpoint_returns_repo_summary_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) =
        request_json(router, "/api/repo/overview?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    redact_repo_overview_payload(&mut payload);
    assert_studio_json_snapshot("repo_overview_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_overview_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-overview]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) =
        request_json(router, "/api/repo/overview?repo=modelica-gateway-overview").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-overview")
    );
    assert_eq!(
        payload.get("display_name").and_then(Value::as_str),
        Some("Projectionica")
    );
    assert!(
        payload
            .get("module_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 1),
        "repo-overview endpoint should expose at least one Modelica module over the external plugin path"
    );
    redact_repo_overview_payload(&mut payload);
    assert_studio_json_snapshot("repo_overview_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_module_search_endpoint_returns_module_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_module_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_module_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-module-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/module-search?repo=modelica-gateway-module-search&query=Projectionica.Controllers&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-module-search")
    );
    let modules = payload
        .get("modules")
        .and_then(Value::as_array)
        .ok_or("repo-module-search payload should include a modules array")?;
    assert!(
        !modules.is_empty(),
        "repo-module-search endpoint should return at least one module over the external Modelica path"
    );
    assert!(
        modules.len() <= 3,
        "repo-module-search endpoint should honor the configured module limit"
    );
    assert!(
        modules.iter().any(|module| {
            module
                .get("qualified_name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.contains("Projectionica.Controllers"))
        }),
        "repo-module-search endpoint should keep module hits anchored to the requested Modelica namespace"
    );
    assert_studio_json_snapshot("repo_module_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_symbol_search_endpoint_returns_symbol_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_symbol_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_symbol_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-symbol-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/symbol-search?repo=modelica-gateway-symbol-search&query=PI&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-symbol-search")
    );
    let symbols = payload
        .get("symbols")
        .and_then(Value::as_array)
        .ok_or("repo-symbol-search payload should include a symbols array")?;
    assert!(
        !symbols.is_empty(),
        "repo-symbol-search endpoint should return at least one symbol over the external Modelica path"
    );
    assert!(
        symbols.len() <= 3,
        "repo-symbol-search endpoint should honor the configured symbol limit"
    );
    assert!(
        symbols.iter().any(|symbol| {
            symbol
                .get("qualified_name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.contains("Projectionica.Controllers.PI"))
        }),
        "repo-symbol-search endpoint should keep symbol hits anchored to the requested Modelica symbol"
    );
    assert_studio_json_snapshot("repo_symbol_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_example_search_endpoint_returns_example_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_example_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_example_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-example-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/example-search?repo=modelica-gateway-example-search&query=Step&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-example-search")
    );
    let examples = payload
        .get("examples")
        .and_then(Value::as_array)
        .ok_or("repo-example-search payload should include an examples array")?;
    assert!(
        !examples.is_empty(),
        "repo-example-search endpoint should return at least one example over the external Modelica path"
    );
    assert!(
        examples.len() <= 3,
        "repo-example-search endpoint should honor the configured example limit"
    );
    assert!(
        examples.iter().any(|example| {
            let title = example
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let path = example
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or_default();
            title.contains("Step") || path.contains("Step.mo")
        }),
        "repo-example-search endpoint should keep example hits anchored to the requested Modelica example"
    );
    assert_studio_json_snapshot("repo_example_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_cached_search_endpoints_return_pending_without_ready_analysis_cache() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    write_default_repo_config_without_priming(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    for uri in [
        "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5",
        "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5",
        "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::CONFLICT, "{uri}");
        assert_eq!(payload["code"], "REPO_INDEX_PENDING", "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn repo_cached_search_endpoints_can_serve_from_published_repo_entity_search_plane()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let state = gateway_state_for_project_with_options(temp.path(), false, false);
    publish_repo_entity_search_plane(state.as_ref(), temp.path(), "gateway-sync").await?;
    let router = studio_router(state);

    let (module_status, module_payload) = request_json(
        router.clone(),
        "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5",
    )
    .await?;
    assert_eq!(module_status, StatusCode::OK);
    assert_eq!(module_payload["repo_id"], "gateway-sync");
    assert_eq!(
        module_payload["modules"][0]["qualified_name"],
        "GatewaySyncPkg"
    );
    assert_eq!(
        module_payload["module_hits"][0]["module"]["module_id"],
        "repo:gateway-sync:module:GatewaySyncPkg"
    );

    let (symbol_status, symbol_payload) = request_json(
        router.clone(),
        "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(symbol_status, StatusCode::OK);
    assert_eq!(symbol_payload["repo_id"], "gateway-sync");
    assert_eq!(symbol_payload["symbols"][0]["name"], "solve");
    assert_eq!(
        symbol_payload["symbol_hits"][0]["audit_status"],
        "unreviewed"
    );

    let (example_status, example_payload) = request_json(
        router,
        "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5",
    )
    .await?;
    assert_eq!(example_status, StatusCode::OK);
    assert_eq!(example_payload["repo_id"], "gateway-sync");
    assert_eq!(example_payload["examples"][0]["title"], "solve_demo");
    assert_eq!(example_payload["example_hits"][0]["rank"], 1);
    Ok(())
}

#[tokio::test]
async fn repo_doc_coverage_endpoint_returns_coverage_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    fs::write(repo_dir.join("docs").join("Problem.md"), "# Problem\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/doc-coverage?repo=gateway-sync&module=GatewaySyncPkg",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_doc_coverage_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_doc_coverage_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-doc-coverage]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/doc-coverage?repo=modelica-gateway-doc-coverage&module=Projectionica.Controllers",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-doc-coverage")
    );
    assert!(
        payload
            .get("module_id")
            .and_then(Value::as_str)
            .is_some_and(|module_id| module_id.contains("Projectionica.Controllers")),
        "repo-doc-coverage endpoint should stay anchored to the requested Modelica module"
    );
    let docs = payload
        .get("docs")
        .and_then(Value::as_array)
        .ok_or("repo-doc-coverage payload should include a docs array")?;
    assert!(
        !docs.is_empty(),
        "repo-doc-coverage endpoint should expose at least one documentation record over the external Modelica path"
    );
    assert_studio_json_snapshot("repo_doc_coverage_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_endpoint_returns_status_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let payload = RepoIndexRequest {
        repo: Some("gateway-sync".to_string()),
        refresh: false,
    };
    let (status, mut payload) = request_json_post(router, "/api/repo/index", &payload).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    assert_eq!(payload.get("queued").and_then(Value::as_u64), Some(1));
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(temp.path(), &repo_dir, "modelica-gateway-index")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let payload = RepoIndexRequest {
        repo: Some("modelica-gateway-index".to_string()),
        refresh: false,
    };
    let (status, mut payload) = request_json_post(router, "/api/repo/index", &payload).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    assert_eq!(payload.get("queued").and_then(Value::as_u64), Some(1));
    let repos = payload
        .get("repos")
        .and_then(Value::as_array)
        .ok_or("repo-index payload should include a repos array")?;
    assert!(
        repos.iter().any(|repo| {
            repo.get("repoId")
                .and_then(Value::as_str)
                .is_some_and(|repo_id| repo_id == "modelica-gateway-index")
        }),
        "repo-index endpoint should queue the requested external Modelica repository",
    );
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_status_endpoint_returns_status_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) =
        request_json(router, "/api/repo/index/status?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingEnabled"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingMode"),
        Some(&Value::String("deferred".to_string()))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationObserved"),
        Some(&Value::Bool(true))
    );
    assert!(
        payload
            .get("studioBootstrapBackgroundIndexingDeferredActivationAt")
            .and_then(Value::as_str)
            .is_some()
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationSource"),
        Some(&Value::String("repo_index_status".to_string()))
    );
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_status_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_index_status_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(temp.path(), &repo_dir, "modelica-gateway-index-status")?;
    let router = studio_router(gateway_state_for_project_with_options(
        temp.path(),
        false,
        false,
    ));

    let enqueue_payload = RepoIndexRequest {
        repo: Some("modelica-gateway-index-status".to_string()),
        refresh: false,
    };
    let (enqueue_status, _) =
        request_json_post(router.clone(), "/api/repo/index", &enqueue_payload).await?;
    assert_eq!(enqueue_status, StatusCode::OK);

    let (status, mut payload) = request_json(
        router,
        "/api/repo/index/status?repo=modelica-gateway-index-status",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingEnabled"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingMode"),
        Some(&Value::String("deferred".to_string()))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationObserved"),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationAt"),
        Some(&Value::Null)
    );
    assert_eq!(
        payload.get("studioBootstrapBackgroundIndexingDeferredActivationSource"),
        Some(&Value::Null)
    );
    assert_eq!(payload.get("total").and_then(Value::as_u64), Some(1));
    let repos = payload
        .get("repos")
        .and_then(Value::as_array)
        .ok_or("repo-index-status payload should include a repos array")?;
    assert_eq!(repos.len(), 1);
    assert_eq!(
        repos[0].get("repoId").and_then(Value::as_str),
        Some("modelica-gateway-index-status")
    );
    redact_repo_index_payload(&mut payload);
    assert_studio_json_snapshot("repo_index_status_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_refine_entity_doc_endpoint_returns_refined_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let state = gateway_state_for_project_with_options(temp.path(), false, false);
    publish_repo_entity_search_plane(state.as_ref(), temp.path(), "gateway-sync").await?;
    let router = studio_router(state);

    let payload = RefineEntityDocRequest {
        repo_id: "gateway-sync".to_string(),
        entity_id: "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
        user_hints: Some("Explain how callers should use this entrypoint.".to_string()),
    };
    let (status, payload) = request_json_post(router, "/api/analysis/refine-doc", &payload).await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_refine_entity_doc_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_sync_endpoint_returns_repo_status_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) =
        request_json(router, "/api/repo/sync?repo=gateway-sync&mode=status").await?;
    assert_eq!(status, StatusCode::OK);
    redact_repo_sync_payload(&mut payload);
    assert_studio_json_snapshot("repo_sync_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_sync_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-sync]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, mut payload) = request_json(
        router,
        "/api/repo/sync?repo=modelica-gateway-sync&mode=status",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-sync")
    );
    assert_eq!(payload.get("mode").and_then(Value::as_str), Some("status"));
    redact_repo_sync_payload(&mut payload);
    assert_studio_json_snapshot("repo_sync_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_pages_endpoint_returns_projection_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) =
        request_json(router, "/api/repo/projected-pages?repo=gateway-sync").await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_pages_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_pages_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-pages]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-pages?repo=modelica-gateway-projected-pages",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let pages = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or("repo-projected-pages payload should include a pages array")?;
    assert!(
        !pages.is_empty(),
        "repo-projected-pages endpoint should return at least one projected page over the external Modelica path"
    );
    assert!(
        pages.iter().any(|page| {
            let title = page
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let page_id = page
                .get("page_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            title.contains("Projectionica.Controllers")
                || title.contains("Step")
                || page_id.contains("Projectionica.Controllers")
        }),
        "repo-projected-pages endpoint should keep projected pages anchored to the external Modelica namespace"
    );
    assert_studio_json_snapshot("repo_projected_pages_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_item_endpoint_returns_gap_bundle() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("orphan.md"), "# orphan\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/planner-item?repo=gateway-sync&gap_id=repo:gateway-sync:projection-gap:documentation_page_without_anchor:repo:gateway-sync:doc:docs/orphan.md&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_planner_item_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_planner_item_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        repo_dir.join("Controllers").join("NoDocs.mo"),
        "within Projectionica.Controllers;\nmodel NoDocs\nend NoDocs;\n",
    )?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-item]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let gap_report = docs_projected_gap_report_from_config(
        &DocsProjectedGapReportQuery {
            repo_id: "modelica-gateway-item".to_string(),
        },
        Some(&temp.path().join("wendao.toml")),
        temp.path(),
    )?;
    let gap = gap_report
        .gaps
        .first()
        .cloned()
        .ok_or("planner-item route expected at least one projected gap")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/planner-item?repo=modelica-gateway-item&gap_id={}&family_kind=how_to&related_limit=3&family_limit=3",
            gap.gap_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let route_gap = payload
        .get("gap")
        .and_then(Value::as_object)
        .ok_or("planner-item payload should include a gap object")?;
    let route_gap_id = route_gap
        .get("gap_id")
        .and_then(Value::as_str)
        .ok_or("planner-item payload should include gap.gap_id")?;
    let route_page_id = route_gap
        .get("page_id")
        .and_then(Value::as_str)
        .ok_or("planner-item payload should include gap.page_id")?;
    let route_title = route_gap
        .get("title")
        .and_then(Value::as_str)
        .ok_or("planner-item payload should include gap.title")?;
    assert_eq!(
        route_gap_id, gap.gap_id,
        "planner-item route should reopen the requested stable gap"
    );
    assert!(
        route_title.contains("NoDocs") || route_page_id.contains("NoDocs"),
        "planner-item route should stay anchored to the injected no-doc target"
    );
    assert_eq!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(route_page_id),
        "planner-item route retrieval hit should stay anchored to the gap page"
    );
    assert_eq!(
        payload
            .get("navigation")
            .and_then(Value::as_object)
            .and_then(|navigation| navigation.get("center"))
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(route_page_id),
        "planner-item route navigation center should stay anchored to the gap page"
    );
    assert_studio_json_snapshot("docs_planner_item_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_search_endpoint_returns_projection_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/search?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/search?repo=modelica-gateway-search&query=Projectionica.Controllers&kind=reference&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let pages = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or("docs-search payload should include a pages array")?;
    assert!(
        !pages.is_empty(),
        "docs-search endpoint should return at least one projected page"
    );
    assert!(
        pages.len() <= 3,
        "docs-search endpoint should honor the configured hit limit"
    );
    assert!(
        pages.iter().all(|page| {
            page.as_object()
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"))
        }),
        "docs-search endpoint pages should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_endpoint_returns_mixed_hits() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/retrieval?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_retrieval_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-retrieval]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/retrieval?repo=modelica-gateway-retrieval&query=Projectionica.Controllers&kind=reference&limit=4",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("docs-retrieval payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "docs-retrieval endpoint should return at least one mixed retrieval hit"
    );
    assert!(
        hits.len() <= 4,
        "docs-retrieval endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "page")
        }),
        "docs-retrieval endpoint should preserve page hits over the external Modelica path"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "page_index_node")
        }),
        "docs-retrieval endpoint should preserve page-index node hits over the external Modelica path"
    );
    assert!(
        hits.iter().all(|hit| {
            let page_anchor = hit
                .get("page")
                .and_then(Value::as_object)
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"));
            let node_anchor = hit
                .get("node")
                .and_then(Value::as_object)
                .is_none_or(|node| node_matches_needle(node, "Projectionica.Controllers"));
            page_anchor && node_anchor
        }),
        "docs-retrieval endpoint hits should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_retrieval_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_context_endpoint_returns_node_context_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/retrieval-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors&related_limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_retrieval_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_context_endpoint_executes_over_external_modelica_plugin_path() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-retrieval-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-retrieval-context".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-retrieval-context".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/retrieval-context?repo=modelica-gateway-retrieval-context&page_id={}&node_id={}&related_limit=3",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-retrieval-context endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("node_context")
            .and_then(Value::as_object)
            .is_some(),
        "docs-retrieval-context endpoint should include node context when reopening a Modelica page-index node"
    );
    assert_studio_json_snapshot("docs_retrieval_context_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_hit_endpoint_returns_hit_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/retrieval-hit?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_retrieval_hit_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_retrieval_hit_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-retrieval-hit]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-retrieval-hit".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-retrieval-hit".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/retrieval-hit?repo=modelica-gateway-retrieval-hit&page_id={}&node_id={}",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "page_index_node"),
        "docs-retrieval-hit endpoint should reopen the requested Modelica page-index node as a node hit"
    );
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("page"))
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-retrieval-hit endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("node"))
            .and_then(Value::as_object)
            .and_then(|node| node.get("node_id"))
            .and_then(Value::as_str)
            .is_some_and(|returned_node_id| returned_node_id == node_id),
        "docs-retrieval-hit endpoint should reopen the requested Modelica page-index node"
    );
    assert_studio_json_snapshot("docs_retrieval_hit_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_endpoint_returns_projection_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/page?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_page_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_index_tree_endpoint_returns_tree_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/page-index-tree?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_page_index_tree_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-page]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-page".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/page?repo=modelica-gateway-page&page_id={}",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("page")
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-page endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("page")
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("title"))
            .and_then(Value::as_str)
            .is_some_and(|title| title == "Projectionica.Controllers.PI"),
        "docs-page endpoint should reopen the requested Modelica projected page title"
    );
    assert_studio_json_snapshot("docs_page_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_page_endpoint_uses_studio_state_without_persisted_config_file() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;

    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap builtin plugin registry: {error}")),
    );
    let repository = RegisteredRepository {
        id: "gateway-sync".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    let analysis =
        analyze_registered_repository_with_registry(&repository, temp.path(), &plugin_registry)?;
    let page_id = build_projected_pages(&analysis)
        .into_iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .map(|page| page.page_id)
        .ok_or("expected a projected symbol-backed page for `Projectionica.Controllers.PI`")?;

    let router = studio_router(gateway_state_for_ui_config(
        temp.path(),
        UiConfig {
            projects: Vec::new(),
            repo_projects: vec![UiRepoProjectConfig {
                id: "gateway-sync".to_string(),
                root: Some(repo_dir.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["modelica".to_string()],
            }],
        },
        plugin_registry,
    ));

    let (status, payload) = request_json(
        router,
        &format!("/api/docs/page?repo=gateway-sync&page_id={page_id}"),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload["page"]["title"].as_str(),
        Some("Projectionica.Controllers.PI")
    );
    Ok(())
}

#[tokio::test]
async fn docs_family_context_endpoint_returns_family_context() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-context?repo=gateway-sync&page_id={}&per_kind_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_family_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_context_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-family-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-family-context".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-context?repo=modelica-gateway-family-context&page_id={}&per_kind_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center_page")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-family-context endpoint should stay anchored to the requested Modelica how-to page"
    );
    let families = payload
        .get("families")
        .and_then(Value::as_array)
        .ok_or("docs-family-context payload should include a families array")?;
    assert!(
        !families.is_empty(),
        "docs-family-context endpoint should return at least one related family over the external Modelica path"
    );
    assert_studio_json_snapshot("docs_family_context_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_search_endpoint_returns_family_clusters() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/family-search?repo=gateway-sync&query=solve&kind=reference&limit=5&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_family_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-family-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/family-search?repo=modelica-gateway-family-search&query=Projectionica.Controllers&kind=reference&limit=3&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("docs-family-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "docs-family-search endpoint should return at least one family-search hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 3,
        "docs-family-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().all(|hit| {
            hit.get("center_page")
                .and_then(Value::as_object)
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"))
        }),
        "docs-family-search endpoint hits should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_family_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_cluster_endpoint_returns_requested_cluster() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-cluster?repo=gateway-sync&page_id={}&kind=reference&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_family_cluster_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_family_cluster_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-family-cluster]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-family-cluster".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference && page.title == "Projectionica.Controllers"
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a module-backed projected reference page titled `Projectionica.Controllers`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/family-cluster?repo=modelica-gateway-family-cluster&page_id={}&kind=how_to&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center_page")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-family-cluster endpoint should stay anchored to the requested Modelica reference page"
    );
    assert!(
        payload
            .get("family")
            .and_then(Value::as_object)
            .and_then(|family| family.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "how_to"),
        "docs-family-cluster endpoint should reopen the requested Modelica how-to family cluster"
    );
    assert_studio_json_snapshot("docs_family_cluster_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_navigation_endpoint_returns_navigation_bundle() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/navigation?repo=gateway-sync&page_id={}&node_id={}&family_kind=how_to&related_limit=3&family_limit=2",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_navigation_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_navigation_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-navigation]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-navigation".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-navigation".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/docs/navigation?repo=modelica-gateway-navigation&page_id={}&node_id={}&family_kind=how_to&related_limit=3&family_limit=2",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("center")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "docs-navigation endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("family_cluster")
            .and_then(Value::as_object)
            .and_then(|family| family.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "how_to"),
        "docs-navigation endpoint should reopen the requested Modelica how-to family cluster"
    );
    assert!(
        payload
            .get("node_context")
            .and_then(Value::as_object)
            .is_some(),
        "docs-navigation endpoint should include node context over the external Modelica path"
    );
    assert_studio_json_snapshot("docs_navigation_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_navigation_search_endpoint_returns_navigation_hits() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/navigation-search?repo=gateway-sync&query=solve&kind=reference&family_kind=how_to&limit=5&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("docs_navigation_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn docs_navigation_search_endpoint_executes_over_external_modelica_plugin_path() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-navigation-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/docs/navigation-search?repo=modelica-gateway-navigation-search&query=Projectionica.Controllers&kind=reference&family_kind=how_to&limit=2&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("docs-navigation-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "docs-navigation-search endpoint should return at least one navigation hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 2,
        "docs-navigation-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().all(|hit| {
            hit.get("navigation")
                .and_then(Value::as_object)
                .and_then(|navigation| navigation.get("center"))
                .and_then(Value::as_object)
                .and_then(|center| center.get("page"))
                .and_then(Value::as_object)
                .is_some_and(|page| page_matches_needle(page, "Projectionica.Controllers"))
        }),
        "docs-navigation-search endpoint hits should stay anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("docs_navigation_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_endpoint_returns_projection_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_endpoint_executes_over_external_modelica_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-page]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-page".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page?repo=modelica-gateway-projected-page&page_id={}",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("page")
            .and_then(Value::as_object)
            .and_then(|page_value| page_value.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "repo-projected-page endpoint should stay anchored to the requested Modelica projected page"
    );
    assert_studio_json_snapshot("repo_projected_page_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_endpoint_returns_tree_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-tree?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_index_tree_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-index-tree]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-index-tree".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-index-tree?repo=modelica-gateway-projected-index-tree&page_id={}",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("tree")
            .and_then(Value::as_object)
            .and_then(|tree| tree.get("page_id"))
            .and_then(Value::as_str)
            .is_some_and(|page_id| page_id == page.page_id),
        "repo-projected-page-index-tree endpoint should stay anchored to the requested Modelica projected page"
    );
    assert!(
        payload
            .get("tree")
            .and_then(Value::as_object)
            .and_then(|tree| tree.get("roots"))
            .and_then(Value::as_array)
            .is_some_and(|roots| !roots.is_empty()),
        "repo-projected-page-index-tree endpoint should reopen a non-empty tree over the external Modelica path"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_tree_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_returns_node_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-node?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_index_node_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-index-node]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-index-node".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-projected-index-node".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-index-node?repo=modelica-gateway-projected-index-node&page_id={}&node_id={}",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("node_id"))
            .and_then(Value::as_str)
            .is_some_and(|returned_node_id| returned_node_id == node_id),
        "repo-projected-page-index-node endpoint should reopen the requested Modelica page-index node"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_node_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_search_endpoint_returns_hit_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-tree-search?repo=gateway-sync&query=anchors&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot(
        "repo_projected_page_index_tree_search_endpoint_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-index-tree-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-tree-search?repo=modelica-gateway-projected-index-tree-search&query=anchors&kind=reference&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-index-tree-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-page-index-tree-search endpoint should return at least one section hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 3,
        "repo-projected-page-index-tree-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("page_title")
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-index-tree-search endpoint should keep section hits anchored to the requested Modelica controller family"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_tree_search_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_search_endpoint_returns_projection_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-search?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-page-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-search?repo=modelica-gateway-projected-page-search&query=Projectionica.Controllers&kind=reference&limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let pages = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-search payload should include a pages array")?;
    assert!(
        !pages.is_empty(),
        "repo-projected-page-search endpoint should return at least one projected page over the external Modelica path"
    );
    assert!(
        pages.len() <= 3,
        "repo-projected-page-search endpoint should honor the configured page limit"
    );
    assert!(
        pages.iter().any(|page| {
            page.get("title")
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-search endpoint should keep page hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("repo_projected_page_search_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_endpoint_returns_mixed_hit_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval?repo=gateway-sync&query=solve&kind=reference&limit=5",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_retrieval_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-retrieval",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval?repo=modelica-gateway-projected-retrieval&query=Projectionica.Controllers&kind=reference&limit=4",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-retrieval payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-retrieval endpoint should return at least one mixed hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 4,
        "repo-projected-retrieval endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("page")
                .and_then(Value::as_object)
                .and_then(|page| page.get("title"))
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-retrieval endpoint should keep mixed hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot("repo_projected_retrieval_endpoint_modelica_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_hit_endpoint_returns_page_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval-hit?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_retrieval_hit_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_hit_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-retrieval-hit]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-retrieval-hit".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-projected-retrieval-hit".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-retrieval-hit?repo=modelica-gateway-projected-retrieval-hit&page_id={}&node_id={}",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("hit")
            .and_then(Value::as_object)
            .and_then(|hit| hit.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "page_index_node"),
        "repo-projected-retrieval-hit endpoint should reopen the requested Modelica page-index node as a node hit"
    );
    assert_studio_json_snapshot(
        "repo_projected_retrieval_hit_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_context_endpoint_returns_node_context_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors&related_limit=3",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_retrieval_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_context_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        temp.path().join("wendao.toml"),
        format!(
            r#"[link_graph.projects.modelica-gateway-projected-retrieval-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-gateway-projected-retrieval-context".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "Projectionica.Controllers.PI"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-gateway-projected-retrieval-context".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-retrieval-context?repo=modelica-gateway-projected-retrieval-context&page_id={}&node_id={}&related_limit=3",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        payload
            .get("node_context")
            .and_then(Value::as_object)
            .is_some(),
        "repo-projected-retrieval-context endpoint should include node context when reopening a Modelica page-index node"
    );
    assert_studio_json_snapshot(
        "repo_projected_retrieval_context_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_context_endpoint_returns_family_clusters() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-context?repo=gateway-sync&page_id={}&per_kind_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_family_context_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_context_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-family-context",
    )?;
    let page_id = projected_page_id_for_title(
        temp.path(),
        "modelica-gateway-projected-family-context",
        ProjectionPageKind::HowTo,
        "Step",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-context?repo=modelica-gateway-projected-family-context&page_id={page_id}&per_kind_limit=2"
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload
            .get("center_page")
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(page_id.as_str())
    );
    assert_studio_json_snapshot(
        "repo_projected_page_family_context_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_search_endpoint_returns_family_clusters() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-family-search?repo=gateway-sync&query=solve&kind=reference&limit=5&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_family_search_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-family-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-family-search?repo=modelica-gateway-projected-family-search&query=Projectionica.Controllers&kind=reference&limit=3&per_kind_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-family-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-page-family-search endpoint should return at least one family hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 3,
        "repo-projected-page-family-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("center_page")
                .and_then(Value::as_object)
                .and_then(|page| page.get("title"))
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-family-search endpoint should keep family hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_family_search_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_cluster_endpoint_returns_family_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id={}&kind=how_to&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_family_cluster_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_cluster_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-family-cluster",
    )?;
    let page_id = projected_page_id_for_title(
        temp.path(),
        "modelica-gateway-projected-family-cluster",
        ProjectionPageKind::Reference,
        "Projectionica.Controllers",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-cluster?repo=modelica-gateway-projected-family-cluster&page_id={page_id}&kind=how_to&limit=2"
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload
            .get("center_page")
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(page_id.as_str())
    );
    assert_studio_json_snapshot(
        "repo_projected_page_family_cluster_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_endpoint_returns_navigation_bundle() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));
    let encoded_node_id = node_id.replace('#', "%23");
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-navigation?repo=gateway-sync&page_id={}&node_id={}&family_kind=how_to&related_limit=3&family_limit=2",
            page.page_id, encoded_node_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_navigation_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-navigation",
    )?;
    let (page_id, node_id) = projected_page_and_node_id_for_title(
        temp.path(),
        "modelica-gateway-projected-navigation",
        "Projectionica.Controllers.PI",
        "Anchors",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-navigation?repo=modelica-gateway-projected-navigation&page_id={}&node_id={}&family_kind=how_to&related_limit=3&family_limit=2",
            page_id,
            node_id.replace('#', "%23")
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload
            .get("center")
            .and_then(Value::as_object)
            .and_then(|center| center.get("page"))
            .and_then(Value::as_object)
            .and_then(|page| page.get("page_id"))
            .and_then(Value::as_str),
        Some(page_id.as_str())
    );
    assert_studio_json_snapshot(
        "repo_projected_page_navigation_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_search_endpoint_returns_navigation_hits() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-navigation-search?repo=gateway-sync&query=solve&kind=reference&family_kind=how_to&limit=5&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot(
        "repo_projected_page_navigation_search_endpoint_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_search_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-navigation-search",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-navigation-search?repo=modelica-gateway-projected-navigation-search&query=Projectionica.Controllers&kind=reference&family_kind=how_to&limit=2&related_limit=3&family_limit=2",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-navigation-search payload should include a hits array")?;
    assert!(
        !hits.is_empty(),
        "repo-projected-page-navigation-search endpoint should return at least one navigation hit over the external Modelica path"
    );
    assert!(
        hits.len() <= 2,
        "repo-projected-page-navigation-search endpoint should honor the configured hit limit"
    );
    assert!(
        hits.iter().any(|hit| {
            hit.get("navigation")
                .and_then(Value::as_object)
                .and_then(|navigation| navigation.get("center"))
                .and_then(Value::as_object)
                .and_then(|center| center.get("page"))
                .and_then(Value::as_object)
                .and_then(|page| page.get("title"))
                .and_then(Value::as_str)
                .is_some_and(|title| title.contains("Projectionica.Controllers"))
        }),
        "repo-projected-page-navigation-search endpoint should keep navigation hits anchored to the requested Modelica controller path"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_navigation_search_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_trees_endpoint_returns_tree_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-trees?repo=gateway-sync",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_studio_json_snapshot("repo_projected_page_index_trees_endpoint_json", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_trees_endpoint_executes_over_external_modelica_plugin_path()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    write_modelica_repo_config(
        temp.path(),
        &repo_dir,
        "modelica-gateway-projected-index-trees",
    )?;
    let selected_page_id = projected_page_id_for_title(
        temp.path(),
        "modelica-gateway-projected-index-trees",
        ProjectionPageKind::Reference,
        "Projectionica.Controllers.PI",
    )?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-trees?repo=modelica-gateway-projected-index-trees",
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload.get("repo_id").and_then(Value::as_str),
        Some("modelica-gateway-projected-index-trees")
    );
    let trees = payload
        .get("trees")
        .and_then(Value::as_array)
        .ok_or("repo-projected-page-index-trees payload should include a trees array")?;
    assert!(
        !trees.is_empty(),
        "repo-projected-page-index-trees endpoint should return at least one projected tree over the external Modelica path"
    );
    assert!(
        trees.iter().any(|tree| {
            tree.get("page_id")
                .and_then(Value::as_str)
                .is_some_and(|page_id| page_id == selected_page_id)
        }),
        "repo-projected-page-index-trees endpoint should include the projected tree for the selected Modelica symbol page"
    );
    assert_studio_json_snapshot(
        "repo_projected_page_index_trees_endpoint_modelica_json",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_gateway_returns_missing_repo_error() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/docs/page-index-tree?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/overview",
        "/api/repo/module-search?query=solve",
        "/api/repo/symbol-search?query=solve",
        "/api/repo/example-search?query=solve",
        "/api/repo/projected-page?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-index-node?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23anchors",
        "/api/repo/projected-retrieval-hit?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-retrieval-context?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-family-context?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-family-search?query=solve",
        "/api/repo/projected-page-family-cluster?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&kind=reference",
        "/api/repo/projected-page-navigation?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-navigation-search?query=solve",
        "/api/repo/projected-page-index-tree?page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "/api/repo/projected-page-index-tree-search?query=anchors",
        "/api/repo/projected-page-search?query=solve",
        "/api/repo/projected-retrieval?query=solve",
        "/api/repo/doc-coverage",
        "/api/repo/sync",
        "/api/repo/projected-pages",
        "/api/repo/projected-page-index-trees",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_gateway_missing_repo_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_gateway_search_endpoints_require_query_param() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/repo/module-search?repo=gateway-sync",
        "/api/repo/symbol-search?repo=gateway-sync",
        "/api/repo/example-search?repo=gateway-sync",
        "/api/repo/projected-page-index-tree-search?repo=gateway-sync",
        "/api/repo/projected-page-search?repo=gateway-sync",
        "/api/repo/projected-page-family-search?repo=gateway-sync",
        "/api/repo/projected-page-navigation-search?repo=gateway-sync",
        "/api/repo/projected-retrieval?repo=gateway-sync",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_gateway_missing_query_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_endpoint_requires_page_id() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/docs/page-index-tree?repo=gateway-sync",
        "/api/repo/projected-page?repo=gateway-sync",
        "/api/repo/projected-page-index-node?repo=gateway-sync&node_id=reference/solve-69592caeddee%23anchors",
        "/api/repo/projected-retrieval-hit?repo=gateway-sync",
        "/api/repo/projected-retrieval-context?repo=gateway-sync",
        "/api/repo/projected-page-family-context?repo=gateway-sync",
        "/api/repo/projected-page-family-cluster?repo=gateway-sync&kind=reference",
        "/api/repo/projected-page-navigation?repo=gateway-sync",
        "/api/repo/projected-page-index-tree?repo=gateway-sync",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_gateway_missing_page_id_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_requires_node_id() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-node?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_studio_json_snapshot("repo_gateway_missing_node_id_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_cluster_endpoint_requires_kind() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_studio_json_snapshot("repo_gateway_missing_kind_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_sync_endpoint_rejects_invalid_mode() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) =
        request_json(router, "/api/repo/sync?repo=gateway-sync&mode=bogus").await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_studio_json_snapshot("repo_sync_endpoint_invalid_mode_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_search_endpoint_rejects_invalid_kind() -> TestResult {
    let temp = tempfile::tempdir()?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    for uri in [
        "/api/repo/projected-page-search?repo=gateway-sync&query=solve&kind=bogus",
        "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&kind=bogus",
        "/api/repo/projected-page-family-search?repo=gateway-sync&query=solve&kind=bogus",
        "/api/repo/projected-page-navigation?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&family_kind=bogus",
        "/api/repo/projected-page-navigation-search?repo=gateway-sync&query=solve&family_kind=bogus",
        "/api/repo/projected-page-navigation-search?repo=gateway-sync&query=solve&kind=bogus",
        "/api/repo/projected-page-index-tree-search?repo=gateway-sync&query=anchors&kind=bogus",
        "/api/repo/projected-retrieval?repo=gateway-sync&query=solve&kind=bogus",
    ] {
        let (status, payload) = request_json(router.clone(), uri).await?;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_studio_json_snapshot("repo_projected_page_search_invalid_kind_error", payload);
    }
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_endpoint_returns_not_found_for_unknown_page() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/missing.md",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_tree_endpoint_returns_not_found_for_unknown_page() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-tree?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/missing.md",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_index_tree_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_index_node_endpoint_returns_not_found_for_unknown_node() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-index-node?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23missing",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_index_node_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_hit_endpoint_returns_not_found_for_unknown_node() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval-hit?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23missing",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_retrieval_hit_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_retrieval_context_endpoint_returns_not_found_for_unknown_node() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-retrieval-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md&node_id=reference/solve-69592caeddee%23missing",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_retrieval_context_not_found_error", payload);
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_context_endpoint_returns_not_found_for_unknown_page()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        "/api/repo/projected-page-family-context?repo=gateway-sync&page_id=repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/missing.md",
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot(
        "repo_projected_page_family_context_not_found_error",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_family_cluster_endpoint_returns_not_found_for_unknown_family()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-family-cluster?repo=gateway-sync&page_id={}&kind=tutorial&limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot(
        "repo_projected_page_family_cluster_not_found_error",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn repo_projected_page_navigation_endpoint_returns_not_found_for_unknown_family() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_git_repo(temp.path(), "GatewaySyncPkg")?;
    fs::write(
        repo_dir.join("src").join("GatewaySyncPkg.jl"),
        "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        "using GatewaySyncPkg\nsolve()\n",
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    write_default_repo_config(temp.path(), &repo_dir, "gateway-sync")?;
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "gateway-sync".to_string(),
        },
        None,
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "GatewaySyncPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a symbol-backed projected reference page titled `GatewaySyncPkg.solve`"
            )
        });
    let router = studio_router(gateway_state_for_project(temp.path()));

    let (status, payload) = request_json(
        router,
        &format!(
            "/api/repo/projected-page-navigation?repo=gateway-sync&page_id={}&family_kind=tutorial&family_limit=2",
            page.page_id
        ),
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_studio_json_snapshot("repo_projected_page_navigation_not_found_error", payload);
    Ok(())
}

fn gateway_state_for_project(project_root: &Path) -> Arc<GatewayState> {
    gateway_state_for_project_with_options(project_root, true, true)
}

fn gateway_state_for_ui_config(
    project_root: &Path,
    ui_config: UiConfig,
    plugin_registry: Arc<xiuxian_wendao::analyzers::PluginRegistry>,
) -> Arc<GatewayState> {
    let repo_index = Arc::new(RepoIndexCoordinator::new(
        project_root.to_path_buf(),
        Arc::clone(&plugin_registry),
        xiuxian_wendao::search::SearchPlaneService::new(project_root.to_path_buf()),
    ));
    repo_index.start();

    Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState {
            project_root: project_root.to_path_buf(),
            config_root: project_root.to_path_buf(),
            bootstrap_background_indexing: false,
            cold_start_process_started_at:
                xiuxian_wendao::gateway::studio::symbol_index::timestamp_now(),
            cold_start_process_started_instant: std::time::Instant::now(),
            cold_start_telemetry: Arc::new(RwLock::new(
                xiuxian_wendao::gateway::studio::router::StudioSearchColdStartTelemetryState::default(),
            )),
            bootstrap_background_indexing_deferred_activation: Arc::new(RwLock::new(None)),
            configured_owners: Arc::new(RwLock::new(
                StudioState::configured_owners_from_ui_config(ui_config),
            )),
            graph_index: Arc::new(RwLock::new(None)),
            symbol_index: Arc::new(RwLock::new(None)),
            symbol_index_coordinator: Arc::new(SymbolIndexCoordinator::new(
                project_root.to_path_buf(),
                project_root.to_path_buf(),
                SearchPlaneService::new(project_root.to_path_buf()),
            )),
            search_plane: SearchPlaneService::new(project_root.to_path_buf()),
            vfs_scan: Arc::new(RwLock::new(None)),
            repo_index,
            plugin_registry,
        }),
    })
}

async fn publish_repo_entity_search_plane(
    state: &GatewayState,
    project_root: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = project_root.join("wendao.toml");
    let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), project_root)?;
    let repository = repo_config
        .repos
        .iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| format!("missing repository `{repo_id}`"))?;
    let analysis = analyze_registered_repository_with_registry(
        repository,
        project_root,
        &state.studio.plugin_registry,
    )?;
    let repository_root = repository
        .path
        .as_ref()
        .ok_or_else(|| format!("repo `{repo_id}` missing path"))?;
    let documents = repo_code_documents(
        repository_root.as_path(),
        &["src/GatewaySyncPkg.jl", "examples/solve_demo.jl"],
    )?;
    publish_repo_entities(
        &state.studio.search_plane,
        repo_id,
        &analysis,
        documents.as_slice(),
        Some("test-rev"),
    )
    .await?;
    Ok(())
}

fn repo_code_documents(
    repo_root: &Path,
    relative_paths: &[&str],
) -> Result<Vec<RepoCodeDocument>, Box<dyn std::error::Error>> {
    let mut documents = Vec::new();
    for relative_path in relative_paths {
        let absolute_path = repo_root.join(relative_path);
        if !absolute_path.exists() {
            continue;
        }
        let metadata = fs::metadata(&absolute_path)?;
        let modified_unix_ms = u64::try_from(
            metadata.modified()?.duration_since(UNIX_EPOCH)?.as_millis(),
        )
        .map_err(|error| {
            std::io::Error::other(format!(
                "repo document modified time overflow for {}: {error}",
                absolute_path.display()
            ))
        })?;
        documents.push(RepoCodeDocument {
            path: (*relative_path).to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(fs::read_to_string(&absolute_path)?),
            size_bytes: metadata.len(),
            modified_unix_ms,
        });
    }
    Ok(documents)
}

fn gateway_state_for_project_with_options(
    project_root: &Path,
    start_repo_index: bool,
    prewarm_repo_analysis_cache: bool,
) -> Arc<GatewayState> {
    let config_root = project_root.to_path_buf();
    let ui_config =
        xiuxian_wendao::gateway::studio::router::load_ui_config_from_wendao_toml(&config_root)
            .unwrap_or_default();
    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap builtin plugin registry: {error}")),
    );
    let repo_index = Arc::new(RepoIndexCoordinator::new(
        project_root.to_path_buf(),
        Arc::clone(&plugin_registry),
        xiuxian_wendao::search::SearchPlaneService::new(project_root.to_path_buf()),
    ));
    if start_repo_index {
        repo_index.start();
    }
    let config_path = config_root.join("wendao.toml");
    if prewarm_repo_analysis_cache && config_path.exists() {
        let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), &config_root)
            .unwrap_or_else(|error| {
                panic!("load repo intelligence config for gateway tests: {error}")
            });
        for repository in &repo_config.repos {
            analyze_registered_repository_with_registry(
                repository,
                config_root.as_path(),
                &plugin_registry,
            )
            .unwrap_or_else(|error| {
                panic!("prewarm repository analysis cache for gateway tests: {error}")
            });
        }
    }

    Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState {
            project_root: project_root.to_path_buf(),
            config_root,
            bootstrap_background_indexing: false,
            cold_start_process_started_at:
                xiuxian_wendao::gateway::studio::symbol_index::timestamp_now(),
            cold_start_process_started_instant: std::time::Instant::now(),
            cold_start_telemetry: Arc::new(RwLock::new(
                xiuxian_wendao::gateway::studio::router::StudioSearchColdStartTelemetryState::default(),
            )),
            bootstrap_background_indexing_deferred_activation: Arc::new(RwLock::new(None)),
            configured_owners: Arc::new(RwLock::new(
                StudioState::configured_owners_from_ui_config(ui_config),
            )),
            graph_index: Arc::new(RwLock::new(None)),
            symbol_index: Arc::new(RwLock::new(None)),
            symbol_index_coordinator: Arc::new(SymbolIndexCoordinator::new(
                project_root.to_path_buf(),
                project_root.to_path_buf(),
                SearchPlaneService::new(project_root.to_path_buf()),
            )),
            search_plane: SearchPlaneService::new(project_root.to_path_buf()),
            vfs_scan: Arc::new(RwLock::new(None)),
            repo_index,
            plugin_registry,
        }),
    })
}

fn write_default_repo_config(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_default_repo_config_without_priming(base, repo_dir, repo_id)?;
    prime_local_julia_fixture_analysis_cache(base, repo_id)?;
    Ok(())
}

fn write_default_repo_config_without_priming(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        base.join("wendao.toml"),
        format!(
            r#"[link_graph.projects.{repo_id}]
root = "{}"
plugins = ["julia"]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(())
}

fn prime_local_julia_fixture_analysis_cache(
    project_root: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = project_root.join("wendao.toml");
    let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), project_root)?;
    let repository = repo_config
        .repos
        .iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| IoError::other(format!("missing repository `{repo_id}`")))?;
    let repository_source =
        resolve_registered_repository_source(repository, project_root, SyncMode::Status)?;
    let checkout_metadata = discover_checkout_metadata(repository_source.checkout_root.as_path());
    let cache_key = build_repository_analysis_cache_key(
        repository,
        &repository_source,
        checkout_metadata.as_ref(),
    );
    let analysis = build_local_julia_fixture_analysis(
        repo_id,
        repository_source.checkout_root.as_path(),
        checkout_metadata
            .as_ref()
            .and_then(|metadata| metadata.revision.clone()),
    )?;
    store_cached_repository_analysis(cache_key, &analysis)?;
    Ok(())
}

fn build_local_julia_fixture_analysis(
    repo_id: &str,
    repo_root: &Path,
    revision: Option<String>,
) -> Result<RepositoryAnalysisOutput, Box<dyn std::error::Error>> {
    let (project_name, version, uuid) = local_project_metadata(repo_root)?;
    let mut modules = Vec::new();
    let mut symbols = Vec::new();
    let mut docs = Vec::new();
    let mut relations = Vec::new();

    for relative_path in collect_relative_files_under(repo_root, "src", "jl")? {
        let contents = fs::read_to_string(repo_root.join(&relative_path))?;
        parse_local_julia_source(
            repo_id,
            &relative_path,
            &contents,
            &mut modules,
            &mut symbols,
            &mut docs,
            &mut relations,
        );
    }

    append_readme_doc_record(repo_id, repo_root, &modules, &mut docs, &mut relations);
    append_markdown_doc_records(
        repo_id,
        repo_root,
        &modules,
        &symbols,
        &mut docs,
        &mut relations,
    )?;
    let mut examples =
        collect_example_records(repo_id, repo_root, &modules, &symbols, &mut relations)?;
    sort_local_fixture_analysis_records(&mut modules, &mut symbols, &mut docs, &mut examples);
    Ok(RepositoryAnalysisOutput {
        repository: Some(build_local_fixture_repository_record(
            repo_id,
            repo_root,
            revision,
            project_name,
            version,
            uuid,
            &modules,
        )),
        modules,
        symbols,
        imports: Vec::new(),
        examples,
        docs,
        relations,
        diagnostics: Vec::new(),
    })
}

fn local_project_metadata(
    repo_root: &Path,
) -> Result<LocalProjectMetadata, Box<dyn std::error::Error>> {
    let project_toml = repo_root.join("Project.toml");
    if !project_toml.exists() {
        return Ok((None, None, None));
    }
    let contents = fs::read_to_string(project_toml)?;
    let mut name = None;
    let mut version = None;
    let mut uuid = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(value) = toml_string_value(trimmed, "name") {
            name = Some(value);
        } else if let Some(value) = toml_string_value(trimmed, "version") {
            version = Some(value);
        } else if let Some(value) = toml_string_value(trimmed, "uuid") {
            uuid = Some(value);
        }
    }
    Ok((name, version, uuid))
}

fn toml_string_value(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    line.strip_prefix(prefix.as_str())
        .and_then(|value| value.trim().strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(ToString::to_string)
}

fn collect_relative_files_under(
    repo_root: &Path,
    relative_root: &str,
    extension: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let root = repo_root.join(relative_root);
    let mut relative_paths = Vec::new();
    if !root.exists() {
        return Ok(relative_paths);
    }
    collect_relative_files_recursive(repo_root, root.as_path(), extension, &mut relative_paths)?;
    relative_paths.sort();
    Ok(relative_paths)
}

fn collect_relative_files_recursive(
    repo_root: &Path,
    current_dir: &Path,
    extension: &str,
    relative_paths: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_recursive(repo_root, path.as_path(), extension, relative_paths)?;
            continue;
        }
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == extension)
        {
            relative_paths.push(repo_relative_path(repo_root, path.as_path())?);
        }
    }
    Ok(())
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(path
        .strip_prefix(repo_root)?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn parse_local_julia_source(
    repo_id: &str,
    relative_path: &str,
    contents: &str,
    modules: &mut Vec<ModuleRecord>,
    symbols: &mut Vec<SymbolRecord>,
    docs: &mut Vec<DocRecord>,
    relations: &mut Vec<RelationRecord>,
) {
    let Some(module_name) = contents.lines().find_map(parse_module_name) else {
        return;
    };
    let module_id = format!("repo:{repo_id}:module:{module_name}");
    modules.push(ModuleRecord {
        repo_id: repo_id.to_string(),
        module_id: module_id.clone(),
        qualified_name: module_name.clone(),
        path: relative_path.to_string(),
    });

    let mut pending_docstring = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if is_single_line_docstring(trimmed) {
            pending_docstring = true;
            continue;
        }
        if let Some((symbol_name, signature, kind)) = parse_source_symbol(trimmed) {
            let symbol_id = format!("repo:{repo_id}:symbol:{module_name}.{symbol_name}");
            symbols.push(SymbolRecord {
                repo_id: repo_id.to_string(),
                symbol_id: symbol_id.clone(),
                module_id: Some(module_id.clone()),
                name: symbol_name.clone(),
                qualified_name: format!("{module_name}.{symbol_name}"),
                kind,
                path: relative_path.to_string(),
                line_start: None,
                line_end: None,
                signature,
                audit_status: None,
                verification_state: Some("unknown".to_string()),
                attributes: BTreeMap::new(),
            });
            if pending_docstring {
                let doc_path = format!("{relative_path}#symbol-id:{symbol_id}");
                let doc_id = format!("repo:{repo_id}:doc:{doc_path}");
                docs.push(DocRecord {
                    repo_id: repo_id.to_string(),
                    doc_id: doc_id.clone(),
                    title: symbol_name.clone(),
                    path: doc_path,
                    format: Some("julia_docstring".to_string()),
                    doc_target: None,
                });
                relations.push(RelationRecord {
                    repo_id: repo_id.to_string(),
                    source_id: doc_id,
                    target_id: symbol_id,
                    kind: RelationKind::Documents,
                });
            }
            pending_docstring = false;
            continue;
        }
        if pending_docstring && !trimmed.is_empty() && !trimmed.starts_with('#') {
            pending_docstring = false;
        }
    }
}

fn parse_module_name(line: &str) -> Option<String> {
    let module_name = line.strip_prefix("module ")?;
    valid_identifier(module_name).then(|| module_name.to_string())
}

fn parse_source_symbol(line: &str) -> Option<(String, Option<String>, RepoSymbolKind)> {
    if let Some(name) = line.strip_prefix("struct ") {
        let name = name.split_whitespace().next()?;
        return valid_identifier(name).then(|| {
            (
                name.to_string(),
                Some(format!("struct {name}")),
                RepoSymbolKind::Type,
            )
        });
    }
    let (head, _) = line.split_once('(')?;
    let name = head.trim();
    if !valid_identifier(name) {
        return None;
    }
    let (_, tail) = line.split_once(')')?;
    tail.trim_start().starts_with('=').then(|| {
        (
            name.to_string(),
            Some(line.to_string()),
            RepoSymbolKind::Function,
        )
    })
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '!')
}

fn is_single_line_docstring(line: &str) -> bool {
    line.starts_with("\"\"\"") && line.ends_with("\"\"\"") && line.len() >= 6
}

fn markdown_title(relative_path: &str, contents: &str) -> String {
    if relative_path == "README.md" {
        return "README.md".to_string();
    }
    contents
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix('#')
                .map(|rest| rest.trim_start_matches('#').trim())
                .filter(|title| !title.is_empty())
        })
        .map_or_else(|| example_title(relative_path), ToString::to_string)
}

fn append_readme_doc_record(
    repo_id: &str,
    repo_root: &Path,
    modules: &[ModuleRecord],
    docs: &mut Vec<DocRecord>,
    relations: &mut Vec<RelationRecord>,
) {
    if !repo_root.join("README.md").exists() {
        return;
    }
    docs.push(DocRecord {
        repo_id: repo_id.to_string(),
        doc_id: format!("repo:{repo_id}:doc:README.md"),
        title: "README.md".to_string(),
        path: "README.md".to_string(),
        format: Some("md".to_string()),
        doc_target: None,
    });
    if let Some(module) = modules.first() {
        relations.push(RelationRecord {
            repo_id: repo_id.to_string(),
            source_id: format!("repo:{repo_id}:doc:README.md"),
            target_id: module.module_id.clone(),
            kind: RelationKind::Documents,
        });
    }
}

fn append_markdown_doc_records(
    repo_id: &str,
    repo_root: &Path,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
    docs: &mut Vec<DocRecord>,
    relations: &mut Vec<RelationRecord>,
) -> Result<(), Box<dyn std::error::Error>> {
    for relative_path in collect_relative_files_under(repo_root, "docs", "md")? {
        let contents = fs::read_to_string(repo_root.join(&relative_path))?;
        let title = markdown_title(&relative_path, &contents);
        let doc_id = format!("repo:{repo_id}:doc:{relative_path}");
        docs.push(DocRecord {
            repo_id: repo_id.to_string(),
            doc_id: doc_id.clone(),
            title: title.clone(),
            path: relative_path.clone(),
            format: Some("md".to_string()),
            doc_target: None,
        });
        if let Some(target_id) = matching_doc_target_id(&title, modules, symbols) {
            relations.push(RelationRecord {
                repo_id: repo_id.to_string(),
                source_id: doc_id,
                target_id,
                kind: RelationKind::Documents,
            });
        }
    }
    Ok(())
}

fn collect_example_records(
    repo_id: &str,
    repo_root: &Path,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
    relations: &mut Vec<RelationRecord>,
) -> Result<Vec<ExampleRecord>, Box<dyn std::error::Error>> {
    let mut examples = Vec::new();
    for relative_path in collect_relative_files_under(repo_root, "examples", "jl")? {
        let contents = fs::read_to_string(repo_root.join(&relative_path))?;
        let example_id = format!("repo:{repo_id}:example:{relative_path}");
        examples.push(ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: example_id.clone(),
            title: example_title(&relative_path),
            path: relative_path.clone(),
            summary: None,
        });
        for target_id in example_target_ids(&contents, modules, symbols) {
            relations.push(RelationRecord {
                repo_id: repo_id.to_string(),
                source_id: example_id.clone(),
                target_id,
                kind: RelationKind::ExampleOf,
            });
        }
    }
    Ok(examples)
}

fn sort_local_fixture_analysis_records(
    modules: &mut [ModuleRecord],
    symbols: &mut [SymbolRecord],
    docs: &mut [DocRecord],
    examples: &mut [ExampleRecord],
) {
    modules.sort_by(|left, right| {
        left.qualified_name
            .cmp(&right.qualified_name)
            .then_with(|| left.path.cmp(&right.path))
    });
    symbols.sort_by(|left, right| {
        left.qualified_name
            .cmp(&right.qualified_name)
            .then_with(|| left.path.cmp(&right.path))
    });
    docs.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.doc_id.cmp(&right.doc_id))
    });
    examples.sort_by(|left, right| left.path.cmp(&right.path));
}

fn build_local_fixture_repository_record(
    repo_id: &str,
    repo_root: &Path,
    revision: Option<String>,
    project_name: Option<String>,
    version: Option<String>,
    uuid: Option<String>,
    modules: &[ModuleRecord],
) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: repo_id.to_string(),
        name: project_name
            .or_else(|| modules.first().map(|module| module.qualified_name.clone()))
            .unwrap_or_else(|| repo_id.to_string()),
        path: repo_root.display().to_string(),
        url: None,
        revision,
        version,
        uuid,
        dependencies: Vec::new(),
    }
}

fn example_title(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(relative_path)
        .to_string()
}

fn matching_doc_target_id(
    title: &str,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
) -> Option<String> {
    symbols
        .iter()
        .find(|symbol| symbol.name == title || symbol.qualified_name == title)
        .map(|symbol| symbol.symbol_id.clone())
        .or_else(|| {
            modules
                .iter()
                .find(|module| {
                    module.qualified_name == title
                        || module
                            .qualified_name
                            .rsplit('.')
                            .next()
                            .is_some_and(|name| name == title)
                })
                .map(|module| module.module_id.clone())
        })
}

fn example_target_ids(
    contents: &str,
    modules: &[ModuleRecord],
    symbols: &[SymbolRecord],
) -> Vec<String> {
    let mut target_ids = Vec::new();
    for symbol in symbols {
        if symbol_mentioned_in_example(contents, &symbol.name) {
            push_unique_string(&mut target_ids, symbol.symbol_id.clone());
            if let Some(module_id) = symbol.module_id.as_ref() {
                push_unique_string(&mut target_ids, module_id.clone());
            }
        }
    }
    if !target_ids.is_empty() {
        return target_ids;
    }
    for module in modules {
        if module_mentioned_in_example(contents, &module.qualified_name) {
            push_unique_string(&mut target_ids, module.module_id.clone());
        }
    }
    target_ids
}

fn symbol_mentioned_in_example(contents: &str, symbol_name: &str) -> bool {
    contents.contains(&format!("{symbol_name}(")) || contents.contains(&format!(".{symbol_name}("))
}

fn module_mentioned_in_example(contents: &str, module_name: &str) -> bool {
    contents.contains(&format!("using {module_name}"))
        || contents.contains(&format!("import {module_name}"))
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn create_local_git_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(repo_dir.join("README.md"), "# Gateway Repo\n")?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            r#"name = "{package_name}"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"
"#
        ),
    )?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!("module {package_name}\nend\n"),
    )?;

    init_git_repository(&repo_dir);
    add_git_remote(
        &repo_dir,
        "origin",
        &format!(
            "https://example.invalid/xiuxian-wendao/{}.git",
            package_name.to_ascii_lowercase()
        ),
    );
    commit_all(&repo_dir, "initial import");
    Ok(repo_dir)
}

fn ensure_linked_modelica_parser_summary_service() -> TestResult {
    let service = LINKED_MODELICA_PARSER_SUMMARY_SERVICE.get_or_init(|| {
        std::thread::spawn(|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            let (base_url, guard) =
                runtime.block_on(spawn_wendaosearch_modelica_parser_summary_service());
            set_linked_modelica_parser_summary_base_url_for_tests(base_url.as_str())?;
            Ok::<LinkedModelicaParserSummaryService, String>(LinkedModelicaParserSummaryService {
                _guard: Mutex::new(guard),
            })
        })
        .join()
        .unwrap_or_else(|panic_payload| {
            Err::<LinkedModelicaParserSummaryService, String>(
                if let Some(message) = panic_payload.downcast_ref::<&'static str>() {
                    (*message).to_string()
                } else if let Some(message) = panic_payload.downcast_ref::<String>() {
                    message.clone()
                } else {
                    "unknown panic payload".to_string()
                },
            )
        })
    });
    service
        .as_ref()
        .map(|_| ())
        .map_err(|message| Box::new(IoError::other(message.clone())) as Box<dyn std::error::Error>)
}

fn create_local_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers").join("Examples"))?;
    fs::create_dir_all(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("Tutorial"),
    )?;
    fs::write(repo_dir.join("README.md"), format!("# {package_name}\n"))?;
    fs::write(repo_dir.join("package.order"), "Controllers\n")?;
    fs::write(
        repo_dir.join("package.mo"),
        format!(
            "within;\npackage {package_name}\n  annotation(Documentation(info = \"<html>{package_name} package docs.</html>\"));\nend {package_name};\n",
        ),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("package.mo"),
        format!("within {package_name};\npackage Controllers\nend Controllers;\n"),
    )?;
    fs::write(
        repo_dir.join("Controllers").join("PI.mo"),
        format!(
            "within {package_name}.Controllers;\nmodel PI\n  annotation(Documentation(info = \"<html>PI controller docs.</html>\"));\nend PI;\n",
        ),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("Examples")
            .join("package.order"),
        "Step\n",
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("Examples")
            .join("Step.mo"),
        format!("within {package_name}.Controllers.Examples;\nmodel Step\nend Step;\n"),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("package.order"),
        "Tutorial\n",
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("package.mo"),
        format!("within {package_name}.Controllers;\npackage UsersGuide\nend UsersGuide;\n"),
    )?;
    fs::write(
        repo_dir
            .join("Controllers")
            .join("UsersGuide")
            .join("Tutorial")
            .join("FirstSteps.mo"),
        format!(
            "within {package_name}.Controllers.UsersGuide.Tutorial;\nmodel FirstSteps\n  annotation(Documentation(info = \"<html>First steps guide.</html>\"));\nend FirstSteps;\n",
        ),
    )?;

    init_git_repository(&repo_dir);
    add_git_remote(
        &repo_dir,
        "origin",
        &format!(
            "https://example.invalid/xiuxian-wendao/{}.git",
            package_name.to_ascii_lowercase()
        ),
    );
    commit_all(&repo_dir, "initial import");
    Ok(repo_dir)
}

fn write_modelica_repo_config(
    base: &Path,
    repo_dir: &Path,
    repo_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        base.join("wendao.toml"),
        format!(
            r#"[link_graph.projects.{repo_id}]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(())
}

fn projected_page_id_for_title(
    base: &Path,
    repo_id: &str,
    kind: ProjectionPageKind,
    title: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: repo_id.to_string(),
        },
        None,
        base,
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| page.kind == kind && page.title == title)
        .unwrap_or_else(|| panic!("expected a projected `{title}` page in repo `{repo_id}`"));
    Ok(page.page_id.clone())
}

fn projected_page_and_node_id_for_title(
    base: &Path,
    repo_id: &str,
    title: &str,
    node_title: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let page_id = projected_page_id_for_title(base, repo_id, ProjectionPageKind::Reference, title)?;
    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: repo_id.to_string(),
        },
        None,
        base,
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for `{title}`"));
    let node_id = find_node_id(tree.roots.as_slice(), node_title)
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `{node_title}`"));
    Ok((page_id, node_id))
}

fn redact_repo_sync_payload(value: &mut Value) {
    if let Some(path) = value.pointer_mut("/checkout_path") {
        *path = Value::String("[checkout-path]".to_string());
    }
    if let Some(path) = value.pointer_mut("/mirror_path") {
        *path = Value::String("[mirror-path]".to_string());
    }
    if let Some(url) = value.pointer_mut("/upstream_url") {
        *url = Value::String("[upstream-url]".to_string());
    }
    if let Some(path) = value.pointer_mut("/checked_at") {
        *path = Value::String("[checked-at]".to_string());
    }
    if let Some(path) = value.pointer_mut("/last_fetched_at") {
        *path = match path {
            Value::Null => Value::Null,
            _ => Value::String("[last-fetched-at]".to_string()),
        };
    }
    if let Some(path) = value.pointer_mut("/status_summary/freshness/checked_at") {
        *path = Value::String("[checked-at]".to_string());
    }
    if let Some(path) = value.pointer_mut("/status_summary/freshness/last_fetched_at") {
        *path = match path {
            Value::Null => Value::Null,
            _ => Value::String("[last-fetched-at]".to_string()),
        };
    }
    redact_revision_pointers(
        value,
        &[
            "/revision",
            "/mirror_revision",
            "/tracking_revision",
            "/status_summary/revisions/checkout_revision",
            "/status_summary/revisions/mirror_revision",
            "/status_summary/revisions/tracking_revision",
        ],
    );
}

fn redact_repo_overview_payload(value: &mut Value) {
    redact_revision_pointers(value, &["/revision"]);
}

fn redact_revision_pointers(value: &mut Value, pointers: &[&str]) {
    for pointer in pointers {
        if let Some(revision) = value.pointer_mut(pointer)
            && !revision.is_null()
        {
            *revision = Value::String("[revision]".to_string());
        }
    }
}

fn redact_repo_index_payload(value: &mut Value) {
    if let Some(max_concurrency) = value.get_mut("maxConcurrency") {
        *max_concurrency = Value::Number(1.into());
    }
    if let Some(sync_concurrency_limit) = value.get_mut("syncConcurrencyLimit") {
        *sync_concurrency_limit = Value::Number(1.into());
    }
    if let Some(repos) = value.get_mut("repos").and_then(Value::as_array_mut) {
        for repo in repos {
            if let Some(updated_at) = repo.get_mut("updatedAt") {
                *updated_at = Value::String("[updated-at]".to_string());
            }
        }
    }
    if let Some(activation_at) =
        value.get_mut("studioBootstrapBackgroundIndexingDeferredActivationAt")
    {
        *activation_at = match activation_at {
            Value::Null => Value::Null,
            _ => Value::String("[activation-at]".to_string()),
        };
    }
}

fn find_node_id(nodes: &[ProjectedPageIndexNode], title: &str) -> Option<String> {
    for node in nodes {
        if node.title == title {
            return Some(node.node_id.clone());
        }
        if let Some(node_id) = find_node_id(node.children.as_slice(), title) {
            return Some(node_id);
        }
    }
    None
}
