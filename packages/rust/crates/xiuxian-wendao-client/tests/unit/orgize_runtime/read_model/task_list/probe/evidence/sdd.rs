use crate::orgize_runtime::read_model::task_list::probe::support::{
    assert_cli_success, run_orgize, tempdir_or_panic,
};

#[test]
fn standalone_orgize_task_probe_recalls_from_linked_sdd_architecture_text() {
    let temp = tempdir_or_panic();
    let sdd_dir = temp.path().join(".cache").join("agent").join("sdd");
    std::fs::create_dir_all(&sdd_dir).unwrap_or_else(|error| panic!("create sdd dir: {error}"));
    std::fs::write(
        sdd_dir.join("control_projection.org"),
        concat!(
            "#+TITLE: Agent Org DuckDB Control Projection\n",
            "* Agent Org DuckDB Control Projection :sdd:system:\n",
            ":PROPERTIES:\n",
            ":ID: sdd-control-projection\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: accepted\n",
            ":END:\n",
            "** Temporary Memory Reasoning Tree View :sdd:view:\n",
            ":PROPERTIES:\n",
            ":ID: sdd-temporary-memory-view\n",
            ":SDD_PARENT: id:sdd-control-projection\n",
            ":SDD_KIND: view\n",
            ":SDD_STATUS: accepted\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write sdd: {error}"));
    let agenda = temp.path().join("sdd_recall_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Generic DuckDB cleanup :agent:\n",
            ":PROPERTIES:\n",
            ":ID: generic-duckdb-task\n",
            ":NEXT_ACTION: Review unrelated storage notes.\n",
            ":END:\n",
            "* TODO Recovery lens implementation :agent:org:memory:\n",
            ":PROPERTIES:\n",
            ":ID: sdd-linked-task\n",
            ":SDD: .cache/agent/sdd/control_projection.org\n",
            ":NEXT_ACTION: Add bounded linked SDD evidence to task-probe ranking.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "temporary memory reasoning tree control projection",
            "--limit",
            "2",
            "sdd_recall_lane.org",
        ],
        "task-probe linked sdd",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .starts_with("title: Recovery lens implementation"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id sdd-linked-task"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        !output
            .stdout
            .contains("Temporary Memory Reasoning Tree View"),
        "SDD body should be ranking evidence, not probe output: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_recalls_from_file_key_tokens() {
    let temp = tempdir_or_panic();
    let target = temp
        .path()
        .join("wendao_client_org_memory_recall_accuracy.org");
    let other = temp.path().join("audio_openrouter_lane.org");
    std::fs::write(
        &target,
        concat!(
            "* TODO Generic recovery slice :agent:org:\n",
            ":PROPERTIES:\n",
            ":ID: file-key-task\n",
            ":PACKAGE: xiuxian-wendao-client\n",
            ":NEXT_ACTION: Continue the active recall scorer.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write target agenda: {error}"));
    std::fs::write(
        &other,
        concat!(
            "* TODO Memory wording in another lane :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: other-task\n",
            ":NEXT_ACTION: Review memory wording in an audio note.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write other agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "org memory recall accuracy",
            "--limit",
            "2",
            "audio_openrouter_lane.org",
            "wendao_client_org_memory_recall_accuracy.org",
        ],
        "task-probe file key",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Generic recovery slice"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id file-key-task"),
        "stdout: {}",
        output.stdout
    );
}
