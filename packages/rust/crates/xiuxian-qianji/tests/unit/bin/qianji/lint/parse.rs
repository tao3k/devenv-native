use super::*;

#[test]
fn parse_lint_command_accepts_inferred_target() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&["qianji", "lint", "plan.json"])),
            "lint parse should succeed",
        ),
        "lint command should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::Auto {
            path: PathBuf::from("plan.json")
        }
    );
}

#[test]
fn parse_lint_command_accepts_bpmn_target() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&[
                "qianji",
                "lint",
                "--bpmn",
                "fixtures/sample.bpmn",
            ])),
            "lint parse should succeed",
        ),
        "lint command should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::Bpmn {
            path: PathBuf::from("fixtures/sample.bpmn")
        }
    );
}

#[test]
fn parse_lint_command_accepts_json_output() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&[
                "qianji",
                "lint",
                "--bpmn",
                "fixtures/sample.bpmn",
                "--json",
            ])),
            "lint parse should succeed",
        ),
        "lint command should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::BpmnJson {
            path: PathBuf::from("fixtures/sample.bpmn")
        }
    );
}

#[test]
fn parse_lint_command_accepts_linter_alias_for_dmn_target() {
    let command = must_some(
        must_ok(
            parse_lint_command(&to_args(&[
                "qianji",
                "linter",
                "--dmn",
                "fixtures/sample.dmn",
            ])),
            "linter alias parse should succeed",
        ),
        "linter alias should be detected",
    );

    assert_eq!(
        command,
        LintCliCommand::Dmn {
            path: PathBuf::from("fixtures/sample.dmn")
        }
    );
}

#[test]
fn parse_lint_command_rejects_mixed_targets() {
    let error = match parse_lint_command(&to_args(&[
        "qianji", "lint", "--bpmn", "a.bpmn", "--dmn", "b.dmn",
    ])) {
        Ok(command) => panic!("mixed lint targets should fail, got {command:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("requires exactly one target path")
    );
}
