use crate::orgize_runtime::support::{assert_cli_success, run_orgize, tempdir_or_panic};

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
            .contains("show: wendao-client orgize ogrid-show --cached --id qwen3-reference-task"),
        "stdout: {}",
        output.stdout
    );
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
            .contains("show: wendao-client orgize ogrid-show --cached --id openrouter-task"),
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
            "show: wendao-client orgize ogrid-show --cached --id coherent-next-action-task"
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
            .contains("show: wendao-client orgize ogrid-show --cached --id completed-memory-task"),
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
            .contains("show: wendao-client orgize ogrid-show --cached --id temporal-task"),
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
            .contains("show: wendao-client orgize ogrid-show --cached --id cjk-memory-task"),
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

fn write_audio_openrouter_probe_agenda(agenda: &std::path::Path) {
    std::fs::write(
        agenda,
        concat!(
            "* TODO Audio OpenRouter gate <2026-05-23 Sat> :agent:audio:\n",
            ":PROPERTIES:\n",
            ":ID: audio-gate\n",
            ":PACKAGE: xiuxian-wendao-analyzer\n",
            ":SLICE: audio-openrouter-gate\n",
            ":SDD: .cache/agent/sdd/audio.org\n",
            ":NEXT_ACTION: Curate reference rows\n",
            ":END:\n",
            "** Evidence\n",
            "Large body should not render in a memory probe.\n",
            "* TODO Other task :agent:\n",
            ":PROPERTIES:\n",
            ":ID: other-task\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write agenda: {error}"));
}

fn assert_audio_openrouter_probe_output(stdout: &str) {
    assert!(
        stdout.contains("title: Audio OpenRouter gate <2026-05-23 Sat>"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("[PROBE"), "stdout: {stdout}");
    assert!(!stdout.contains("backend:"), "stdout: {stdout}");
    assert!(!stdout.contains("database:"), "stdout: {stdout}");
    assert!(!stdout.contains("rows:"), "stdout: {stdout}");
    assert!(!stdout.contains("orgid: audio-gate"), "stdout: {stdout}");
    assert!(!stdout.contains("state: TODO"), "stdout: {stdout}");
    assert!(
        !stdout.contains("file-key: audio_openrouter_lane"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("status: active"), "stdout: {stdout}");
    assert_probe_metadata(stdout);
    assert!(
        stdout.contains("show: wendao-client orgize ogrid-show --cached --id audio-gate"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(
            "graph: flowchart LR;T[\"task:audio-gate\"]-->N0[\"sdd:.cache/agent/sdd/audio.org\"]"
        ),
        "stdout: {stdout}"
    );
    assert!(
        !stdout.contains("Large body should not render"),
        "stdout: {stdout}"
    );
}

fn assert_probe_metadata(stdout: &str) {
    assert!(
        stdout.contains("package: xiuxian-wendao-analyzer"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("slice: audio-openrouter-gate"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("\nsdd:"), "stdout: {stdout}");
    assert!(
        stdout.contains(
            "graph: flowchart LR;T[\"task:audio-gate\"]-->N0[\"sdd:.cache/agent/sdd/audio.org\"]"
        ),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("next: Curate reference rows"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("execplan:"), "stdout: {stdout}");
    assert!(!stdout.contains("stable-ref:"), "stdout: {stdout}");
    assert!(!stdout.contains("query-title:"), "stdout: {stdout}");
    assert!(!stdout.contains("query-file:"), "stdout: {stdout}");
}

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
            .contains("show: wendao-client orgize ogrid-show --cached --id reference-task"),
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
            .contains("show: wendao-client orgize ogrid-show --cached --id sdd-linked-task"),
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
            .contains("show: wendao-client orgize ogrid-show --cached --id multi-facet-task"),
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
            .contains("show: wendao-client orgize ogrid-show --cached --id memory-recall-task"),
        "stdout: {}",
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
            .contains("show: wendao-client orgize ogrid-show --cached --id file-key-task"),
        "stdout: {}",
        output.stdout
    );
}

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
