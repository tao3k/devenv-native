//! Gateway runtime policy and public-surface limits.

use std::time::Duration;

use xiuxian_config_core::{lookup_bool_flag, lookup_positive_parsed};

use crate::bin_support::wendao::execute::gateway::config::GatewayRuntimeTomlConfig;

const GATEWAY_LISTEN_BACKLOG_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_LISTEN_BACKLOG";
const GATEWAY_STUDIO_CONCURRENCY_LIMIT_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT";
const GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS";
const GATEWAY_FLIGHT_CONCURRENCY_LIMIT_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_FLIGHT_CONCURRENCY_LIMIT";
const GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS";
const GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND";
const GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND";
const GATEWAY_HTTPS_STREAM_BUDGET_BYTES_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_HTTPS_STREAM_BUDGET_BYTES";
const GATEWAY_FLIGHT_STREAM_BUDGET_BYTES_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES";
const GATEWAY_FLIGHT_GRPC_WEB_ENABLED_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_FLIGHT_GRPC_WEB_ENABLED";
const DEFAULT_GATEWAY_LISTEN_BACKLOG: u32 = 2048;
const MIN_GATEWAY_LISTEN_BACKLOG: u32 = 128;
const MAX_GATEWAY_LISTEN_BACKLOG: u32 = 8192;
const DEFAULT_GATEWAY_STUDIO_CONCURRENCY_FALLBACK: usize = 8;
const MIN_GATEWAY_STUDIO_CONCURRENCY_LIMIT: usize = 32;
const MAX_GATEWAY_STUDIO_CONCURRENCY_LIMIT: usize = 128;
const DEFAULT_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS: u64 = 15;
const MIN_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS: u64 = 5;
const MAX_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS: u64 = 60;
const MIN_GATEWAY_FLIGHT_CONCURRENCY_LIMIT: usize = 4;
const MAX_GATEWAY_FLIGHT_CONCURRENCY_LIMIT: usize = 128;
const MIN_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS: u64 = 5;
const MAX_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS: u64 = 900;
const MIN_GATEWAY_RATE_LIMIT_PER_SECOND: u64 = 1;
const MAX_GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND: u64 = 4096;
const MAX_GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND: u64 = 2048;
const DEFAULT_GATEWAY_HTTPS_STREAM_BUDGET_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES: usize = 1024 * 1024 * 1024;
const MIN_GATEWAY_STREAM_BUDGET_BYTES: usize = 1024;
const MAX_GATEWAY_HTTPS_STREAM_BUDGET_BYTES: usize = 512 * 1024 * 1024;
const MAX_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES: usize = 4 * 1024 * 1024 * 1024;
const DEFAULT_GATEWAY_FLIGHT_GRPC_WEB_ENABLED: bool = false;

pub(crate) fn gateway_listen_backlog(runtime_config: Option<GatewayRuntimeTomlConfig>) -> u32 {
    gateway_listen_backlog_with_lookup(runtime_config, &|key| std::env::var(key).ok())
}

pub(crate) fn gateway_listen_backlog_with_lookup(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u32 {
    runtime_config
        .and_then(|config| config.listen_backlog)
        .or_else(|| lookup_positive_parsed::<u32>(GATEWAY_LISTEN_BACKLOG_ENV, lookup))
        .unwrap_or(DEFAULT_GATEWAY_LISTEN_BACKLOG)
        .clamp(MIN_GATEWAY_LISTEN_BACKLOG, MAX_GATEWAY_LISTEN_BACKLOG)
}

pub(crate) fn gateway_studio_concurrency_limit(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
) -> usize {
    gateway_studio_concurrency_limit_with_lookup(
        runtime_config,
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .ok()
            .map(std::num::NonZeroUsize::get),
    )
}

pub(crate) fn gateway_studio_concurrency_limit_with_lookup(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    runtime_config
        .and_then(|config| config.studio_concurrency_limit)
        .or_else(|| lookup_positive_parsed::<usize>(GATEWAY_STUDIO_CONCURRENCY_LIMIT_ENV, lookup))
        .unwrap_or_else(|| default_gateway_studio_concurrency_limit(available_parallelism))
        .clamp(
            MIN_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
            MAX_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
        )
}

fn default_gateway_studio_concurrency_limit(available_parallelism: Option<usize>) -> usize {
    available_parallelism
        .unwrap_or(DEFAULT_GATEWAY_STUDIO_CONCURRENCY_FALLBACK)
        .saturating_mul(4)
        .clamp(
            MIN_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
            MAX_GATEWAY_STUDIO_CONCURRENCY_LIMIT,
        )
}

pub(crate) fn gateway_studio_request_timeout(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
) -> Duration {
    Duration::from_secs(gateway_studio_request_timeout_secs_with_lookup(
        runtime_config,
        &|key| std::env::var(key).ok(),
    ))
}

pub(crate) fn gateway_studio_request_timeout_secs_with_lookup(
    runtime_config: Option<GatewayRuntimeTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u64 {
    runtime_config
        .and_then(|config| config.studio_request_timeout_secs)
        .or_else(|| lookup_positive_parsed::<u64>(GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS_ENV, lookup))
        .unwrap_or(DEFAULT_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS)
        .clamp(
            MIN_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS,
            MAX_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS,
        )
}

pub(crate) fn gateway_flight_concurrency_limit(studio_concurrency_limit: usize) -> usize {
    gateway_flight_concurrency_limit_with_lookup(studio_concurrency_limit, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn gateway_flight_concurrency_limit_with_lookup(
    studio_concurrency_limit: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    lookup_positive_parsed::<usize>(GATEWAY_FLIGHT_CONCURRENCY_LIMIT_ENV, lookup)
        .unwrap_or(studio_concurrency_limit)
        .clamp(
            MIN_GATEWAY_FLIGHT_CONCURRENCY_LIMIT,
            MAX_GATEWAY_FLIGHT_CONCURRENCY_LIMIT,
        )
}

pub(crate) fn gateway_flight_request_timeout(studio_request_timeout: Duration) -> Duration {
    Duration::from_secs(gateway_flight_request_timeout_secs_with_lookup(
        studio_request_timeout.as_secs(),
        &|key| std::env::var(key).ok(),
    ))
}

pub(crate) fn gateway_flight_request_timeout_secs_with_lookup(
    studio_request_timeout_secs: u64,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u64 {
    lookup_positive_parsed::<u64>(GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS_ENV, lookup)
        .unwrap_or(studio_request_timeout_secs)
        .clamp(
            MIN_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS,
            MAX_GATEWAY_FLIGHT_REQUEST_TIMEOUT_SECS,
        )
}

pub(crate) fn gateway_https_rate_limit_per_second(studio_concurrency_limit: usize) -> u64 {
    gateway_https_rate_limit_per_second_with_lookup(studio_concurrency_limit, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn gateway_https_rate_limit_per_second_with_lookup(
    studio_concurrency_limit: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u64 {
    lookup_positive_parsed::<u64>(GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND_ENV, lookup)
        .unwrap_or_else(|| default_gateway_rate_limit_per_second(studio_concurrency_limit))
        .clamp(
            MIN_GATEWAY_RATE_LIMIT_PER_SECOND,
            MAX_GATEWAY_HTTPS_RATE_LIMIT_PER_SECOND,
        )
}

pub(crate) fn gateway_flight_rate_limit_per_second(flight_concurrency_limit: usize) -> u64 {
    gateway_flight_rate_limit_per_second_with_lookup(flight_concurrency_limit, &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn gateway_flight_rate_limit_per_second_with_lookup(
    flight_concurrency_limit: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> u64 {
    lookup_positive_parsed::<u64>(GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND_ENV, lookup)
        .unwrap_or_else(|| default_gateway_rate_limit_per_second(flight_concurrency_limit))
        .clamp(
            MIN_GATEWAY_RATE_LIMIT_PER_SECOND,
            MAX_GATEWAY_FLIGHT_RATE_LIMIT_PER_SECOND,
        )
}

fn default_gateway_rate_limit_per_second(concurrency_limit: usize) -> u64 {
    u64::try_from(concurrency_limit)
        .unwrap_or(u64::MAX)
        .saturating_mul(8)
        .max(MIN_GATEWAY_RATE_LIMIT_PER_SECOND)
}

pub(crate) fn gateway_https_stream_budget_bytes() -> usize {
    gateway_https_stream_budget_bytes_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_https_stream_budget_bytes_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    lookup_positive_parsed::<usize>(GATEWAY_HTTPS_STREAM_BUDGET_BYTES_ENV, lookup)
        .unwrap_or(DEFAULT_GATEWAY_HTTPS_STREAM_BUDGET_BYTES)
        .clamp(
            MIN_GATEWAY_STREAM_BUDGET_BYTES,
            MAX_GATEWAY_HTTPS_STREAM_BUDGET_BYTES,
        )
}

pub(crate) fn gateway_flight_stream_budget_bytes() -> usize {
    gateway_flight_stream_budget_bytes_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_flight_stream_budget_bytes_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    lookup_positive_parsed::<usize>(GATEWAY_FLIGHT_STREAM_BUDGET_BYTES_ENV, lookup)
        .unwrap_or(DEFAULT_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES)
        .clamp(
            MIN_GATEWAY_STREAM_BUDGET_BYTES,
            MAX_GATEWAY_FLIGHT_STREAM_BUDGET_BYTES,
        )
}

pub(crate) fn gateway_flight_grpc_web_enabled() -> bool {
    gateway_flight_grpc_web_enabled_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_flight_grpc_web_enabled_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    lookup_bool_flag(GATEWAY_FLIGHT_GRPC_WEB_ENABLED_ENV, lookup)
        .unwrap_or(DEFAULT_GATEWAY_FLIGHT_GRPC_WEB_ENABLED)
}
