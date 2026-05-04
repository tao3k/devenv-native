//! Local `wendao get` command execution.

mod facade;
mod render;
mod section_level;

pub(super) use super::{
    DocsPageIndexDocumentsResult, DocsPageIndexTreesResult, GetCommand, GetScopeArgs,
    ProjectedPageIndexDocument, ProjectedPageIndexLink, ProjectedPageIndexNode,
    ProjectedPageIndexSection, ProjectedPageIndexTree, ProjectionPageKind, configured_ignore_dirs,
};
pub(crate) use facade::run_command;
#[cfg(feature = "performance")]
pub(crate) use facade::{
    build_local_page_index_trees_with_ignore, canonical_scope_target, default_ignore_dir_names,
};
pub(super) use section_level::effective_section_level;
