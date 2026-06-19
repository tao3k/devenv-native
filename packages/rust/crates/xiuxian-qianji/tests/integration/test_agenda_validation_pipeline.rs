//! Agenda validation pipeline integration tests.

#![cfg(feature = "wendao-integration")]

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use xiuxian_qianji::safety::logic::Invariant;
use xiuxian_qianji::{
    FlowInstruction, QianjiApp, QianjiEngine, QianjiManifest, QianjiManifestPipelineRequest,
    QianjiMechanism, QianjiOutput, QianjiPipelineDependencies, QianjiScheduler,
    manifest_declares_annotation_bindings, manifest_requires_llm,
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

fn annotation_binding(
    node: &xiuxian_qianji::contracts::NodeDefinition,
) -> &xiuxian_qianji::contracts::NodeAnnotationBinding {
    node.annotation
        .as_ref()
        .unwrap_or_else(|| panic!("{} should declare annotation binding", node.id))
}

#[test]
fn agenda_validation_manifest_contains_required_nodes_and_bindings() {
    let manifest_toml = agenda_validation_manifest_toml();
    let manifest = parse_agenda_validation_manifest();

    let student = node_by_id(&manifest, "Student_Ambition");
    let student_binding = annotation_binding(student);
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
    let steward_binding = annotation_binding(steward);
    assert_eq!(
        steward_binding.persona_id.as_deref(),
        Some("$wendao://skills/agenda-management/references/steward.md")
    );
    assert_eq!(
        steward_binding.output_key.as_deref(),
        Some("steward_feedback")
    );

    let professor = node_by_id(&manifest, "Professor_Audit");
    let professor_binding = annotation_binding(professor);
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
    let reflection_binding = annotation_binding(reflection);
    assert_eq!(
        reflection_binding.template_target.as_deref(),
        Some("$wendao://skills/agenda-management/references/final_agenda.j2")
    );
    assert_eq!(
        reflection_binding.output_key.as_deref(),
        Some("final_synaptic_report")
    );

    assert!(
        manifest_declares_annotation_bindings(manifest_toml).unwrap_or_else(|error| panic!(
            "manifest should parse for annotation binding inspection: {error}"
        ))
    );
    assert!(
        manifest_requires_llm(manifest_toml).unwrap_or_else(|error| panic!(
            "manifest should parse for llm requirement inspection: {error}"
        ))
    );
}

#[test]
fn agenda_validation_pipeline_rejects_retired_local_llm_execution() {
    let temp_dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp_dir.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let manifest_toml = agenda_validation_manifest_toml();

    let dependencies = QianjiPipelineDependencies::new(index);
    let error = QianjiApp::create_pipeline_from_manifest(QianjiManifestPipelineRequest {
        manifest_toml,
        dependencies,
    })
    .err()
    .unwrap_or_else(|| panic!("agenda validation pipeline should fail without llm feature"));
    let message = error.to_string();
    assert!(message.contains("formal_audit"));
    assert!(
        message.contains("local Qianji LLM execution is retired")
            || message.contains("marlin-agent-core")
    );
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
