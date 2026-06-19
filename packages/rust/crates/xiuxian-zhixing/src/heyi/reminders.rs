//! Reminder polling and metadata update behavior for Zhixing-Heyi.

use super::constants::{ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED};
use super::schedule_time::render_scheduled_time_local;
use super::{ReminderQueueTask, ZhixingHeyi};
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use xiuxian_wendao::entity::Entity;

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

struct GraphReminderCandidate {
    signal: ReminderSignal,
    updated_entity: Entity,
}

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

        self.graph
            .get_entities_by_type(TASK_ENTITY_TYPE)
            .into_iter()
            .filter_map(backfill_candidate)
            .try_fold(0usize, |enqueued, task| {
                reminder_queue
                    .enqueue_task(task)
                    .map_err(crate::Error::Internal)?;
                Ok(enqueued + 1)
            })
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

        self.render_with_manifestation_context(
            "reminder_notice.md",
            payload,
            REMINDER_STATE_CONTEXT,
        )
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

        self.poll_graph_reminders()
    }

    fn poll_graph_reminders(&self) -> Vec<ReminderSignal> {
        let now_local = Utc::now().with_timezone(&self.time_zone);
        let candidates = self
            .graph
            .get_entities_by_type(TASK_ENTITY_TYPE)
            .into_iter()
            .filter_map(|entity| graph_reminder_candidate(entity, &now_local, self.time_zone))
            .collect::<Vec<_>>();
        self.persist_graph_reminder_updates(
            candidates
                .iter()
                .map(|candidate| candidate.updated_entity.clone()),
        );
        candidates
            .into_iter()
            .map(|candidate| candidate.signal)
            .collect()
    }

    fn persist_graph_reminder_updates(&self, updates: impl IntoIterator<Item = Entity>) {
        updates.into_iter().for_each(|updated| {
            if let Err(error) = self.graph.add_entity(updated) {
                log::warn!("Failed to update reminder state in graph: {error}");
            }
        });
    }
}

fn graph_reminder_candidate(
    entity: Entity,
    now_local: &DateTime<Tz>,
    time_zone: Tz,
) -> Option<GraphReminderCandidate> {
    let scheduled = entity
        .metadata
        .get(ATTR_TIMER_SCHEDULED)
        .and_then(serde_json::Value::as_str)?
        .to_string();
    let reminded = entity
        .metadata
        .get(ATTR_TIMER_REMINDED)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if reminded {
        return None;
    }

    let scheduled_at_utc = DateTime::parse_from_rfc3339(&scheduled).ok()?;
    let scheduled_local = scheduled_at_utc.with_timezone(&time_zone);
    if !is_inside_reminder_window(now_local, &scheduled_local) {
        return None;
    }

    let signal = ReminderSignal {
        task_id: entity.id.clone(),
        title: entity.name.clone(),
        task_brief: (!entity.description.trim().is_empty()).then(|| entity.description.clone()),
        scheduled_at: Some(scheduled),
        recipient: entity
            .metadata
            .get(ATTR_TIMER_RECIPIENT)
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
    };
    let mut updated_entity = entity;
    updated_entity
        .metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), json!(true));
    Some(GraphReminderCandidate {
        signal,
        updated_entity,
    })
}

fn is_inside_reminder_window(now_local: &DateTime<Tz>, scheduled_local: &DateTime<Tz>) -> bool {
    let reminder_window_start = *scheduled_local - Duration::minutes(15);
    now_local >= &reminder_window_start && now_local < scheduled_local
}

fn backfill_candidate(entity: Entity) -> Option<ReminderQueueTask> {
    let scheduled_at = entity
        .metadata
        .get(ATTR_TIMER_SCHEDULED)
        .and_then(serde_json::Value::as_str)?
        .to_string();
    let reminded = entity
        .metadata
        .get(ATTR_TIMER_REMINDED)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if reminded {
        return None;
    }

    let task_brief = entity.description.trim();
    let recipient = entity
        .metadata
        .get(ATTR_TIMER_RECIPIENT)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);

    let mut task = ReminderQueueTask::new(entity.id, entity.name, scheduled_at);
    if !task_brief.is_empty() {
        task = task.with_task_brief(task_brief);
    }
    if let Some(recipient) = recipient {
        task = task.with_recipient(recipient);
    }
    Some(task)
}

#[cfg(test)]
#[path = "../../tests/unit/heyi/reminders.rs"]
mod tests;
