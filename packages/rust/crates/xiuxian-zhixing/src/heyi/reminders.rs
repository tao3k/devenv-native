use super::ZhixingHeyi;
use super::constants::{ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED};
use super::schedule_time::render_scheduled_time_local;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

/// Notification payload emitted by the timer watcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderSignal {
    /// Task entity ID used for deterministic reopening.
    pub task_id: String,
    /// Task title rendered in reminder message.
    pub title: String,
    /// Optional task detail/body to clarify intended action.
    pub task_brief: Option<String>,
    /// Canonical RFC3339 UTC scheduled time.
    pub scheduled_at: Option<String>,
    /// Delivery target (for example `telegram:1304799691`).
    pub recipient: Option<String>,
}

const REMINDER_STATE_CONTEXT: &str = "SUCCESS_STREAK";
const DEFAULT_PERSONA_NAME: &str = "Agenda Steward";
const TASK_ENTITY_TYPE: &str = "TASK";

fn escape_markdown_v2_text(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut escaped, ch| {
            match ch {
                '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '='
                | '|' | '{' | '}' | '.' | '!' | '\\' => {
                    escaped.push('\\');
                    escaped.push(ch);
                }
                _ => escaped.push(ch),
            }
            escaped
        })
}

fn escape_markdown_v2_code(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut escaped, ch| {
            if ch == '\\' || ch == '`' {
                escaped.push('\\');
            }
            escaped.push(ch);
            escaped
        })
}

impl ZhixingHeyi {
    /// Starts the background timer watcher to proactively monitor scheduled tasks.
    /// This fully encapsulates the domain logic of Agenda/Journal timeouts
    /// and uses an abstract channel to push notifications back to the host system.
    #[must_use]
    pub fn start_timer_watcher(
        self: Arc<Self>,
        notifier: Sender<ReminderSignal>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_mins(1));
            loop {
                interval.tick().await;
                let reminders = self.poll_reminders();
                for title in reminders {
                    if notifier.send(title).await.is_err() {
                        log::warn!("Timer watcher notification channel closed, stopping watcher.");
                        break;
                    }
                }
            }
        })
    }

    fn mark_reminders_delivered(&self, task_ids: &[String]) {
        for task_id in task_ids {
            let Some(mut entity) = self.graph.get_entity(task_id) else {
                continue;
            };
            entity
                .metadata
                .insert(ATTR_TIMER_REMINDED.to_string(), json!(true));
            if let Err(error) = self.graph.add_entity(entity) {
                log::warn!("Failed to update reminder state in graph: {error}");
            }
        }
    }

    fn poll_due_queue_reminders(&self) -> Option<Vec<ReminderSignal>> {
        let reminder_queue = self.reminder_queue.as_ref()?;
        match reminder_queue.poll_due(Utc::now().timestamp()) {
            Ok(due_records) => {
                let task_ids = due_records
                    .iter()
                    .map(|record| record.task_id.clone())
                    .collect::<Vec<_>>();
                if !task_ids.is_empty() {
                    self.mark_reminders_delivered(&task_ids);
                }
                Some(
                    due_records
                        .into_iter()
                        .map(|record| record.into_signal(self.time_zone))
                        .collect(),
                )
            }
            Err(error) => {
                log::warn!("Failed to poll reminder due queue: {error}");
                None
            }
        }
    }

    /// Enqueue existing scheduled tasks into the optional due queue backend.
    ///
    /// # Errors
    /// Returns an error when queue IO fails.
    pub fn backfill_due_reminders(&self) -> crate::Result<usize> {
        let Some(reminder_queue) = self.reminder_queue.as_ref() else {
            return Ok(0);
        };

        let mut enqueued = 0usize;
        for entity in self.graph.get_entities_by_type(TASK_ENTITY_TYPE) {
            let Some(scheduled_at) = entity
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let reminded = entity
                .metadata
                .get(ATTR_TIMER_REMINDED)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            if reminded {
                continue;
            }
            let recipient = entity
                .metadata
                .get(ATTR_TIMER_RECIPIENT)
                .and_then(serde_json::Value::as_str);
            let task_brief = entity.description.trim();
            reminder_queue
                .enqueue_task(
                    &entity.id,
                    &entity.name,
                    (!task_brief.is_empty()).then_some(task_brief),
                    scheduled_at,
                    recipient,
                )
                .map_err(crate::Error::Internal)?;
            enqueued += 1;
        }

        Ok(enqueued)
    }

    /// Render a reminder notice using the live Zhixing manifestation template surface.
    ///
    /// # Errors
    /// Returns an error when template rendering fails.
    pub fn render_reminder_notice_markdown(
        &self,
        signal: &ReminderSignal,
    ) -> crate::Result<String> {
        let persona_name = self
            .active_persona
            .as_ref()
            .map_or(DEFAULT_PERSONA_NAME, |persona| persona.name.as_str());
        let scheduled_local = signal
            .scheduled_at
            .as_deref()
            .map(|value| render_scheduled_time_local(value, self.time_zone));
        let payload = json!({
            "persona_name_mdv2": escape_markdown_v2_text(persona_name),
            "task_title_mdv2": escape_markdown_v2_text(&signal.title),
            "task_brief_mdv2": signal
                .task_brief
                .as_deref()
                .map(escape_markdown_v2_text),
            "scheduled_local_mdv2": scheduled_local
                .as_deref()
                .map(escape_markdown_v2_text),
            "task_id_mdv2": escape_markdown_v2_code(&signal.task_id),
        });

        self.render_with_qianhuan_context("reminder_notice.md", payload, REMINDER_STATE_CONTEXT)
    }

    /// Checks for tasks that need immediate reminders in local time.
    ///
    /// Tasks scheduled within the next 15 minutes are returned once and then
    /// marked with `timer:reminded=true`.
    #[must_use]
    pub fn poll_reminders(&self) -> Vec<ReminderSignal> {
        if let Some(reminders) = self.poll_due_queue_reminders() {
            return reminders;
        }

        let tasks = self.graph.get_entities_by_type(TASK_ENTITY_TYPE);
        let now_local = Utc::now().with_timezone(&self.time_zone);
        let mut reminders = Vec::new();
        let mut pending_updates = Vec::new();

        for entity in tasks {
            let scheduled = entity
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str);
            let reminded = entity
                .metadata
                .get(ATTR_TIMER_REMINDED)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let recipient = entity
                .metadata
                .get(ATTR_TIMER_RECIPIENT)
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string);

            let Some(scheduled) = scheduled else {
                continue;
            };
            let Ok(scheduled_at_utc) = DateTime::parse_from_rfc3339(scheduled) else {
                continue;
            };

            let scheduled_local = scheduled_at_utc.with_timezone(&self.time_zone);
            let reminder_window_start = scheduled_local - Duration::minutes(15);
            if !reminded && now_local >= reminder_window_start && now_local < scheduled_local {
                reminders.push(ReminderSignal {
                    task_id: entity.id.clone(),
                    title: entity.name.clone(),
                    task_brief: (!entity.description.trim().is_empty())
                        .then(|| entity.description.clone()),
                    scheduled_at: Some(scheduled.to_string()),
                    recipient,
                });
                let mut updated = entity.clone();
                updated
                    .metadata
                    .insert(ATTR_TIMER_REMINDED.to_string(), json!(true));
                pending_updates.push(updated);
            }
        }

        for updated in pending_updates {
            if let Err(error) = self.graph.add_entity(updated) {
                log::warn!("Failed to update reminder state in graph: {error}");
            }
        }

        reminders
    }
}

#[cfg(test)]
#[path = "../../tests/unit/heyi/reminders.rs"]
mod tests;
