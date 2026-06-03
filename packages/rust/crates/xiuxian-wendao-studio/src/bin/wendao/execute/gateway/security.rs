//! Gateway public-surface security middleware.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use std::time::SystemTime;

use axum::Json;
use axum::extract::{Request, State};
use axum::http::header::{self, CONTENT_LENGTH};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::Router;
use xiuxian_config_core::first_non_empty_lookup;
use xiuxian_security::{
    PublicApiTokenParts, PublicApiTokenScopeSet, PublicApiTokenVerifier, PublicPlaneRateLimiter,
    SignedPrincipalSigner,
};

pub(crate) use xiuxian_security::{
    PublicProtocolSurface as GatewayPublicProtocolSurface,
    PublicSurfacePolicy as GatewaySurfacePolicy, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
    XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV,
};

pub(crate) const GATEWAY_BEARER_TOKEN_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN";
pub(crate) const GATEWAY_API_TOKEN_VERIFIER_SECRET_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_API_TOKEN_VERIFIER_SECRET";
pub(crate) const GATEWAY_API_TOKEN_PREFIX_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_API_TOKEN_PREFIX";
pub(crate) const GATEWAY_API_TOKEN_VERIFIER_HASH_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_API_TOKEN_VERIFIER_HASH";
pub(crate) const GATEWAY_API_TOKEN_SCOPES_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_API_TOKEN_SCOPES";
pub(crate) const GATEWAY_API_TOKEN_STATUS_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_API_TOKEN_STATUS";
pub(crate) const GATEWAY_API_TOKEN_EXPIRES_AT_UNIX_SECONDS_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_API_TOKEN_EXPIRES_AT_UNIX_SECONDS";
const GATEWAY_INTERNAL_PRINCIPAL_SECRET_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_INTERNAL_PRINCIPAL_SECRET";

#[derive(Clone)]
pub(crate) struct GatewaySurfaceSecurity {
    surface: GatewayPublicProtocolSurface,
    bearer_token: Option<Arc<str>>,
    api_token_verifier: Option<PublicApiTokenVerifier>,
    api_token_authority: Option<GatewayApiTokenRepositoryHandle>,
    signing_secret: Option<Arc<str>>,
    policy: GatewaySurfacePolicy,
    rate_limiter: Arc<PublicPlaneRateLimiter>,
}

pub(crate) type GatewayApiTokenAdmission =
    (PublicApiTokenVerifier, GatewayApiTokenRepositoryHandle);
pub(crate) type GatewayApiTokenRepositoryHandle = Arc<dyn GatewayApiTokenRepository>;

pub(crate) trait GatewayApiTokenAuthority: Send + Sync {
    fn lookup<'a>(&'a self, token_prefix: &'a str) -> GatewayApiTokenLookupFuture<'a>;
}

pub(crate) type GatewayApiTokenLookupFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Option<GatewayApiTokenRecord>, GatewayApiTokenRepositoryError>>
            + Send
            + 'a,
    >,
>;

pub(crate) trait GatewayApiTokenRepository: GatewayApiTokenAuthority {
    fn insert<'a>(&'a self, record: GatewayApiTokenRecord) -> GatewayApiTokenInsertFuture<'a>;
}

pub(crate) type GatewayApiTokenInsertFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), GatewayApiTokenRepositoryError>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GatewayApiTokenRepositoryError {
    message: Arc<str>,
}

impl GatewayApiTokenRepositoryError {
    pub(crate) fn new(message: impl Into<Arc<str>>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for GatewayApiTokenRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GatewayApiTokenRepositoryError {}

#[derive(Clone, Debug, Default)]
pub(crate) struct GatewayInMemoryApiTokenStore {
    records: Arc<RwLock<HashMap<Arc<str>, GatewayApiTokenRecord>>>,
}

impl GatewayInMemoryApiTokenStore {
    /// Insert a token record into this process-local cache.
    ///
    /// This is intentionally not a durable token authority. Production
    /// deployments should first source token lifecycle truth from a
    /// PostgreSQL-compatible control-plane store, then project audit/read-model
    /// facts elsewhere. Managed AuthZ services can wrap that repository in
    /// later slices, but they are not the first supported authority.
    pub(crate) fn insert(&self, record: GatewayApiTokenRecord) {
        if let Ok(mut records) = self.records.write() {
            records.insert(Arc::clone(&record.token_prefix), record);
        }
    }

    fn lookup_record(&self, token_prefix: &str) -> Option<GatewayApiTokenRecord> {
        let Ok(records) = self.records.read() else {
            return None;
        };
        records.get(token_prefix).cloned()
    }
}

impl GatewayApiTokenAuthority for GatewayInMemoryApiTokenStore {
    fn lookup<'a>(&'a self, token_prefix: &'a str) -> GatewayApiTokenLookupFuture<'a> {
        Box::pin(async move { Ok(self.lookup_record(token_prefix)) })
    }
}

impl GatewayApiTokenRepository for GatewayInMemoryApiTokenStore {
    fn insert<'a>(&'a self, record: GatewayApiTokenRecord) -> GatewayApiTokenInsertFuture<'a> {
        Box::pin(async move {
            self.insert(record);
            Ok(())
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GatewayApiTokenStatus {
    Active,
    Revoked,
}

impl GatewayApiTokenStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Revoked => "revoked",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GatewayApiTokenRecord {
    token_prefix: Arc<str>,
    verifier_hash: Arc<str>,
    scopes: PublicApiTokenScopeSet,
    status: GatewayApiTokenStatus,
    expires_at: Option<SystemTime>,
}

impl GatewayApiTokenRecord {
    pub(crate) fn new(
        token_prefix: Arc<str>,
        verifier_hash: Arc<str>,
        scopes: PublicApiTokenScopeSet,
    ) -> Self {
        Self {
            token_prefix,
            verifier_hash,
            scopes,
            status: GatewayApiTokenStatus::Active,
            expires_at: None,
        }
    }

    pub(crate) const fn revoked(mut self) -> Self {
        self.status = GatewayApiTokenStatus::Revoked;
        self
    }

    pub(crate) const fn expires_at(mut self, expires_at: SystemTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub(crate) fn token_prefix(&self) -> &str {
        self.token_prefix.as_ref()
    }

    pub(crate) fn verifier_hash(&self) -> &str {
        self.verifier_hash.as_ref()
    }

    pub(crate) const fn scopes(&self) -> &PublicApiTokenScopeSet {
        &self.scopes
    }

    pub(crate) const fn status(&self) -> GatewayApiTokenStatus {
        self.status
    }

    pub(crate) const fn expires_at_time(&self) -> Option<SystemTime> {
        self.expires_at
    }

    fn is_admissible_at(&self, now: SystemTime) -> bool {
        if self.status != GatewayApiTokenStatus::Active {
            return false;
        }
        match self.expires_at {
            Some(expires_at) => expires_at > now,
            None => true,
        }
    }
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
            api_token_verifier: None,
            api_token_authority: None,
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

    pub(crate) fn with_api_token_admission(
        self,
        verifier: PublicApiTokenVerifier,
        repository: GatewayApiTokenRepositoryHandle,
    ) -> Self {
        self.with_api_token_authority(verifier, repository)
    }

    pub(crate) fn with_api_token_authority(
        mut self,
        verifier: PublicApiTokenVerifier,
        authority: GatewayApiTokenRepositoryHandle,
    ) -> Self {
        self.api_token_verifier = Some(verifier);
        self.api_token_authority = Some(authority);
        self
    }

    #[cfg(test)]
    pub(crate) fn with_signing_secret(mut self, signing_secret: Option<Arc<str>>) -> Self {
        self.signing_secret = signing_secret.or_else(|| self.bearer_token.clone());
        self
    }

    fn requires_public_credential(&self) -> bool {
        self.bearer_token.is_some()
            || (self.api_token_verifier.is_some() && self.api_token_authority.is_some())
    }

    async fn presented_token_admission(
        &self,
        presented_token: &str,
    ) -> GatewayPresentedTokenAdmission {
        if self.accepts_static_bearer_token(presented_token) {
            return GatewayPresentedTokenAdmission::Accepted;
        }
        match self.accepts_public_api_token(presented_token).await {
            Ok(true) => GatewayPresentedTokenAdmission::Accepted,
            Ok(false) => GatewayPresentedTokenAdmission::Rejected,
            Err(_) => GatewayPresentedTokenAdmission::AuthorityUnavailable,
        }
    }

    fn accepts_static_bearer_token(&self, presented_token: &str) -> bool {
        self.bearer_token
            .as_ref()
            .is_some_and(|expected_token| presented_token == expected_token.as_ref())
    }

    async fn accepts_public_api_token(
        &self,
        presented_token: &str,
    ) -> Result<bool, GatewayApiTokenRepositoryError> {
        let Some(verifier) = self.api_token_verifier.as_ref() else {
            return Ok(false);
        };
        let Some(authority) = self.api_token_authority.as_ref() else {
            return Ok(false);
        };
        let Ok(parts) = PublicApiTokenParts::parse(presented_token) else {
            return Ok(false);
        };
        let Some(record) = authority.lookup(parts.token_prefix()).await? else {
            return Ok(false);
        };
        if !record.is_admissible_at(SystemTime::now()) {
            return Ok(false);
        };
        Ok(record.scopes.allows_surface(self.surface)
            && verifier.verify_presented_token(
                presented_token,
                record.token_prefix.as_ref(),
                record.verifier_hash.as_ref(),
            ))
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GatewayPresentedTokenAdmission {
    Accepted,
    Rejected,
    AuthorityUnavailable,
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

pub(crate) fn gateway_api_token_admission() -> Option<GatewayApiTokenAdmission> {
    gateway_api_token_admission_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_api_token_admission_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<GatewayApiTokenAdmission> {
    let verifier_secret = lookup(GATEWAY_API_TOKEN_VERIFIER_SECRET_ENV)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())?;
    let verifier = PublicApiTokenVerifier::new(Arc::<str>::from(verifier_secret)).ok()?;
    let store = GatewayInMemoryApiTokenStore::default();
    if let Some(record) = gateway_seed_api_token_record_with_lookup(lookup) {
        store.insert(record);
    }
    Some((verifier, Arc::new(store)))
}

fn gateway_seed_api_token_record_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<GatewayApiTokenRecord> {
    let token_prefix = non_empty_lookup(GATEWAY_API_TOKEN_PREFIX_ENV, lookup)?;
    let verifier_hash = non_empty_lookup(GATEWAY_API_TOKEN_VERIFIER_HASH_ENV, lookup)?;
    let scopes = lookup(GATEWAY_API_TOKEN_SCOPES_ENV)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|scope| !scope.is_empty())
                .map(Arc::<str>::from)
                .collect::<Vec<_>>()
        })
        .filter(|scopes| !scopes.is_empty())?;
    let status = gateway_seed_api_token_status_with_lookup(lookup)?;
    let expires_at = gateway_seed_api_token_expires_at_with_lookup(lookup)?;
    let mut record = GatewayApiTokenRecord::new(
        Arc::<str>::from(token_prefix),
        Arc::<str>::from(verifier_hash),
        PublicApiTokenScopeSet::new(scopes),
    );
    if status == GatewayApiTokenStatus::Revoked {
        record = record.revoked();
    }
    if let Some(expires_at) = expires_at {
        record = record.expires_at(expires_at);
    }
    Some(record)
}

fn gateway_seed_api_token_status_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<GatewayApiTokenStatus> {
    match non_empty_lookup(GATEWAY_API_TOKEN_STATUS_ENV, lookup).as_deref() {
        Some(value) => GatewayApiTokenStatus::parse(value),
        None => Some(GatewayApiTokenStatus::Active),
    }
}

fn gateway_seed_api_token_expires_at_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Option<SystemTime>> {
    let Some(raw_seconds) = non_empty_lookup(GATEWAY_API_TOKEN_EXPIRES_AT_UNIX_SECONDS_ENV, lookup)
    else {
        return Some(None);
    };
    let seconds = raw_seconds.parse::<u64>().ok()?;
    let expires_at = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))?;
    Some(Some(expires_at))
}

pub(crate) fn non_empty_lookup(
    key: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    lookup(key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

    if !security.requires_public_credential() {
        request.headers_mut().remove(header::AUTHORIZATION);
        insert_internal_headers(&mut request, security.surface, None);
        return next.run(request).await;
    }
    let Some(presented_token) = extract_bearer_token(&request) else {
        return unauthorized_response(security.surface);
    };
    match security.presented_token_admission(presented_token).await {
        GatewayPresentedTokenAdmission::Accepted => {}
        GatewayPresentedTokenAdmission::Rejected => return unauthorized_response(security.surface),
        GatewayPresentedTokenAdmission::AuthorityUnavailable => {
            return token_authority_unavailable_response(security.surface);
        }
    }

    let Some(signed_principal) = security.signed_principal(presented_token) else {
        return unauthorized_response(security.surface);
    };
    request.headers_mut().remove(header::AUTHORIZATION);
    insert_internal_headers(
        &mut request,
        security.surface,
        Some(signed_principal.as_str()),
    );
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

fn token_authority_unavailable_response(surface: GatewayPublicProtocolSurface) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "gateway token authority unavailable",
            "code": "TOKEN_AUTHORITY_UNAVAILABLE",
            "protocol": surface.protocol(),
            "requiredScope": surface.scope(),
        })),
    )
        .into_response()
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/security.rs"]
mod tests;
