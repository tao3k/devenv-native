use super::common::{
    FORGE_FLOW_URI_CANONICAL, bootcamp_context_from_env, ensure_runtime_forge_context_defaults,
    result_error, runtime_default_llm_options, zhixing_mount,
};
use serde_json::json;
use std::fs;
use tempfile::tempdir;
use xiuxian_qianji::run_scenario;

#[tokio::test]
async fn bootcamp_runs_embedded_forge_flow_with_native_annotation() {
    let mounts = zhixing_mount(
        "forge-evolution",
        "zhixing/skills/forge-evolution/references",
    );
    let options = runtime_default_llm_options();
    let project_root = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let target_persona_dir = project_root.path().join("personas");
    let expected_manifested_path = target_persona_dir.join("soul_forger_v2.md");

    let report = run_scenario(
        FORGE_FLOW_URI_CANONICAL,
        json!({
            "failure_trace": "retry loop exceeded threshold in agenda validation for three consecutive sessions",
            "failure_cluster": "planning-consistency-regression",
            "target_domain": "agenda-management",
            "raw_facts": "three repeated low scores below 0.5; stale carryover accumulation; unstable prioritization output",
            "wendao_search_results": "<hit id=\"audit:1\" type=\"audit\" score=\"0.42\">Repeated overload planning failure</hit>",
            "project_root": project_root.path().display().to_string(),
            "target_persona_dir": target_persona_dir.display().to_string(),
            "role_id": "soul_forger_v2",
            "forge_changed_paths": [expected_manifested_path.display().to_string()]
        }),
        &mounts,
        options,
    )
    .await
    .unwrap_or_else(|error| panic!("embedded forge flow should run natively: {error}"));

    assert!(!report.requires_llm);
    let manifested = fs::read_to_string(&expected_manifested_path)
        .unwrap_or_else(|error| panic!("manifested persona should be written: {error}"));
    assert!(manifested.contains("retry loop exceeded threshold"));
}

#[tokio::test]
async fn bootcamp_runs_real_forge_flow() {
    let mounts = zhixing_mount(
        "forge-evolution",
        "zhixing/skills/forge-evolution/references",
    );

    let Some(mut context) = bootcamp_context_from_env() else {
        return;
    };
    ensure_runtime_forge_context_defaults(&mut context);
    let options = runtime_default_llm_options();

    let error = result_error(
        run_scenario(FORGE_FLOW_URI_CANONICAL, context, &mounts, options).await,
        "local Qianji LLM execution should be retired",
    );
    assert!(
        error
            .to_string()
            .contains("local Qianji LLM execution is retired"),
        "unexpected error: {error}"
    );
}
