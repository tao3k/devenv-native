//! Directory bundle helpers for artifact blob cache payloads.

use std::fs;
use std::io::Cursor;
use std::path::{Component, Path};

use crate::artifact_cache::ArtifactCacheError;

/// Encode a directory tree into a tar byte payload for an artifact blob cache.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when the directory cannot be read, contains
/// symlinks or special file types, or cannot be encoded.
pub fn pack_artifact_directory(root: &Path) -> Result<Vec<u8>, ArtifactCacheError> {
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|error| ArtifactCacheError::io("reading", root, error))?;
    if !root_metadata.is_dir() {
        return Err(ArtifactCacheError::backend(
            "directory-bundle",
            "packing",
            format!("root `{}` is not a directory", root.display()),
        ));
    }
    let mut builder = tar::Builder::new(Vec::new());
    append_directory_entries(&mut builder, root, root)?;
    builder.finish().map_err(|error| {
        ArtifactCacheError::backend("directory-bundle", "finishing tar", error.to_string())
    })?;
    builder.into_inner().map_err(|error| {
        ArtifactCacheError::backend("directory-bundle", "finalizing tar", error.to_string())
    })
}

/// Decode a directory bundle produced by [`pack_artifact_directory`].
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when the bundle cannot be decoded, contains
/// path traversal, contains unsupported entry types, or cannot be written.
pub fn unpack_artifact_directory(bytes: &[u8], target: &Path) -> Result<(), ArtifactCacheError> {
    fs::create_dir_all(target)
        .map_err(|error| ArtifactCacheError::io("creating", target, error))?;
    let mut archive = tar::Archive::new(Cursor::new(bytes));
    let entries = archive.entries().map_err(|error| {
        ArtifactCacheError::backend("directory-bundle", "reading tar entries", error.to_string())
    })?;
    for entry in entries {
        let mut entry = entry.map_err(|error| {
            ArtifactCacheError::backend("directory-bundle", "reading tar entry", error.to_string())
        })?;
        let entry_type = entry.header().entry_type();
        if !(entry_type.is_file() || entry_type.is_dir()) {
            return Err(ArtifactCacheError::backend(
                "directory-bundle",
                "validating tar entry",
                "entry type must be a regular file or directory",
            ));
        }
        let path = entry
            .path()
            .map_err(|error| {
                ArtifactCacheError::backend(
                    "directory-bundle",
                    "reading tar entry path",
                    error.to_string(),
                )
            })?
            .into_owned();
        validate_relative_bundle_path(path.as_path())?;
        let unpacked = entry.unpack_in(target).map_err(|error| {
            ArtifactCacheError::backend(
                "directory-bundle",
                "unpacking tar entry",
                error.to_string(),
            )
        })?;
        if !unpacked {
            return Err(ArtifactCacheError::backend(
                "directory-bundle",
                "unpacking tar entry",
                "entry path escaped target directory",
            ));
        }
    }
    Ok(())
}

fn append_directory_entries(
    builder: &mut tar::Builder<Vec<u8>>,
    root: &Path,
    current: &Path,
) -> Result<(), ArtifactCacheError> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| ArtifactCacheError::io("reading directory", current, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ArtifactCacheError::io("reading directory entry", current, error))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(path.as_path())
            .map_err(|error| ArtifactCacheError::io("reading metadata", path.as_path(), error))?;
        if metadata.file_type().is_symlink() {
            return Err(ArtifactCacheError::backend(
                "directory-bundle",
                "packing",
                format!("symlink `{}` is not supported", path.display()),
            ));
        }
        let relative_path = path.strip_prefix(root).map_err(|error| {
            ArtifactCacheError::backend(
                "directory-bundle",
                "computing relative path",
                error.to_string(),
            )
        })?;
        validate_relative_bundle_path(relative_path)?;
        if metadata.is_dir() {
            builder
                .append_dir(relative_path, path.as_path())
                .map_err(|error| {
                    ArtifactCacheError::backend(
                        "directory-bundle",
                        "appending directory",
                        error.to_string(),
                    )
                })?;
            append_directory_entries(builder, root, path.as_path())?;
        } else if metadata.is_file() {
            builder
                .append_path_with_name(path.as_path(), relative_path)
                .map_err(|error| {
                    ArtifactCacheError::backend(
                        "directory-bundle",
                        "appending file",
                        error.to_string(),
                    )
                })?;
        } else {
            return Err(ArtifactCacheError::backend(
                "directory-bundle",
                "packing",
                format!("special file `{}` is not supported", path.display()),
            ));
        }
    }
    Ok(())
}

fn validate_relative_bundle_path(path: &Path) -> Result<(), ArtifactCacheError> {
    if path.as_os_str().is_empty() {
        return Err(ArtifactCacheError::backend(
            "directory-bundle",
            "validating path",
            "entry path must not be empty",
        ));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(ArtifactCacheError::backend(
                    "directory-bundle",
                    "validating path",
                    format!("entry path `{}` must be relative", path.display()),
                ));
            }
        }
    }
    Ok(())
}
