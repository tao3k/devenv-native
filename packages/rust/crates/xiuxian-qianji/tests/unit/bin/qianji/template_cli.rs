use super::*;
use qianji_bpmn_engine::{BpmnSourceFile, DmnSourceFile, lint_bpmn_source, lint_dmn_source};

#[test]
fn parse_template_command_accepts_bpmn_target() {
    let command = must_some(
        must_ok(
            parse_template_command(&to_args(&["qianji", "template", "--bpmn"])),
            "template parse should succeed",
        ),
        "template command should be detected",
    );

    assert_eq!(command, TemplateCliCommand::Bpmn);
}

#[test]
fn parse_template_command_accepts_dmn_target() {
    let command = must_some(
        must_ok(
            parse_template_command(&to_args(&["qianji", "template", "--dmn"])),
            "template parse should succeed",
        ),
        "template command should be detected",
    );

    assert_eq!(command, TemplateCliCommand::Dmn);
}

#[test]
fn parse_template_command_rejects_ambiguous_target() {
    let error = parse_template_command(&to_args(&["qianji", "template", "--bpmn", "--dmn"]))
        .expect_err("ambiguous template target should fail");

    assert!(error.to_string().contains("exactly one"));
}

#[test]
fn run_template_command_renders_lint_clean_bpmn() {
    let output = must_ok(
        run_template_command(TemplateCliCommand::Bpmn),
        "template command should render BPMN",
    );
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "template.bpmn".to_string(),
        output.rendered.clone(),
    ));

    assert!(output.rendered.contains("<serviceTask"));
    assert!(output.rendered.contains("skillsc:config"));
    assert!(report.ok, "BPMN template should lint clean: {report:?}");
}

#[test]
fn run_template_command_renders_lint_clean_dmn() {
    let output = must_ok(
        run_template_command(TemplateCliCommand::Dmn),
        "template command should render DMN",
    );
    let report = lint_dmn_source(&DmnSourceFile::new(
        "template.dmn".to_string(),
        output.rendered.clone(),
    ));

    assert!(output.rendered.contains("<decisionTable"));
    assert!(report.ok, "DMN template should lint clean: {report:?}");
}
