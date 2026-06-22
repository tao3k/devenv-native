use super::common::{
    AGENDA_FACTS, AGENDA_FLOW_URI_FROM_ALIAS, bootcamp_context_from_env, mock_llm_options,
    result_error, runtime_default_llm_options, zhixing_mount,
};
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::run_scenario;
use xiuxian_wendao::link_graph::LinkGraphIndex;

#[tokio::test]
async fn bootcamp_runs_real_adversarial_flow() {
    let mounts = zhixing_mount("zhixing", "zhixing/skills/agenda-management/references");

    let Some(initial_context) = bootcamp_context_from_env() else {
        return;
    };
    let empty_index_root =
        tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should succeed: {error}"));
    let mut options = runtime_default_llm_options();
    options.index = Some(Arc::new(
        LinkGraphIndex::build(empty_index_root.path())
            .unwrap_or_else(|error| panic!("empty bootcamp index should build: {error}")),
    ));

    let flow_uri = "wendao://skills/zhixing/references/agenda_flow.toml";
    let error = result_error(
        run_scenario(flow_uri, initial_context, &mounts, options).await,
        "local Qianji LLM execution should be retired",
    );
    assert!(
        error
            .to_string()
            .contains("local Qianji LLM execution is retired"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn bootcamp_runs_embedded_agenda_flow_with_mock_llm() {
    let mounts = zhixing_mount("agenda-lab", "zhixing/skills/agenda-management/references");
    let options = mock_llm_options(
        "<agenda_critique_report><score>0.95</score><reason>approved</reason></agenda_critique_report>",
    );

    let error = result_error(
        run_scenario(
            AGENDA_FLOW_URI_FROM_ALIAS,
            json!({
                "request": "Generate today's agenda and then critique it.",
                "raw_facts": AGENDA_FACTS
            }),
            &mounts,
            options,
        )
        .await,
        "local Qianji LLM execution should be retired",
    );
    assert!(
        error
            .to_string()
            .contains("local Qianji LLM execution is retired"),
        "unexpected error: {error}"
    );
}
