use std::path::Path;

use xiuxian_git_repo::{RepoError, RepoErrorKind, RepoRefreshPolicy, RepoSpec, RevisionSelector};

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::{RegisteredRepository, RepositoryRef, RepositoryRefreshPolicy};

/// Resolve one registered repository through the shared repo substrate.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the registered repository cannot be
/// mapped to a valid repo source or the substrate fails to prepare it.
pub(crate) fn resolve_registered_repository_source(
    repository: &RegisteredRepository,
    cwd: &Path,
    mode: xiuxian_git_repo::SyncMode,
) -> Result<xiuxian_git_repo::MaterializedRepo, RepoIntelligenceError> {
    let spec = repo_spec_from_registered(repository);
    xiuxian_git_repo::resolve_repository_source(&spec, cwd, mode)
        .map_err(|error| map_repo_error(repository, error))
}

fn repo_spec_from_registered(repository: &RegisteredRepository) -> RepoSpec {
    RepoSpec {
        id: repository.id.clone(),
        local_path: repository.path.clone(),
        remote_url: repository.url.clone(),
        revision: repository.git_ref.as_ref().map(repo_revision_selector),
        refresh: repo_refresh_policy(repository.refresh),
    }
}

fn repo_revision_selector(revision: &RepositoryRef) -> RevisionSelector {
    match revision {
        RepositoryRef::Branch(value) => RevisionSelector::Branch(value.clone()),
        RepositoryRef::Tag(value) => RevisionSelector::Tag(value.clone()),
        RepositoryRef::Commit(value) => RevisionSelector::Commit(value.clone()),
    }
}

fn repo_refresh_policy(policy: RepositoryRefreshPolicy) -> RepoRefreshPolicy {
    match policy {
        RepositoryRefreshPolicy::Fetch => RepoRefreshPolicy::Fetch,
        RepositoryRefreshPolicy::Manual => RepoRefreshPolicy::Manual,
    }
}

fn map_repo_error(repository: &RegisteredRepository, error: RepoError) -> RepoIntelligenceError {
    match error.kind {
        RepoErrorKind::MissingSource => RepoIntelligenceError::MissingRepositorySource {
            repo_id: repository.id.clone(),
        },
        RepoErrorKind::InvalidPath => RepoIntelligenceError::InvalidRepositoryPath {
            repo_id: repository.id.clone(),
            path: repository
                .path
                .as_ref()
                .map_or_else(String::new, |path| path.display().to_string()),
            reason: error.message,
        },
        RepoErrorKind::Unsupported => RepoIntelligenceError::UnsupportedRepositoryLayout {
            repo_id: repository.id.clone(),
            message: error.message,
        },
        _ => RepoIntelligenceError::AnalysisFailed {
            message: error.message,
        },
    }
}
