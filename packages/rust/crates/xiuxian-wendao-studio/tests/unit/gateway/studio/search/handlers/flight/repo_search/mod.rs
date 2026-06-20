#![cfg(test)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::studio::arrow_types::{LanceFloat64Array, LanceRecordBatch, LanceStringArray};
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightDescriptor, FlightInfo};
use tonic::Request;

use super::{
    StudioFlightRoots, StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_studio_flight_service,
    build_studio_flight_service_for_roots,
};
use crate::contracts::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use crate::studio::search::build_symbol_index;
use crate::studio::test_support::commit_all;
use crate::studio::{GatewayState, StudioState, configured_repositories};
use crate::transport::{
    ANALYSIS_MARKDOWN_ROUTE, RepoSearchFlightRequest, RepoSearchFlightRouteProvider,
    SEARCH_SYMBOLS_ROUTE, WENDAO_ANALYSIS_PATH_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER, flight_descriptor_path,
};
use xiuxian_git_repo::SyncMode;
use xiuxian_wendao::analyzers::bootstrap_builtin_registry;
use xiuxian_wendao::analyzers::resolve_registered_repository_source;
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};

mod bootstrap;
mod filters;
mod provider;
mod ranking;
mod routes;
mod support;

use support::{
    RepoSearchRequestFilters, commit_all_or_panic, create_dir_all_or_panic, init_git_repo_or_panic,
    populate_markdown_analysis_headers, populate_search_headers, repo_document,
    repo_search_batch_or_panic, repo_search_request, string_column, tempdir_or_panic,
    test_studio_state, write_file_or_panic,
};
use support::{first_ticket, float_column};
