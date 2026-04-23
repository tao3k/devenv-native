use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use super::{
    remote_operation_timeout_with_lookup, run_interruptible_remote_operation_with_timeout,
};
use crate::backend::BackendError;
use crate::backend::gix::tuning::default_remote_operation_timeout_for_parallelism;

#[test]
fn remote_operation_timeout_defaults_when_env_is_missing() {
    assert_eq!(
        remote_operation_timeout_with_lookup(&|_| None),
        default_remote_operation_timeout_for_parallelism(
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        )
    );
}

#[test]
fn remote_operation_timeout_uses_positive_override() {
    let timeout = remote_operation_timeout_with_lookup(&|key| {
        (key == "XIUXIAN_GIT_REPO_REMOTE_OPERATION_TIMEOUT_SECS").then(|| "12".to_string())
    });
    assert_eq!(timeout.as_secs(), 12);
}

#[test]
fn remote_operation_timeout_ignores_invalid_override() {
    let timeout = remote_operation_timeout_with_lookup(&|key| {
        (key == "XIUXIAN_GIT_REPO_REMOTE_OPERATION_TIMEOUT_SECS").then(|| "invalid".to_string())
    });
    assert_eq!(
        timeout,
        default_remote_operation_timeout_for_parallelism(
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        )
    );
}

#[test]
fn interruptible_remote_operation_returns_immediately_on_fast_success_path() {
    let started_at = Instant::now();
    let result = run_interruptible_remote_operation_with_timeout(
        "fetch origin",
        Duration::from_secs(1),
        |_should_interrupt| Ok::<usize, BackendError>(7),
    );
    let elapsed = started_at.elapsed();

    assert_eq!(result.ok(), Some(7));
    assert!(
        elapsed < Duration::from_millis(250),
        "fast success path should not wait on the full timeout budget, elapsed={elapsed:?}"
    );
}

#[test]
fn interruptible_remote_operation_reports_timeout_when_watchdog_fires() {
    let result = run_interruptible_remote_operation_with_timeout(
        "fetch origin",
        Duration::from_millis(20),
        |should_interrupt| {
            while !should_interrupt.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(5));
            }
            Err::<(), BackendError>(BackendError::new("operation interrupted"))
        },
    );
    let result = match result {
        Ok(()) => panic!("expected timeout failure"),
        Err(error) => error,
    };

    assert!(result.message().contains("fetch origin"));
    assert!(result.message().contains("timed out"));
}

#[test]
fn interruptible_remote_operation_preserves_non_timeout_errors() {
    let result = run_interruptible_remote_operation_with_timeout(
        "probe remote",
        Duration::from_secs(1),
        |_should_interrupt| Err::<(), BackendError>(BackendError::new("authentication failed")),
    );
    let result = match result {
        Ok(()) => panic!("expected authentication failure"),
        Err(error) => error,
    };

    assert_eq!(result.message(), "authentication failed");
}
