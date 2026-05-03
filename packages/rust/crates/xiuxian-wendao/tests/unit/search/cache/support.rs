use crate::search::SearchPlaneCache;
use crate::search::{SearchFileFingerprint, SearchManifestKeyspace};

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
