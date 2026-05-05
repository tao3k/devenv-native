//! Managed repository cache path derivation.

use std::path::PathBuf;

use url::Url;
use xiuxian_io::PrjDirs;

use crate::spec::RepoSpec;

fn substrate_root() -> PathBuf {
    // Keep the existing managed repository cache root stable during the crate
    // extraction slice so on-disk paths do not drift while ownership moves.
    PrjDirs::data_home()
        .join("xiuxian-wendao")
        .join("repo-intelligence")
}

/// Returns the managed checkout root for a repository.
#[must_use]
pub fn managed_checkout_root_for(spec: &RepoSpec) -> PathBuf {
    let mut root = substrate_root().join("repos");
    root.push(managed_repo_namespace(spec));
    root
}

/// Returns the managed mirror root for a repository.
#[must_use]
pub fn managed_mirror_root_for(spec: &RepoSpec) -> PathBuf {
    let mut root = substrate_root().join("mirrors");
    root.push(managed_repo_namespace(spec));

    let leaf = root
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| sanitize_repo_id(spec.id.as_str()));
    root.set_file_name(format!("{leaf}.git"));
    root
}

fn managed_repo_namespace(spec: &RepoSpec) -> PathBuf {
    spec.remote_url
        .as_deref()
        .and_then(repo_namespace_from_remote_url)
        .unwrap_or_else(|| PathBuf::from(sanitize_repo_id(spec.id.as_str())))
}

fn repo_namespace_from_remote_url(remote_url: &str) -> Option<PathBuf> {
    remote_namespace_segments(remote_url).map(|segments| {
        let mut namespace = PathBuf::new();
        for segment in segments {
            namespace.push(segment);
        }
        namespace
    })
}

fn remote_namespace_segments(remote_url: &str) -> Option<Vec<String>> {
    if let Ok(parsed) = Url::parse(remote_url) {
        let host = parsed.host_str()?.trim();
        if host.is_empty() {
            return None;
        }

        let mut segments = vec![sanitize_namespace_segment(host)];
        segments.extend(
            parsed
                .path_segments()?
                .filter(|segment| !segment.trim().is_empty())
                .map(sanitize_namespace_segment),
        );
        trim_git_suffix(&mut segments);
        return (!segments.is_empty()).then_some(segments);
    }

    let (remote, path) = remote_url.rsplit_once(':')?;
    if remote.contains('/') {
        return None;
    }

    let host = remote
        .rsplit_once('@')
        .map_or(remote, |(_, host)| host)
        .trim();
    if host.is_empty() {
        return None;
    }

    let mut segments = vec![sanitize_namespace_segment(host)];
    segments.extend(
        path.split('/')
            .filter(|segment| !segment.trim().is_empty())
            .map(sanitize_namespace_segment),
    );
    trim_git_suffix(&mut segments);
    (!segments.is_empty()).then_some(segments)
}

fn sanitize_namespace_segment(segment: &str) -> String {
    segment
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn trim_git_suffix(segments: &mut [String]) {
    if let Some(last) = segments.last_mut()
        && let Some(stripped) = last.strip_suffix(".git")
        && !stripped.is_empty()
    {
        *last = stripped.to_string();
    }
}

/// Sanitizes a repository identifier for filesystem usage.
#[must_use]
pub fn sanitize_repo_id(repo_id: &str) -> String {
    repo_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
