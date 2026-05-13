//! Git-tracked file scope helpers.

use std::path::Path;

use gix::bstr::ByteSlice;
use gix::index::entry::{Mode, Stage};

use crate::error::{RepoError, RepoErrorKind};

/// Lists Git-tracked file paths for a checkout, relative to the repository root.
///
/// # Errors
///
/// Returns an error when the checkout cannot be opened as a Git repository or
/// when its index cannot be read.
pub fn list_tracked_file_paths(checkout_root: &Path) -> Result<Vec<String>, RepoError> {
    let repository = gix::open(checkout_root).map_err(|error| {
        repo_error_from_message(
            format!("open git checkout `{}`: {error}", checkout_root.display()),
            RepoErrorKind::InvalidPath,
        )
    })?;
    if repository.is_bare() {
        return Err(RepoError::new(
            RepoErrorKind::Unsupported,
            format!("git checkout `{}` is bare", checkout_root.display()),
        ));
    }

    let index = repository.open_index().map_err(|error| {
        repo_error_from_message(
            format!("read git index for `{}`: {error}", checkout_root.display()),
            RepoErrorKind::RepositoryCorrupt,
        )
    })?;
    let mut paths = index
        .entries_with_paths_by_filter_map(|path, entry| {
            is_file_scope_entry(entry).then(|| path.to_str().ok().map(str::to_owned))?
        })
        .map(|(_path, relative_path)| relative_path)
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn is_file_scope_entry(entry: &gix::index::Entry) -> bool {
    entry.stage() == Stage::Unconflicted
        && matches!(
            entry.mode,
            mode if mode == Mode::FILE || mode == Mode::FILE_EXECUTABLE || mode == Mode::SYMLINK
        )
}

fn repo_error_from_message(message: String, fallback: RepoErrorKind) -> RepoError {
    let kind = match RepoError::classify_message(message.as_str()) {
        RepoErrorKind::Permanent => fallback,
        kind => kind,
    };
    RepoError::new(kind, message)
}
