#[path = "lookup/mod.rs"]
mod lookup;

#[cfg(test)]
#[path = "../../../../tests/unit/search/attachment/query/mod.rs"]
mod tests;

pub use lookup::AttachmentSearchError;
pub(crate) use lookup::search_attachment_hits;
