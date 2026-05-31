//! Shared multiplexed Valkey client execution and reconnect handling.

use std::sync::Arc;

use redis::FromRedisValue;
use tokio::sync::{Mutex, RwLock};

use crate::valkey::{
    ValkeyStoreConfig,
    error::{ValkeyStoreError, validate_positive_ttl},
};

/// Shared multiplexed Valkey client.
#[derive(Clone)]
pub struct ValkeyClient {
    config: ValkeyStoreConfig,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl ValkeyClient {
    /// Creates a client.
    #[must_use]
    pub fn new(config: ValkeyStoreConfig) -> Self {
        Self {
            config,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Borrows the config.
    #[must_use]
    pub const fn config(&self) -> &ValkeyStoreConfig {
        &self.config
    }

    pub(crate) async fn run_command<T, F>(
        &self,
        operation: &'static str,
        build: F,
    ) -> Result<T, ValkeyStoreError>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_error: Option<redis::RedisError> = None;
        for _ in 0..2 {
            let mut connection = self.acquire_connection().await?;
            let command = build();
            let result: redis::RedisResult<T> = command.query_async(&mut connection).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    self.invalidate_connection().await;
                    last_error = Some(error);
                }
            }
        }
        Err(ValkeyStoreError::Storage {
            operation,
            message: last_error.map_or_else(
                || "Valkey command failed unexpectedly".to_owned(),
                |error| error.to_string(),
            ),
        })
    }

    async fn acquire_connection(
        &self,
    ) -> Result<redis::aio::MultiplexedConnection, ValkeyStoreError> {
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let _guard = self.reconnect_lock.lock().await;
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let client = redis::Client::open(self.config.redis_url()).map_err(|error| {
            ValkeyStoreError::Storage {
                operation: "open_valkey_client",
                message: error.to_string(),
            }
        })?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| ValkeyStoreError::Storage {
                operation: "connect_valkey",
                message: error.to_string(),
            })?;
        {
            let mut guard = self.connection.write().await;
            *guard = Some(connection.clone());
        }
        Ok(connection)
    }

    async fn invalidate_connection(&self) {
        let mut guard = self.connection.write().await;
        *guard = None;
    }

    /// Stores a string with a millisecond TTL.
    ///
    /// # Errors
    ///
    /// Returns an error when `ttl_ms` is zero or the Valkey command fails.
    pub async fn set_string_with_ttl(
        &self,
        key: &str,
        value: &str,
        ttl_ms: u64,
    ) -> Result<(), ValkeyStoreError> {
        validate_positive_ttl("ttl_ms", ttl_ms)?;
        let _: String = self
            .run_command("valkey_set_string_with_ttl", || {
                let mut command = redis::cmd("SET");
                command.arg(key).arg(value).arg("PX").arg(ttl_ms);
                command
            })
            .await?;
        Ok(())
    }

    /// Loads a string value.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn get_string(&self, key: &str) -> Result<Option<String>, ValkeyStoreError> {
        self.run_command("valkey_get_string", || {
            let mut command = redis::cmd("GET");
            command.arg(key);
            command
        })
        .await
    }

    /// Lists keys matching a pattern.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>, ValkeyStoreError> {
        self.run_command("valkey_keys", || {
            let mut command = redis::cmd("KEYS");
            command.arg(pattern);
            command
        })
        .await
    }
}
