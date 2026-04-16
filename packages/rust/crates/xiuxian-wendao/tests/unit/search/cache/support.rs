use std::collections::BTreeMap;

use crate::search::cache::SearchPlaneCache;
use crate::search::{
    SearchFileFingerprint, SearchManifestKeyspace, SearchRepoCorpusSnapshotRecord,
};

#[derive(Debug, Default)]
pub(crate) struct TestCacheShadow {
    pub(crate) generic_json_payloads: BTreeMap<String, String>,
    pub(crate) corpus_manifests: BTreeMap<SearchCorpusKind, SearchManifestRecord>,
    pub(crate) repo_corpus_records: BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord>,
    pub(crate) repo_corpus_snapshot: Option<SearchRepoCorpusSnapshotRecord>,
    pub(crate) repo_publications_by_revision:
        BTreeMap<(SearchCorpusKind, String, String), SearchRepoPublicationRecord>,
    pub(crate) repo_publication_revision_indexes: BTreeMap<(SearchCorpusKind, String), Vec<String>>,
    pub(crate) corpus_file_fingerprints:
        BTreeMap<SearchCorpusKind, BTreeMap<String, SearchFileFingerprint>>,
    pub(crate) repo_corpus_file_fingerprints:
        BTreeMap<(SearchCorpusKind, String), BTreeMap<String, SearchFileFingerprint>>,
}

impl SearchPlaneCache {
    pub(crate) fn clear_repo_shadow_for_tests(&self, repo_id: &str) {
        let mut shadow = self
            .shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        shadow
            .repo_corpus_records
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
        if let Some(snapshot) = shadow.repo_corpus_snapshot.as_mut() {
            snapshot.records.retain(|record| record.repo_id != repo_id);
            if snapshot.records.is_empty() {
                shadow.repo_corpus_snapshot = None;
            }
        }
        shadow
            .repo_corpus_file_fingerprints
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
        shadow
            .repo_publications_by_revision
            .retain(|(_, candidate_repo_id, _), _| candidate_repo_id != repo_id);
        shadow
            .repo_publication_revision_indexes
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
    }
}

pub(super) fn required_cache_key(key: Option<String>, context: &str) -> String {
    key.unwrap_or_else(|| panic!("{context}"))
}

pub(super) fn cache_for_tests() -> SearchPlaneCache {
    SearchPlaneCache::for_tests(SearchManifestKeyspace::new("xiuxian:test:search_plane"))
}

pub(super) fn sample_file_fingerprint(
    relative_path: &str,
    partition_id: &str,
    size_bytes: u64,
    modified_unix_ms: u64,
) -> SearchFileFingerprint {
    SearchFileFingerprint {
        relative_path: relative_path.to_string(),
        partition_id: Some(partition_id.to_string()),
        size_bytes,
        modified_unix_ms,
        extractor_version: 1,
        schema_version: 1,
        blake3: None,
    }
}

pub(super) use crate::search::cache::SearchPlaneFileFingerprintScope;
pub(super) use crate::search::{
    SearchCorpusKind, SearchManifestRecord, SearchPublicationStorageFormat, SearchRepoCorpusRecord,
    SearchRepoPublicationInput, SearchRepoPublicationRecord,
};
