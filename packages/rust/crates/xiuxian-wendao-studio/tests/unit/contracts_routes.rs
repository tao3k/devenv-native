use std::collections::BTreeSet;

#[cfg(feature = "openapi-artifacts")]
use serde_json::Value;
use xiuxian_wendao_studio::contracts::WENDAO_GATEWAY_ROUTE_CONTRACTS;
use xiuxian_wendao_studio::contracts::routes::{
    API_AUTH_TOKENS_OPENAPI_PATH, API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH, API_HEALTH_OPENAPI_PATH,
    API_NOTIFY_OPENAPI_PATH, API_REPO_SYNC_OPENAPI_PATH, API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH,
};
#[cfg(feature = "openapi-artifacts")]
use xiuxian_wendao_studio::openapi::load_bundled_wendao_gateway_openapi_document;

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
    let openapi_paths = route_openapi_paths();

    assert!(openapi_paths.contains(API_HEALTH_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_NOTIFY_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_AUTH_TOKENS_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_REPO_SYNC_OPENAPI_PATH));
    assert!(openapi_paths.contains(API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH));
}

#[test]
fn route_inventory_paths_are_unique() {
    let openapi_paths = route_openapi_paths();

    assert_eq!(openapi_paths.len(), WENDAO_GATEWAY_ROUTE_CONTRACTS.len());
}

#[test]
fn route_inventory_omits_retired_flight_only_http_paths() {
    let openapi_paths = route_openapi_paths();

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
        "stable shared route inventory must not re-expose retired symbols HTTP path"
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
    let Some(route) = WENDAO_GATEWAY_ROUTE_CONTRACTS
        .iter()
        .find(|route| route.openapi_path == API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH)
    else {
        panic!("generic plugin artifact route should be declared");
    };

    assert_eq!(route.path_params, ["plugin_id", "artifact_id"]);
}

fn route_openapi_paths() -> BTreeSet<&'static str> {
    WENDAO_GATEWAY_ROUTE_CONTRACTS
        .iter()
        .map(|route| route.openapi_path)
        .collect()
}

#[cfg(feature = "openapi-artifacts")]
fn operation_summary(operation: &Value) -> &str {
    operation
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[cfg(feature = "openapi-artifacts")]
fn operation_description(operation: &Value) -> &str {
    operation
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[cfg(feature = "openapi-artifacts")]
#[test]
fn bundled_gateway_openapi_document_covers_declared_route_inventory() {
    let document = load_bundled_wendao_gateway_openapi_document()
        .unwrap_or_else(|error| panic!("bundled gateway OpenAPI should parse: {error}"));
    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        panic!("bundled gateway OpenAPI should contain a `paths` object");
    };

    for route in WENDAO_GATEWAY_ROUTE_CONTRACTS {
        let Some(path_item) = paths.get(route.openapi_path).and_then(Value::as_object) else {
            panic!(
                "bundled gateway OpenAPI should document path {}",
                route.openapi_path
            );
        };

        for method in route.methods {
            let Some(operation) = path_item.get(*method) else {
                panic!(
                    "bundled gateway OpenAPI should document {} {}",
                    method, route.openapi_path
                );
            };
            assert!(
                !operation_summary(operation).trim().is_empty(),
                "{} {} should include a non-empty summary",
                method,
                route.openapi_path
            );
            assert!(
                !operation_description(operation).trim().is_empty(),
                "{} {} should include a non-empty description",
                method,
                route.openapi_path
            );

            let Some(responses) = operation.get("responses").and_then(Value::as_object) else {
                panic!(
                    "{} {} should include OpenAPI responses",
                    method, route.openapi_path
                );
            };
            assert!(
                !responses.is_empty(),
                "{} {} should document at least one response",
                method,
                route.openapi_path
            );
            for (status, response) in responses {
                let description = response
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                assert!(
                    !description.trim().is_empty(),
                    "{} {} response {} should include a non-empty description",
                    method,
                    route.openapi_path,
                    status
                );
            }

            if !route.path_params.is_empty() {
                let Some(parameters) = operation.get("parameters").and_then(Value::as_array) else {
                    panic!(
                        "{} {} should include path parameter declarations",
                        method, route.openapi_path
                    );
                };
                for required_param in route.path_params {
                    let matches_param = parameters.iter().any(|parameter| {
                        parameter.get("name").and_then(Value::as_str) == Some(*required_param)
                            && parameter.get("in").and_then(Value::as_str) == Some("path")
                            && parameter.get("required").and_then(Value::as_bool) == Some(true)
                    });
                    assert!(
                        matches_param,
                        "{} {} should declare required path parameter `{}`",
                        method, route.openapi_path, required_param
                    );
                }
            }
        }
    }
}
