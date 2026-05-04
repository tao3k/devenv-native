//! Administrative table lifecycle methods included into `VectorStore`.

mod context;
mod delete_ops;
mod guards;
mod index_ops;
mod table_ops;

pub(super) use std::sync::Arc;

pub(super) use anyhow::Result;
pub(super) use futures::TryStreamExt;
pub(super) use lance::Dataset;
pub(super) use lance::index::vector::VectorIndexParams;
pub(super) use lance_index::IndexType;
pub(super) use lance_index::scalar::inverted::tokenizer::InvertedIndexParams;
pub(super) use lance_index::scalar::{BuiltinIndexType, ScalarIndexParams};
pub(super) use lance_linalg::distance::DistanceType;

pub use context::ScalarIndexType;
pub(crate) use context::{
    index_type_name, is_dataset_not_found_or_invalid, open_uri_for_background,
};

pub(super) use super::VectorStore;
pub(super) use crate::ops::{
    FragmentInfo, TableColumnAlteration, TableColumnType, TableInfo, TableNewColumn,
    TableVersionInfo,
};
pub(super) use crate::{ID_COLUMN, METADATA_COLUMN, VECTOR_COLUMN, VectorStoreError};
