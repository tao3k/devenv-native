use super::support::{
    ATTR_TIMER_RECIPIENT, ATTR_TIMER_SCHEDULED, Arc, ChronoDuration, Entity, EntityType,
    NativeTool, NativeToolCallContext, TaskAddTool, Utc, build_heyi, build_heyi_with_time_zone,
    build_manifestation_manager, json, render_task_add_response,
};

#[tokio::test]
async fn task_add_tool_binds_recipient_from_session_context()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;
    let tool = TaskAddTool {
        heyi: Arc::clone(&heyi),
    };

    tool.call(
        Some(json!({
            "title": "Session-bound reminder task",
            "scheduled_at": (Utc::now() + ChronoDuration::minutes(20)).to_rfc3339(),
        })),
        &NativeToolCallContext {
            session_id: Some("telegram:1304799691".to_string()),
            tool_call_id: None,
        },
    )
    .await?;

    let tasks = heyi.graph.get_entities_by_type("TASK");
    let has_recipient = tasks.iter().any(|task| {
        task.metadata
            .get(ATTR_TIMER_RECIPIENT)
            .and_then(serde_json::Value::as_str)
            == Some("telegram:1304799691")
    });
    assert!(
        has_recipient,
        "task metadata should include reminder recipient"
    );
    Ok(())
}

#[tokio::test]
async fn task_add_tool_normalizes_human_local_time_input()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi_with_time_zone("America/Los_Angeles")?;
    let tool = TaskAddTool {
        heyi: Arc::clone(&heyi),
    };

    let response: String = tool
        .call(
            Some(json!({
                "title": "Local human time task",
                "time": "2026-02-25 10:09 PM",
            })),
            &NativeToolCallContext {
                session_id: Some("telegram:1304799691".to_string()),
                tool_call_id: None,
            },
        )
        .await?;

    let tasks = heyi.graph.get_entities_by_type("TASK");
    let has_expected_schedule = tasks.iter().any(|task| {
        task.metadata
            .get(ATTR_TIMER_SCHEDULED)
            .and_then(serde_json::Value::as_str)
            == Some("2026-02-26T06:09:00+00:00")
    });
    assert!(
        has_expected_schedule,
        "task metadata should store normalized UTC RFC3339 schedule"
    );
    assert!(
        !response.trim().is_empty(),
        "task.add should return a non-empty response"
    );
    Ok(())
}

#[tokio::test]
async fn task_add_confirmation_can_be_rendered_from_task_id()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let manifestation = build_manifestation_manager()?;
    let task_id = "task:render-confirmation";
    let mut task = Entity::new(
        task_id.to_string(),
        "验证知行提醒模板".to_string(),
        EntityType::Other("Task".to_string()),
        "检查角色注入文案是否出现并且可读".to_string(),
    );
    task.metadata.insert(
        ATTR_TIMER_SCHEDULED.to_string(),
        json!("2026-02-26T08:50:00+00:00"),
    );
    let rendered = render_task_add_response(&manifestation, &task)?;
    assert!(
        rendered.contains("Mock Manifestation Content"),
        "expected manifestation render output"
    );
    Ok(())
}
