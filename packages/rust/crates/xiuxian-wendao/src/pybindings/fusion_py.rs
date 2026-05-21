//! `PyO3` bindings for fusion recall boost.

use pyo3::types::{PyAny, PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods};
use pyo3::{Bound, IntoPyObject, Py, PyResult, Python, pyfunction};
use std::collections::{HashMap, HashSet};

use crate::fusion::{RecallResult, apply_link_graph_proximity_boost};

/// Apply `LinkGraph` link/tag proximity boost to recall results (Rust implementation).
///
/// Args:
///     results: List of dicts with keys: source, score, content, title
///     `stem_links`: Dict[str, List[str]] — stem -> linked stems
///     `stem_tags`: Dict[str, List[str]] — stem -> tags
///     `link_boost`: Score boost for bidirectional link
///     `tag_boost`: Score boost for shared tags
///
/// Returns:
///     List of dicts (same structure) with boosted scores, sorted by score desc.
///
/// # Errors
///
/// Returns an error if Python inputs cannot be converted to expected Rust types.
#[pyfunction]
#[pyo3(signature = (results, stem_links, stem_tags, link_boost, tag_boost))]
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn apply_link_graph_proximity_boost_py(
    py: Python<'_>,
    results: &Bound<'_, pyo3::types::PyList>,
    stem_links: &Bound<'_, pyo3::types::PyDict>,
    stem_tags: &Bound<'_, pyo3::types::PyDict>,
    link_boost: f64,
    tag_boost: f64,
) -> PyResult<Vec<Py<PyAny>>> {
    let mut rust_results = extract_recall_results(results)?;
    let links_map = extract_string_set_map(stem_links)?;
    let tags_map = extract_string_set_map(stem_tags)?;

    apply_link_graph_proximity_boost(
        &mut rust_results,
        &links_map,
        &tags_map,
        link_boost,
        tag_boost,
    );

    recall_results_to_py(py, rust_results)
}

fn extract_recall_results(results: &Bound<'_, PyList>) -> PyResult<Vec<RecallResult>> {
    results
        .iter()
        .map(|obj| extract_recall_result(&obj))
        .collect()
}

fn extract_recall_result(obj: &Bound<'_, PyAny>) -> PyResult<RecallResult> {
    let dict = obj.clone().cast_into::<PyDict>()?;
    Ok(RecallResult::new(
        extract_optional_string(&dict, "source")?,
        extract_optional_f64(&dict, "score")?,
        extract_optional_string(&dict, "content")?,
        extract_optional_string(&dict, "title")?,
    ))
}

fn extract_optional_string(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    Ok(dict
        .get_item(key)?
        .and_then(|value: Bound<'_, PyAny>| value.extract::<String>().ok())
        .unwrap_or_default())
}

fn extract_optional_f64(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<f64> {
    Ok(dict
        .get_item(key)?
        .and_then(|value: Bound<'_, PyAny>| value.extract::<f64>().ok())
        .unwrap_or(0.0))
}

fn extract_string_set_map(dict: &Bound<'_, PyDict>) -> PyResult<HashMap<String, HashSet<String>>> {
    dict.iter()
        .map(|(key, value)| Ok((key.extract::<String>()?, extract_string_set(&value)?)))
        .collect()
}

fn extract_string_set(value: &Bound<'_, PyAny>) -> PyResult<HashSet<String>> {
    Ok(value
        .cast::<PyList>()?
        .iter()
        .filter_map(|item: Bound<'_, PyAny>| item.extract::<String>().ok())
        .collect())
}

fn recall_results_to_py(py: Python<'_>, results: Vec<RecallResult>) -> PyResult<Vec<Py<PyAny>>> {
    results
        .into_iter()
        .map(|result| recall_result_to_py(py, result))
        .collect()
}

fn recall_result_to_py(py: Python<'_>, result: RecallResult) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("source", result.source)?;
    dict.set_item("score", result.score)?;
    dict.set_item("content", result.content)?;
    dict.set_item("title", result.title)?;
    Ok(dict.into_pyobject(py)?.into())
}
