use crate::orgize_runtime::read_model::task_list::probe::support::{
    assert_audio_openrouter_probe_output, assert_cli_success, run_orgize, tempdir_or_panic,
    write_audio_openrouter_probe_agenda,
};

#[test]
fn standalone_orgize_task_probe_renders_compact_memory_candidates() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("audio_openrouter_lane.org");
    write_audio_openrouter_probe_agenda(&agenda);

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "Audio OpenRouter",
            "audio_openrouter_lane.org",
        ],
        "task-probe",
    );

    assert_cli_success(&output);
    assert_audio_openrouter_probe_output(&output.stdout);
}
#[test]
fn standalone_orgize_task_probe_splits_agent_camel_case_tokens() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("camel_case_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Audio OpenRouter gate :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: openrouter-task\n",
            ":NEXT_ACTION: Continue OpenRouter provider work.\n",
            ":END:\n",
            "* TODO Open reference cleanup :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: open-reference-task\n",
            ":NEXT_ACTION: Continue generic open reference work.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "open router",
            "--limit",
            "2",
            "camel_case_lane.org",
        ],
        "task-probe camel case",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Audio OpenRouter gate"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id openrouter-task"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_matches_cjk_agent_text_without_embedding_backend() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("cjk_memory_lane.org");
    let cjk_memory = "\u{8bb0}\u{5fc6}";
    let cjk_recall = "\u{53ec}\u{56de}";
    let cjk_next = "\u{7ee7}\u{7eed}\u{6821}\u{51c6}\u{4e34}\u{65f6}\u{8bb0}\u{5fc6}";
    std::fs::write(
        &agenda,
        format!(
            concat!(
                "* TODO Generic English memory task :agent:memory:\n",
                ":PROPERTIES:\n",
                ":ID: generic-memory-task\n",
                ":NEXT_ACTION: Continue generic memory work.\n",
                ":END:\n",
                "* TODO {} {} calibration :agent:memory:\n",
                ":PROPERTIES:\n",
                ":ID: cjk-memory-task\n",
                ":NEXT_ACTION: {}\n",
                ":END:\n",
            ),
            cjk_memory, cjk_recall, cjk_next
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
    let query = format!("{cjk_memory}{cjk_recall}");

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            query.as_str(),
            "--limit",
            "2",
            "cjk_memory_lane.org",
        ],
        "task-probe cjk",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .starts_with(format!("title: {cjk_memory} {cjk_recall} calibration").as_str()),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id cjk-memory-task"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_returns_empty_when_remembered_text_has_no_evidence() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("unmatched_memory_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Audio OpenRouter gate :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: audio-task\n",
            ":NEXT_ACTION: Continue audio provider work.\n",
            ":END:\n",
            "* TODO L2 artifact route :agent:l2:\n",
            ":PROPERTIES:\n",
            ":ID: l2-task\n",
            ":NEXT_ACTION: Continue route cache work.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "\u{8bb0}\u{5fc6}\u{53ec}\u{56de}",
            "--limit",
            "2",
            "unmatched_memory_lane.org",
        ],
        "task-probe unmatched remembered text",
    );

    assert_cli_success(&output);
    assert_eq!(output.stdout.trim(), "", "stdout: {}", output.stdout);
}
