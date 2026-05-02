//! Parser-owned document metadata aggregation.

mod api;
mod types;

pub use api::parse_markdown_document;
pub(crate) use api::parse_markdown_document_from_parts;
pub use types::{
    DocumentCore, DocumentEnvelope, DocumentFormat, MarkdownDocument, OrgDocument,
    OrgDocumentMetadata,
};
