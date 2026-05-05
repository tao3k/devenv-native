use crate::search::SearchManifestKeyspace;
#[cfg(any(test, feature = "test-support"))]
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchManifestRecord, SearchRepoCorpusRecord,
    SearchRepoCorpusSnapshotRecord, SearchRepoPublicationRecord,
};

use super::config::SearchPlaneCacheConfig;
#[cfg(any(test, feature = "test-support"))]
use std::collections::BTreeMap;
#[cfg(any(test, feature = "test-support"))]
use std::sync::{Arc, RwLock};

#[cfg(any(test, feature = "test-support"))]
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

#[derive(Debug, Clone)]
pub(crate) struct SearchPlaneCache {
    pub(crate) client: Option<redis::Client>,
    pub(crate) config: SearchPlaneCacheConfig,
    pub(crate) keyspace: SearchManifestKeyspace,
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) shadow: Arc<RwLock<TestCacheShadow>>,
}
