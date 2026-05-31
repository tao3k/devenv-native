use crate::orgize_runtime::read_model::task_list::probe::support::{
    assert_cli_success, run_orgize, tempdir_or_panic,
};

#[test]
fn standalone_orgize_task_probe_suppresses_unarchived_completed_checklists() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("completion_noise_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO OpenRouter lane [2/2] [100%] :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: completed-openrouter-task\n",
            ":NEXT_ACTION: Completed task should not be active recall noise.\n",
            ":END:\n",
            "** Task Checklist [2/2] [100%]\n",
            "- [X] Finish one.\n",
            "- [X] Finish two.\n",
            "* TODO OpenRouter lane [1/2] [50%] :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: active-openrouter-task\n",
            ":NEXT_ACTION: Continue active unfinished task.\n",
            ":END:\n",
            "** Task Checklist [1/2] [50%]\n",
            "- [X] Finish one.\n",
            "- [ ] Finish two.\n",
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
            "completion_noise_lane.org",
        ],
        "task-probe completion noise",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .contains("next: Continue active unfinished task."),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output
            .stdout
            .contains("Completed task should not be active recall noise."),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_keeps_exact_completed_orgid_when_included() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("completed_identity_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Active memory followup :agent:memory:\n",
            ":PROPERTIES:\n",
            ":ID: active-memory-task\n",
            ":NEXT_ACTION: Continue the active task.\n",
            ":END:\n",
            "* DONE Completed memory recall [1/1] [100%] :agent:memory:ARCHIVE:\n",
            "CLOSED: [2026-05-24 Sun]\n",
            ":PROPERTIES:\n",
            ":ID: completed-memory-task\n",
            ":NEXT_ACTION: Review archived memory evidence.\n",
            ":END:\n",
            "** Task Checklist [1/1] [100%]\n",
            "- [X] Finish archived recall.\n",
            "** Reflection\n",
            "- Summary: Archived memory recall evidence is available.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "completed-memory-task",
            "--include-done",
            "--include-archived",
            "--limit",
            "2",
            "completed_identity_lane.org",
        ],
        "task-probe completed identity",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Completed memory recall"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id completed-memory-task"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output.stdout.contains("title: Active memory followup"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_recalls_from_org_planning_timestamp() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("temporal_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Generic task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: generic-task\n",
            ":END:\n",
            "* TODO Temporal recovery task :agent:\n",
            "SCHEDULED: <2026-05-24 Sun>\n",
            ":PROPERTIES:\n",
            ":ID: temporal-task\n",
            ":NEXT_ACTION: Continue scheduled recovery.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &["task-probe", "--text", "2026-05-24", "temporal_lane.org"],
        "task-probe planning timestamp",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Temporal recovery task"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id temporal-task"),
        "stdout: {}",
        output.stdout
    );
}
