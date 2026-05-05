//! Hot-reload target registration records and path matching.

use anyhow::{Result, anyhow};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Callback that reloads a target and returns whether in-memory state changed.
pub type HotReloadCallback = Arc<dyn Fn() -> Result<bool> + Send + Sync>;

/// One hot-reload registration target.
pub struct HotReloadTarget {
    id: String,
    roots: Vec<PathBuf>,
    include_globs: Vec<String>,
    include_matcher: GlobSet,
    reload_if_changed: HotReloadCallback,
}

impl HotReloadTarget {
    /// Creates a target registration.
    ///
    /// # Errors
    ///
    /// Returns an error when target metadata is invalid or any glob is invalid.
    pub fn new(
        id: impl Into<String>,
        roots: Vec<PathBuf>,
        include_globs: Vec<String>,
        reload_if_changed: HotReloadCallback,
    ) -> Result<Self> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(anyhow!("hot reload target id cannot be empty"));
        }
        if roots.is_empty() {
            return Err(anyhow!("hot reload target '{id}' has no watch roots"));
        }
        let normalized_roots = normalize_roots(roots);
        let normalized_globs = if include_globs.is_empty() {
            vec!["**/*".to_string()]
        } else {
            include_globs
        };
        let include_matcher = compile_globs(&normalized_globs)?;
        Ok(Self {
            id,
            roots: normalized_roots,
            include_globs: normalized_globs,
            include_matcher,
            reload_if_changed,
        })
    }

    /// Returns the stable target identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the configured watcher roots.
    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// Returns include glob patterns used by this target.
    #[must_use]
    pub fn include_globs(&self) -> &[String] {
        &self.include_globs
    }

    /// Returns true when the path belongs to this target.
    #[must_use]
    pub fn matches_path(&self, path: &Path) -> bool {
        self.roots.iter().any(|root| {
            if !path.starts_with(root) {
                return false;
            }
            match path.strip_prefix(root) {
                Ok(relative) => {
                    self.include_matcher.is_match(relative) || self.include_matcher.is_match(path)
                }
                Err(_) => self.include_matcher.is_match(path),
            }
        })
    }

    /// Executes target reload logic.
    ///
    /// The callback returns `true` when state actually changed.
    ///
    /// # Errors
    ///
    /// Returns an error when the underlying reload callback fails.
    pub fn reload_if_changed(&self) -> Result<bool> {
        (self.reload_if_changed)()
    }
}

fn normalize_roots(mut roots: Vec<PathBuf>) -> Vec<PathBuf> {
    roots.sort();
    roots.dedup();
    roots
}

fn compile_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern)
            .map_err(|error| anyhow!("invalid hot reload glob '{pattern}': {error}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|error| anyhow!("failed to compile hot reload glob set: {error}"))
}
