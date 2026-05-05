//! Shared Tantivy-backed search primitives.

mod compare;
mod document;
mod fields;
mod fragments;
mod identifier;
#[path = "index/mod.rs"]
mod index;
mod matcher;
mod tokenizer;

pub use document::{SearchDocument, SearchDocumentHit, SearchDocumentMatchField};
pub use fields::SearchDocumentFields;
pub use index::SearchDocumentIndex;
pub use matcher::{TantivyDocumentMatch, TantivyMatcher};

#[cfg(test)]
#[path = "../../../tests/unit/search/tantivy/mod.rs"]
mod tests;
