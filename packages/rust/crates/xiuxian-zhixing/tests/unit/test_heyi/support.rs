pub(super) use chrono::{Duration, Utc};
pub(super) use serde_json::{Value, json};
pub(super) use std::error::Error;
pub(super) use std::fs;
pub(super) use std::sync::Arc;
pub(super) use tempfile::{TempDir, tempdir};
pub(super) use xiuxian_qianhuan::ManifestationInterface;
pub(super) use xiuxian_wendao::entity::{Entity, EntityType};
pub(super) use xiuxian_wendao::graph::KnowledgeGraph;
pub(super) use xiuxian_zhixing::storage::MarkdownStorage;
pub(super) use xiuxian_zhixing::{
    ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, ReminderSignal, ZhixingHeyi, ZhixingHeyiInit,
};

pub(super) type TestResult = std::result::Result<(), Box<dyn Error>>;

pub(super) struct EchoManifestation;

impl ManifestationInterface for EchoManifestation {
    fn render_template(
        &self,
        _template_name: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<String> {
        Ok(data.to_string())
    }

    fn inject_context(&self, state_context: &str) -> String {
        state_context.to_string()
    }
}

pub(super) struct TestContext {
    pub(super) graph: Arc<KnowledgeGraph>,
    pub(super) temp_dir: TempDir,
    pub(super) heyi: ZhixingHeyi,
}

pub(super) fn context(time_zone: &str) -> std::result::Result<TestContext, Box<dyn Error>> {
    let graph = Arc::new(KnowledgeGraph::new());
    let temp_dir = tempdir()?;
    let storage = Arc::new(MarkdownStorage::new(temp_dir.path().to_path_buf()));
    let manifestation = Arc::new(EchoManifestation);
    let heyi = ZhixingHeyi::new(ZhixingHeyiInit {
        graph: Arc::clone(&graph),
        manifestation,
        storage,
        scope_key: "test".to_string(),
        time_zone_str: time_zone.to_string(),
    })?;

    Ok(TestContext {
        graph,
        temp_dir,
        heyi,
    })
}
