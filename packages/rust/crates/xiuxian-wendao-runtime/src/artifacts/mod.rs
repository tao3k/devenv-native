//! Runtime artifact rendering and path-resolution boundary for generated Wendao assets.

/// Runtime-owned bundled `OpenAPI` artifact helpers.
pub mod openapi;

/// Runtime-owned embedded zhixing artifact helpers.
pub mod zhixing;

mod identifiers;
mod render;
mod resolve;

pub use identifiers::{RuntimeArtifactIdRef, RuntimePluginIdRef};
pub use render::{render_plugin_artifact_toml_for_selector_with, render_plugin_artifact_toml_with};
pub use resolve::{resolve_plugin_artifact_for_selector_with, resolve_plugin_artifact_with};
