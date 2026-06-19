//! Integration tests for strict-teacher blocker behavior.

use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use xiuxian_wendao::entity::{Entity, EntityType};
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_zhixing::ATTR_JOURNAL_CARRYOVER;
use xiuxian_zhixing::storage::MarkdownStorage;
use xiuxian_zhixing::{ManifestationInterface, ZhixingHeyi, ZhixingHeyiInit};

struct EchoManifestation;

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

#[tokio::test]
async fn test_strict_teacher_blocker() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let graph = Arc::new(KnowledgeGraph::new());

    // Add a stale task with carryover = 3
    let mut stale_task = Entity::new(
        "task:stale".to_string(),
        "Stale Task".to_string(),
        EntityType::Other("Task".to_string()),
        "Some description".to_string(),
    );
    stale_task
        .metadata
        .insert(ATTR_JOURNAL_CARRYOVER.to_string(), json!(3));
    graph.add_entity(stale_task)?;

    let tmp = tempdir()?;
    let storage = Arc::new(MarkdownStorage::new(tmp.path().to_path_buf()));
    let manifestation = Arc::new(EchoManifestation);

    let heyi = ZhixingHeyi::new(ZhixingHeyiInit {
        graph: graph.clone(),
        manifestation,
        storage,
        scope_key: "strict-teacher".to_string(),
        time_zone_str: "UTC".to_string(),
    })?;

    // Should be blocked
    let result = heyi.check_heart_demon_blocker();
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Blocked by 1 Heart-Demons"));
    }

    // Strict teacher blocks task creation path.
    let add_result = heyi.add_task("Try to bypass blocker", None).await;
    assert!(add_result.is_err());

    // Strict teacher blocks agenda view path.
    let agenda_result = heyi.render_agenda();
    assert!(agenda_result.is_err());
    Ok(())
}
