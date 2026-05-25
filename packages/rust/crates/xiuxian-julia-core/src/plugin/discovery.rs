use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use xiuxian_wendao_core::repo_intelligence::{DocRecord, ExampleRecord, RepoIntelligenceError};

pub(crate) fn discover_examples(
    repo_id: &str,
    repository_root: &Path,
) -> Result<Vec<ExampleRecord>, RepoIntelligenceError> {
    let mut records = Vec::new();
    for scope in ["examples", "test"] {
        let scope_root = repository_root.join(scope);
        if !scope_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&scope_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file()
                || entry
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "jl")
            {
                continue;
            }

            let relative = relative_path_string(repository_root, entry.path())?;
            let title = entry
                .path()
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("example")
                .to_string();
            records.push(ExampleRecord {
                repo_id: (repo_id.to_string()).into(),
                example_id: (format!("repo:{repo_id}:example:{relative}")).into(),
                title,
                path: (relative).into(),
                summary: None,
            });
        }
    }
    Ok(records)
}

pub(crate) fn discover_docs(
    repo_id: &str,
    repository_root: &Path,
) -> Result<Vec<DocRecord>, RepoIntelligenceError> {
    let mut records = Vec::new();

    for entry in
        fs::read_dir(repository_root).map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to read `{}`: {error}", repository_root.display()),
        })?
    {
        let entry = entry.map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "failed to enumerate `{}`: {error}",
                repository_root.display()
            ),
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.to_ascii_uppercase().starts_with("README") {
            continue;
        }
        let relative = relative_path_string(repository_root, &path)?;
        records.push(DocRecord {
            repo_id: (repo_id.to_string()).into(),
            doc_id: (format!("repo:{repo_id}:doc:{relative}")).into(),
            title: name.to_string(),
            path: (relative).into(),
            format: path
                .extension()
                .and_then(|value| value.to_str())
                .map(str::to_string),
            doc_target: None,
        });
    }

    let docs_root = repository_root.join("docs");
    if docs_root.exists() {
        for entry in WalkDir::new(&docs_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file()
                || entry
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "md")
            {
                continue;
            }
            let relative = relative_path_string(repository_root, entry.path())?;
            let title = entry
                .path()
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("doc")
                .to_string();
            records.push(DocRecord {
                repo_id: (repo_id.to_string()).into(),
                doc_id: (format!("repo:{repo_id}:doc:{relative}")).into(),
                title,
                path: (relative).into(),
                format: Some("md".to_string()),
                doc_target: None,
            });
        }
    }

    Ok(records)
}

pub(crate) fn relative_path_string(
    root: &Path,
    path: &Path,
) -> Result<String, RepoIntelligenceError> {
    let relative =
        path.strip_prefix(root)
            .map_err(|error| RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "failed to compute relative path for `{}` against `{}`: {error}",
                    path.display(),
                    root.display()
                ),
            })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
