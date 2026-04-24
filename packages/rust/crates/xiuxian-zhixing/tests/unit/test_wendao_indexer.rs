//! Integration tests for Wendao indexer boundary behavior.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use xiuxian_wendao::entity::{Entity, Relation, RelationType};
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_wendao::skill_runtime::zhixing::ZhixingWendaoIndexer;
use xiuxian_wendao::sync::IncrementalSyncPolicy;
use xiuxian_zhixing::ATTR_JOURNAL_CARRYOVER;
use xiuxian_zhixing::ATTR_TIMER_REMINDED;
use xiuxian_zhixing::ATTR_TIMER_SCHEDULED;

#[test]
fn test_indexer_initialization_and_run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let journal_dir = tmp.path().join("journal");
    let agenda_dir = tmp.path().join("agenda");
    fs::create_dir_all(&journal_dir)?;
    fs::create_dir_all(&agenda_dir)?;

    fs::write(
        journal_dir.join("2026-02-26.md"),
        "## Reflection\nObserved stronger execution discipline.\n",
    )?;
    fs::write(
        agenda_dir.join("2026-02-26.md"),
        "- [ ] Review earnings <!-- id: a1, journal:carryover: 3 -->\n- [x] Summarize chart action <!-- id: a2, journal:carryover: 1 -->\n",
    )?;

    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = ZhixingWendaoIndexer::new(Arc::clone(&graph), tmp.path().to_path_buf());
    let summary = indexer.index_all_domain_objects()?;

    assert_eq!(summary.journal_documents, 1);
    assert_eq!(summary.agenda_documents, 1);
    assert_eq!(summary.task_entities, 2);
    assert!(summary.entities_added >= 4);
    assert_eq!(summary.relations_linked, 2);

    let documents = graph.get_entities_by_type("DOCUMENT");
    assert!(
        documents.len() >= 2,
        "zhixing index now includes skill/reference documents in addition to notebook docs"
    );
    let Some(agenda_doc) = documents
        .iter()
        .find(|entity| entity.name == "Agenda 2026-02-26")
    else {
        return Err(std::io::Error::other("agenda document must exist").into());
    };
    assert_eq!(
        agenda_doc.metadata.get("zhixing_domain"),
        Some(&serde_json::json!("agenda"))
    );
    assert_eq!(
        agenda_doc.metadata.get("open_task_count"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        agenda_doc.metadata.get("done_task_count"),
        Some(&serde_json::json!(1))
    );
    let Some(journal_doc) = documents
        .iter()
        .find(|entity| entity.name == "Journal 2026-02-26")
    else {
        return Err(std::io::Error::other("journal document must exist").into());
    };
    assert_eq!(
        journal_doc.metadata.get("reflection_sections"),
        Some(&serde_json::json!(1))
    );

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 2);
    let carryover_three_exists = tasks.iter().any(|task| {
        task.metadata
            .get(ATTR_JOURNAL_CARRYOVER)
            .and_then(serde_json::Value::as_u64)
            == Some(3)
    });
    assert!(carryover_three_exists);
    let task_a1_exists = tasks.iter().any(|task| {
        task.metadata
            .get("task_id")
            .and_then(serde_json::Value::as_str)
            == Some("a1")
            && task
                .metadata
                .get("task_status")
                .and_then(serde_json::Value::as_str)
                == Some("todo")
    });
    assert!(task_a1_exists);
    let completed_task_exists = tasks.iter().any(|task| {
        task.metadata
            .get("task_status")
            .and_then(serde_json::Value::as_str)
            == Some("done")
    });
    assert!(completed_task_exists);
    let timer_fields_absent_by_default = tasks.iter().all(|task| {
        !task.metadata.contains_key(ATTR_TIMER_SCHEDULED)
            && !task.metadata.contains_key(ATTR_TIMER_REMINDED)
    });
    assert!(timer_fields_absent_by_default);

    let contains_relations =
        graph.get_relations(Some("Agenda 2026-02-26"), Some(RelationType::Contains));
    assert_eq!(contains_relations.len(), 2);
    Ok(())
}

#[test]
fn test_incremental_sync_changed_path_reindexes_agenda_file()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let agenda_dir = tmp.path().join("agenda");
    fs::create_dir_all(&agenda_dir)?;
    let agenda_file = agenda_dir.join("2026-02-27.md");
    fs::write(
        &agenda_file,
        "- [ ] First task <!-- id: t1, journal:carryover: 0 -->\n",
    )?;

    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = ZhixingWendaoIndexer::new(Arc::clone(&graph), tmp.path().to_path_buf());
    let _ = indexer.index_all_domain_objects()?;

    let configured_extensions = vec!["md".to_string()];
    let policy = IncrementalSyncPolicy::new(&configured_extensions);
    fs::write(
        &agenda_file,
        "- [ ] Second task <!-- id: t2, journal:carryover: 1 -->\n",
    )?;
    let changed = indexer.sync_changed_path(Path::new(&agenda_file), &policy)?;
    assert!(
        changed,
        "agenda markdown path should trigger incremental sync"
    );

    let tasks = graph.get_entities_by_type("TASK");
    assert_eq!(tasks.len(), 1);
    let Some(task) = tasks.first() else {
        return Err(std::io::Error::other("expected one task after incremental reindex").into());
    };
    assert!(
        task.name.contains("Second task"),
        "task should be updated to second task after reindex"
    );
    assert_eq!(
        task.metadata
            .get("task_id")
            .and_then(serde_json::Value::as_str),
        Some("t2")
    );

    Ok(())
}

#[test]
fn test_incremental_sync_ignores_unsupported_extension()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let agenda_dir = tmp.path().join("agenda");
    fs::create_dir_all(&agenda_dir)?;
    let cfg_file = agenda_dir.join("2026-02-27.toml");
    fs::write(&cfg_file, "title = \"Agenda Config\"")?;

    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = ZhixingWendaoIndexer::new(Arc::clone(&graph), tmp.path().to_path_buf());
    let configured_extensions = vec!["md".to_string()];
    let policy = IncrementalSyncPolicy::new(&configured_extensions);
    let changed = indexer.sync_changed_path(Path::new(&cfg_file), &policy)?;
    assert!(
        !changed,
        "unsupported extension should be skipped by incremental sync policy"
    );
    assert!(graph.get_entities_by_type("DOCUMENT").is_empty());
    Ok(())
}

#[test]
fn test_indexer_injects_embedded_skill_reference_graph()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    fs::create_dir_all(tmp.path().join("journal"))?;
    fs::create_dir_all(tmp.path().join("agenda"))?;

    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = ZhixingWendaoIndexer::new(Arc::clone(&graph), tmp.path().to_path_buf());
    let summary = indexer.index_all_domain_objects()?;

    assert!(
        summary.skill_reference_entities_added >= 8,
        "expected skill, reference, and intent entities to be added"
    );
    assert!(
        summary.skill_reference_relations >= 8,
        "expected semantic references plus typed governs/manifests relations"
    );

    let skills = graph.get_entities_by_type("SKILL");
    let skill_entity = skills
        .iter()
        .find(|entity| entity.name == "agenda-management")
        .ok_or_else(|| std::io::Error::other("skill semantic entity should exist"))?;
    assert_skill_metadata(skill_entity);

    let template_relations =
        graph.get_relations(Some("agenda-management"), Some(RelationType::RelatedTo));
    assert_relation_targets(
        &template_relations,
        &["draft_agenda", "critique_agenda", "final_agenda"],
        "template relations",
    );

    let manifests = graph.get_relations(Some("agenda-management"), Some(RelationType::Manifests));
    assert_relation_targets(&manifests, &["steward", "teacher"], "manifests relations");

    let knowledge_relations =
        graph.get_relations(Some("agenda-management"), Some(RelationType::DocumentedIn));
    assert_relation_targets(
        &knowledge_relations,
        &["rules", "classifier", "agenda_validation_genesis_rules"],
        "knowledge relations",
    );

    let governs = graph.get_relations(Some("agenda-management"), Some(RelationType::Governs));
    assert_relation_targets(&governs, &["agenda_flow"], "governs relations");
    assert!(
        governs.iter().any(|relation| {
            relation
                .metadata
                .get("intent")
                .and_then(serde_json::Value::as_str)
                == Some("Draft a new schedule")
        }),
        "intents should be promoted to governs relations"
    );

    assert_reference_type_hint(&manifests, "steward", "persona");
    assert_reference_type_hint(&manifests, "teacher", "persona");
    assert_reference_type_hint(&knowledge_relations, "rules", "knowledge");
    assert_reference_type_hint(&governs, "agenda_flow", "qianji-flow");

    Ok(())
}

fn assert_skill_metadata(skill_entity: &Entity) {
    for key in ["routing_keywords", "intents"] {
        assert!(
            skill_entity.metadata.contains_key(key),
            "skill metadata should include {key}"
        );
    }
}

fn assert_relation_targets(relations: &[Relation], targets: &[&str], relation_label: &str) {
    for target in targets {
        assert!(
            relations.iter().any(|relation| relation.target == *target),
            "expected target `{target}` in {relation_label}"
        );
    }
}

fn assert_reference_type_hint(relations: &[Relation], target: &str, expected_type: &str) {
    assert!(
        relations.iter().any(|relation| {
            relation.target == target
                && relation
                    .metadata
                    .get("reference_type")
                    .and_then(serde_json::Value::as_str)
                    == Some(expected_type)
        }),
        "target `{target}` should preserve reference_type `{expected_type}`"
    );
}
