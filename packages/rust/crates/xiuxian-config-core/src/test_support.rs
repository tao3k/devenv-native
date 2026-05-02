//! Test-only helpers for deterministic path resolution assertions.

use std::path::{Path, PathBuf};

/// Resolve a project-local home directory from an explicit env-like value.
///
/// This test support adapter mirrors the internal `resolve_home_from_value`
/// behavior without requiring source-inline test modules.
#[must_use]
pub fn resolve_home_from_value(
    project_root: Option<&Path>,
    env_value: Option<&str>,
    default_relative: &str,
) -> Option<PathBuf> {
    crate::paths::resolve_home_from_value(project_root, env_value, default_relative)
}
