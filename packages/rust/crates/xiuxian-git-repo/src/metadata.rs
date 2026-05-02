//! Checkout metadata discovery and managed remote probe state.

use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use chrono::Utc;
use gix::Repository;
use gix::bstr::ByteSlice;
use serde::{Deserialize, Serialize};

use crate::spec::RevisionSelector;
const MANAGED_REMOTE_PROBE_STATE_FILE: &str = "xiuxian-upstream-probe-state.json";

/// Minimal metadata observed from a local checkout.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalCheckoutMetadata {
    /// Current checkout revision.
    pub revision: Option<String>,
    /// Configured `origin` remote URL.
    pub remote_url: Option<String>,
}

/// Status of the last managed-remote probe.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ManagedRemoteProbeStatus {
    /// The probe succeeded.
    #[default]
    Success,
    /// The probe failed with a retryable transport-like issue.
    RetryableFailure,
    /// The probe failed permanently.
    Failure,
}

/// Persisted state for one managed-remote probe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManagedRemoteProbeState {
    /// Timestamp of the current observation.
    pub checked_at: String,
    /// Probe status.
    #[serde(default)]
    pub status: ManagedRemoteProbeStatus,
    /// Latest successful or current target revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
    /// Backend-provided error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Timestamp of the last successful probe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_success_checked_at: Option<String>,
    /// Target revision from the last successful probe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_success_target_revision: Option<String>,
}

/// Drift state between managed mirror and checkout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepoDriftState {
    /// Drift does not apply.
    #[default]
    NotApplicable,
    /// Drift could not be determined.
    Unknown,
    /// Checkout and mirror are aligned.
    InSync,
    /// Checkout has local commits ahead of mirror.
    Ahead,
    /// Checkout is behind mirror.
    Behind,
    /// Checkout and mirror diverged.
    Diverged,
}

/// Discovers metadata from a local checkout path.
#[must_use]
pub fn discover_checkout_metadata(path: &Path) -> Option<LocalCheckoutMetadata> {
    if !path.is_dir() {
        return None;
    }

    let repository = gix::open(path.to_path_buf()).ok()?;
    Some(LocalCheckoutMetadata {
        revision: resolve_head_revision(&repository),
        remote_url: repository.find_remote("origin").ok().and_then(|remote| {
            remote
                .url(gix::remote::Direction::Fetch)
                .map(display_remote_url)
        }),
    })
}

pub(crate) fn resolve_head_revision(repository: &Repository) -> Option<String> {
    repository.head_id().ok().map(|head| head.to_string())
}

pub(crate) fn resolve_tracking_revision(
    repository: &Repository,
    revision: Option<&RevisionSelector>,
) -> Option<String> {
    match revision {
        Some(RevisionSelector::Commit(sha)) => Some(sha.clone()),
        Some(RevisionSelector::Tag(tag)) => repository
            .find_reference(format!("refs/tags/{tag}").as_str())
            .ok()
            .and_then(|mut reference| reference.peel_to_id().ok().map(|oid| oid.to_string())),
        Some(RevisionSelector::Branch(branch)) => repository
            .find_reference(format!("refs/remotes/origin/{branch}").as_str())
            .ok()
            .and_then(|mut reference| reference.peel_to_id().ok().map(|oid| oid.to_string())),
        None => repository
            .find_reference("refs/remotes/origin/HEAD")
            .ok()
            .and_then(|reference| match reference.target() {
                gix::refs::TargetRef::Symbolic(target) => Some(target.to_string()),
                gix::refs::TargetRef::Object(_) => None,
            })
            .and_then(|target| repository.find_reference(target.as_str()).ok())
            .and_then(|mut reference| reference.peel_to_id().ok().map(|oid| oid.to_string()))
            .or_else(|| {
                ["refs/remotes/origin/main", "refs/remotes/origin/master"]
                    .into_iter()
                    .find_map(|reference| {
                        repository
                            .find_reference(reference)
                            .ok()
                            .and_then(|mut reference| {
                                reference.peel_to_id().ok().map(|oid| oid.to_string())
                            })
                    })
            }),
    }
}

pub(crate) fn compute_managed_drift_state(
    repository: &Repository,
    checkout_revision: Option<&str>,
    tracking_revision: Option<&str>,
    mirror_revision: Option<&str>,
) -> RepoDriftState {
    let Some(checkout_revision) = checkout_revision else {
        return RepoDriftState::Unknown;
    };
    let Some(mirror_revision) = mirror_revision else {
        return RepoDriftState::Unknown;
    };

    if checkout_revision == mirror_revision {
        return RepoDriftState::InSync;
    }

    let Some(tracking_revision) = tracking_revision else {
        return RepoDriftState::Unknown;
    };

    if checkout_revision == tracking_revision {
        return RepoDriftState::Behind;
    }

    if tracking_revision == mirror_revision {
        return match compare_revision_lineage(repository, checkout_revision, tracking_revision) {
            Some(Ordering::Greater) => RepoDriftState::Ahead,
            Some(Ordering::Less) => RepoDriftState::Behind,
            Some(Ordering::Equal) => RepoDriftState::InSync,
            None => RepoDriftState::Diverged,
        };
    }

    RepoDriftState::Diverged
}

fn compare_revision_lineage(repository: &Repository, left: &str, right: &str) -> Option<Ordering> {
    if left == right {
        return Some(Ordering::Equal);
    }

    let left = repository.rev_parse_single(left).ok()?.detach();
    let right = repository.rev_parse_single(right).ok()?.detach();
    let left_descends = repository
        .merge_base(left, right)
        .ok()
        .is_some_and(|base| base.detach() == right);
    let right_descends = repository
        .merge_base(right, left)
        .ok()
        .is_some_and(|base| base.detach() == left);

    match (left_descends, right_descends) {
        (true, false) => Some(Ordering::Greater),
        (false, true) => Some(Ordering::Less),
        (false, false) => None,
        (true, true) => Some(Ordering::Equal),
    }
}

pub(crate) fn discover_last_fetched_at(mirror_root: &Path) -> Option<String> {
    ["FETCH_HEAD", "HEAD"]
        .into_iter()
        .filter_map(|name| fs::metadata(mirror_root.join(name)).ok())
        .filter_map(|metadata| metadata.modified().ok())
        .max()
        .map(|modified| chrono::DateTime::<Utc>::from(modified).to_rfc3339())
}

/// Discovers persisted probe state for one managed mirror.
#[must_use]
pub fn discover_managed_remote_probe_state(mirror_root: &Path) -> Option<ManagedRemoteProbeState> {
    let payload = fs::read(mirror_root.join(MANAGED_REMOTE_PROBE_STATE_FILE)).ok()?;
    serde_json::from_slice(&payload).ok()
}

/// Records a successful probe result for one managed mirror.
///
/// # Errors
///
/// Returns an error if the probe payload cannot be encoded or written to the
/// managed mirror state file.
pub fn record_managed_remote_probe_state(
    mirror_root: &Path,
    target_revision: Option<&str>,
) -> std::io::Result<()> {
    let checked_at = Utc::now().to_rfc3339();
    let target_revision = target_revision.map(str::to_string);
    let payload = ManagedRemoteProbeState {
        checked_at: checked_at.clone(),
        status: ManagedRemoteProbeStatus::Success,
        target_revision: target_revision.clone(),
        error_message: None,
        last_success_checked_at: Some(checked_at),
        last_success_target_revision: target_revision,
    };
    write_managed_remote_probe_state(mirror_root, &payload)
}

/// Records a failed probe result for one managed mirror.
///
/// # Errors
///
/// Returns an error if the failure payload cannot be encoded or written to the
/// managed mirror state file.
pub fn record_managed_remote_probe_failure(
    mirror_root: &Path,
    error_message: &str,
    retryable: bool,
) -> std::io::Result<()> {
    let (last_success_checked_at, last_success_target_revision) =
        discover_managed_remote_probe_state(mirror_root).map_or((None, None), |state| match state
            .status
        {
            ManagedRemoteProbeStatus::Success => (Some(state.checked_at), state.target_revision),
            ManagedRemoteProbeStatus::RetryableFailure | ManagedRemoteProbeStatus::Failure => (
                state.last_success_checked_at,
                state.last_success_target_revision,
            ),
        });
    let payload = ManagedRemoteProbeState {
        checked_at: Utc::now().to_rfc3339(),
        status: if retryable {
            ManagedRemoteProbeStatus::RetryableFailure
        } else {
            ManagedRemoteProbeStatus::Failure
        },
        target_revision: None,
        error_message: Some(error_message.to_string()),
        last_success_checked_at,
        last_success_target_revision,
    };
    write_managed_remote_probe_state(mirror_root, &payload)
}

/// Removes persisted probe state for one managed mirror.
///
/// # Errors
///
/// Returns an error if the state file exists but cannot be removed.
pub fn clear_managed_remote_probe_state(mirror_root: &Path) -> std::io::Result<()> {
    match fs::remove_file(mirror_root.join(MANAGED_REMOTE_PROBE_STATE_FILE)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn write_managed_remote_probe_state(
    mirror_root: &Path,
    payload: &ManagedRemoteProbeState,
) -> std::io::Result<()> {
    let payload = serde_json::to_vec(payload)
        .map_err(|error| std::io::Error::other(format!("encode probe state: {error}")))?;
    fs::write(mirror_root.join(MANAGED_REMOTE_PROBE_STATE_FILE), payload)
}

fn display_remote_url(url: &gix::Url) -> String {
    if url.scheme == gix::url::Scheme::File {
        return gix::path::from_bstr(url.path.as_bstr())
            .into_owned()
            .display()
            .to_string();
    }
    url.to_bstring().to_string()
}

#[cfg(test)]
#[path = "../tests/unit/metadata.rs"]
mod tests;
