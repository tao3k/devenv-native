use std::fs;
use std::sync::Arc;

use tempfile::tempdir;
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_wendao::skill_runtime::zhixing::{ZhixingIndexSummary, ZhixingWendaoIndexer};
use xiuxian_zhixing::{ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED};

fn run_agenda_index(
    agenda_markdown: &str,
) -> std::result::Result<(Arc<KnowledgeGraph>, ZhixingIndexSummary), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let agenda_dir = tmp.path().join("agenda");
    fs::create_dir_all(&agenda_dir)?;
    fs::write(agenda_dir.join("2026-02-26.md"), agenda_markdown)?;

    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = ZhixingWendaoIndexer::new(Arc::clone(&graph), tmp.path().to_path_buf());
    let summary = indexer.index_all_domain_objects()?;
    Ok((graph, summary))
}

#[test]
fn test_projection_extracts_task_id_priority_and_carryover()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (graph, summary) = run_agenda_index(
        "- [ ] Ship feature <!-- id: T-01, priority: P1, journal:carryover: 3 -->\n",
    )?;
    assert_eq!(summary.task_entities, 1);

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];
    assert_eq!(
        task.metadata.get("task_id"),
        Some(&serde_json::json!("T-01"))
    );
    assert_eq!(
        task.metadata.get("task_priority"),
        Some(&serde_json::json!("P1"))
    );
    assert_eq!(
        task.metadata.get(ATTR_JOURNAL_CARRYOVER),
        Some(&serde_json::json!(3))
    );
    Ok(())
}

#[test]
fn test_projection_extracts_rfc3339_timer_fields()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (graph, summary) = run_agenda_index(
        "- [x] Follow up <!-- id: a2, timer:scheduled: 2026-02-26T09:00:00Z, timer:reminded: true -->\n",
    )?;
    assert_eq!(summary.task_entities, 1);

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];
    assert_eq!(
        task.metadata.get(ATTR_TIMER_SCHEDULED),
        Some(&serde_json::json!("2026-02-26T09:00:00Z"))
    );
    assert_eq!(
        task.metadata.get(ATTR_TIMER_REMINDED),
        Some(&serde_json::json!(true))
    );
    Ok(())
}

#[test]
fn test_projection_accepts_single_colon_metadata_format()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let (graph, summary) = run_agenda_index(
        "- [ ] Compile report <!-- id:a7, priority:P2, journal:carryover: 4 -->\n",
    )?;
    assert_eq!(summary.task_entities, 1);

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];
    assert_eq!(task.metadata.get("task_id"), Some(&serde_json::json!("a7")));
    assert_eq!(
        task.metadata.get("task_priority"),
        Some(&serde_json::json!("P2"))
    );
    assert_eq!(
        task.metadata.get(ATTR_JOURNAL_CARRYOVER),
        Some(&serde_json::json!(4))
    );
    Ok(())
}

#[test]
fn test_projection_skips_invalid_lines_and_normalizes_identity_token()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let content = [
        "not-a-task-line",
        "- [ ]   ",
        "- [ ] Normalize token <!-- id: A/B:C-01 -->",
        "",
    ]
    .join("\n");
    let (graph, summary) = run_agenda_index(&content)?;
    assert_eq!(summary.task_entities, 1);

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);
    assert!(tasks[0].id.ends_with(":a-b-c-01"));
    Ok(())
}
