//! `OpenAPI` examples for linked builtin artifact routes.

use serde_json::Value;

use xiuxian_julia_runtime::wendao::link_graph::{
    julia_deployment_artifact_openapi_json_example, julia_deployment_artifact_openapi_toml_example,
    julia_plugin_artifact_openapi_json_example, julia_plugin_artifact_openapi_toml_example,
};

/// Return the current linked builtin example for the generic plugin-artifact
/// `OpenAPI` route.
#[must_use]
pub fn linked_builtin_plugin_artifact_openapi_json_example() -> Value {
    julia_plugin_artifact_openapi_json_example()
}

/// Return the current linked builtin TOML example for the generic
/// plugin-artifact `OpenAPI` route.
#[must_use]
pub fn linked_builtin_plugin_artifact_openapi_toml_example() -> String {
    julia_plugin_artifact_openapi_toml_example()
}

/// Return the current linked builtin JSON example for the legacy
/// Julia-deployment-artifact `OpenAPI` route.
#[must_use]
pub fn linked_builtin_julia_deployment_artifact_openapi_json_example() -> Value {
    julia_deployment_artifact_openapi_json_example()
}

/// Return the current linked builtin TOML example for the legacy
/// Julia-deployment-artifact `OpenAPI` route.
///
/// # Errors
///
/// Returns an error when the linked builtin example cannot be serialized into
/// TOML.
pub fn linked_builtin_julia_deployment_artifact_openapi_toml_example()
-> Result<String, toml::ser::Error> {
    julia_deployment_artifact_openapi_toml_example()
}
