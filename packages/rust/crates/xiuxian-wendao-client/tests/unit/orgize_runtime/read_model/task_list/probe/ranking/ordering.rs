use crate::orgize_runtime::read_model::task_list::probe::support::{
    assert_cli_success, run_orgize, tempdir_or_panic,
};

#[test]
fn standalone_orgize_task_probe_reranks_candidates_with_evidence_window_features() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("audio_openrouter_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO OpenRouter generic followup :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: generic-task\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-generic\n",
            ":NEXT_ACTION: Check Qwen3 reference notes if needed.\n",
            ":END:\n",
            "* TODO Audio Qwen3 reference gate :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: qwen3-reference-task\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-qwen3-reference-gate\n",
            ":NEXT_ACTION: Close curated reference review.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "Qwen3 reference",
            "--limit",
            "2",
            "audio_openrouter_lane.org",
        ],
        "task-probe",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .starts_with("title: Audio Qwen3 reference gate"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id qwen3-reference-task"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_prefers_coherent_next_action_over_distributed_tokens() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("next_action_coherence_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Source cleanup :agent:contract:\n",
            ":PROPERTIES:\n",
            ":ID: distributed-token-task\n",
            ":NEXT_ACTION: Continue generic cleanup.\n",
            ":END:\n",
            "* TODO Repair slice :agent:\n",
            ":PROPERTIES:\n",
            ":ID: coherent-next-action-task\n",
            ":NEXT_ACTION: Continue source contract repair.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "source contract",
            "--limit",
            "2",
            "next_action_coherence_lane.org",
        ],
        "task-probe next action coherence",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Repair slice"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains(
            "show: wendao-client orgize orgid-show --cached --id coherent-next-action-task"
        ),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_prefers_recent_modified_candidate_when_memory_signal_ties() {
    let temp = tempdir_or_panic();
    let old_agenda = temp.path().join("a_openrouter_lane.org");
    let new_agenda = temp.path().join("z_openrouter_lane.org");
    std::fs::write(
        &old_agenda,
        concat!(
            "* TODO OpenRouter lane :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: old-openrouter-task\n",
            ":NEXT_ACTION: Continue older context.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write old agenda: {error}"));
    std::thread::sleep(std::time::Duration::from_millis(25));
    std::fs::write(
        &new_agenda,
        concat!(
            "* TODO OpenRouter lane :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: new-openrouter-task\n",
            ":NEXT_ACTION: Continue newest context.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write new agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "OpenRouter lane",
            "--limit",
            "2",
            "a_openrouter_lane.org",
            "z_openrouter_lane.org",
        ],
        "task-probe temporal recency",
    );

    assert_cli_success(&output);
    let newest = output
        .stdout
        .find("next: Continue newest context.")
        .unwrap_or_else(|| panic!("stdout: {}", output.stdout));
    let older = output
        .stdout
        .find("next: Continue older context.")
        .unwrap_or_else(|| panic!("stdout: {}", output.stdout));
    assert!(newest < older, "stdout: {}", output.stdout);
}
#[test]
fn standalone_orgize_task_probe_uses_section_order_when_file_timestamp_ties() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("single_file_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO OpenRouter lane :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: earlier-openrouter-task\n",
            ":NEXT_ACTION: Continue earlier section.\n",
            ":END:\n",
            "* TODO OpenRouter lane :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: later-openrouter-task\n",
            ":NEXT_ACTION: Continue later section.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "OpenRouter lane",
            "--limit",
            "2",
            "single_file_lane.org",
        ],
        "task-probe section order",
    );

    assert_cli_success(&output);
    let later = output
        .stdout
        .find("next: Continue later section.")
        .unwrap_or_else(|| panic!("stdout: {}", output.stdout));
    let earlier = output
        .stdout
        .find("next: Continue earlier section.")
        .unwrap_or_else(|| panic!("stdout: {}", output.stdout));
    assert!(later < earlier, "stdout: {}", output.stdout);
}
