use std::fs;
use std::time::Duration;

use xiuxian_git_repo::{
    RepoErrorKind, RepoRefreshPolicy, RepoSpec, acquire_managed_checkout_lock_with_policy,
    checkout_lock_max_wait_with_lookup, is_descriptor_pressure_error, managed_lock_path_for,
};

use crate::support::must;

#[test]
fn managed_checkout_lock_reclaims_stale_lockfiles() {
    let spec = RepoSpec {
        id: format!("managed-lock-{}", uuid::Uuid::new_v4()),
        local_path: None,
        remote_url: Some(format!(
            "https://example.com/org/{}.git",
            uuid::Uuid::new_v4()
        )),
        revision: None,
        refresh: RepoRefreshPolicy::Manual,
    };
    let lock_path = managed_lock_path_for(&spec);
    if let Some(parent) = lock_path.parent() {
        must(fs::create_dir_all(parent), "create lock dir");
    }
    must(fs::write(&lock_path, "stale"), "write stale lock");

    let guard = must(
        acquire_managed_checkout_lock_with_policy(
            lock_path.clone(),
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::ZERO,
        ),
        "reclaim stale lock",
    );

    assert!(lock_path.exists());
    drop(guard);
    assert!(!lock_path.exists());
}

#[test]
fn managed_checkout_lock_times_out_for_active_lockfiles() {
    let spec = RepoSpec {
        id: format!("managed-lock-busy-{}", uuid::Uuid::new_v4()),
        local_path: None,
        remote_url: Some(format!(
            "https://example.com/org/{}.git",
            uuid::Uuid::new_v4()
        )),
        revision: None,
        refresh: RepoRefreshPolicy::Manual,
    };
    let lock_path = managed_lock_path_for(&spec);
    if let Some(parent) = lock_path.parent() {
        must(fs::create_dir_all(parent), "create lock dir");
    }
    must(fs::write(&lock_path, "busy"), "write active lock");

    let error = match acquire_managed_checkout_lock_with_policy(
        lock_path.clone(),
        Duration::from_millis(1),
        Duration::from_millis(5),
        Duration::from_mins(1),
    ) {
        Ok(_guard) => panic!("active lock should time out"),
        Err(error) => error,
    };

    assert_eq!(error.kind, RepoErrorKind::LockBusy);
    assert!(
        error
            .message
            .contains("timed out waiting for managed checkout lock")
    );

    must(fs::remove_file(&lock_path), "cleanup active lock");
}

#[test]
fn managed_checkout_lock_wait_defaults_to_pressure_tolerant_window() {
    let wait = checkout_lock_max_wait_with_lookup(&|_| None);

    assert_eq!(wait, Duration::from_secs(20));
}

#[test]
fn managed_checkout_lock_wait_accepts_positive_env_override() {
    let wait = checkout_lock_max_wait_with_lookup(&|key| {
        (key == "XIUXIAN_WENDAO_CHECKOUT_LOCK_MAX_WAIT_SECS").then(|| "30".to_string())
    });

    assert_eq!(wait, Duration::from_secs(30));
}

#[test]
fn managed_checkout_lock_recognizes_descriptor_pressure_errors() {
    let error = std::io::Error::from_raw_os_error(24);
    assert!(is_descriptor_pressure_error(&error));
}
