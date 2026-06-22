//! Integration tests for Repo Intelligence relation graph output.

#[cfg(feature = "julia")]
use std::fs;

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, write_repo_config,
};
use serde_json::json;
use xiuxian_wendao::analyzers::analyze_repository_from_config;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn analysis_emits_structural_and_semantic_relations() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "RelationPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "relation-sample")?;

    let analysis =
        analyze_repository_from_config("relation-sample", Some(&config_path), temp.path())?;
    let relations = normalized_relations_payload(analysis.relations);

    assert_repo_json_snapshot("repo_relations_result", json!(relations));
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_emits_structural_and_semantic_relations() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Relationica")?;
    let config_path = temp.path().join("modelica-relations.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-relations]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let analysis =
        analyze_repository_from_config("modelica-relations", Some(&config_path), temp.path())?;
    let relations = normalized_relations_payload(analysis.relations);

    assert_repo_json_snapshot("repo_relations_modelica_result", json!(relations));
    Ok(())
}

fn normalized_relations_payload(
    relations: Vec<xiuxian_wendao::analyzers::RelationRecord>,
) -> Vec<serde_json::Value> {
    let mut relations = relations
        .into_iter()
        .map(|relation| {
            json!({
                "kind": format!("{:?}", relation.kind).to_ascii_lowercase(),
                "source_id": relation.source_id,
                "target_id": relation.target_id,
            })
        })
        .collect::<Vec<_>>();
    relations.sort_by(|left, right| {
        left["kind"]
            .as_str()
            .cmp(&right["kind"].as_str())
            .then_with(|| left["source_id"].as_str().cmp(&right["source_id"].as_str()))
            .then_with(|| left["target_id"].as_str().cmp(&right["target_id"].as_str()))
    });
    relations
}
