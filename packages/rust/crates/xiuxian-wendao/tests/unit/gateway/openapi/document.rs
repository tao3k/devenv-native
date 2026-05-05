use serde_json::Value;
#[cfg(feature = "julia")]
use xiuxian_wendao_builtin::{
    linked_builtin_julia_deployment_artifact_openapi_json_example,
    linked_builtin_julia_deployment_artifact_openapi_toml_example,
    linked_builtin_plugin_artifact_openapi_json_example,
    linked_builtin_plugin_artifact_openapi_toml_example,
};

use super::{
    bundled_wendao_gateway_openapi_document, bundled_wendao_gateway_openapi_path,
    load_bundled_wendao_gateway_openapi_document,
};

#[test]
fn bundled_gateway_openapi_document_is_valid_json() {
    let document = load_bundled_wendao_gateway_openapi_document()
        .unwrap_or_else(|error| panic!("bundled gateway OpenAPI should parse: {error}"));

    assert_eq!(document["openapi"], Value::String("3.1.0".to_string()));
    assert_eq!(
        document["info"]["title"],
        Value::String("Wendao Gateway".to_string())
    );
    assert!(
        bundled_wendao_gateway_openapi_path().is_file(),
        "bundled gateway OpenAPI path should exist on disk"
    );
    assert!(
        bundled_wendao_gateway_openapi_document().contains("\"paths\""),
        "bundled gateway OpenAPI text should include paths"
    );
}

#[test]
fn bundled_gateway_openapi_document_declares_public_json_bearer_boundary() {
    let document = load_bundled_wendao_gateway_openapi_document()
        .unwrap_or_else(|error| panic!("bundled gateway OpenAPI should parse: {error}"));

    let description = document["info"]["description"].as_str().unwrap_or_default();
    assert!(
        description.contains("HTTPS JSON"),
        "bundled gateway OpenAPI should describe the public JSON API boundary"
    );
    assert!(
        description.contains("Accept: text/event-stream"),
        "bundled gateway OpenAPI should describe the public SSE streaming boundary"
    );
    assert_eq!(
        document["security"][0]["WendaoBearerAuth"],
        Value::Array(vec![])
    );
    assert_eq!(
        document["components"]["securitySchemes"]["WendaoBearerAuth"]["type"],
        Value::String("http".to_string())
    );
    assert_eq!(
        document["components"]["securitySchemes"]["WendaoBearerAuth"]["scheme"],
        Value::String("bearer".to_string())
    );
    assert_eq!(
        document["components"]["securitySchemes"]["WendaoBearerAuth"]["bearerFormat"],
        Value::String("Wendao bearer token, for example wd_...".to_string())
    );
    assert_eq!(
        document["paths"]["/api/health"]["get"]["security"],
        Value::Array(vec![])
    );
    assert!(
        document["paths"]["/v1/responses"]["post"]["responses"]["200"]["content"]
            .get("text/event-stream")
            .is_some(),
        "public responses route should document the SSE response contract"
    );
    assert_eq!(
        document["paths"]["/v1/responses"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        Value::String("#/components/schemas/GatewayResponseRequest".to_string())
    );
    assert_eq!(
        document["components"]["schemas"]["GatewayResponse"]["properties"]["object"]["const"],
        Value::String("response".to_string())
    );
}

#[cfg(feature = "julia")]
#[test]
fn bundled_gateway_openapi_document_declares_rerank_plugin_artifact_examples() {
    let document = load_bundled_wendao_gateway_openapi_document()
        .unwrap_or_else(|error| panic!("bundled gateway OpenAPI should parse: {error}"));
    let get = &document["paths"]["/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}"]["get"];
    let expected_toml = linked_builtin_plugin_artifact_openapi_toml_example();

    assert_eq!(
        get["responses"]["200"]["content"]["application/json"]["example"],
        linked_builtin_plugin_artifact_openapi_json_example()
    );
    assert_eq!(
        get["responses"]["200"]["content"]["text/plain"]["example"].as_str(),
        Some(expected_toml.as_str())
    );
}

#[cfg(feature = "julia")]
#[test]
fn bundled_gateway_openapi_document_declares_rerank_julia_deployment_artifact_examples() {
    let document = load_bundled_wendao_gateway_openapi_document()
        .unwrap_or_else(|error| panic!("bundled gateway OpenAPI should parse: {error}"));
    let get = &document["paths"]["/api/ui/julia-deployment-artifact"]["get"];
    let expected_toml = linked_builtin_julia_deployment_artifact_openapi_toml_example()
        .unwrap_or_else(|error| {
            panic!("render Julia deployment artifact OpenAPI example: {error}")
        });

    assert_eq!(
        get["responses"]["200"]["content"]["application/json"]["example"],
        linked_builtin_julia_deployment_artifact_openapi_json_example()
    );
    assert_eq!(
        get["responses"]["200"]["content"]["text/plain"]["example"].as_str(),
        Some(expected_toml.as_str())
    );
}

#[test]
fn bundled_gateway_openapi_document_omits_flight_only_http_paths() {
    let document = load_bundled_wendao_gateway_openapi_document()
        .unwrap_or_else(|error| panic!("bundled gateway OpenAPI should parse: {error}"));
    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        panic!("bundled gateway OpenAPI should contain a `paths` object");
    };

    assert!(
        !paths.contains_key("/api/search"),
        "bundled gateway OpenAPI must not expose the retired knowledge HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/definition"),
        "bundled gateway OpenAPI must not expose the retired definition HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/autocomplete"),
        "bundled gateway OpenAPI must not expose the retired autocomplete HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/intent"),
        "bundled gateway OpenAPI must not expose the retired intent HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/attachments"),
        "bundled gateway OpenAPI must not expose the retired attachments HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/references"),
        "bundled gateway OpenAPI must not expose the retired references HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/symbols"),
        "bundled gateway OpenAPI must not expose the retired symbols HTTP path"
    );
    assert!(
        !paths.contains_key("/api/search/ast"),
        "bundled gateway OpenAPI must not expose the retired AST HTTP path"
    );
    assert!(
        !paths.contains_key("/api/graph/neighbors/{id}"),
        "bundled gateway OpenAPI must not expose the retired graph-neighbors HTTP path"
    );
    assert!(
        !paths.contains_key("/api/neighbors/{id}"),
        "bundled gateway OpenAPI must not expose the retired node-neighbors HTTP path"
    );
    assert!(
        !paths.contains_key("/api/analysis/markdown"),
        "bundled gateway OpenAPI must not expose the retired markdown HTTP path"
    );
    assert!(
        !paths.contains_key("/api/analysis/code-ast"),
        "bundled gateway OpenAPI must not expose the retired code-ast HTTP path"
    );
    assert!(
        !paths.contains_key("/api/ui/config"),
        "bundled gateway OpenAPI must not expose the retired UI config HTTP path"
    );
    assert!(
        !paths.contains_key("/api/analysis/markdown/retrieval-arrow"),
        "bundled gateway OpenAPI must not expose the retired markdown retrieval-arrow path"
    );
    assert!(
        !paths.contains_key("/api/analysis/code-ast/retrieval-arrow"),
        "bundled gateway OpenAPI must not expose the retired code-ast retrieval-arrow path"
    );
    assert!(
        !paths.contains_key("/arrow.flight.protocol.FlightService/{*grpc_method}"),
        "bundled gateway OpenAPI must not expose the Flight transport route as public JSON API"
    );
}
