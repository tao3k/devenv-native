use crate::observability::SessionEvent;
use crate::session::ChatMessage;
use anyhow::Result;

use super::types::{
    SessionContextBackup, SessionContextBackupMetadata, SessionContextSnapshotInfo,
};
use super::{Agent, SessionContextStats, backup_metadata_session_id, now_unix_ms};

impl Agent {
    /// Moves the current session context into the backup slot and clears the
    /// active context window.
    ///
    /// # Errors
    ///
    /// Returns an error when the current session cannot be loaded, the backup
    /// snapshot cannot be persisted, or the active session cannot be cleared.
    pub async fn reset_context_window(
        &self,
        session_id: impl AsRef<str>,
    ) -> Result<SessionContextStats> {
        let session_id = session_id.as_ref();
        let backup = self.capture_session_backup(session_id).await?;
        let stats = backup.stats();
        if backup.is_empty() {
            self.clear_session(session_id).await?;
            return Ok(stats);
        }

        let backup_session_id = super::backup_session_id(session_id);
        self.store_session_backup(&backup_session_id, &backup)
            .await?;
        self.store_backup_metadata(session_id, stats).await?;
        self.clear_session(session_id).await?;
        Ok(stats)
    }

    /// Restores a previously backed-up session context into the active window.
    ///
    /// # Errors
    ///
    /// Returns an error when the backup snapshot cannot be loaded, restored, or
    /// cleaned up.
    pub async fn resume_context_window(
        &self,
        session_id: impl AsRef<str>,
    ) -> Result<Option<SessionContextStats>> {
        let session_id = session_id.as_ref();
        let backup_session_id = super::backup_session_id(session_id);
        let backup = self.capture_session_backup(&backup_session_id).await?;
        if backup.is_empty() {
            self.clear_backup_metadata(session_id).await?;
            return Ok(None);
        }

        let stats = backup.stats();
        self.restore_session_backup(session_id, backup).await?;
        self.clear_session(&backup_session_id).await?;
        self.clear_backup_metadata(session_id).await?;
        Ok(Some(stats))
    }

    /// Returns metadata about the currently stored session-context backup, when
    /// one exists.
    ///
    /// # Errors
    ///
    /// Returns an error when the backup snapshot or its metadata cannot be
    /// loaded.
    pub async fn peek_context_window_backup(
        &self,
        session_id: impl AsRef<str>,
    ) -> Result<Option<SessionContextSnapshotInfo>> {
        let session_id = session_id.as_ref();
        let backup_session_id = super::backup_session_id(session_id);
        let backup = self.capture_session_backup(&backup_session_id).await?;
        let metadata = self.load_backup_metadata(session_id).await?;
        if backup.is_empty() && metadata.is_none() {
            return Ok(None);
        }

        let stats = backup.stats();
        let saved_at_unix_ms = metadata.as_ref().map(|snapshot| snapshot.saved_at_unix_ms);
        let saved_age_secs = saved_at_unix_ms.map(|saved_at| {
            now_unix_ms()
                .saturating_sub(saved_at)
                .checked_div(1000)
                .unwrap_or(0)
        });
        Ok(Some(SessionContextSnapshotInfo {
            messages: metadata
                .as_ref()
                .map_or(stats.messages, |snapshot| snapshot.messages),
            summary_segments: metadata
                .as_ref()
                .map_or(stats.summary_segments, |snapshot| snapshot.summary_segments),
            saved_at_unix_ms,
            saved_age_secs,
        }))
    }

    /// Deletes the stored backup for the given session, if present.
    ///
    /// # Errors
    ///
    /// Returns an error when the backup snapshot or its metadata cannot be
    /// loaded or cleared.
    pub async fn drop_context_window_backup(&self, session_id: impl AsRef<str>) -> Result<bool> {
        let session_id = session_id.as_ref();
        let backup_session_id = super::backup_session_id(session_id);
        let backup = self.capture_session_backup(&backup_session_id).await?;
        let metadata = self.load_backup_metadata(session_id).await?;
        if backup.is_empty() && metadata.is_none() {
            return Ok(false);
        }

        self.clear_session(&backup_session_id).await?;
        self.clear_backup_metadata(session_id).await?;
        Ok(true)
    }

    pub(super) async fn capture_session_backup(
        &self,
        session_id: &str,
    ) -> Result<SessionContextBackup> {
        if let Some(ref w) = self.bounded_session {
            let limit_slots = self
                .config
                .window_max_turns
                .unwrap_or(512)
                .saturating_mul(2);
            let window_slots = w.get_recent_slots(session_id, limit_slots).await?;
            let summary_segments = w
                .get_recent_summary_segments(session_id, self.config.summary_max_segments)
                .await?;
            tracing::debug!(
                event = SessionEvent::ContextBackupCaptured.as_str(),
                session_id,
                messages = window_slots.len(),
                summary_segments = summary_segments.len(),
                backend = "bounded",
                "session context backup captured"
            );
            return Ok(SessionContextBackup {
                messages: Vec::new(),
                summary_segments,
                window_slots,
            });
        }

        let messages = self.session.get(session_id).await?;
        tracing::debug!(
            event = SessionEvent::ContextBackupCaptured.as_str(),
            session_id,
            messages = messages.len(),
            backend = "session-store",
            "session context backup captured"
        );
        Ok(SessionContextBackup {
            messages,
            summary_segments: Vec::new(),
            window_slots: Vec::new(),
        })
    }

    pub(super) async fn store_session_backup(
        &self,
        session_id: &str,
        backup: &SessionContextBackup,
    ) -> Result<()> {
        self.clear_session(session_id).await?;

        if let Some(ref w) = self.bounded_session {
            for segment in &backup.summary_segments {
                w.append_summary_segment(session_id, segment.clone())
                    .await?;
            }
            w.replace_window_slots(session_id, &backup.window_slots)
                .await?;
            return Ok(());
        }

        self.session
            .append(session_id, backup.messages.clone())
            .await
    }

    pub(super) async fn restore_session_backup(
        &self,
        session_id: &str,
        backup: SessionContextBackup,
    ) -> Result<()> {
        self.clear_session(session_id).await?;

        if let Some(ref w) = self.bounded_session {
            for segment in backup.summary_segments {
                w.append_summary_segment(session_id, segment).await?;
            }
            w.replace_window_slots(session_id, &backup.window_slots)
                .await?;
            return Ok(());
        }

        self.session.append(session_id, backup.messages).await
    }

    pub(super) async fn store_backup_metadata(
        &self,
        session_id: &str,
        stats: SessionContextStats,
    ) -> Result<()> {
        let metadata_session_id = backup_metadata_session_id(session_id);
        let metadata = SessionContextBackupMetadata {
            messages: stats.messages,
            summary_segments: stats.summary_segments,
            saved_at_unix_ms: now_unix_ms(),
        };
        let content = serde_json::to_string(&metadata)?;
        self.session.clear(&metadata_session_id).await?;
        self.session
            .append(
                &metadata_session_id,
                vec![ChatMessage {
                    role: "system".to_string(),
                    content: Some(content),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
            )
            .await
    }

    pub(super) async fn load_backup_metadata(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionContextBackupMetadata>> {
        let metadata_session_id = backup_metadata_session_id(session_id);
        let messages = self.session.get(&metadata_session_id).await?;
        let Some(content) = messages
            .into_iter()
            .rev()
            .find_map(|message| message.content)
        else {
            return Ok(None);
        };
        Ok(serde_json::from_str(&content).ok())
    }

    pub(super) async fn clear_backup_metadata(&self, session_id: &str) -> Result<()> {
        self.session
            .clear(&backup_metadata_session_id(session_id))
            .await
    }
}
