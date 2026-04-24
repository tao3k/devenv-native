use std::sync::Arc;

use crate::agent::bootstrap::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use crate::agent::native_tools::NativeAliasTool;
use crate::agent::native_tools::macros::register_native_tools;
use crate::agent::native_tools::registry::NativeTool;
use crate::agent::native_tools::registry::NativeToolRegistry;
use crate::agent::native_tools::spider::SpiderCrawlTool;
use crate::agent::native_tools::wendao_search::{WendaoSearchTool, WendaoSearchToolConfig};
use crate::agent::native_tools::zhixing::{AgendaViewTool, JournalRecordTool, TaskAddTool};
use crate::config::XiuxianConfig;
use xiuxian_wendao::ingress::SpiderWendaoBridge;
use xiuxian_wendao::skill_runtime::authority::AuthorizedSkillNativeAliasScan;
use xiuxian_wendao::skill_runtime::{
    SkillNativeAliasSpec, SkillRuntimeResolver, SkillWorkflowType,
};
use xiuxian_zhixing::ZhixingHeyi;

pub(in crate::agent::bootstrap) fn mount_native_tool_cauldron(
    xiuxian_cfg: Option<&XiuxianConfig>,
    heyi: Option<&Arc<ZhixingHeyi>>,
    skill_runtime_resolver: Option<&Arc<SkillRuntimeResolver>>,
    native_tools: &mut NativeToolRegistry,
    mounts: &mut ServiceMountCatalog,
) {
    if let Some(heyi) = heyi {
        register_zhixing_native_tools(heyi, native_tools, mounts);
    } else {
        mounts.skipped(
            "zhixing.native_tools",
            "tooling",
            ServiceMountMeta::default().detail("heyi_unavailable"),
        );
    }

    mount_spider_tool(heyi, native_tools, mounts);
    mount_wendao_search_tool(xiuxian_cfg, native_tools, mounts);
    mount_skill_aliases(skill_runtime_resolver, native_tools, mounts);
}

pub(in crate::agent::bootstrap) fn register_zhixing_native_tools(
    heyi: &Arc<ZhixingHeyi>,
    native_tools: &mut NativeToolRegistry,
    mounts: &mut ServiceMountCatalog,
) {
    register_native_tools!(
        native_tools,
        JournalRecordTool {
            heyi: Arc::clone(heyi),
        },
        TaskAddTool {
            heyi: Arc::clone(heyi),
        },
        AgendaViewTool {
            heyi: Arc::clone(heyi),
        }
    );
    mounts.mounted(
        "zhixing.native_tools",
        "tooling",
        ServiceMountMeta::default().detail("tools=journal.record,task.add,agenda.view"),
    );
}

type RuntimeAliasSpec = SkillNativeAliasSpec<SkillWorkflowType>;

fn mount_spider_tool(
    heyi: Option<&Arc<ZhixingHeyi>>,
    native_tools: &mut NativeToolRegistry,
    mounts: &mut ServiceMountCatalog,
) {
    let ingress = heyi.map(|heyi| {
        Arc::new(SpiderWendaoBridge::for_shared_knowledge_graph(Arc::clone(
            &heyi.graph,
        )))
    });
    native_tools.register(Arc::new(SpiderCrawlTool {
        ingress: ingress.clone(),
    }));

    let detail = if ingress.is_some() {
        "ingress=enabled"
    } else {
        "ingress=disabled"
    };
    mounts.mounted(
        "native.web_crawl",
        "tooling",
        ServiceMountMeta::default().detail(detail),
    );
}

fn mount_skill_aliases(
    skill_runtime_resolver: Option<&Arc<SkillRuntimeResolver>>,
    native_tools: &mut NativeToolRegistry,
    mounts: &mut ServiceMountCatalog,
) {
    let Some(resolver) = skill_runtime_resolver else {
        mounts.skipped(
            "native.skill_aliases",
            "tooling",
            ServiceMountMeta::default().detail("skill_runtime_unavailable"),
        );
        return;
    };

    let Some(root) = resolver.runtime_roots().first() else {
        mounts.skipped(
            "native.skill_aliases",
            "tooling",
            ServiceMountMeta::default().detail("no_runtime_roots"),
        );
        return;
    };

    let scan = match resolver.scan_authorized_native_aliases(root.as_path()) {
        Ok(scan) => scan,
        Err(error) => {
            mounts.failed(
                "native.skill_aliases",
                "tooling",
                ServiceMountMeta::default().detail(format!("scan_failed: {error}")),
            );
            return;
        }
    };

    let AuthorizedSkillNativeAliasScan {
        report,
        compiled_specs,
    } = scan;
    let compiled_count = compiled_specs.len();
    if report.is_critically_failed() {
        mounts.failed(
            "native.skill_aliases",
            "tooling",
            ServiceMountMeta::default()
                .detail(format!("ghost_links_detected: {}", report.ghost_count())),
        );
        return;
    }

    if report.unauthorized_count() > 0 {
        tracing::warn!(
            event = "agent.bootstrap.skill_alias.unauthorized",
            unauthorized = report.unauthorized_count(),
            "unauthorized skill manifests detected; skipping those manifests"
        );
    }

    let (mounted_names, alias_issues) = register_skill_aliases(compiled_specs, native_tools);
    let issue_count = report.issues.len() + alias_issues.len();
    if issue_count > 0 {
        tracing::warn!(
            event = "agent.bootstrap.skill_alias.issues",
            issues = issue_count,
            "skill alias mounting reported issues"
        );
    }

    let detail = format!(
        "authorized={},compiled={},mounted={},ghosts={},unauthorized={},issues={}",
        report.authorized_count(),
        compiled_count,
        mounted_names.len(),
        report.ghost_count(),
        report.unauthorized_count(),
        issue_count,
    );
    mounts.mounted(
        "native.skill_aliases",
        "tooling",
        ServiceMountMeta::default().detail(detail),
    );
}

fn register_skill_aliases(
    compiled_specs: Vec<RuntimeAliasSpec>,
    native_tools: &mut NativeToolRegistry,
) -> (Vec<String>, Vec<String>) {
    let mut mounted = Vec::new();
    let mut issues = Vec::new();

    for spec in compiled_specs {
        let alias_name = spec.tool_name.trim().to_string();
        if alias_name.is_empty() {
            issues.push("skill alias missing tool_name".to_string());
            continue;
        }

        if native_tools.get(alias_name.as_str()).is_some() {
            issues.push(format!("alias name already registered: {alias_name}"));
            continue;
        }

        let Some(target_tool) = native_tools.get(spec.target_tool_name.as_str()) else {
            issues.push(format!(
                "alias target not registered: {} -> {}",
                alias_name, spec.target_tool_name
            ));
            continue;
        };

        if target_tool.name() == alias_name {
            issues.push(format!("alias target matches alias name: {alias_name}"));
            continue;
        }

        let tool = NativeAliasTool::new(
            alias_name.clone(),
            spec.description.clone(),
            target_tool.parameters(),
            target_tool,
        );
        native_tools.register(Arc::new(tool));
        mounted.push(alias_name);
    }

    (mounted, issues)
}

fn mount_wendao_search_tool(
    xiuxian_cfg: Option<&XiuxianConfig>,
    native_tools: &mut NativeToolRegistry,
    mounts: &mut ServiceMountCatalog,
) {
    let endpoint = xiuxian_cfg
        .and_then(WendaoSearchToolConfig::from_xiuxian_config)
        .map(|config| config.query_endpoint().to_string());
    let tool = Arc::new(WendaoSearchTool::new_runtime_default());
    let alias = Arc::new(NativeAliasTool::new(
        "knowledge.search".to_string(),
        "Search indexed project knowledge, code, docs, schema catalogs, and entities through a bounded workflow. Preferred knowledge-facing entrypoint for repo-specific questions instead of answering from memory.".to_string(),
        tool.parameters(),
        tool.clone(),
    ));
    native_tools.register(tool);
    native_tools.register(alias);
    mounts.mounted(
        "native.wendao_search",
        "tooling",
        ServiceMountMeta {
            endpoint,
            storage: None,
            detail: Some(
                "workflow=wendao_sql_authoring_v1,config=runtime_dynamic,aliases=knowledge.search"
                    .to_string(),
            ),
        },
    );
}

#[cfg(test)]
#[path = "../../../tests/unit/agent/bootstrap/native_tools.rs"]
mod tests;
