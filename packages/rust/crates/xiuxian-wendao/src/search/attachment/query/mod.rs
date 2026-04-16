mod lookup;

#[cfg(test)]
#[path = "../../../../tests/unit/search/attachment/query/mod.rs"]
mod tests;

pub(crate) use lookup::{AttachmentSearchError, search_attachment_hits};
