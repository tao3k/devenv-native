//! Managed checkout lock acquisition and stale-lock recovery.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use xiuxian_config_core::ProjectDirs;

use crate::error::{RepoError, RepoErrorKind};
use crate::layout::managed_mirror_root_for;
use crate::spec::RepoSpec;

const CHECKOUT_LOCK_RETRY_DELAY_ENV: &str = "XIUXIAN_GIT_REPO_CHECKOUT_LOCK_RETRY_DELAY_MS";
const CHECKOUT_LOCK_MAX_WAIT_ENV: &str = "XIUXIAN_WENDAO_CHECKOUT_LOCK_MAX_WAIT_SECS";
const DEFAULT_CHECKOUT_LOCK_MAX_WAIT_SECS: u64 = 20;
const CHECKOUT_LOCK_STALE_AFTER: Duration = Duration::from_mins(2);
const TOO_MANY_OPEN_FILES_OS_ERROR: i32 = 24;
const MIN_CHECKOUT_LOCK_RETRY_DELAY_MS: u64 = 50;
const MAX_CHECKOUT_LOCK_RETRY_DELAY_MS: u64 = 150;
const CHECKOUT_LOCK_RETRY_DELAY_BASE_MS: u64 = 40;
const CHECKOUT_LOCK_RETRY_DELAY_MS_PER_CORE: u64 = 5;

/// Guard for one managed checkout lockfile.
#[derive(Debug)]
pub struct ManagedCheckoutLock {
    path: PathBuf,
    _file: fs::File,
}

impl Drop for ManagedCheckoutLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Acquires the managed checkout lock for a repository.
///
/// # Errors
///
/// Returns an error if the managed lock path cannot be created, acquired,
/// reclaimed after staleness, or waited on within the configured timeout.
pub fn acquire_managed_checkout_lock(spec: &RepoSpec) -> Result<ManagedCheckoutLock, RepoError> {
    acquire_managed_checkout_lock_with_policy(
        managed_lock_path_for(spec),
        checkout_lock_retry_delay(),
        checkout_lock_max_wait(),
        CHECKOUT_LOCK_STALE_AFTER,
    )
}

fn checkout_lock_retry_delay() -> Duration {
    checkout_lock_retry_delay_with_lookup(&|key| std::env::var(key).ok())
}

fn checkout_lock_max_wait() -> Duration {
    checkout_lock_max_wait_with_lookup(&|key| std::env::var(key).ok())
}

fn checkout_lock_retry_delay_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Duration {
    lookup(CHECKOUT_LOCK_RETRY_DELAY_ENV)
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .map_or_else(default_checkout_lock_retry_delay, Duration::from_millis)
}

fn default_checkout_lock_retry_delay() -> Duration {
    default_checkout_lock_retry_delay_for_parallelism(
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
    )
}

fn default_checkout_lock_retry_delay_for_parallelism(parallelism: usize) -> Duration {
    let parallelism = u64::try_from(parallelism.max(1)).unwrap_or(u64::MAX);
    let delay_ms = CHECKOUT_LOCK_RETRY_DELAY_BASE_MS
        .saturating_add(parallelism.saturating_mul(CHECKOUT_LOCK_RETRY_DELAY_MS_PER_CORE))
        .clamp(
            MIN_CHECKOUT_LOCK_RETRY_DELAY_MS,
            MAX_CHECKOUT_LOCK_RETRY_DELAY_MS,
        );
    Duration::from_millis(delay_ms)
}

/// Resolves the checkout-lock timeout using the provided lookup function.
#[must_use]
pub fn checkout_lock_max_wait_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Duration {
    let parsed = lookup(CHECKOUT_LOCK_MAX_WAIT_ENV)
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0);
    Duration::from_secs(parsed.unwrap_or(DEFAULT_CHECKOUT_LOCK_MAX_WAIT_SECS))
}

/// Computes the managed checkout lock path for a repository.
#[must_use]
pub fn managed_lock_path_for(spec: &RepoSpec) -> PathBuf {
    let intelligence_root = ProjectDirs::data_home()
        .join("xiuxian-wendao")
        .join("repo-intelligence");
    let mirrors_root = intelligence_root.join("mirrors");
    let managed_mirror_root = managed_mirror_root_for(spec);
    let relative_path = managed_mirror_root.strip_prefix(&mirrors_root).map_or_else(
        |_| PathBuf::from(format!("{}.git", spec.id)),
        Path::to_path_buf,
    );

    intelligence_root
        .join("locks")
        .join(relative_path)
        .with_extension("lock")
}

/// Acquires a managed checkout lock using explicit policy values.
///
/// # Errors
///
/// Returns an error if the lock directory cannot be created, the lock cannot
/// be acquired before `max_wait`, or a stale lock cannot be reclaimed.
pub fn acquire_managed_checkout_lock_with_policy(
    lock_path: PathBuf,
    retry_delay: Duration,
    max_wait: Duration,
    stale_after: Duration,
) -> Result<ManagedCheckoutLock, RepoError> {
    ensure_lock_parent_directory(&lock_path)?;
    wait_for_managed_checkout_lock(lock_path, retry_delay, max_wait, stale_after)
}

fn ensure_lock_parent_directory(lock_path: &Path) -> Result<(), RepoError> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|error| lock_parent_directory_error(parent, &error))?;
    }
    Ok(())
}

fn lock_parent_directory_error(parent: &Path, error: &std::io::Error) -> RepoError {
    RepoError::new(
        RepoErrorKind::Permanent,
        format!(
            "failed to create managed checkout lock dir `{}`: {error}",
            parent.display()
        ),
    )
}

fn wait_for_managed_checkout_lock(
    lock_path: PathBuf,
    retry_delay: Duration,
    max_wait: Duration,
    stale_after: Duration,
) -> Result<ManagedCheckoutLock, RepoError> {
    let started_at = Instant::now();
    loop {
        match try_managed_checkout_lock(&lock_path, stale_after)? {
            LockAttempt::Acquired(file) => {
                return Ok(ManagedCheckoutLock {
                    path: lock_path,
                    _file: file,
                });
            }
            LockAttempt::Busy => wait_or_timeout(
                &lock_path,
                started_at,
                max_wait,
                retry_delay,
                RepoErrorKind::LockBusy,
                None,
            )?,
            LockAttempt::DescriptorPressure(error) => wait_or_timeout(
                &lock_path,
                started_at,
                max_wait,
                retry_delay,
                RepoErrorKind::DescriptorPressure,
                Some(&error),
            )?,
        }
    }
}

enum LockAttempt {
    Acquired(fs::File),
    Busy,
    DescriptorPressure(std::io::Error),
}

fn try_managed_checkout_lock(
    lock_path: &Path,
    stale_after: Duration,
) -> Result<LockAttempt, RepoError> {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)
    {
        Ok(mut file) => {
            write_lockfile_metadata(&mut file);
            Ok(LockAttempt::Acquired(file))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            reclaim_stale_lock_if_needed(lock_path, stale_after)?;
            Ok(LockAttempt::Busy)
        }
        Err(error) if is_descriptor_pressure_error(&error) => {
            Ok(LockAttempt::DescriptorPressure(error))
        }
        Err(error) => Err(acquire_lock_error(lock_path, &error)),
    }
}

fn write_lockfile_metadata(file: &mut fs::File) {
    let _ = writeln!(
        file,
        "pid={} acquired_at={}",
        std::process::id(),
        Utc::now().to_rfc3339()
    );
}

fn reclaim_stale_lock_if_needed(lock_path: &Path, stale_after: Duration) -> Result<(), RepoError> {
    if !lockfile_is_stale(lock_path, stale_after) {
        return Ok(());
    }
    match fs::remove_file(lock_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(reclaim_stale_lock_error(lock_path, &error)),
    }
}

fn wait_or_timeout(
    lock_path: &Path,
    started_at: Instant,
    max_wait: Duration,
    retry_delay: Duration,
    timeout_kind: RepoErrorKind,
    cause: Option<&std::io::Error>,
) -> Result<(), RepoError> {
    if started_at.elapsed() >= max_wait {
        return Err(timeout_waiting_for_lock_error(
            lock_path,
            timeout_kind,
            cause,
        ));
    }
    thread::sleep(retry_delay);
    Ok(())
}

fn timeout_waiting_for_lock_error(
    lock_path: &Path,
    kind: RepoErrorKind,
    cause: Option<&std::io::Error>,
) -> RepoError {
    let suffix = cause.map_or_else(String::new, |error| {
        format!(" while file-descriptor pressure persisted: {error}")
    });
    RepoError::new(
        kind,
        format!(
            "timed out waiting for managed checkout lock `{}`{suffix}",
            lock_path.display()
        ),
    )
}

fn acquire_lock_error(lock_path: &Path, error: &std::io::Error) -> RepoError {
    RepoError::new(
        RepoErrorKind::Permanent,
        format!(
            "failed to acquire managed checkout lock `{}`: {error}",
            lock_path.display()
        ),
    )
}

fn reclaim_stale_lock_error(lock_path: &Path, error: &std::io::Error) -> RepoError {
    RepoError::new(
        RepoErrorKind::Permanent,
        format!(
            "failed to reclaim stale managed checkout lock `{}`: {error}",
            lock_path.display()
        ),
    )
}

fn lockfile_is_stale(lock_path: &Path, stale_after: Duration) -> bool {
    fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified_at| modified_at.elapsed().ok())
        .is_some_and(|elapsed| elapsed >= stale_after)
}

/// Returns true when the IO error indicates file-descriptor pressure.
#[must_use]
pub fn is_descriptor_pressure_error(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(TOO_MANY_OPEN_FILES_OS_ERROR)
}

#[cfg(test)]
#[path = "../tests/unit/lock.rs"]
mod tests;
