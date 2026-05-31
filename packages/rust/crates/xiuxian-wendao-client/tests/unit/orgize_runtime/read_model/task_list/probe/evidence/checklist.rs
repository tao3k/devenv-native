use crate::orgize_runtime::read_model::task_list::probe::support::{
    assert_cli_success, run_orgize, tempdir_or_panic,
};

#[test]
fn standalone_orgize_task_probe_recalls_from_direct_checklist_text() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("audio_openrouter_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Generic OpenRouter cleanup :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: generic-task\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-generic\n",
            ":END:\n",
            "- [ ] Review provider configuration.\n",
            "* TODO Audio reference closure :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: reference-task\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-reference-closure\n",
            ":END:\n",
            "** Task Checklist [1/2] [50%]\n",
            ":PROPERTIES:\n",
            ":COOKIE_DATA: direct\n",
            ":END:\n",
            "- [X] Materialize private review rows.\n",
            "- [ ] Convert candidate-draft rows to curated reference rows and rerun the CER gate.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "candidate-draft rows",
            "audio_openrouter_lane.org",
        ],
        "task-probe",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Audio reference closure"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id reference-task"),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_prefers_org_facet_diversity_over_flat_title_match() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("facet_diversity_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Reference rows SDD notes :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: flat-title-task\n",
            ":END:\n",
            "* TODO Audio closure :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: facet-rich-task\n",
            ":SDD: .cache/agent/sdd/reference-sdd.org\n",
            ":END:\n",
            "** Task Checklist [0/1] [0%]\n",
            "- [ ] Curate rows.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "rows reference sdd",
            "--limit",
            "2",
            "facet_diversity_lane.org",
        ],
        "task-probe facet diversity",
    );

    assert_cli_success(&output);
    assert!(
        output.stdout.starts_with("title: Audio closure"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("graph: flowchart LR;T[\"task:facet-rich-task\"]-->N0[\"sdd:.cache/agent/sdd/reference-sdd.org\"]"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("checklist-progress: [0/1] [0%]"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output.stdout.contains("next-unchecked: - [ ] Curate rows."),
        "stdout: {}",
        output.stdout
    );
}
#[test]
fn standalone_orgize_task_probe_prefers_multi_facet_memory_over_title_phrase_decoy() {
    let temp = tempdir_or_panic();
    let agenda = temp.path().join("recommendation_rank_lane.org");
    std::fs::write(
        &agenda,
        concat!(
            "* TODO Memory engine signal fusion notes :agent:notes:\n",
            ":PROPERTIES:\n",
            ":ID: title-decoy-task\n",
            ":END:\n",
            "* TODO Recommendation calibration :agent:org:memory:\n",
            ":PROPERTIES:\n",
            ":ID: multi-facet-task\n",
            ":PACKAGE: xiuxian-wendao-client; xiuxian-memory-engine\n",
            ":SLICE: org-memory-signal-fusion\n",
            ":NEXT_ACTION: Tune memory recommendation rank fusion.\n",
            ":END:\n",
            "** Task Checklist [0/2] [0%]\n",
            "- [ ] Fuse memory ranking signals across Org facets.\n",
            "- [ ] Validate the recommendation comparator against title decoys.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));

    let output = run_orgize(
        temp.path(),
        &[
            "task-probe",
            "--text",
            "memory engine signal fusion",
            "--limit",
            "2",
            "recommendation_rank_lane.org",
        ],
        "task-probe recommendation rank fusion",
    );

    assert_cli_success(&output);
    assert!(
        output
            .stdout
            .starts_with("title: Recommendation calibration"),
        "stdout: {}",
        output.stdout
    );
    assert!(
        output
            .stdout
            .contains("show: wendao-client orgize orgid-show --cached --id multi-facet-task"),
        "stdout: {}",
        output.stdout
    );
}
