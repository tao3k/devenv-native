//! Local `wendao get` projections for document table-of-contents and page-index views.

mod command;
mod config;
/// Criterion benchmark helpers for local `wendao get` projections.
#[cfg(feature = "performance")]
pub mod perf_support;
mod run;
mod scope;
mod types;

pub use command::GetCommand;
pub(crate) use config::configured_ignore_dirs;
pub(crate) use run::run_command;
pub use scope::GetScopeArgs;
pub use types::{
    DocsPageIndexDocumentsResult, DocsPageIndexTreesResult, ProjectedPageIndexDocument,
    ProjectedPageIndexLink, ProjectedPageIndexNode, ProjectedPageIndexSection,
    ProjectedPageIndexTree, ProjectionPageKind,
};
