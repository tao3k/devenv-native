use super::common::{AGENDA_FACTS, AGENDA_FLOW_URI_FROM_ALIAS};
use super::common::{AGENDA_FLOW_URI_CANONICAL, AGENDA_OVERRIDE_RESOURCES};
use serde_json::json;
use tempfile::tempdir;
use xiuxian_qianji::{
    BootcampRunOptions, BootcampVfsMount, run_scenario, run_workflow,
    run_workflow_from_manifest_toml,
};

#[tokio::test]
async fn bootcamp_rejects_unknown_flow_uri() {
    let Err(error) = run_workflow(
        "wendao://skills/agenda-management/references/not_exists.toml",
        json!({}),
        BootcampRunOptions::default(),
    )
    .await
    else {
        panic!("unknown workflow URI should fail");
    };

    assert!(
        error
            .to_string()
            .contains("semantic flow manifest not found")
    );
}

#[tokio::test]
async fn bootcamp_mounts_override_runtime_wendao_uri_resolution() {
    let mounts = [BootcampVfsMount::new(
        "agenda-management",
        "skills/agenda-management/references",
        &AGENDA_OVERRIDE_RESOURCES,
    )];
    let report = run_scenario(
        AGENDA_FLOW_URI_CANONICAL,
        json!({
            "request": "Generate a tiny agenda.",
            "raw_facts": "tiny facts"
        }),
        &mounts,
        BootcampRunOptions::default(),
    )
    .await
    .unwrap_or_else(|error| panic!("mount override scenario should succeed: {error}"));

    assert_eq!(report.manifest_name, "Agenda_Override_Mount_Test");
    assert_eq!(report.node_count, 1);
}

#[tokio::test]
async fn bootcamp_runs_inline_manifest_toml() {
    let repo_root = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let report = run_workflow_from_manifest_toml(
        r#"
name = "Inline_Manifest_Test"

[[nodes]]
id = "Done"
task_type = "command"
weight = 1.0
params = { command = "printf inline_bootcamp_ok" }
"#,
        json!({}),
        BootcampRunOptions {
            repo_path: Some(repo_root.path().to_path_buf()),
            ..BootcampRunOptions::default()
        },
    )
    .await
    .unwrap_or_else(|error| panic!("inline bootcamp manifest should succeed: {error}"));

    assert_eq!(report.flow_uri, "inline://qianji/manifest");
    assert_eq!(report.manifest_name, "Inline_Manifest_Test");
    assert_eq!(report.node_count, 1);
}

#[tokio::test]
async fn bootcamp_flags_llm_feature_requirement_for_agenda_flow() {
    let Err(error) = run_workflow(
        AGENDA_FLOW_URI_FROM_ALIAS,
        json!({
            "request": "Generate today's agenda and then critique it.",
            "raw_facts": AGENDA_FACTS
        }),
        BootcampRunOptions::default(),
    )
    .await
    else {
        panic!("alias URI should fail without explicit mounts");
    };

    assert!(
        error
            .to_string()
            .contains("semantic flow manifest not found")
    );
}
