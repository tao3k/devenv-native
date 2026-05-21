//! `search::tantivy::index` owns Wendao search tantivy index behavior.

mod core;
mod exact;
mod fuzzy;
mod helpers;
mod prefix;

pub use core::SearchDocumentIndex;
