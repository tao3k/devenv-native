use std::fs;
use std::path::{Path, PathBuf};

use crate::studio::StudioState;
use crate::studio::types::{VfsContentResponse, VfsEntry};

use super::filters::VfsError;
use super::roots::resolve_all_vfs_roots;

pub(crate) struct RawVfsContent {
    pub(crate) content: Vec<u8>,
    pub(crate) content_type: String,
}

pub(crate) fn get_entry(state: &StudioState, path: &str) -> Result<VfsEntry, VfsError> {
    let resolved = resolve_vfs_path(state, path)?;
    let metadata = fs::metadata(&resolved.full_path)
        .map_err(|error| VfsError::internal("IO_ERROR", error.to_string(), None))?;

    Ok(VfsEntry {
        path: path.to_string(),
        name: resolved
            .full_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified: unix_timestamp_secs(&metadata),
        content_type: None,
        project_name: None,
        root_label: None,
        project_root: None,
        project_dirs: None,
    })
}

#[allow(clippy::unused_async)]
pub(crate) async fn read_content(
    state: &StudioState,
    path: &str,
) -> Result<VfsContentResponse, VfsError> {
    let resolved = resolve_vfs_path(state, path)?;
    let content = fs::read_to_string(&resolved.full_path)
        .map_err(|error| VfsError::internal("IO_ERROR", error.to_string(), None))?;
    let metadata = fs::metadata(&resolved.full_path)
        .map_err(|error| VfsError::internal("IO_ERROR", error.to_string(), None))?;

    Ok(VfsContentResponse {
        path: path.to_string(),
        content_type: "text/plain".to_string(),
        content,
        modified: unix_timestamp_secs(&metadata),
    })
}

#[allow(clippy::unused_async)]
pub(crate) async fn read_raw_content(
    state: &StudioState,
    path: &str,
) -> Result<RawVfsContent, VfsError> {
    let resolved = resolve_vfs_path(state, path)?;
    let content = fs::read(&resolved.full_path)
        .map_err(|error| VfsError::internal("IO_ERROR", error.to_string(), None))?;

    Ok(RawVfsContent {
        content,
        content_type: infer_vfs_content_type(path).to_string(),
    })
}

pub(super) struct ResolvedVfsPath {
    pub(super) full_path: PathBuf,
}

pub(crate) fn resolve_vfs_file_path(state: &StudioState, path: &str) -> Result<PathBuf, VfsError> {
    resolve_vfs_path(state, path).map(|resolved| resolved.full_path)
}

pub(super) fn resolve_vfs_path(
    state: &StudioState,
    path: &str,
) -> Result<ResolvedVfsPath, VfsError> {
    let path = path.trim();
    if let Some(resolved) = resolve_vfs_path_from_roots(state, path) {
        return Ok(resolved);
    }

    Err(VfsError::not_found(format!("VFS path not found: {path}")))
}

fn resolve_vfs_path_from_roots(state: &StudioState, path: &str) -> Option<ResolvedVfsPath> {
    for root in resolve_all_vfs_roots(state) {
        if path == root.request_root {
            return Some(ResolvedVfsPath {
                full_path: root.full_path,
            });
        }
        let prefix = format!("{}/", root.request_root);
        if path.starts_with(&prefix) {
            let rel = &path[prefix.len()..];
            return Some(ResolvedVfsPath {
                full_path: root.full_path.join(rel),
            });
        }
    }
    None
}

pub(super) fn unix_timestamp_secs(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_secs())
}

fn infer_vfs_content_type(path: &str) -> &'static str {
    match file_extension(path).as_deref() {
        Some("aac") => "audio/aac",
        Some("apng") => "image/apng",
        Some("avi") => "video/x-msvideo",
        Some("avif") => "image/avif",
        Some("bmp") => "image/bmp",
        Some("flac") => "audio/flac",
        Some("gif") => "image/gif",
        Some("heic") => "image/heic",
        Some("heif") => "image/heif",
        Some("ico") => "image/x-icon",
        Some("jpeg" | "jpg") => "image/jpeg",
        Some("m4a") => "audio/mp4",
        Some("m4v" | "mp4") => "video/mp4",
        Some("markdown" | "md") => "text/markdown",
        Some("mkv") => "video/x-matroska",
        Some("mov") => "video/quicktime",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("opus") => "audio/opus",
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("tif" | "tiff") => "image/tiff",
        Some("wav") => "audio/wav",
        Some("webm") => "video/webm",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

fn file_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/vfs/content.rs"]
mod tests;
