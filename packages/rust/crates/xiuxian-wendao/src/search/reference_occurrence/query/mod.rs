#[path = "lookup/mod.rs"]
mod lookup;
#[cfg(test)]
#[path = "../../../../tests/unit/search/reference_occurrence/query/mod.rs"]
mod tests;

pub(crate) use lookup::{ReferenceOccurrenceSearchError, search_reference_occurrences};
