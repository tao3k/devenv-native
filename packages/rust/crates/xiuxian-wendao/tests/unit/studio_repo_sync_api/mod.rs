#![cfg(feature = "zhenfa-router")]

use crate as xiuxian_wendao;

use std::collections::BTreeMap;
use std::fs;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::UNIX_EPOCH;

use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

use xiuxian_git_repo::{SyncMode, discover_checkout_metadata};
use xiuxian_wendao::analyzers::resolve_registered_repository_source;
use xiuxian_wendao::analyzers::{
    DocRecord, DocsProjectedGapReportQuery, ExampleRecord, ProjectedPageIndexNode,
    ProjectionPageKind, RefineEntityDocRequest, RegisteredRepository,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, RepoSymbolKind,
    RepositoryAnalysisOutput, RepositoryPluginConfig, RepositoryRecord, RepositoryRefreshPolicy,
    SymbolRecord, analyze_registered_repository_with_registry, build_projected_pages,
    build_repository_analysis_cache_key, docs_projected_gap_report_from_config,
    load_repo_intelligence_config, repo_projected_page_index_trees_from_config,
    repo_projected_pages_from_config, store_cached_repository_analysis,
};
use xiuxian_wendao::analyzers::{ModuleRecord, RelationKind, RelationRecord};
use xiuxian_wendao::gateway::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service;
use xiuxian_wendao::gateway::studio::symbol_index::SymbolIndexCoordinator;
use xiuxian_wendao::gateway::studio::test_support::{
    add_git_remote, assert_studio_json_snapshot, commit_all, init_git_repository,
};
use xiuxian_wendao::gateway::studio::types::{UiConfig, UiRepoProjectConfig};
use xiuxian_wendao::gateway::studio::{GatewayState, StudioState, studio_router};
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::repo_index::{RepoIndexCoordinator, RepoIndexRequest};
use xiuxian_wendao::search::{SearchPlaneService, publish_repo_entities};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type LocalProjectMetadata = (Option<String>, Option<String>, Option<String>);

mod docs_endpoints;
mod error_cases;
mod gap_reports;
mod planner;
mod repo_endpoints;
mod repo_projected_context;
mod repo_projected_lookup;
mod support;

use support::*;
