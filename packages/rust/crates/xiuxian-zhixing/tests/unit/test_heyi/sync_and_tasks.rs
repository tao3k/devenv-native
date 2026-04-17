use super::support::*;

#[test]
fn test_sync_from_disk_indexes_notebook_into_wendao_graph() -> TestResult {
    let TestContext {
        graph,
        temp_dir,
        heyi,
    } = context("UTC")?;

    let journal_dir = temp_dir.path().join("journal");
    let agenda_dir = temp_dir.path().join("agenda");
    fs::create_dir_all(&journal_dir)?;
    fs::create_dir_all(&agenda_dir)?;
    fs::write(
        journal_dir.join("2026-02-26.md"),
        "## Reflection\nObserved execution discipline improvement.\n",
    )?;
    fs::write(
        agenda_dir.join("2026-02-26.md"),
        "- [ ] Verify sync path <!-- id: sync-1, journal:carryover: 1 -->\n",
    )?;

    let summary = heyi.sync_from_disk()?;
    assert_eq!(summary.journal_documents, 1);
    assert_eq!(summary.agenda_documents, 1);
    assert_eq!(summary.task_entities, 1);

    let documents = graph.get_entities_by_type("DOCUMENT");
    assert!(
        documents.len() >= 2,
        "sync should include at least agenda/journal documents; got {}",
        documents.len()
    );
    assert!(
        documents
            .iter()
            .any(|entity| entity.name == "Journal 2026-02-26"),
        "journal notebook document should exist after sync"
    );
    assert!(
        documents.iter().any(|entity| entity.name == "Agenda 2026-02-26"),
        "agenda notebook document should exist after sync"
    );
    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_add_task_preserves_scheduled_input_on_heyi_surface() -> TestResult {
    let TestContext {
        graph,
        temp_dir: _temp_dir,
        heyi,
    } = context("America/Los_Angeles")?;

    let response = heyi
        .add_task(
            "Normalize local time",
            Some("2026-02-25 10:09 PM".to_string()),
        )
        .await?;

    let tasks = graph.get_entities_by_type("TASK");
    let has_expected_schedule = tasks.iter().any(|task| {
        task.metadata
            .get(ATTR_TIMER_SCHEDULED)
            .and_then(serde_json::Value::as_str)
            == Some("2026-02-25 10:09 PM")
    });
    assert!(has_expected_schedule);
    assert!(response.contains("Normalize local time"));
    Ok(())
}

#[tokio::test]
async fn test_add_task_accepts_unparsed_scheduled_input_on_heyi_surface() -> TestResult {
    let TestContext {
        graph,
        temp_dir: _temp_dir,
        heyi,
    } = context("America/Los_Angeles")?;

    let marker = "Reject invalid time marker";
    let response = heyi
        .add_task(marker, Some("blorp-not-a-time".to_string()))
        .await?;
    assert!(response.contains(marker));

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks[0]
            .metadata
            .get(ATTR_TIMER_SCHEDULED)
            .and_then(serde_json::Value::as_str),
        Some("blorp-not-a-time")
    );

    Ok(())
}
