use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use tokio::task::JoinHandle;
use walkdir::WalkDir;
use xiuxian_git_repo::diff_checkout_revisions;

use crate::analyzers::RepoIntelligenceError;
use crate::repo_index::types::RepoCodeDocument;

use super::language::{infer_code_language, is_excluded_code_path, is_supported_code_path};

pub(super) async fn await_analysis_completion(
    repo_id: &str,
    task: JoinHandle<Result<crate::analyzers::RepositoryAnalysisOutput, RepoIntelligenceError>>,
    timeout: Duration,
) -> Result<crate::analyzers::RepositoryAnalysisOutput, RepoIntelligenceError> {
    match tokio::time::timeout(timeout, task).await {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => Err(RepoIntelligenceError::AnalysisFailed {
            message: format!("repo `{repo_id}` indexing worker terminated unexpectedly: {error}"),
        }),
        Err(_) => Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{repo_id}` indexing timed out after {}s while analysis was running",
                timeout.as_secs()
            ),
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IncrementalCodeDocumentCollection {
    pub(crate) changed_documents: Vec<RepoCodeDocument>,
    pub(crate) deleted_paths: BTreeSet<String>,
}

pub(crate) fn collect_code_documents(
    root: &Path,
    mut is_cancelled: impl FnMut() -> bool,
) -> Option<Vec<RepoCodeDocument>> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .try_fold(Vec::new(), |mut documents, entry| {
            if is_cancelled() {
                return None;
            }
            if !entry.file_type().is_file() {
                return Some(documents);
            }
            let relative_path = entry.path().strip_prefix(root).ok().map_or_else(
                || entry.path().to_string_lossy().replace('\\', "/"),
                |path| path.to_string_lossy().replace('\\', "/"),
            );
            if is_excluded_code_path(relative_path.as_str())
                || !is_supported_code_path(relative_path.as_str())
            {
                return Some(documents);
            }
            let Some(document) = collect_code_document(entry.path(), relative_path) else {
                return Some(documents);
            };
            documents.push(document);
            Some(documents)
        })
}

pub(crate) fn collect_incremental_code_documents(
    root: &Path,
    changed_paths: &BTreeSet<String>,
    deleted_paths: &BTreeSet<String>,
    mut is_cancelled: impl FnMut() -> bool,
) -> Option<IncrementalCodeDocumentCollection> {
    let (changed_documents, current_deleted_paths) = changed_paths.iter().try_fold(
        (Vec::new(), deleted_paths.clone()),
        |(mut changed_documents, mut current_deleted_paths), relative_path| {
            if is_cancelled() {
                return None;
            }
            if is_excluded_code_path(relative_path.as_str())
                || !is_supported_code_path(relative_path.as_str())
            {
                current_deleted_paths.insert(relative_path.clone());
                return Some((changed_documents, current_deleted_paths));
            }
            let path = root.join(relative_path);
            let Some(document) = collect_code_document(path.as_path(), relative_path.clone())
            else {
                current_deleted_paths.insert(relative_path.clone());
                return Some((changed_documents, current_deleted_paths));
            };
            changed_documents.push(document);
            Some((changed_documents, current_deleted_paths))
        },
    )?;

    Some(IncrementalCodeDocumentCollection {
        changed_documents,
        deleted_paths: current_deleted_paths,
    })
}

pub(crate) fn collect_incremental_code_documents_from_revision_diff(
    root: &Path,
    previous_revision: &str,
    current_revision: &str,
    is_cancelled: impl FnMut() -> bool,
) -> Result<Option<IncrementalCodeDocumentCollection>, RepoIntelligenceError> {
    let diff = diff_checkout_revisions(root, previous_revision, current_revision).map_err(|error| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "failed to diff checkout revisions `{previous_revision}` -> `{current_revision}` for incremental code-document collection: {error}"
            ),
        }
    })?;
    Ok(collect_incremental_code_documents(
        root,
        &diff.changed_paths(),
        &diff.deleted_paths(),
        is_cancelled,
    ))
}

fn collect_code_document(path: &Path, relative_path: String) -> Option<RepoCodeDocument> {
    let Ok(metadata) = std::fs::metadata(path) else {
        return None;
    };
    if !metadata.is_file() {
        return None;
    }
    let Ok(contents) = std::fs::read_to_string(path) else {
        return None;
    };
    Some(RepoCodeDocument {
        language: infer_code_language(relative_path.as_str()),
        path: relative_path,
        contents: Arc::<str>::from(contents),
        size_bytes: metadata.len(),
        modified_unix_ms: metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |duration| {
                u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
            }),
    })
}
