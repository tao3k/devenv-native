//! `dependency_indexer::indexer::core` owns Wendao dependency indexer indexer core behavior.

mod build;
mod engine;
mod process;
mod query;

pub use engine::DependencyIndexer;
