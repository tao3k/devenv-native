//! Integration tests for strict-teacher blocker behavior.

use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use xiuxian_qianhuan::ManifestationInterface;
use xiuxian_qianji::{BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, run_scenario};
use xiuxian_wendao::entity::{Entity, EntityType};
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_zhixing::ATTR_JOURNAL_CARRYOVER;
use xiuxian_zhixing::RESOURCES;
use xiuxian_zhixing::storage::MarkdownStorage;
use xiuxian_zhixing::{ZhixingHeyi, ZhixingHeyiInit};

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

#[tokio::test]
async fn test_strict_teacher_agenda_flow_runs_via_qianji_scenario()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mounts = [BootcampVfsMount::new(
        "agenda-management",
        "zhixing/skills/agenda-management/references",
        &RESOURCES,
    )];
    let options = BootcampRunOptions {
        llm_mode: BootcampLlmMode::Mock {
            response: "<agenda_critique_report><score>0.95</score><critique>Scope is executable.</critique></agenda_critique_report>"
                .to_string(),
        },
        ..BootcampRunOptions::default()
    };

    let report = run_scenario(
        "wendao://skills/agenda-management/references/agenda_flow.toml",
        json!({
            "request": "Plan the afternoon with strict feasibility checks.",
            "raw_facts": "timeboxing, risk-first planning with milimeter-level alignment, full audit trail, end-to-end traceability, and architectural consistency constraints",
            "wendao_search_results": "<hit id=\"task:stale\" type=\"task\" carryover=\"3\">Stale Task</hit>"
        }),
        &mounts,
        options,
    )
    .await?;

    assert!(
        matches!(
            report.manifest_name.as_str(),
            "Triangular_Agenda_Governance_Flow" | "Self_Healing_Agenda_Governance_Flow"
        ),
        "unexpected manifest: {}",
        report.manifest_name
    );
    assert_eq!(report.final_context["audit_status"], "passed");
    let governance_score = report.final_context["governance_score"]
        .as_f64()
        .ok_or("governance_score should be numeric")?;
    assert!((governance_score - 0.95).abs() < 1e-5);
    assert!(report.final_context["final_synaptic_report"].is_string());
    Ok(())
}
