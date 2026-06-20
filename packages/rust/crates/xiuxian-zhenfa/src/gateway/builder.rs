//! Gateway builder for composing routes, methods, and notification workers.

use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;

use axum::{Extension, Router, routing::get, routing::post};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::contracts::{JsonRpcErrorObject, JsonRpcMeta};
use crate::native::ZhenfaSignal;
use crate::router::{MethodRegistry, ZhenfaRouter};

use super::handlers::{health_handler, rpc_handler};
use super::notification::{NotificationService, WebhookConfig, notification_worker};

/// Build errors for the Zhenfa gateway.
#[derive(Debug, Error)]
pub enum ZhenfaGatewayBuildError {
    /// Invalid router prefix format.
    #[error("invalid router prefix `{0}` (must start with `/` and cannot be `/`)")]
    InvalidPrefix(String),
    /// Duplicate router prefix.
    #[error("duplicate router prefix `{0}`")]
    DuplicatePrefix(String),
}

/// Builder for the unified Zhenfa gateway.
#[derive(Default)]
pub struct ZhenfaGatewayBuilder {
    routers: Vec<Box<dyn ZhenfaRouter>>,
    methods: MethodRegistry,
    /// Signal receiver for notification service.
    signal_rx: Option<UnboundedReceiver<ZhenfaSignal>>,
    /// Webhook configuration for notifications.
    webhook_config: Option<WebhookConfig>,
    /// Shared HTTP client for webhook requests.
    http_client: Option<reqwest::Client>,
}

impl ZhenfaGatewayBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one domain router plugin.
    #[must_use]
    pub fn add_router<R>(mut self, router: R) -> Self
    where
        R: ZhenfaRouter + 'static,
    {
        self.routers.push(Box::new(router));
        self
    }

    /// Register one ad-hoc RPC method handler.
    #[must_use]
    pub fn register_method_fn<F, Fut>(mut self, method: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Value, Option<JsonRpcMeta>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, JsonRpcErrorObject>> + Send + 'static,
    {
        self.methods.register_fn(method, handler);
        self
    }

    /// Add signal receiver for webhook notifications.
    ///
    /// When signals are received, the gateway will send notifications
    /// to the configured webhook endpoint.
    #[must_use]
    pub fn with_signal_receiver(mut self, rx: UnboundedReceiver<ZhenfaSignal>) -> Self {
        self.signal_rx = Some(rx);
        self
    }

    /// Configure webhook for notifications.
    #[must_use]
    pub fn with_webhook(mut self, config: WebhookConfig) -> Self {
        self.webhook_config = Some(config);
        self
    }

    /// Use a shared HTTP client for webhook requests.
    ///
    /// This enables efficient connection pooling across all notification requests.
    #[must_use]
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the Axum router with optional notification sidecar.
    ///
    /// # Errors
    /// Returns an error when router prefixes are invalid or duplicate.
    pub fn build(mut self) -> Result<Router, ZhenfaGatewayBuildError> {
        let mut prefixes = HashSet::new();
        for router in &self.routers {
            let prefix = router.prefix();
            validate_prefix(prefix)?;
            if !prefixes.insert(prefix.to_string()) {
                return Err(ZhenfaGatewayBuildError::DuplicatePrefix(prefix.to_string()));
            }
            router.register_methods(&mut self.methods);
        }

        let methods = self.methods.clone();
        let mut app = Router::new()
            .route("/healthz", get(health_handler))
            .route("/rpc", post(rpc_handler))
            .layer(Extension(methods));

        for router in &self.routers {
            app = router.mount(app);
        }

        // Spawn notification sidecar if configured
        if let (Some(rx), Some(config)) = (self.signal_rx, self.webhook_config) {
            let client = self.http_client.unwrap_or_else(|| {
                reqwest::Client::builder()
                    .brotli(true)
                    .deflate(true)
                    .gzip(true)
                    .zstd(true)
                    .pool_idle_timeout(std::time::Duration::from_secs(120))
                    .pool_max_idle_per_host(32)
                    .connect_timeout(std::time::Duration::from_secs(5))
                    .timeout(std::time::Duration::from_secs(config.timeout_secs))
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new())
            });

            let service = Arc::new(NotificationService::with_client(config, client));
            tokio::spawn(notification_worker(rx, service));
        }

        Ok(app)
    }
}

fn validate_prefix(prefix: &str) -> Result<(), ZhenfaGatewayBuildError> {
    let valid = prefix.starts_with('/') && prefix != "/" && !prefix.contains("//");
    if valid {
        Ok(())
    } else {
        Err(ZhenfaGatewayBuildError::InvalidPrefix(prefix.to_string()))
    }
}
