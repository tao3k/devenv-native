use super::support::{assert_cli_success, run_orgize, tempdir_or_panic};

#[test]
fn standalone_orgize_task_sdd_renders_task_relation_graph() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("audio_openrouter_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Audio OpenRouter gate <2026-05-23 Sat> :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: audio-gate\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-gate\n",
            ":SDD: .cache/agent/sdd/audio.org\n",
            ":STABLE_REF: packages/python/xiuxian-wendao-analyzer/README.md\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-sdd",
            "--id",
            "audio-gate",
            "audio_openrouter_lane.org",
        ],
        "task-sdd",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .contains("task: Audio OpenRouter gate <2026-05-23 Sat>"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains(
            "graph: flowchart LR;T[\"task:audio-gate\"]-->N0[\"sdd:.cache/agent/sdd/audio.org\"];T-->N1[\"package:xiuxian-wendao-analyzer\"];T-->N2[\"slice:audio-openrouter-gate\"]"
        ),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("inspect-sdd: wendao-client orgize sdd status .cache/agent/sdd/audio.org"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("execplan:"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("stable-ref:"),
        "stdout: {}",
        output.stdout
    );
}
