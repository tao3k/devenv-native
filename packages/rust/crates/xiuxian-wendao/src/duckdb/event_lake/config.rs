//! Local path configuration for Wendao's embedded event lake.

use std::path::{Path, PathBuf};

use xiuxian_db_store::duckdb::{DuckLakeAttachConfig, ensure_duckdb_identifier};
use xiuxian_io::PrjDirs;

use super::WENDAO_EVENT_LAKE_DEFAULT_ALIAS;
use super::handle::WendaoEventLake;

const WENDAO_EVENT_LAKE_DATA_HOME_DIR: &str = "wendao";
const WENDAO_EVENT_LAKE_ROOT_DIR: &str = "event_lake";
const WENDAO_EVENT_LAKE_METADATA_DIR: &str = "metadata";
const WENDAO_EVENT_LAKE_DATA_DIR: &str = "data";
const WENDAO_EVENT_LAKE_METADATA_FILE: &str = "wendao.ducklake";

/// Wendao-owned local DuckLake path contract for the event lake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoEventLakeLocalConfig {
    catalog_alias: String,
    event_lake_root: PathBuf,
    metadata_path: PathBuf,
    data_path: PathBuf,
}

impl WendaoEventLakeLocalConfig {
    /// Build local event-lake config from the active `PRJ_DATA_HOME`.
    ///
    /// # Errors
    ///
    /// Returns an error when the default catalog alias is invalid.
    pub fn from_prj_data_home() -> Result<Self, String> {
        Self::from_data_home(PrjDirs::data_home())
    }

    /// Build local event-lake config from a project data-home path.
    ///
    /// # Errors
    ///
    /// Returns an error when the default catalog alias is invalid.
    pub fn from_data_home(data_home: impl AsRef<Path>) -> Result<Self, String> {
        Self::from_data_home_with_alias(WENDAO_EVENT_LAKE_DEFAULT_ALIAS, data_home)
    }

    /// Build local event-lake config from a project data-home path and alias.
    ///
    /// # Errors
    ///
    /// Returns an error when the alias is not a valid DuckDB identifier.
    pub fn from_data_home_with_alias(
        catalog_alias: impl Into<String>,
        data_home: impl AsRef<Path>,
    ) -> Result<Self, String> {
        let event_lake_root = data_home
            .as_ref()
            .join(WENDAO_EVENT_LAKE_DATA_HOME_DIR)
            .join(WENDAO_EVENT_LAKE_ROOT_DIR);
        Self::from_event_lake_root(catalog_alias, event_lake_root)
    }

    /// Build local event-lake config from a fully resolved event-lake root.
    ///
    /// # Errors
    ///
    /// Returns an error when the alias is not a valid DuckDB identifier.
    pub fn from_event_lake_root(
        catalog_alias: impl Into<String>,
        event_lake_root: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        let catalog_alias = catalog_alias.into();
        ensure_duckdb_identifier(&catalog_alias, "DuckLake catalog")?;
        let event_lake_root = event_lake_root.into();
        let metadata_path = event_lake_root
            .join(WENDAO_EVENT_LAKE_METADATA_DIR)
            .join(WENDAO_EVENT_LAKE_METADATA_FILE);
        let data_path = event_lake_root.join(WENDAO_EVENT_LAKE_DATA_DIR);
        Ok(Self {
            catalog_alias,
            event_lake_root,
            metadata_path,
            data_path,
        })
    }

    /// Access the attached DuckLake catalog alias.
    #[must_use]
    pub fn catalog_alias(&self) -> &str {
        self.catalog_alias.as_str()
    }

    /// Access the resolved event-lake root directory.
    #[must_use]
    pub fn event_lake_root(&self) -> &Path {
        self.event_lake_root.as_path()
    }

    /// Access the local DuckLake metadata catalog path.
    #[must_use]
    pub fn metadata_path(&self) -> &Path {
        self.metadata_path.as_path()
    }

    /// Access the local DuckLake data directory path.
    #[must_use]
    pub fn data_path(&self) -> &Path {
        self.data_path.as_path()
    }

    /// Convert this local policy into the generic db-store attach config.
    #[must_use]
    pub fn ducklake_attach_config(&self) -> DuckLakeAttachConfig {
        DuckLakeAttachConfig::local(
            self.catalog_alias.as_str(),
            self.metadata_path.clone(),
            self.data_path.clone(),
        )
    }

    /// Attach this local event lake and ensure its table exists.
    ///
    /// # Errors
    ///
    /// Returns an error when DuckLake attach or event table setup fails.
    pub fn attach(&self, connection: &duckdb::Connection) -> Result<WendaoEventLake, String> {
        WendaoEventLake::attach(connection, &self.ducklake_attach_config())
    }
}
