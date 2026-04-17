use super::support::{
    ATTR_JOURNAL_CARRYOVER, AgendaViewTool, Arc, Entity, EntityType, NativeTool,
    NativeToolCallContext, TaskAddTool, build_heyi, json,
};

#[tokio::test]
async fn task_add_tool_respects_strict_teacher_blocker()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;
    let mut stale_task = Entity::new(
        "task:stale-host".to_string(),
        "Stale Host Task".to_string(),
        EntityType::Other("Task".to_string()),
        "stale".to_string(),
    );
    stale_task
        .metadata
        .insert(ATTR_JOURNAL_CARRYOVER.to_string(), json!(3));
    heyi.graph.add_entity(stale_task)?;

    let tool = TaskAddTool {
        heyi: Arc::clone(&heyi),
    };
    let error = match tool
        .call(
            Some(json!({"title": "This should be blocked"})),
            &NativeToolCallContext::default(),
        )
        .await
    {
        Ok(value) => panic!("strict teacher should block task.add, got: {value}"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("Heart-Demons"),
        "strict teacher error should include blocker hint"
    );
    Ok(())
}

#[tokio::test]
async fn agenda_view_tool_respects_strict_teacher_blocker()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;
    let mut stale_task = Entity::new(
        "task:stale-agenda".to_string(),
        "Stale Agenda Task".to_string(),
        EntityType::Other("Task".to_string()),
        "stale".to_string(),
    );
    stale_task
        .metadata
        .insert(ATTR_JOURNAL_CARRYOVER.to_string(), json!(3));
    heyi.graph.add_entity(stale_task)?;

    let tool = AgendaViewTool {
        heyi: Arc::clone(&heyi),
    };
    let error = match tool.call(None, &NativeToolCallContext::default()).await {
        Ok(value) => panic!("strict teacher should block agenda.view, got: {value}"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("Heart-Demons"),
        "strict teacher error should include blocker hint"
    );
    Ok(())
}
