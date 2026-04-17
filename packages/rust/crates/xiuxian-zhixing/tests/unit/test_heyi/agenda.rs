use super::support::*;

#[test]
fn test_reminder_trigger_logic() -> TestResult {
    let TestContext {
        graph,
        temp_dir: _temp_dir,
        heyi,
    } = context("UTC")?;

    let scheduled_time = Utc::now() + Duration::minutes(10);
    let mut entity = Entity::new(
        "task:reminder-trigger-logic".to_string(),
        "Trigger Task".to_string(),
        EntityType::Other("Task".to_string()),
        String::new(),
    );
    entity.metadata.insert(
        ATTR_TIMER_SCHEDULED.to_string(),
        json!(scheduled_time.to_rfc3339()),
    );
    entity
        .metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), json!(false));
    graph.add_entity(entity)?;

    let reminders = heyi.poll_reminders();
    assert_eq!(reminders.len(), 1);
    assert_eq!(reminders[0].title, "Trigger Task");
    assert_eq!(reminders[0].recipient, None);

    let reminders_second = heyi.poll_reminders();
    assert_eq!(reminders_second.len(), 0);
    Ok(())
}

#[test]
fn test_render_agenda_uses_human_local_time() -> TestResult {
    let TestContext {
        graph,
        temp_dir: _temp_dir,
        heyi,
    } = context("America/Los_Angeles")?;

    let mut entity = Entity::new(
        "task:render-human-time".to_string(),
        "Render Human Time Task".to_string(),
        EntityType::Other("Task".to_string()),
        String::new(),
    );
    entity.metadata.insert(
        ATTR_TIMER_SCHEDULED.to_string(),
        json!("2026-02-26T06:09:00+00:00"),
    );
    entity
        .metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), json!(false));
    graph.add_entity(entity)?;

    let rendered = heyi.render_agenda()?;
    assert!(
        rendered.contains("2026-02-25 10:09 PM"),
        "agenda output should render local human time: {rendered}"
    );
    assert!(
        !rendered.contains("2026-02-26T06:09:00+00:00"),
        "agenda output should not leak raw RFC3339 metadata: {rendered}"
    );
    Ok(())
}

#[test]
fn test_render_agenda_prefers_today_journal_note_from_wendao() -> TestResult {
    let TestContext {
        graph: _graph,
        temp_dir,
        heyi,
    } = context("America/Los_Angeles")?;

    let local_date = Utc::now()
        .with_timezone(&heyi.time_zone)
        .format("%Y-%m-%d")
        .to_string();
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;
    let note_rel_path = format!("journal/{local_date}.md");
    let note_path = temp_dir.path().join(&note_rel_path);
    fs::write(
        &note_path,
        "## [21:11:15] Reflection\n检查timer通知\n<!-- id: test, tags: [] -->\n",
    )?;

    let rendered = heyi.render_agenda()?;
    assert!(
        rendered.contains(&format!("# Daily Agenda ({local_date})")),
        "agenda output should include local-date agenda heading: {rendered}"
    );
    assert!(
        !rendered.contains("Semantic query:"),
        "agenda output should not leak internal search diagnostics: {rendered}"
    );
    assert!(
        rendered.contains("检查timer通知"),
        "agenda output should come from Wendao hit note content: {rendered}"
    );
    assert!(
        !rendered.contains("<!-- id: test, tags: [] -->"),
        "agenda output should not expose html metadata comments: {rendered}"
    );
    assert!(
        !rendered.contains(&note_rel_path),
        "agenda output should not expose note source path: {rendered}"
    );
    Ok(())
}

#[test]
fn test_render_reminder_notice_markdown_uses_live_signal_fields() -> TestResult {
    let TestContext {
        graph: _graph,
        temp_dir: _temp_dir,
        heyi,
    } = context("America/Los_Angeles")?;

    let rendered = heyi.render_reminder_notice_markdown(&ReminderSignal {
        task_id: "task:render-from-id".to_string(),
        title: "验证知行提醒模板".to_string(),
        task_brief: Some("检查角色注入文案是否出现并且可读".to_string()),
        scheduled_at: Some("2026-02-26T08:50:00+00:00".to_string()),
        recipient: Some("llm:test".to_string()),
    })?;
    let payload: Value = serde_json::from_str(&rendered)?;
    assert_eq!(payload["task_title_mdv2"], json!("验证知行提醒模板"));
    assert_eq!(
        payload["task_brief_mdv2"],
        json!("检查角色注入文案是否出现并且可读")
    );
    assert_eq!(payload["task_id_mdv2"], json!("task:render-from-id"));
    assert_eq!(
        payload["qianhuan"]["state_context"],
        json!("SUCCESS_STREAK")
    );
    assert!(
        payload["scheduled_local_mdv2"]
            .as_str()
            .is_some_and(|value| value.contains("12:50 AM PST")),
        "expected local time in rendered payload: {payload}"
    );
    Ok(())
}
