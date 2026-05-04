//! Studio-owned capability and search contract manifests.

use serde::{Deserialize, Serialize};
use specta::Type;

pub use xiuxian_wendao::search::contracts::{UiProjectConfig, UiRepoProjectConfig};

/// Global UI configuration for Studio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    /// Local project roots to scan.
    pub projects: Vec<UiProjectConfig>,
    /// External repository projects.
    pub repo_projects: Vec<UiRepoProjectConfig>,
}

/// Gateway-reported studio capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiCapabilities {
    /// Local project roots available to the current Studio runtime.
    pub projects: Vec<UiProjectConfig>,
    /// External repository projects available to the current Studio runtime.
    pub repo_projects: Vec<UiRepoProjectConfig>,
    /// Supported language identifiers reported by the gateway capability surface.
    #[serde(rename = "supportedLanguages")]
    pub languages: Vec<String>,
    /// Supported repository identifiers reported by the gateway UI config.
    #[serde(rename = "supportedRepositories")]
    pub repositories: Vec<String>,
    /// Supported code filter kinds reported by the gateway capability surface.
    #[serde(rename = "supportedKinds")]
    pub kinds: Vec<String>,
    /// Rust-owned search contract manifest for frontend search alignment.
    pub search_contract: UiSearchContract,
    /// Whether bootstrap-time background indexing is enabled during gateway startup.
    pub studio_bootstrap_background_indexing_enabled: bool,
    /// Stable mode label for bootstrap-time background indexing during gateway startup.
    pub studio_bootstrap_background_indexing_mode: String,
    /// Whether deferred bootstrap indexing has been lazily activated since process boot.
    pub studio_bootstrap_background_indexing_deferred_activation_observed: bool,
}

/// Rust-owned search contract manifest for Studio frontend consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiSearchContract {
    /// Stable schema version for the exported contract payload.
    pub contract_version: String,
    /// Code-search grammar and lane contract.
    pub code_search: UiCodeSearchContract,
    /// Repo-discovery surface semantics for suggestion, facet, and inventory consumers.
    pub repo_discovery: UiRepoDiscoveryContract,
}

/// Studio code-search contract exported for frontend validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiCodeSearchContract {
    /// Stable grammar version for the backend parser-backed query shape.
    pub query_grammar_version: String,
    /// Search intent label used for backend code-search requests.
    pub intent: String,
    /// Parser-owned backend prefixes accepted by the Rust gateway.
    pub backend_prefixes: Vec<String>,
    /// Frontend-composed prefixes that stay valid within the Studio search UI.
    pub composed_prefixes: Vec<String>,
    /// Stable alias mappings accepted by the grammar.
    pub prefix_aliases: Vec<UiSearchContractAlias>,
    /// Structural prefixes preserved by the frontend and interpreted by Rust.
    pub structural_prefixes: Vec<String>,
    /// Backend-supported `kind:` directive values.
    pub backend_kind_filters: Vec<String>,
    /// Stable route bindings used by code-search transport paths.
    pub routes: UiCodeSearchRoutes,
    /// Normative examples used by frontend contract validation.
    pub examples: Vec<UiCodeSearchContractExample>,
}

/// One accepted alias for a canonical search prefix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiSearchContractAlias {
    /// Accepted alias token.
    pub alias: String,
    /// Canonical token expected by the stable contract surface.
    pub canonical: String,
}

/// Stable route bindings for Studio code-search control-plane requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiCodeSearchRoutes {
    /// Backend route for `code_search` intent requests.
    pub knowledge: String,
    /// Backend route for intent classification requests.
    pub intent: String,
    /// Backend route for autocomplete requests.
    pub autocomplete: String,
}

/// One normative code-search query example exported by the Rust contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiCodeSearchContractExample {
    /// Stable example identifier.
    pub id: String,
    /// Execution lane label for the example.
    pub lane: String,
    /// Example user query accepted by the frontend search surface.
    pub query: String,
    /// Expected normalized query after frontend canonicalization.
    pub normalized_query: String,
    /// Expected free-text base query after filter extraction.
    pub base_query: String,
    /// Expected normalized language filters.
    pub language_filters: Vec<String>,
    /// Expected normalized kind filters.
    pub kind_filters: Vec<String>,
    /// Expected normalized repo filters.
    pub repo_filters: Vec<String>,
    /// Expected normalized path filters.
    pub path_filters: Vec<String>,
}

/// Studio repo-discovery contract exported for frontend validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiRepoDiscoveryContract {
    /// Prefix-oriented repo suggestion surface.
    pub suggest: UiRepoDiscoverySurfaceContract,
    /// Query-scoped repo facet surface.
    pub facet: UiRepoDiscoverySurfaceContract,
    /// Exhaustive repo inventory surface.
    pub inventory: UiRepoDiscoverySurfaceContract,
}

/// One repo-discovery surface exported by the Studio contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiRepoDiscoverySurfaceContract {
    /// Stable owner label for the underlying source surface.
    pub source: String,
    /// Default UI budget or browse window for this surface.
    pub default_limit: usize,
    /// Whether the surface is scoped to the active query/result set.
    pub query_scoped: bool,
    /// Whether the surface is allowed to claim exhaustive repo coverage.
    pub exhaustive: bool,
}
