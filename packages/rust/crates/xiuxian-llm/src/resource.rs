//! Source-owned system configuration resources for `xiuxian-llm`.

use std::path::Path;

use xiuxian_config_core::load_toml_value_with_imports_and_paths;

pub(crate) const WENDAO_LLM_SYSTEM_DEFAULT_TOML: &str = include_str!("resource/llm.toml");
const WENDAO_LLM_SYSTEM_DEFAULT_TOML_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/resource/llm.toml");

pub(crate) const WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML: &str =
    include_str!("resource/model_routing.toml");
const WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/resource/model_routing.toml"
);

pub(crate) fn load_llm_system_default_toml_value() -> Result<toml::Value, String> {
    load_source_toml_value(
        WENDAO_LLM_SYSTEM_DEFAULT_TOML,
        WENDAO_LLM_SYSTEM_DEFAULT_TOML_PATH,
        "Wendao LLM defaults",
    )
}

pub(crate) fn load_model_routing_system_default_toml_value() -> Result<toml::Value, String> {
    load_source_toml_value(
        WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML,
        WENDAO_MODEL_ROUTING_SYSTEM_DEFAULT_TOML_PATH,
        "Wendao model-routing defaults",
    )
}

fn load_source_toml_value(
    raw: &str,
    source_path: &str,
    label: &str,
) -> Result<toml::Value, String> {
    match load_toml_value_with_imports_and_paths(Path::new(source_path), None, None) {
        Ok(value) => Ok(value),
        Err(_) => toml::from_str::<toml::Value>(raw)
            .map_err(|error| format!("parse embedded {label}: {error}")),
    }
}
