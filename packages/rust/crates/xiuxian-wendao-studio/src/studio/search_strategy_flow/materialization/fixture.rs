//! Owns the Studio search strategy flow materialization fixture surface.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use tempfile::TempDir;

use crate::studio::arrow_types::LanceRecordBatch;
use crate::studio::{GatewayState, StudioState, build_studio_flight_service};
use crate::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
};
use xiuxian_git_repo::{SyncMode, discover_checkout_metadata};
use xiuxian_wendao::analyzers::{
    DocRecord, ExampleRecord, ModuleRecord, RelationKind, RelationRecord, RepoSymbolKind,
    RepositoryAnalysisOutput, RepositoryRecord, SymbolRecord, build_repository_analysis_cache_key,
    load_repo_intelligence_config, resolve_registered_repository_source,
    store_cached_repository_analysis,
};
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::search::SearchPlaneService;

use super::flight::{
    collect_route_batches, decoded_payload_receipt, find_node_id_by_title, first_string, first_u64,
    populate_graph_neighbors_headers, populate_repo_projected_page_index_tree_headers,
    populate_repo_projected_retrieval_context_headers, populate_repo_search_headers, route_receipt,
    string_values,
};
use super::receipt::{
    RouteDecodedPayloadReceipt, RouteMaterializationReceipt,
    SearchStrategyFlowMaterializationError, SearchStrategyFlowMaterializationReceipt,
};

const REPO_ID: &str = "gateway-sync";
const PAGE_ID: &str =
    "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md";
const TEST_GIT_AUTHOR_NAME: &str = "Xiuxian Test";
const TEST_GIT_AUTHOR_EMAIL: &str = "test@example.com";
const TEST_GIT_COMMIT_TIME: &str = "1700000000 +0000";

/// Executes the fixture SearchStrategyFlow native Flight route sequence and
/// returns a decoded materialization receipt as JSON.
///
/// # Errors
///
/// Returns an error when the fixture cannot be created, a Flight route cannot
/// be executed, or the decoded route payloads do not contain the expected
/// evidence anchors.
pub async fn materialize_fixture_receipt_json()
-> Result<serde_json::Value, SearchStrategyFlowMaterializationError> {
    let receipt = materialize_fixture_receipt().await?;
    receipt.to_json()
}

async fn materialize_fixture_receipt()
-> Result<SearchStrategyFlowMaterializationReceipt, SearchStrategyFlowMaterializationError> {
    let fixture = SearchStrategyFlowFixture::create().await?;
    let service = build_studio_flight_service(
        Arc::new(fixture.state.studio.search_plane.clone()),
        Arc::clone(&fixture.state),
        "v2",
        3,
    )
    .map_err(|error| {
        SearchStrategyFlowMaterializationError::message(format!(
            "build materialization Flight service: {error}"
        ))
    })?;

    let repo_search_batches = collect_route_batches(
        &service,
        REPO_SEARCH_ROUTE,
        "SearchStrategyFlow repo search materialization",
        |metadata| populate_repo_search_headers(metadata, REPO_ID, "solve anchors", 5),
    )
    .await?;
    require(
        string_values(&repo_search_batches[0], REPO_SEARCH_PATH_COLUMN)?
            .iter()
            .any(|path| path.contains("solve")),
        "repo search should materialize a solve-related repository hit",
    )?;

    let page_index_batches = collect_route_batches(
        &service,
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        "SearchStrategyFlow page-index materialization",
        |metadata| populate_repo_projected_page_index_tree_headers(metadata, REPO_ID, PAGE_ID),
    )
    .await?;
    require_eq(first_string(&page_index_batches[0], "pageId")?, PAGE_ID)?;
    require(
        first_u64(&page_index_batches[0], "rootCount")? > 0,
        "page-index tree should expose roots",
    )?;
    let roots_json = first_string(&page_index_batches[0], "rootsJson")?;
    require(
        roots_json.contains("Anchors"),
        "page-index tree should expose section-level anchors for agent traversal",
    )?;
    let node_id = find_node_id_by_title(&serde_json::from_str(&roots_json)?, "Anchors")
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(
                "page-index tree should contain an Anchors node",
            )
        })?;

    let retrieval_context_batches = collect_route_batches(
        &service,
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        "SearchStrategyFlow retrieval-context materialization",
        |metadata| {
            populate_repo_projected_retrieval_context_headers(
                metadata,
                REPO_ID,
                PAGE_ID,
                Some(node_id.as_str()),
                3,
            )
        },
    )
    .await?;
    require_eq(
        first_string(&retrieval_context_batches[0], "pageId")?,
        PAGE_ID,
    )?;
    require_eq(
        first_string(&retrieval_context_batches[0], "nodeId")?,
        node_id.as_str(),
    )?;
    require(
        first_string(&retrieval_context_batches[0], "centerJson")?.contains("Anchors"),
        "retrieval context should preserve requested section content through the center page",
    )?;
    require(
        first_string(&retrieval_context_batches[0], "nodeContextJson")?.contains("Documentation"),
        "retrieval context should preserve the requested section neighborhood",
    )?;

    let graph_batches = collect_route_batches(
        &service,
        GRAPH_NEIGHBORS_ROUTE,
        "SearchStrategyFlow graph-neighbor materialization",
        |metadata| {
            populate_graph_neighbors_headers(metadata, "kernel/docs/alpha.md", "both", 1, 20)
        },
    )
    .await?;
    require(
        string_values(&graph_batches[0], "rowType")?
            .iter()
            .any(|row_type| row_type == "node"),
        "graph-neighbors route should materialize node rows",
    )?;

    let route_receipts = route_receipts(
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
    )?;
    let decoded_payload_receipts = decoded_payload_receipts(
        &repo_search_batches,
        &page_index_batches,
        &retrieval_context_batches,
        &graph_batches,
        &node_id,
    )?;
    Ok(SearchStrategyFlowMaterializationReceipt::executed(
        "studio-flight-proof",
        route_receipts,
        decoded_payload_receipts,
    ))
}

fn route_receipts(
    repo_search_batches: &[LanceRecordBatch],
    page_index_batches: &[LanceRecordBatch],
    retrieval_context_batches: &[LanceRecordBatch],
    graph_batches: &[LanceRecordBatch],
) -> Result<Vec<RouteMaterializationReceipt>, SearchStrategyFlowMaterializationError> {
    Ok(vec![
        route_receipt(REPO_SEARCH_ROUTE, repo_search_batches)?,
        route_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            page_index_batches,
        )?,
        route_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            retrieval_context_batches,
        )?,
        route_receipt(GRAPH_NEIGHBORS_ROUTE, graph_batches)?,
    ])
}

fn decoded_payload_receipts(
    repo_search_batches: &[LanceRecordBatch],
    page_index_batches: &[LanceRecordBatch],
    retrieval_context_batches: &[LanceRecordBatch],
    graph_batches: &[LanceRecordBatch],
    node_id: &str,
) -> Result<Vec<RouteDecodedPayloadReceipt>, SearchStrategyFlowMaterializationError> {
    Ok(vec![
        decoded_payload_receipt(
            REPO_SEARCH_ROUTE,
            repo_search_batches,
            vec![REPO_SEARCH_PATH_COLUMN],
            format!(
                "path:{}",
                first_string(&repo_search_batches[0], REPO_SEARCH_PATH_COLUMN)?
            ),
        )?,
        decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
            page_index_batches,
            vec!["pageId", "rootCount", "rootsJson"],
            format!("node:{node_id}"),
        )?,
        decoded_payload_receipt(
            ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
            retrieval_context_batches,
            vec!["pageId", "nodeId", "centerJson", "nodeContextJson"],
            format!(
                "node-context:{}",
                first_string(&retrieval_context_batches[0], "nodeId")?
            ),
        )?,
        decoded_payload_receipt(
            GRAPH_NEIGHBORS_ROUTE,
            graph_batches,
            vec!["rowType"],
            "row-type:node".to_string(),
        )?,
    ])
}

struct SearchStrategyFlowFixture {
    _temp_dir: TempDir,
    state: Arc<GatewayState>,
}

impl SearchStrategyFlowFixture {
    async fn create() -> Result<Self, SearchStrategyFlowMaterializationError> {
        let temp_dir = tempfile::tempdir()?;
        write_fixture_files(
            temp_dir.path(),
            &[
                ("docs/alpha.md", "# Alpha\n\nSee [[beta]].\n"),
                ("docs/beta.md", "# Beta\n\nBody.\n"),
            ],
        )?;
        let repo_dir = create_gateway_sync_julia_repo(temp_dir.path())?;
        fs::write(
            temp_dir.path().join("wendao.toml"),
            format!(
                r#"[link_graph.projects.kernel]
root = "."
dirs = ["docs"]

[link_graph.projects.gateway-sync]
root = "{}"
plugins = ["julia-code-parser"]
refresh = "manual"
"#,
                repo_dir.display()
            ),
        )?;
        prime_fixture_analysis_cache(temp_dir.path())?;

        let plugin_registry = Arc::new(
            xiuxian_wendao::analyzers::bootstrap_builtin_registry().map_err(|error| {
                SearchStrategyFlowMaterializationError::message(format!(
                    "bootstrap builtin plugin registry: {error}"
                ))
            })?,
        );
        let search_plane = SearchPlaneService::new(temp_dir.path().to_path_buf());
        let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
            plugin_registry,
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
            search_plane,
        );
        studio
            .search_plane
            .publish_repo_content_chunks_with_revision(
                REPO_ID,
                &[RepoCodeDocument {
                    path: ("docs/solve.md".to_string()).into(),
                    language: Some("markdown".to_string()),
                    contents: Arc::<str>::from("solve anchors and source examples"),
                    size_bytes: 33,
                    modified_unix_ms: 10,
                }],
                Some("search-strategy-flow-rev-1"),
            )
            .await
            .map_err(|error| {
                SearchStrategyFlowMaterializationError::message(format!(
                    "publish SearchStrategyFlow repo search fixture: {error}"
                ))
            })?;

        Ok(Self {
            _temp_dir: temp_dir,
            state: Arc::new(GatewayState {
                index: None,
                signal_tx: None,
                webhook_url: None,
                studio: Arc::new(studio),
            }),
        })
    }
}

fn prime_fixture_analysis_cache(
    project_root: &Path,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    let config_path = project_root.join("wendao.toml");
    let repo_config = load_repo_intelligence_config(Some(config_path.as_path()), project_root)
        .map_err(|error| {
            SearchStrategyFlowMaterializationError::message(format!(
                "load fixture repo config: {error}"
            ))
        })?;
    let repository = repo_config
        .repos
        .iter()
        .find(|repository| repository.id == REPO_ID)
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(format!(
                "missing fixture repository `{REPO_ID}`"
            ))
        })?;
    let repository_source =
        resolve_registered_repository_source(repository, project_root, SyncMode::Status).map_err(
            |error| {
                SearchStrategyFlowMaterializationError::message(format!(
                    "resolve fixture repository source: {error}"
                ))
            },
        )?;
    let checkout_metadata = discover_checkout_metadata(repository_source.checkout_root.as_path());
    let cache_key = build_repository_analysis_cache_key(
        repository,
        &repository_source,
        checkout_metadata.as_ref(),
    );
    let analysis = fixture_repository_analysis(
        repository_source.checkout_root.as_path(),
        checkout_metadata
            .as_ref()
            .and_then(|metadata| metadata.revision.clone()),
    );
    store_cached_repository_analysis(cache_key, &analysis).map_err(|error| {
        SearchStrategyFlowMaterializationError::message(format!(
            "store fixture repo analysis cache: {error}"
        ))
    })
}

fn fixture_repository_analysis(
    repo_root: &Path,
    revision: Option<String>,
) -> RepositoryAnalysisOutput {
    let module_id = format!("repo:{REPO_ID}:module:GatewaySyncPkg");
    let symbol_id = format!("repo:{REPO_ID}:symbol:GatewaySyncPkg.solve");
    let readme_doc_id = format!("repo:{REPO_ID}:doc:README.md");
    let docstring_doc_id =
        format!("repo:{REPO_ID}:doc:src/GatewaySyncPkg.jl#symbol-id:{symbol_id}");
    let solve_doc_id = format!("repo:{REPO_ID}:doc:docs/solve.md");
    let example_id = format!("repo:{REPO_ID}:example:examples/solve_demo.jl");

    RepositoryAnalysisOutput {
        repository: Some(RepositoryRecord {
            repo_id: (REPO_ID.to_string()).into(),
            name: "GatewaySyncPkg".to_string(),
            path: (repo_root.display().to_string()).into(),
            url: None,
            revision,
            version: Some("0.1.0".to_string()),
            uuid: Some("12345678-1234-1234-1234-123456789abc".to_string()),
            dependencies: Vec::new(),
        }),
        modules: vec![ModuleRecord {
            repo_id: (REPO_ID.to_string()).into(),
            module_id: (module_id.clone()).into(),
            qualified_name: "GatewaySyncPkg".to_string(),
            path: ("src/GatewaySyncPkg.jl".to_string()).into(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: (REPO_ID.to_string()).into(),
            symbol_id: (symbol_id.clone()).into(),
            module_id: Some((module_id.clone()).into()),
            name: "solve".to_string(),
            qualified_name: "GatewaySyncPkg.solve".to_string(),
            kind: RepoSymbolKind::Function,
            path: ("src/GatewaySyncPkg.jl".to_string()).into(),
            line_start: None,
            line_end: None,
            signature: Some("solve() = nothing".to_string()),
            audit_status: None,
            verification_state: Some(("unknown".to_string()).into()),
            attributes: BTreeMap::new(),
        }],
        imports: Vec::new(),
        examples: vec![ExampleRecord {
            repo_id: (REPO_ID.to_string()).into(),
            example_id: (example_id.clone()).into(),
            title: "solve_demo".to_string(),
            path: ("examples/solve_demo.jl".to_string()).into(),
            summary: None,
        }],
        docs: vec![
            DocRecord {
                repo_id: (REPO_ID.to_string()).into(),
                doc_id: (readme_doc_id.clone()).into(),
                title: "README.md".to_string(),
                path: ("README.md".to_string()).into(),
                format: Some("md".to_string()),
                doc_target: None,
            },
            DocRecord {
                repo_id: (REPO_ID.to_string()).into(),
                doc_id: (docstring_doc_id.clone()).into(),
                title: "solve".to_string(),
                path: (format!("src/GatewaySyncPkg.jl#symbol-id:{symbol_id}")).into(),
                format: Some("julia_docstring".to_string()),
                doc_target: None,
            },
            DocRecord {
                repo_id: (REPO_ID.to_string()).into(),
                doc_id: (solve_doc_id.clone()).into(),
                title: "solve".to_string(),
                path: ("docs/solve.md".to_string()).into(),
                format: Some("md".to_string()),
                doc_target: None,
            },
        ],
        relations: vec![
            RelationRecord {
                repo_id: (REPO_ID.to_string()).into(),
                source_id: readme_doc_id,
                target_id: module_id.clone(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: (REPO_ID.to_string()).into(),
                source_id: docstring_doc_id,
                target_id: symbol_id.clone(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: (REPO_ID.to_string()).into(),
                source_id: solve_doc_id,
                target_id: symbol_id.clone(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: (REPO_ID.to_string()).into(),
                source_id: example_id,
                target_id: symbol_id,
                kind: RelationKind::ExampleOf,
            },
        ],
        diagnostics: Vec::new(),
    }
}

fn create_gateway_sync_julia_repo(
    base: &Path,
) -> Result<PathBuf, SearchStrategyFlowMaterializationError> {
    let repo_dir = base.join("gatewaysyncpkg");
    write_fixture_files(
        repo_dir.as_path(),
        &[
            ("README.md", "# Gateway Sync Package\n"),
            (
                "Project.toml",
                r#"name = "GatewaySyncPkg"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"
"#,
            ),
            (
                "src/GatewaySyncPkg.jl",
                "module GatewaySyncPkg\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n",
            ),
            ("examples/solve_demo.jl", "using GatewaySyncPkg\nsolve()\n"),
            ("docs/solve.md", "# solve\n"),
        ],
    )?;
    init_git_repository(repo_dir.as_path())?;
    add_git_remote(
        repo_dir.as_path(),
        "origin",
        "https://example.invalid/xiuxian-wendao/gatewaysyncpkg.git",
    )?;
    commit_all(repo_dir.as_path(), "initial import")?;
    Ok(repo_dir)
}

fn write_fixture_files(
    root: &Path,
    files: &[(&str, &str)],
) -> Result<(), SearchStrategyFlowMaterializationError> {
    for (path, contents) in files {
        let full_path = root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, contents)?;
    }
    Ok(())
}

fn init_git_repository(path: &Path) -> Result<(), SearchStrategyFlowMaterializationError> {
    let path_arg = path.display().to_string();
    run_git(None, &["init", "--quiet", path_arg.as_str()])
}

fn add_git_remote(
    path: &Path,
    remote_name: &str,
    remote_url: &str,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    run_git(Some(path), &["remote", "add", remote_name, remote_url])
}

fn commit_all(path: &Path, message: &str) -> Result<(), SearchStrategyFlowMaterializationError> {
    run_git(Some(path), &["add", "--all"])?;
    run_git(Some(path), &["commit", "--quiet", "-m", message])?;
    run_git(Some(path), &["branch", "-M", "main"])
}

fn run_git(
    cwd: Option<&Path>,
    args: &[&str],
) -> Result<(), SearchStrategyFlowMaterializationError> {
    let mut command = Command::new("git");
    if let Some(cwd) = cwd {
        command.arg("-C").arg(cwd);
    }
    let output = command
        .args(args)
        .env("GIT_AUTHOR_NAME", TEST_GIT_AUTHOR_NAME)
        .env("GIT_AUTHOR_EMAIL", TEST_GIT_AUTHOR_EMAIL)
        .env("GIT_COMMITTER_NAME", TEST_GIT_AUTHOR_NAME)
        .env("GIT_COMMITTER_EMAIL", TEST_GIT_AUTHOR_EMAIL)
        .env("GIT_AUTHOR_DATE", TEST_GIT_COMMIT_TIME)
        .env("GIT_COMMITTER_DATE", TEST_GIT_COMMIT_TIME)
        .output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = match (stderr.is_empty(), stdout.is_empty()) {
        (false, true) => stderr,
        (true, false) => stdout,
        (false, false) => format!("{stderr}; stdout: {stdout}"),
        (true, true) => "unknown git error".to_string(),
    };
    Err(SearchStrategyFlowMaterializationError::message(format!(
        "git {} failed: {detail}",
        args.join(" ")
    )))
}

fn require(condition: bool, message: &str) -> Result<(), SearchStrategyFlowMaterializationError> {
    if condition {
        return Ok(());
    }
    Err(SearchStrategyFlowMaterializationError::message(message))
}

fn require_eq(
    actual: impl AsRef<str>,
    expected: &str,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    if actual.as_ref() == expected {
        return Ok(());
    }
    Err(SearchStrategyFlowMaterializationError::message(format!(
        "expected `{expected}`, got `{}`",
        actual.as_ref()
    )))
}
