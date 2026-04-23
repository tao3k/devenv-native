//! Integration tests for `xiuxian_qianji::executors::annotation`.

use std::sync::Arc;

use serde_json::json;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::{PersonaProfile, PersonaRegistry};
use xiuxian_qianji::contracts::{NodeQianhuanExecutionMode, QianjiMechanism};
use xiuxian_qianji::executors::ContextAnnotator;

#[tokio::test]
async fn context_annotator_can_load_persona_via_wendao_uri() {
    let annotator = ContextAnnotator {
        orchestrator: Arc::new(ThousandFacesOrchestrator::new(
            "keep plans executable".to_string(),
            None,
        )),
        registry: Arc::new(PersonaRegistry::new()),
        persona_id: "$wendao://skills/agenda-management/references/steward.md".to_string(),
        template_target: Some(
            "$wendao://skills/agenda-management/references/draft_agenda.j2".to_string(),
        ),
        execution_mode: NodeQianhuanExecutionMode::Isolated,
        input_keys: vec!["raw_facts".to_string()],
        history_key: "qianhuan_history".to_string(),
        output_key: "annotated_prompt".to_string(),
    };

    let output = annotator
        .execute(&json!({
            "raw_facts": "agenda planning execution Draft a realistic schedule Translate user intent into tasks Audit agenda quality"
        }))
        .await
        .unwrap_or_else(|error| panic!("annotation execution should succeed: {error}"));
    let Some(persona_id) = output
        .data
        .get("annotated_persona_id")
        .and_then(serde_json::Value::as_str)
    else {
        panic!("expected annotated_persona_id in annotation output");
    };
    assert!(
        persona_id == "pragmatic_agenda_steward"
            || persona_id == "professional_identity_the_clockwork_guardian",
        "unexpected persona id: {persona_id}"
    );
    let Some(template_target) = output
        .data
        .get("annotated_template_target")
        .and_then(serde_json::Value::as_str)
    else {
        panic!("expected annotated_template_target in annotation output");
    };
    assert_eq!(
        template_target,
        "wendao://skills/agenda-management/references/draft_agenda.j2"
    );
}

#[tokio::test]
async fn context_annotator_compacts_nested_prompt_snapshots_in_input_blocks() {
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
        metadata: std::collections::HashMap::new(),
    });

    let annotator = ContextAnnotator {
        orchestrator: Arc::new(ThousandFacesOrchestrator::new(
            "keep plans executable".to_string(),
            None,
        )),
        registry: Arc::new(registry),
        persona_id: "binding_tester".to_string(),
        template_target: None,
        execution_mode: NodeQianhuanExecutionMode::Isolated,
        input_keys: vec!["draft_reflection_xml".to_string()],
        history_key: "qianhuan_history".to_string(),
        output_key: "annotated_prompt".to_string(),
    };

    let nested_snapshot = r"
<system_prompt_injection>
  <genesis_rules>Safety rules</genesis_rules>
  <persona_steering>
    <tone>Strict</tone>
  </persona_steering>
  <narrative_context>
    <entry>alpha</entry>
    <entry>&lt;system_prompt_injection&gt;&lt;narrative_context&gt;&lt;entry&gt;beta&lt;/entry&gt;&lt;/narrative_context&gt;&lt;working_history&gt;gamma&lt;/working_history&gt;&lt;/system_prompt_injection&gt;</entry>
  </narrative_context>
  <working_history>delta</working_history>
</system_prompt_injection>
";

    let output = annotator
        .execute(&json!({
            "draft_reflection_xml": nested_snapshot
        }))
        .await
        .unwrap_or_else(|error| panic!("annotation execution should succeed: {error}"));

    let Some(prompt) = output.data["annotated_prompt"].as_str() else {
        panic!("annotated_prompt should be present");
    };
    assert!(prompt.contains("alpha"));
    assert!(prompt.contains("beta"));
    assert!(prompt.contains("gamma"));
    assert!(prompt.contains("delta"));
    assert!(
        !prompt.contains("&lt;system_prompt_injection&gt;"),
        "nested prompt snapshots should be flattened before re-injection: {prompt}"
    );
}

#[tokio::test]
async fn context_annotator_deduplicates_normalized_blocks_across_input_keys() {
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
        metadata: std::collections::HashMap::new(),
    });

    let annotator = ContextAnnotator {
        orchestrator: Arc::new(ThousandFacesOrchestrator::new(
            "keep plans executable".to_string(),
            None,
        )),
        registry: Arc::new(registry),
        persona_id: "binding_tester".to_string(),
        template_target: None,
        execution_mode: NodeQianhuanExecutionMode::Isolated,
        input_keys: vec![
            "first_snapshot".to_string(),
            "second_snapshot".to_string(),
            "plain_duplicate".to_string(),
        ],
        history_key: "qianhuan_history".to_string(),
        output_key: "annotated_prompt".to_string(),
    };

    let duplicate_snapshot = r"
<system_prompt_injection>
  <narrative_context>
    <entry>dedup-marker-narrative-71e3</entry>
  </narrative_context>
  <working_history>dedup-marker-history-acde</working_history>
</system_prompt_injection>
";

    let output = annotator
        .execute(&json!({
            "first_snapshot": duplicate_snapshot,
            "second_snapshot": duplicate_snapshot,
            "plain_duplicate": "dedup-marker-narrative-71e3\n\ndedup-marker-history-acde"
        }))
        .await
        .unwrap_or_else(|error| panic!("annotation execution should succeed: {error}"));

    let Some(prompt) = output.data["annotated_prompt"].as_str() else {
        panic!("annotated_prompt should be present");
    };
    assert_eq!(
        prompt.matches("dedup-marker-narrative-71e3").count(),
        1,
        "duplicate normalized narrative blocks should collapse into one entry: {prompt}"
    );
    assert_eq!(
        prompt.matches("dedup-marker-history-acde").count(),
        1,
        "duplicate normalized working history blocks should collapse into one entry: {prompt}"
    );
}

xiuxian_testing::crate_test_policy_harness!();
