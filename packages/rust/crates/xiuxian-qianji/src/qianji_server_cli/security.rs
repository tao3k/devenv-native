//! Internal-plane security for `qianji-server` business routes.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::Router;
use xiuxian_config_core::first_non_empty_lookup;
use xiuxian_security::{
    PublicProtocolSurface, SignedPrincipalVerifier, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
    XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV,
};

const QIANJI_INTERNAL_PRINCIPAL_SECRET_ENV: &str = "XIUXIAN_QIANJI_INTERNAL_PRINCIPAL_SECRET";
const LEGACY_GATEWAY_INTERNAL_PRINCIPAL_SECRET_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_INTERNAL_PRINCIPAL_SECRET";

#[derive(Clone)]
pub(crate) struct QianjiInternalServiceSecurity {
    verifier: SignedPrincipalVerifier,
}

impl QianjiInternalServiceSecurity {
    pub(crate) fn new(signing_secret: Arc<str>) -> Self {
        Self {
            verifier: SignedPrincipalVerifier::new(
                Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
                signing_secret,
            ),
        }
    }
}

pub(crate) fn qianji_internal_service_security() -> Option<QianjiInternalServiceSecurity> {
    qianji_internal_principal_secret_with_lookup(&|key| std::env::var(key).ok())
        .map(QianjiInternalServiceSecurity::new)
}

#[cfg(test)]
pub(crate) fn qianji_internal_principal_secret_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    internal_principal_secret_with_lookup(lookup)
}

#[cfg(not(test))]
fn qianji_internal_principal_secret_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    internal_principal_secret_with_lookup(lookup)
}

fn internal_principal_secret_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    first_non_empty_lookup(
        &[
            QIANJI_INTERNAL_PRINCIPAL_SECRET_ENV,
            XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV,
            LEGACY_GATEWAY_INTERNAL_PRINCIPAL_SECRET_ENV,
        ],
        lookup,
    )
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .map(Arc::<str>::from)
}

pub(crate) fn with_qianji_internal_service_security<S>(
    router: Router<S>,
    security: QianjiInternalServiceSecurity,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.route_layer(middleware::from_fn_with_state(
        security,
        require_qianji_internal_service_security,
    ))
}

async fn require_qianji_internal_service_security(
    State(security): State<QianjiInternalServiceSecurity>,
    request: Request,
    next: Next,
) -> Response {
    if request.headers().contains_key(header::AUTHORIZATION) {
        return unauthorized_response("raw public Authorization header is not accepted");
    }

    let Some(service_identity) = header_str(&request, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER)
    else {
        return unauthorized_response("missing internal service identity");
    };
    let Some(protocol) = header_str(&request, WENDAO_PUBLIC_PROTOCOL_HEADER) else {
        return unauthorized_response("missing public protocol");
    };
    let Some(scope) = header_str(&request, WENDAO_AUTH_SCOPE_HEADER) else {
        return unauthorized_response("missing auth scope");
    };
    let Some(signed_principal) = header_str(&request, WENDAO_SIGNED_PRINCIPAL_HEADER) else {
        return unauthorized_response("missing signed principal");
    };
    let Some(surface) = surface_from_headers(protocol, scope) else {
        return unauthorized_response("invalid public protocol or auth scope");
    };
    if !security
        .verifier
        .verify_signed_principal(surface, service_identity, signed_principal)
    {
        return unauthorized_response("invalid signed principal");
    }

    next.run(request).await
}

fn header_str<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

fn surface_from_headers(protocol: &str, scope: &str) -> Option<PublicProtocolSurface> {
    [
        PublicProtocolSurface::HttpsJsonSse,
        PublicProtocolSurface::ArrowFlight,
    ]
    .into_iter()
    .find(|surface| surface.protocol() == protocol && surface.scope() == scope)
}

fn unauthorized_response(message: &'static str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": message,
            "code": "QIANJI_INTERNAL_PRINCIPAL_REQUIRED",
        })),
    )
        .into_response()
}
