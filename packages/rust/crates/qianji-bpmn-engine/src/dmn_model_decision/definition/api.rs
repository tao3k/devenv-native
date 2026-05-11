//! Public dmn model decision definition contracts for BPMN/DMN engine integration.

use super::{
    Arc, DmnContextExpression, DmnDecisionRef, DmnDecisionTable,
    DmnInformationRequirementReference, DmnInvocation, DmnKnowledgeRequirementReference,
    DmnListExpression, DmnLiteralExpression, DmnRelationExpression,
};

/// One bounded DMN decision definition with one executable decision surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionDefinition {
    /// Source identifier used for diagnostics.
    pub source_id: Arc<str>,
    /// Stable decision reference.
    pub decision: DmnDecisionRef,
    /// Optional decision name.
    pub name: Option<Arc<str>>,
    /// The single bounded decision table.
    pub table: DmnDecisionTable,
    /// Optional bounded direct literal-expression decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub literal_expression: Option<DmnLiteralExpression>,
    /// Optional bounded direct list decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_expression: Option<DmnListExpression>,
    /// Optional bounded direct context decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_expression: Option<DmnContextExpression>,
    /// Optional bounded direct relation decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_expression: Option<DmnRelationExpression>,
    /// Optional bounded direct invocation decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<DmnInvocation>,
    /// Direct executable information-requirement href placeholders.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_requirements: Vec<DmnInformationRequirementReference>,
    /// Direct executable knowledge-requirement href placeholders.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_requirements: Vec<DmnKnowledgeRequirementReference>,
}

impl DmnDecisionDefinition {
    /// Creates one bounded decision definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        decision: DmnDecisionRef,
        name: Option<impl AsRef<str>>,
        table: DmnDecisionTable,
    ) -> Self {
        Self {
            source_id: (Arc::<str>::from(source_id.as_ref())).into(),
            decision,
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            table,
            literal_expression: None,
            list_expression: None,
            context_expression: None,
            relation_expression: None,
            invocation: None,
            information_requirements: Vec::new(),
            knowledge_requirements: Vec::new(),
        }
    }

    /// Attaches one bounded direct literal-expression decision body.
    #[must_use]
    pub fn with_literal_expression(mut self, literal_expression: DmnLiteralExpression) -> Self {
        self.literal_expression = Some(literal_expression);
        self
    }

    /// Attaches one bounded direct list decision body.
    #[must_use]
    pub fn with_list_expression(mut self, list_expression: DmnListExpression) -> Self {
        self.list_expression = Some(list_expression);
        self
    }

    /// Attaches one bounded direct context decision body.
    #[must_use]
    pub fn with_context_expression(mut self, context_expression: DmnContextExpression) -> Self {
        self.context_expression = Some(context_expression);
        self
    }

    /// Attaches one bounded direct relation decision body.
    #[must_use]
    pub fn with_relation_expression(mut self, relation_expression: DmnRelationExpression) -> Self {
        self.relation_expression = Some(relation_expression);
        self
    }

    /// Attaches one bounded direct invocation decision body.
    #[must_use]
    pub fn with_invocation(mut self, invocation: DmnInvocation) -> Self {
        self.invocation = Some(invocation);
        self
    }

    /// Attaches bounded direct information-requirement references.
    #[must_use]
    pub fn with_information_requirements(
        mut self,
        information_requirements: Vec<DmnInformationRequirementReference>,
    ) -> Self {
        self.information_requirements = information_requirements;
        self
    }

    /// Attaches bounded direct knowledge-requirement references.
    #[must_use]
    pub fn with_knowledge_requirements(
        mut self,
        knowledge_requirements: Vec<DmnKnowledgeRequirementReference>,
    ) -> Self {
        self.knowledge_requirements = knowledge_requirements;
        self
    }

    /// Returns whether the provided reference matches this parsed decision.
    #[must_use]
    pub fn matches_reference(&self, other: &DmnDecisionRef) -> bool {
        self.decision.decision_id == other.decision_id
            && other
                .source_id
                .as_deref()
                .is_none_or(|source_id| source_id == self.source_id.as_ref())
    }
}
