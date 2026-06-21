//! Integration tests for deterministic docs-facing deep-wiki planner item reopening.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsPlannerItemQuery, DocsProjectedGapReportQuery, ProjectionPageKind,
    docs_planner_item_from_config, docs_projected_gap_report_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_planner_item_executes_over_external_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        repo_dir.join("Controllers").join("NoDocs.mo"),
        "within Projectionica.Controllers;\nmodel NoDocs\nend NoDocs;\n",
    )?;
    let config_path = temp.path().join("modelica-docs-planner-item.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-planner-item]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let gap_report = docs_projected_gap_report_from_config(
        &DocsProjectedGapReportQuery {
            repo_id: "modelica-docs-planner-item".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let gap = gap_report
        .gaps
        .first()
        .cloned()
        .ok_or("expected at least one projected gap in sample Modelica repo")?;

    let result = docs_planner_item_from_config(
        &DocsPlannerItemQuery {
            repo_id: "modelica-docs-planner-item".to_string(),
            gap_id: gap.gap_id.clone(),
            family_kind: Some(ProjectionPageKind::HowTo),
            related_limit: 3,
            family_limit: 3,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(
        result.gap.gap_id, gap.gap_id,
        "planner item should reopen the requested stable gap"
    );
    assert_eq!(
        result.hit.page.page_id, result.gap.page_id,
        "planner item retrieval hit should stay anchored to the gap page"
    );
    assert_eq!(
        result
            .navigation
            .center
            .as_ref()
            .map(|center| center.page.page_id.as_str()),
        Some(result.gap.page_id.as_str()),
        "planner item navigation bundle should stay centered on the gap page"
    );
    assert!(
        result.navigation.related_pages.len() <= 3,
        "planner item should honor the related-page limit"
    );

    assert_repo_json_snapshot("docs_planner_item_modelica_result", json!(result));
    Ok(())
}
