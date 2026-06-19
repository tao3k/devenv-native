use crate::ManifestationInterface;
use crate::storage::MarkdownStorage;
use crate::{
    ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, ReminderSignal, ZhixingHeyiInit, heyi::ZhixingHeyi,
};
use chrono::{Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use xiuxian_wendao::entity::{Entity, EntityType};
use xiuxian_wendao::graph::KnowledgeGraph;

struct EmbeddedManifestation {
    templates: std::collections::HashMap<String, String>,
}

impl EmbeddedManifestation {
    fn new(embedded_templates: &[(&str, &str)]) -> Self {
        Self {
            templates: embedded_templates
                .iter()
                .map(|(name, source)| ((*name).to_string(), (*source).to_string()))
                .collect(),
        }
    }
}

impl ManifestationInterface for EmbeddedManifestation {
    fn render_template(
        &self,
        template_name: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<String> {
        let mut rendered = self
            .templates
            .get(template_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("template `{template_name}` is not registered"))?;
        if let Some(map) = data.as_object() {
            for (key, value) in map {
                let replacement = value
                    .as_str()
                    .map_or_else(|| value.to_string(), ToString::to_string);
                rendered = rendered.replace(format!("{{{{ {key} }}}}").as_str(), &replacement);
            }
        }
        Ok(rendered)
    }

    fn inject_context(&self, state_context: &str) -> String {
        state_context.to_string()
    }
}

fn build_test_heyi(
    embedded_templates: &[(&str, &str)],
) -> Result<ZhixingHeyi, Box<dyn std::error::Error>> {
    let graph = Arc::new(KnowledgeGraph::new());
    let storage_root =
        std::env::temp_dir().join(format!("xiuxian-zhixing-reminders-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&storage_root)?;
    let storage = Arc::new(MarkdownStorage::new(storage_root));
    let manifestation = Arc::new(EmbeddedManifestation::new(embedded_templates));
    Ok(ZhixingHeyi::new(ZhixingHeyiInit {
        graph,
        manifestation,
        storage,
        scope_key: "test-reminder-scope".to_string(),
        time_zone_str: "UTC".to_string(),
    })?)
}

#[test]
fn poll_reminders_carries_task_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let heyi = build_test_heyi(&[("reminder_notice.md", "{{ task_title_mdv2 }}")])?;
    let mut task = Entity::new(
        "task:test-reminder".to_string(),
        "Finish migration batch".to_string(),
        EntityType::Other("Task".to_string()),
        "Close the current bounded remediation bundle".to_string(),
    );
    task.metadata.insert(
        ATTR_TIMER_SCHEDULED.to_string(),
        json!((Utc::now() + Duration::minutes(10)).to_rfc3339()),
    );
    task.metadata
        .insert(ATTR_TIMER_REMINDED.to_string(), json!(false));
    heyi.graph.add_entity(task)?;

    let reminders = heyi.poll_reminders();
    assert_eq!(reminders.len(), 1);
    let reminder = &reminders[0];
    assert_eq!(reminder.task_id, "task:test-reminder");
    assert_eq!(reminder.title, "Finish migration batch");
    assert_eq!(
        reminder.task_brief.as_deref(),
        Some("Close the current bounded remediation bundle")
    );
    assert!(reminder.scheduled_at.is_some());
    Ok(())
}

#[test]
fn render_reminder_notice_uses_live_template_surface() -> Result<(), Box<dyn std::error::Error>> {
    let heyi = build_test_heyi(&[(
        "reminder_notice.md",
        "{{ task_title_mdv2 }}|{{ task_id_mdv2 }}|{{ scheduled_local_mdv2 }}",
    )])?;
    let signal = ReminderSignal {
        task_id: "task:demo".to_string(),
        title: "Prepare [agenda]".to_string(),
        task_brief: Some("Escapes MarkdownV2 fields".to_string()),
        scheduled_at: Some("2026-02-26T08:50:00+00:00".to_string()),
        recipient: Some("llm:test".to_string()),
    };

    let rendered = heyi.render_reminder_notice_markdown(&signal)?;
    assert!(rendered.contains("Prepare \\[agenda\\]"));
    assert!(rendered.contains("task:demo"));
    assert!(rendered.contains("2026\\-02\\-26"));
    Ok(())
}
