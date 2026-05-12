//! `pybindings::dep_indexer_py::registration` owns Wendao pybindings dep indexer py registration behavior.

use pyo3::types::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

use super::{
    PyDependencyConfig, PyDependencyIndexResult, PyDependencyIndexer, PyDependencyStats,
    PyExternalDependency, PyExternalSymbol, PySymbolIndex,
};

/// Register dependency indexer module with Python.
///
/// # Errors
///
/// Returns `PyErr` when class registration fails.
pub fn register_dependency_indexer_module(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyExternalSymbol>()?;
    m.add_class::<PyExternalDependency>()?;
    m.add_class::<PySymbolIndex>()?;
    m.add_class::<PyDependencyConfig>()?;
    m.add_class::<PyDependencyIndexResult>()?;
    m.add_class::<PyDependencyStats>()?;
    m.add_class::<PyDependencyIndexer>()?;
    Ok(())
}
