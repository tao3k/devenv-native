#![cfg(test)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightDescriptor, FlightInfo};
use tonic::Request;
use xiuxian_db_store::{LanceFloat64Array, LanceRecordBatch, LanceStringArray};

use super::{
    StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_studio_flight_service,
    build_studio_flight_service_for_roots,
};
use crate::analyzers::bootstrap_builtin_registry;
use crate::analyzers::resolve_registered_repository_source;
use crate::gateway::studio::search::build_symbol_index;
use crate::gateway::studio::test_support::commit_all;
use crate::gateway::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use crate::gateway::studio::{GatewayState, StudioState, configured_repositories};
use crate::repo_index::RepoCodeDocument;
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_CODE_AST_ROUTE, WENDAO_ANALYSIS_LINE_HEADER, WENDAO_ANALYSIS_REPO_HEADER,
};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_MARKDOWN_ROUTE, RepoSearchFlightRequest, RepoSearchFlightRouteProvider,
    SEARCH_SYMBOLS_ROUTE, WENDAO_ANALYSIS_PATH_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER, flight_descriptor_path,
};

mod bootstrap;
mod filters;
mod provider;
mod ranking;
mod routes;
mod support;

use support::{
    RepoSearchRequestFilters, commit_all_or_panic, create_dir_all_or_panic, init_git_repo_or_panic,
    populate_code_ast_analysis_headers, populate_markdown_analysis_headers,
    populate_search_headers, repo_document, repo_search_batch_or_panic, repo_search_request,
    string_column, tempdir_or_panic, test_studio_state, write_file_or_panic,
};
use support::{first_ticket, float_column};
