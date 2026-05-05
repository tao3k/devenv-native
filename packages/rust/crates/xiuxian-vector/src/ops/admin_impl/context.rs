//! Administrative method context shared by table, index, and guard operations.

use super::{Dataset, Result};

/// Open a dataset by URI for background tasks (Send-safe; no store state).
pub(crate) async fn open_uri_for_background(
    uri: &str,
    index_cache_size_bytes: Option<usize>,
) -> Result<Dataset, crate::error::VectorStoreError> {
    match index_cache_size_bytes {
        None => Dataset::open(uri).await.map_err(Into::into),
        Some(n) => lance::dataset::builder::DatasetBuilder::from_uri(uri)
            .with_index_cache_size_bytes(n)
            .load()
            .await
            .map_err(Into::into),
    }
}

/// True if the error indicates the dataset path exists but is not a valid Lance dataset
/// (e.g. after `drop_table` removed `_versions` / `data`).
pub(crate) fn is_dataset_not_found_or_invalid(e: &crate::error::VectorStoreError) -> bool {
    match e {
        crate::error::VectorStoreError::LanceDB(inner) => {
            let s = inner.to_string();
            s.contains("DatasetNotFound") || s.contains("NotFound") || s.contains("_versions")
        }
        _ => false,
    }
}

/// Scalar index type for exact, categorical, text, or array filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarIndexType {
    /// `BTree`: exact match, range queries (e.g. `skill_name = 'git'`).
    BTree,
    /// `Bitmap`: low-cardinality enums (e.g. `category = 'git'`).
    Bitmap,
    /// Inverted index for generic text or array columns.
    Inverted,
}

pub(crate) fn index_type_name(t: ScalarIndexType) -> &'static str {
    match t {
        ScalarIndexType::BTree => "btree",
        ScalarIndexType::Bitmap => "bitmap",
        ScalarIndexType::Inverted => "inverted",
    }
}
