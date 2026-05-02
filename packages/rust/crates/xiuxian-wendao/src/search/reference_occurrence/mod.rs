#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(test)]
pub(crate) use build::ensure_reference_occurrence_index_started;
pub(crate) use build::ensure_reference_occurrence_index_started_with_scanned_files;
#[cfg(test)]
pub(crate) use build::{
    ReferenceOccurrenceBuildError, publish_reference_occurrences_from_projects,
};
pub(crate) use query::{ReferenceOccurrenceSearchError, search_reference_occurrences};
#[cfg(test)]
pub(crate) use schema::reference_occurrence_batches;
