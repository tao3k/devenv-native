use super::support::{FlowhubTestProject, rendered_json, run, run_error};

#[test]
fn init_generates_and_lints_tracking_surface() {
    let project = FlowhubTestProject::live();

    let init_output = run(
        &project.init_args("agent-coding", None, false),
        "agent-coding init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);
    assert_eq!(init_output.generated_paths.len(), 3);
    assert!(
        project
            .cache_home
            .join("agent/sdd/agent_coding.org")
            .is_file()
    );
    assert!(
        project
            .cache_home
            .join("agent/org/agent_coding.org")
            .is_file()
    );
    assert!(
        project
            .cache_home
            .join("agent/execplans/agent_coding.org")
            .is_file()
    );

    let lint_output = run(&project.lint_args(None, None, false), "agent-coding lint");
    assert!(lint_output.passed, "{}", lint_output.rendered);
    assert!(lint_output.rendered.contains("Flowhub contract: passed"));
    assert!(lint_output.rendered.contains("Generated files: passed"));
    assert!(lint_output.rendered.contains("Org lint: passed"));
}

#[test]
fn default_lint_reports_missing_generated_files() {
    let project = FlowhubTestProject::live();

    let output = run(
        &project.lint_args(None, None, false),
        "missing generated file lint",
    );
    assert!(!output.passed, "{}", output.rendered);
    assert!(output.rendered.contains("Generated files: failed"));
}

#[test]
fn flowhub_check_is_not_supported_as_lint_alias() {
    let project = FlowhubTestProject::live();
    let mut args = project.flowhub_args(&["check"]);
    args.push("--json".to_string());

    let message = run_error(&args, "flowhub check alias");
    assert!(message.contains("unsupported qianji-client flowhub argument `check`"));
    assert!(message.contains("qianji-client flowhub lint"));
}

#[test]
fn lint_json_reports_missing_generated_files() {
    let project = FlowhubTestProject::live();

    let output = run(
        &project.lint_args(None, None, true),
        "missing generated file JSON lint",
    );
    assert!(!output.passed, "{}", output.rendered);
    let rendered = rendered_json(&output);
    assert_eq!(rendered["action"], "lint");
    assert_eq!(rendered["passed"], false);
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], false);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], false);
    assert_eq!(rendered["validation"]["orgLintPassed"], false);
    assert!(
        rendered["validation"]["diagnostics"]
            .as_array()
            .unwrap_or_else(|| panic!("diagnostics should be an array"))
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|message| message.contains("missing generated agent tracking file")))
    );
}

#[test]
fn init_and_lint_json_render_machine_readable_receipts() {
    let project = FlowhubTestProject::live();

    let init_output = run(
        &project.init_args("agent-coding", None, true),
        "agent-coding JSON init",
    );
    assert!(init_output.passed, "{}", init_output.rendered);
    let init_rendered = rendered_json(&init_output);
    assert_eq!(init_rendered["action"], "init");
    assert_eq!(init_rendered["passed"], true);
    assert_eq!(init_rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(init_rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(
        init_rendered["validation"]["generatedMetadataMatched"],
        true
    );
    assert_eq!(init_rendered["validation"]["orgLintPassed"], true);
    assert_eq!(
        init_rendered["generatedFiles"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedFiles should be an array"))
            .len(),
        3
    );

    let lint_output = run(
        &project.lint_args(None, None, true),
        "agent-coding JSON lint",
    );
    assert!(lint_output.passed, "{}", lint_output.rendered);
    let lint_rendered = rendered_json(&lint_output);
    assert_eq!(lint_rendered["action"], "lint");
    assert_eq!(lint_rendered["passed"], true);
    assert_eq!(lint_rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(lint_rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(
        lint_rendered["validation"]["generatedMetadataMatched"],
        true
    );
    assert_eq!(lint_rendered["validation"]["orgLintPassed"], true);
    assert_eq!(
        lint_rendered["generatedFiles"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedFiles should be an array"))
            .len(),
        3
    );
}

#[test]
fn lint_all_validates_every_generated_tracking_surface() {
    let project = FlowhubTestProject::live();

    let agent_output = run(
        &project.init_args("agent-coding", None, false),
        "agent-coding init",
    );
    assert!(agent_output.passed, "{}", agent_output.rendered);
    let paper_output = run(
        &project.init_args("deep_read", Some("paper-deep-read"), false),
        "deep_read init",
    );
    assert!(paper_output.passed, "{}", paper_output.rendered);
    std::fs::write(
        project.cache_home.join("agent/org/unrelated.org"),
        "* TODO Unrelated local agent task\n:PROPERTIES:\n:STATUS: active\n:END:\n",
    )
    .unwrap_or_else(|error| panic!("unrelated Org task should be writable: {error}"));

    let lint_output = run(&project.lint_all_args(true), "lint all generated plans");
    assert!(lint_output.passed, "{}", lint_output.rendered);
    let rendered = rendered_json(&lint_output);
    assert_eq!(rendered["action"], "lint");
    assert_eq!(rendered["passed"], true);
    assert_eq!(rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(rendered["validation"]["generatedMetadataMatched"], true);
    assert_eq!(rendered["validation"]["orgLintPassed"], true);
    assert_eq!(
        rendered["generatedFiles"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedFiles should be an array"))
            .len(),
        6
    );
}
