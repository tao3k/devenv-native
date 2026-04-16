mod contracts;
mod options;
#[cfg(feature = "zhenfa-router")]
mod runtime;
mod segment;
mod service;

pub use contracts::{
    DOCS_CONTRACT_IDS, DOCS_NAVIGATION_CONTRACT_ID, DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID,
    DocsCapabilityContractAssets, DocsCapabilityContractSnapshot, DocsCliContractSnapshot,
    DocsContractDefaultValue, DocsContractParamSnapshot, DocsHttpContractSnapshot,
    DocsNavigationToolArgs, DocsRetrievalContextToolArgs, DocsToolContractSnapshot,
    docs_capability_contract_assets, docs_capability_contract_snapshot,
    docs_capability_schema_snapshot,
};
pub use options::{DocsNavigationOptions, DocsRetrievalContextOptions};
#[cfg(feature = "zhenfa-router")]
pub(crate) use runtime::{DocsToolRuntime, DocsToolRuntimeHandle};
pub use segment::DocsDocumentSegmentResult;
pub(crate) use segment::build_document_segment;
pub use service::DocsToolService;
