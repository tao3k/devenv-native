//! Studio gateway capability handlers.

mod deployment;
mod service;
mod types;

pub(crate) use deployment::get_plugin_artifact;
pub(crate) use service::get;
#[cfg(test)]
pub(crate) use types::{PluginArtifactPath, PluginArtifactQuery};
