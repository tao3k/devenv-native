use std::path::{Path, PathBuf};

use super::core::SkillRuntimeResolver;
use crate::skill_runtime::SkillRuntimeError;
use xiuxian_config_core::{resolve_path_from_value, resolve_project_root_or_cwd_from_value};

impl SkillRuntimeResolver {
    /// Discover runtime semantic skill roots from the current process environment.
    #[must_use]
    pub fn discover_runtime_roots() -> Vec<PathBuf> {
        let project_root = resolve_project_root();
        let crates_root = project_root.join("packages").join("rust").join("crates");
        let mut roots = discover_crate_skill_roots(crates_root.as_path());
        roots.push(project_root.join("assets").join("skills"));

        let config_home = env_path("PRJ_CONFIG_HOME", project_root.as_path())
            .unwrap_or_else(|| project_root.join(".config"));
        roots.push(config_home.join("xiuxian-artisan-workshop").join("skills"));

        if let Some(resource_root) = env_path("XIUXIAN_RESOURCE_ROOT", project_root.as_path()) {
            roots.push(resource_root.join("skills"));
        }

        if let Ok(executable_path) = std::env::current_exe()
            && let Some(executable_dir) = executable_path.parent()
        {
            roots.push(executable_dir.join("resources").join("skills"));
            roots.push(executable_dir.join("..").join("resources").join("skills"));
        }

        roots.retain(|path| path.exists() && path.is_dir());
        dedup_paths(&mut roots);
        roots
    }

    /// Resolve the runtime skill root from an explicit project root and optional env override.
    #[must_use]
    pub fn resolve_runtime_skill_root_with(
        project_root: &Path,
        env_value: Option<&str>,
    ) -> PathBuf {
        resolve_path_from_value(Some(project_root), env_value)
            .unwrap_or_else(|| project_root.join("skills"))
    }

    /// Resolve the runtime skill root from the current process environment.
    #[must_use]
    pub fn resolve_runtime_skill_root() -> PathBuf {
        let project_root = resolve_project_root();
        Self::resolve_runtime_skill_root_with(
            project_root.as_path(),
            std::env::var("PRJ_SKILLS_DIR").ok().as_deref(),
        )
    }

    /// Discover runtime skill roots from the current process environment.
    #[must_use]
    pub fn discover_runtime_skill_roots() -> Vec<PathBuf> {
        let mut roots = vec![Self::resolve_runtime_skill_root()];
        roots.retain(|path| path.exists() && path.is_dir());
        dedup_paths(&mut roots);
        roots
    }

    /// Build a resolver from runtime-discovered roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace indexing fails.
    pub fn from_runtime_roots() -> Result<Self, SkillRuntimeError> {
        let roots = Self::discover_runtime_roots();
        let runtime_roots = Self::discover_runtime_skill_roots();
        Self::from_roots_with_runtime(roots.as_slice(), runtime_roots.as_slice())
    }

    /// Build a resolver from runtime-discovered roots with embedded mounts enabled.
    ///
    /// # Errors
    ///
    /// Returns [`SkillRuntimeError`] when namespace indexing fails.
    pub fn from_runtime_roots_with_embedded() -> Result<Self, SkillRuntimeError> {
        let roots = Self::discover_runtime_roots();
        let runtime_roots = Self::discover_runtime_skill_roots();
        Self::from_roots_with_embedded_and_runtime(roots.as_slice(), runtime_roots.as_slice())
    }
}

fn resolve_project_root() -> PathBuf {
    let raw_project_root = std::env::var("PRJ_ROOT").ok();
    let current_dir = std::env::current_dir().ok();
    resolve_project_root_or_cwd_from_value(raw_project_root.as_deref(), current_dir.as_deref())
}

fn env_path(key: &str, project_root: &Path) -> Option<PathBuf> {
    let raw_value = std::env::var(key).ok();
    resolve_path_from_value(Some(project_root), raw_value.as_deref())
}

fn discover_crate_skill_roots(crates_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(crates_root) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|crate_dir| crate_dir.join("resources").join("skills"))
        .collect()
}

fn dedup_paths(paths: &mut Vec<PathBuf>) {
    let mut unique = Vec::new();
    for path in std::mem::take(paths) {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }
    *paths = unique;
}

#[cfg(test)]
#[path = "../../../tests/unit/skill_runtime/resolver/runtime.rs"]
mod tests;
