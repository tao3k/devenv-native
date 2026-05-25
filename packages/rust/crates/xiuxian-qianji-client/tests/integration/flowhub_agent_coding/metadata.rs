use super::support::{FlowhubTestProject, copy_agent_coding_pair, rendered_json, run};

#[test]
fn lint_rejects_generated_metadata_drift() {
    let project = FlowhubTestProject::live();
    let init_output = run(
        &project.init_args("agent-coding", None, false),
        "agent-coding init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);

    let org_task = project.cache_home.join("agent/org/agent_coding.org");
    let original = std::fs::read_to_string(&org_task)
        .unwrap_or_else(|error| panic!("generated Org task should be readable: {error}"));
    let drifted = original.replace(
        ":BPMN_PROCESS_ID: agent_coding",
        ":BPMN_PROCESS_ID: wrong_process",
    );
    std::fs::write(&org_task, drifted)
        .unwrap_or_else(|error| panic!("generated Org task should be editable: {error}"));

    let lint_output = run(&project.lint_args(None, None, true), "metadata drift lint");
    assert!(!lint_output.passed, "{}", lint_output.rendered);
    let rendered = rendered_json(&lint_output);
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], false);
    assert_eq!(rendered["validation"]["orgLintPassed"], true);
    assert!(
        rendered["validation"]["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("diagnostics should be an array"))
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|message| message.contains("BPMN_PROCESS_ID `wrong_process`")))
    );
    assert!(
        rendered["validation"]["generatedMetadataFailures"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedMetadataFailures should be an array"))
            .iter()
            .any(|failure| failure["key"] == "BPMN_PROCESS_ID"
                && failure["actual"] == "wrong_process"
                && failure["expected"] == "agent_coding")
    );
}

#[test]
fn init_rejects_existing_slug_drift_without_overwrite() {
    let project = FlowhubTestProject::live();
    let init_args = project.init_args("agent-coding", None, true);
    let init_output = run(&init_args, "agent-coding init");
    assert!(init_output.passed, "{}", init_output.rendered);

    let org_task = project.cache_home.join("agent/org/agent_coding.org");
    let original = std::fs::read_to_string(&org_task)
        .unwrap_or_else(|error| panic!("generated Org task should be readable: {error}"));
    let drifted = original.replace(":FLOWHUB_SLUG: agent-coding", ":FLOWHUB_SLUG: stale-plan");
    std::fs::write(&org_task, drifted)
        .unwrap_or_else(|error| panic!("generated Org task should be editable: {error}"));

    let second_init_output = run(&init_args, "drifted init");
    assert!(
        !second_init_output.passed,
        "{}",
        second_init_output.rendered
    );
    let rendered = rendered_json(&second_init_output);
    assert_eq!(rendered["action"], "init");
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], false);
    assert_eq!(rendered["validation"]["orgLintPassed"], true);
    assert!(
        rendered["generatedFiles"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedFiles should be an array"))
            .iter()
            .all(|file| file["created"] == false)
    );
    assert!(
        rendered["validation"]["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("diagnostics should be an array"))
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|message| message.contains("FLOWHUB_SLUG `stale-plan`")))
    );
    assert!(
        rendered["validation"]["generatedMetadataFailures"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedMetadataFailures should be an array"))
            .iter()
            .any(|failure| failure["key"] == "FLOWHUB_SLUG"
                && failure["actual"] == "stale-plan"
                && failure["expected"] == "agent-coding")
    );

    let after_second_init = std::fs::read_to_string(&org_task)
        .unwrap_or_else(|error| panic!("generated Org task should remain readable: {error}"));
    assert!(after_second_init.contains(":FLOWHUB_SLUG: stale-plan"));
}

#[test]
fn registry_resolves_non_default_scenario_from_org_properties() {
    let project = FlowhubTestProject::live();

    let output = run(
        &project.init_args("deep_read", Some("paper-deep-read"), false),
        "deep_read source-pair init",
    );
    assert!(output.passed, "{}", output.rendered);
    assert!(output.rendered.contains("Flowhub contract: passed"));
    let org_task = project.cache_home.join("agent/org/paper_deep_read.org");
    assert!(
        project
            .cache_home
            .join("agent/sdd/paper_deep_read.org")
            .is_file()
    );
    assert!(org_task.is_file());
    let org_task_source = std::fs::read_to_string(&org_task)
        .unwrap_or_else(|error| panic!("generated Org task should be readable: {error}"));
    assert!(org_task_source.contains(":FLOWHUB_SLUG: paper-deep-read"));
    assert!(org_task_source.contains(":FLOWHUB_SCENARIO_ID: deep_read"));
    assert!(org_task_source.contains("paper-deep-read.org"));
    assert!(org_task_source.contains(":BPMN_PROCESS_ID: paper_deep_read"));
}

#[test]
fn lint_inferrs_non_default_scenario_from_generated_files() {
    let project = FlowhubTestProject::live();
    let init_output = run(
        &project.init_args("deep_read", Some("paper-deep-read"), false),
        "deep_read init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);

    let lint_output = run(
        &project.lint_args(None, Some("paper-deep-read"), true),
        "inferred scenario lint",
    );
    assert!(lint_output.passed, "{}", lint_output.rendered);
    let rendered = rendered_json(&lint_output);
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], true);
    assert_eq!(rendered["validation"]["orgLintPassed"], true);
}

#[test]
fn lint_rejects_inconsistent_generated_scenario_ids() {
    let project = FlowhubTestProject::live();
    let init_output = run(
        &project.init_args("deep_read", Some("paper-deep-read"), false),
        "deep_read init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);

    let execplan_path = project
        .cache_home
        .join("agent/execplans/paper_deep_read.org");
    let original = std::fs::read_to_string(&execplan_path)
        .unwrap_or_else(|error| panic!("generated ExecPlan should be readable: {error}"));
    let drifted = original.replace(
        ":FLOWHUB_SCENARIO_ID: deep_read",
        ":FLOWHUB_SCENARIO_ID: agent-coding",
    );
    std::fs::write(&execplan_path, drifted)
        .unwrap_or_else(|error| panic!("generated ExecPlan should be editable: {error}"));

    let lint_output = run(
        &project.lint_args(None, Some("paper-deep-read"), true),
        "inconsistent scenario lint",
    );
    assert!(!lint_output.passed, "{}", lint_output.rendered);
    let rendered = rendered_json(&lint_output);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], false);
    assert!(
        rendered["validation"]["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("diagnostics should be an array"))
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|message| message.contains("FLOWHUB_SCENARIO_ID `deep_read`")))
    );
    assert!(
        rendered["validation"]["generatedMetadataFailures"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedMetadataFailures should be an array"))
            .iter()
            .any(|failure| failure["key"] == "FLOWHUB_SCENARIO_ID"
                && failure["actual"] == "deep_read"
                && failure["expected"] == "agent-coding")
    );
}

#[test]
fn lint_rejects_flowhub_org_source_hash_drift() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");

    let init_output = run(
        &project.init_args("agent-coding", None, false),
        "agent-coding init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);

    let org_source = project.flowhub_root.join("plan/agent-coding.org");
    std::fs::write(
        &org_source,
        format!(
            "{}\n** Source Hash Drift\n\nThis valid Org change should force generated metadata drift.\n",
            std::fs::read_to_string(&org_source)
                .unwrap_or_else(|error| panic!("Flowhub Org source should be readable: {error}"))
        ),
    )
    .unwrap_or_else(|error| panic!("Flowhub Org source should be editable: {error}"));

    let lint_output = run(
        &project.lint_args(None, None, true),
        "source hash drift lint",
    );
    assert!(!lint_output.passed, "{}", lint_output.rendered);
    let rendered = rendered_json(&lint_output);
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], false);
    assert_eq!(rendered["validation"]["orgLintPassed"], true);
    assert!(
        rendered["validation"]["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("diagnostics should be an array"))
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|message| message.contains("FLOWHUB_ORG_SHA256")))
    );
    assert!(
        rendered["validation"]["generatedMetadataFailures"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedMetadataFailures should be an array"))
            .iter()
            .any(|failure| failure["key"] == "FLOWHUB_ORG_SHA256"
                && failure["actual"].as_str().is_some()
                && failure["expected"].as_str().is_some())
    );
}

#[test]
fn lint_all_rejects_flowhub_org_source_hash_drift() {
    let project = FlowhubTestProject::isolated_flowhub();
    copy_agent_coding_pair(&project, "plan");

    let init_output = run(
        &project.init_args("agent-coding", None, false),
        "agent-coding init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);

    let org_source = project.flowhub_root.join("plan/agent-coding.org");
    std::fs::write(
        &org_source,
        format!(
            "{}\n** Lint All Source Hash Drift\n\nThis valid Org change should force generated metadata drift.\n",
            std::fs::read_to_string(&org_source)
                .unwrap_or_else(|error| panic!("Flowhub Org source should be readable: {error}"))
        ),
    )
    .unwrap_or_else(|error| panic!("Flowhub Org source should be editable: {error}"));

    let lint_output = run(&project.lint_all_args(true), "lint all source drift");
    assert!(!lint_output.passed, "{}", lint_output.rendered);
    let rendered = rendered_json(&lint_output);
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], false);
    assert_eq!(rendered["validation"]["orgLintPassed"], true);
    assert!(
        rendered["validation"]["generatedMetadataFailures"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedMetadataFailures should be an array"))
            .iter()
            .any(|failure| failure["key"] == "FLOWHUB_ORG_SHA256"
                && failure["actual"].as_str().is_some()
                && failure["expected"].as_str().is_some())
    );
}
