//! `search::reference_occurrence::query::lookup` owns Wendao reference occurrence query lookup behavior.

mod candidates;
mod decode;
mod helpers;
mod route;

pub use route::ReferenceOccurrenceSearchError;
pub(crate) use route::search_reference_occurrences;
