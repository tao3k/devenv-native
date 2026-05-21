//! `skill_runtime::zhixing::indexer` owns Wendao skill runtime zhixing indexer behavior.

mod documents;
mod file_discovery;
#[path = "resource_graph/mod.rs"]
mod resource_graph;
mod stats;
#[path = "tasks/mod.rs"]
mod tasks;
mod types;

pub use types::{ZhixingIndexSummary, ZhixingWendaoIndexer};
