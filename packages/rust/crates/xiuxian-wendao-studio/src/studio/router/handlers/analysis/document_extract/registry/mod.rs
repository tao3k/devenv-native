//! DuckDB-backed document extraction job registry facade.

mod artifacts;
mod hash;
mod jobs;
mod lifecycle;
mod queries;
mod recovery;
mod types;

pub(super) use artifacts::artifact_ready;
pub(crate) use artifacts::default_output_dir;
pub(super) use types::DocumentExtractJobRegistry;
pub(crate) use types::{DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus};

#[cfg(test)]
use std::fs;

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/registry.rs"]
mod tests;
