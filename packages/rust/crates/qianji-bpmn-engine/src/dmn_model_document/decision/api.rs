use super::{DmnFunctionDefinitionSnapshot, DmnInvocationSnapshot};

/// Snapshot of one direct decision-owned requirement reference placeholder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRequirementReferenceSnapshot {
    /// Parent requirement element kind, such as `informationRequirement`.
    pub requirement_kind: String,
    /// Direct reference element kind, such as `requiredInput`.
    pub reference_kind: String,
    /// Optional direct `href` payload preserved from the reference element.
    pub href: Option<String>,
}

/// Snapshot of one DMN decision header.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionSnapshot {
    /// Stable DMN decision identifier.
    pub decision_id: String,
    /// Optional human-readable decision name.
    pub name: Option<String>,
    /// Number of direct `allowedAnswers` children discovered for the decision.
    pub allowed_answers_count: usize,
    /// Number of direct `decisionMaker` children discovered for the decision.
    pub decision_maker_count: usize,
    /// Number of direct `decisionOwner` children discovered for the decision.
    pub decision_owner_count: usize,
    /// Number of nested `decisionTable` elements discovered for the decision.
    pub decision_table_count: usize,
    /// Number of direct `informationRequirement` children discovered for the decision.
    pub information_requirement_count: usize,
    /// Number of nested `requiredInput` children discovered under information requirements.
    pub required_input_count: usize,
    /// Number of nested `requiredDecision` children discovered under information requirements.
    pub required_decision_count: usize,
    /// Number of direct `knowledgeRequirement` children discovered for the decision.
    pub knowledge_requirement_count: usize,
    /// Number of nested `requiredKnowledge` children discovered under knowledge requirements.
    pub required_knowledge_count: usize,
    /// Number of direct `authorityRequirement` children discovered for the decision.
    pub authority_requirement_count: usize,
    /// Number of nested `requiredAuthority` children discovered under authority requirements.
    pub required_authority_count: usize,
    /// Number of direct `literalExpression` children discovered for the decision.
    pub literal_expression_count: usize,
    /// Number of direct `context` children discovered for the decision.
    pub context_count: usize,
    /// Number of direct `invocation` children discovered for the decision.
    pub invocation_count: usize,
    /// Number of direct `relation` children discovered for the decision.
    pub relation_count: usize,
    /// Number of direct `functionDefinition` children discovered for the decision.
    pub function_definition_count: usize,
    /// Number of direct `list` children discovered for the decision.
    pub list_count: usize,
    /// Direct invocation placeholders preserved for lint and adapter evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invocations: Vec<DmnInvocationSnapshot>,
    /// Direct function-definition placeholders preserved for lint and adapter evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub function_definitions: Vec<DmnFunctionDefinitionSnapshot>,
    /// Direct requirement target placeholders preserved for lint and adapter evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_references: Vec<DmnRequirementReferenceSnapshot>,
}
