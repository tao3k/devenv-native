//! Runtime host setting resolution for Wendao Flight rerank services.

use super::RerankScoreWeights;
use std::env;

/// Effective runtime host settings after precedence resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectiveRerankFlightHostSettings {
    /// Schema version expected by the host.
    pub expected_schema_version: String,
    /// Embedding dimension expected by the host rerank route.
    pub rerank_dimension: usize,
    /// Score weights used by the host scorer.
    pub rerank_weights: RerankScoreWeights,
}

/// Parsed host-setting overrides and remaining positional args.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedRerankFlightHostOverrides {
    /// Parsed schema-version override, if provided.
    pub schema_version_override: Option<String>,
    /// Parsed rerank-dimension override, if provided.
    pub rerank_dimension_override: Option<usize>,
    /// Remaining positional args after removing known flags.
    pub positional_args: Vec<String>,
}

/// Inputs used to resolve effective rerank Flight host settings.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectiveRerankFlightHostSettingsInput {
    /// Explicit schema-version override from the command line.
    pub schema_version_override: Option<String>,
    /// Explicit rerank-dimension override from the command line.
    pub rerank_dimension_override: Option<usize>,
    /// Schema version supplied by file-backed configuration.
    pub file_backed_schema_version: Option<String>,
    /// Weights supplied by file-backed configuration.
    pub file_backed_weights: Option<RerankScoreWeights>,
    /// Fallback dimension supplied by env/default resolution.
    pub fallback_dimension: usize,
    /// Fallback weights supplied by env/default resolution.
    pub fallback_weights: RerankScoreWeights,
}

/// Split optional explicit host-setting overrides from binary args.
///
/// Returns parsed overrides plus the remaining positional args.
///
/// # Errors
///
/// Returns an error when a known host-setting flag is present without a value,
/// with a blank value, or with an invalid rerank dimension.
pub fn split_rerank_flight_host_overrides<I>(
    args: I,
) -> Result<ParsedRerankFlightHostOverrides, String>
where
    I: IntoIterator<Item = String>,
{
    let mut schema_version_override = None;
    let mut rerank_dimension_override = None;
    let mut positional_args = Vec::new();
    for argument in args {
        if let Some(flag_value) = argument.strip_prefix("--schema-version=") {
            if flag_value.trim().is_empty() {
                return Err("--schema-version must not be blank".to_string());
            }
            schema_version_override = Some(flag_value.to_string());
            continue;
        }
        if argument == "--schema-version" {
            return Err(
                "--schema-version requires a value; use --schema-version=<value>".to_string(),
            );
        }
        if let Some(flag_value) = argument.strip_prefix("--rerank-dimension=") {
            if flag_value.trim().is_empty() {
                return Err("--rerank-dimension must not be blank".to_string());
            }
            let dimension = flag_value
                .parse::<usize>()
                .map_err(|error| format!("invalid --rerank-dimension: {error}"))?;
            if dimension == 0 {
                return Err("--rerank-dimension must be greater than zero".to_string());
            }
            rerank_dimension_override = Some(dimension);
            continue;
        }
        if argument == "--rerank-dimension" {
            return Err(
                "--rerank-dimension requires a value; use --rerank-dimension=<value>".to_string(),
            );
        }
        positional_args.push(argument);
    }
    Ok(ParsedRerankFlightHostOverrides {
        schema_version_override,
        rerank_dimension_override,
        positional_args,
    })
}

/// Resolve the effective runtime host settings after applying precedence.
///
/// Precedence:
/// 1. explicit schema-version override
/// 2. file-backed schema version
/// 3. default `"v2"`
///
/// For score weights:
/// 1. file-backed weights
/// 2. env/default fallback weights supplied by the caller
#[must_use]
pub fn resolve_effective_rerank_flight_host_settings(
    input: EffectiveRerankFlightHostSettingsInput,
) -> EffectiveRerankFlightHostSettings {
    EffectiveRerankFlightHostSettings {
        expected_schema_version: input
            .schema_version_override
            .or(input.file_backed_schema_version)
            .unwrap_or_else(|| "v2".to_string()),
        rerank_dimension: input
            .rerank_dimension_override
            .unwrap_or(input.fallback_dimension),
        rerank_weights: input.file_backed_weights.unwrap_or(input.fallback_weights),
    }
}

/// Parse rerank score weights from environment variables.
///
/// Uses:
/// - `WENDAO_RERANK_VECTOR_WEIGHT`
/// - `WENDAO_RERANK_SEMANTIC_WEIGHT`
///
/// Missing values fall back to the shared `RerankScoreWeights::default()`.
///
/// # Errors
///
/// Returns an error when either env value cannot be parsed as `f64` or the
/// resolved weights are invalid.
pub fn rerank_score_weights_from_env() -> Result<RerankScoreWeights, String> {
    let vector_weight = env::var("WENDAO_RERANK_VECTOR_WEIGHT")
        .ok()
        .map(|value| {
            value
                .parse::<f64>()
                .map_err(|error| format!("invalid WENDAO_RERANK_VECTOR_WEIGHT: {error}"))
        })
        .transpose()?
        .unwrap_or(RerankScoreWeights::default().vector_weight);
    let semantic_weight = env::var("WENDAO_RERANK_SEMANTIC_WEIGHT")
        .ok()
        .map(|value| {
            value
                .parse::<f64>()
                .map_err(|error| format!("invalid WENDAO_RERANK_SEMANTIC_WEIGHT: {error}"))
        })
        .transpose()?
        .unwrap_or(RerankScoreWeights::default().semantic_weight);
    RerankScoreWeights::new(vector_weight, semantic_weight)
}

#[cfg(test)]
#[path = "../../tests/unit/transport/host_settings.rs"]
mod tests;
