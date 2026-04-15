use serde::{Deserialize, Serialize};

use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::manifest::SearchRepoPublicationInput;
use crate::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchMaintenanceStatus, SearchPlanePhase,
};

/// Persisted storage format for a published search-plane dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchPublicationStorageFormat {
    /// Legacy Lance-backed publication.
    #[default]
    Lance,
    /// Parquet publication readable by the query-engine lane.
    Parquet,
}

/// Materialized manifest row persisted to Valkey for one corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchManifestRecord {
    /// Corpus this manifest row belongs to.
    pub corpus: SearchCorpusKind,
    /// Active published epoch for readers.
    pub active_epoch: Option<u64>,
    /// Schema version associated with the active epoch.
    pub schema_version: u32,
    #[serde(default)]
    /// Storage format associated with the active publication.
    pub storage_format: SearchPublicationStorageFormat,
    /// Current published or in-flight fingerprint.
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Published row count for the active epoch, when known.
    pub row_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Published fragment count for the active epoch, when known.
    pub fragment_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp for the latest completed build attempt.
    pub build_finished_at: Option<String>,
    /// RFC3339 time when the manifest was updated.
    pub updated_at: Option<String>,
}

impl SearchManifestRecord {
    /// Project a manifest row from an in-memory coordinator status.
    #[must_use]
    pub fn from_status(status: &SearchCorpusStatus) -> Self {
        Self {
            corpus: status.corpus,
            active_epoch: status.active_epoch,
            schema_version: status.schema_version,
            storage_format: SearchPublicationStorageFormat::Parquet,
            fingerprint: status.fingerprint.clone(),
            row_count: status.row_count,
            fragment_count: status.fragment_count,
            build_finished_at: status.build_finished_at.clone(),
            updated_at: status.updated_at.clone(),
        }
    }

    /// Recover a readable ready status row from the persisted local manifest.
    #[must_use]
    pub fn to_status(&self) -> Option<SearchCorpusStatus> {
        let active_epoch = self.active_epoch?;
        if self.schema_version != self.corpus.schema_version() {
            return None;
        }

        let mut status = SearchCorpusStatus::new(self.corpus);
        status.phase = SearchPlanePhase::Ready;
        status.active_epoch = Some(active_epoch);
        status.schema_version = self.schema_version;
        status.fingerprint.clone_from(&self.fingerprint);
        status.row_count = self.row_count;
        status.fragment_count = self.fragment_count;
        status.build_finished_at.clone_from(&self.build_finished_at);
        status.updated_at.clone_from(&self.updated_at);
        if status.updated_at.is_none() {
            status.updated_at.clone_from(&self.build_finished_at);
        }
        Some(status)
    }
}

/// Materialized publication row for one published repo-backed corpus table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRepoPublicationRecord {
    /// Repo-backed corpus this manifest row belongs to.
    pub corpus: SearchCorpusKind,
    /// Stable repository identifier.
    pub repo_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Stable epoch token for the currently readable repo-backed publication.
    pub active_epoch: Option<u64>,
    /// Explicit publication token for the currently readable repo-backed table.
    pub publication_id: String,
    /// Table name that currently serves reads for this repo-backed corpus.
    pub table_name: String,
    /// Current table version id published to readers.
    pub table_version_id: u64,
    /// Schema version associated with the published table.
    pub schema_version: u32,
    #[serde(default)]
    /// Storage format associated with the published table.
    pub storage_format: SearchPublicationStorageFormat,
    /// Source revision that produced the published table, when known.
    pub source_revision: Option<String>,
    /// Logical row count for the published table.
    pub row_count: u64,
    /// Fragment count for the published table.
    pub fragment_count: u64,
    /// RFC3339 timestamp of the published table commit.
    pub published_at: String,
}

impl SearchRepoPublicationRecord {
    /// Construct repo publication metadata from one published table snapshot.
    #[must_use]
    pub(crate) fn new(
        corpus: SearchCorpusKind,
        repo_id: impl Into<String>,
        input: SearchRepoPublicationInput,
    ) -> Self {
        Self::new_with_storage_format(
            corpus,
            repo_id,
            input,
            SearchPublicationStorageFormat::Lance,
        )
    }

    /// Construct repo publication metadata for an explicit storage format.
    #[must_use]
    pub(crate) fn new_with_storage_format(
        corpus: SearchCorpusKind,
        repo_id: impl Into<String>,
        input: SearchRepoPublicationInput,
        storage_format: SearchPublicationStorageFormat,
    ) -> Self {
        let repo_id = repo_id.into();
        let publication_id =
            build_repo_publication_id(corpus, repo_id.as_str(), &input, storage_format);
        Self {
            corpus,
            active_epoch: Some(build_repo_publication_epoch(publication_id.as_str())),
            publication_id,
            repo_id,
            table_name: input.table_name,
            table_version_id: input.table_version_id,
            schema_version: input.schema_version,
            storage_format,
            source_revision: input.source_revision,
            row_count: input.row_count,
            fragment_count: input.fragment_count,
            published_at: input.published_at,
        }
    }

    /// Stable cache/status token that changes only when the published table changes.
    #[must_use]
    pub fn cache_version(&self) -> String {
        format!(
            "{}:schema:{}:repo:{}:publication:{}",
            self.corpus,
            self.schema_version,
            self.repo_id.trim().to_ascii_lowercase(),
            self.publication_id
        )
    }

    /// Stable epoch token for the readable repo-backed publication.
    #[must_use]
    pub fn active_epoch_value(&self) -> u64 {
        self.active_epoch
            .unwrap_or_else(|| build_repo_publication_epoch(self.publication_id.as_str()))
    }

    /// Whether this publication is readable by the Parquet query-engine lane.
    #[must_use]
    pub fn is_parquet_query_readable(&self) -> bool {
        matches!(self.storage_format, SearchPublicationStorageFormat::Parquet)
    }
}

/// Materialized runtime row for one repository's indexing/search readiness state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRepoRuntimeRecord {
    /// Stable repository identifier.
    pub repo_id: String,
    /// Latest repo indexing phase observed by the producer-side coordinator.
    pub phase: RepoIndexPhase,
    /// Latest source revision observed by repo indexing, when known.
    pub last_revision: Option<String>,
    /// Latest repo indexing error surfaced by the producer, when known.
    pub last_error: Option<String>,
    /// RFC3339 timestamp associated with the latest runtime snapshot, when known.
    pub updated_at: Option<String>,
}

impl SearchRepoRuntimeRecord {
    /// Project a persisted runtime row from one repo-index status entry.
    #[must_use]
    pub fn from_status(status: &RepoIndexEntryStatus) -> Self {
        Self {
            repo_id: status.repo_id.clone(),
            phase: status.phase,
            last_revision: status.last_revision.clone(),
            last_error: status.last_error.clone(),
            updated_at: status.updated_at.clone(),
        }
    }
}

/// Combined repo-backed corpus record that folds runtime and publication into one row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRepoCorpusRecord {
    /// Repo-backed corpus this row belongs to.
    pub corpus: SearchCorpusKind,
    /// Stable repository identifier.
    pub repo_id: String,
    /// Latest repo runtime state known to the search plane, when available.
    pub runtime: Option<SearchRepoRuntimeRecord>,
    /// Latest readable publication for this repo-backed corpus, when available.
    pub publication: Option<SearchRepoPublicationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Latest repo-backed maintenance metadata known to the search plane.
    pub maintenance: Option<SearchMaintenanceStatus>,
}

impl SearchRepoCorpusRecord {
    /// Construct one combined repo-backed corpus record.
    #[must_use]
    pub fn new(
        corpus: SearchCorpusKind,
        repo_id: impl Into<String>,
        runtime: Option<SearchRepoRuntimeRecord>,
        publication: Option<SearchRepoPublicationRecord>,
    ) -> Self {
        Self {
            corpus,
            repo_id: repo_id.into(),
            runtime,
            publication,
            maintenance: None,
        }
    }

    /// Attach the latest repo-backed maintenance metadata to this combined row.
    #[must_use]
    pub fn with_maintenance(mut self, maintenance: Option<SearchMaintenanceStatus>) -> Self {
        self.maintenance = maintenance;
        self
    }
}

/// Full combined repo-backed corpus snapshot owned by the search plane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRepoCorpusSnapshotRecord {
    /// Combined repo-backed corpus rows across all tracked repos and corpora.
    pub records: Vec<SearchRepoCorpusRecord>,
}

fn build_repo_publication_id(
    corpus: SearchCorpusKind,
    repo_id: &str,
    input: &SearchRepoPublicationInput,
    storage_format: SearchPublicationStorageFormat,
) -> String {
    let payload = format!(
        "{corpus}|{}|{}|{schema_version}|{}|{table_version_id}|{row_count}|{fragment_count}|{}|{storage_format:?}",
        repo_id.trim().to_ascii_lowercase(),
        input.table_name.trim().to_ascii_lowercase(),
        input
            .source_revision
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .to_ascii_lowercase(),
        input.published_at.trim().to_ascii_lowercase(),
        schema_version = input.schema_version,
        table_version_id = input.table_version_id,
        row_count = input.row_count,
        fragment_count = input.fragment_count,
    );
    blake3::hash(payload.as_bytes()).to_hex().to_string()
}

pub(crate) fn build_repo_publication_epoch(publication_id: &str) -> u64 {
    let hash = blake3::hash(publication_id.trim().as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_be_bytes(bytes)
}
