#[path = "lookup/mod.rs"]
mod lookup;
#[cfg(test)]
#[path = "../../../../tests/unit/search/reference_occurrence/query/mod.rs"]
mod tests;

pub use lookup::ReferenceOccurrenceSearchError;
pub(crate) use lookup::search_reference_occurrences;
