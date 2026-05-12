//! `repo_index::state::coordinator::runtime` owns Wendao state coordinator runtime behavior.

mod active;
mod incremental;
mod repository;
mod scheduler;
mod task;

#[cfg(test)]
pub(crate) use incremental::PreparedIncrementalAnalysis;
