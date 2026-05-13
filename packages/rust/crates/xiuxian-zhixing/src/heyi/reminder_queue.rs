//! Persistent reminder queue settings and storage helpers.

use super::reminders::ReminderSignal;
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

const DEFAULT_QUEUE_KEY_PREFIX: &str = "xiuxian_zhixing:heyi:reminder";
const DEFAULT_POLL_INTERVAL_SECONDS: u64 = 5;
const DEFAULT_POLL_BATCH_SIZE: usize = 128;
const REMINDER_LEAD_MINUTES: i64 = 15;

/// Runtime settings for Valkey-backed reminder scheduling.
#[derive(Debug, Clone)]
pub struct ReminderQueueSettings {
    /// Valkey connection URL (for example `redis://127.0.0.1:6379/0`).
    pub valkey_url: String,
    /// Namespace prefix for queue keys.
    pub key_prefix: String,
    /// Poll interval used by watcher loop.
    pub poll_interval_seconds: u64,
    /// Maximum due reminders fetched per poll.
    pub poll_batch_size: usize,
}

impl ReminderQueueSettings {
    /// Build settings with defaults for optional fields.
    #[must_use]
    pub fn with_defaults(
        valkey_url: String,
        key_prefix: Option<String>,
        poll_interval_seconds: Option<u64>,
        poll_batch_size: Option<usize>,
    ) -> Self {
        Self {
            valkey_url,
            key_prefix: key_prefix
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| DEFAULT_QUEUE_KEY_PREFIX.to_string()),
            poll_interval_seconds: poll_interval_seconds
                .filter(|value| *value > 0)
                .unwrap_or(DEFAULT_POLL_INTERVAL_SECONDS),
            poll_batch_size: poll_batch_size
                .filter(|value| *value > 0)
                .unwrap_or(DEFAULT_POLL_BATCH_SIZE),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReminderQueuePayload {
    task_id: String,
    title: String,
    task_brief: Option<String>,
    recipient: Option<String>,
    scheduled_at: String,
}

/// Due reminder record used for graph state reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueReminderRecord {
    /// Task entity ID.
    pub task_id: String,
    /// Human-readable task title.
    pub title: String,
    /// Optional task detail/body to clarify intended action.
    pub task_brief: Option<String>,
    /// Delivery target (for example `telegram:1304799691`).
    pub recipient: Option<String>,
    /// Canonical RFC3339 UTC scheduled time.
    pub scheduled_at: String,
}

impl DueReminderRecord {
    /// Convert queue record into external reminder payload.
    #[must_use]
    pub fn into_signal(self, _time_zone: Tz) -> ReminderSignal {
        ReminderSignal {
            task_id: self.task_id,
            title: self.title,
            task_brief: self.task_brief,
            scheduled_at: Some(self.scheduled_at),
            recipient: self.recipient,
        }
    }
}

/// Domain identifier for one scheduled reminder task.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReminderTaskId(String);

impl ReminderTaskId {
    /// Returns the serialized task identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ReminderTaskId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ReminderTaskId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Named request for adding one task to the reminder queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderQueueTask {
    /// Task entity ID.
    pub task_id: ReminderTaskId,
    /// Human-readable task title.
    pub title: String,
    /// Optional task detail/body to clarify intended action.
    pub task_brief: Option<String>,
    /// Scheduled time in RFC3339 form.
    pub scheduled_at_rfc3339: String,
    /// Delivery target (for example `telegram:1304799691`).
    pub recipient: Option<String>,
}

impl ReminderQueueTask {
    /// Creates one reminder queue request with no optional brief or recipient.
    #[must_use]
    pub fn new(
        task_id: impl Into<ReminderTaskId>,
        title: impl Into<String>,
        scheduled_at_rfc3339: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            title: title.into(),
            task_brief: None,
            scheduled_at_rfc3339: scheduled_at_rfc3339.into(),
            recipient: None,
        }
    }

    /// Adds an optional task brief.
    #[must_use]
    pub fn with_task_brief(mut self, task_brief: impl Into<String>) -> Self {
        self.task_brief = Some(task_brief.into());
        self
    }

    /// Adds an optional reminder recipient.
    #[must_use]
    pub fn with_recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipient = Some(recipient.into());
        self
    }
}

/// Valkey-backed reminder due-queue implemented with ZSET + HASH.
#[derive(Debug, Clone)]
pub struct ReminderQueueStore {
    settings: ReminderQueueSettings,
    scope_key: String,
}

impl ReminderQueueStore {
    /// Create a queue store bound to one logical graph scope.
    ///
    /// # Errors
    /// Returns an error when the valkey URL is empty.
    pub fn new(settings: ReminderQueueSettings, scope_key: String) -> Result<Self, String> {
        if settings.valkey_url.trim().is_empty() {
            return Err("reminder queue valkey_url must be non-empty".to_string());
        }
        if scope_key.trim().is_empty() {
            return Err("reminder queue scope_key must be non-empty".to_string());
        }
        Ok(Self {
            settings,
            scope_key,
        })
    }

    /// Returns configured poll interval seconds.
    #[must_use]
    pub fn poll_interval_seconds(&self) -> u64 {
        self.settings.poll_interval_seconds
    }

    /// Enqueue one scheduled task reminder.
    ///
    /// # Errors
    /// Returns an error when schedule parsing or Valkey IO fails.
    pub fn enqueue_task(&self, task: ReminderQueueTask) -> Result<(), String> {
        let task_id = task.task_id.as_str();
        let scheduled_at = DateTime::parse_from_rfc3339(&task.scheduled_at_rfc3339)
            .map_err(|error| format!("invalid scheduled_at rfc3339: {error}"))?
            .with_timezone(&Utc);
        let due_at_unix = (scheduled_at - Duration::minutes(REMINDER_LEAD_MINUTES)).timestamp();

        let payload = ReminderQueuePayload {
            task_id: task_id.to_string(),
            title: task.title,
            task_brief: task
                .task_brief
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            recipient: task.recipient,
            scheduled_at: scheduled_at.to_rfc3339(),
        };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|error| format!("failed to serialize reminder payload: {error}"))?;

        let client = redis::Client::open(self.settings.valkey_url.as_str())
            .map_err(|error| format!("invalid valkey url for reminder queue: {error}"))?;
        let mut conn = client
            .get_connection()
            .map_err(|error| format!("failed to connect valkey for reminder queue: {error}"))?;

        let due_key = self.due_key();
        let payload_key = self.payload_key();
        redis::cmd("ZADD")
            .arg(&due_key)
            .arg(due_at_unix)
            .arg(task_id)
            .query::<i64>(&mut conn)
            .map_err(|error| format!("failed to ZADD reminder queue due key: {error}"))?;
        redis::cmd("HSET")
            .arg(&payload_key)
            .arg(task_id)
            .arg(payload_json)
            .query::<i64>(&mut conn)
            .map_err(|error| format!("failed to HSET reminder queue payload key: {error}"))?;
        Ok(())
    }

    /// Poll due reminders and atomically consume them from queue.
    ///
    /// # Errors
    /// Returns an error when Valkey IO fails.
    pub fn poll_due(&self, now_unix: i64) -> Result<Vec<DueReminderRecord>, String> {
        let client = redis::Client::open(self.settings.valkey_url.as_str())
            .map_err(|error| format!("invalid valkey url for reminder queue: {error}"))?;
        let mut conn = client
            .get_connection()
            .map_err(|error| format!("failed to connect valkey for reminder queue: {error}"))?;

        let due_key = self.due_key();
        let payload_key = self.payload_key();
        let due_task_ids: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&due_key)
            .arg("-inf")
            .arg(now_unix)
            .arg("LIMIT")
            .arg(0)
            .arg(self.settings.poll_batch_size)
            .query(&mut conn)
            .map_err(|error| format!("failed to ZRANGEBYSCORE due reminder queue: {error}"))?;

        if due_task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut reminders = Vec::with_capacity(due_task_ids.len());
        for task_id in &due_task_ids {
            let payload_raw: Option<String> = redis::cmd("HGET")
                .arg(&payload_key)
                .arg(task_id)
                .query(&mut conn)
                .map_err(|error| format!("failed to HGET reminder queue payload: {error}"))?;
            let Some(payload_raw) = payload_raw else {
                continue;
            };
            let payload: ReminderQueuePayload = serde_json::from_str(&payload_raw)
                .map_err(|error| format!("failed to deserialize reminder payload: {error}"))?;
            reminders.push(DueReminderRecord {
                task_id: payload.task_id,
                title: payload.title,
                task_brief: payload.task_brief,
                recipient: payload.recipient,
                scheduled_at: payload.scheduled_at,
            });
        }

        redis::cmd("ZREM")
            .arg(&due_key)
            .arg(&due_task_ids)
            .query::<i64>(&mut conn)
            .map_err(|error| format!("failed to ZREM consumed reminder queue entries: {error}"))?;
        redis::cmd("HDEL")
            .arg(&payload_key)
            .arg(&due_task_ids)
            .query::<i64>(&mut conn)
            .map_err(|error| {
                format!("failed to HDEL consumed reminder payload entries: {error}")
            })?;

        Ok(reminders)
    }

    fn due_key(&self) -> String {
        format!(
            "{}:due:{}",
            self.settings.key_prefix,
            normalize_scope_key(&self.scope_key)
        )
    }

    fn payload_key(&self) -> String {
        format!(
            "{}:payload:{}",
            self.settings.key_prefix,
            normalize_scope_key(&self.scope_key)
        )
    }
}

fn normalize_scope_key(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}
