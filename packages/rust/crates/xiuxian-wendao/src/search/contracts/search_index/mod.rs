mod conversions;
#[path = "definitions/mod.rs"]
mod definitions;
#[path = "diagnostics/mod.rs"]
mod diagnostics;
mod status;
#[cfg(test)]
#[path = "../../../../tests/unit/search/contracts/search_index/mod.rs"]
mod tests;

pub use definitions::{
    SearchCorpusIndexStatus, SearchIndexAggregateMaintenanceSummary,
    SearchIndexAggregateQueryTelemetry, SearchIndexAggregateStatusReason,
    SearchIndexMaintenanceStatus, SearchIndexPhase, SearchIndexRepoReadPressure,
    SearchIndexStatusResponse,
};
#[cfg(test)]
pub(crate) use definitions::{
    SearchIndexIssueCode, SearchIndexIssueFamily, SearchIndexIssueSummary,
    SearchIndexQueryTelemetrySource, SearchIndexStatusAction, SearchIndexStatusReason,
    SearchIndexStatusReasonCode, SearchIndexStatusSeverity,
};
#[cfg(all(test, feature = "duckdb"))]
pub(crate) use diagnostics::configured_status_diagnostics_engine_kind;
