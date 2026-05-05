//! Registry bootstrap for the linked builtin plugin bundle.

use xiuxian_wendao_core::repo_intelligence::{
    PluginRegistry, RepoIntelligenceError, builtin_plugin_registrars,
};

use crate::link;

/// Register built-in repo-intelligence plugins into a fresh registry.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] if a linked builtin plugin registrar
/// fails while registering into the fresh registry.
pub fn bootstrap_builtin_registry() -> Result<PluginRegistry, RepoIntelligenceError> {
    link::ensure_builtin_plugins_linked();
    let mut registry = PluginRegistry::new();
    let mut registrars = builtin_plugin_registrars();
    registrars.sort_by(|left, right| left.plugin_id().cmp(right.plugin_id()));
    for registrar in registrars {
        registrar.register(&mut registry)?;
    }

    Ok(registry)
}
