//! Revision diff summaries and blob reads for local checkouts.

use std::collections::BTreeSet;
use std::path::Path;

use crate::backend::{RepositoryHandle, open_checkout_with_retry};
use crate::error::{RepoError, RepoErrorKind};

/// Classification for one revision diff path change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionChangeKind {
    /// A new path was added in the target revision.
    Added,
    /// A path was removed in the target revision.
    Deleted,
    /// A path changed while remaining the same kind.
    Modified,
    /// A path changed type, such as file-to-symlink.
    TypeChanged,
    /// A path was renamed from one location to another.
    Renamed,
    /// A path was copied to a new location.
    Copied,
}

/// One detached path change between two revisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionPathChange {
    /// High-level change classification.
    pub kind: RevisionChangeKind,
    /// Source path for deletes, renames, and copies.
    pub previous_path: Option<String>,
    /// Destination path for adds, modifies, renames, and copies.
    pub path: String,
}

/// Summary of repository-relative path changes between two revisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionDiffSummary {
    /// Base revision used as the diff source.
    pub previous_revision: String,
    /// Target revision used as the diff destination.
    pub revision: String,
    /// Detached path changes required to transform `previous_revision` into `revision`.
    pub changes: Vec<RevisionPathChange>,
}

impl RevisionDiffSummary {
    /// Returns true when the two revisions do not differ in tracked tree contents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Collect the repository-relative paths that changed in the target revision.
    #[must_use]
    pub fn changed_paths(&self) -> BTreeSet<String> {
        self.changes
            .iter()
            .filter_map(|change| match change.kind {
                RevisionChangeKind::Deleted => None,
                RevisionChangeKind::Added
                | RevisionChangeKind::Modified
                | RevisionChangeKind::TypeChanged
                | RevisionChangeKind::Renamed
                | RevisionChangeKind::Copied => Some(change.path.clone()),
            })
            .collect()
    }

    /// Collect repository-relative paths that disappeared from the target revision.
    #[must_use]
    pub fn deleted_paths(&self) -> BTreeSet<String> {
        self.changes
            .iter()
            .filter_map(|change| match change.kind {
                RevisionChangeKind::Deleted | RevisionChangeKind::Renamed => change
                    .previous_path
                    .clone()
                    .or_else(|| Some(change.path.clone())),
                RevisionChangeKind::Added
                | RevisionChangeKind::Modified
                | RevisionChangeKind::TypeChanged
                | RevisionChangeKind::Copied => None,
            })
            .collect()
    }
}

/// Diff two revisions in a local checkout using the crate-owned repository substrate.
///
/// # Errors
///
/// Returns [`RepoError`] when the checkout cannot be opened or either revision
/// cannot be resolved to a tree.
pub fn diff_checkout_revisions(
    checkout_root: &Path,
    previous_revision: &str,
    revision: &str,
) -> Result<RevisionDiffSummary, RepoError> {
    let repository = open_checkout_with_retry(checkout_root).map_err(|error| {
        RepoError::new(
            RepoErrorKind::InvalidPath,
            format!(
                "failed to open checkout `{}` for revision diff: {error}",
                checkout_root.display()
            ),
        )
    })?;
    diff_repository_revisions(&repository, previous_revision, revision)
}

/// Read one repository-relative file blob from a target revision in a local
/// checkout.
///
/// Returns `Ok(None)` when the path does not exist at the requested revision or
/// does not resolve to a blob.
///
/// # Errors
///
/// Returns [`RepoError`] when the checkout cannot be opened or the revision
/// cannot be resolved.
pub fn read_checkout_file_bytes_at_revision(
    checkout_root: &Path,
    revision: &str,
    relative_path: &str,
) -> Result<Option<Vec<u8>>, RepoError> {
    let repository = open_checkout_with_retry(checkout_root).map_err(|error| {
        RepoError::new(
            RepoErrorKind::InvalidPath,
            format!(
                "failed to open checkout `{}` for revision file read: {error}",
                checkout_root.display()
            ),
        )
    })?;
    read_repository_file_bytes_at_revision(&repository, revision, relative_path)
}

fn diff_repository_revisions(
    repository: &RepositoryHandle,
    previous_revision: &str,
    revision: &str,
) -> Result<RevisionDiffSummary, RepoError> {
    let previous_tree = resolve_revision_tree(repository, previous_revision)?;
    let target_tree = resolve_revision_tree(repository, revision)?;
    let changes = repository
        .diff_tree_to_tree(Some(&previous_tree), Some(&target_tree), None)
        .map_err(|error| {
            RepoError::new(
                RepoError::classify_message(error.to_string().as_str()),
                format!(
                    "failed to diff revision `{previous_revision}` against `{revision}`: {error}"
                ),
            )
        })?;

    Ok(RevisionDiffSummary {
        previous_revision: previous_revision.to_string(),
        revision: revision.to_string(),
        changes: changes.into_iter().map(map_change).collect(),
    })
}

fn read_repository_file_bytes_at_revision(
    repository: &RepositoryHandle,
    revision: &str,
    relative_path: &str,
) -> Result<Option<Vec<u8>>, RepoError> {
    let tree = resolve_revision_tree(repository, revision)?;
    let Some(entry) = tree.lookup_entry_by_path(relative_path).map_err(|error| {
        RepoError::new(
            RepoError::classify_message(error.to_string().as_str()),
            format!("failed to look up `{relative_path}` in revision `{revision}`: {error}"),
        )
    })?
    else {
        return Ok(None);
    };
    let object = entry.object().map_err(|error| {
        RepoError::new(
            RepoError::classify_message(error.to_string().as_str()),
            format!("failed to load `{relative_path}` object in revision `{revision}`: {error}"),
        )
    })?;
    let blob = object.try_into_blob().map_err(|error| {
        RepoError::new(
            RepoErrorKind::RepositoryCorrupt,
            format!(
                "expected `{relative_path}` at revision `{revision}` to resolve to a blob: {error}"
            ),
        )
    })?;
    Ok(Some(blob.data.clone()))
}

fn resolve_revision_tree<'repo>(
    repository: &'repo RepositoryHandle,
    revision: &str,
) -> Result<gix::Tree<'repo>, RepoError> {
    let normalized = revision.trim();
    if normalized.is_empty() {
        return Err(RepoError::new(
            RepoErrorKind::RevisionNotFound,
            "revision diff requires a non-empty revision",
        ));
    }

    let object = repository.rev_parse_single(normalized).map_err(|error| {
        RepoError::new(
            RepoErrorKind::RevisionNotFound,
            format!("failed to resolve revision `{normalized}`: {error}"),
        )
    })?;
    object
        .object()
        .map_err(|error| {
            RepoError::new(
                RepoErrorKind::RevisionNotFound,
                format!("failed to load revision `{normalized}` object: {error}"),
            )
        })?
        .peel_to_tree()
        .map_err(|error| {
            RepoError::new(
                RepoErrorKind::RevisionNotFound,
                format!("failed to peel revision `{normalized}` to a tree: {error}"),
            )
        })
}

fn map_change(change: gix::object::tree::diff::ChangeDetached) -> RevisionPathChange {
    match change {
        gix::object::tree::diff::ChangeDetached::Addition { location, .. } => RevisionPathChange {
            kind: RevisionChangeKind::Added,
            previous_path: None,
            path: location_to_string(&location),
        },
        gix::object::tree::diff::ChangeDetached::Deletion { location, .. } => RevisionPathChange {
            kind: RevisionChangeKind::Deleted,
            previous_path: Some(location_to_string(&location)),
            path: location_to_string(&location),
        },
        gix::object::tree::diff::ChangeDetached::Modification {
            location,
            previous_entry_mode,
            entry_mode,
            ..
        } => RevisionPathChange {
            kind: if previous_entry_mode == entry_mode {
                RevisionChangeKind::Modified
            } else {
                RevisionChangeKind::TypeChanged
            },
            previous_path: None,
            path: location_to_string(&location),
        },
        gix::object::tree::diff::ChangeDetached::Rewrite {
            source_location,
            location,
            copy,
            ..
        } => RevisionPathChange {
            kind: if copy {
                RevisionChangeKind::Copied
            } else {
                RevisionChangeKind::Renamed
            },
            previous_path: Some(location_to_string(&source_location)),
            path: location_to_string(&location),
        },
    }
}

fn location_to_string(location: &gix::bstr::BString) -> String {
    String::from_utf8_lossy(location.as_ref()).into_owned()
}
