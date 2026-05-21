//! Method-to-tool registry for native Zhenfa dispatch.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::ZhenfaTool;
use crate::ZhenfaToolIdRef;

/// In-memory native tool registry used by the zhenfa orchestrator.
#[derive(Clone, Default)]
pub struct ZhenfaRegistry {
    tools: HashMap<String, Arc<dyn ZhenfaTool>>,
}

impl ZhenfaRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one native tool by its `id()`.
    pub fn register(&mut self, tool: Arc<dyn ZhenfaTool>) -> Option<Arc<dyn ZhenfaTool>> {
        self.tools.insert(tool.id().to_string(), tool)
    }

    /// Resolve one tool by id.
    #[must_use]
    pub fn get(&self, tool_id: ZhenfaToolIdRef<'_>) -> Option<Arc<dyn ZhenfaTool>> {
        self.tools.get(tool_id).cloned()
    }

    /// Return true when no tools are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Number of registered tools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns true when the tool id is registered.
    #[must_use]
    pub fn contains(&self, tool_id: ZhenfaToolIdRef<'_>) -> bool {
        self.tools.contains_key(tool_id)
    }

    /// Snapshot current tool definitions indexed by id.
    #[must_use]
    pub fn definitions(&self) -> HashMap<String, Value> {
        self.tools
            .iter()
            .map(|(id, tool)| (id.clone(), tool.definition()))
            .collect()
    }
}
