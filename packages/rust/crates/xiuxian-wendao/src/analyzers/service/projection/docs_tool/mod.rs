//! `analyzers::service::projection::docs_tool` owns Wendao service projection docs tool behavior.

mod contracts;
mod facade;
mod options;
#[cfg(feature = "zhenfa-router")]
mod runtime;
mod segment;

pub use contracts::{
    DOCS_CONTRACT_IDS, DOCS_DOCUMENT_CONTRACT_ID, DOCS_NAVIGATION_CONTRACT_ID,
    DOCS_PAGE_INDEX_TREE_CONTRACT_ID, DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID, DOCS_SEARCH_CONTRACT_ID,
    DocsCapabilityContractAssets, DocsCapabilityContractSnapshot, DocsCliContractSnapshot,
    DocsContractDefaultValue, DocsContractParamSnapshot, DocsDocumentToolArgs,
    DocsHttpContractSnapshot, DocsNavigationToolArgs, DocsPageIndexTreeToolArgs,
    DocsRetrievalContextToolArgs, DocsSearchToolArgs, DocsToolContractSnapshot,
    docs_capability_contract_assets, docs_capability_contract_snapshot,
    docs_capability_schema_snapshot,
};
pub use facade::DocsToolService;
pub use options::{DocsNavigationOptions, DocsRetrievalContextOptions};
#[cfg(feature = "zhenfa-router")]
pub(crate) use runtime::{DocsToolRuntime, DocsToolRuntimeHandle};
pub use segment::DocsDocumentSegmentResult;
pub(crate) use segment::build_document_segment;
