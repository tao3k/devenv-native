//! DuckDB-backed document extraction job registry facade.

mod hash;
mod jobs;
mod lifecycle;
mod queries;
mod recovery;
mod types;
mod utils;

pub(super) use types::DocumentExtractJobRegistry;
pub(crate) use types::{DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus};
pub(super) use utils::artifact_ready;
pub(crate) use utils::default_output_dir;

#[cfg(test)]
use std::fs;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/registry.rs"]
mod tests;
