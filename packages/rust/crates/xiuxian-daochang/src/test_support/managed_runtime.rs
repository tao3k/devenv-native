//! Managed-runtime helpers exposed for integration tests.

use std::path::Path;

use crate::RuntimeSettings;
use crate::channels::managed_runtime::{session_partition_persistence, turn};

use super::TestSupportResult;

/// Test-facing partition-persistence target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPartitionPersistenceTarget {
    Telegram,
    Discord,
}

/// Persist the selected session-partition mode when persistence is enabled.
///
/// # Errors
///
/// Returns an error when settings load/write fails.
pub fn persist_session_partition_mode_if_enabled(
    target: SessionPartitionPersistenceTarget,
    mode: &str,
) -> TestSupportResult<bool> {
    session_partition_persistence::persist_session_partition_mode_if_enabled(
        to_internal_target(target),
        mode,
    )
}

/// Resolve whether session-partition persistence is enabled for one channel target.
#[must_use]
pub fn resolve_session_partition_persist_enabled<F>(
    target: SessionPartitionPersistenceTarget,
    settings: &RuntimeSettings,
    lookup_env: F,
) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    session_partition_persistence::resolve_session_partition_persist_enabled(
        to_internal_target(target),
        settings,
        lookup_env,
    )
}

/// Persist the selected session-partition mode into a target `xiuxian.toml` path.
///
/// # Errors
///
/// Returns an error when the target path cannot be parsed/serialized/written.
pub fn persist_session_partition_mode_to_path(
    user_settings_path: &Path,
    target: SessionPartitionPersistenceTarget,
    mode: &str,
) -> TestSupportResult<()> {
    session_partition_persistence::persist_session_partition_mode_to_path(
        user_settings_path,
        to_internal_target(target),
        mode,
    )
}

/// Build canonical managed-runtime session id from channel + session key.
#[must_use]
pub fn build_session_id(channel: &str, session_key: &str) -> String {
    turn::build_session_id(channel, session_key)
}

/// Classify one turn-execution error chain into a telemetry bucket.
#[must_use]
pub fn classify_turn_error(error: &str) -> &'static str {
    turn::classify_turn_error(error)
}

const fn to_internal_target(
    target: SessionPartitionPersistenceTarget,
) -> session_partition_persistence::SessionPartitionPersistenceTarget {
    match target {
        SessionPartitionPersistenceTarget::Telegram => {
            session_partition_persistence::SessionPartitionPersistenceTarget::Telegram
        }
        SessionPartitionPersistenceTarget::Discord => {
            session_partition_persistence::SessionPartitionPersistenceTarget::Discord
        }
    }
}
