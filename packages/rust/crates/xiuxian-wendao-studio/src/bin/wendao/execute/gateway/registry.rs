//! Gateway plugin registry bootstrap.

use std::sync::Arc;

use anyhow::Result;

use xiuxian_wendao::analyzers::bootstrap_builtin_registry;

/// Build the plugin registry for the gateway.
pub(crate) fn build_plugin_registry() -> Result<Arc<xiuxian_wendao::analyzers::PluginRegistry>> {
    Ok(Arc::new(bootstrap_builtin_registry()?))
}
