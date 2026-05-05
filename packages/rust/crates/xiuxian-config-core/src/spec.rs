//! Declarative cascade specification types for layered TOML config loading.

/// Array merge behavior when resolving layered TOML values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ArrayMergeStrategy {
    /// Replace destination arrays with the latest source array.
    #[default]
    Overwrite,
    /// Append source array items to destination arrays.
    Append,
}

/// Immutable runtime spec used by the cascading resolver.
#[derive(Debug, Clone, Copy)]
pub struct ConfigCascadeSpec<'a> {
    /// Namespace key inside `xiuxian.toml` (for example `skills`).
    pub namespace: &'a str,
    /// Embedded baseline TOML payload bundled in the crate binary.
    pub embedded_toml: &'a str,
    /// Optional absolute source path for the embedded TOML payload.
    ///
    /// When present, relative `imports` inside `embedded_toml` are resolved
    /// against this path's parent directory.
    pub embedded_source_path: Option<&'a str>,
    /// Optional standalone/orphan config filename (for example `orphan.toml`).
    pub orphan_file: &'a str,
    /// Strategy for merging TOML arrays.
    pub array_merge_strategy: ArrayMergeStrategy,
}

impl<'a> ConfigCascadeSpec<'a> {
    /// Build a new cascade spec.
    #[must_use]
    pub const fn new(namespace: &'a str, embedded_toml: &'a str, orphan_file: &'a str) -> Self {
        Self {
            namespace,
            embedded_toml,
            embedded_source_path: None,
            orphan_file,
            array_merge_strategy: ArrayMergeStrategy::Overwrite,
        }
    }

    /// Attach the physical source path for the embedded TOML payload.
    #[must_use]
    pub const fn with_embedded_source_path(self, embedded_source_path: &'a str) -> Self {
        Self {
            embedded_source_path: Some(embedded_source_path),
            ..self
        }
    }

    /// Override the default array merge strategy.
    #[must_use]
    pub const fn with_array_merge_strategy(self, strategy: ArrayMergeStrategy) -> Self {
        Self {
            array_merge_strategy: strategy,
            ..self
        }
    }
}
