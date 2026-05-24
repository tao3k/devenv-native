use super::support::{FlowhubTestProject, rendered_json, run};

#[test]
fn init_generates_and_checks_tracking_surface() {
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

    let check_output = run(&project.check_args(None, None, false), "agent-coding check");
    assert!(check_output.passed, "{}", check_output.rendered);
    assert!(check_output.rendered.contains("Flowhub contract: passed"));
    assert!(check_output.rendered.contains("Generated files: passed"));
    assert!(check_output.rendered.contains("Org lint: passed"));
}

#[test]
fn default_check_reports_missing_generated_files() {
    let project = FlowhubTestProject::live();

    let output = run(
        &project.check_args(None, None, false),
        "missing generated file check",
    );
    assert!(!output.passed, "{}", output.rendered);
    assert!(output.rendered.contains("Generated files: failed"));
}

#[test]
fn check_json_reports_missing_generated_files() {
    let project = FlowhubTestProject::live();

    let output = run(
        &project.check_args(None, None, true),
        "missing generated file JSON check",
    );
    assert!(!output.passed, "{}", output.rendered);
    let rendered = rendered_json(&output);
    assert_eq!(rendered["action"], "check");
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
fn init_and_check_json_render_machine_readable_receipts() {
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

    let check_output = run(
        &project.check_args(None, None, true),
        "agent-coding JSON check",
    );
    assert!(check_output.passed, "{}", check_output.rendered);
    let check_rendered = rendered_json(&check_output);
    assert_eq!(check_rendered["action"], "check");
    assert_eq!(check_rendered["passed"], true);
    assert_eq!(check_rendered["validation"]["flowhubContractPassed"], true);
    assert_eq!(check_rendered["validation"]["generatedFilesPresent"], true);
    assert_eq!(
        check_rendered["validation"]["generatedMetadataMatched"],
        true
    );
    assert_eq!(check_rendered["validation"]["orgLintPassed"], true);
    assert_eq!(
        check_rendered["generatedFiles"]
            .as_array()
            .unwrap_or_else(|| panic!("generatedFiles should be an array"))
            .len(),
        3
    );
}
