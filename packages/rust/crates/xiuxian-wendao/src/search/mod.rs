//! Shared search infrastructure and primitives for Wendao.

#[cfg(feature = "search-runtime")]
mod attachment;
#[cfg(feature = "search-runtime")]
mod cache;
#[cfg(feature = "search-runtime")]
mod coordinator;
#[cfg(feature = "search-runtime")]
mod corpus;
/// Shared lexical fuzzy-search utilities.
pub mod fuzzy;
#[cfg(feature = "search-runtime")]
mod knowledge_section;
#[cfg(feature = "search-runtime")]
mod local_publication_parquet;
#[cfg(feature = "search-runtime")]
mod local_symbol;
#[cfg(feature = "search-runtime")]
mod manifest;
#[cfg(feature = "search-runtime")]
mod markdown_snapshot;
/// Synthetic benchmark and probe helpers for search-plane performance seams.
#[cfg(all(feature = "search-runtime", any(test, feature = "performance")))]
pub mod perf_support;
#[cfg(feature = "search-runtime")]
mod project_fingerprint;
/// Shared query-language adapters that sit above the Wendao search runtime.
pub mod queries;
#[cfg(feature = "search-runtime")]
mod ranking;
#[cfg(feature = "search-runtime")]
mod reference_occurrence;
#[cfg(feature = "search-runtime")]
mod repo_content_chunk;
#[cfg(feature = "search-runtime")]
mod repo_entity;
#[cfg(feature = "search-runtime")]
mod repo_publication_parquet;
/// Shared repo-search execution seams above the search runtime.
#[cfg(feature = "search-runtime")]
pub(crate) mod repo_search;
#[cfg(feature = "search-runtime")]
mod repo_staging;
#[cfg(feature = "search-runtime")]
mod semantic_fingerprint;
#[cfg(feature = "search-runtime")]
mod service;
#[cfg(feature = "search-runtime")]
mod source_snapshot;
#[cfg(feature = "search-runtime")]
mod status;
/// Shared Tantivy-backed search primitives.
#[cfg(feature = "repo-lexical-index")]
pub mod tantivy;

#[cfg(feature = "search-runtime")]
pub(crate) use attachment::AttachmentSearchError;
#[cfg(all(test, feature = "search-runtime"))]
pub(crate) use cache::SearchPlaneCache;
#[cfg(feature = "search-runtime")]
pub(crate) use cache::SearchPlaneCacheTtl;
#[cfg(feature = "search-runtime")]
pub(crate) use cache::SearchPlaneFileFingerprintScope;
#[cfg(feature = "search-runtime")]
pub(crate) use cache::resolve_search_plane_cache_connection_target;
#[cfg(feature = "search-runtime")]
pub use coordinator::{BeginBuildDecision, SearchBuildLease, SearchPlaneCoordinator};
#[cfg(feature = "search-runtime")]
pub use corpus::SearchCorpusKind;
pub use fuzzy::{
    FuzzyMatch, FuzzyMatcher, FuzzyScore, FuzzySearchOptions, LexicalMatcher, edit_distance,
    levenshtein_distance, normalized_score, passes_prefix_requirement, shared_prefix_len,
};
#[cfg(feature = "search-runtime")]
pub(crate) use knowledge_section::KnowledgeSectionSearchError;
#[cfg(feature = "search-runtime")]
pub(crate) use local_symbol::{LocalSymbolSearchError, restore_local_symbol_hits};
#[cfg(feature = "search-runtime")]
pub(crate) use manifest::SearchRepoPublicationInput;
#[cfg(feature = "search-runtime")]
pub use manifest::{
    SearchFileFingerprint, SearchManifestKeyspace, SearchManifestRecord,
    SearchPublicationStorageFormat, SearchRepoCorpusRecord, SearchRepoCorpusSnapshotRecord,
    SearchRepoPublicationRecord, SearchRepoRuntimeRecord,
};
#[cfg(feature = "search-runtime")]
pub(crate) use markdown_snapshot::{
    MarkdownProjectSnapshot, MarkdownSnapshotEntry, build_markdown_snapshot_entry,
    markdown_snapshot_entry_cache_key,
};
#[cfg(feature = "search-runtime")]
pub(crate) use project_fingerprint::{
    ProjectScanInventory, ProjectScannedFile, fingerprint_note_projects_from_scanned_files,
    fingerprint_source_projects_from_scanned_files, fingerprint_symbol_projects_from_scanned_files,
    scan_supported_project_files,
};
#[cfg(all(test, feature = "search-runtime"))]
pub(crate) use project_fingerprint::{
    fingerprint_note_projects, fingerprint_note_projects_with_scanned_files,
    fingerprint_source_projects, fingerprint_source_projects_with_scanned_files,
    fingerprint_symbol_projects, fingerprint_symbol_projects_with_scanned_files,
    scan_note_project_files, scan_source_project_files, scan_symbol_project_files,
};
#[cfg(feature = "search-runtime")]
pub(crate) use reference_occurrence::ReferenceOccurrenceSearchError;
#[cfg(all(test, feature = "search-runtime"))]
pub(crate) use reference_occurrence::{reference_occurrence_batches, reference_occurrence_schema};
#[cfg(feature = "search-runtime")]
pub(crate) use repo_content_chunk::RepoContentChunkSearchFilters;
#[cfg(any(test, feature = "performance"))]
pub(crate) use repo_content_chunk::repo_content_chunk_file_fingerprints;
#[cfg(all(test, feature = "search-runtime"))]
pub(crate) use repo_entity::publish_repo_entities;
#[cfg(feature = "search-runtime")]
pub(crate) use repo_entity::{
    RepoEntityOverviewSummary, RepoEntitySearchError, search_repo_entity_example_results,
    search_repo_entity_import_results, search_repo_entity_module_results,
    search_repo_entity_symbol_results, summarize_repo_entity_overview,
};
#[cfg(feature = "search-runtime")]
pub(crate) use repo_staging::{
    RepoStagedMutationAction, RepoStagedMutationPlan, plan_repo_staged_mutation,
};
#[cfg(feature = "search-runtime")]
pub(crate) use semantic_fingerprint::{
    ast_hits_fingerprint, attachment_hits_fingerprint, reference_hits_fingerprint,
    stable_payload_fingerprint,
};
#[cfg(feature = "search-runtime")]
pub(crate) use service::RepoSearchAvailability;
#[cfg(feature = "search-runtime")]
pub(crate) use service::RepoSearchPublicationState;
#[cfg(feature = "search-runtime")]
pub(crate) use service::RepoSearchQueryCacheKeyInput;
#[cfg(feature = "search-runtime")]
pub(crate) use service::SearchBuildRepeatWorkTelemetry;
#[cfg(feature = "search-runtime")]
pub use service::SearchPlaneService;
#[cfg(feature = "search-runtime")]
pub(crate) use source_snapshot::{
    SourceSnapshotEntry, build_source_snapshot_entry, source_snapshot_entry_cache_key,
};
#[cfg(feature = "search-runtime")]
pub use status::{
    SearchCorpusIssue, SearchCorpusIssueCode, SearchCorpusIssueFamily, SearchCorpusIssueSummary,
    SearchCorpusStatus, SearchCorpusStatusAction, SearchCorpusStatusReason,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchMaintenancePolicy,
    SearchMaintenanceStatus, SearchPlanePhase, SearchPlaneStatusSnapshot, SearchQueryTelemetry,
    SearchQueryTelemetrySource, SearchRepoReadPressure,
};
#[cfg(feature = "repo-lexical-index")]
pub use tantivy::{
    SearchDocument, SearchDocumentFields, SearchDocumentHit, SearchDocumentIndex,
    SearchDocumentMatchField, TantivyDocumentMatch, TantivyMatcher,
};
