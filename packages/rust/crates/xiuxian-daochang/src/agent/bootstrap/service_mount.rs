//! Service-mount policy types for agent bootstrap configuration.

use serde::{Deserialize, Serialize};

/// Service mount lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMountStatus {
    /// Service mounted successfully and is active.
    Mounted,
    /// Service mount was intentionally skipped.
    Skipped,
    /// Service mount attempted and failed.
    Failed,
}

impl ServiceMountStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Mounted => "mounted",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

/// Standardized mount metadata for service wiring.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMountMeta {
    pub endpoint: Option<String>,
    pub storage: Option<String>,
    pub detail: Option<String>,
}

impl ServiceMountMeta {
    /// Add details metadata.
    #[must_use]
    pub fn detail(mut self, value: impl Into<String>) -> Self {
        self.detail = Some(value.into());
        self
    }
}

/// Service mount category used by runtime diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServiceMountCategory(String);

impl ServiceMountCategory {
    /// Build a service mount category.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the category string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ServiceMountCategory {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ServiceMountCategory {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Durable mount record exposed for runtime diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMountRecord {
    /// Service name that was mounted, skipped, or failed.
    pub service: String,
    /// Service category used for runtime diagnostics.
    pub category: ServiceMountCategory,
    /// Final mount status.
    pub status: ServiceMountStatus,
    /// Optional endpoint associated with the service.
    pub endpoint: Option<String>,
    /// Optional storage backend associated with the service.
    pub storage: Option<String>,
    /// Optional human-readable detail for diagnostics.
    pub detail: Option<String>,
}

/// In-memory catalog used during bootstrap and emitted as standardized logs.
#[derive(Debug, Default)]
pub(crate) struct ServiceMountCatalog {
    records: Vec<ServiceMountRecord>,
}

impl ServiceMountCatalog {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn mounted(
        &mut self,
        service: impl Into<String>,
        category: impl Into<ServiceMountCategory>,
        meta: ServiceMountMeta,
    ) {
        self.record(service, category, ServiceMountStatus::Mounted, meta);
    }

    pub(crate) fn skipped(
        &mut self,
        service: impl Into<String>,
        category: impl Into<ServiceMountCategory>,
        meta: ServiceMountMeta,
    ) {
        self.record(service, category, ServiceMountStatus::Skipped, meta);
    }

    pub(crate) fn failed(
        &mut self,
        service: impl Into<String>,
        category: impl Into<ServiceMountCategory>,
        meta: ServiceMountMeta,
    ) {
        self.record(service, category, ServiceMountStatus::Failed, meta);
    }

    fn record(
        &mut self,
        service: impl Into<String>,
        category: impl Into<ServiceMountCategory>,
        status: ServiceMountStatus,
        meta: ServiceMountMeta,
    ) {
        let record = ServiceMountRecord {
            service: service.into(),
            category: category.into(),
            status,
            endpoint: meta.endpoint,
            storage: meta.storage,
            detail: meta.detail,
        };
        tracing::info!(
            event = "agent.service.mount",
            service = %record.service,
            category = %record.category.as_str(),
            status = record.status.as_str(),
            endpoint = %record.endpoint.as_deref().unwrap_or(""),
            storage = %record.storage.as_deref().unwrap_or(""),
            detail = %record.detail.as_deref().unwrap_or(""),
            "service mount recorded"
        );
        self.records.push(record);
    }

    #[cfg(test)]
    pub(crate) fn finish(self) -> Vec<ServiceMountRecord> {
        self.records
    }
}
