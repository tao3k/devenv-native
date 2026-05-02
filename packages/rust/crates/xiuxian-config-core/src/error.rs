//! Error types emitted by config path resolution, loading, and merging.

use thiserror::Error;

/// Errors produced while resolving cascading TOML configuration.
#[derive(Debug, Error)]
pub enum ConfigCoreError {
    /// Embedded baseline TOML failed to parse.
    #[error("failed to parse embedded config for namespace `{namespace}`: {source}")]
    ParseEmbedded {
        /// Namespace for the failing config.
        namespace: String,
        /// TOML parser error.
        source: toml::de::Error,
    },
    /// One config file failed to read.
    #[error("failed to read config file {path}: {source}")]
    ReadFile {
        /// Absolute path of the failing file.
        path: String,
        /// I/O error source.
        source: std::io::Error,
    },
    /// One config file failed to parse.
    #[error("failed to parse TOML {path}: {source}")]
    ParseFile {
        /// Absolute path of the failing file.
        path: String,
        /// TOML parser error.
        source: toml::de::Error,
    },
    /// Both `xiuxian.toml` and orphan config files are present.
    #[error(
        "redundant config detected for namespace `{namespace}`: found orphan config files [{orphans}] while xiuxian.toml is active"
    )]
    RedundantOrphan {
        /// Namespace in conflict.
        namespace: String,
        /// Human-readable orphan path list.
        orphans: String,
    },
    /// Merged TOML failed to deserialize into typed config.
    #[error("failed to deserialize merged config for namespace `{namespace}`: {source}")]
    DeserializeMerged {
        /// Namespace in scope.
        namespace: String,
        /// Deserialization error.
        source: toml::de::Error,
    },
    /// TOML imports reference a cycle.
    #[error("cyclic config import detected: {chain}")]
    ImportCycle {
        /// Human-readable import chain.
        chain: String,
    },
    /// An `imports` key had an invalid shape.
    #[error("invalid imports declaration in {path}: {message}")]
    InvalidImports {
        /// File or embedded source path where the declaration was found.
        path: String,
        /// Validation message.
        message: String,
    },
    /// One import path referenced an environment variable that was not available.
    #[error("failed to expand import path in {path}: missing environment variable `{variable}`")]
    UnresolvedEnvironmentVariable {
        /// File or embedded source path where expansion failed.
        path: String,
        /// Missing environment variable name.
        variable: String,
    },
}
