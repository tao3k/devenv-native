use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use xiuxian_ast::Lang;
use xiuxian_wendao_runtime::transport::{
    SEARCH_AUTOCOMPLETE_ROUTE, SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE,
};

use crate::gateway::studio::router::repository::configured_repositories;
use crate::gateway::studio::router::sanitization::{sanitize_projects, sanitize_repo_projects};
use crate::gateway::studio::router::state::helpers::supported_code_kinds;
use crate::gateway::studio::router::state::types::{StudioConfiguredOwners, StudioState};
use crate::gateway::studio::types::{
    UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample, UiCodeSearchRoutes,
    UiConfig, UiProjectConfig, UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract,
    UiRepoProjectConfig, UiSearchContract, UiSearchContractAlias,
};
use crate::parsers::search::repo_code_query::{
    REPO_CODE_SEARCH_BACKEND_PREFIXES, REPO_CODE_SEARCH_KIND_FILTER_VALUES,
    REPO_CODE_SEARCH_PREFIX_ALIASES, REPO_CODE_SEARCH_STRUCTURAL_PREFIXES,
};
use crate::repo_index::RepoIndexStatusResponse;
use crate::search::SearchCorpusKind;

const BOOTSTRAP_RUNTIME_SOURCE: &str = "studio_bootstrap";
#[cfg(test)]
const TEST_CONFIGURED_OWNER_SEED_SOURCE: &str = "test_configured_owner_seed";
const STUDIO_SEARCH_CONTRACT_VERSION: &str = "1";
const STUDIO_CODE_SEARCH_GRAMMAR_VERSION: &str = "repo_code_query.v1";
const STUDIO_REPO_SUGGEST_DEFAULT_LIMIT: usize = 6;
const STUDIO_REPO_FACET_DEFAULT_LIMIT: usize = 6;
const STUDIO_REPO_INVENTORY_DEFAULT_LIMIT: usize = 200;

fn build_search_contract() -> UiSearchContract {
    UiSearchContract {
        contract_version: STUDIO_SEARCH_CONTRACT_VERSION.to_string(),
        code_search: UiCodeSearchContract {
            query_grammar_version: STUDIO_CODE_SEARCH_GRAMMAR_VERSION.to_string(),
            intent: "code_search".to_string(),
            backend_prefixes: REPO_CODE_SEARCH_BACKEND_PREFIXES
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            composed_prefixes: vec!["path".to_string()],
            prefix_aliases: REPO_CODE_SEARCH_PREFIX_ALIASES
                .iter()
                .map(|(alias, canonical)| UiSearchContractAlias {
                    alias: (*alias).to_string(),
                    canonical: (*canonical).to_string(),
                })
                .collect(),
            structural_prefixes: REPO_CODE_SEARCH_STRUCTURAL_PREFIXES
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            backend_kind_filters: REPO_CODE_SEARCH_KIND_FILTER_VALUES
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            routes: UiCodeSearchRoutes {
                knowledge: SEARCH_KNOWLEDGE_ROUTE.to_string(),
                intent: SEARCH_INTENT_ROUTE.to_string(),
                autocomplete: SEARCH_AUTOCOMPLETE_ROUTE.to_string(),
            },
            examples: vec![
                UiCodeSearchContractExample {
                    id: "dedupe_free_text".to_string(),
                    lane: "backend_code_search".to_string(),
                    query: "sec lang:julia sec kind:function".to_string(),
                    normalized_query: "sec lang:julia kind:function".to_string(),
                    base_query: "sec".to_string(),
                    language_filters: vec!["julia".to_string()],
                    kind_filters: vec!["function".to_string()],
                    repo_filters: Vec::new(),
                    path_filters: Vec::new(),
                },
                UiCodeSearchContractExample {
                    id: "structural_repo_query".to_string(),
                    lane: "backend_code_search".to_string(),
                    query: "repo:lancd lang:rust ast:\"fn $NAME($$$ARGS) { $$$BODY }\"".to_string(),
                    normalized_query: "repo:lancd lang:rust ast:\"fn $NAME($$$ARGS) { $$$BODY }\""
                        .to_string(),
                    base_query: "ast:\"fn $NAME($$$ARGS) { $$$BODY }\"".to_string(),
                    language_filters: vec!["rust".to_string()],
                    kind_filters: Vec::new(),
                    repo_filters: vec!["lancd".to_string()],
                    path_filters: Vec::new(),
                },
                UiCodeSearchContractExample {
                    id: "frontend_path_filter".to_string(),
                    lane: "frontend_composed_filter".to_string(),
                    query: "solver path:src/".to_string(),
                    normalized_query: "solver path:src/".to_string(),
                    base_query: "solver".to_string(),
                    language_filters: Vec::new(),
                    kind_filters: Vec::new(),
                    repo_filters: Vec::new(),
                    path_filters: vec!["src/".to_string()],
                },
            ],
        },
        repo_discovery: UiRepoDiscoveryContract {
            suggest: UiRepoDiscoverySurfaceContract {
                source: "repo_index_status".to_string(),
                default_limit: STUDIO_REPO_SUGGEST_DEFAULT_LIMIT,
                query_scoped: false,
                exhaustive: true,
            },
            facet: UiRepoDiscoverySurfaceContract {
                source: "search_results".to_string(),
                default_limit: STUDIO_REPO_FACET_DEFAULT_LIMIT,
                query_scoped: true,
                exhaustive: false,
            },
            inventory: UiRepoDiscoverySurfaceContract {
                source: "repo_index_status".to_string(),
                default_limit: STUDIO_REPO_INVENTORY_DEFAULT_LIMIT,
                query_scoped: false,
                exhaustive: true,
            },
        },
    }
}

impl StudioState {
    pub(crate) fn configured_owners_from_ui_config(config: UiConfig) -> StudioConfiguredOwners {
        StudioConfiguredOwners {
            projects: sanitize_projects(config.projects),
            repo_projects: sanitize_repo_projects(config.repo_projects),
        }
    }

    pub(crate) fn ui_config(&self) -> UiConfig {
        self.configured_owners
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ui_config()
    }

    pub(crate) fn ui_capabilities(&self) -> UiCapabilities {
        let ui_config = self.ui_config();
        let bootstrap_background_indexing = self.bootstrap_background_indexing_telemetry();
        let mut seen_repositories = HashSet::new();
        let supported_repositories = ui_config
            .repo_projects
            .iter()
            .filter_map(|project| {
                let repository_id = project.id.trim().to_string();
                if repository_id.is_empty() || !seen_repositories.insert(repository_id.clone()) {
                    return None;
                }
                Some(repository_id)
            })
            .collect();
        let plugin_languages = self
            .plugin_registry
            .plugin_ids()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let mut supported_languages = Lang::all()
            .iter()
            .copied()
            .map(Lang::as_str)
            .map(ToOwned::to_owned)
            .collect::<BTreeSet<_>>();
        supported_languages.extend(plugin_languages);
        let supported_languages = supported_languages.into_iter().collect();

        UiCapabilities {
            projects: ui_config.projects,
            repo_projects: ui_config.repo_projects,
            languages: supported_languages,
            repositories: supported_repositories,
            kinds: supported_code_kinds(),
            search_contract: build_search_contract(),
            studio_bootstrap_background_indexing_enabled: bootstrap_background_indexing.enabled(),
            studio_bootstrap_background_indexing_mode: bootstrap_background_indexing
                .mode()
                .to_string(),
            studio_bootstrap_background_indexing_deferred_activation_observed:
                bootstrap_background_indexing.deferred_activation_observed(),
        }
    }

    #[cfg(test)]
    pub(crate) fn seed_eager_configured_owners_for_tests(&self, config: UiConfig) {
        self.seed_configured_owners_for_tests(config, true);
    }

    pub(crate) fn bootstrap_runtime_ui_config(
        &self,
        config: UiConfig,
        eager_background_indexing: bool,
    ) {
        self.sync_configured_runtime_owners(
            config,
            eager_background_indexing,
            BOOTSTRAP_RUNTIME_SOURCE,
        );
    }

    fn ensure_repo_background_indexing_started(&self, source: &'static str) {
        let repositories = configured_repositories(self);
        if repositories.is_empty() {
            return;
        }

        self.record_deferred_bootstrap_background_indexing_activation(source);
        self.repo_index.sync_repositories(repositories);
    }

    fn ensure_background_indexes_started(&self, source: &'static str) {
        let configured_projects = self.configured_projects();
        if !configured_projects.is_empty() {
            self.record_deferred_bootstrap_background_indexing_activation(source);
            let search_projects = configured_projects.clone();
            let scan_inventory = self
                .search_plane
                .scan_supported_projects_with_repeat_work_details(
                    source,
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    search_projects.as_slice(),
                );
            let note_files = scan_inventory.note_files();
            let source_files = scan_inventory.source_files();
            self.symbol_index_coordinator
                .sync_projects(search_projects.clone(), Arc::clone(&self.symbol_index));
            if self
                .search_plane
                .ensure_knowledge_section_index_started_with_scanned_files(
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    search_projects.as_slice(),
                    note_files.as_slice(),
                )
            {
                self.record_local_corpus_index_started(SearchCorpusKind::KnowledgeSection, source);
            }
            if self
                .search_plane
                .ensure_local_symbol_index_started_with_scanned_files(
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    search_projects.as_slice(),
                    scan_inventory.symbol_files(),
                )
            {
                self.record_local_corpus_index_started(SearchCorpusKind::LocalSymbol, source);
            }
            if self
                .search_plane
                .ensure_attachment_index_started_with_scanned_files(
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    search_projects.as_slice(),
                    note_files.as_slice(),
                )
            {
                self.record_local_corpus_index_started(SearchCorpusKind::Attachment, source);
            }
            if self
                .search_plane
                .ensure_reference_occurrence_index_started_with_scanned_files(
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    search_projects.as_slice(),
                    source_files.as_slice(),
                )
            {
                self.record_local_corpus_index_started(
                    SearchCorpusKind::ReferenceOccurrence,
                    source,
                );
            }
        }
        self.ensure_repo_background_indexing_started(source);
    }

    fn sync_configured_runtime_owners(
        &self,
        config: UiConfig,
        eager_background_indexing: bool,
        source: &'static str,
    ) {
        let configured_owners = Self::configured_owners_from_ui_config(config);
        let mut guard = self
            .configured_owners
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *guard == configured_owners {
            drop(guard);
            if eager_background_indexing {
                self.ensure_background_indexes_started(source);
            }
            return;
        }
        *guard = configured_owners;
        drop(guard);

        let mut graph_guard = self
            .graph_index
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *graph_guard = None;
        drop(graph_guard);

        let mut symbol_guard = self
            .symbol_index
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *symbol_guard = None;
        drop(symbol_guard);

        let mut vfs_guard = self
            .vfs_scan
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *vfs_guard = None;
        drop(vfs_guard);

        if eager_background_indexing {
            self.ensure_background_indexes_started(source);
        }
    }

    #[cfg(test)]
    pub(crate) fn seed_configured_owners_for_tests(
        &self,
        config: UiConfig,
        eager_background_indexing: bool,
    ) {
        self.sync_configured_runtime_owners(
            config,
            eager_background_indexing,
            TEST_CONFIGURED_OWNER_SEED_SOURCE,
        );
    }

    pub(crate) fn repo_index_status(&self, repo: Option<&str>) -> RepoIndexStatusResponse {
        let status = self.repo_index.status_response(repo);
        if status.total > 0 {
            return status;
        }

        self.ensure_repo_background_indexing_started("repo_index_status");
        self.repo_index.status_response(repo)
    }

    pub(crate) fn configured_projects(&self) -> Vec<UiProjectConfig> {
        self.configured_owners
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .projects
            .clone()
    }

    pub(crate) fn configured_repo_projects(&self) -> Vec<UiRepoProjectConfig> {
        self.configured_owners
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_projects
            .clone()
    }
}
