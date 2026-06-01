//! DuckDB-backed document extraction job registry facade.

mod artifacts;
#[cfg(not(feature = "duckdb"))]
mod disabled;
#[cfg(feature = "duckdb")]
mod hash;
#[cfg(feature = "duckdb")]
mod jobs;
#[cfg(feature = "duckdb")]
mod lifecycle;
#[cfg(feature = "duckdb")]
mod queries;
#[cfg(feature = "duckdb")]
mod recovery;
mod types;

pub(super) use artifacts::artifact_ready;
pub(crate) use artifacts::default_output_dir;
pub(super) use types::DocumentExtractJobRegistry;
pub(crate) use types::{DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus};

#[cfg(all(test, feature = "duckdb"))]
use std::fs;

#[cfg(all(test, feature = "duckdb"))]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/registry.rs"]
mod tests;
