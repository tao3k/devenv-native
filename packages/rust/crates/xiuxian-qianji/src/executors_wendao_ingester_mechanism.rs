use super::entity::{build_promotion_entity, build_promotion_relation, build_topic_entity};
use super::persistence::persist_promotion_graph;
use super::scope::resolve_graph_scope;
use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};

/// Promotes validated reflection context into a structured `Wendao` graph entity.
pub struct WendaoIngesterMechanism {
    /// Output key used for the emitted entity payload.
    pub output_key: String,
    /// Static graph scope fallback.
    pub graph_scope: Option<String>,
    /// Optional context key that provides graph scope dynamically.
    pub graph_scope_key: Option<String>,
    /// Vector dimension metadata used by `KnowledgeGraph::save_to_valkey`.
    pub graph_dimension: usize,
    /// Whether persistence should be attempted.
    pub persist: bool,
    /// Whether persistence failures should be recorded as output instead of failing the node.
    pub persist_best_effort: bool,
}

#[async_trait]
impl QianjiMechanism for WendaoIngesterMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let selected_route = context
            .get("selected_route")
            .and_then(Value::as_str)
            .unwrap_or("Promote");
        let decision = selected_route.to_lowercase();

        let graph_scope = resolve_graph_scope(
            context,
            self.graph_scope.as_ref(),
            self.graph_scope_key.as_ref(),
        );
        let entity = build_promotion_entity(context, &decision);
        let topic_entity = build_topic_entity(context);
        let relation = build_promotion_relation(&entity, &topic_entity, &decision);
        let mut persisted = false;
        let mut persist_error: Option<String> = None;

        if self.persist && decision == "promote" {
            match persist_promotion_graph(
                &graph_scope,
                &entity,
                &topic_entity,
                &relation,
                self.graph_dimension,
            ) {
                Ok(()) => persisted = true,
                Err(error) if self.persist_best_effort => {
                    persist_error = Some(error);
                    log::warn!(
                        "qianji wendao ingester best-effort persistence failed: {}",
                        persist_error.as_deref().unwrap_or("")
                    );
                }
                Err(error) => return Err(error),
            }
        }

        let mut data = Map::new();
        data.insert(
            "promotion_decision".to_string(),
            Value::String(decision.clone()),
        );
        data.insert(self.output_key.clone(), json!(entity));
        data.insert(
            "promotion_graph_scope".to_string(),
            Value::String(graph_scope.clone()),
        );
        data.insert("promotion_topic_entity".to_string(), json!(topic_entity));
        data.insert("promotion_relation".to_string(), json!(relation));
        data.insert("promotion_persisted".to_string(), Value::Bool(persisted));
        if let Some(error) = persist_error {
            data.insert("promotion_persist_error".to_string(), Value::String(error));
        }

        Ok(QianjiOutput {
            data: Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        2.0
    }
}
