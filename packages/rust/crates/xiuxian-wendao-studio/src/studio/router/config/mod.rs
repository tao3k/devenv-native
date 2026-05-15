//! TOML configuration loading for Studio API bootstrap.

mod load;
mod paths;
mod sanitize;
#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/router/config/mod.rs"]
mod tests;
mod types;

pub(crate) use load::load_document_extract_endpoint_from_wendao_toml;
pub(crate) use load::load_episteme_registry_from_wendao_toml;
#[cfg(feature = "cli-bin-support")]
pub(crate) use load::load_episteme_registry_from_wendao_toml_path;
pub use load::{load_ui_config_from_wendao_toml, load_ui_config_from_wendao_toml_path};
pub use paths::{
    resolve_studio_config_root, studio_effective_wendao_toml_path, studio_wendao_overlay_toml_path,
    studio_wendao_toml_path,
};
