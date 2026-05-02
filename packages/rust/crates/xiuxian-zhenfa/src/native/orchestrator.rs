//! Native orchestration pipeline for tool dispatch and audit hooks.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tracing::warn;

use super::{ZhenfaContext, ZhenfaError, ZhenfaRegistry, ZhenfaSignal};

/// Guard returned by distributed lock implementations.
pub trait ZhenfaMutationGuard: Send + Sync {}

impl ZhenfaMutationGuard for () {}

/// Optional result-cache contract for native tool dispatch.
#[async_trait]
pub trait ZhenfaResultCache: Send + Sync {
    /// Lookup a previously cached stripped payload by key.
    ///
    /// # Errors
    /// Returns [`ZhenfaError`] when backend cache IO fails.
    async fn get(&self, key: &str) -> Result<Option<String>, ZhenfaError>;

    /// Store stripped payload by cache key.
    ///
    /// # Errors
    /// Returns [`ZhenfaError`] when backend cache IO fails.
    async fn set(&self, key: &str, value: &str) -> Result<(), ZhenfaError>;
}

/// Optional distributed lock contract for mutation tools.
#[async_trait]
pub trait ZhenfaMutationLock: Send + Sync {
    /// Acquire one mutation guard for the provided scope.
    ///
    /// # Errors
    /// Returns [`ZhenfaError`] when lock acquisition fails.
    async fn acquire(&self, scope: &str) -> Result<Box<dyn ZhenfaMutationGuard>, ZhenfaError>;
}

/// Dispatch outcome emitted into audit sinks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZhenfaDispatchOutcome {
    /// Tool call succeeded and executed natively.
    Success,
    /// Result was served from cache without invoking native execution.
    Cached,
    /// Tool dispatch failed before or during execution.
    Failed,
}

/// Audit event emitted after each dispatch attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZhenfaDispatchEvent {
    /// Invoked tool identifier.
    pub tool_id: String,
    /// Optional runtime session identifier.
    pub session_id: Option<String>,
    /// Optional trace identifier for correlation.
    pub trace_id: Option<String>,
    /// Dispatch elapsed milliseconds.
    pub elapsed_ms: u128,
    /// Success/cached/failure outcome.
    pub outcome: ZhenfaDispatchOutcome,
    /// Optional domain/system code for failure outcomes.
    pub error_code: Option<i32>,
    /// Optional human-readable failure message.
    pub error_message: Option<String>,
}

/// Optional audit sink for tool dispatch lifecycle events.
#[async_trait]
pub trait ZhenfaAuditSink: Send + Sync {
    /// Emit one dispatch event.
    ///
    /// # Errors
    /// Returns [`ZhenfaError`] when audit backend operations fail.
    async fn emit(&self, event: ZhenfaDispatchEvent) -> Result<(), ZhenfaError>;
}

/// Optional runtime signal sink for fire-and-forget signal emission.
#[async_trait]
pub trait ZhenfaSignalSink: Send + Sync {
    /// Consume one signal emitted by native tool execution.
    ///
    /// # Errors
    /// Returns [`ZhenfaError`] when downstream signal processing fails.
    async fn emit(&self, ctx: &ZhenfaContext, signal: ZhenfaSignal) -> Result<(), ZhenfaError>;
}

/// Optional runtime hooks injected into the orchestrator.
#[derive(Clone, Default)]
pub struct ZhenfaOrchestratorHooks {
    /// Optional read-through cache used for deterministic tool responses.
    pub cache: Option<Arc<dyn ZhenfaResultCache>>,
    /// Optional distributed lock used for mutation tools.
    pub mutation_lock: Option<Arc<dyn ZhenfaMutationLock>>,
    /// Optional audit sink for observability.
    pub audit_sink: Option<Arc<dyn ZhenfaAuditSink>>,
    /// Optional signal sink for asynchronous fire-and-forget runtime signals.
    pub signal_sink: Option<Arc<dyn ZhenfaSignalSink>>,
}

/// Native in-process tool orchestrator.
#[derive(Clone, Default)]
pub struct ZhenfaOrchestrator {
    registry: ZhenfaRegistry,
    hooks: ZhenfaOrchestratorHooks,
}

impl ZhenfaOrchestrator {
    /// Create an orchestrator from one tool registry.
    #[must_use]
    pub fn new(registry: ZhenfaRegistry) -> Self {
        Self::with_hooks(registry, ZhenfaOrchestratorHooks::default())
    }

    /// Create an orchestrator from one tool registry and runtime hooks.
    #[must_use]
    pub fn with_hooks(registry: ZhenfaRegistry, hooks: ZhenfaOrchestratorHooks) -> Self {
        Self { registry, hooks }
    }

    /// Expose immutable access to the tool registry.
    #[must_use]
    pub fn registry(&self) -> &ZhenfaRegistry {
        &self.registry
    }

    /// Expose immutable access to runtime hooks.
    #[must_use]
    pub fn hooks(&self) -> &ZhenfaOrchestratorHooks {
        &self.hooks
    }

    /// Dispatch one native tool call by id.
    ///
    /// # Errors
    /// Returns [`ZhenfaError::NotFound`] when the tool id is not registered.
    /// Returns [`ZhenfaError::InvalidArguments`] when `args` is not an object.
    /// Returns tool execution errors from [`ZhenfaTool::call_native`].
    pub async fn dispatch(
        &self,
        tool_id: &str,
        ctx: &ZhenfaContext,
        args: Value,
    ) -> Result<String, ZhenfaError> {
        let started_at = Instant::now();
        if !args.is_object() {
            let error = ZhenfaError::invalid_arguments("tool arguments must be a JSON object");
            self.emit_failure_event(tool_id, ctx, started_at, &error)
                .await;
            return Err(error);
        }
        let mut dispatch_ctx = ctx.clone();
        dispatch_ctx.set_correlation_id_if_absent(format!("zhenfa:{tool_id}"));
        let (signal_tx, signal_rx) = unbounded_channel::<ZhenfaSignal>();
        dispatch_ctx.attach_signal_sender(signal_tx);
        let mut signal_rx = Some(signal_rx);

        let tool = self
            .registry
            .get(tool_id)
            .ok_or_else(|| ZhenfaError::not_found(tool_id));
        let tool = match tool {
            Ok(tool) => tool,
            Err(error) => {
                self.emit_failure_event(tool_id, &dispatch_ctx, started_at, &error)
                    .await;
                if let Some(signal_rx) = signal_rx.take() {
                    self.drain_emitted_signals(&dispatch_ctx, signal_rx);
                }
                return Err(error);
            }
        };

        let cache_key = tool.cache_key(&dispatch_ctx, &args);
        if let Some(cached) = self.try_cache_lookup(cache_key.as_deref()).await {
            self.emit_success_event(
                tool_id,
                &dispatch_ctx,
                started_at,
                ZhenfaDispatchOutcome::Cached,
            )
            .await;
            if let Some(signal_rx) = signal_rx.take() {
                self.drain_emitted_signals(&dispatch_ctx, signal_rx);
            }
            return Ok(cached);
        }

        let _lock_guard = match (
            tool.mutation_scope(&dispatch_ctx, &args).as_deref(),
            self.hooks.mutation_lock.as_ref(),
        ) {
            (Some(scope), Some(locker)) => match locker.acquire(scope).await {
                Ok(guard) => Some(guard),
                Err(error) => {
                    self.emit_failure_event(tool_id, &dispatch_ctx, started_at, &error)
                        .await;
                    if let Some(signal_rx) = signal_rx.take() {
                        self.drain_emitted_signals(&dispatch_ctx, signal_rx);
                    }
                    return Err(error);
                }
            },
            _ => None,
        };

        let result = tool.call_native(&dispatch_ctx, args).await;
        match result {
            Ok(output) => {
                self.try_cache_store(cache_key.as_deref(), &output).await;
                self.emit_success_event(
                    tool_id,
                    &dispatch_ctx,
                    started_at,
                    ZhenfaDispatchOutcome::Success,
                )
                .await;
                if let Some(signal_rx) = signal_rx.take() {
                    self.drain_emitted_signals(&dispatch_ctx, signal_rx);
                }
                Ok(output)
            }
            Err(error) => {
                self.emit_failure_event(tool_id, &dispatch_ctx, started_at, &error)
                    .await;
                if let Some(signal_rx) = signal_rx.take() {
                    self.drain_emitted_signals(&dispatch_ctx, signal_rx);
                }
                Err(error)
            }
        }
    }

    async fn try_cache_lookup(&self, cache_key: Option<&str>) -> Option<String> {
        let cache_key = cache_key?;
        let cache = self.hooks.cache.as_ref()?;
        match cache.get(cache_key).await {
            Ok(cached) => cached,
            Err(error) => {
                warn!(
                    target: "xiuxian_zhenfa::native::orchestrator",
                    event = "zhenfa.cache.lookup_failed",
                    cache_key,
                    error = %error
                );
                None
            }
        }
    }

    async fn try_cache_store(&self, cache_key: Option<&str>, payload: &str) {
        let Some(cache_key) = cache_key else {
            return;
        };
        let Some(cache) = self.hooks.cache.as_ref() else {
            return;
        };
        if let Err(error) = cache.set(cache_key, payload).await {
            warn!(
                target: "xiuxian_zhenfa::native::orchestrator",
                event = "zhenfa.cache.store_failed",
                cache_key,
                error = %error
            );
        }
    }

    async fn emit_success_event(
        &self,
        tool_id: &str,
        ctx: &ZhenfaContext,
        started_at: Instant,
        outcome: ZhenfaDispatchOutcome,
    ) {
        let event = ZhenfaDispatchEvent {
            tool_id: tool_id.to_string(),
            session_id: ctx.session_id.clone(),
            trace_id: ctx.trace_id.clone(),
            elapsed_ms: started_at.elapsed().as_millis(),
            outcome,
            error_code: None,
            error_message: None,
        };
        self.emit_audit_event(event).await;
    }

    async fn emit_failure_event(
        &self,
        tool_id: &str,
        ctx: &ZhenfaContext,
        started_at: Instant,
        error: &ZhenfaError,
    ) {
        let event = ZhenfaDispatchEvent {
            tool_id: tool_id.to_string(),
            session_id: ctx.session_id.clone(),
            trace_id: ctx.trace_id.clone(),
            elapsed_ms: started_at.elapsed().as_millis(),
            outcome: ZhenfaDispatchOutcome::Failed,
            error_code: error_code(error),
            error_message: Some(error.to_string()),
        };
        self.emit_audit_event(event).await;
    }

    async fn emit_audit_event(&self, event: ZhenfaDispatchEvent) {
        let Some(audit_sink) = self.hooks.audit_sink.as_ref() else {
            return;
        };
        if let Err(error) = audit_sink.emit(event).await {
            warn!(
                target: "xiuxian_zhenfa::native::orchestrator",
                event = "zhenfa.audit.emit_failed",
                error = %error
            );
        }
    }

    fn drain_emitted_signals(
        &self,
        ctx: &ZhenfaContext,
        mut signal_rx: UnboundedReceiver<ZhenfaSignal>,
    ) {
        while let Ok(signal) = signal_rx.try_recv() {
            self.emit_signal_event(ctx, signal);
        }
    }

    fn emit_signal_event(&self, ctx: &ZhenfaContext, signal: ZhenfaSignal) {
        let Some(signal_sink) = self.hooks.signal_sink.as_ref() else {
            return;
        };
        let signal_sink = Arc::clone(signal_sink);
        let signal_ctx = ctx.clone();
        tokio::spawn(async move {
            if let Err(error) = signal_sink.emit(&signal_ctx, signal).await {
                warn!(
                    target: "xiuxian_zhenfa::native::orchestrator",
                    event = "zhenfa.signal.emit_failed",
                    error = %error,
                    "zhenfa signal sink emit failed"
                );
            }
        });
    }
}

#[must_use]
fn error_code(error: &ZhenfaError) -> Option<i32> {
    match error {
        ZhenfaError::Execution { code, .. } => *code,
        ZhenfaError::NotFound { .. } | ZhenfaError::InvalidArguments { .. } => None,
    }
}
