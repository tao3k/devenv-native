//! `search::reference_occurrence::build` owns Wendao search reference occurrence build behavior.

mod extract;
mod orchestration;
mod plan;
mod types;
mod write;

#[cfg(test)]
#[path = "../../../../tests/unit/search/reference_occurrence/build/mod.rs"]
mod tests;

#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::ensure_reference_occurrence_index_started;
pub(crate) use orchestration::ensure_reference_occurrence_index_started_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::publish_reference_occurrences_from_projects;
#[cfg(all(not(test), feature = "test-support"))]
pub(crate) use plan::plan_reference_occurrence_build;
pub(crate) use plan::plan_reference_occurrence_build_with_scanned_files;
#[cfg(test)]
pub(crate) use plan::{fingerprint_projects, plan_reference_occurrence_build};
#[cfg(any(test, feature = "test-support"))]
pub use types::ReferenceOccurrenceBuildError;
pub(crate) use types::{ReferenceOccurrenceBuildPlan, ReferenceOccurrenceWriteResult};
pub(crate) use write::write_reference_occurrence_epoch;
