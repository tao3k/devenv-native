mod command;
mod context;
mod navigation;
mod node;
mod page;
mod page_index;
mod page_index_outline;
mod search;
mod search_page_index;
mod segment;
mod toc;
mod tree;

pub(crate) use self::command::DocsCommand;
#[cfg(test)]
pub(crate) use self::command::docs;
pub(crate) use self::context::DocsContextArgs;
pub(crate) use self::navigation::DocsNavigationArgs;
pub(crate) use self::node::DocsNodeArgs;
pub(crate) use self::page::DocsPageArgs;
pub(crate) use self::page_index::DocsPageIndexArgs;
pub(crate) use self::page_index_outline::DocsPageIndexOutlineArgs;
pub(crate) use self::search::DocsSearchArgs;
pub(crate) use self::search_page_index::DocsSearchPageIndexArgs;
pub(crate) use self::segment::DocsSegmentArgs;
pub(crate) use self::toc::DocsTocArgs;
pub(crate) use self::tree::DocsTreeArgs;

#[cfg(test)]
#[path = "../../../../../../tests/unit/bin/wendao/types/commands/docs/mod.rs"]
mod tests;
