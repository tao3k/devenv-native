//! `search::local_symbol::build` owns Wendao search local symbol build behavior.

mod orchestration;
mod partitions;
mod plan;
mod types;
mod write;

#[cfg(test)]
#[path = "../../../../tests/unit/search/local_symbol/build/mod.rs"]
mod tests;

#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::ensure_local_symbol_index_started;
pub(crate) use orchestration::ensure_local_symbol_index_started_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::publish_local_symbol_hits;
pub(crate) use plan::plan_local_symbol_build_with_scanned_files;
#[cfg(test)]
pub(crate) use plan::{fingerprint_projects, plan_local_symbol_build};
#[cfg(any(test, feature = "test-support"))]
pub use types::LocalSymbolBuildError;
pub(crate) use types::{
    LocalSymbolBuildPlan, LocalSymbolPartitionBuildPlan, LocalSymbolWriteResult,
};
pub(crate) use write::write_local_symbol_epoch;
