//! Dataset write and ingestion methods included into `VectorStore`.

mod batch_builders;
mod context;
mod dataset_lifecycle;
mod ingest_ops;

pub(super) use std::sync::Arc;

pub(super) use anyhow::Result;
pub(super) use lance::dataset::Dataset;

pub(super) use super::VectorStore;
pub(super) use crate::VectorStoreError;
pub(super) use crate::ops::MergeInsertStats;
pub(crate) use context::{
    build_vector_list_array, default_write_params, has_lance_data, parse_document_metadata_columns,
    parse_metadata_value, validate_document_batch_inputs,
};
