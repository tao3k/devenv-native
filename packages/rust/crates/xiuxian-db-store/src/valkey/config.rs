//! Valkey connection and key namespace configuration.

use crate::valkey::error::{ValkeyStoreError, non_blank};

const DEFAULT_KEY_NAMESPACE: &str = "xiuxian:db-store";

/// Valkey key namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyKeyNamespace(String);

impl ValkeyKeyNamespace {
    /// Creates a non-empty namespace.
    ///
    /// # Errors
    ///
    /// Returns an error when the namespace is blank.
    pub fn new(value: impl Into<String>) -> Result<Self, ValkeyStoreError> {
        Ok(Self(non_blank(value.into(), "namespace")?))
    }

    /// Borrows the namespace.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for ValkeyKeyNamespace {
    fn default() -> Self {
        Self(DEFAULT_KEY_NAMESPACE.to_owned())
    }
}

/// Shared Valkey client configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyStoreConfig {
    redis_url: String,
    namespace: ValkeyKeyNamespace,
}

impl ValkeyStoreConfig {
    /// Creates a config from a Redis/Valkey URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is blank.
    pub fn new(redis_url: impl Into<String>) -> Result<Self, ValkeyStoreError> {
        Ok(Self {
            redis_url: non_blank(redis_url.into(), "redis_url")?,
            namespace: ValkeyKeyNamespace::default(),
        })
    }

    /// Sets a custom key namespace.
    ///
    /// # Errors
    ///
    /// Returns an error when the namespace is blank.
    pub fn with_namespace(
        mut self,
        namespace: impl Into<String>,
    ) -> Result<Self, ValkeyStoreError> {
        self.namespace = ValkeyKeyNamespace::new(namespace)?;
        Ok(self)
    }

    /// Borrows the configured URL.
    #[must_use]
    pub fn redis_url(&self) -> &str {
        self.redis_url.as_str()
    }

    /// Borrows the key namespace.
    #[must_use]
    pub const fn namespace(&self) -> &ValkeyKeyNamespace {
        &self.namespace
    }
}
