#![doc = "Integration tests for context isolation, appended windows, and concurrent gathering."]

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use xiuxian_qianji::executors::ContextAnnotator;
use xiuxian_qianji::{
    FlowInstruction, NodeAnnotationExecutionMode, QianjiEngine, QianjiMechanism, QianjiOutput,
    QianjiScheduler,
};

struct ProbedAnnotator {
    inner: Arc<ContextAnnotator>,
    probe_inflight: Arc<AtomicUsize>,
    probe_max: Arc<AtomicUsize>,
    delay: Duration,
}

impl ProbedAnnotator {
    fn enter(&self) {
        let inflight = self.probe_inflight.fetch_add(1, Ordering::SeqCst) + 1;
        loop {
            let observed = self.probe_max.load(Ordering::SeqCst);
            if inflight <= observed {
                break;
            }
            if self
                .probe_max
                .compare_exchange(observed, inflight, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
    }

    fn leave(&self) {
        self.probe_inflight.fetch_sub(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl QianjiMechanism for ProbedAnnotator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        self.enter();
        sleep(self.delay).await;
        let result = self.inner.execute(context).await;
        self.leave();
        result
    }

    fn weight(&self) -> f32 {
        self.inner.weight()
    }
}

struct AggregationValidator {
    required_keys: Vec<String>,
}

#[async_trait]
impl QianjiMechanism for AggregationValidator {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        for key in &self.required_keys {
            let Some(value) = context.get(key).and_then(serde_json::Value::as_str) else {
                return Err(format!("missing aggregated critic output key: {key}"));
            };
            if value.is_empty() {
                return Err(format!("aggregated critic output is empty: {key}"));
            }
        }
        Ok(QianjiOutput {
            data: json!({
                "gather_status": "ok",
                "gathered_keys": self.required_keys,
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn annotator(
    output_key: &str,
    mode: NodeAnnotationExecutionMode,
    input_keys: Vec<String>,
    history_key: &str,
) -> ContextAnnotator {
    ContextAnnotator {
        persona_id: "test_critic".to_string(),
        template_target: Some("audit_template.j2".to_string()),
        execution_mode: mode,
        input_keys,
        history_key: history_key.to_string(),
        output_key: output_key.to_string(),
    }
}

#[tokio::test]
async fn isolated_mode_quarantines_history_and_non_whitelisted_fields() {
    let mut engine = QianjiEngine::new();
    let node = Arc::new(annotator(
        "isolated_snapshot_xml",
        NodeAnnotationExecutionMode::Isolated,
        vec!["allowed_payload".to_string()],
        "shared_history",
    ));
    engine.add_mechanism("IsolatedAnnotator", node);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler
        .run(json!({
            "allowed_payload": "allowed-structured-xml",
            "forbidden_payload": "must-not-leak",
            "shared_history": "must-not-enter-isolated-history"
        }))
        .await
        .unwrap_or_else(|error| panic!("isolated mode run should succeed: {error}"));

    let snapshot = result["isolated_snapshot_xml"]
        .as_str()
        .unwrap_or_else(|| panic!("isolated_snapshot_xml should be a string"));
    assert!(snapshot.contains("allowed-structured-xml"));
    assert!(!snapshot.contains("must-not-leak"));
    assert!(!snapshot.contains("must-not-enter-isolated-history"));
}

#[tokio::test]
async fn appended_mode_persists_and_reuses_history_between_nodes() {
    let mut engine = QianjiEngine::new();
    let draft_idx = engine.add_mechanism(
        "DraftAnnotator",
        Arc::new(annotator(
            "draft_snapshot_xml",
            NodeAnnotationExecutionMode::Appended,
            vec!["raw_facts".to_string()],
            "shared_history",
        )),
    );
    let review_idx = engine.add_mechanism(
        "ReviewAnnotator",
        Arc::new(annotator(
            "review_snapshot_xml",
            NodeAnnotationExecutionMode::Appended,
            vec!["draft_snapshot_xml".to_string()],
            "shared_history",
        )),
    );
    engine.add_link(draft_idx, review_idx, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler
        .run(json!({
            "raw_facts": "agenda-draft-fact",
            "shared_history": "seed-history"
        }))
        .await
        .unwrap_or_else(|error| panic!("appended mode run should succeed: {error}"));

    let review_snapshot = result["review_snapshot_xml"]
        .as_str()
        .unwrap_or_else(|| panic!("review_snapshot_xml should be a string"));
    assert!(review_snapshot.contains("seed-history"));

    let merged_history = result["shared_history"]
        .as_str()
        .unwrap_or_else(|| panic!("shared_history should be a string"));
    assert!(merged_history.contains("seed-history"));
    assert!(merged_history.contains("<system_prompt_injection>"));
}

#[tokio::test]
async fn concurrent_critics_run_in_parallel_and_join_at_aggregator() {
    let inflight = Arc::new(AtomicUsize::new(0));
    let max_parallel = Arc::new(AtomicUsize::new(0));

    let security = ProbedAnnotator {
        inner: Arc::new(annotator(
            "security_critic_xml",
            NodeAnnotationExecutionMode::Isolated,
            vec!["raw_facts".to_string()],
            "unused_history",
        )),
        probe_inflight: inflight.clone(),
        probe_max: max_parallel.clone(),
        delay: Duration::from_millis(120),
    };
    let performance = ProbedAnnotator {
        inner: Arc::new(annotator(
            "performance_critic_xml",
            NodeAnnotationExecutionMode::Isolated,
            vec!["raw_facts".to_string()],
            "unused_history",
        )),
        probe_inflight: inflight,
        probe_max: max_parallel.clone(),
        delay: Duration::from_millis(120),
    };

    let mut engine = QianjiEngine::new();
    let security_idx = engine.add_mechanism("SecurityCritic", Arc::new(security));
    let performance_idx = engine.add_mechanism("PerformanceCritic", Arc::new(performance));
    let gather_idx = engine.add_mechanism(
        "Gather",
        Arc::new(AggregationValidator {
            required_keys: vec![
                "security_critic_xml".to_string(),
                "performance_critic_xml".to_string(),
            ],
        }),
    );
    engine.add_link(security_idx, gather_idx, None, 1.0);
    engine.add_link(performance_idx, gather_idx, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler
        .run(json!({ "raw_facts": "critic-input" }))
        .await
        .unwrap_or_else(|error| panic!("parallel critics run should succeed: {error}"));

    assert_eq!(result["gather_status"], "ok");
    assert!(
        result["security_critic_xml"]
            .as_str()
            .is_some_and(|value| value.contains("<system_prompt_injection>"))
    );
    assert!(
        result["performance_critic_xml"]
            .as_str()
            .is_some_and(|value| value.contains("<system_prompt_injection>"))
    );
    assert!(
        max_parallel.load(Ordering::SeqCst) >= 2,
        "expected at least two critic nodes running concurrently"
    );
}
