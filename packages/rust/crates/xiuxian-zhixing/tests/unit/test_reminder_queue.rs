//! Tests for Valkey-backed reminder due queue.

use chrono::{Duration, Utc};
use xiuxian_zhixing::{ReminderQueueSettings, ReminderQueueStore, ReminderQueueTask};

#[test]
fn reminder_queue_settings_apply_defaults() {
    let settings = ReminderQueueSettings::with_defaults(
        "redis://127.0.0.1:6379/0".to_string(),
        None,
        None,
        None,
    );
    assert_eq!(settings.key_prefix, "xiuxian_zhixing:heyi:reminder");
    assert_eq!(settings.poll_interval_seconds, 5);
    assert_eq!(settings.poll_batch_size, 128);
}

#[test]
fn reminder_queue_store_rejects_empty_valkey_url() {
    let settings = ReminderQueueSettings::with_defaults(String::new(), None, None, None);
    let result = ReminderQueueStore::new(settings, "scope".to_string());
    assert!(result.is_err());
}

#[test]
fn reminder_queue_round_trip_with_live_valkey()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let valkey_url = std::env::var("XIUXIAN_WENDAO_VALKEY_URL")
        .ok()
        .or_else(|| std::env::var("VALKEY_URL").ok());
    let Some(valkey_url) = valkey_url else {
        eprintln!("skip: set XIUXIAN_WENDAO_VALKEY_URL or VALKEY_URL");
        return Ok(());
    };

    let settings = ReminderQueueSettings::with_defaults(
        valkey_url,
        Some(format!(
            "xiuxian_zhixing:test:reminder:{}",
            Utc::now().timestamp()
        )),
        Some(1),
        Some(8),
    );
    let store = ReminderQueueStore::new(settings, format!("scope-{}", Utc::now().timestamp()))?;
    let scheduled_at = (Utc::now() + Duration::minutes(5)).to_rfc3339();
    store.enqueue_task(
        ReminderQueueTask::new("task:test-round-trip", "Round Trip Task", &scheduled_at)
            .with_task_brief("Review overnight agenda and execute first item")
            .with_recipient("llm:test"),
    )?;

    let due = store.poll_due(Utc::now().timestamp())?;
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].task_id, "task:test-round-trip");
    assert_eq!(due[0].title, "Round Trip Task");
    assert_eq!(
        due[0].task_brief.as_deref(),
        Some("Review overnight agenda and execute first item")
    );
    assert_eq!(due[0].recipient.as_deref(), Some("llm:test"));
    assert_eq!(due[0].scheduled_at, scheduled_at);
    Ok(())
}
