//! Qianhuan binding contract and runtime tests for Qianji nodes.

#![cfg(feature = "wendao-integration")]

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
use xiuxian_qianji::{QianjiCompiler, QianjiManifest, QianjiScheduler};
use xiuxian_wendao::link_graph::LinkGraphIndex;
use xiuxian_wendao_runtime::artifacts::zhixing::embedded_resource_text_from_wendao_uri;

#[test]
fn qianji_manifest_parses_node_level_qianhuan_binding() {
    let manifest_toml = r#"
name = "Binding_Interface_Contract"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "student_proposer"
template_target = "draft_reflection.md"
execution_mode = "appended"
input_keys = ["draft_reflection_xml", "validator_notes"]
history_key = "agenda_history"
output_key = "agenda_draft_xml"
"#;

    let manifest: QianjiManifest = toml::from_str(manifest_toml)
        .unwrap_or_else(|error| panic!("manifest should parse qianhuan binding: {error}"));
    let binding = manifest.nodes[0]
        .qianhuan
        .as_ref()
        .unwrap_or_else(|| panic!("qianhuan binding should be present"));

    assert_eq!(binding.persona_id.as_deref(), Some("student_proposer"));
    assert_eq!(
        binding.template_target.as_deref(),
        Some("draft_reflection.md")
    );
    assert_eq!(binding.execution_mode.as_str(), "appended");
    assert_eq!(
        binding.input_keys,
        vec![
            "draft_reflection_xml".to_string(),
            "validator_notes".to_string()
        ]
    );
    assert_eq!(binding.history_key.as_deref(), Some("agenda_history"));
    assert_eq!(binding.output_key.as_deref(), Some("agenda_draft_xml"));
}

#[tokio::test]
async fn annotation_node_uses_qianhuan_binding_fields() {
    let manifest_toml = r#"
name = "Binding_Runtime_Test"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "binding_tester"
template_target = "critique_report.md"
"#;

    let tmp = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(tmp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "binding_tester".to_string(),
        name: "Binding Tester".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: Vec::new(),
        cot_template: "1. Read -> 2. Validate -> 3. Return.".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), None);
    let engine = compiler
        .compile(manifest_toml)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({ "raw_facts": "structured facts for binding coverage" }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should execute successfully: {error}"));

    assert_eq!(output["annotated_persona_id"], "binding_tester");
    assert_eq!(output["annotated_template_target"], "critique_report.md");
    assert!(
        output["annotated_prompt"]
            .as_str()
            .is_some_and(|prompt| prompt.contains("<system_prompt_injection>"))
    );
}

#[tokio::test]
async fn annotation_node_supports_appended_mode_and_custom_output_key() {
    let manifest_toml = r#"
name = "Binding_Appended_Test"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "binding_tester"
template_target = "critique_report.md"
execution_mode = "appended"
input_keys = ["draft_reflection_xml"]
history_key = "audit_history"
output_key = "critic_snapshot_xml"
"#;

    let tmp = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(tmp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "binding_tester".to_string(),
        name: "Binding Tester".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: Vec::new(),
        cot_template: "1. Read -> 2. Validate -> 3. Return.".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), None);
    let engine = compiler
        .compile(manifest_toml)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "draft_reflection_xml": "<agenda_draft>candidate</agenda_draft>",
            "audit_history": "previous-critic-turn"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should execute successfully: {error}"));

    assert!(
        output["critic_snapshot_xml"]
            .as_str()
            .is_some_and(|prompt| prompt.contains("<system_prompt_injection>"))
    );
    assert_eq!(output["critic_snapshot_xml_execution_mode"], "appended");
    assert_eq!(output["critic_snapshot_xml_history_key"], "audit_history");
    assert_eq!(
        output["critic_snapshot_xml_template_target"],
        "critique_report.md"
    );
    assert!(
        output["audit_history"]
            .as_str()
            .is_some_and(|history| history.contains("previous-critic-turn"))
    );
}

#[tokio::test]
async fn annotation_node_resolves_semantic_uri_input_keys_with_dollar_prefix() {
    const FLOW_URI: &str = "wendao://skills/agenda-management/references/agenda_flow.toml";
    let expected_reference = embedded_resource_text_from_wendao_uri(FLOW_URI)
        .unwrap_or_else(|| panic!("embedded semantic URI should resolve: {FLOW_URI}"));
    assert!(
        expected_reference.contains("Student_Ambition"),
        "fixture URI should contain agenda validation flow nodes"
    );

    let manifest_toml = r#"
name = "Binding_Semantic_Uri_Input_Keys"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "binding_tester"
input_keys = ["$wendao://skills/agenda-management/references/agenda_flow.toml"]
"#;

    let tmp = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(tmp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "binding_tester".to_string(),
        name: "Binding Tester".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: Vec::new(),
        cot_template: "1. Read -> 2. Validate -> 3. Return.".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), None);
    let engine = compiler
        .compile(manifest_toml)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({}))
        .await
        .unwrap_or_else(|error| panic!("scheduler should execute successfully: {error}"));

    assert!(
        output["annotated_prompt"]
            .as_str()
            .is_some_and(|prompt| prompt.contains("Student_Ambition"))
    );
}

#[tokio::test]
async fn annotation_node_resolves_semantic_context_placeholders() {
    let manifest_toml = r#"
name = "Binding_Context_Placeholders"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "$persona_selector"
template_target = "$template_selector"
input_keys = ["$raw_facts_selector"]
"#;

    let tmp = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(tmp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "binding_tester".to_string(),
        name: "Binding Tester".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: Vec::new(),
        cot_template: "1. Read -> 2. Validate -> 3. Return.".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), None);
    let engine = compiler
        .compile(manifest_toml)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "persona_selector": "binding_tester",
            "template_selector": "dynamic-template.md",
            "raw_facts_selector": "context-driven-facts",
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should execute successfully: {error}"));

    assert_eq!(output["annotated_persona_id"], "binding_tester");
    assert_eq!(output["annotated_template_target"], "dynamic-template.md");
    assert!(
        output["annotated_prompt"]
            .as_str()
            .is_some_and(|prompt| prompt.contains("context-driven-facts"))
    );
}
