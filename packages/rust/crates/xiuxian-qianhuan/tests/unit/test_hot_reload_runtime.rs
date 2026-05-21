//! Integration tests for the shared hot-reload runtime.

use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;
use xiuxian_qianhuan::{
    HotReloadRuntime, HotReloadStatus, HotReloadTargetId, InMemoryHotReloadVersionBackend,
    ManifestationManager, ManifestationTemplateTarget,
};

fn create_manager(root: &Path) -> Result<Arc<ManifestationManager>> {
    let glob = format!("{}/*", root.display());
    Ok(Arc::new(ManifestationManager::new(&[glob.as_str()])?))
}

fn register_manifestation_target(
    runtime: &HotReloadRuntime,
    manager: &Arc<ManifestationManager>,
) -> Result<()> {
    let target = Arc::clone(manager).hot_reload_target("qianhuan.manifestation.templates")?;
    runtime.register_target(target)?;
    Ok(())
}

#[test]
fn hot_reload_runtime_local_change_refreshes_manifestation_templates() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("daily_agenda.md");
    fs::write(&template_path, "Agenda v1: {{ title }}")?;

    let manager = create_manager(dir.path())?;
    let backend = Arc::new(InMemoryHotReloadVersionBackend::default());
    let runtime = HotReloadRuntime::new(Some(backend));
    register_manifestation_target(&runtime, &manager)?;

    let first = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({ "title": "Morning" }),
    )?;
    assert_eq!(first, "Agenda v1: Morning");

    thread::sleep(Duration::from_millis(10));
    fs::write(&template_path, "Agenda v2 updated: {{ title }}")?;

    let outcomes = runtime.on_local_path_change(&template_path)?;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].status, HotReloadStatus::Reloaded);
    assert_eq!(outcomes[0].version, 1);

    let second = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({ "title": "Morning" }),
    )?;
    assert_eq!(second, "Agenda v2 updated: Morning");
    Ok(())
}

#[test]
fn hot_reload_runtime_remote_version_sync_refreshes_target() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("daily_agenda.md");
    fs::write(&template_path, "Agenda v1: {{ title }}")?;

    let manager = create_manager(dir.path())?;
    let backend = Arc::new(InMemoryHotReloadVersionBackend::default());
    let runtime = HotReloadRuntime::new(Some(backend.clone()));
    register_manifestation_target(&runtime, &manager)?;

    thread::sleep(Duration::from_millis(10));
    fs::write(&template_path, "Agenda v2 remote: {{ title }}")?;
    backend.set_version(
        &HotReloadTargetId::new("qianhuan.manifestation.templates"),
        3,
    )?;

    let outcomes = runtime.sync_remote_versions()?;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].status, HotReloadStatus::Reloaded);
    assert_eq!(outcomes[0].version, 3);

    let rendered = manager.render_target(
        &ManifestationTemplateTarget::DailyAgenda,
        json!({ "title": "Evening" }),
    )?;
    assert_eq!(rendered, "Agenda v2 remote: Evening");
    Ok(())
}

#[test]
fn hot_reload_runtime_reports_no_change_without_version_bump() -> Result<()> {
    let dir = tempdir()?;
    let template_path = dir.path().join("daily_agenda.md");
    fs::write(&template_path, "Agenda stable: {{ title }}")?;

    let manager = create_manager(dir.path())?;
    let backend = Arc::new(InMemoryHotReloadVersionBackend::default());
    let runtime = HotReloadRuntime::new(Some(backend));
    register_manifestation_target(&runtime, &manager)?;

    let outcomes = runtime.on_local_path_change(&template_path)?;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].status, HotReloadStatus::NoChange);
    assert_eq!(outcomes[0].version, 0);

    let versions = runtime.local_versions()?;
    assert_eq!(
        versions.get("qianhuan.manifestation.templates"),
        Some(&0_u64)
    );
    Ok(())
}
