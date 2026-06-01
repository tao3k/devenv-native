//! Internal-plane security for `qianji-server` business routes.

use std::sync::Arc;

use anyhow::{Result, anyhow};
use axum::routing::Router;
use xiuxian_config_core::first_non_empty_lookup;
use xiuxian_security::{
    InternalServiceSecurity, XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV, with_internal_service_security,
};

pub(crate) const QIANJI_INTERNAL_PRINCIPAL_SECRET_ENV: &str =
    "XIUXIAN_QIANJI_INTERNAL_PRINCIPAL_SECRET";
const LEGACY_GATEWAY_INTERNAL_PRINCIPAL_SECRET_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_INTERNAL_PRINCIPAL_SECRET";
const QIANJI_INTERNAL_PRINCIPAL_REQUIRED_CODE: &str = "QIANJI_INTERNAL_PRINCIPAL_REQUIRED";

pub(crate) type QianjiInternalServiceSecurity = InternalServiceSecurity;

pub(crate) fn qianji_internal_service_security() -> Option<QianjiInternalServiceSecurity> {
    qianji_internal_principal_secret_with_lookup(&|key| std::env::var(key).ok()).map(|secret| {
        InternalServiceSecurity::gateway(
            secret,
            Arc::<str>::from(QIANJI_INTERNAL_PRINCIPAL_REQUIRED_CODE),
        )
    })
}

pub(crate) fn require_qianji_internal_service_security() -> Result<QianjiInternalServiceSecurity> {
    qianji_internal_service_security().ok_or_else(|| {
        anyhow!(
            "qianji-server is an internal-plane service and requires `{QIANJI_INTERNAL_PRINCIPAL_SECRET_ENV}` or `{XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV}`"
        )
    })
}

#[cfg(test)]
pub(crate) fn qianji_internal_principal_secret_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Arc<str>> {
    internal_principal_secret_with_lookup(lookup)
}

#[cfg(test)]
pub(crate) fn require_qianji_internal_service_security_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<QianjiInternalServiceSecurity> {
    qianji_internal_principal_secret_with_lookup(lookup)
        .map(|secret| {
            InternalServiceSecurity::gateway(
                secret,
                Arc::<str>::from(QIANJI_INTERNAL_PRINCIPAL_REQUIRED_CODE),
            )
        })
        .ok_or_else(|| {
            anyhow!(
                "qianji-server is an internal-plane service and requires `{QIANJI_INTERNAL_PRINCIPAL_SECRET_ENV}` or `{XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV}`"
            )
        })
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
    with_internal_service_security(router, security)
}
