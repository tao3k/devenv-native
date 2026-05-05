use crate::schemas;
use pyo3::types::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyErr, PyResult, pyfunction, wrap_pyfunction};

#[pyfunction]
fn get_schema(name: &str) -> PyResult<String> {
    schemas::get_schema(name)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown schema name: {name}"))
        })
}

/// Registers schema helper functions into the Python extension module.
pub(crate) fn register_schema_module(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_schema, m.py())?)?;
    Ok(())
}
