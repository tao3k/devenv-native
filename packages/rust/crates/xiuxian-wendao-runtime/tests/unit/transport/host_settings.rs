use std::fmt::Display;

use super::{
    EffectiveRerankFlightHostSettings, EffectiveRerankFlightHostSettingsInput,
    ParsedRerankFlightHostOverrides, rerank_score_weights_from_env,
    resolve_effective_rerank_flight_host_settings, split_rerank_flight_host_overrides,
};
use crate::transport::RerankScoreWeights;

fn must_ok<T, E: Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn must_err<T, E>(result: Result<T, E>, context: &str) -> E {
    match result {
        Ok(_) => panic!("{context}"),
        Err(error) => error,
    }
}

#[test]
fn split_rerank_flight_host_overrides_extracts_flags() {
    let overrides: ParsedRerankFlightHostOverrides = must_ok(
        split_rerank_flight_host_overrides(vec![
            "--schema-version=v8".to_string(),
            "--rerank-dimension=4".to_string(),
            "alpha/repo".to_string(),
            "3".to_string(),
        ]),
        "host-setting flags should parse",
    );

    assert_eq!(overrides.schema_version_override.as_deref(), Some("v8"));
    assert_eq!(overrides.rerank_dimension_override, Some(4));
    assert_eq!(
        overrides.positional_args,
        vec!["alpha/repo".to_string(), "3".to_string()]
    );
}

#[test]
fn split_rerank_flight_host_overrides_rejects_blank_schema_version() {
    let error = must_err(
        split_rerank_flight_host_overrides(vec!["--schema-version=".to_string()]),
        "blank schema-version should fail",
    );

    assert_eq!(error, "--schema-version must not be blank");
}

#[test]
fn split_rerank_flight_host_overrides_rejects_zero_dimension() {
    let error = must_err(
        split_rerank_flight_host_overrides(vec!["--rerank-dimension=0".to_string()]),
        "zero rerank dimension should fail",
    );

    assert_eq!(error, "--rerank-dimension must be greater than zero");
}

#[test]
fn resolve_effective_rerank_flight_host_settings_applies_precedence() {
    let fallback_weights = must_ok(
        RerankScoreWeights::new(0.4, 0.6),
        "fallback weights should validate",
    );
    let file_backed_weights = must_ok(
        RerankScoreWeights::new(0.9, 0.1),
        "file-backed weights should validate",
    );

    let settings: EffectiveRerankFlightHostSettings =
        resolve_effective_rerank_flight_host_settings(EffectiveRerankFlightHostSettingsInput {
            schema_version_override: Some("v8".to_string()),
            rerank_dimension_override: Some(4),
            file_backed_schema_version: Some("v9".to_string()),
            file_backed_weights: Some(file_backed_weights),
            fallback_dimension: 3,
            fallback_weights,
        });

    assert_eq!(settings.expected_schema_version, "v8");
    assert_eq!(settings.rerank_dimension, 4);
    assert_eq!(settings.rerank_weights, file_backed_weights);
}

#[test]
fn rerank_score_weights_from_env_defaults_when_unset() {
    let weights = must_ok(
        rerank_score_weights_from_env(),
        "default env weights should resolve",
    );

    assert_eq!(weights, RerankScoreWeights::default());
}
