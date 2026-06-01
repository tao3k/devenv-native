//! Gateway public-surface security middleware.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Request, State};
use axum::http::header::{self, CONTENT_LENGTH};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::Router;
use xiuxian_config_core::first_non_empty_lookup;
use xiuxian_security::{PublicPlaneRateLimiter, SignedPrincipalSigner};

pub(crate) use xiuxian_security::{
    PublicProtocolSurface as GatewayPublicProtocolSurface,
    PublicSurfacePolicy as GatewaySurfacePolicy, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
    XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV,
};

pub(crate) const GATEWAY_BEARER_TOKEN_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN";
const GATEWAY_INTERNAL_PRINCIPAL_SECRET_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_INTERNAL_PRINCIPAL_SECRET";

#[derive(Clone)]
pub(crate) struct GatewaySurfaceSecurity {
    surface: GatewayPublicProtocolSurface,
    bearer_token: Option<Arc<str>>,
    signing_secret: Option<Arc<str>>,
    policy: GatewaySurfacePolicy,
    rate_limiter: Arc<PublicPlaneRateLimiter>,
}

impl GatewaySurfaceSecurity {
    pub(crate) fn new(
        surface: GatewayPublicProtocolSurface,
        bearer_token: Option<Arc<str>>,
    ) -> Self {
        let signing_secret = gateway_internal_principal_secret().or_else(|| bearer_token.clone());
        let policy = GatewaySurfacePolicy::new(u64::MAX, usize::MAX);
        Self {
            surface,
            bearer_token,
            signing_secret,
            policy: policy.clone(),
            rate_limiter: Arc::new(PublicPlaneRateLimiter::new(policy.rate_limit_per_second())),
        }
    }

    pub(crate) fn with_policy(mut self, policy: GatewaySurfacePolicy) -> Self {
        self.rate_limiter = Arc::new(PublicPlaneRateLimiter::new(policy.rate_limit_per_second()));
        self.policy = policy;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_signing_secret(mut self, signing_secret: Option<Arc<str>>) -> Self {
        self.signing_secret = signing_secret.or_else(|| self.bearer_token.clone());
        self
    }

    fn signed_principal(&self, presented_token: &str) -> Option<String> {
        let signing_secret = self.signing_secret.as_ref()?;
        Some(
            SignedPrincipalSigner::new(
                Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
                Arc::clone(signing_secret),
            )
            .sign_user_token(self.surface, presented_token),
        )
    }
}

pub(crate) fn gateway_bearer_token() -> Option<Arc<str>> {
    gateway_bearer_token_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_bearer_token_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    lookup(GATEWAY_BEARER_TOKEN_ENV)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(Arc::<str>::from)
}

pub(crate) fn gateway_internal_principal_secret() -> Option<Arc<str>> {
    gateway_internal_principal_secret_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_internal_principal_secret_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    first_non_empty_lookup(
        &[
            XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV,
            GATEWAY_INTERNAL_PRINCIPAL_SECRET_ENV,
        ],
        lookup,
    )
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .map(Arc::<str>::from)
}

pub(crate) fn with_gateway_surface_security<S>(
    router: Router<S>,
    security: GatewaySurfaceSecurity,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.route_layer(middleware::from_fn_with_state(
        security,
        require_gateway_surface_security,
    ))
}

async fn require_gateway_surface_security(
    State(security): State<GatewaySurfaceSecurity>,
    mut request: Request,
    next: Next,
) -> Response {
    if !security.rate_limiter.allow() {
        return rate_limited_response(security.surface);
    }
    if exceeds_declared_stream_budget(&request, security.policy.stream_budget_bytes()) {
        return stream_budget_response(security.surface, security.policy.stream_budget_bytes());
    }

    let Some(expected_token) = security.bearer_token.as_ref() else {
        request.headers_mut().remove(header::AUTHORIZATION);
        insert_internal_headers(&mut request, security.surface, None);
        return next.run(request).await;
    };
    let Some(presented_token) = extract_bearer_token(&request) else {
        return unauthorized_response(security.surface);
    };
    if presented_token != expected_token.as_ref() {
        return unauthorized_response(security.surface);
    }

    let signed_principal = security.signed_principal(presented_token);
    request.headers_mut().remove(header::AUTHORIZATION);
    insert_internal_headers(&mut request, security.surface, signed_principal.as_deref());
    next.run(request).await
}

fn exceeds_declared_stream_budget(request: &Request, stream_budget_bytes: usize) -> bool {
    request
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > stream_budget_bytes)
}

fn insert_internal_headers(
    request: &mut Request,
    surface: GatewayPublicProtocolSurface,
    signed_principal: Option<&str>,
) {
    insert_static_header(
        request,
        WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
        WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
    );
    insert_static_header(request, WENDAO_PUBLIC_PROTOCOL_HEADER, surface.protocol());
    insert_static_header(request, WENDAO_AUTH_SCOPE_HEADER, surface.scope());
    if let Some(signed_principal) = signed_principal {
        insert_dynamic_header(request, WENDAO_SIGNED_PRINCIPAL_HEADER, signed_principal);
    }
}

fn extract_bearer_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

fn insert_static_header(request: &mut Request, header_name: &'static str, value: &'static str) {
    request
        .headers_mut()
        .insert(header_name, HeaderValue::from_static(value));
}

fn insert_dynamic_header(request: &mut Request, header_name: &'static str, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        request.headers_mut().insert(header_name, value);
    }
}

fn rate_limited_response(surface: GatewayPublicProtocolSurface) -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "error": "gateway public protocol rate limit exceeded",
            "code": "RATE_LIMITED",
            "protocol": surface.protocol(),
            "requiredScope": surface.scope(),
        })),
    )
        .into_response()
}

fn stream_budget_response(
    surface: GatewayPublicProtocolSurface,
    stream_budget_bytes: usize,
) -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        Json(serde_json::json!({
            "error": "gateway public protocol stream budget exceeded",
            "code": "STREAM_BUDGET_EXCEEDED",
            "protocol": surface.protocol(),
            "requiredScope": surface.scope(),
            "streamBudgetBytes": stream_budget_bytes,
        })),
    )
        .into_response()
}

fn unauthorized_response(surface: GatewayPublicProtocolSurface) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "missing or invalid bearer token",
            "code": "UNAUTHORIZED",
            "protocol": surface.protocol(),
            "requiredScope": surface.scope(),
        })),
    )
        .into_response()
}
