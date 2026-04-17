use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::json;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tokio::sync::mpsc;
use tokio::time::timeout;
use xiuxian_daochang::{
    AgendaViewTool, JournalRecordTool, NotificationDispatcher, NotificationProvider, TaskAddTool,
};
use xiuxian_daochang::{NativeTool, NativeToolCallContext};
use xiuxian_qianhuan::{
    ManifestationInterface, ManifestationManager, MemoryPersonaRecord, MemoryTemplateRecord,
    PersonaRegistry,
};
use xiuxian_wendao::enhancer::{MarkdownConfigBlock, extract_markdown_config_blocks};
use xiuxian_wendao::entity::{Entity, EntityType};
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
    ZhixingHeyi, storage::MarkdownStorage,
};

fn build_manifestation_manager()
-> std::result::Result<ManifestationManager, Box<dyn std::error::Error>> {
    Ok(ManifestationManager::new_with_embedded_templates(
        &[],
        &[
            (
                "task_add_response.md",
                "Mock Manifestation Content -> {{ task_title }} :: {{ task_id }}",
            ),
            ("daily_agenda.md", "Agenda Template"),
            ("reminder_notice.md", "{{ task_title_mdv2 }}"),
        ],
    )?)
}

fn render_task_add_response(
    manifestation: &dyn ManifestationInterface,
    task: &Entity,
) -> anyhow::Result<String> {
    manifestation.render_template(
        "task_add_response.md",
        json!({
            "task_title": task.name,
            "task_detail": task.description,
            "task_id": task.id,
            "scheduled_local": task
                .metadata
                .get(ATTR_TIMER_SCHEDULED)
                .and_then(serde_json::Value::as_str),
            "reminder_lead_minutes": 10,
            "qianhuan": {
                "persona": {
                    "name": "Mock Persona",
                }
            },
        }),
    )
}

fn build_heyi_with_time_zone(
    time_zone: &str,
) -> std::result::Result<(Arc<ZhixingHeyi>, tempfile::TempDir), Box<dyn std::error::Error>> {
    let graph = Arc::new(KnowledgeGraph::new());
    let tmp = tempdir()?;
    let storage = Arc::new(MarkdownStorage::new(tmp.path().to_path_buf()));
    let manifestation = Arc::new(build_manifestation_manager()?);
    let heyi = ZhixingHeyi::new(
        graph,
        manifestation,
        storage,
        "host-e2e".to_string(),
        time_zone,
    )?;
    Ok((Arc::new(heyi), tmp))
}

fn build_heyi()
-> std::result::Result<(Arc<ZhixingHeyi>, tempfile::TempDir), Box<dyn std::error::Error>> {
    build_heyi_with_time_zone("UTC")
}

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
    let result = tool
        .call(
            Some(json!({"title": "This should be blocked"})),
            &NativeToolCallContext::default(),
        )
        .await;
    assert!(result.is_err());
    let error = match result {
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
    let result = tool.call(None, &NativeToolCallContext::default()).await;
    assert!(result.is_err());
    let error = match result {
        Ok(value) => panic!("strict teacher should block agenda.view, got: {value}"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("Heart-Demons"),
        "strict teacher error should include blocker hint"
    );
    Ok(())
}

#[tokio::test]
async fn journal_record_tool_succeeds_in_host_tool_path()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;
    let tool = JournalRecordTool {
        heyi: Arc::clone(&heyi),
    };
    let result = tool
        .call(
            Some(json!({"content": "Today I reviewed execution discipline."})),
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
async fn task_add_tool_binds_recipient_from_session_context()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (heyi, _tmp) = build_heyi()?;
    let tool = TaskAddTool {
        heyi: Arc::clone(&heyi),
    };

    tool.call(
        Some(json!({
            "title": "Session-bound reminder task",
            "scheduled_at": (Utc::now() + Duration::minutes(20)).to_rfc3339(),
        })),
        &NativeToolCallContext {
            session_id: Some("telegram:1304799691".to_string()),
            tool_call_id: None,
        },
    )
    .await?;

    let tasks = heyi.graph.get_entities_by_type("OTHER(Task)");
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

    let response = tool
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

    let tasks = heyi.graph.get_entities_by_type("OTHER(Task)");
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

struct MockNotificationProvider {
    sent: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl NotificationProvider for MockNotificationProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn supports(&self, recipient: &str) -> bool {
        recipient == "llm:test"
    }

    async fn send(&self, _recipient: &str, content: &str) -> anyhow::Result<()> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(content.to_string());
        Ok(())
    }
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
        json!((Utc::now() + Duration::minutes(10)).to_rfc3339()),
    );
    scheduled
        .metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), json!(false));
    scheduled
        .metadata
        .insert(ATTR_TIMER_RECIPIENT.to_string(), json!("llm:test"));
    heyi.graph.add_entity(scheduled)?;

    let (tx, mut rx) = mpsc::channel(8);
    let watcher = Arc::clone(&heyi).start_timer_watcher(tx);
    let Some(reminder_signal) = timeout(std::time::Duration::from_secs(2), rx.recv()).await? else {
        return Err(std::io::Error::other("watcher should publish reminder").into());
    };
    watcher.abort();

    let sent = Arc::new(Mutex::new(Vec::new()));
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

#[tokio::test]
async fn task_add_render_uses_hot_reloaded_manifestation_template()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let template_tmp = tempdir()?;
    let template_path = template_tmp.path().join("task_add_response.md");
    fs::write(&template_path, "Template v1 -> {{ task_title }}")?;

    let template_glob = format!("{}/*.md", template_tmp.path().display());
    let manifestation = ManifestationManager::new(&[template_glob.as_str()])?;

    let task_id = "task:hot-reload-confirmation";
    let task = Entity::new(
        task_id.to_string(),
        "Hot Reload Task".to_string(),
        EntityType::Other("Task".to_string()),
        "Verify manifestation template reload path".to_string(),
    );

    let first = render_task_add_response(&manifestation, &task)?;
    assert!(
        first.contains("Template v1 -> Hot Reload Task"),
        "expected v1 template output, got: {first}"
    );

    fs::write(&template_path, "Template v2 -> {{ task_title }}")?;
    let second = render_task_add_response(&manifestation, &task)?;
    assert!(
        second.contains("Template v2 -> Hot Reload Task"),
        "expected v2 template output without restart, got: {second}"
    );

    Ok(())
}

#[tokio::test]
async fn task_add_render_supports_markdown_ast_memory_bridge()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let markdown = r#"
## Persona: Agenda Steward
<!-- id: "agenda_steward", type: "persona" -->

```toml
name = "Agenda Steward"
voice_tone = "Structured and practical."
style_anchors = ["agenda", "clarity"]
cot_template = "Observe -> draft -> validate"
forbidden_words = ["impossible"]
```

## Template: Task Add Response
<!-- id: "task_add_response.j2", type: "template", target: "task_add_response.md" -->

```jinja2
Markdown Bridge Template -> {{ task_title }} :: {{ task_id }}
```
"#;

    let blocks = extract_markdown_config_blocks(markdown);

    let mut registry = PersonaRegistry::new();
    let loaded_personas = registry.load_from_memory_records(persona_records(&blocks))?;
    assert_eq!(loaded_personas, 1);
    assert!(registry.get("agenda_steward").is_some());

    let manifestation_manager = ManifestationManager::new_with_embedded_templates(&[], &[])?;
    let loaded_templates =
        manifestation_manager.load_templates_from_memory(template_records(&blocks))?;
    assert_eq!(loaded_templates, 2);

    let task_id = "task:markdown-bridge";
    let task = Entity::new(
        task_id.to_string(),
        "Bridge Render Task".to_string(),
        EntityType::Other("Task".to_string()),
        "Verify markdown AST memory bridge".to_string(),
    );

    let rendered = render_task_add_response(&manifestation_manager, &task)?;
    assert!(rendered.contains("Markdown Bridge Template"));
    assert!(rendered.contains("Bridge Render Task"));
    assert!(rendered.contains(task_id));
    Ok(())
}

fn template_records(blocks: &[MarkdownConfigBlock]) -> Vec<MemoryTemplateRecord> {
    blocks
        .iter()
        .filter(|block| block.config_type.eq_ignore_ascii_case("template"))
        .map(|block| {
            MemoryTemplateRecord::new(
                block.id.clone(),
                block.target.clone(),
                block.content.clone(),
            )
        })
        .collect()
}

fn persona_records(blocks: &[MarkdownConfigBlock]) -> Vec<MemoryPersonaRecord> {
    blocks
        .iter()
        .filter(|block| block.config_type.eq_ignore_ascii_case("persona"))
        .map(|block| MemoryPersonaRecord::new(block.id.clone(), block.content.clone()))
        .collect()
}
