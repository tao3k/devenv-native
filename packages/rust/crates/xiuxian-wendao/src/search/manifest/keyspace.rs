//! `search::manifest::keyspace` owns Wendao search manifest keyspace behavior.

use crate::search::SearchCorpusKind;

/// Valkey key namespace for search-plane manifests, leases, and caches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchManifestKeyspace {
    namespace: String,
}

impl SearchManifestKeyspace {
    /// Build a keyspace rooted at a caller-supplied namespace prefix.
    #[must_use]
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }

    /// Return the raw namespace prefix.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Key that stores the published manifest for one corpus.
    #[must_use]
    pub fn corpus_manifest_key(&self, corpus: SearchCorpusKind) -> String {
        format!("{}:manifest:{corpus}", self.namespace)
    }

    /// Key that stores file-level fingerprints for one corpus.
    #[must_use]
    pub fn corpus_file_fingerprints_key(&self, corpus: SearchCorpusKind) -> String {
        format!("{}:file-fingerprints:{corpus}", self.namespace)
    }

    /// Key that stores repo-scoped file-level fingerprints for one repo-backed corpus.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn repo_corpus_file_fingerprints_key(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> String {
        format!(
            "{}:repo-file-fingerprints:{corpus}:repo:{}",
            self.namespace,
            blake3::hash(repo_id.as_bytes()).to_hex()
        )
    }

    /// Key that stores the combined repo-backed corpus record for one repo/corpus pair.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn repo_corpus_record_key(&self, corpus: SearchCorpusKind, repo_id: &str) -> String {
        format!(
            "{}:repo-corpus:{corpus}:repo:{}",
            self.namespace,
            blake3::hash(repo_id.as_bytes()).to_hex()
        )
    }

    /// Key that stores one repo-backed publication record for a specific source revision.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn repo_publication_revision_key(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
    ) -> String {
        let normalized_revision = revision.trim().to_ascii_lowercase();
        format!(
            "{}:repo-publication:{corpus}:repo:{}:revision:{}",
            self.namespace,
            blake3::hash(repo_id.as_bytes()).to_hex(),
            blake3::hash(normalized_revision.as_bytes()).to_hex(),
        )
    }

    /// Key that stores the retained revision list for one repo-backed corpus.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn repo_publication_revision_index_key(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> String {
        format!(
            "{}:repo-publication-index:{corpus}:repo:{}",
            self.namespace,
            blake3::hash(repo_id.as_bytes()).to_hex(),
        )
    }

    /// Key that stores the latest full combined repo-backed corpus snapshot.
    #[must_use]
    pub fn repo_corpus_snapshot_key(&self) -> String {
        format!("{}:repo-corpus:snapshot", self.namespace)
    }

    /// Lease key used to enforce single-flight indexing for one corpus.
    #[must_use]
    pub fn corpus_lease_key(&self, corpus: SearchCorpusKind) -> String {
        format!("{}:lease:{corpus}", self.namespace)
    }

    /// Key for short-lived query result caching.
    #[must_use]
    pub fn corpus_query_cache_key(&self, corpus: SearchCorpusKind, cache_key: &str) -> String {
        self.search_query_cache_key(corpus.as_str(), cache_key)
    }

    /// Key for short-lived search response caching scoped by logical query surface.
    #[must_use]
    pub fn search_query_cache_key(&self, scope: &str, cache_key: &str) -> String {
        format!("{}:query:{scope}:{cache_key}", self.namespace)
    }

    /// Key for autocomplete prefix caching.
    #[must_use]
    pub fn autocomplete_cache_key(&self, prefix: &str) -> String {
        format!("{}:autocomplete:{prefix}", self.namespace)
    }
}

impl Default for SearchManifestKeyspace {
    fn default() -> Self {
        Self::new("xiuxian:wendao:search_plane")
    }
}
