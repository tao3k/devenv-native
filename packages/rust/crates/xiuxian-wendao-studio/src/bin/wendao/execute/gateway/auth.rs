//! Gateway-owned public API token issuance.

use std::sync::Arc;

use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{Router, post};
use serde::{Deserialize, Serialize};
use tower_http::limit::RequestBodyLimitLayer;
use xiuxian_security::{
    PublicApiTokenEnvironment, PublicApiTokenScopeSet, PublicApiTokenVerifier,
    PublicPlaneRateLimiter, PublicProtocolSurface,
};

use crate::bin_support::wendao::execute::gateway::security::{
    GatewayApiTokenAdmission, GatewayApiTokenRecord, GatewayApiTokenRepositoryHandle,
    non_empty_lookup,
};
use crate::contracts::routes::API_AUTH_TOKENS_AXUM_PATH;

pub(crate) const GATEWAY_AUTH_USERNAME_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_AUTH_USERNAME";
pub(crate) const GATEWAY_AUTH_PASSWORD_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_AUTH_PASSWORD";
pub(crate) const GATEWAY_AUTH_ALLOWED_SCOPES_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_AUTH_ALLOWED_SCOPES";

const AUTH_TOKEN_REQUEST_BUDGET_BYTES: usize = 16 * 1024;

/// Gateway API-token issuer for bootstrap and development flows.
///
/// This issuer writes verifier metadata into the configured token repository.
/// Production deployments should use the PostgreSQL-compatible repository
/// adapter instead of the process-local in-memory repository.
#[derive(Clone)]
pub(crate) struct GatewayAuthIssuer {
    verifier: PublicApiTokenVerifier,
    repository: GatewayApiTokenRepositoryHandle,
    username: Arc<str>,
    password: Arc<str>,
    allowed_scopes: PublicApiTokenScopeSet,
    rate_limiter: Arc<PublicPlaneRateLimiter>,
}

impl GatewayAuthIssuer {
    pub(crate) fn new(
        verifier: PublicApiTokenVerifier,
        repository: GatewayApiTokenRepositoryHandle,
        username: Arc<str>,
        password: Arc<str>,
        allowed_scopes: PublicApiTokenScopeSet,
        rate_limit_per_second: u64,
    ) -> Self {
        Self {
            verifier,
            repository,
            username,
            password,
            allowed_scopes,
            rate_limiter: Arc::new(PublicPlaneRateLimiter::new(rate_limit_per_second)),
        }
    }

    fn authenticate(&self, username: &str, password: &str) -> bool {
        constant_time_eq(username.as_bytes(), self.username.as_bytes())
            && constant_time_eq(password.as_bytes(), self.password.as_bytes())
    }

    async fn issue(
        &self,
        request: &GatewayAuthTokenRequest,
    ) -> Result<GatewayAuthTokenResponse, GatewayAuthError> {
        if !self.rate_limiter.allow() {
            return Err(GatewayAuthError::RateLimited);
        }
        if !self.authenticate(request.username.trim(), request.password.as_str()) {
            return Err(GatewayAuthError::InvalidCredentials);
        }

        let scopes = self.resolve_scopes(&request.scopes)?;
        let issued = self.verifier.issue(request.environment.into());
        self.repository
            .insert(GatewayApiTokenRecord::new(
                Arc::<str>::from(issued.token_prefix()),
                Arc::<str>::from(issued.verifier_hash()),
                scopes.clone(),
            ))
            .await
            .map_err(|_error| GatewayAuthError::RepositoryUnavailable)?;

        Ok(GatewayAuthTokenResponse {
            object: "api_key",
            token: issued.presented_token().to_string(),
            token_prefix: issued.token_prefix().to_string(),
            environment: request.environment,
            scopes: scopes
                .scopes()
                .iter()
                .map(|scope| scope.as_ref().to_string())
                .collect(),
        })
    }

    fn resolve_scopes(
        &self,
        requested_scopes: &[String],
    ) -> Result<PublicApiTokenScopeSet, GatewayAuthError> {
        let requested = requested_scopes
            .iter()
            .map(|scope| scope.trim())
            .filter(|scope| !scope.is_empty())
            .collect::<Vec<_>>();
        if requested.is_empty() {
            return Ok(self.allowed_scopes.clone());
        }
        for scope in &requested {
            if !self.allowed_scopes.contains_scope(scope) {
                return Err(GatewayAuthError::ScopeNotAllowed);
            }
        }
        Ok(PublicApiTokenScopeSet::new(
            requested.into_iter().map(Arc::<str>::from),
        ))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GatewayAuthTokenEnvironment {
    Live,
    Test,
}

impl Default for GatewayAuthTokenEnvironment {
    fn default() -> Self {
        Self::Test
    }
}

impl From<GatewayAuthTokenEnvironment> for PublicApiTokenEnvironment {
    fn from(environment: GatewayAuthTokenEnvironment) -> Self {
        match environment {
            GatewayAuthTokenEnvironment::Live => Self::Live,
            GatewayAuthTokenEnvironment::Test => Self::Test,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GatewayAuthTokenRequest {
    username: String,
    password: String,
    #[serde(default)]
    scopes: Vec<String>,
    #[serde(default)]
    environment: GatewayAuthTokenEnvironment,
}

#[derive(Debug, Serialize)]
struct GatewayAuthTokenResponse {
    object: &'static str,
    token: String,
    #[serde(rename = "tokenPrefix")]
    token_prefix: String,
    environment: GatewayAuthTokenEnvironment,
    scopes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GatewayAuthError {
    InvalidCredentials,
    RateLimited,
    RepositoryUnavailable,
    ScopeNotAllowed,
}

impl GatewayAuthError {
    const fn status(self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::RepositoryUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::ScopeNotAllowed => StatusCode::FORBIDDEN,
        }
    }

    const fn code(self) -> &'static str {
        match self {
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::RateLimited => "RATE_LIMITED",
            Self::RepositoryUnavailable => "TOKEN_REPOSITORY_UNAVAILABLE",
            Self::ScopeNotAllowed => "SCOPE_NOT_ALLOWED",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::InvalidCredentials => "invalid gateway login credentials",
            Self::RateLimited => "gateway login rate limit exceeded",
            Self::RepositoryUnavailable => "gateway token repository unavailable",
            Self::ScopeNotAllowed => "requested gateway auth scope is not allowed",
        }
    }
}

pub(crate) fn gateway_auth_router<S>(issuer: GatewayAuthIssuer) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route(API_AUTH_TOKENS_AXUM_PATH, post(issue_gateway_api_token))
        .layer(RequestBodyLimitLayer::new(AUTH_TOKEN_REQUEST_BUDGET_BYTES))
        .layer(Extension(issuer))
}

pub(crate) fn gateway_auth_issuer(
    admission: Option<GatewayApiTokenAdmission>,
    rate_limit_per_second: u64,
) -> Option<GatewayAuthIssuer> {
    gateway_auth_issuer_with_lookup(admission, rate_limit_per_second, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn gateway_auth_issuer_with_lookup(
    admission: Option<GatewayApiTokenAdmission>,
    rate_limit_per_second: u64,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<GatewayAuthIssuer> {
    let (verifier, store) = admission?;
    let username = non_empty_lookup(GATEWAY_AUTH_USERNAME_ENV, lookup)?;
    let password = non_empty_lookup(GATEWAY_AUTH_PASSWORD_ENV, lookup)?;
    Some(GatewayAuthIssuer::new(
        verifier,
        store,
        Arc::<str>::from(username),
        Arc::<str>::from(password),
        gateway_auth_allowed_scopes_with_lookup(lookup),
        rate_limit_per_second,
    ))
}

pub(crate) fn gateway_auth_allowed_scopes_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> PublicApiTokenScopeSet {
    let scopes = match lookup(GATEWAY_AUTH_ALLOWED_SCOPES_ENV)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        Some(value) => value
            .split(',')
            .map(str::trim)
            .filter(|scope| !scope.is_empty())
            .filter(|scope| is_public_gateway_scope(scope))
            .map(Arc::<str>::from)
            .collect::<Vec<_>>(),
        None => default_gateway_auth_scopes(),
    };
    PublicApiTokenScopeSet::new(scopes)
}

async fn issue_gateway_api_token(
    Extension(issuer): Extension<GatewayAuthIssuer>,
    Json(request): Json<GatewayAuthTokenRequest>,
) -> Response {
    match issuer.issue(&request).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => (
            error.status(),
            Json(serde_json::json!({
                "error": error.message(),
                "code": error.code(),
            })),
        )
            .into_response(),
    }
}

fn default_gateway_auth_scopes() -> Vec<Arc<str>> {
    vec![
        Arc::<str>::from(PublicProtocolSurface::HttpsJsonSse.scope()),
        Arc::<str>::from(PublicProtocolSurface::ArrowFlight.scope()),
    ]
}

fn is_public_gateway_scope(scope: &str) -> bool {
    scope == PublicProtocolSurface::HttpsJsonSse.scope()
        || scope == PublicProtocolSurface::ArrowFlight.scope()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0;
    for (left_byte, right_byte) in left.iter().zip(right) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/auth.rs"]
mod tests;
