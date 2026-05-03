#![cfg(feature = "zhenfa-router")]

use std::fs;

use axum::http::StatusCode;

use crate::studio::studio_router;
use crate::studio::test_support::assert_studio_json_snapshot;
use xiuxian_wendao::analyzers::{
    DocsProjectedGapReportQuery, ProjectionPageKind, RefineEntityDocRequest, RegisteredRepository,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, RepositoryPluginConfig,
    RepositoryRefreshPolicy, analyze_registered_repository_with_registry, build_projected_pages,
    docs_projected_gap_report_from_config, repo_projected_page_index_trees_from_config,
    repo_projected_pages_from_config,
};
use xiuxian_wendao::repo_index::RepoIndexRequest;
use xiuxian_wendao::search::contracts::{UiConfig, UiRepoProjectConfig};

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
