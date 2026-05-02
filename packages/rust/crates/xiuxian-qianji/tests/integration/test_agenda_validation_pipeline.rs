//! Agenda validation pipeline integration tests.

use async_trait::async_trait;
#[cfg(feature = "llm")]
use futures::stream;
use serde_json::json;
#[cfg(feature = "llm")]
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "llm")]
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(feature = "llm")]
use xiuxian_llm::llm::client::ChatStream;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmResult};
#[cfg(feature = "llm")]
use xiuxian_qianhuan::persona::PersonaProfile;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::safety::logic::Invariant;
use xiuxian_qianji::{
    FlowInstruction, QianjiApp, QianjiEngine, QianjiManifest, QianjiMechanism, QianjiOutput,
    QianjiScheduler, manifest_declares_qianhuan_bindings, manifest_requires_llm,
};
use xiuxian_wendao::link_graph::LinkGraphIndex;
use xiuxian_wendao_runtime::artifacts::zhixing::embedded_resource_text_from_wendao_uri;

const AGENDA_VALIDATION_WORKFLOW_URI: &str =
    "wendao://skills/agenda-management/references/agenda_flow.toml";

fn agenda_validation_manifest_toml() -> &'static str {
    embedded_resource_text_from_wendao_uri(AGENDA_VALIDATION_WORKFLOW_URI).unwrap_or_else(|| {
        panic!("expected embedded agenda validation workflow at {AGENDA_VALIDATION_WORKFLOW_URI}")
    })
}

fn parse_agenda_validation_manifest() -> QianjiManifest {
    let manifest_toml = agenda_validation_manifest_toml();
    toml::from_str(manifest_toml)
        .unwrap_or_else(|error| panic!("agenda validation manifest should parse: {error}"))
}

fn node_by_id<'a>(
    manifest: &'a QianjiManifest,
    node_id: &str,
) -> &'a xiuxian_qianji::contracts::NodeDefinition {
    manifest
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .unwrap_or_else(|| panic!("{node_id} node should exist"))
}

fn qianhuan_binding(
    node: &xiuxian_qianji::contracts::NodeDefinition,
) -> &xiuxian_qianji::contracts::NodeQianhuanBinding {
    node.qianhuan
        .as_ref()
        .unwrap_or_else(|| panic!("{} should declare qianhuan binding", node.id))
}

#[test]
fn agenda_validation_manifest_contains_required_nodes_and_bindings() {
    let manifest_toml = agenda_validation_manifest_toml();
    let manifest = parse_agenda_validation_manifest();

    let student = node_by_id(&manifest, "Student_Ambition");
    let student_binding = qianhuan_binding(student);
    assert_eq!(
        student_binding.persona_id.as_deref(),
        Some("$wendao://skills/agenda-management/references/student.md")
    );
    assert_eq!(student_binding.template_target.as_deref(), None);
    assert_eq!(
        student_binding.output_key.as_deref(),
        Some("student_proposal")
    );

    let steward = node_by_id(&manifest, "Steward_Logistics");
    let steward_binding = qianhuan_binding(steward);
    assert_eq!(
        steward_binding.persona_id.as_deref(),
        Some("$wendao://skills/agenda-management/references/steward.md")
    );
    assert_eq!(
        steward_binding.output_key.as_deref(),
        Some("steward_feedback")
    );

    let professor = node_by_id(&manifest, "Professor_Audit");
    let professor_binding = qianhuan_binding(professor);
    assert_eq!(
        professor_binding.persona_id.as_deref(),
        Some(
            "$wendao://skills/agenda-management/references/teacher.md#[Heading:Architecture, Paragraph:3]"
        )
    );
    assert_eq!(professor_binding.template_target.as_deref(), None);
    assert_eq!(
        professor_binding.output_key.as_deref(),
        Some("professor_annotated_prompt")
    );
    assert_eq!(
        professor
            .params
            .get("max_retries")
            .and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert!(
        professor.llm.is_some(),
        "Professor_Audit should declare nodes.llm for LLM-augmented formal audit"
    );
    assert_eq!(
        professor
            .params
            .get("output_key")
            .and_then(serde_json::Value::as_str),
        Some("professor_conclusion")
    );
    assert_eq!(
        professor
            .params
            .get("score_key")
            .and_then(serde_json::Value::as_str),
        Some("governance_score")
    );
    assert_eq!(
        professor
            .params
            .get("retry_targets")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.first())
            .and_then(serde_json::Value::as_str),
        Some("Professor_Audit")
    );

    let reflection = node_by_id(&manifest, "Final_Reflection");
    let reflection_binding = qianhuan_binding(reflection);
    assert_eq!(
        reflection_binding.template_target.as_deref(),
        Some("$wendao://skills/agenda-management/references/final_agenda.j2")
    );
    assert_eq!(
        reflection_binding.output_key.as_deref(),
        Some("final_synaptic_report")
    );

    assert!(
        manifest_declares_qianhuan_bindings(manifest_toml).unwrap_or_else(|error| panic!(
            "manifest should parse for qianhuan binding inspection: {error}"
        ))
    );
    assert!(
        manifest_requires_llm(manifest_toml).unwrap_or_else(|error| panic!(
            "manifest should parse for llm requirement inspection: {error}"
        ))
    );
}

#[cfg(feature = "llm")]
struct StaticScoreLlmClient {
    response: String,
    seen_models: Arc<Mutex<Vec<String>>>,
}

#[cfg(feature = "llm")]
#[async_trait]
impl LlmClient for StaticScoreLlmClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        Ok(self.response.clone())
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        Ok(Box::pin(stream::iter(vec![Ok(self.response.clone())])))
    }
}

#[cfg(feature = "llm")]
#[tokio::test]
async fn agenda_validation_pipeline_compiles_and_runs_happy_path() {
    let temp_dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp_dir.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));
    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "agenda_steward".to_string(),
        name: "Agenda Steward".to_string(),
        background: None,
        voice_tone: "Direct and structured.".to_string(),
        guidelines: Vec::new(),
        style_anchors: vec!["timeboxing".to_string()],
        cot_template: "1. Parse intent -> 2. Produce agenda.".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
    });
    registry.register(PersonaProfile {
        id: "strict_teacher".to_string(),
        name: "Strict Teacher".to_string(),
        background: None,
        voice_tone: "Critical and precise.".to_string(),
        guidelines: vec!["Evaluate agenda rigor.".to_string()],
        style_anchors: Vec::new(),
        cot_template: "1. Critique -> 2. Score -> 3. Decide".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
    });
    let registry = Arc::new(registry);
    let seen_models = Arc::new(Mutex::new(Vec::new()));
    let llm_client: Arc<xiuxian_qianji::QianjiLlmClient> = Arc::new(StaticScoreLlmClient {
        response: "<score>0.95</score><reason>acceptable</reason>".to_string(),
        seen_models: Arc::clone(&seen_models),
    });
    let manifest_toml = agenda_validation_manifest_toml();

    let scheduler = QianjiApp::create_pipeline_from_manifest(
        manifest_toml,
        index,
        orchestrator,
        registry,
        Some(llm_client),
    )
    .unwrap_or_else(|error| {
        panic!("agenda validation pipeline should compile successfully: {error}")
    });
    let output = scheduler
        .run(json!({
            "raw_facts": "timeboxing; milimeter-level alignment; architectural consistency",
            "request": "Critique today's agenda",
        }))
        .await
        .unwrap_or_else(|error| panic!("agenda validation pipeline should execute: {error}"));

    assert_eq!(output["audit_status"], "passed");
    assert!(
        output["student_proposal"].as_str().is_some(),
        "student proposal should be present"
    );
    assert!(
        output["steward_feedback"].as_str().is_some(),
        "steward feedback should be present"
    );
    assert!(
        output["professor_annotated_prompt"].as_str().is_some(),
        "professor audit prompt should be present"
    );
    assert!(
        output["professor_conclusion"].as_str().is_some(),
        "professor conclusion should be present"
    );
    assert!(
        output["final_synaptic_report"].as_str().is_some(),
        "final reflection should be materialized when audit passes"
    );
    let governance_score = output["governance_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("governance_score should be numeric"));
    assert!((governance_score - 0.95).abs() < 1e-6);
    let models = seen_models
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(models, vec![String::new()]);
}

#[cfg(feature = "llm")]
#[tokio::test]
async fn agenda_validation_pipeline_professor_prompt_stays_flattened() {
    const DEDUP_MARKER: &str = "agenda-dedup-marker-8cf1";

    let temp_dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp_dir.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));
    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "agenda_steward".to_string(),
        name: "Agenda Steward".to_string(),
        background: None,
        voice_tone: "Direct and structured.".to_string(),
        guidelines: Vec::new(),
        style_anchors: vec!["timeboxing".to_string()],
        cot_template: "1. Parse intent -> 2. Produce agenda.".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
    });
    registry.register(PersonaProfile {
        id: "strict_teacher".to_string(),
        name: "Strict Teacher".to_string(),
        background: None,
        voice_tone: "Critical and precise.".to_string(),
        guidelines: vec!["Evaluate agenda rigor.".to_string()],
        style_anchors: Vec::new(),
        cot_template: "1. Critique -> 2. Score -> 3. Decide".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
    });
    let registry = Arc::new(registry);
    let llm_client: Arc<xiuxian_qianji::QianjiLlmClient> = Arc::new(StaticScoreLlmClient {
        response: "<score>0.95</score><reason>acceptable</reason>".to_string(),
        seen_models: Arc::new(Mutex::new(Vec::new())),
    });
    let manifest_toml = agenda_validation_manifest_toml();

    let scheduler = QianjiApp::create_pipeline_from_manifest(
        manifest_toml,
        index,
        orchestrator,
        registry,
        Some(llm_client),
    )
    .unwrap_or_else(|error| {
        panic!("agenda validation pipeline should compile successfully: {error}")
    });
    let output = scheduler
        .run(json!({
            "raw_facts": format!(
                "timeboxing; execution order; deadline awareness; review loop; tool output fidelity; single message clarity; language alignment; cognitive load; historical carryover; execution realism; risk-first review; carryover=1; milimeter-level alignment; audit trail; traceability; architectural consistency; {DEDUP_MARKER}"
            ),
            "request": "Generate today agenda and critique it with explicit risk-first review.",
        }))
        .await
        .unwrap_or_else(|error| panic!("agenda validation pipeline should execute: {error}"));

    let professor_prompt = output["professor_annotated_prompt"]
        .as_str()
        .unwrap_or_else(|| panic!("professor_annotated_prompt should be a string"));
    assert!(
        !professor_prompt.contains("&lt;system_prompt_injection&gt;"),
        "professor prompt should not contain escaped nested snapshots"
    );
    assert!(
        professor_prompt.len() <= 4096,
        "professor prompt should stay within a bounded transport budget, got {} bytes",
        professor_prompt.len()
    );
    assert_eq!(
        professor_prompt.matches(DEDUP_MARKER).count(),
        1,
        "professor prompt should not carry duplicate flattened narrative segments: {professor_prompt}"
    );
}

#[cfg(not(feature = "llm"))]
#[test]
fn agenda_validation_pipeline_requires_llm_feature() {
    let temp_dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp_dir.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    let manifest_toml = agenda_validation_manifest_toml();

    let error = QianjiApp::create_pipeline_from_manifest(
        manifest_toml,
        index,
        orchestrator,
        registry,
        None,
    )
    .err()
    .unwrap_or_else(|| panic!("agenda validation pipeline should fail without llm feature"));
    let message = error.to_string();
    assert!(message.contains("formal_audit"));
    assert!(message.contains("feature `llm`"));
}

struct AgendaStewardLoopProposer {
    attempts: Arc<AtomicU32>,
}

#[async_trait]
impl QianjiMechanism for AgendaStewardLoopProposer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let current_attempt = self.attempts.fetch_add(1, Ordering::SeqCst) + 1;
        let retry_mode = context.get("audit_status").and_then(|v| v.as_str()) == Some("failed");
        let has_grounding = retry_mode;

        let predicate = if retry_mode {
            "RevisedAgenda"
        } else {
            "OverloadedAgenda"
        };

        Ok(QianjiOutput {
            data: json!({
                "analysis_trace": [
                    {
                        "predicate": predicate,
                        "has_grounding": has_grounding,
                        "confidence": 0.95
                    }
                ],
                "agenda_proposal_attempt": current_attempt,
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

struct AgendaCommitRecorder;

#[async_trait]
impl QianjiMechanism for AgendaCommitRecorder {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        Ok(QianjiOutput {
            data: json!({
                "agenda_commit_status": "validated"
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

#[tokio::test]
async fn agenda_validation_loop_converges_after_teacher_retry() {
    let attempts = Arc::new(AtomicU32::new(0));
    let proposer = Arc::new(AgendaStewardLoopProposer {
        attempts: attempts.clone(),
    });
    let critic = Arc::new(xiuxian_qianji::executors::FormalAuditMechanism {
        invariants: vec![Invariant::MustBeGrounded],
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
    });
    let commit = Arc::new(AgendaCommitRecorder);

    let mut engine = QianjiEngine::new();
    let proposer_idx = engine.add_mechanism("Agenda_Steward_Proposer", proposer);
    let critic_idx = engine.add_mechanism("Strict_Teacher_Critic", critic);
    let commit_idx = engine.add_mechanism("Agenda_Commit", commit);
    engine.add_link(proposer_idx, critic_idx, None, 1.0);
    engine.add_link(critic_idx, commit_idx, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let output = scheduler
        .run(json!({}))
        .await
        .unwrap_or_else(|error| panic!("loop scenario should converge successfully: {error}"));

    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(output["audit_status"], "passed");
    assert_eq!(output["agenda_commit_status"], "validated");
    assert_eq!(output["agenda_proposal_attempt"], 2);
}
