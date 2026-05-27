use super::{
    TemplateCliCommand, must_ok, must_some, parse_template_command, run_template_command, to_args,
};
use crate::QianjiCompiler;
use std::sync::Arc;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji_bpmn_engine::{
    BpmnParseOptions, BpmnSourceFile, DmnSourceFile, lint_bpmn_source, lint_dmn_source,
    parse_bpmn_package, snapshot_bpmn_source,
};
use xiuxian_wendao::LinkGraphIndex;

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
fn parse_template_command_accepts_semantic_guard_route_target() {
    let command = must_some(
        must_ok(
            parse_template_command(&to_args(&["qianji", "template", "--semantic-guard-route"])),
            "template parse should succeed",
        ),
        "template command should be detected",
    );

    assert_eq!(command, TemplateCliCommand::SemanticGuardRoute);
}

#[test]
fn parse_template_command_rejects_ambiguous_target() {
    let error = parse_template_command(&to_args(&[
        "qianji",
        "template",
        "--bpmn",
        "--semantic-guard-route",
    ]))
    .err()
    .unwrap_or_else(|| panic!("ambiguous template target should fail"));

    assert!(error.to_string().contains("exactly one"));
}

#[test]
fn run_template_command_renders_native_bpmn_with_standard_di() {
    let output = run_template_command(&TemplateCliCommand::Bpmn);
    let source = BpmnSourceFile::new("template.bpmn".to_string(), output.rendered.clone());
    let report = lint_bpmn_source(&source);
    let snapshot = must_ok(
        snapshot_bpmn_source(&source),
        "BPMN template should snapshot cleanly",
    );
    must_ok(
        parse_bpmn_package(&[source], &BpmnParseOptions::default()),
        "BPMN template should parse cleanly",
    );

    assert!(output.rendered.contains("<serviceTask"));
    assert!(output.rendered.contains("<ioSpecification>"));
    assert!(output.rendered.contains("<dataOutput"));
    assert!(output.rendered.contains("xmlns:bpmndi"));
    assert!(output.rendered.contains("<bpmndi:BPMNDiagram"));
    assert!(output.rendered.contains("<dc:Bounds"));
    assert!(output.rendered.contains("<di:waypoint"));
    assert!(!output.rendered.contains("xmlns:qianji"));
    assert_eq!(snapshot.root.diagram_count, 1);
    let plane = snapshot.root.diagrams[0]
        .plane
        .as_ref()
        .unwrap_or_else(|| panic!("BPMN template should preserve a BPMNPlane"));
    assert_eq!(plane.shapes.len(), 3);
    assert_eq!(plane.edges.len(), 2);
    assert!(
        report.ok,
        "BPMN template should lint clean with native BPMNDI: {report:?}"
    );
    assert!(report.issues.is_empty());
}

#[test]
fn run_template_command_renders_lint_clean_dmn() {
    let output = run_template_command(&TemplateCliCommand::Dmn);
    let report = lint_dmn_source(&DmnSourceFile::new(
        "template.dmn".to_string(),
        output.rendered.clone(),
    ));

    assert!(output.rendered.contains("<decisionTable"));
    assert!(output.rendered.contains("https://qianji.dev/dmn"));
    assert!(report.ok, "DMN template should lint clean: {report:?}");
}

#[test]
fn run_template_command_renders_semantic_guard_route_manifest() {
    let output = run_template_command(&TemplateCliCommand::SemanticGuardRoute);

    assert!(output.rendered.contains("Semantic_Guard_Route_Branch_Test"));
    assert!(output.rendered.contains("semantic_guard_route = true"));
    assert!(output.rendered.contains("review_required"));
}

#[test]
fn run_template_command_renders_compilable_semantic_guard_route_manifest() {
    let output = run_template_command(&TemplateCliCommand::SemanticGuardRoute);
    let temp = tempfile::tempdir().unwrap_or_else(|error| {
        panic!("temporary index root should be created: {error}");
    });
    let index = Arc::new(LinkGraphIndex::build(temp.path()).unwrap_or_else(|error| {
        panic!("link graph index should build: {error}");
    }));
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    let compiler = QianjiCompiler::new(index, orchestrator, registry, None);

    compiler
        .compile(&output.rendered)
        .unwrap_or_else(|error| panic!("semantic guard route template should compile: {error}"));
}
