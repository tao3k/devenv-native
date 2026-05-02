//! Local `wendao get` projections for document table-of-contents and page-index views.

mod command;
mod config;
#[cfg(feature = "performance")]
pub mod perf_support;
mod run;
mod scope;
mod types;

pub use command::GetCommand;
pub(crate) use run::run_command;
pub use scope::GetScopeArgs;
pub use types::{
    DocsPageIndexDocumentsResult, DocsPageIndexTreesResult, ProjectedPageIndexDocument,
    ProjectedPageIndexLink, ProjectedPageIndexNode, ProjectedPageIndexSection,
    ProjectedPageIndexTree, ProjectionPageKind,
};
