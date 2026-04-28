use std::fs;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use tempfile::TempDir;
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_zhixing::storage::MarkdownStorage;

use super::*;
use crate::config::{WendaoGatewayConfig, XiuxianConfig};

const AGENDA_ADD_MANIFEST: &str = r#"id = "xiuxian.zhixing.agenda.add"

[qianhuan]
background = "$wendao://skills/agenda/SKILL.md"

[tool_contract]
category = "write"
name = "agenda.add"
description = """
Draft and add a new vow.

Args:
    - title: str - The title of the task.

Examples:
    Input:
        { "title": "Ship loader" }
    Output:
        "done"
"""

[workflow]
type = "qianji_flow"
binding_id = "xiuxian.native.zhixing.add"
flow_definition = "$wendao://skills/agenda/references/add/qianji.toml"
"#;

const AGENDA_VIEW_MANIFEST: &str = r#"id = "xiuxian.zhixing.agenda.view"

[qianhuan]
background = "$wendao://skills/agenda/SKILL.md"

[tool_contract]
category = "read"
name = "agenda.view"
description = """
Review agenda entries.

Args:
    - limit: int - Maximum number of items to show.

Examples:
    Input:
        { "limit": 5 }
    Output:
        "ok"
"""

[workflow]
type = "native_dispatch"
binding_id = "xiuxian.native.zhixing.view"
"#;

fn build_heyi() -> Result<(Arc<ZhixingHeyi>, TempDir)> {
    let graph = Arc::new(KnowledgeGraph::new());
    let tmp = TempDir::new()?;
    let storage = Arc::new(MarkdownStorage::new(tmp.path().to_path_buf()));
    let manifestation = Arc::new(ManifestationManager::new_with_embedded_templates(
        &[],
        &[("task_add_response.md", "ok")],
    )?);
    let heyi = ZhixingHeyi::new(
        graph,
        manifestation,
        storage,
        "skill-alias-test".to_string(),
        "UTC",
    )?;
    Ok((Arc::new(heyi), tmp))
}

fn write_fixture_tree(
    root: &std::path::Path,
    include_ghost: bool,
    include_unauthorized: bool,
) -> Result<()> {
    let skill_root = root.join("skills").join("agenda");
    let references_root = skill_root.join("references");
    let add_root = references_root.join("add");
    let view_root = references_root.join("view");
    fs::create_dir_all(&add_root)?;
    fs::write(add_root.join("qianji.toml"), AGENDA_ADD_MANIFEST)?;

    if include_unauthorized {
        fs::create_dir_all(&view_root)?;
        fs::write(view_root.join("qianji.toml"), AGENDA_VIEW_MANIFEST)?;
    }

    let mut tags = vec!["wendao://skills/agenda/references/add/qianji.toml".to_string()];
    if include_ghost {
        tags.push("wendao://skills/agenda/references/missing/qianji.toml".to_string());
    }
    let skill_doc = format!(
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "type: skill\n",
            "title: Agenda\n",
            "category: skills\n",
            "tags:\n",
            "{}\n",
            "name: agenda\n",
            "description: Internal agenda skill background.\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "metadata:\n",
            "  version: \"0.1.0\"\n",
            "  source: xiuxian-daochang-test-fixture\n",
            "  routing_keywords:\n",
            "    - agenda\n",
            "    - native-tool\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "# Agenda\n",
        ),
        tags.iter()
            .map(|tag| format!("  - {tag}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::create_dir_all(&skill_root)?;
    fs::write(skill_root.join("SKILL.md"), skill_doc)?;
    Ok(())
}

fn build_resolver(root: &std::path::Path) -> Result<Arc<SkillRuntimeResolver>> {
    let runtime_root = root.join("skills");
    let resolver =
        SkillRuntimeResolver::from_roots_with_embedded_and_runtime(&[], &[runtime_root])?;
    Ok(Arc::new(resolver))
}

#[test]
fn blocks_alias_mount_on_ghost_links() -> Result<()> {
    let tmp = TempDir::new()?;
    write_fixture_tree(tmp.path(), true, false)?;
    let resolver = build_resolver(tmp.path())?;
    let (heyi, _storage) = build_heyi()?;

    let mut registry = NativeToolRegistry::new();
    let mut mounts = ServiceMountCatalog::new();
    mount_native_tool_cauldron(
        None,
        Some(&heyi),
        Some(&resolver),
        &mut registry,
        &mut mounts,
    );

    assert!(registry.get("agenda.add").is_none());
    let records = mounts.finish();
    let alias_record = records
        .iter()
        .find(|record| record.service == "native.skill_aliases")
        .ok_or_else(|| anyhow!("alias mount record should exist"))?;
    assert_eq!(
        serde_json::to_value(alias_record.status)?,
        serde_json::Value::String("failed".to_string())
    );
    Ok(())
}

#[test]
fn mounts_authorized_aliases_and_skips_unauthorized() -> Result<()> {
    let tmp = TempDir::new()?;
    write_fixture_tree(tmp.path(), false, true)?;
    let resolver = build_resolver(tmp.path())?;
    let (heyi, _storage) = build_heyi()?;

    let mut registry = NativeToolRegistry::new();
    let mut mounts = ServiceMountCatalog::new();
    mount_native_tool_cauldron(
        None,
        Some(&heyi),
        Some(&resolver),
        &mut registry,
        &mut mounts,
    );

    assert!(registry.get("agenda.add").is_some());
    assert!(registry.get("agenda.view").is_some());

    let alias_tool = registry
        .get("agenda.add")
        .ok_or_else(|| anyhow!("alias tool mounted"))?;
    assert!(alias_tool.description().contains("Draft and add a new vow"));

    let records = mounts.finish();
    let alias_record = records
        .iter()
        .find(|record| record.service == "native.skill_aliases")
        .ok_or_else(|| anyhow!("alias mount record should exist"))?;
    assert_eq!(
        serde_json::to_value(alias_record.status)?,
        serde_json::Value::String("mounted".to_string())
    );
    assert!(
        alias_record
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("unauthorized=1")
    );
    Ok(())
}

#[test]
fn mounts_wendao_search_when_gateway_configured() -> Result<()> {
    let mut registry = NativeToolRegistry::new();
    let mut mounts = ServiceMountCatalog::new();
    let mut config = XiuxianConfig::default();
    config.wendao_gateway = WendaoGatewayConfig {
        query_endpoint: Some("http://127.0.0.1:18093/query".to_string()),
        default_project_root: Some("/repo/default".to_string()),
        session_project_roots: std::collections::HashMap::new(),
    };

    mount_native_tool_cauldron(Some(&config), None, None, &mut registry, &mut mounts);

    assert!(registry.get("wendao.search").is_some());
    assert!(registry.get("knowledge.search").is_some());
    let knowledge_search = registry
        .get("knowledge.search")
        .ok_or_else(|| anyhow!("knowledge search alias should exist"))?;
    let parameters = knowledge_search.parameters();
    assert!(
        parameters
            .get("properties")
            .and_then(|value| value.get("query"))
            .is_some()
    );
    assert_eq!(
        parameters
            .get("required")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default(),
        vec![serde_json::Value::String("query".to_string())]
    );
    let records = mounts.finish();
    let record = records
        .iter()
        .find(|record| record.service == "native.wendao_search")
        .ok_or_else(|| anyhow!("wendao search mount record should exist"))?;
    assert_eq!(
        serde_json::to_value(record.status)?,
        serde_json::Value::String("mounted".to_string())
    );
    assert_eq!(
        record.endpoint.as_deref(),
        Some("http://127.0.0.1:18093/query")
    );
    assert!(
        record
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("aliases=knowledge.search")
    );
    let advertised_names = registry
        .list_for_llm()
        .into_iter()
        .filter_map(|tool| {
            tool.get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    let knowledge_index = advertised_names
        .iter()
        .position(|name| name == "knowledge.search")
        .ok_or_else(|| anyhow!("knowledge.search should be advertised to the llm"))?;
    let wendao_index = advertised_names
        .iter()
        .position(|name| name == "wendao.search")
        .ok_or_else(|| anyhow!("wendao.search should remain advertised for compatibility"))?;
    assert!(
        knowledge_index < wendao_index,
        "knowledge.search should sort ahead of wendao.search for llm exposure: {advertised_names:?}"
    );
    Ok(())
}

#[test]
fn mounts_wendao_search_even_when_gateway_endpoint_is_not_preconfigured() -> Result<()> {
    let mut registry = NativeToolRegistry::new();
    let mut mounts = ServiceMountCatalog::new();

    mount_native_tool_cauldron(
        Some(&XiuxianConfig::default()),
        None,
        None,
        &mut registry,
        &mut mounts,
    );

    assert!(registry.get("wendao.search").is_some());
    assert!(registry.get("knowledge.search").is_some());
    let records = mounts.finish();
    let record = records
        .iter()
        .find(|record| record.service == "native.wendao_search")
        .ok_or_else(|| anyhow!("wendao search mount record should exist"))?;
    assert_eq!(
        serde_json::to_value(record.status)?,
        serde_json::Value::String("mounted".to_string())
    );
    assert!(
        record
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("config=runtime_dynamic")
    );
    assert!(
        record
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("aliases=knowledge.search")
    );
    assert!(record.endpoint.is_none());
    Ok(())
}
