//! Builtin artifact selectors and serialized fixture rendering.

mod dispatch;
mod gateway;
mod openapi;
#[cfg(test)]
#[path = "../../tests/unit/artifacts/mod.rs"]
mod tests;

pub use dispatch::{
    render_builtin_plugin_artifact_toml_for_selector,
    render_builtin_plugin_artifact_toml_for_selector_with_settings,
    resolve_builtin_plugin_artifact_for_selector,
    resolve_builtin_plugin_artifact_for_selector_with_settings,
};
pub use gateway::{
    linked_builtin_julia_gateway_artifact_base_url,
    linked_builtin_julia_gateway_artifact_expected_json_fragments,
    linked_builtin_julia_gateway_artifact_expected_toml_fragments,
    linked_builtin_julia_gateway_artifact_path, linked_builtin_julia_gateway_artifact_route,
    linked_builtin_julia_gateway_artifact_rpc_params_fixture,
    linked_builtin_julia_gateway_artifact_runtime_config_toml,
    linked_builtin_julia_gateway_artifact_schema_version,
    linked_builtin_julia_gateway_artifact_selected_transport,
    linked_builtin_julia_gateway_artifact_ui_payload_fixture,
    linked_builtin_julia_gateway_launcher_path,
};
pub use openapi::{
    linked_builtin_julia_deployment_artifact_openapi_json_example,
    linked_builtin_julia_deployment_artifact_openapi_toml_example,
    linked_builtin_plugin_artifact_openapi_json_example,
    linked_builtin_plugin_artifact_openapi_toml_example,
};
