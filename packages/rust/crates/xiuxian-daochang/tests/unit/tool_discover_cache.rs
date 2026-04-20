//! External tool discover read-through cache integration tests.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::unit::live_gates::resolve_enabled_live_valkey_url;
use crate::unit::tool_runtime_mock::{
    MockCallToolReply, MockToolRuntimeConfig, call_handler, reserve_local_addr,
    spawn_mock_tool_runtime, text_result, tool_definition,
};
use anyhow::Result;
use xiuxian_daochang::{ToolPoolConnectConfig, connect_tool_pool};

async fn spawn_mock_server_with_discover_counter(
    addr: std::net::SocketAddr,
) -> (tokio::task::JoinHandle<()>, Arc<AtomicUsize>) {
    let discover_calls_total = Arc::new(AtomicUsize::new(0));
    let server = spawn_mock_tool_runtime(
        addr,
        MockToolRuntimeConfig::with_static_tools(
            vec![tool_definition(
                "skill.discover",
                "Mock discover tool for cache verification",
                &serde_json::json!({
                    "type": "object",
                    "properties": {
                        "intent": { "type": "string" },
                        "limit": { "type": "integer" }
                    },
                    "required": ["intent"]
                }),
            )],
            call_handler({
                let discover_calls_total = Arc::clone(&discover_calls_total);
                move |request| {
                    let discover_calls_total = Arc::clone(&discover_calls_total);
                    async move {
                        if request.name != "skill.discover" {
                            return MockCallToolReply::RpcError {
                                code: -32_603,
                                message: "unsupported tool in discover cache test".to_string(),
                                data: None,
                            };
                        }
                        discover_calls_total.fetch_add(1, Ordering::SeqCst);
                        let intent = request
                            .arguments
                            .as_ref()
                            .and_then(|value| value.get("intent"))
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default();
                        MockCallToolReply::Result(text_result(format!("discover:{intent}")))
                    }
                }
            }),
        ),
    )
    .await;
    (server, discover_calls_total)
}

fn reconnect_test_config() -> ToolPoolConnectConfig {
    ToolPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 1,
        connect_retries: 6,
        connect_retry_backoff_ms: 100,
        tool_timeout_secs: 10,
        list_tools_cache_ttl_ms: 1_000,
    }
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn p95(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = sorted.len().saturating_mul(95).div_ceil(100);
    let index = rank.saturating_sub(1).min(sorted.len().saturating_sub(1));
    sorted[index]
}

#[tokio::test]
async fn discover_calls_use_valkey_read_through_cache_when_configured() -> Result<()> {
    if resolve_enabled_live_valkey_url("live cache test").is_none() {
        return Ok(());
    }

    let addr = reserve_local_addr().await;
    let (handle, discover_calls_total) = spawn_mock_server_with_discover_counter(addr).await;
    let url = format!("http://{addr}/sse");
    let pool = connect_tool_pool(&url, reconnect_test_config()).await;
    let pool = match pool {
        Ok(pool) => pool,
        Err(error) => panic!("connect pool: {error}"),
    };

    let Some(initial_stats) = pool.discover_cache_stats_snapshot() else {
        handle.abort();
        let _ = handle.await;
        eprintln!("skip: discover cache disabled in runtime settings");
        return Ok(());
    };
    assert_eq!(initial_stats.requests_total, 0);

    let iterations = env_usize("XIUXIAN_DAOCHANG_DISCOVER_CACHE_BENCH_ITERATIONS", 12);
    let hit_p95_slo_ms = env_f64("XIUXIAN_DAOCHANG_DISCOVER_CACHE_HIT_P95_MS", 15.0);
    let miss_p95_slo_ms = env_f64("XIUXIAN_DAOCHANG_DISCOVER_CACHE_MISS_P95_MS", 80.0);
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros();

    let mut miss_latencies_ms = Vec::with_capacity(iterations);
    let mut hit_latencies_ms = Vec::with_capacity(iterations);
    for iteration in 0..iterations {
        let intent = format!("discover-cache-canonicalization-{suffix}-{iteration}");
        let args_miss = serde_json::json!({
            "intent": intent,
            "limit": 5
        });
        let miss_started = Instant::now();
        let first = pool
            .call_tool("skill.discover".to_string(), Some(args_miss))
            .await;
        let first = match first {
            Ok(first) => first,
            Err(error) => panic!("first discover call: {error}"),
        };
        miss_latencies_ms.push(miss_started.elapsed().as_secs_f64() * 1000.0);
        assert!(!first.is_error);

        let args_hit = serde_json::json!({
            "limit": 5,
            "intent": intent
        });
        let hit_started = Instant::now();
        let second = pool
            .call_tool("skill.discover".to_string(), Some(args_hit))
            .await;
        let second = match second {
            Ok(second) => second,
            Err(error) => panic!("second discover call: {error}"),
        };
        hit_latencies_ms.push(hit_started.elapsed().as_secs_f64() * 1000.0);
        assert!(!second.is_error);
    }
    assert_eq!(
        discover_calls_total.load(Ordering::SeqCst),
        iterations,
        "discover backend should only be hit on cache miss requests"
    );

    let miss_p95 = p95(&miss_latencies_ms);
    let hit_p95 = p95(&hit_latencies_ms);
    assert!(
        miss_p95 <= miss_p95_slo_ms,
        "discover cache miss p95 exceeded SLO: miss_p95={miss_p95:.2}ms > miss_p95_slo_ms={miss_p95_slo_ms:.2}ms"
    );
    assert!(
        hit_p95 <= hit_p95_slo_ms,
        "discover cache hit p95 exceeded SLO: hit_p95={hit_p95:.2}ms > hit_p95_slo_ms={hit_p95_slo_ms:.2}ms"
    );

    let iterations_u64 = iterations as u64;
    let stats = pool.discover_cache_stats_snapshot();
    let Some(stats) = stats else {
        panic!("discover cache stats snapshot");
    };
    assert_eq!(stats.requests_total, iterations_u64 * 2);
    assert_eq!(stats.cache_hits, iterations_u64);
    assert_eq!(stats.cache_misses, iterations_u64);
    assert_eq!(stats.cache_writes, iterations_u64);
    assert!(
        (stats.hit_rate_pct - 50.0).abs() <= 1e-6,
        "expected hit rate 50.0, got {}",
        stats.hit_rate_pct
    );

    handle.abort();
    let _ = handle.await;
    Ok(())
}
