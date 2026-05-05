use super::{RequestCaptureLlmClient, SequencedMockLlmClient, make_registry};
use serde_json::json;
use std::sync::{Arc, Mutex};
use xiuxian_llm::llm::MessageContent;
use xiuxian_llm::llm::client::MessageRole;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism};
use xiuxian_qianji::executors::{ContextAnnotator, LlmAugmentedAuditMechanism};
use xiuxian_qianji::{NodeQianhuanExecutionMode, QianjiCompiler, QianjiLlmClient, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

#[tokio::test]
async fn llm_augmented_audit_retries_when_score_is_below_threshold() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.42</score><reason>too risky</reason>".to_string(),
    ]));

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda has 12 heavy tasks in one day.",
            "request": "Critique this agenda."
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "failed");
    let audit_score = output.data["audit_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("audit_score should be a numeric value"));
    assert!((audit_score - 0.42).abs() < 1e-6);
    let memrl_reward = output.data["memrl_reward"]
        .as_f64()
        .unwrap_or_else(|| panic!("memrl_reward should be a numeric value"));
    assert!((memrl_reward - 0.42).abs() < 1e-6);
    assert_eq!(
        output.data["memrl_signal_source"],
        json!("formal_audit.llm")
    );
    assert_eq!(
        output.data["audit_critique"],
        json!("<score>0.42</score><reason>too risky</reason>")
    );
    assert_eq!(output.data["audit_retry_count"], json!(1));
    let FlowInstruction::RetryNodes(nodes) = output.instruction else {
        panic!("expected RetryNodes instruction for score below threshold");
    };
    assert_eq!(nodes, vec!["Agenda_Steward_Proposer".to_string()]);
}

#[tokio::test]
async fn llm_augmented_audit_adds_strict_xml_score_contract() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let llm = Arc::new(RequestCaptureLlmClient {
        requests: Arc::clone(&requests),
        response: "<professor_audit><score>0.91</score><reason>clear</reason></professor_audit>"
            .to_string(),
    });

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda has a realistic workload.",
            "request": "Critique this agenda."
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "passed");

    let requests = requests
        .lock()
        .unwrap_or_else(|_| panic!("captured request queue should be accessible"));
    let request = requests
        .first()
        .unwrap_or_else(|| panic!("one llm request should be captured"));
    let system_messages = request
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::System)
        .collect::<Vec<_>>();
    assert_eq!(system_messages.len(), 2);
    let Some(MessageContent::Text(contract)) = system_messages
        .get(1)
        .and_then(|message| message.content.as_ref())
    else {
        panic!("formal audit contract system message should carry plain text");
    };
    assert!(contract.contains("<score> tag"));
    assert!(contract.contains("Do not emit Markdown"));
}

#[tokio::test]
async fn llm_augmented_audit_records_parse_error_when_score_tag_missing() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<reason>missing score</reason>".to_string(),
    ]));

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda has no breaks.",
            "request": "Critique this agenda."
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "failed");
    assert_eq!(output.data["audit_score"], json!(0.0));
    assert_eq!(output.data["memrl_reward"], json!(0.0));
    assert_eq!(
        output.data["memrl_signal_source"],
        json!("formal_audit.llm")
    );
    assert_eq!(output.data["audit_retry_count"], json!(1));
    let audit_errors = output.data["audit_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("audit_errors should be an array"));
    assert!(
        audit_errors.iter().any(|value| value
            .as_str()
            .is_some_and(|text| text.contains("missing or invalid"))),
        "expected parse-failure audit error, got: {audit_errors:?}"
    );
    let FlowInstruction::RetryNodes(nodes) = output.instruction else {
        panic!("expected RetryNodes instruction when score tag is missing");
    };
    assert_eq!(nodes, vec!["Agenda_Steward_Proposer".to_string()]);
}

#[tokio::test]
async fn llm_augmented_audit_aborts_when_retry_budget_is_exhausted() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.10</score><reason>still overloaded</reason>".to_string(),
    ]));

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 1,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda still has overload risk.",
            "request": "Critique this agenda.",
            "audit_retry_count": 1
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "failed");
    assert_eq!(output.data["audit_retry_count"], json!(2));
    assert_eq!(output.data["audit_retry_exhausted"], json!(true));
    let audit_errors = output.data["audit_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("audit_errors should be an array"));
    assert!(
        audit_errors.iter().any(|value| value
            .as_str()
            .is_some_and(|text| text.contains("retry budget exceeded"))),
        "expected retry-budget audit error, got: {audit_errors:?}"
    );
    let FlowInstruction::Abort(reason) = output.instruction else {
        panic!("expected Abort instruction when retry budget is exhausted");
    };
    assert_eq!(reason, "formal_audit.max_retries_exceeded");
}

#[tokio::test]
async fn compiler_builds_llm_augmented_formal_audit_and_converges() {
    let manifest = r#"
name = "AugmentedAuditLoop"

[[nodes]]
id = "Agenda_Steward_Proposer"
task_type = "mock"
weight = 1.0
params = {}

[[nodes]]
id = "Strict_Teacher_Critic"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Agenda_Steward_Proposer"], threshold_score = 0.8, max_retries = 3, output_key = "teacher_critique", score_key = "teacher_score" }
[nodes.qianhuan]
persona_id = "strict_teacher"
template_target = "critique_agenda.j2"
[nodes.llm]
model = "audit-model"

[[edges]]
from = "Agenda_Steward_Proposer"
to = "Strict_Teacher_Critic"
weight = 1.0
"#;

    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.30</score><reason>overloaded</reason>".to_string(),
        "<score>0.95</score><reason>acceptable</reason>".to_string(),
    ]));
    let models_probe = Arc::clone(&llm.seen_models);
    let llm_client: Arc<QianjiLlmClient> = llm;
    let compiler = QianjiCompiler::new(index, orchestrator, make_registry(), Some(llm_client));
    let engine = compiler
        .compile(manifest)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "raw_facts": "Initial agenda draft"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should converge: {error}"));

    assert_eq!(output["audit_status"], "passed");
    let teacher_score = output["teacher_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("teacher_score should be a numeric value"));
    assert!((teacher_score - 0.95).abs() < 1e-6);
    let memrl_reward = output["memrl_reward"]
        .as_f64()
        .unwrap_or_else(|| panic!("memrl_reward should be a numeric value"));
    assert!((memrl_reward - 0.95).abs() < 1e-6);
    assert_eq!(output["memrl_signal_source"], json!("formal_audit.llm"));
    assert_eq!(
        output["teacher_critique"],
        json!("<score>0.95</score><reason>acceptable</reason>")
    );

    let models = models_probe
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(
        models,
        vec!["audit-model".to_string(), "audit-model".to_string()]
    );
}
