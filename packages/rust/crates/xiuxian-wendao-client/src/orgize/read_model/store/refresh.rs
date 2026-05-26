use std::path::PathBuf;

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::{
    OrgizeAgentTaskReadModelRequest, OrgizeOrgElementReadModelRequest, collect_agent_task_rows,
    collect_org_element_rows,
};

use crate::ClientContext;
use crate::orgize::read_model::model::RefreshedAgentOrgReadModel;
use crate::orgize::read_model::settings::{resolve_read_model_settings, resolve_source_paths};

use super::materialize::materialize_agent_org_tasks;

pub(in crate::orgize::read_model) fn refresh_agent_org_read_model(
    paths: &[PathBuf],
    context: &ClientContext,
) -> Result<RefreshedAgentOrgReadModel> {
    let settings = resolve_read_model_settings(context)?;
    let source_paths = resolve_source_paths(paths, context, settings.cache_home.as_path());
    let report = collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: source_paths.clone(),
        match_expression: Some("+agent".to_string()),
        include_comments: false,
    })?;
    let element_report = collect_org_element_rows(&OrgizeOrgElementReadModelRequest {
        paths: source_paths.clone(),
    })?;
    let materialized = materialize_agent_org_tasks(&settings, &report.rows, &element_report.rows)
        .with_context(|| {
        format!(
            "failed to materialize Org agent read model at `{}`",
            settings.database_path.display()
        )
    })?;

    Ok(RefreshedAgentOrgReadModel {
        settings,
        source_paths,
        materialized,
    })
}
