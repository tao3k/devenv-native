//! Search execution leaf modules for vector retrieval and IPC conversion.

use futures::TryStreamExt;
use serde_json::Value;
use xiuxian_types::VectorSearchResult;

use crate::search::SearchOptions;
use crate::{
    CONTENT_COLUMN, ID_COLUMN, METADATA_COLUMN, VECTOR_COLUMN, VectorStore, VectorStoreError,
};

mod fetch_ops;
mod filter;
mod ipc;
mod rows;
mod vector_ops;

use ipc::search_results_to_ipc;
use rows::{build_search_result_row, extract_vector_row_columns};

pub use filter::json_to_lance_where;
pub(crate) use ipc::search_results_to_ipc as search_results_to_ipc_for_test;
