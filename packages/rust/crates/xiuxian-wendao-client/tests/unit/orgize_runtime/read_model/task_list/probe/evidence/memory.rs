use crate::orgize_runtime::read_model::task_list::probe::support::{
    assert_cli_success, run_orgize, tempdir_or_panic,
};

#[test]
fn standalone_orgize_task_probe_recalls_reflection_question_memory_objects() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("reflection_memory_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Generic command cleanup :agent:org:memory:\n",
            ":PROPERTIES:\n",
            ":ID: generic-command-cleanup\n",
            ":NEXT_ACTION: Review command docs.\n",
            ":END:\n",
            "* DONE Runtime rename receipt [1/1] [100%] :agent:org:memory:ARCHIVE:\n",
            "CLOSED: [2026-05-24 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: runtime-rename-receipt\n",
            ":END:\n",
            "- [X] Rename the runtime command.\n",
            "** Reflection Questions\n",
            "| Question | Value |\n",
            "|---+---|\n",
            "| Which preference or naming correction should future generated plans preserve? | Use orgid-show and do not keep the legacy alias. |\n",
            "| Which failure mode should future agents avoid? | Do not reintroduce the old misspelled command name. |\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "legacy alias",
            "--include-done",
            "--include-archived",
            "--limit",
            "2",
            "reflection_memory_lane.org",
        ],
        "task-probe reflection memory objects",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Runtime rename receipt"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id runtime-rename-receipt"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_prefers_specific_memory_tokens_over_weak_org_token() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("mixed_memory_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Audio OpenRouter Qwen3 reference gate :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: audio-reference-task\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-qwen3-reference-gate\n",
            ":NEXT_ACTION: Curate the private 20-row Org reference review checklist.\n",
            ":END:\n",
            "* TODO Memory engine recall calibration :agent:org:memory:\n",
            ":PROPERTIES:\n",
            ":ID: memory-recall-task\n",
            ":PACKAGE: xiuxian-wendao-client\n",
            ":SLICE: org-memory-recall-accuracy\n",
            ":NEXT_ACTION: Implement temporary memory multi-signal ranking.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "latest memory engine org recall",
            "--limit",
            "2",
            "mixed_memory_lane.org",
        ],
        "task-probe weak org token",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .starts_with("title: Memory engine recall calibration"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id memory-recall-task"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_uses_memory_lifecycle_prior_for_ranking() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("memory_lifecycle_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Scoped episodic recall prior :agent:memory:\n",
            ":PROPERTIES:\n",
            ":ID: scoped-episodic-memory\n",
            ":MEMORY_LAYER: episodic\n",
            ":MEMORY_STATUS: closed\n",
            ":RECALL_DEFAULT: scoped\n",
            ":REUSABLE_KNOWLEDGE: lifecycle prior invariant selects durable recall policy.\n",
            ":END:\n",
            "* TODO Long term knowledge recall prior :agent:memory:\n",
            ":PROPERTIES:\n",
            ":ID: knowledge-memory\n",
            ":MEMORY_LAYER: knowledge\n",
            ":MEMORY_STATUS: active\n",
            ":RECALL_DEFAULT: yes\n",
            ":REUSABLE_KNOWLEDGE: lifecycle prior invariant selects durable recall policy.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "lifecycle prior invariant durable recall policy",
            "--limit",
            "1",
            "memory_lifecycle_lane.org",
        ],
        "task-probe memory lifecycle prior",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .starts_with("title: Long term knowledge recall prior"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id knowledge-memory"),
        "stdout: {}",
        output.stdout
    );
}
