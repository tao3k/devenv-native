//! Integration target for saliency-aware Quantum Fusion retrieval budgets.

use serial_test::serial;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::link_graph::{
    LinkGraphRetrievalBudget, LinkGraphSearchOptions, LinkGraphSemanticSearchPolicy,
    QuantumFusionOptions, QuantumSemanticIgnition, QuantumSemanticIgnitionFuture,
    QuantumSemanticSearchRequest, set_link_graph_wendao_config_override, valkey_saliency_get,
    valkey_saliency_get_with_valkey, valkey_saliency_touch_with_valkey,
};
use xiuxian_wendao::{LinkGraphIndex, LinkGraphSaliencyTouchRequest};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";

#[test]
#[serial(link_graph_runtime_config)]
fn test_quantum_fusion_retrieval_budget_expands_for_hot_graph_hit()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let outcome = (|| -> Result<(), Box<dyn std::error::Error>> {
        let (_temp, index) = build_budget_test_index(&prefix)?;
        let base_budget = LinkGraphRetrievalBudget {
            candidate_limit: 1,
            max_sources: 1,
            rows_per_source: 1,
        };

        touch_hot_alpha(&prefix)?;
        assert_runtime_saliency_visible(&prefix)?;

        let weighted_payload =
            index.search_planned_payload("id:alpha", 1, LinkGraphSearchOptions::default());
        let weighted_plan = weighted_payload
            .retrieval_plan
            .as_ref()
            .ok_or_else(|| std::io::Error::other("missing weighted retrieval plan"))?;
        assert_eq!(
            weighted_payload.hits.first().map(|hit| hit.stem.as_str()),
            Some("alpha")
        );
        assert!(weighted_plan.budget.candidate_limit > base_budget.candidate_limit);
        assert!(weighted_plan.budget.max_sources > base_budget.max_sources);
        assert!(weighted_plan.budget.rows_per_source > base_budget.rows_per_source);
        assert_eq!(weighted_plan.budget.candidate_limit, 2);
        assert_eq!(weighted_plan.budget.max_sources, 2);
        assert_eq!(weighted_plan.budget.rows_per_source, 2);

        let semantic_policy = LinkGraphSemanticSearchPolicy {
            min_vector_score: Some(0.72),
            ..Default::default()
        };
        let built_request = QuantumSemanticSearchRequest::from_retrieval_budget(
            Some("  alpha  "),
            &[0.25, 0.75],
            Some(&weighted_plan.budget),
            Some(semantic_policy),
        );
        assert_eq!(built_request.query_text, Some("alpha"));
        assert_eq!(built_request.candidate_limit, 2);
        assert_eq!(built_request.min_vector_score, Some(0.72));

        let ignition = RecordingIgnition::default();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let contexts = runtime.block_on(index.quantum_contexts_from_retrieval_plan(
            &ignition,
            Some("  alpha  "),
            &[0.25, 0.75],
            Some(weighted_plan),
            Some(semantic_policy),
            &QuantumFusionOptions::default(),
        ))?;
        assert!(contexts.is_empty());

        let recorded = ignition
            .last_request()
            .ok_or_else(|| std::io::Error::other("missing recorded semantic request"))?;
        assert_eq!(recorded.query_text.as_deref(), Some("alpha"));
        assert_eq!(recorded.candidate_limit, 2);
        assert_eq!(recorded.min_vector_score, Some(0.72));
        assert_eq!(recorded.query_vector_len, 2);

        Ok(())
    })();

    let _ = clear_prefix(&prefix);
    outcome
}

fn build_budget_test_index(
    prefix: &str,
) -> Result<(tempfile::TempDir, LinkGraphIndex), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().to_path_buf();

    for (name, content) in [
        ("alpha", "# Alpha\n\nalpha signal remains dominant.\n"),
        (
            "beta",
            "# Beta\n\nbeta is only here to keep the graph non-empty.\n",
        ),
    ] {
        fs::write(root.join(format!("{name}.md")), content)?;
    }

    let config_path = root.join("wendao-test.toml");
    fs::write(
        &config_path,
        format!(
            "[link_graph.cache]\nvalkey_url = \"{TEST_VALKEY_URL}\"\nkey_prefix = \"{prefix}\"\n\n[link_graph.retrieval]\ncandidate_multiplier = 1\nmax_sources = 1\ngraph_rows_per_source = 1\n"
        ),
    )?;
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    Ok((temp, LinkGraphIndex::build(&root)?))
}

fn touch_hot_alpha(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    valkey_saliency_touch_with_valkey(
        LinkGraphSaliencyTouchRequest {
            node_id: "alpha".to_string(),
            activation_delta: 64,
            saliency_base: Some(10.0),
            alpha: Some(1.0),
            minimum_saliency: Some(1.0),
            maximum_saliency: Some(10.0),
            now_unix: Some(1_700_000_000),
            ..Default::default()
        },
        TEST_VALKEY_URL,
        Some(prefix),
    )
    .map(|_| ())
    .map_err(|err| std::io::Error::other(err).into())
}

fn assert_runtime_saliency_visible(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let runtime_state = valkey_saliency_get("alpha").map_err(std::io::Error::other)?;
    let direct_state = valkey_saliency_get_with_valkey("alpha", TEST_VALKEY_URL, Some(prefix))
        .map_err(std::io::Error::other)?;
    let runtime_current = runtime_state
        .as_ref()
        .map(|state| state.current_saliency)
        .ok_or_else(|| std::io::Error::other("missing runtime saliency state"))?;
    let direct_current = direct_state
        .as_ref()
        .map(|state| state.current_saliency)
        .ok_or_else(|| std::io::Error::other("missing direct saliency state"))?;
    assert!(runtime_current >= 9.0);
    assert!(direct_current >= 9.0);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct RecordedRequest {
    query_text: Option<String>,
    candidate_limit: usize,
    min_vector_score: Option<f64>,
    query_vector_len: usize,
}

#[derive(Clone, Default)]
struct RecordingIgnition {
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl RecordingIgnition {
    fn last_request(&self) -> Option<RecordedRequest> {
        self.requests
            .lock()
            .ok()
            .and_then(|guard| guard.last().cloned())
    }
}

impl QuantumSemanticIgnition for RecordingIgnition {
    type Error = std::io::Error;

    fn backend_name(&self) -> &'static str {
        "recording-ignition"
    }

    fn search_anchors<'a>(
        &'a self,
        request: QuantumSemanticSearchRequest<'a>,
    ) -> QuantumSemanticIgnitionFuture<'a, Self::Error> {
        let requests = Arc::clone(&self.requests);
        Box::pin(async move {
            let mut guard = requests
                .lock()
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            guard.push(RecordedRequest {
                query_text: request.query_text.map(str::to_string),
                candidate_limit: request.candidate_limit,
                min_vector_score: request.min_vector_score,
                query_vector_len: request.query_vector.len(),
            });
            Ok(Vec::new())
        })
    }
}

fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_nanos());
    format!("omni:test:quantum-fusion-budget:{nanos}")
}

fn clear_prefix(prefix: &str) -> Result<(), String> {
    let client = redis::Client::open(TEST_VALKEY_URL).map_err(|err| err.to_string())?;
    let mut conn = client.get_connection().map_err(|err| err.to_string())?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query(&mut conn)
        .map_err(|err| err.to_string())?;
    if !keys.is_empty() {
        redis::cmd("DEL")
            .arg(keys)
            .query::<()>(&mut conn)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}
