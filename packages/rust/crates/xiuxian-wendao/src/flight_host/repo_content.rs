use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use walkdir::WalkDir;
use xiuxian_git_repo::{SyncMode, discover_checkout_metadata, list_tracked_file_paths};

use crate::analyzers::{
    DocRecord, RegisteredRepository, RepositoryAnalysisOutput, RepositoryRecord,
    build_repository_analysis_cache_key, load_repo_intelligence_config,
    resolve_registered_repository_source, store_cached_repository_analysis,
};
use crate::repo_index::RepoCodeDocument;
use crate::search::SearchPlaneService;

pub(super) const BOOTSTRAP_CONFIGURED_REPO_CONTENT_ENV: &str =
    "WENDAO_BOOTSTRAP_CONFIGURED_REPO_CONTENT";

pub(super) async fn maybe_bootstrap_configured_repo_content(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    project_root: &Path,
    config_path: Option<&Path>,
) -> Result<Option<RepositoryAnalysisOutput>> {
    if std::env::var_os(BOOTSTRAP_CONFIGURED_REPO_CONTENT_ENV).is_none() {
        return Ok(None);
    }

    let repository = configured_bootstrap_repository(repo_id, project_root, config_path)?;
    let checkout_root = repository
        .path
        .as_deref()
        .ok_or_else(|| anyhow!("configured repo-content bootstrap repo `{repo_id}` has no path"))?;
    let documents = collect_configured_repo_content_documents(checkout_root)?;
    if documents.is_empty() {
        return Err(anyhow!(
            "configured repo-content bootstrap found no supported documents in `{}`",
            checkout_root.display()
        ));
    }

    publish_configured_repo_content_documents(search_plane, repo_id, documents.as_slice()).await?;
    println!(
        "BOOTSTRAPPED_REPO_CONTENT {repo_id} documents={}",
        documents.len()
    );
    let analysis = prime_configured_repo_content_analysis_cache(
        &repository,
        project_root,
        config_path,
        documents.as_slice(),
    )?;
    Ok(Some(analysis))
}

async fn publish_configured_repo_content_documents(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    documents: &[RepoCodeDocument],
) -> Result<()> {
    let revision = Some("configured-repo-content-bootstrap");
    let deleted_paths = BTreeSet::new();
    match search_plane
        .publish_repo_content_chunks_incremental_with_revision(
            repo_id,
            documents,
            &deleted_paths,
            revision,
        )
        .await
    {
        Ok(()) => Ok(()),
        Err(incremental_error) => search_plane
            .publish_repo_content_chunks_with_revision(repo_id, documents, revision)
            .await
            .map_err(|replace_error| {
                anyhow!(
                    "publish configured repo-content bootstrap failed; incremental error: {incremental_error}; replace-all error: {replace_error}"
                )
            }),
    }
}

pub(super) fn configured_repo_root(
    repo_id: &str,
    project_root: &Path,
    config_path: Option<&Path>,
) -> Result<PathBuf> {
    let repository = configured_bootstrap_repository(repo_id, project_root, config_path)?;
    repository
        .path
        .ok_or_else(|| anyhow!("configured repo `{repo_id}` has no path"))
}

fn configured_bootstrap_repository(
    repo_id: &str,
    project_root: &Path,
    config_path: Option<&Path>,
) -> Result<RegisteredRepository> {
    if let Some(config_path) = config_path {
        let repo_config = load_repo_intelligence_config(Some(config_path), project_root)
            .map_err(|error| anyhow!("load configured repo-content bootstrap config: {error}"))?;
        if let Some(repository) = repo_config
            .repos
            .iter()
            .find(|repository| repository.id == repo_id)
        {
            return Ok(repository.clone());
        }
    }

    Ok(RegisteredRepository {
        id: repo_id.to_owned(),
        path: Some(project_root.to_path_buf()),
        ..RegisteredRepository::default()
    })
}

fn collect_configured_repo_content_documents(repo_root: &Path) -> Result<Vec<RepoCodeDocument>> {
    if let Some(documents) = collect_git_tracked_configured_repo_content_documents(repo_root)? {
        return Ok(documents);
    }

    let mut documents = Vec::new();
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|entry| !is_ignored_repo_content_path(entry.path()))
    {
        let entry = entry.map_err(|error| anyhow!("walk configured repo content: {error}"))?;
        let path = entry.path();
        if !path.is_file() || !is_supported_repo_content_path(path) {
            continue;
        }
        let relative_path = path
            .strip_prefix(repo_root)
            .map_err(|error| anyhow!("strip configured repo content path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        documents.push(repo_content_document(relative_path.as_str(), path)?);
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn collect_git_tracked_configured_repo_content_documents(
    repo_root: &Path,
) -> Result<Option<Vec<RepoCodeDocument>>> {
    let Ok(tracked_paths) = list_tracked_file_paths(repo_root) else {
        return Ok(None);
    };

    let mut documents = Vec::new();
    for relative_path in tracked_paths {
        let path = repo_root.join(relative_path.as_str());
        if !path.is_file() || !is_supported_repo_content_path(path.as_path()) {
            continue;
        }
        documents.push(repo_content_document(
            relative_path.as_str(),
            path.as_path(),
        )?);
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Some(documents))
}

fn repo_content_document(relative_path: &str, path: &Path) -> Result<RepoCodeDocument> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| anyhow!("read configured repo content `{}`: {error}", path.display()))?;
    let metadata = path
        .metadata()
        .map_err(|error| anyhow!("read configured repo content metadata: {error}"))?;
    Ok(RepoCodeDocument {
        path: relative_path.replace('\\', "/"),
        language: language_for_repo_content_path(path).map(str::to_owned),
        contents: Arc::<str>::from(contents),
        size_bytes: metadata.len(),
        modified_unix_ms: metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |duration| {
                u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
            }),
    })
}

fn is_ignored_repo_content_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".cache" | ".devenv" | ".git" | ".run" | "target" | "node_modules"
            )
        })
}

fn is_supported_repo_content_path(path: &Path) -> bool {
    language_for_repo_content_path(path).is_some()
}

fn language_for_repo_content_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("jl") => Some("julia"),
        Some("md") => Some("markdown"),
        Some("py") => Some("python"),
        Some("rs") => Some("rust"),
        Some("toml") => Some("toml"),
        _ => None,
    }
}

fn prime_configured_repo_content_analysis_cache(
    repository: &RegisteredRepository,
    project_root: &Path,
    config_path: Option<&Path>,
    documents: &[RepoCodeDocument],
) -> Result<RepositoryAnalysisOutput> {
    let repository_source =
        resolve_registered_repository_source(repository, project_root, SyncMode::Status)
            .map_err(|error| anyhow!("resolve configured repo-content analysis source: {error}"))?;
    let checkout_metadata = discover_checkout_metadata(repository_source.checkout_root.as_path());
    let cache_key = build_repository_analysis_cache_key(
        repository,
        &repository_source,
        checkout_metadata.as_ref(),
    );
    let analysis = configured_repo_content_analysis(
        repository,
        repository_source.checkout_root.as_path(),
        checkout_metadata
            .as_ref()
            .and_then(|metadata| metadata.revision.clone()),
        documents,
    );
    let _ = config_path;
    store_cached_repository_analysis(cache_key, &analysis)
        .map_err(|error| anyhow!("store configured repo-content analysis cache: {error}"))?;
    Ok(analysis)
}

fn configured_repo_content_analysis(
    repository: &RegisteredRepository,
    repo_root: &Path,
    revision: Option<String>,
    documents: &[RepoCodeDocument],
) -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        repository: Some(RepositoryRecord {
            repo_id: repository.id.clone().into(),
            name: repository.id.clone(),
            path: repo_root.display().to_string().into(),
            url: repository.url.clone(),
            revision,
            version: None,
            uuid: None,
            dependencies: Vec::new(),
        }),
        docs: documents
            .iter()
            .map(|document| configured_repo_content_doc_record(repository.id.as_str(), document))
            .collect(),
        ..RepositoryAnalysisOutput::default()
    }
}

fn configured_repo_content_doc_record(repo_id: &str, document: &RepoCodeDocument) -> DocRecord {
    DocRecord {
        repo_id: repo_id.to_owned().into(),
        doc_id: format!("repo:{repo_id}:doc:{}", document.path).into(),
        title: configured_repo_content_doc_title(document.path.as_str()),
        path: document.path.clone().into(),
        format: Some(configured_repo_content_doc_format(document).to_owned()),
        doc_target: None,
    }
}

fn configured_repo_content_doc_title(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(path)
        .to_owned()
}

fn configured_repo_content_doc_format(document: &RepoCodeDocument) -> &'static str {
    if Path::new(document.path.as_str())
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
    {
        "md"
    } else {
        "reference"
    }
}
