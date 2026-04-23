use std::time::Duration;

use super::{
    checkout_lock_retry_delay_with_lookup, default_checkout_lock_retry_delay_for_parallelism,
};

#[test]
fn checkout_lock_retry_delay_scales_with_parallelism() {
    assert_eq!(
        default_checkout_lock_retry_delay_for_parallelism(1),
        Duration::from_millis(50)
    );
    assert_eq!(
        default_checkout_lock_retry_delay_for_parallelism(12),
        Duration::from_millis(100)
    );
    assert_eq!(
        default_checkout_lock_retry_delay_for_parallelism(24),
        Duration::from_millis(150)
    );
}

#[test]
fn checkout_lock_retry_delay_defaults_when_env_is_missing() {
    assert_eq!(
        checkout_lock_retry_delay_with_lookup(&|_| None),
        default_checkout_lock_retry_delay_for_parallelism(
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        )
    );
}

#[test]
fn checkout_lock_retry_delay_uses_positive_override() {
    assert_eq!(
        checkout_lock_retry_delay_with_lookup(&|key| {
            (key == "XIUXIAN_GIT_REPO_CHECKOUT_LOCK_RETRY_DELAY_MS").then(|| "125".to_string())
        }),
        Duration::from_millis(125)
    );
}

#[test]
fn checkout_lock_retry_delay_ignores_invalid_override() {
    assert_eq!(
        checkout_lock_retry_delay_with_lookup(&|key| {
            (key == "XIUXIAN_GIT_REPO_CHECKOUT_LOCK_RETRY_DELAY_MS").then(|| "invalid".to_string())
        }),
        default_checkout_lock_retry_delay_for_parallelism(
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        )
    );
}
