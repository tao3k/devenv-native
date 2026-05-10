//! Catalog and path configuration types for `DuckLake` attachments.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Metadata catalog backing for one `DuckLake` attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuckLakeCatalog {
    /// `DuckLake` metadata stored in a local `DuckDB` catalog file.
    LocalMetadataFile(PathBuf),
    /// `DuckLake` metadata stored in `PostgreSQL` through `DuckDB`'s
    /// `postgres` extension.
    PostgresConnectionString(String),
}

impl DuckLakeCatalog {
    /// Build a local `DuckDB`-backed `DuckLake` catalog reference.
    #[must_use]
    pub fn local_metadata_file(path: impl Into<PathBuf>) -> Self {
        Self::LocalMetadataFile(path.into())
    }

    /// Build a `PostgreSQL`-backed `DuckLake` catalog reference.
    #[must_use]
    pub fn postgres_connection_string(connection_string: impl Into<String>) -> Self {
        Self::PostgresConnectionString(connection_string.into())
    }

    pub(super) fn attach_uri(&self) -> Result<String, String> {
        match self {
            Self::LocalMetadataFile(path) => {
                validate_path_is_not_empty(path, "DuckLake local metadata path")?;
                Ok(format!("ducklake:{}", path.to_string_lossy()))
            }
            Self::PostgresConnectionString(connection_string) => {
                if connection_string.trim().is_empty() {
                    return Err(
                        "DuckLake PostgreSQL catalog connection string cannot be blank".to_string(),
                    );
                }
                Ok(format!("ducklake:postgres:{connection_string}"))
            }
        }
    }

    pub(super) fn needs_postgres_extension(&self) -> bool {
        matches!(self, Self::PostgresConnectionString(_))
    }
}

/// Location where `DuckLake` stores data files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuckLakeDataPath {
    /// Local filesystem path prepared by the embedded runtime before attach.
    LocalPath(PathBuf),
    /// Remote object-store or HTTP-style URI rendered directly into
    /// `DuckLake`'s `DATA_PATH`.
    RemoteUri(String),
}

impl DuckLakeDataPath {
    /// Build a local `DuckLake` data path.
    #[must_use]
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::LocalPath(path.into())
    }

    /// Build a remote `DuckLake` data path URI such as `s3://bucket/prefix/`.
    #[must_use]
    pub fn remote_uri(uri: impl Into<String>) -> Self {
        Self::RemoteUri(uri.into())
    }

    /// Build an `S3`-compatible `DuckLake` data path URI.
    #[must_use]
    pub fn s3(uri: impl Into<String>) -> Self {
        Self::RemoteUri(uri.into())
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        match self {
            Self::LocalPath(path) => validate_path_is_not_empty(path, "DuckLake data path"),
            Self::RemoteUri(uri) => validate_remote_uri(uri),
        }
    }

    pub(super) fn sql_value(&self) -> Result<String, String> {
        self.validate()?;
        Ok(match self {
            Self::LocalPath(path) => path.to_string_lossy().to_string(),
            Self::RemoteUri(uri) => uri.trim().to_string(),
        })
    }

    #[cfg(feature = "duckdb")]
    pub(super) fn local_path(&self) -> Option<&std::path::Path> {
        match self {
            Self::LocalPath(path) => Some(path.as_path()),
            Self::RemoteUri(_) => None,
        }
    }
}

/// Generic `DuckLake` attachment configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuckLakeAttachConfig {
    /// Catalog alias used inside the embedded `DuckDB` connection.
    pub alias: String,
    /// Metadata catalog backing.
    pub catalog: DuckLakeCatalog,
    /// Location where `DuckLake` stores data files.
    pub data_path: DuckLakeDataPath,
    /// Whether attach SQL should install and load required extensions first.
    pub bootstrap_extensions: bool,
}

impl DuckLakeAttachConfig {
    /// Build a local `DuckDB`-backed `DuckLake` attachment.
    #[must_use]
    pub fn local(
        alias: impl Into<String>,
        metadata_path: impl Into<PathBuf>,
        data_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            alias: alias.into(),
            catalog: DuckLakeCatalog::local_metadata_file(metadata_path),
            data_path: DuckLakeDataPath::local(data_path),
            bootstrap_extensions: true,
        }
    }

    /// Build a `PostgreSQL`-backed `DuckLake` attachment.
    #[must_use]
    pub fn postgres(
        alias: impl Into<String>,
        connection_string: impl Into<String>,
        data_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            alias: alias.into(),
            catalog: DuckLakeCatalog::postgres_connection_string(connection_string),
            data_path: DuckLakeDataPath::local(data_path),
            bootstrap_extensions: true,
        }
    }

    /// Build a `PostgreSQL`-backed `DuckLake` attachment with a remote data path.
    #[must_use]
    pub fn postgres_remote_data_path(
        alias: impl Into<String>,
        connection_string: impl Into<String>,
        data_path_uri: impl Into<String>,
    ) -> Self {
        Self {
            alias: alias.into(),
            catalog: DuckLakeCatalog::postgres_connection_string(connection_string),
            data_path: DuckLakeDataPath::remote_uri(data_path_uri),
            bootstrap_extensions: true,
        }
    }

    /// Replace the data path while preserving the catalog and alias.
    #[must_use]
    pub fn with_data_path(mut self, data_path: DuckLakeDataPath) -> Self {
        self.data_path = data_path;
        self
    }

    pub(super) fn data_path_sql_value(&self) -> Result<String, String> {
        self.data_path.sql_value()
    }
}

fn validate_path_is_not_empty(path: &std::path::Path, label: &str) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err(format!("{label} cannot be blank"));
    }
    Ok(())
}

fn validate_remote_uri(uri: &str) -> Result<(), String> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return Err("DuckLake remote data path URI cannot be blank".to_string());
    }
    let supported = ["s3://", "r2://", "gcs://", "gs://", "http://", "https://"];
    if supported.iter().any(|prefix| trimmed.starts_with(prefix)) {
        return Ok(());
    }
    Err(format!(
        "DuckLake remote data path URI `{trimmed}` must start with one of: {}",
        supported.join(", ")
    ))
}
