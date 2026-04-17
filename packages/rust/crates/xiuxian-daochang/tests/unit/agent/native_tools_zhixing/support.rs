use async_trait::async_trait;
pub(super) use chrono::{Duration as ChronoDuration, Utc};
pub(super) use serde_json::json;
pub(super) use std::fs;
pub(super) use std::sync::Arc;
use std::sync::Mutex;
pub(super) use tempfile::tempdir;
pub(super) use tokio::sync::mpsc;
pub(super) use tokio::time::timeout;
pub(super) use xiuxian_daochang::{
    AgendaViewTool, JournalRecordTool, NativeTool, NativeToolCallContext, NotificationDispatcher,
    NotificationProvider, TaskAddTool,
};
use xiuxian_qianhuan::{ManifestationInterface, MemoryPersonaRecord, MemoryTemplateRecord};
pub(super) use xiuxian_qianhuan::{ManifestationManager, PersonaRegistry};
pub(super) use xiuxian_wendao::enhancer::{MarkdownConfigBlock, extract_markdown_config_blocks};
pub(super) use xiuxian_wendao::entity::{Entity, EntityType};
use xiuxian_wendao::graph::KnowledgeGraph;
pub(super) use xiuxian_zhixing::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
};
use xiuxian_zhixing::{ZhixingHeyi, storage::MarkdownStorage};

pub(super) fn build_manifestation_manager()
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

pub(super) fn render_task_add_response(
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

pub(super) fn build_heyi_with_time_zone(
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

pub(super) fn build_heyi()
-> std::result::Result<(Arc<ZhixingHeyi>, tempfile::TempDir), Box<dyn std::error::Error>> {
    build_heyi_with_time_zone("UTC")
}

pub(super) struct MockNotificationProvider {
    pub(super) sent: Arc<Mutex<Vec<String>>>,
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

pub(super) fn template_records(blocks: &[MarkdownConfigBlock]) -> Vec<MemoryTemplateRecord> {
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

pub(super) fn persona_records(blocks: &[MarkdownConfigBlock]) -> Vec<MemoryPersonaRecord> {
    blocks
        .iter()
        .filter(|block| block.config_type.eq_ignore_ascii_case("persona"))
        .map(|block| MemoryPersonaRecord::new(block.id.clone(), block.content.clone()))
        .collect()
}
