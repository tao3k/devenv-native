use std::collections::BTreeSet;

use crate::gateway::{
    API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH, API_HEALTH_OPENAPI_PATH, API_NOTIFY_OPENAPI_PATH,
    API_REPO_SYNC_OPENAPI_PATH, API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH,
    WENDAO_GATEWAY_ROUTE_CONTRACTS,
};

const RETIRED_SEARCH_AST_OPENAPI_PATH: &str = "/api/search/ast";
const RETIRED_SEARCH_DEFINITION_OPENAPI_PATH: &str = "/api/search/definition";
const RETIRED_SEARCH_AUTOCOMPLETE_OPENAPI_PATH: &str = "/api/search/autocomplete";
const RETIRED_SEARCH_KNOWLEDGE_OPENAPI_PATH: &str = "/api/search";
const RETIRED_SEARCH_INTENT_OPENAPI_PATH: &str = "/api/search/intent";
const RETIRED_SEARCH_ATTACHMENTS_OPENAPI_PATH: &str = "/api/search/attachments";
const RETIRED_SEARCH_REFERENCES_OPENAPI_PATH: &str = "/api/search/references";
const RETIRED_SEARCH_SYMBOLS_OPENAPI_PATH: &str = "/api/search/symbols";
const RETIRED_GRAPH_NEIGHBORS_OPENAPI_PATH: &str = "/api/graph/neighbors/{id}";
const RETIRED_NODE_NEIGHBORS_OPENAPI_PATH: &str = "/api/neighbors/{id}";
const RETIRED_ANALYSIS_MARKDOWN_OPENAPI_PATH: &str = "/api/analysis/markdown";
const RETIRED_ANALYSIS_CODE_AST_OPENAPI_PATH: &str = "/api/analysis/code-ast";
const RETIRED_UI_CONFIG_OPENAPI_PATH: &str = "/api/ui/config";

#[test]
fn route_inventory_keeps_core_endpoints() {
    let openapi_paths = WENDAO_GATEWAY_ROUTE_CONTRACTS
        .iter()
        .map(|route| route.openapi_path)
        .collect::<BTreeSet<_>>();

    assert!(openapi_paths.contains(API_HEALTH_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_NOTIFY_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_REPO_SYNC_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH));
}

#[test]
fn route_inventory_paths_are_unique() {
    let openapi_paths = WENDAO_GATEWAY_ROUTE_CONTRACTS
        .iter()
        .map(|route| route.openapi_path)
        .collect::<BTreeSet<_>>();

    assert_eq!(openapi_paths.len(), WENDAO_GATEWAY_ROUTE_CONTRACTS.len());
}

#[test]
fn route_inventory_omits_retired_flight_only_http_paths() {
    let openapi_paths = WENDAO_GATEWAY_ROUTE_CONTRACTS
        .iter()
        .map(|route| route.openapi_path)
        .collect::<BTreeSet<_>>();

    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_KNOWLEDGE_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired knowledge HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_DEFINITION_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired definition HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_AUTOCOMPLETE_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired autocomplete HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_INTENT_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired intent HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_ATTACHMENTS_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired attachment HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_AST_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired AST HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_REFERENCES_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired references HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_SEARCH_SYMBOLS_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired symbols HTTP search path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_GRAPH_NEIGHBORS_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired graph-neighbors HTTP path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_NODE_NEIGHBORS_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired node-neighbors HTTP path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_ANALYSIS_MARKDOWN_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired markdown HTTP analysis path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_ANALYSIS_CODE_AST_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired code-AST HTTP analysis path"
    );
    assert!(
        !openapi_paths.contains(RETIRED_UI_CONFIG_OPENAPI_PATH),
        "stable shared route inventory must not re-expose retired UI config HTTP path"
    );
}

#[test]
fn generic_plugin_artifact_route_contract_matches_canonical_path() {
    assert_eq!(
        super::inventory::UI_PLUGIN_ARTIFACT.openapi_path,
        API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH
    );
    assert_eq!(
        super::inventory::UI_PLUGIN_ARTIFACT.path_params,
        ["plugin_id", "artifact_id"]
    );
}
