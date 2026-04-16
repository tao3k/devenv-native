use crate::analyzers::RegisteredRepository;
use crate::analyzers::cache::identity::collect_repository_analysis_identity;
use crate::search::FuzzySearchOptions;
use xiuxian_git_repo::{LocalCheckoutMetadata, MaterializedRepo};

/// Cache key for repository analysis results.
#[derive(Debug, Clone)]
pub struct RepositoryAnalysisCacheKey {
    /// Repository identifier.
    pub repo_id: String,
    /// Root path of the checkout.
    pub checkout_root: String,
    /// Stable identity for analysis-affecting repository inputs.
    pub analysis_identity: String,
    /// Revision of the checkout.
    pub checkout_revision: Option<String>,
    /// Revision of the mirror.
    pub mirror_revision: Option<String>,
    /// Revision being tracked.
    pub tracking_revision: Option<String>,
    /// Sorted list of plugin identifiers used.
    pub plugin_ids: Vec<String>,
}

impl RepositoryAnalysisCacheKey {
    fn cache_identity(&self) -> (&str, &str, &str, &Vec<String>) {
        (
            self.repo_id.as_str(),
            self.checkout_root.as_str(),
            self.analysis_identity.as_str(),
            &self.plugin_ids,
        )
    }

    pub(crate) fn revision(&self) -> Option<&str> {
        self.checkout_revision
            .as_deref()
            .or(self.mirror_revision.as_deref())
            .or(self.tracking_revision.as_deref())
    }

    #[cfg(feature = "zhenfa-router")]
    pub(crate) fn matches_revision_lookup(
        &self,
        repo_id: &str,
        checkout_root: &str,
        plugin_ids: &[String],
        revision: &str,
    ) -> bool {
        self.repo_id == repo_id
            && self.checkout_root == checkout_root
            && self.plugin_ids == plugin_ids
            && self.revision() == Some(revision)
    }
}

impl PartialEq for RepositoryAnalysisCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.cache_identity() == other.cache_identity()
    }
}

impl Eq for RepositoryAnalysisCacheKey {}

impl PartialOrd for RepositoryAnalysisCacheKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RepositoryAnalysisCacheKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cache_identity().cmp(&other.cache_identity())
    }
}

/// Cache key for final repo-search endpoint payloads.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepositorySearchQueryCacheKey {
    /// The underlying analysis cache identity.
    pub analysis_key: RepositoryAnalysisCacheKey,
    /// Stable endpoint identifier.
    pub endpoint: String,
    /// Raw query text.
    pub query: String,
    /// Optional endpoint-specific filter such as projected-page kind.
    pub filter: Option<String>,
    /// Maximum edit distance for the search profile.
    pub max_distance: u8,
    /// Required shared prefix length for the search profile.
    pub prefix_length: usize,
    /// Whether transpositions are allowed for the search profile.
    pub transposition: bool,
    /// Result limit.
    pub limit: usize,
}

impl RepositorySearchQueryCacheKey {
    /// Build one endpoint cache key from the shared analysis identity plus query settings.
    #[must_use]
    pub fn new(
        analysis_key: &RepositoryAnalysisCacheKey,
        endpoint: &str,
        query: &str,
        filter: Option<String>,
        options: FuzzySearchOptions,
        limit: usize,
    ) -> Self {
        Self {
            analysis_key: analysis_key.clone(),
            endpoint: endpoint.to_string(),
            query: query.to_string(),
            filter,
            max_distance: options.max_distance,
            prefix_length: options.prefix_length,
            transposition: options.transposition,
            limit,
        }
    }
}

/// Builds a cache key from repository configuration and resolved source.
#[must_use]
pub(crate) fn build_repository_analysis_cache_key(
    repository: &RegisteredRepository,
    source: &MaterializedRepo,
    metadata: Option<&LocalCheckoutMetadata>,
) -> RepositoryAnalysisCacheKey {
    let plugin_ids = repository.configured_plugin_ids();
    let analysis_identity = collect_repository_analysis_identity(
        repository,
        source.checkout_root.as_path(),
        plugin_ids.as_slice(),
    )
    .unwrap_or_else(|| fallback_analysis_identity(source, metadata, plugin_ids.as_slice()));

    RepositoryAnalysisCacheKey {
        repo_id: repository.id.clone(),
        checkout_root: source.checkout_root.display().to_string(),
        analysis_identity,
        checkout_revision: metadata.and_then(|item| item.revision.clone()),
        mirror_revision: source.mirror_revision.clone(),
        tracking_revision: source.tracking_revision.clone(),
        plugin_ids,
    }
}

fn fallback_analysis_identity(
    source: &MaterializedRepo,
    metadata: Option<&LocalCheckoutMetadata>,
    plugin_ids: &[String],
) -> String {
    let payload = format!(
        "fallback|root:{}|checkout:{}|mirror:{}|tracking:{}|plugins:{}",
        source.checkout_root.display(),
        metadata
            .and_then(|item| item.revision.as_deref())
            .unwrap_or_default(),
        source.mirror_revision.as_deref().unwrap_or_default(),
        source.tracking_revision.as_deref().unwrap_or_default(),
        plugin_ids.join(","),
    );
    blake3::hash(payload.as_bytes()).to_hex().to_string()
}
