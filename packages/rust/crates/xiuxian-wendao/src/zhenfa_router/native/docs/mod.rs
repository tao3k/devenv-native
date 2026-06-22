//! `zhenfa_router::native::docs` owns Wendao zhenfa router native docs behavior.

mod document;
mod navigation;
mod node;
mod page_index;
mod page_index_outline;
mod page_index_tree;
mod retrieval_context;
mod search;
mod search_page_index;
mod segment;
mod shared;
mod toc;

pub use document::{WendaoDocsGetDocumentArgs, wendao_docs_get_document};
pub use navigation::{WendaoDocsGetNavigationArgs, wendao_docs_get_navigation};
pub use node::{WendaoDocsGetDocumentNodeArgs, wendao_docs_get_document_node};
pub use page_index::{WendaoDocsGetPageIndexArgs, wendao_docs_get_page_index};
pub use page_index_outline::{
    WendaoDocsGetPageIndexOutlineArgs, wendao_docs_get_page_index_outline,
};
pub use page_index_tree::{WendaoDocsGetPageIndexTreeArgs, wendao_docs_get_page_index_tree};
pub use retrieval_context::{WendaoDocsGetRetrievalContextArgs, wendao_docs_get_retrieval_context};
pub use search::{WendaoDocsSearchArgs, wendao_docs_search};
pub use search_page_index::{WendaoDocsSearchPageIndexArgs, wendao_docs_search_page_index};
pub use segment::{WendaoDocsGetDocumentSegmentArgs, wendao_docs_get_document_segment};
pub use toc::{WendaoDocsGetTocDocumentsArgs, wendao_docs_get_toc_documents};

#[cfg(test)]
#[path = "../../../../tests/unit/zhenfa_router/native/docs/mod.rs"]
mod tests;
