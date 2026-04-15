use futures::TryStreamExt;
use lance_index::scalar::FullTextSearchQuery;
use num_traits::ToPrimitive;
use serde_json::Value;
use xiuxian_types::VectorSearchResult;

use crate::search::SearchOptions;
use crate::{
    CONTENT_COLUMN, HybridSearchResult, ID_COLUMN, KEYWORD_WEIGHT, METADATA_COLUMN, RRF_K,
    SEMANTIC_WEIGHT, VECTOR_COLUMN, VectorStore, VectorStoreError, apply_weighted_rrf,
};

mod boost_ops;
mod confidence;
mod fetch_ops;
mod filter;
mod hybrid_ops;
mod ipc;
mod rows;
mod vector_ops;

use confidence::KEYWORD_BOOST;
use ipc::{search_results_to_ipc, tool_search_results_to_ipc};
use rows::{
    FtsRowColumns, build_fts_result_row, build_search_result_row, extract_vector_row_columns,
    required_lance_string_column,
};

pub use filter::json_to_lance_where;

fn f64_to_f32_saturating(value: f64) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    if value > f64::from(f32::MAX) {
        return f32::MAX;
    }
    if value < f64::from(f32::MIN) {
        return f32::MIN;
    }
    value.to_f32().unwrap_or_else(|| {
        if value.is_sign_negative() {
            f32::MIN
        } else {
            f32::MAX
        }
    })
}

pub(crate) fn search_results_to_ipc_for_test(
    results: &[VectorSearchResult],
    projection: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    search_results_to_ipc(results, projection)
}

pub(crate) fn tool_search_results_to_ipc_for_test(
    results: &[crate::skill::ToolSearchResult],
) -> Result<Vec<u8>, String> {
    tool_search_results_to_ipc(results)
}

#[must_use]
pub(crate) fn keyword_boost_for_test() -> f32 {
    KEYWORD_BOOST
}
