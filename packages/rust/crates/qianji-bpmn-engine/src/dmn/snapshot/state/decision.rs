use crate::dmn::snapshot::xml::{attribute_value, required_attribute};
use crate::dmn_model_api::{DmnDecisionSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempDecisionSnapshot {
    decision_id: String,
    name: Option<String>,
    allowed_answers_count: usize,
    decision_maker_count: usize,
    decision_owner_count: usize,
    decision_table_count: usize,
    information_requirement_count: usize,
    required_input_count: usize,
    required_decision_count: usize,
    knowledge_requirement_count: usize,
    required_knowledge_count: usize,
    authority_requirement_count: usize,
    required_authority_count: usize,
    literal_expression_count: usize,
    context_count: usize,
    invocation_count: usize,
    relation_count: usize,
    function_definition_count: usize,
    list_count: usize,
}

impl From<TempDecisionSnapshot> for DmnDecisionSnapshot {
    fn from(value: TempDecisionSnapshot) -> Self {
        Self {
            decision_id: value.decision_id,
            name: value.name,
            allowed_answers_count: value.allowed_answers_count,
            decision_maker_count: value.decision_maker_count,
            decision_owner_count: value.decision_owner_count,
            decision_table_count: value.decision_table_count,
            information_requirement_count: value.information_requirement_count,
            required_input_count: value.required_input_count,
            required_decision_count: value.required_decision_count,
            knowledge_requirement_count: value.knowledge_requirement_count,
            required_knowledge_count: value.required_knowledge_count,
            authority_requirement_count: value.authority_requirement_count,
            required_authority_count: value.required_authority_count,
            literal_expression_count: value.literal_expression_count,
            context_count: value.context_count,
            invocation_count: value.invocation_count,
            relation_count: value.relation_count,
            function_definition_count: value.function_definition_count,
            list_count: value.list_count,
        }
    }
}

impl TempDecisionSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            decision_id: required_attribute(source, reader, event, "decision", "id")?,
            name: attribute_value(source, reader, event, "name")?,
            allowed_answers_count: 0,
            decision_maker_count: 0,
            decision_owner_count: 0,
            decision_table_count: 0,
            information_requirement_count: 0,
            required_input_count: 0,
            required_decision_count: 0,
            knowledge_requirement_count: 0,
            required_knowledge_count: 0,
            authority_requirement_count: 0,
            required_authority_count: 0,
            literal_expression_count: 0,
            context_count: 0,
            invocation_count: 0,
            relation_count: 0,
            function_definition_count: 0,
            list_count: 0,
        })
    }

    pub(super) fn track_construct(&mut self, tag: &str, parent_tag: Option<&str>) {
        match (parent_tag, tag) {
            (Some("decision"), "allowedAnswers") => self.allowed_answers_count += 1,
            (Some("decision"), "decisionMaker") => self.decision_maker_count += 1,
            (Some("decision"), "decisionOwner") => self.decision_owner_count += 1,
            (Some("decision"), "decisionTable") => self.decision_table_count += 1,
            (Some("decision"), "informationRequirement") => {
                self.information_requirement_count += 1;
            }
            (Some("informationRequirement"), "requiredInput") => {
                self.required_input_count += 1;
            }
            (Some("informationRequirement"), "requiredDecision") => {
                self.required_decision_count += 1;
            }
            (Some("decision"), "knowledgeRequirement") => {
                self.knowledge_requirement_count += 1;
            }
            (Some("knowledgeRequirement"), "requiredKnowledge") => {
                self.required_knowledge_count += 1;
            }
            (Some("decision"), "authorityRequirement") => {
                self.authority_requirement_count += 1;
            }
            (Some("authorityRequirement"), "requiredAuthority") => {
                self.required_authority_count += 1;
            }
            (Some("decision"), "literalExpression") => self.literal_expression_count += 1,
            (Some("decision"), "context") => self.context_count += 1,
            (Some("decision"), "invocation") => self.invocation_count += 1,
            (Some("decision"), "relation") => self.relation_count += 1,
            (Some("decision"), "functionDefinition") => {
                self.function_definition_count += 1;
            }
            (Some("decision"), "list") => self.list_count += 1,
            _ => {}
        }
    }
}
