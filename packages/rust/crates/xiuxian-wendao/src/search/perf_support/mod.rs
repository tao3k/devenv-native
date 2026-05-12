//! Repo-content search performance fixtures.

mod fixture;
mod mutation;
mod query;
mod samples;

pub use mutation::{
    RepoContentParquetMutationBenchmarkFixture, RepoContentParquetMutationBenchmarkIteration,
};
pub use query::{RepoContentQueryBenchmarkFixture, RepoContentQueryBenchmarkIteration};
pub use samples::{
    RepoContentFlightBatchBenchmarkSample, RepoContentParquetMutationBenchmarkSnapshot,
    RepoContentQueryBenchmarkSample, RepoContentQueryBenchmarkSnapshot,
};

#[cfg(test)]
#[path = "../../../tests/unit/search/perf_support.rs"]
mod tests;
