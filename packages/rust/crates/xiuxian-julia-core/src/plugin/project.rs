use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_wendao_core::repo_intelligence::{DiagnosticRecord, RepoIntelligenceError};

use super::discovery::relative_path_string;

#[derive(Debug, Clone, Default)]
pub(crate) struct JuliaProjectMetadata {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) uuid: Option<String>,
    pub(crate) dependencies: Vec<String>,
}

pub(crate) fn load_project_metadata(
    repo_id: &str,
    repository_root: &Path,
) -> Result<JuliaProjectMetadata, RepoIntelligenceError> {
    let project_path = repository_root.join("Project.toml");
    if !project_path.is_file() {
        return Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: (repo_id.to_string()).into(),
            message: "missing Project.toml".to_string(),
        });
    }

    let contents =
        fs::read_to_string(&project_path).map_err(|error| RepoIntelligenceError::ConfigLoad {
            message: format!("failed to read `{}`: {error}", project_path.display()),
        })?;
    let toml_value: toml::Value =
        toml::from_str(&contents).map_err(|error| RepoIntelligenceError::ConfigLoad {
            message: format!("failed to parse `{}`: {error}", project_path.display()),
        })?;

    let name = toml_value
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if name.is_empty() {
        return Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: (repo_id.to_string()).into(),
            message: "Project.toml is missing package name".to_string(),
        });
    }

    let version = toml_value
        .get("version")
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    let uuid = toml_value
        .get("uuid")
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    let dependencies = toml_value
        .get("deps")
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    Ok(JuliaProjectMetadata {
        name,
        version,
        uuid,
        dependencies,
    })
}

pub(crate) fn locate_root_module_file(
    repo_id: &str,
    repository_root: &Path,
    project_name: &str,
    diagnostics: &mut Vec<DiagnosticRecord>,
) -> Result<PathBuf, RepoIntelligenceError> {
    let src_dir = repository_root.join("src");
    if !src_dir.is_dir() {
        return Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: (repo_id.to_string()).into(),
            message: "missing src/ directory".to_string(),
        });
    }

    let expected = src_dir.join(format!("{project_name}.jl"));
    if expected.is_file() {
        return Ok(expected);
    }

    let mut candidates = fs::read_dir(&src_dir)
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to scan `{}`: {error}", src_dir.display()),
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "jl"))
        .collect::<Vec<_>>();
    candidates.sort();

    let Some(fallback) = candidates.into_iter().next() else {
        return Err(RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: (repo_id.to_string()).into(),
            message: format!("no Julia source files found under `{}`", src_dir.display()),
        });
    };

    diagnostics.push(DiagnosticRecord {
        repo_id: (repo_id.to_string()).into(),
        path: relative_path_string(repository_root, &fallback)
            .unwrap_or_else(|_| fallback.display().to_string())
            .into(),
        line: 0,
        message: format!(
            "expected root file `src/{project_name}.jl` was not found; using `{}` instead",
            fallback
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
        ),
        severity: "warning".to_string(),
    });
    Ok(fallback)
}
