//! Type-indexed execution context for native tool dispatch.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use super::signal::ZhenfaSignal;
use crate::{JsonRpcMeta, ZhenfaSessionId, ZhenfaTraceId};

type ExtensionValue = Arc<dyn Any + Send + Sync>;
type ExtensionMap = HashMap<TypeId, ExtensionValue>;

/// Runtime context propagated to native zhenfa tools.
#[derive(Clone, Default)]
pub struct ZhenfaContext {
    /// Optional session identifier propagated from caller runtime.
    pub session_id: Option<ZhenfaSessionId>,
    /// Optional trace identifier for correlation.
    pub trace_id: Option<ZhenfaTraceId>,
    /// Additional metadata fields propagated by the caller.
    pub extra: HashMap<String, Value>,
    extensions: Arc<ExtensionMap>,
}

impl ZhenfaContext {
    /// Build a context from explicit metadata fields.
    #[must_use]
    pub fn new(
        session_id: Option<ZhenfaSessionId>,
        trace_id: Option<ZhenfaTraceId>,
        extra: HashMap<String, Value>,
    ) -> Self {
        Self {
            session_id,
            trace_id,
            extra,
            extensions: Arc::default(),
        }
    }

    /// Build a context from optional JSON-RPC metadata.
    #[must_use]
    pub fn from_meta(meta: Option<JsonRpcMeta>) -> Self {
        meta.map_or_else(Self::default, Self::from)
    }

    /// Set the trace identifier when absent.
    pub fn set_correlation_id_if_absent(&mut self, correlation_id: impl Into<ZhenfaTraceId>) {
        let should_set = self
            .trace_id
            .as_ref()
            .is_none_or(|value| value.as_str().trim().is_empty());
        if should_set {
            self.trace_id = Some(correlation_id.into());
        }
    }

    /// Attach a signal sender for fire-and-forget signal emission.
    pub fn attach_signal_sender(&mut self, sender: UnboundedSender<ZhenfaSignal>) {
        let _ = self.insert_extension(sender);
    }

    /// Insert one owned extension value.
    ///
    /// Returns the previous extension for the same type when present.
    pub fn insert_extension<T>(&mut self, value: T) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.insert_shared_extension(Arc::new(value))
    }

    /// Insert one shared extension value.
    ///
    /// Returns the previous extension for the same type when present.
    pub fn insert_shared_extension<T>(&mut self, value: Arc<T>) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let previous = Arc::make_mut(&mut self.extensions).insert(TypeId::of::<T>(), value);
        previous.and_then(|erased| Arc::downcast::<T>(erased).ok())
    }

    /// Fetch one typed extension value.
    #[must_use]
    pub fn get_extension<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let value = self.extensions.get(&TypeId::of::<T>())?.clone();
        Arc::downcast::<T>(value).ok()
    }

    /// Returns true when one typed extension is registered.
    #[must_use]
    pub fn has_extension<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.extensions.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of registered extension types.
    #[must_use]
    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }
}

impl From<JsonRpcMeta> for ZhenfaContext {
    fn from(meta: JsonRpcMeta) -> Self {
        Self::new(
            meta.session_id.map(ZhenfaSessionId::from),
            meta.trace_id.map(ZhenfaTraceId::from),
            meta.extra,
        )
    }
}

impl std::fmt::Debug for ZhenfaContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ZhenfaContext")
            .field("session_id", &self.session_id)
            .field("trace_id", &self.trace_id)
            .field("extra", &self.extra)
            .field("extensions", &self.extension_count())
            .finish()
    }
}

impl PartialEq for ZhenfaContext {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.trace_id == other.trace_id
            && self.extra == other.extra
    }
}
