//! Query request and response contracts for repository intelligence.

#[path = "docs/mod.rs"]
mod docs;
mod example;
mod family;
mod gaps;
mod imports;
mod index_tree;
mod module;
mod navigation;
mod overview;
mod projected_pages;
mod refine;
mod retrieval;
mod symbol;
mod sync;

pub use docs::{
    DocCoverageQuery, DocCoverageResult, DocsFamilyClusterQuery, DocsFamilyClusterResult,
    DocsFamilyContextQuery, DocsFamilyContextResult, DocsFamilySearchQuery, DocsFamilySearchResult,
    DocsMarkdownDocumentsQuery, DocsMarkdownDocumentsResult, DocsNavigationQuery,
    DocsNavigationResult, DocsNavigationSearchQuery, DocsNavigationSearchResult,
    DocsPageIndexDocumentsQuery, DocsPageIndexDocumentsResult, DocsPageIndexNodeQuery,
    DocsPageIndexNodeResult, DocsPageIndexTreeQuery, DocsPageIndexTreeResult,
    DocsPageIndexTreeSearchQuery, DocsPageIndexTreeSearchResult, DocsPageIndexTreesQuery,
    DocsPageIndexTreesResult, DocsPageQuery, DocsPageResult, DocsPlannerItemQuery,
    DocsPlannerItemResult, DocsPlannerQueueGroup, DocsPlannerQueueQuery, DocsPlannerQueueResult,
    DocsPlannerRankHit, DocsPlannerRankQuery, DocsPlannerRankReason, DocsPlannerRankReasonCode,
    DocsPlannerRankResult, DocsPlannerSearchHit, DocsPlannerSearchQuery, DocsPlannerSearchResult,
    DocsPlannerWorksetBalance, DocsPlannerWorksetFamilyBalanceEntry, DocsPlannerWorksetFamilyGroup,
    DocsPlannerWorksetGapKindBalanceEntry, DocsPlannerWorksetGroup, DocsPlannerWorksetQuery,
    DocsPlannerWorksetQuotaHint, DocsPlannerWorksetResult, DocsPlannerWorksetStrategy,
    DocsPlannerWorksetStrategyCode, DocsPlannerWorksetStrategyReason,
    DocsPlannerWorksetStrategyReasonCode, DocsProjectedGapReportQuery,
    DocsProjectedGapReportResult, DocsRetrievalContextQuery, DocsRetrievalContextResult,
    DocsRetrievalHitQuery, DocsRetrievalHitResult, DocsRetrievalQuery, DocsRetrievalResult,
    DocsSearchQuery, DocsSearchResult,
};
pub use example::{ExampleSearchHit, ExampleSearchQuery, ExampleSearchResult};
pub use family::{
    ProjectedPageFamilyCluster, ProjectedPageFamilyContextEntry, ProjectedPageFamilySearchHit,
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageFamilyClusterResult,
    RepoProjectedPageFamilyContextQuery, RepoProjectedPageFamilyContextResult,
    RepoProjectedPageFamilySearchQuery, RepoProjectedPageFamilySearchResult,
};
pub use gaps::{
    ProjectedGapKind, ProjectedGapRecord, ProjectedGapSummary, ProjectedGapSummaryEntry,
    RepoProjectedGapReportQuery, RepoProjectedGapReportResult,
};
pub use imports::{ImportSearchHit, ImportSearchQuery, ImportSearchResult};
pub use index_tree::{
    ProjectedPageIndexNodeContext, ProjectedPageIndexNodeHit, RepoProjectedPageIndexNodeQuery,
    RepoProjectedPageIndexNodeResult, RepoProjectedPageIndexTreeQuery,
    RepoProjectedPageIndexTreeResult, RepoProjectedPageIndexTreeSearchQuery,
    RepoProjectedPageIndexTreeSearchResult, RepoProjectedPageIndexTreesQuery,
    RepoProjectedPageIndexTreesResult,
};
pub use module::{ModuleSearchHit, ModuleSearchQuery, ModuleSearchResult, RepoBacklinkItem};
pub use navigation::{
    ProjectedPageNavigationSearchHit, RepoProjectedPageNavigationQuery,
    RepoProjectedPageNavigationResult, RepoProjectedPageNavigationSearchQuery,
    RepoProjectedPageNavigationSearchResult,
};
pub use overview::{RepoOverviewQuery, RepoOverviewResult};
pub use projected_pages::{
    RepoProjectedPageQuery, RepoProjectedPageResult, RepoProjectedPageSearchQuery,
    RepoProjectedPageSearchResult, RepoProjectedPagesQuery, RepoProjectedPagesResult,
};
pub use refine::{RefineEntityDocRequest, RefineEntityDocResponse};
pub use retrieval::{
    ProjectedRetrievalHit, ProjectedRetrievalHitKind, RepoProjectedRetrievalContextQuery,
    RepoProjectedRetrievalContextResult, RepoProjectedRetrievalHitQuery,
    RepoProjectedRetrievalHitResult, RepoProjectedRetrievalQuery, RepoProjectedRetrievalResult,
};
pub use symbol::{SymbolSearchHit, SymbolSearchQuery, SymbolSearchResult};
pub use sync::{
    RepoSourceKind, RepoSyncDriftState, RepoSyncFreshnessSummary, RepoSyncHealthState,
    RepoSyncLifecycleSummary, RepoSyncMode, RepoSyncQuery, RepoSyncResult, RepoSyncRevisionSummary,
    RepoSyncStalenessState, RepoSyncState, RepoSyncStatusSummary,
};
