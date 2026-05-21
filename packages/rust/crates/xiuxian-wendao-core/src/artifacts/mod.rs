//! Artifact selectors, launches, and payloads for Wendao tool results.

mod launch;
mod payload;
mod selector;

pub use launch::PluginLaunchSpec;
pub use payload::PluginArtifactPayload;
pub use selector::PluginArtifactSelector;
