//! Dependency parser for Python `pyproject.toml` package metadata.

use std::fs::read_to_string;
use std::path::Path;

use super::regex::{RE_DEP, RE_EXACT_DEP, RE_SIMPLE};
use super::types::PyprojectDependency;

/// Parse dependencies from a `pyproject.toml` file.
///
/// # Errors
///
/// Returns I/O errors when the pyproject file cannot be read.
pub fn parse_pyproject_dependencies(
    path: &Path,
) -> Result<Vec<PyprojectDependency>, std::io::Error> {
    let content = read_to_string(path)?;
    Ok(parse_pyproject_dependency_content(content.as_str()))
}

fn parse_pyproject_dependency_content(content: &str) -> Vec<PyprojectDependency> {
    match content.parse::<toml::Value>() {
        Ok(toml) => parse_toml_project_dependencies(&toml),
        Err(_) => parse_regex_project_dependencies(content),
    }
}

fn parse_toml_project_dependencies(toml: &toml::Value) -> Vec<PyprojectDependency> {
    toml.get("project")
        .and_then(|project| project.get("dependencies"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter_map(parse_pyproject_dep)
        .map(|(name, version)| PyprojectDependency::new(name, Some(version)))
        .collect()
}

fn parse_regex_project_dependencies(content: &str) -> Vec<PyprojectDependency> {
    RE_DEP
        .captures_iter(content)
        .map(|cap| {
            let name = cap[1].to_string();
            let version = cap[2].trim().to_string();
            PyprojectDependency::new(name, Some(version))
        })
        .collect()
}

fn parse_pyproject_dep(dep: &str) -> Option<(String, String)> {
    RE_EXACT_DEP
        .captures(dep)
        .map(|cap| (cap[1].to_string(), cap[2].to_string()))
        .or_else(|| {
            RE_SIMPLE
                .captures(dep)
                .map(|cap| (cap[1].to_string(), "latest".to_string()))
        })
}
