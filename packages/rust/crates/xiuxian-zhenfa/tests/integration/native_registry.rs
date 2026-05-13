//! Integration tests for native in-process zhenfa registry/orchestrator.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use xiuxian_zhenfa::{
    ZhenfaAuditSink, ZhenfaContext, ZhenfaDispatchOutcome, ZhenfaError, ZhenfaMutationGuard,
    ZhenfaMutationLock, ZhenfaOrchestrator, ZhenfaOrchestratorHooks, ZhenfaRegistry,
    ZhenfaResultCache, ZhenfaSignal, ZhenfaSignalSink, ZhenfaTool,
};

struct EchoTool;

#[async_trait]
impl ZhenfaTool for EchoTool {
    fn id(&self) -> &'static str {
        "echo.tool"
    }

    fn definition(&self) -> serde_json::Value {
        json!({
            "name": "echo.tool",
            "description": "Echo payload for test",
            "parameters": {
                "type": "object",
                "properties": { "value": { "type": "string" } },
                "required": ["value"]
            }
        })
    }

    async fn call_native(
        &self,
        ctx: &ZhenfaContext,
        args: serde_json::Value,
    ) -> Result<String, ZhenfaError> {
        let value = args
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        Ok(format!(
            "<echo session=\"{}\">{value}</echo>",
            ctx.session_id.clone().unwrap_or_default()
        ))
    }
}

struct CountingEchoTool {
    id: &'static str,
    calls: Arc<AtomicUsize>,
    mutation_scope: Option<&'static str>,
    force_failure: bool,
}

struct RewardSignalTool;

#[async_trait]
impl ZhenfaTool for RewardSignalTool {
    fn id(&self) -> &'static str {
        "reward.signal"
    }

    fn definition(&self) -> serde_json::Value {
        json!({
            "name": "reward.signal",
            "description": "Emit one reward signal for orchestrator sink tests",
            "parameters": {
                "type": "object",
                "properties": {
                    "episode_id": { "type": "string" },
                    "value": { "type": "number" }
                },
                "required": ["episode_id", "value"]
            }
        })
    }

    async fn call_native(
        &self,
        ctx: &ZhenfaContext,
        args: serde_json::Value,
    ) -> Result<String, ZhenfaError> {
        let episode_id = args
            .get("episode_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let value = args
            .get("value")
            .and_then(serde_json::Value::as_f64)
            .and_then(|value| value.to_string().parse::<f32>().ok())
            .unwrap_or(0.0);

        let signal_sender = ctx
            .get_extension::<UnboundedSender<ZhenfaSignal>>()
            .ok_or_else(|| ZhenfaError::execution("signal sender missing from context"))?;
        signal_sender
            .send(ZhenfaSignal::Reward {
                episode_id,
                value,
                source: "strict_teacher".to_string(),
            })
            .map_err(|_| ZhenfaError::execution("signal receiver closed"))?;

        Ok("<ok/>".to_string())
    }
}

#[async_trait]
impl ZhenfaTool for CountingEchoTool {
    fn id(&self) -> &str {
        self.id
    }

    fn definition(&self) -> serde_json::Value {
        json!({
            "name": self.id,
            "description": "Counting echo payload for test",
            "parameters": {
                "type": "object",
                "properties": { "value": { "type": "string" } },
                "required": ["value"]
            }
        })
    }

    async fn call_native(
        &self,
        ctx: &ZhenfaContext,
        args: serde_json::Value,
    ) -> Result<String, ZhenfaError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.force_failure {
            return Err(ZhenfaError::execution_with_code(
                500,
                "forced failure for audit test",
            ));
        }
        let value = args
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        Ok(format!(
            "<echo session=\"{}\">{value}</echo>",
            ctx.session_id.clone().unwrap_or_default()
        ))
    }

    fn cache_key(&self, _ctx: &ZhenfaContext, args: &serde_json::Value) -> Option<String> {
        let value = args.get("value")?.as_str().unwrap_or_default();
        Some(format!("{}::{value}", self.id))
    }

    fn mutation_scope(&self, _ctx: &ZhenfaContext, _args: &serde_json::Value) -> Option<String> {
        self.mutation_scope.map(str::to_string)
    }
}

#[derive(Default)]
struct InMemoryResultCache {
    values: Mutex<HashMap<String, String>>,
    gets: AtomicUsize,
    sets: AtomicUsize,
}

impl InMemoryResultCache {
    async fn insert(&self, key: &str, value: &str) {
        self.values
            .lock()
            .await
            .insert(key.to_string(), value.to_string());
    }

    async fn get_value(&self, key: &str) -> Option<String> {
        self.values.lock().await.get(key).cloned()
    }

    fn get_count(&self) -> usize {
        self.gets.load(Ordering::SeqCst)
    }

    fn set_count(&self) -> usize {
        self.sets.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ZhenfaResultCache for InMemoryResultCache {
    async fn get(&self, key: &str) -> Result<Option<String>, ZhenfaError> {
        self.gets.fetch_add(1, Ordering::SeqCst);
        Ok(self.values.lock().await.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), ZhenfaError> {
        self.sets.fetch_add(1, Ordering::SeqCst);
        self.values
            .lock()
            .await
            .insert(key.to_string(), value.to_string());
        Ok(())
    }
}

#[derive(Default)]
struct TrackingMutationLock {
    acquired_scopes: Mutex<Vec<String>>,
    acquire_count: AtomicUsize,
}

struct TestMutationGuard;

impl ZhenfaMutationGuard for TestMutationGuard {}

impl TrackingMutationLock {
    fn acquire_count(&self) -> usize {
        self.acquire_count.load(Ordering::SeqCst)
    }

    async fn scopes(&self) -> Vec<String> {
        self.acquired_scopes.lock().await.clone()
    }
}

#[async_trait]
impl ZhenfaMutationLock for TrackingMutationLock {
    async fn acquire(&self, scope: &str) -> Result<Box<dyn ZhenfaMutationGuard>, ZhenfaError> {
        self.acquire_count.fetch_add(1, Ordering::SeqCst);
        self.acquired_scopes.lock().await.push(scope.to_string());
        Ok(Box::new(TestMutationGuard))
    }
}

#[derive(Default)]
struct CollectingAuditSink {
    events: Mutex<Vec<xiuxian_zhenfa::ZhenfaDispatchEvent>>,
}

#[derive(Default)]
struct CollectingSignalSink {
    signals: Mutex<Vec<(Option<xiuxian_zhenfa::ZhenfaTraceId>, ZhenfaSignal)>>,
}

impl CollectingSignalSink {
    async fn len(&self) -> usize {
        self.signals.lock().await.len()
    }

    async fn first(&self) -> Option<(Option<xiuxian_zhenfa::ZhenfaTraceId>, ZhenfaSignal)> {
        self.signals.lock().await.first().cloned()
    }
}

#[async_trait]
impl ZhenfaSignalSink for CollectingSignalSink {
    async fn emit(&self, ctx: &ZhenfaContext, signal: ZhenfaSignal) -> Result<(), ZhenfaError> {
        self.signals
            .lock()
            .await
            .push((ctx.trace_id.clone(), signal));
        Ok(())
    }
}

impl CollectingAuditSink {
    async fn events(&self) -> Vec<xiuxian_zhenfa::ZhenfaDispatchEvent> {
        self.events.lock().await.clone()
    }
}

#[async_trait]
impl ZhenfaAuditSink for CollectingAuditSink {
    async fn emit(&self, event: xiuxian_zhenfa::ZhenfaDispatchEvent) -> Result<(), ZhenfaError> {
        self.events.lock().await.push(event);
        Ok(())
    }
}

#[tokio::test]
async fn orchestrator_dispatches_registered_tool() {
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(EchoTool));
    let orchestrator = ZhenfaOrchestrator::new(registry);

    let mut ctx = ZhenfaContext::default();
    ctx.session_id = Some(xiuxian_zhenfa::ZhenfaSessionId::from("telegram:42"));
    let result = orchestrator
        .dispatch("echo.tool", &ctx, json!({ "value": "native" }))
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
    assert_eq!(result, "<echo session=\"telegram:42\">native</echo>");
}

#[tokio::test]
async fn orchestrator_returns_not_found_for_missing_tool() {
    let orchestrator = ZhenfaOrchestrator::new(ZhenfaRegistry::new());
    let Err(error) = orchestrator
        .dispatch("unknown.tool", &ZhenfaContext::default(), json!({}))
        .await
    else {
        panic!("dispatch should fail")
    };
    assert!(matches!(error, ZhenfaError::NotFound { .. }));
}

#[tokio::test]
async fn orchestrator_rejects_non_object_arguments() {
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(EchoTool));
    let orchestrator = ZhenfaOrchestrator::new(registry);

    let Err(error) = orchestrator
        .dispatch(
            "echo.tool",
            &ZhenfaContext::default(),
            json!(["not", "object"]),
        )
        .await
    else {
        panic!("dispatch should fail")
    };
    assert!(matches!(error, ZhenfaError::InvalidArguments { .. }));
}

#[test]
fn registry_returns_tool_definitions_snapshot() {
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(EchoTool));
    let definitions = registry.definitions();
    assert!(definitions.contains_key("echo.tool"));
    assert_eq!(
        definitions["echo.tool"]["description"],
        serde_json::Value::String("Echo payload for test".to_string())
    );
}

#[tokio::test]
async fn orchestrator_returns_cached_payload_without_native_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(CountingEchoTool {
        id: "echo.cached",
        calls: Arc::clone(&calls),
        mutation_scope: None,
        force_failure: false,
    }));

    let cache = Arc::new(InMemoryResultCache::default());
    cache
        .insert("echo.cached::native", "<cached>native</cached>")
        .await;
    let audit = Arc::new(CollectingAuditSink::default());
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: Some(cache.clone()),
            mutation_lock: None,
            audit_sink: Some(audit.clone()),
            signal_sink: None,
        },
    );

    let result = orchestrator
        .dispatch(
            "echo.cached",
            &ZhenfaContext::default(),
            json!({ "value": "native" }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed from cache: {error}"));
    assert_eq!(result, "<cached>native</cached>");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(cache.get_count(), 1);
    assert_eq!(cache.set_count(), 0);
    let events = audit.events().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, ZhenfaDispatchOutcome::Cached);
}

#[tokio::test]
async fn orchestrator_stores_successful_result_into_cache() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(CountingEchoTool {
        id: "echo.cached",
        calls: Arc::clone(&calls),
        mutation_scope: None,
        force_failure: false,
    }));

    let cache = Arc::new(InMemoryResultCache::default());
    let audit = Arc::new(CollectingAuditSink::default());
    let mut ctx = ZhenfaContext::default();
    ctx.session_id = Some(xiuxian_zhenfa::ZhenfaSessionId::from("discord:100"));
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: Some(cache.clone()),
            mutation_lock: None,
            audit_sink: Some(audit.clone()),
            signal_sink: None,
        },
    );

    let result = orchestrator
        .dispatch("echo.cached", &ctx, json!({ "value": "writeback" }))
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed and store cache: {error}"));
    assert_eq!(result, "<echo session=\"discord:100\">writeback</echo>");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(cache.get_count(), 1);
    assert_eq!(cache.set_count(), 1);
    assert_eq!(
        cache.get_value("echo.cached::writeback").await,
        Some("<echo session=\"discord:100\">writeback</echo>".to_string())
    );
    let events = audit.events().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, ZhenfaDispatchOutcome::Success);
}

#[tokio::test]
async fn orchestrator_acquires_mutation_lock_when_tool_declares_scope() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(CountingEchoTool {
        id: "echo.mutation",
        calls,
        mutation_scope: Some("agenda.write"),
        force_failure: false,
    }));

    let lock = Arc::new(TrackingMutationLock::default());
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: Some(lock.clone()),
            audit_sink: None,
            signal_sink: None,
        },
    );

    let _ = orchestrator
        .dispatch(
            "echo.mutation",
            &ZhenfaContext::default(),
            json!({ "value": "locked" }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed with lock: {error}"));
    assert_eq!(lock.acquire_count(), 1);
    assert_eq!(lock.scopes().await, vec!["agenda.write".to_string()]);
}

#[tokio::test]
async fn orchestrator_emits_failure_audit_event() {
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(CountingEchoTool {
        id: "echo.fail",
        calls: Arc::new(AtomicUsize::new(0)),
        mutation_scope: None,
        force_failure: true,
    }));
    let audit = Arc::new(CollectingAuditSink::default());
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: Some(audit.clone()),
            signal_sink: None,
        },
    );

    let Err(error) = orchestrator
        .dispatch(
            "echo.fail",
            &ZhenfaContext::default(),
            json!({ "value": "x" }),
        )
        .await
    else {
        panic!("dispatch should fail")
    };
    assert!(matches!(error, ZhenfaError::Execution { .. }));
    let events = audit.events().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, ZhenfaDispatchOutcome::Failed);
    assert_eq!(events[0].error_code, Some(500));
}

#[tokio::test]
async fn orchestrator_routes_tool_emitted_signals_to_signal_sink() {
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(RewardSignalTool));
    let signal_sink = Arc::new(CollectingSignalSink::default());
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: None,
            signal_sink: Some(signal_sink.clone()),
        },
    );
    let mut ctx = ZhenfaContext::default();
    ctx.set_correlation_id_if_absent("corr:reward-test");

    let output = orchestrator
        .dispatch(
            "reward.signal",
            &ctx,
            json!({ "episode_id": "episode:42", "value": 0.82 }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
    assert_eq!(output, "<ok/>");

    for _ in 0..40 {
        if signal_sink.len().await > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let Some((trace_id, payload)) = signal_sink.first().await else {
        panic!("signal sink should receive one emitted signal");
    };
    assert_eq!(
        trace_id.as_ref().map(xiuxian_zhenfa::ZhenfaTraceId::as_str),
        Some("corr:reward-test")
    );
    let ZhenfaSignal::Reward {
        episode_id,
        value,
        source,
    } = payload
    else {
        panic!("expected reward signal payload");
    };
    assert_eq!(episode_id, "episode:42");
    assert!((value - 0.82).abs() < 1e-4);
    assert_eq!(source, "strict_teacher");
}
