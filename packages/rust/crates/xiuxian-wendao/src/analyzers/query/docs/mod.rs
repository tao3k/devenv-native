//! Docs-facing query and response contracts for repository intelligence.

mod coverage;
#[path = "planner/mod.rs"]
mod planner;
mod search;

pub use coverage::{
    DocCoverageQuery, DocCoverageResult, DocsProjectedGapReportQuery, DocsProjectedGapReportResult,
};
pub use planner::{
    DocsPlannerItemQuery, DocsPlannerItemResult, DocsPlannerQueueGroup, DocsPlannerQueueQuery,
    DocsPlannerQueueResult, DocsPlannerRankHit, DocsPlannerRankQuery, DocsPlannerRankReason,
    DocsPlannerRankReasonCode, DocsPlannerRankResult, DocsPlannerSearchHit, DocsPlannerSearchQuery,
    DocsPlannerSearchResult, DocsPlannerWorksetBalance, DocsPlannerWorksetFamilyBalanceEntry,
    DocsPlannerWorksetFamilyGroup, DocsPlannerWorksetGapKindBalanceEntry, DocsPlannerWorksetGroup,
    DocsPlannerWorksetQuery, DocsPlannerWorksetQuotaHint, DocsPlannerWorksetResult,
    DocsPlannerWorksetStrategy, DocsPlannerWorksetStrategyCode, DocsPlannerWorksetStrategyReason,
    DocsPlannerWorksetStrategyReasonCode,
};
pub use search::{
    DocsFamilyClusterQuery, DocsFamilyClusterResult, DocsFamilyContextQuery,
    DocsFamilyContextResult, DocsFamilySearchQuery, DocsFamilySearchResult,
    DocsMarkdownDocumentsQuery, DocsMarkdownDocumentsResult, DocsNavigationQuery,
    DocsNavigationResult, DocsNavigationSearchQuery, DocsNavigationSearchResult,
    DocsPageIndexDocumentsQuery, DocsPageIndexDocumentsResult, DocsPageIndexNodeQuery,
    DocsPageIndexNodeResult, DocsPageIndexTreeQuery, DocsPageIndexTreeResult,
    DocsPageIndexTreeSearchQuery, DocsPageIndexTreeSearchResult, DocsPageIndexTreesQuery,
    DocsPageIndexTreesResult, DocsPageQuery, DocsPageResult, DocsRetrievalContextQuery,
    DocsRetrievalContextResult, DocsRetrievalHitQuery, DocsRetrievalHitResult, DocsRetrievalQuery,
    DocsRetrievalResult, DocsSearchQuery, DocsSearchResult,
};
