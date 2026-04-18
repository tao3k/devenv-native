mod command;
mod context;
mod navigation;
mod node;
mod page;
mod search;
mod search_structure;
mod segment;
mod structure_catalog;
mod toc;
mod tree;
mod tree_outline;

pub(crate) use self::command::DocsCommand;
#[cfg(test)]
pub(crate) use self::command::docs;
pub(crate) use self::context::DocsContextArgs;
pub(crate) use self::navigation::DocsNavigationArgs;
pub(crate) use self::node::DocsNodeArgs;
pub(crate) use self::page::DocsPageArgs;
pub(crate) use self::search::DocsSearchArgs;
pub(crate) use self::search_structure::DocsSearchStructureArgs;
pub(crate) use self::segment::DocsSegmentArgs;
pub(crate) use self::structure_catalog::DocsStructureCatalogArgs;
pub(crate) use self::toc::DocsTocArgs;
pub(crate) use self::tree::DocsTreeArgs;
pub(crate) use self::tree_outline::DocsTreeOutlineArgs;

#[cfg(test)]
#[path = "../../../../../../tests/unit/bin/wendao/types/commands/docs.rs"]
mod tests;
