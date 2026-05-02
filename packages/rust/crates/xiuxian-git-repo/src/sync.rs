//! Repository source materialization and refresh orchestration.

use std::fs;
use std::path::Path;

use chrono::Utc;

use crate::backend::{
    clone_bare_with_retry, clone_checkout_from_mirror, ensure_remote_url, fetch_origin_with_retry,
    is_retryable_remote_error_message, open_bare_with_retry, open_checkout_with_retry,
    probe_remote_target_revision_with_retry, should_fetch,
};
use crate::error::{RepoError, RepoErrorKind};
use crate::layout::{managed_checkout_root_for, managed_mirror_root_for};
use crate::lock::acquire_managed_checkout_lock;
use crate::metadata::{
    RepoDriftState, clear_managed_remote_probe_state, compute_managed_drift_state,
    discover_last_fetched_at, record_managed_remote_probe_failure,
    record_managed_remote_probe_state, resolve_head_revision, resolve_tracking_revision,
};
use crate::revision::{desired_managed_checkout_revision, sync_checkout_head};
use crate::spec::{RepoRefreshPolicy, RepoSpec, RevisionSelector};

/// Synchronization mode for repository source preparation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SyncMode {
    /// Prepare the repository source while respecting the refresh policy.
    #[default]
    Ensure,
    /// Force a refresh for managed remotes.
    Refresh,
    /// Observe current state without mutating managed assets.
    Status,
}

/// Resolved repository source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepoSourceKind {
    /// Operator-provided local checkout.
    #[default]
    LocalCheckout,
    /// Managed mirror-backed checkout.
    ManagedRemote,
}

/// Lifecycle state for one repository substrate asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoLifecycleState {
    /// Lifecycle does not apply.
    NotApplicable,
    /// The asset is missing.
    Missing,
    /// The asset was validated.
    Validated,
    /// The asset was observed without mutation.
    Observed,
    /// The asset was created.
    Created,
    /// The asset was reused without refresh.
    Reused,
    /// The asset was refreshed.
    Refreshed,
}

/// Resolved materialized repository state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedRepo {
    /// Root path of the working checkout.
    pub checkout_root: std::path::PathBuf,
    /// Managed mirror root, when the repository is remote-managed.
    pub mirror_root: Option<std::path::PathBuf>,
    /// Current revision observed from the managed mirror.
    pub mirror_revision: Option<String>,
    /// Tracking revision derived from the checkout or requested revision.
    pub tracking_revision: Option<String>,
    /// Timestamp of the last local fetch.
    pub last_fetched_at: Option<String>,
    /// Drift summary between checkout and mirror.
    pub drift_state: RepoDriftState,
    /// Lifecycle state of the mirror.
    pub mirror_state: RepoLifecycleState,
    /// Lifecycle state of the checkout.
    pub checkout_state: RepoLifecycleState,
    /// Resolved source kind.
    pub source_kind: RepoSourceKind,
}

/// Resolves and materializes the repository source for a repository spec.
///
/// # Errors
///
/// Returns an error if the repository source is missing or cannot be prepared.
pub fn resolve_repository_source(
    spec: &RepoSpec,
    cwd: &Path,
    mode: SyncMode,
) -> Result<MaterializedRepo, RepoError> {
    if let Some(path) = spec.local_path.as_ref() {
        return resolve_local_checkout(spec, cwd, path, mode);
    }

    if spec.remote_url.is_some() {
        return resolve_managed_checkout(spec, mode);
    }

    Err(RepoError::new(
        RepoErrorKind::MissingSource,
        format!(
            "repo `{}` must declare a local path or upstream url",
            spec.id
        ),
    ))
}

fn resolve_local_checkout(
    spec: &RepoSpec,
    cwd: &Path,
    path: &Path,
    mode: SyncMode,
) -> Result<MaterializedRepo, RepoError> {
    let checkout_root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    if checkout_root.exists() && !checkout_root.is_dir() {
        return Err(RepoError::new(
            RepoErrorKind::InvalidPath,
            format!(
                "repo `{}` has invalid local path `{}`: path exists but is not a directory",
                spec.id,
                checkout_root.display()
            ),
        ));
    }

    if !checkout_root.exists() && !matches!(mode, SyncMode::Status) {
        return Err(RepoError::new(
            RepoErrorKind::InvalidPath,
            format!(
                "repo `{}` has invalid local path `{}`: directory does not exist",
                spec.id,
                checkout_root.display()
            ),
        ));
    }

    let checkout_state = if checkout_root.is_dir() {
        open_checkout_with_retry(&checkout_root).map_err(|error| {
            RepoError::new(
                RepoErrorKind::InvalidPath,
                format!(
                    "repo `{}` has invalid local path `{}`: path is not a git checkout: {error}",
                    spec.id,
                    checkout_root.display()
                ),
            )
        })?;
        RepoLifecycleState::Validated
    } else {
        RepoLifecycleState::Missing
    };

    Ok(MaterializedRepo {
        checkout_root,
        mirror_root: None,
        mirror_revision: None,
        tracking_revision: None,
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state,
        source_kind: RepoSourceKind::LocalCheckout,
    })
}

#[allow(clippy::too_many_lines)]
fn resolve_managed_checkout(
    spec: &RepoSpec,
    mode: SyncMode,
) -> Result<MaterializedRepo, RepoError> {
    let upstream_url = spec
        .remote_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            RepoError::new(
                RepoErrorKind::MissingSource,
                format!(
                    "repo `{}` must declare a local path or upstream url",
                    spec.id
                ),
            )
        })?;
    let mirror_root = managed_mirror_root_for(spec);
    let checkout_root = managed_checkout_root_for(spec);

    if (!mirror_root.exists() || !checkout_root.exists()) && matches!(mode, SyncMode::Status) {
        return Ok(MaterializedRepo {
            checkout_root,
            mirror_root: Some(mirror_root),
            mirror_revision: None,
            tracking_revision: None,
            last_fetched_at: None,
            drift_state: RepoDriftState::Unknown,
            mirror_state: RepoLifecycleState::Missing,
            checkout_state: RepoLifecycleState::Missing,
            source_kind: RepoSourceKind::ManagedRemote,
        });
    }

    let _checkout_lock = (!matches!(mode, SyncMode::Status))
        .then(|| acquire_managed_checkout_lock(spec))
        .transpose()?;

    if let Some(parent) = mirror_root.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            RepoError::new(
                RepoErrorKind::Permanent,
                format!(
                    "failed to create managed mirror dir `{}`: {error}",
                    parent.display()
                ),
            )
        })?;
    }
    if let Some(parent) = checkout_root.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            RepoError::new(
                RepoErrorKind::Permanent,
                format!(
                    "failed to create managed checkout dir `{}`: {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let mirror_existed = mirror_root.exists();
    let (mirror_repository, mirror_mutated) = if mirror_existed {
        let mut repo = open_bare_with_retry(&mirror_root).map_err(|error| {
            RepoError::new(
                RepoError::classify_message(error.message()),
                format!(
                    "failed to open managed mirror `{}` as bare git repository: {error}",
                    mirror_root.display()
                ),
            )
        })?;
        let remote_updated = if matches!(mode, SyncMode::Status) {
            false
        } else {
            ensure_remote_url(&mut repo, "origin", upstream_url).map_err(|error| {
                RepoError::new(
                    RepoErrorKind::RemoteMisconfigured,
                    format!(
                        "failed to align managed mirror `{}` remote with `{upstream_url}`: {error}",
                        spec.id
                    ),
                )
            })?
        };
        let current_mirror_revision =
            desired_managed_checkout_revision(&repo, spec.revision.as_ref());
        let probed_target_revision = if matches!(mode, SyncMode::Ensure)
            && matches!(spec.refresh, RepoRefreshPolicy::Fetch)
            && !remote_updated
            && !matches!(spec.revision, Some(RevisionSelector::Commit(_)))
        {
            Some(probe_remote_target_revision_with_retry(
                &repo,
                spec.revision.as_ref(),
            ))
        } else {
            None
        };
        let mirror_requires_fetch = mirror_requires_fetch(
            mode,
            spec.refresh,
            spec.revision.as_ref(),
            remote_updated,
            current_mirror_revision.as_deref(),
            probed_target_revision
                .as_ref()
                .and_then(|result| result.as_ref().ok())
                .and_then(|revision| revision.as_deref()),
        );
        if let Some(Err(error)) = probed_target_revision.as_ref() {
            record_managed_remote_probe_failure(
                &mirror_root,
                error.message(),
                is_retryable_remote_error_message(error.message()),
            )
            .map_err(|record_error| {
                RepoError::new(
                    RepoErrorKind::Permanent,
                    format!(
                        "failed to record managed mirror probe failure for `{}`: {record_error}",
                        spec.id
                    ),
                )
            })?;
        }
        if mirror_requires_fetch {
            fetch_origin_with_retry(&repo).map_err(|error| {
                RepoError::new(
                    RepoError::classify_message(error.message()),
                    format!(
                        "failed to refresh managed mirror `{}` from `{upstream_url}`: {error}",
                        spec.id
                    ),
                )
            })?;
            clear_managed_remote_probe_state(&mirror_root).map_err(|error| {
                RepoError::new(
                    RepoErrorKind::Permanent,
                    format!(
                        "failed to clear managed mirror probe state for `{}` after fetch: {error}",
                        spec.id
                    ),
                )
            })?;
        } else if let Some(Ok(target_revision)) = probed_target_revision.as_ref() {
            record_managed_remote_probe_state(&mirror_root, target_revision.as_deref()).map_err(
                |error| {
                    RepoError::new(
                        RepoErrorKind::Permanent,
                        format!(
                            "failed to record managed mirror probe state for `{}`: {error}",
                            spec.id
                        ),
                    )
                },
            )?;
        }
        (repo, remote_updated || mirror_requires_fetch)
    } else {
        (
            clone_bare_with_retry(upstream_url, &mirror_root).map_err(|error| {
                RepoError::new(
                    RepoError::classify_message(error.message()),
                    format!(
                        "failed to clone mirror for repository `{}` from `{upstream_url}`: {error}",
                        spec.id
                    ),
                )
            })?,
            true,
        )
    };
    let mirror_revision = resolve_head_revision(&mirror_repository);
    let mirror_state = lifecycle_state_for(mode, mirror_existed, mirror_mutated);
    let desired_checkout_revision =
        desired_managed_checkout_revision(&mirror_repository, spec.revision.as_ref());
    let checkout_existed = checkout_root.exists();
    let mirror_origin = std::fs::canonicalize(&mirror_root)
        .unwrap_or_else(|_| mirror_root.clone())
        .display()
        .to_string();
    let (mut repository_handle, checkout_mutated) = if checkout_existed {
        let mut repo = open_checkout_with_retry(&checkout_root).map_err(|error| {
            RepoError::new(
                RepoError::classify_message(error.message()),
                format!(
                    "failed to open managed checkout `{}` as git repository: {error}",
                    checkout_root.display()
                ),
            )
        })?;
        let current_checkout_revision = resolve_head_revision(&repo);
        let current_tracking_revision = resolve_tracking_revision(&repo, spec.revision.as_ref());
        let remote_updated = if matches!(mode, SyncMode::Status) {
            false
        } else {
            ensure_remote_url(&mut repo, "origin", mirror_origin.as_str()).map_err(|error| {
                RepoError::new(
                    RepoErrorKind::RemoteMisconfigured,
                    format!(
                        "failed to align managed checkout `{}` remote with mirror `{mirror_origin}`: {error}",
                        spec.id
                    ),
                )
            })?
        };
        let checkout_requires_fetch = mirror_mutated
            || checkout_requires_fetch(
                mode,
                spec.refresh,
                remote_updated,
                current_checkout_revision.as_deref(),
                desired_checkout_revision.as_deref(),
            );
        let mut checkout_was_rebuilt = false;
        if checkout_requires_fetch {
            match fetch_origin_with_retry(&repo) {
                Ok(()) => {}
                Err(error)
                    if managed_checkout_rebuild_allowed(
                        current_checkout_revision.as_deref(),
                        current_tracking_revision.as_deref(),
                    ) =>
                {
                    drop(repo);
                    fs::remove_dir_all(&checkout_root).map_err(|remove_error| {
                        RepoError::new(
                            RepoErrorKind::Permanent,
                            format!(
                                "failed to rebuild managed checkout `{}` after fetch failure `{error}`: {remove_error}",
                                spec.id
                            ),
                        )
                    })?;
                    repo = clone_checkout_from_mirror(&mirror_origin, &checkout_root).map_err(
                        |clone_error| {
                            RepoError::new(
                                RepoError::classify_message(clone_error.message()),
                                format!(
                                    "failed to rebuild managed checkout `{}` from mirror `{mirror_origin}` after fetch failure `{error}`: {clone_error}",
                                    spec.id
                                ),
                            )
                        },
                    )?;
                    checkout_was_rebuilt = true;
                }
                Err(error) => {
                    return Err(RepoError::new(
                        RepoError::classify_message(error.message()),
                        format!(
                            "failed to refresh managed checkout `{}` from mirror `{mirror_origin}`: {error}",
                            spec.id
                        ),
                    ));
                }
            }
        }
        let checkout_head_revision = if checkout_was_rebuilt {
            resolve_head_revision(&repo)
        } else {
            current_checkout_revision
        };
        let checkout_requires_head_sync = !matches!(mode, SyncMode::Status)
            && checkout_head_revision.as_deref() != desired_checkout_revision.as_deref();
        if checkout_requires_head_sync {
            sync_checkout_head(&mut repo, spec.revision.as_ref()).map_err(|error| {
                RepoError::new(
                    RepoError::classify_message(error.message()),
                    format!(
                        "failed to materialize requested git ref for `{}`: {error}",
                        spec.id
                    ),
                )
            })?;
        }
        (
            repo,
            remote_updated
                || checkout_requires_fetch
                || checkout_requires_head_sync
                || checkout_was_rebuilt,
        )
    } else {
        clone_checkout_from_mirror(&mirror_origin, &checkout_root)
            .map(|repo| (repo, true))
            .map_err(|error| {
                RepoError::new(
                    RepoError::classify_message(error.message()),
                    format!(
                        "failed to materialize managed checkout `{}` from mirror `{mirror_origin}`: {error}",
                        spec.id
                    ),
                )
            })?
    };

    if !matches!(mode, SyncMode::Status) && !checkout_existed && desired_checkout_revision.is_some()
    {
        sync_checkout_head(&mut repository_handle, spec.revision.as_ref()).map_err(|error| {
            RepoError::new(
                RepoError::classify_message(error.message()),
                format!(
                    "failed to materialize requested git ref for `{}`: {error}",
                    spec.id
                ),
            )
        })?;
    }

    let revision = resolve_head_revision(&repository_handle);
    let tracking_revision = resolve_tracking_revision(&repository_handle, spec.revision.as_ref());
    let checkout_state = lifecycle_state_for(mode, checkout_existed, checkout_mutated);
    let fetched_at = discover_last_fetched_at(&mirror_root)
        .or_else(|| (!matches!(mode, SyncMode::Status)).then(|| Utc::now().to_rfc3339()));

    Ok(MaterializedRepo {
        checkout_root: checkout_root.clone(),
        mirror_root: Some(mirror_root),
        mirror_revision: mirror_revision.clone(),
        tracking_revision: tracking_revision.clone(),
        last_fetched_at: fetched_at,
        drift_state: compute_managed_drift_state(
            &repository_handle,
            revision.as_deref(),
            tracking_revision.as_deref(),
            mirror_revision.as_deref(),
        ),
        mirror_state,
        checkout_state,
        source_kind: RepoSourceKind::ManagedRemote,
    })
}

fn lifecycle_state_for(mode: SyncMode, existed: bool, mutated: bool) -> RepoLifecycleState {
    if !existed {
        return RepoLifecycleState::Created;
    }
    if matches!(mode, SyncMode::Status) {
        return RepoLifecycleState::Observed;
    }
    if mutated {
        RepoLifecycleState::Refreshed
    } else {
        RepoLifecycleState::Reused
    }
}

fn checkout_requires_fetch(
    mode: SyncMode,
    refresh: RepoRefreshPolicy,
    remote_updated: bool,
    current_checkout_revision: Option<&str>,
    desired_checkout_revision: Option<&str>,
) -> bool {
    if matches!(mode, SyncMode::Status | SyncMode::Refresh) {
        return matches!(mode, SyncMode::Refresh);
    }
    if remote_updated {
        return true;
    }
    if !should_fetch(refresh, mode) {
        return false;
    }
    current_checkout_revision != desired_checkout_revision
}

fn managed_checkout_rebuild_allowed(
    current_checkout_revision: Option<&str>,
    current_tracking_revision: Option<&str>,
) -> bool {
    current_checkout_revision.is_some() && current_checkout_revision == current_tracking_revision
}

fn mirror_requires_fetch(
    mode: SyncMode,
    refresh: RepoRefreshPolicy,
    revision: Option<&RevisionSelector>,
    remote_updated: bool,
    current_mirror_revision: Option<&str>,
    probed_target_revision: Option<&str>,
) -> bool {
    if matches!(mode, SyncMode::Refresh) {
        return true;
    }
    if matches!(mode, SyncMode::Status) {
        return false;
    }
    if remote_updated {
        return true;
    }
    if !should_fetch(refresh, mode) {
        return false;
    }

    match revision {
        Some(RevisionSelector::Commit(commit)) => current_mirror_revision != Some(commit.as_str()),
        _ => probed_target_revision != current_mirror_revision,
    }
}
