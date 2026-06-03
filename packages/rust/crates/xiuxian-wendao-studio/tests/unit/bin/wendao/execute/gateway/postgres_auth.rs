use std::sync::Arc;

use xiuxian_security::{PublicApiTokenEnvironment, PublicApiTokenScopeSet, PublicApiTokenVerifier};

use super::{
    GATEWAY_API_TOKEN_POSTGRES_AUTO_MIGRATE_ENV, GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL,
    GatewayPostgresApiTokenRepository, gateway_postgres_auto_migrate_with_lookup,
};
use crate::bin_support::wendao::execute::gateway::security::{
    GatewayApiTokenAuthority, GatewayApiTokenRecord, GatewayApiTokenRepository,
};

#[test]
fn gateway_postgres_api_token_schema_declares_control_plane_table() {
    assert!(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL.contains("wendao_gateway_api_tokens"));
    assert!(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL.contains("token_prefix TEXT PRIMARY KEY"));
    assert!(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL.contains("verifier_hash TEXT NOT NULL"));
    assert!(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL.contains("scopes TEXT[] NOT NULL"));
    assert!(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL.contains("status TEXT NOT NULL"));
    assert!(GATEWAY_API_TOKEN_POSTGRES_SCHEMA_SQL.contains("expires_at_unix_seconds BIGINT"));
}

#[test]
fn gateway_postgres_auto_migrate_defaults_to_enabled() {
    assert!(gateway_postgres_auto_migrate_with_lookup(&|_| None));
}

#[test]
fn gateway_postgres_auto_migrate_accepts_disabled_flags() {
    for value in ["0", "false", "no", "off"] {
        assert!(!gateway_postgres_auto_migrate_with_lookup(&|key| {
            match key {
                GATEWAY_API_TOKEN_POSTGRES_AUTO_MIGRATE_ENV => Some(value.to_string()),
                _ => None,
            }
        }));
    }
}

#[tokio::test]
async fn gateway_postgres_repository_live_round_trip_when_test_dsn_is_set() {
    let Ok(dsn) = std::env::var("XIUXIAN_WENDAO_GATEWAY_AUTH_POSTGRES_TEST_DSN") else {
        return;
    };
    let repository = GatewayPostgresApiTokenRepository::connect(dsn.as_str(), true)
        .await
        .unwrap_or_else(|error| panic!("postgres test repository should connect: {error}"));
    let verifier = PublicApiTokenVerifier::new(Arc::<str>::from("gateway-public-token-verifier"))
        .unwrap_or_else(|error| panic!("verifier secret should be accepted: {error}"));
    let issued = verifier.issue(PublicApiTokenEnvironment::Test);
    repository
        .insert(GatewayApiTokenRecord::new(
            Arc::<str>::from(issued.token_prefix()),
            Arc::<str>::from(issued.verifier_hash()),
            PublicApiTokenScopeSet::new([Arc::<str>::from("gateway:https-json-sse")]),
        ))
        .await
        .unwrap_or_else(|error| panic!("postgres token insert should succeed: {error}"));

    let stored = repository
        .lookup(issued.token_prefix())
        .await
        .unwrap_or_else(|error| panic!("postgres token lookup should succeed: {error}"))
        .unwrap_or_else(|| panic!("postgres token lookup should return inserted token"));

    assert_eq!(stored.token_prefix(), issued.token_prefix());
    assert_eq!(stored.verifier_hash(), issued.verifier_hash());
}
