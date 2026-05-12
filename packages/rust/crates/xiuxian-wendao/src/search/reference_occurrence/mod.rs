//! `search::reference_occurrence` owns Wendao search reference occurrence behavior.

#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(any(test, feature = "test-support"))]
pub use build::ReferenceOccurrenceBuildError;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::ensure_reference_occurrence_index_started;
pub(crate) use build::ensure_reference_occurrence_index_started_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::publish_reference_occurrences_from_projects;
pub use query::ReferenceOccurrenceSearchError;
pub(crate) use query::search_reference_occurrences;
#[cfg(test)]
pub(crate) use schema::reference_occurrence_batches;
