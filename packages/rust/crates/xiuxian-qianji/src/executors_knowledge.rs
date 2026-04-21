//! Wendao knowledge retrieval mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_wendao::LinkGraphIndex;
use xiuxian_wendao_core::LinkGraphSearchOptions;

/// Mechanism responsible for performing topological graph search.
pub struct KnowledgeSeeker {
    /// Reference to the Wendao link-graph index.
    pub index: Arc<LinkGraphIndex>,
}

#[async_trait]
impl QianjiMechanism for KnowledgeSeeker {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let query = context
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'query' in context")?;

        let (_, hits) = self
            .index
            .search_planned(query, 5, LinkGraphSearchOptions::default());

        Ok(QianjiOutput {
            data: json!({ "raw_facts": hits }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        5.0
    }
}
