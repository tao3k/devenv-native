//! Stable test-support APIs for integration harnesses.

use xiuxian_types::VectorSearchResult;

/// Encodes vector search results into Arrow IPC bytes.
///
/// # Errors
///
/// Returns an error when projection is invalid or Arrow serialization fails.
pub fn search_results_to_ipc(
    results: &[VectorSearchResult],
    projection: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    crate::search_impl::search_results_to_ipc_for_test(results, projection)
}
