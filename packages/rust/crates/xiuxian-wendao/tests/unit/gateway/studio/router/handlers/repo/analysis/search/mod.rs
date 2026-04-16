use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::analyzers::{
    ExampleRecord, ImportKind, ImportRecord, ModuleRecord, RegisteredRepository, RepoSymbolKind,
    RepositoryAnalysisOutput, RepositoryPluginConfig, RepositoryRefreshPolicy, SymbolRecord,
    bootstrap_builtin_registry, resolve_registered_repository_source,
};
use crate::analyzers::{
    RepositorySearchQueryCacheKey, build_repository_analysis_cache_key,
    store_cached_repository_analysis,
};
use crate::gateway::studio::router::StudioApiError;
use crate::gateway::studio::router::configured_repository;
use crate::gateway::studio::router::handlers::repo::analysis::search::cache::{
    repository_search_key, with_cached_repo_search_result,
};
use crate::gateway::studio::router::handlers::repo::analysis::search::service::imports::run_repo_import_search;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::test_support::{
    assert_studio_json_snapshot, commit_all, init_git_repository,
};
use crate::gateway::studio::types::{UiConfig, UiRepoProjectConfig};
use crate::query_core::{
    query_repo_entity_example_results_if_published, query_repo_entity_import_results_if_published,
    query_repo_entity_module_results_if_published, query_repo_entity_symbol_results_if_published,
};
use crate::repo_index::RepoCodeDocument;
use crate::search::{
    FuzzySearchOptions, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    publish_repo_entities,
};
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
    SyncMode, discover_checkout_metadata,
};

mod cache_behavior;
mod import_fast_path;
mod query_core;
mod support;
