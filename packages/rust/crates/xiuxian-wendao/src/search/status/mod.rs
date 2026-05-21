//! `search::status` owns Wendao search status behavior.

#[path = "corpus.rs"]
pub(crate) mod corpus;
#[path = "issues.rs"]
pub(crate) mod issues;
#[path = "maintenance.rs"]
pub(crate) mod maintenance;
#[path = "phase.rs"]
pub(crate) mod phase;
#[path = "reason.rs"]
pub(crate) mod reason;
#[path = "snapshot.rs"]
pub(crate) mod snapshot;
#[path = "telemetry.rs"]
pub(crate) mod telemetry;
#[cfg(test)]
#[path = "../../../tests/unit/search/status/mod.rs"]
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
