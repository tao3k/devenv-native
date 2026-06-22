//! PostgreSQL-compatible Gateway API token repository.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio_postgres::{Client, NoTls, Row};
use xiuxian_security::PublicApiTokenScopeSet;

use crate::bin_support::wendao::execute::gateway::security::{
    GatewayApiTokenAuthority, GatewayApiTokenInsertFuture, GatewayApiTokenLookupFuture,
    GatewayApiTokenRecord, GatewayApiTokenRepository, GatewayApiTokenRepositoryError,
    GatewayApiTokenRepositoryHandle, GatewayApiTokenStatus, non_empty_lookup,
};

pub(crate) const GATEWAY_API_TOKEN_POSTGRES_DSN_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_AUTH_POSTGRES_DSN";
pub(crate) const GATEWAY_API_TOKEN_POSTGRES_AUTO_MIGRATE_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_AUTH_POSTGRES_AUTO_MIGRATE";

pub(crate) const GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL: &str = r"
CREATE TABLE IF NOT EXISTS wendao_gateway_api_tokens (
    token_prefix TEXT PRIMARY KEY,
    verifier_hash TEXT NOT NULL,
    scopes TEXT[] NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
    expires_at_unix_seconds BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS wendao_gateway_api_tokens_status_idx
    ON wendao_gateway_api_tokens (status);
";

const SELECT_API_TOKEN_SQL: &str = r"
SELECT token_prefix, verifier_hash, scopes, status, expires_at_unix_seconds
FROM wendao_gateway_api_tokens
WHERE token_prefix = $1
";

const UPSERT_API_TOKEN_SQL: &str = r"
INSERT INTO wendao_gateway_api_tokens (
    token_prefix,
    verifier_hash,
    scopes,
    status,
    expires_at_unix_seconds
) VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (token_prefix) DO UPDATE SET
    verifier_hash = EXCLUDED.verifier_hash,
    scopes = EXCLUDED.scopes,
    status = EXCLUDED.status,
    expires_at_unix_seconds = EXCLUDED.expires_at_unix_seconds,
    updated_at = now()
";

#[derive(Clone)]
pub(crate) struct GatewayPostgresApiTokenRepository {
    client: Arc<Client>,
}

impl GatewayPostgresApiTokenRepository {
    pub(crate) async fn connect(
        dsn: &str,
        auto_migrate: bool,
    ) -> Result<GatewayApiTokenRepositoryHandle, GatewayApiTokenRepositoryError> {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls)
            .await
            .map_err(repository_error)?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        if auto_migrate {
            client
                .batch_execute(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL)
                .await
                .map_err(repository_error)?;
        }
        Ok(Arc::new(Self {
            client: Arc::new(client),
        }))
    }

    async fn lookup_record(
        &self,
        token_prefix: &str,
    ) -> Result<Option<GatewayApiTokenRecord>, GatewayApiTokenRepositoryError> {
        let row = self
            .client
            .query_opt(SELECT_API_TOKEN_SQL, &[&token_prefix])
            .await
            .map_err(repository_error)?;
        row.map(record_from_row).transpose()
    }

    async fn insert_record(
        &self,
        record: GatewayApiTokenRecord,
    ) -> Result<(), GatewayApiTokenRepositoryError> {
        let scopes = record
            .scopes()
            .scopes()
            .iter()
            .map(|scope| scope.as_ref().to_string())
            .collect::<Vec<_>>();
        let expires_at_unix_seconds = record
            .expires_at_time()
            .and_then(system_time_to_unix_seconds);
        self.client
            .execute(
                UPSERT_API_TOKEN_SQL,
                &[
                    &record.token_prefix(),
                    &record.verifier_hash(),
                    &scopes,
                    &record.status().as_str(),
                    &expires_at_unix_seconds,
                ],
            )
            .await
            .map_err(repository_error)?;
        Ok(())
    }
}

impl GatewayApiTokenAuthority for GatewayPostgresApiTokenRepository {
    fn lookup<'a>(&'a self, token_prefix: &'a str) -> GatewayApiTokenLookupFuture<'a> {
        Box::pin(async move { self.lookup_record(token_prefix).await })
    }
}

impl GatewayApiTokenRepository for GatewayPostgresApiTokenRepository {
    fn insert<'a>(&'a self, record: GatewayApiTokenRecord) -> GatewayApiTokenInsertFuture<'a> {
        Box::pin(async move { self.insert_record(record).await })
    }
}

pub(crate) async fn gateway_postgres_api_token_repository_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<GatewayApiTokenRepositoryHandle>, GatewayApiTokenRepositoryError> {
    let Some(dsn) = non_empty_lookup(GATEWAY_API_TOKEN_POSTGRES_DSN_ENV, lookup) else {
        return Ok(None);
    };
    GatewayPostgresApiTokenRepository::connect(
        dsn.as_str(),
        gateway_postgres_auto_migrate_with_lookup(lookup),
    )
    .await
    .map(Some)
}

pub(crate) fn gateway_postgres_auto_migrate_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    match non_empty_lookup(GATEWAY_API_TOKEN_POSTGRES_AUTO_MIGRATE_ENV, lookup).as_deref() {
        Some("0" | "false" | "no" | "off") => false,
        Some(_) | None => true,
    }
}

fn record_from_row(row: Row) -> Result<GatewayApiTokenRecord, GatewayApiTokenRepositoryError> {
    let token_prefix = row.get::<_, String>("token_prefix");
    let verifier_hash = row.get::<_, String>("verifier_hash");
    let scopes = row
        .get::<_, Vec<String>>("scopes")
        .into_iter()
        .map(Arc::<str>::from)
        .collect::<Vec<_>>();
    let status = row.get::<_, String>("status");
    let Some(status) = GatewayApiTokenStatus::parse(status.as_str()) else {
        return Err(GatewayApiTokenRepositoryError::new(
            "postgres token record has invalid status",
        ));
    };
    let expires_at = row
        .get::<_, Option<i64>>("expires_at_unix_seconds")
        .map(system_time_from_unix_seconds)
        .transpose()?;
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
    Ok(record)
}

fn system_time_from_unix_seconds(
    seconds: i64,
) -> Result<SystemTime, GatewayApiTokenRepositoryError> {
    let seconds = u64::try_from(seconds).map_err(|_error| {
        GatewayApiTokenRepositoryError::new("postgres token expiry cannot be negative")
    })?;
    SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(seconds))
        .ok_or_else(|| GatewayApiTokenRepositoryError::new("postgres token expiry is out of range"))
}

fn system_time_to_unix_seconds(time: SystemTime) -> Option<i64> {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    i64::try_from(duration.as_secs()).ok()
}

fn repository_error(error: impl std::fmt::Display) -> GatewayApiTokenRepositoryError {
    GatewayApiTokenRepositoryError::new(format!("postgres token repository error: {error}"))
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/execute/gateway/postgres_auth.rs"]
mod tests;
