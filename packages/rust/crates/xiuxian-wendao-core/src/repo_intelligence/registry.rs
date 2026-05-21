//! Registry for resolving repository-intelligence plugins by typed plugin ids.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::config::RegisteredRepository;
use super::errors::RepoIntelligenceError;
use super::plugin::RepoIntelligencePlugin;
use super::records::RepoIntelligencePluginId;

/// In-memory plugin registry for Repo Intelligence analyzers.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: BTreeMap<String, Arc<dyn RepoIntelligencePlugin>>,
}

impl PluginRegistry {
    /// Construct an empty plugin registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one plugin implementation.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] if a plugin with the same identifier
    /// has already been registered.
    pub fn register<P>(&mut self, plugin: P) -> Result<(), RepoIntelligenceError>
    where
        P: RepoIntelligencePlugin + 'static,
    {
        let plugin_id = plugin.id().to_string();
        if self.plugins.contains_key(&plugin_id) {
            return Err(RepoIntelligenceError::DuplicatePlugin {
                plugin_id: plugin_id.into(),
            });
        }

        self.plugins.insert(plugin_id, Arc::new(plugin));
        Ok(())
    }

    /// Fetch a plugin by identifier.
    #[must_use]
    pub fn get(
        &self,
        plugin_id: impl Into<RepoIntelligencePluginId>,
    ) -> Option<Arc<dyn RepoIntelligencePlugin>> {
        let plugin_id = plugin_id.into();
        self.plugins.get(plugin_id.as_str()).map(Arc::clone)
    }

    /// Return the registered plugin identifiers in stable order.
    #[must_use]
    pub fn plugin_ids(&self) -> Vec<&str> {
        self.plugins.keys().map(String::as_str).collect()
    }

    /// Resolve all plugins configured for a repository.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when a configured plugin identifier
    /// is missing from the registry.
    pub fn resolve_for_repository(
        &self,
        repository: &RegisteredRepository,
    ) -> Result<Vec<Arc<dyn RepoIntelligencePlugin>>, RepoIntelligenceError> {
        repository
            .repo_intelligence_plugins()
            .map(|plugin| self.resolve_plugin_for_repository(repository, plugin.id()))
            .filter_map(Result::transpose)
            .collect()
    }

    fn resolve_plugin_for_repository(
        &self,
        repository: &RegisteredRepository,
        plugin_id: &str,
    ) -> Result<Option<Arc<dyn RepoIntelligencePlugin>>, RepoIntelligenceError> {
        let registered =
            self.get(plugin_id)
                .ok_or_else(|| RepoIntelligenceError::MissingPlugin {
                    plugin_id: plugin_id.into(),
                })?;
        Ok(registered
            .supports_repository(repository)
            .then_some(registered))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/repo_intelligence/registry.rs"]
mod tests;
