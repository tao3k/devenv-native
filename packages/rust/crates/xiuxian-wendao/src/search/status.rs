#[path = "status/corpus.rs"]
pub(crate) mod corpus;
#[path = "status/issues.rs"]
pub(crate) mod issues;
#[path = "status/maintenance.rs"]
pub(crate) mod maintenance;
#[path = "status/phase.rs"]
pub(crate) mod phase;
#[path = "status/reason.rs"]
pub(crate) mod reason;
#[path = "status/snapshot.rs"]
pub(crate) mod snapshot;
#[path = "status/telemetry.rs"]
pub(crate) mod telemetry;
#[cfg(test)]
#[path = "../../tests/unit/search/status/mod.rs"]
mod tests;

pub use corpus::SearchCorpusStatus;
pub use issues::{
    SearchCorpusIssue, SearchCorpusIssueCode, SearchCorpusIssueFamily, SearchCorpusIssueSummary,
};
pub use maintenance::{SearchMaintenancePolicy, SearchMaintenanceStatus};
pub use phase::SearchPlanePhase;
pub use reason::{
    SearchCorpusStatusAction, SearchCorpusStatusReason, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity,
};
pub use snapshot::{SearchPlaneStatusSnapshot, SearchRepoReadPressure};
pub use telemetry::{SearchQueryTelemetry, SearchQueryTelemetrySource};
