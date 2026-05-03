//! Graph traversal and metadata command execution.

mod dispatch;
#[path = "metadata_resolve.rs"]
mod metadata_resolve;
#[path = "neighbors_related.rs"]
mod neighbors_related;
#[path = "stats_toc.rs"]
mod stats_toc;

pub(super) use dispatch::handle;
