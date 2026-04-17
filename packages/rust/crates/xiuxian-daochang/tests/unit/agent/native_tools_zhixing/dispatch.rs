use super::support::{
    ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, Arc, ChronoDuration, Entity,
    EntityType, JournalRecordTool, MockNotificationProvider, NativeTool, NativeToolCallContext,
    NotificationDispatcher, Utc, build_heyi, mpsc, timeout,
};

#[tokio::test]
async fn journal_record_tool_succeeds_in_host_tool_path()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;
    let tool = JournalRecordTool {
        heyi: Arc::clone(&heyi),
    };
    let result: String = tool
        .call(
            Some(serde_json::json!({"content": "Today I reviewed execution discipline."})),
            &NativeToolCallContext::default(),
        )
        .await?;

    assert!(
        !result.trim().is_empty(),
        "journal.record should return a non-empty response"
    );
    Ok(())
}

#[tokio::test]
async fn reminder_signal_flows_to_host_dispatcher()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;

    let mut scheduled = Entity::new(
        "task:host-reminder".to_string(),
        "Host Reminder Task".to_string(),
        EntityType::Other("Task".to_string()),
        "scheduled".to_string(),
    );
    scheduled.metadata.insert(
        ATTR_TIMER_SCHEDULED.to_string(),
        serde_json::json!((Utc::now() + ChronoDuration::minutes(10)).to_rfc3339()),
    );
    scheduled
        .metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), serde_json::json!(false));
    scheduled.metadata.insert(
        ATTR_TIMER_RECIPIENT.to_string(),
        serde_json::json!("llm:test"),
    );
    heyi.graph.add_entity(scheduled)?;

    let (tx, mut rx) = mpsc::channel::<xiuxian_zhixing::ReminderSignal>(8);
    let watcher = Arc::clone(&heyi).start_timer_watcher(tx);
    let Some(reminder_signal) = timeout(std::time::Duration::from_secs(2), rx.recv()).await? else {
        return Err(std::io::Error::other("watcher should publish reminder").into());
    };
    watcher.abort();

    let sent = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dispatcher = NotificationDispatcher::new();
    dispatcher
        .register(Arc::new(MockNotificationProvider {
            sent: Arc::clone(&sent),
        }))
        .await;

    let content = format!("⏰ <b>Vajra Reminder:</b> {}", reminder_signal.title);
    let Some(recipient) = reminder_signal.recipient.as_deref() else {
        return Err(std::io::Error::other("recipient should be present for reminder").into());
    };
    let receipt = dispatcher.dispatch(recipient, &content).await?;
    assert_eq!(receipt.provider, "mock");
    let sent_messages = sent
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(sent_messages.len(), 1);
    assert_eq!(sent_messages[0], content);
    Ok(())
}
