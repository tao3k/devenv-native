use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use super::classify::{FingerprintMode, analysis_fingerprint_mode};
#[cfg(feature = "search-runtime")]
use super::semantic::{
    SemanticFingerprintOwner, compute_semantic_fingerprint, semantic_fingerprint_owner,
};
use crate::analyzers::RegisteredRepository;

pub(crate) fn collect_repository_analysis_identity(
    repository: &RegisteredRepository,
    repository_root: &Path,
    plugin_ids: &[String],
) -> Option<String> {
    #[cfg(not(feature = "search-runtime"))]
    let _ = repository;
    if !repository_root.is_dir() {
        return None;
    }

    let mut relevant_files = WalkDir::new(repository_root)
        .into_iter()
        .filter_entry(|entry| !should_skip_walk_entry(entry))
        .filter_map(Result::ok)
        .filter_map(|entry| relevant_file(repository_root, entry, plugin_ids))
        .collect::<Vec<_>>();

    relevant_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xiuxian_wendao.repo_analysis_identity.v1\0");

    if relevant_files.is_empty() {
        hasher.update(b"empty\0");
        return Some(hasher.finalize().to_hex().to_string());
    }

    for file in relevant_files {
        hasher.update(file.relative_path.as_bytes());
        hasher.update(b"\0");
        let mode_label = relevant_file_identity_mode_label(&file.mode);
        hasher.update(mode_label.as_bytes());
        hasher.update(b"\0");
        match file.mode {
            RelevantFileIdentityMode::PathOnly => {}
            RelevantFileIdentityMode::Contents => {
                let contents = fs::read(&file.absolute_path).ok()?;
                hasher.update(contents.as_slice());
                hasher.update(b"\0");
            }
            #[cfg(feature = "search-runtime")]
            RelevantFileIdentityMode::SemanticFingerprint(owner) => {
                hash_semantic_identity(
                    repository,
                    &file.absolute_path,
                    file.relative_path.as_str(),
                    &owner,
                    &mut hasher,
                )?;
            }
        }
    }

    Some(hasher.finalize().to_hex().to_string())
}

#[derive(Debug)]
struct RelevantFile {
    relative_path: String,
    absolute_path: PathBuf,
    mode: RelevantFileIdentityMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RelevantFileIdentityMode {
    PathOnly,
    Contents,
    #[cfg(feature = "search-runtime")]
    SemanticFingerprint(SemanticFingerprintOwner),
}

fn relevant_file(
    repository_root: &Path,
    entry: DirEntry,
    plugin_ids: &[String],
) -> Option<RelevantFile> {
    if !entry.file_type().is_file() {
        return None;
    }

    let relative_path = entry
        .path()
        .strip_prefix(repository_root)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    let mode = relevant_file_identity_mode(relative_path.as_str(), plugin_ids)?;

    Some(RelevantFile {
        relative_path,
        absolute_path: entry.into_path(),
        mode,
    })
}

fn should_skip_walk_entry(entry: &DirEntry) -> bool {
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| name == ".git")
}

fn relevant_file_identity_mode(
    relative_path: &str,
    plugin_ids: &[String],
) -> Option<RelevantFileIdentityMode> {
    let mode = analysis_fingerprint_mode(relative_path, plugin_ids)?;
    #[cfg(feature = "search-runtime")]
    if matches!(mode, FingerprintMode::Contents)
        && let Some(owner) = semantic_fingerprint_owner(relative_path, plugin_ids)
    {
        return Some(RelevantFileIdentityMode::SemanticFingerprint(owner));
    }

    Some(match mode {
        FingerprintMode::PathOnly => RelevantFileIdentityMode::PathOnly,
        FingerprintMode::Contents => RelevantFileIdentityMode::Contents,
    })
}

fn relevant_file_identity_mode_label(mode: &RelevantFileIdentityMode) -> String {
    match mode {
        RelevantFileIdentityMode::PathOnly => "path".to_string(),
        RelevantFileIdentityMode::Contents => "contents".to_string(),
        #[cfg(feature = "search-runtime")]
        RelevantFileIdentityMode::SemanticFingerprint(owner) => owner.mode_label(),
    }
}

#[cfg(feature = "search-runtime")]
fn hash_semantic_identity(
    repository: &RegisteredRepository,
    absolute_path: &Path,
    relative_path: &str,
    owner: &SemanticFingerprintOwner,
    hasher: &mut blake3::Hasher,
) -> Option<()> {
    let contents = fs::read(absolute_path).ok()?;
    let semantic_fingerprint =
        std::str::from_utf8(contents.as_slice())
            .ok()
            .and_then(|source_text| {
                compute_semantic_fingerprint(owner, repository, relative_path, source_text)
            });
    match semantic_fingerprint {
        Some(fingerprint) => hasher.update(fingerprint.as_bytes()),
        None => hasher.update(contents.as_slice()),
    };
    hasher.update(b"\0");
    Some(())
}
