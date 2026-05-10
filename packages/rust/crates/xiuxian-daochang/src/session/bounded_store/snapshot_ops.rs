use anyhow::{Context, Result};

use crate::session::BoundedSessionSnapshotStats;

use super::BoundedSessionStore;

impl BoundedSessionStore {
    /// Atomically reset active bounded-session state into backup keys.
    ///
    /// # Errors
    /// Returns an error when the underlying Valkey atomic reset operation fails.
    pub async fn atomic_reset_snapshot(
        &self,
        session_id: impl AsRef<str>,
        backup_session_id: impl AsRef<str>,
        metadata_session_id: impl AsRef<str>,
        saved_at_unix_ms: u64,
    ) -> Result<Option<BoundedSessionSnapshotStats>> {
        let session_id = session_id.as_ref();
        let backup_session_id = backup_session_id.as_ref();
        let metadata_session_id = metadata_session_id.as_ref();
        let Some(ref redis) = self.redis else {
            return Ok(None);
        };
        let stats = redis
            .atomic_reset_bounded_snapshot(
                session_id,
                backup_session_id,
                metadata_session_id,
                saved_at_unix_ms,
            )
            .await
            .with_context(|| {
                format!("atomic bounded snapshot reset failed for session_id={session_id}")
            })?;
        Ok(Some(stats.into()))
    }

    /// Atomically restore active bounded-session state from backup keys.
    ///
    /// # Errors
    /// Returns an error when the underlying Valkey atomic resume operation fails.
    pub async fn atomic_resume_snapshot(
        &self,
        session_id: impl AsRef<str>,
        backup_session_id: impl AsRef<str>,
        metadata_session_id: impl AsRef<str>,
    ) -> Result<Option<BoundedSessionSnapshotStats>> {
        let session_id = session_id.as_ref();
        let backup_session_id = backup_session_id.as_ref();
        let metadata_session_id = metadata_session_id.as_ref();
        let Some(ref redis) = self.redis else {
            return Ok(None);
        };
        redis
            .atomic_resume_bounded_snapshot(session_id, backup_session_id, metadata_session_id)
            .await
            .map(|stats| stats.map(Into::into))
            .with_context(|| {
                format!("atomic bounded snapshot resume failed for session_id={session_id}")
            })
    }

    /// Atomically delete bounded-session backup keys.
    ///
    /// # Errors
    /// Returns an error when the underlying Valkey atomic drop operation fails.
    pub async fn atomic_drop_snapshot(
        &self,
        backup_session_id: impl AsRef<str>,
        metadata_session_id: impl AsRef<str>,
    ) -> Result<Option<bool>> {
        let backup_session_id = backup_session_id.as_ref();
        let metadata_session_id = metadata_session_id.as_ref();
        let Some(ref redis) = self.redis else {
            return Ok(None);
        };
        let dropped = redis
            .atomic_drop_bounded_snapshot(backup_session_id, metadata_session_id)
            .await
            .with_context(|| {
                format!(
                    "atomic bounded snapshot drop failed for backup_session_id={backup_session_id}"
                )
            })?;
        Ok(Some(dropped))
    }
}
