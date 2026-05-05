//! Public dmn model business knowledge contracts for BPMN/DMN engine integration.

use crate::dmn_model_document::{
    DmnBusinessKnowledgeModelLiteralSnapshot, DmnBusinessKnowledgeModelSnapshot,
    DmnFunctionDefinitionSnapshot,
};
use std::sync::Arc;

/// One bounded executable DMN business-knowledge-model contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnBusinessKnowledgeModelDefinition {
    /// Source identifier used for diagnostics.
    pub source_id: Arc<str>,
    /// Stable top-level `businessKnowledgeModel` identifier when present.
    pub business_knowledge_model_id: Option<Arc<str>>,
    /// Human-readable top-level `businessKnowledgeModel` name when present.
    pub name: Option<Arc<str>>,
    /// Direct nested DMN invocable `variable` name when present.
    pub variable_name: Option<Arc<str>>,
    /// Direct nested DMN invocable `variable` `typeRef` when present.
    pub variable_type_ref: Option<Arc<str>>,
    /// Preserved bounded direct `encapsulatedLogic` placeholder when present.
    pub encapsulated_logic: Option<DmnFunctionDefinitionSnapshot>,
    /// Preserved bounded direct body placeholder when present.
    pub body: Option<DmnBusinessKnowledgeModelLiteralSnapshot>,
}

impl DmnBusinessKnowledgeModelDefinition {
    /// Creates one bounded executable business-knowledge-model definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        business_knowledge_model_id: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            source_id: Arc::<str>::from(source_id.as_ref()),
            business_knowledge_model_id: business_knowledge_model_id
                .map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            variable_name: None,
            variable_type_ref: None,
            encapsulated_logic: None,
            body: None,
        }
    }

    /// Attaches one bounded direct invocable variable contract.
    #[must_use]
    pub fn with_variable(
        mut self,
        variable_name: Option<impl AsRef<str>>,
        variable_type_ref: Option<impl AsRef<str>>,
    ) -> Self {
        self.variable_name = variable_name.map(|value| Arc::<str>::from(value.as_ref()));
        self.variable_type_ref = variable_type_ref.map(|value| Arc::<str>::from(value.as_ref()));
        self
    }

    /// Attaches one bounded direct `encapsulatedLogic` placeholder.
    #[must_use]
    pub fn with_encapsulated_logic(
        mut self,
        encapsulated_logic: Option<DmnFunctionDefinitionSnapshot>,
    ) -> Self {
        self.encapsulated_logic = encapsulated_logic;
        self
    }

    /// Attaches one bounded direct BKM body placeholder.
    #[must_use]
    pub fn with_body(mut self, body: Option<DmnBusinessKnowledgeModelLiteralSnapshot>) -> Self {
        self.body = body;
        self
    }

    /// Builds one executable business-knowledge-model definition from one
    /// bounded snapshot entry.
    #[must_use]
    pub fn from_snapshot(
        source_id: impl AsRef<str>,
        snapshot: &DmnBusinessKnowledgeModelSnapshot,
    ) -> Self {
        Self::new(
            source_id,
            snapshot.business_knowledge_model_id.as_deref(),
            snapshot.name.as_deref(),
        )
        .with_variable(
            snapshot
                .variable
                .as_ref()
                .and_then(|variable| variable.name.as_deref()),
            snapshot
                .variable
                .as_ref()
                .and_then(|variable| variable.type_ref.as_deref()),
        )
        .with_encapsulated_logic(snapshot.encapsulated_logic.clone())
        .with_body(snapshot.body.clone())
    }
}
