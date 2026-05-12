//! `analyzers::service::bootstrap` owns Wendao analyzers service bootstrap behavior.

#[cfg(feature = "builtin-plugins")]
pub use xiuxian_wendao_builtin::bootstrap_builtin_registry;
#[cfg(not(feature = "builtin-plugins"))]
use xiuxian_wendao_core::repo_intelligence::{PluginRegistry, RepoIntelligenceError};

#[cfg(not(feature = "builtin-plugins"))]
/// Report that builtin plugin bootstrap is unavailable in lightweight builds.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::ConfigLoad`] because lightweight builds do not ship the
/// builtin plugin registry.
pub fn bootstrap_builtin_registry() -> Result<PluginRegistry, RepoIntelligenceError> {
    Err(RepoIntelligenceError::ConfigLoad {
        message: "builtin plugin registry is unavailable because `xiuxian-wendao` was built without the `builtin-plugins` feature".to_string(),
    })
}

#[cfg(all(test, feature = "julia"))]
#[path = "../../../tests/unit/analyzers/service/bootstrap.rs"]
mod tests;
