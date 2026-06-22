//! `memory::julia::runtime` owns Wendao memory julia runtime behavior.

use crate::settings::merged_wendao_settings;
use xiuxian_julia_runtime::wendao::build_memory_julia_compute_bindings;
use xiuxian_wendao_core::{
    capabilities::PluginCapabilityBinding, repo_intelligence::RepoIntelligenceError,
};
use xiuxian_wendao_runtime::config::{
    MemoryJuliaComputeRuntimeConfig, resolve_memory_julia_compute_runtime_with_settings,
};

/// Resolve the current `memory.julia_compute` runtime configuration from the
/// merged Wendao settings surface.
#[must_use]
pub fn resolve_memory_julia_compute_runtime() -> MemoryJuliaComputeRuntimeConfig {
    let settings = merged_wendao_settings();
    resolve_memory_julia_compute_runtime_with_settings(&settings)
}

/// Resolve the current memory-family Julia capability bindings from merged
/// Wendao settings.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the resolved runtime config cannot be
/// normalized into valid generic plugin capability bindings.
pub fn resolve_memory_julia_compute_bindings()
-> Result<Vec<PluginCapabilityBinding>, RepoIntelligenceError> {
    let runtime = resolve_memory_julia_compute_runtime();
    build_memory_julia_compute_bindings(&runtime)
}

pub(crate) fn ensure_enabled_memory_julia_compute_runtime(
    runtime: MemoryJuliaComputeRuntimeConfig,
    target: &str,
) -> Result<MemoryJuliaComputeRuntimeConfig, RepoIntelligenceError> {
    if !runtime.enabled {
        return Err(RepoIntelligenceError::ConfigLoad {
            message: format!("memory Julia compute runtime is disabled for `{target}`"),
        });
    }
    Ok(runtime)
}

pub(crate) fn resolve_enabled_memory_julia_compute_runtime(
    target: &str,
) -> Result<MemoryJuliaComputeRuntimeConfig, RepoIntelligenceError> {
    ensure_enabled_memory_julia_compute_runtime(resolve_memory_julia_compute_runtime(), target)
}
