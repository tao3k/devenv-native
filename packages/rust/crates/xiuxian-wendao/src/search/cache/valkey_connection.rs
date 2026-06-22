//! Shared Valkey connection cache for Wendao search cache traffic.

use redis::{AsyncConnectionConfig, Client, Connection, aio::MultiplexedConnection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Duration;
use tokio::sync::RwLock as AsyncRwLock;

type AsyncConnectionCache = Arc<AsyncRwLock<HashMap<String, MultiplexedConnection>>>;
type BlockingConnection = Arc<Mutex<Connection>>;
type BlockingConnectionCache = Arc<RwLock<HashMap<String, BlockingConnection>>>;

fn async_connection_cache() -> &'static AsyncConnectionCache {
    static CACHE: OnceLock<AsyncConnectionCache> = OnceLock::new();
    CACHE.get_or_init(|| Arc::new(AsyncRwLock::new(HashMap::new())))
}

fn blocking_connection_cache() -> &'static BlockingConnectionCache {
    static CACHE: OnceLock<BlockingConnectionCache> = OnceLock::new();
    CACHE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

fn to_cache_key(url: &str, connection_timeout: Duration, response_timeout: Duration) -> String {
    format!(
        "{url}|{}|{}",
        connection_timeout.as_nanos(),
        response_timeout.as_nanos()
    )
}

fn normalize_connection_url(url: &str) -> &str {
    url.trim().trim_end_matches('/')
}

pub(crate) async fn get_shared_multiplexed_connection(
    client: &Client,
    valkey_url: &str,
    connection_timeout: Duration,
    response_timeout: Duration,
) -> Option<MultiplexedConnection> {
    let url = normalize_connection_url(valkey_url);
    let cache_key = to_cache_key(url, connection_timeout, response_timeout);
    if let Some(connection) = async_connection_cache()
        .read()
        .await
        .get(&cache_key)
        .cloned()
    {
        return Some(connection);
    }

    let config = AsyncConnectionConfig::new()
        .set_connection_timeout(Some(connection_timeout))
        .set_response_timeout(Some(response_timeout));
    let connection = client
        .get_multiplexed_async_connection_with_config(&config)
        .await
        .ok()?;
    let mut cache = async_connection_cache().write().await;
    cache.insert(cache_key, connection.clone());
    Some(connection)
}

pub(crate) fn get_shared_blocking_connection(
    client: &Client,
    valkey_url: &str,
    connection_timeout: Duration,
    response_timeout: Duration,
) -> Option<Arc<Mutex<Connection>>> {
    let url = normalize_connection_url(valkey_url);
    let cache_key = to_cache_key(url, connection_timeout, response_timeout);
    if let Some(connection) = blocking_connection_cache()
        .read()
        .ok()?
        .get(&cache_key)
        .cloned()
    {
        return Some(connection);
    }

    let connection = client
        .get_connection_with_timeout(connection_timeout)
        .ok()?;
    let _ = connection.set_read_timeout(Some(response_timeout));
    let _ = connection.set_write_timeout(Some(response_timeout));
    let mut cache = blocking_connection_cache().write().ok()?;
    let shared = Arc::new(Mutex::new(connection));
    cache.insert(cache_key, shared.clone());
    Some(shared)
}
