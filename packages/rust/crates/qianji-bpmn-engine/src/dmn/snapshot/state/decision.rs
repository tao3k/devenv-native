use crate::dmn::snapshot::xml::{attribute_value, required_attribute};
use crate::dmn_model_api::{
    DmnDecisionSnapshot, DmnFunctionDefinitionLiteralSnapshot,
    DmnFunctionDefinitionParameterSnapshot, DmnFunctionDefinitionSnapshot,
    DmnInvocationBindingSnapshot, DmnInvocationLiteralSnapshot, DmnInvocationParameterSnapshot,
    DmnInvocationSnapshot, DmnRequirementReferenceSnapshot, DmnSourceFile,
};
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
    invocations: Vec<TempInvocationSnapshot>,
    function_definitions: Vec<TempFunctionDefinitionSnapshot>,
    requirement_references: Vec<TempRequirementReferenceSnapshot>,
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
            invocations: value.invocations.into_iter().map(Into::into).collect(),
            function_definitions: value
                .function_definitions
                .into_iter()
                .map(Into::into)
                .collect(),
            requirement_references: value
                .requirement_references
                .into_iter()
                .map(Into::into)
                .collect(),
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
            invocations: Vec::new(),
            function_definitions: Vec::new(),
            requirement_references: Vec::new(),
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
            (Some("informationRequirement" | "authorityRequirement"), "requiredInput") => {
                self.required_input_count += 1;
            }
            (Some("informationRequirement" | "authorityRequirement"), "requiredDecision") => {
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

    pub(super) fn push_invocation(&mut self, invocation: TempInvocationSnapshot) {
        self.invocations.push(invocation);
    }

    pub(super) fn push_function_definition(
        &mut self,
        function_definition: TempFunctionDefinitionSnapshot,
    ) {
        self.function_definitions.push(function_definition);
    }

    pub(super) fn push_requirement_reference(
        &mut self,
        reference: TempRequirementReferenceSnapshot,
    ) {
        self.requirement_references.push(reference);
    }
}

#[derive(Debug)]
pub(super) struct TempRequirementReferenceSnapshot {
    requirement_kind: String,
    reference_kind: String,
    href: Option<String>,
}

impl From<TempRequirementReferenceSnapshot> for DmnRequirementReferenceSnapshot {
    fn from(value: TempRequirementReferenceSnapshot) -> Self {
        Self {
            requirement_kind: value.requirement_kind,
            reference_kind: value.reference_kind,
            href: value.href,
        }
    }
}

impl TempRequirementReferenceSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        requirement_kind: &str,
        reference_kind: &str,
    ) -> Result<Self> {
        Ok(Self {
            requirement_kind: requirement_kind.to_string(),
            reference_kind: reference_kind.to_string(),
            href: attribute_value(source, reader, event, "href")?,
        })
    }
}

#[derive(Debug)]
pub(super) struct TempFunctionDefinitionSnapshot {
    function_definition_id: Option<String>,
    kind: Option<String>,
    parameters: Vec<TempFunctionDefinitionParameterSnapshot>,
    body: Option<TempFunctionDefinitionLiteralSnapshot>,
}

impl From<TempFunctionDefinitionSnapshot> for DmnFunctionDefinitionSnapshot {
    fn from(value: TempFunctionDefinitionSnapshot) -> Self {
        Self {
            function_definition_id: value.function_definition_id,
            kind: value.kind,
            parameters: value.parameters.into_iter().map(Into::into).collect(),
            body: value.body.map(Into::into),
        }
    }
}

impl TempFunctionDefinitionSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            function_definition_id: attribute_value(source, reader, event, "id")?,
            kind: attribute_value(source, reader, event, "kind")?,
            parameters: Vec::new(),
            body: None,
        })
    }

    pub(super) fn push_parameter(&mut self, parameter: TempFunctionDefinitionParameterSnapshot) {
        self.parameters.push(parameter);
    }

    pub(super) fn set_body(&mut self, body: TempFunctionDefinitionLiteralSnapshot) {
        self.body = Some(body);
    }
}

#[derive(Debug)]
pub(super) struct TempFunctionDefinitionParameterSnapshot {
    parameter_id: Option<String>,
    name: Option<String>,
    type_ref: Option<String>,
}

impl From<TempFunctionDefinitionParameterSnapshot> for DmnFunctionDefinitionParameterSnapshot {
    fn from(value: TempFunctionDefinitionParameterSnapshot) -> Self {
        Self {
            parameter_id: value.parameter_id,
            name: value.name,
            type_ref: value.type_ref,
        }
    }
}

impl TempFunctionDefinitionParameterSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            parameter_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
        })
    }
}

#[derive(Debug)]
pub(super) struct TempFunctionDefinitionLiteralSnapshot {
    expression_id: Option<String>,
    type_ref: Option<String>,
    text: String,
}

impl From<TempFunctionDefinitionLiteralSnapshot> for DmnFunctionDefinitionLiteralSnapshot {
    fn from(value: TempFunctionDefinitionLiteralSnapshot) -> Self {
        let text = non_empty_text(&value.text);
        Self {
            expression_id: value.expression_id,
            type_ref: value.type_ref,
            text,
        }
    }
}

impl TempFunctionDefinitionLiteralSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            expression_id: attribute_value(source, reader, event, "id")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
            text: String::new(),
        })
    }

    pub(super) fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }
}

#[derive(Debug)]
pub(super) struct TempInvocationSnapshot {
    invocation_id: Option<String>,
    invoked_expression: Option<TempInvocationLiteralSnapshot>,
    bindings: Vec<TempInvocationBindingSnapshot>,
}

impl From<TempInvocationSnapshot> for DmnInvocationSnapshot {
    fn from(value: TempInvocationSnapshot) -> Self {
        Self {
            invocation_id: value.invocation_id,
            invoked_expression: value.invoked_expression.map(Into::into),
            bindings: value.bindings.into_iter().map(Into::into).collect(),
        }
    }
}

impl TempInvocationSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            invocation_id: attribute_value(source, reader, event, "id")?,
            invoked_expression: None,
            bindings: Vec::new(),
        })
    }

    pub(super) fn set_invoked_expression(&mut self, expression: TempInvocationLiteralSnapshot) {
        self.invoked_expression = Some(expression);
    }

    pub(super) fn push_binding(&mut self, binding: TempInvocationBindingSnapshot) {
        self.bindings.push(binding);
    }
}

#[derive(Debug)]
pub(super) struct TempInvocationBindingSnapshot {
    binding_id: Option<String>,
    parameter: Option<TempInvocationParameterSnapshot>,
    argument: Option<TempInvocationLiteralSnapshot>,
}

impl From<TempInvocationBindingSnapshot> for DmnInvocationBindingSnapshot {
    fn from(value: TempInvocationBindingSnapshot) -> Self {
        Self {
            binding_id: value.binding_id,
            parameter: value.parameter.map(Into::into),
            argument: value.argument.map(Into::into),
        }
    }
}

impl TempInvocationBindingSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            binding_id: attribute_value(source, reader, event, "id")?,
            parameter: None,
            argument: None,
        })
    }

    pub(super) fn set_parameter(&mut self, parameter: TempInvocationParameterSnapshot) {
        self.parameter = Some(parameter);
    }

    pub(super) fn set_argument(&mut self, argument: TempInvocationLiteralSnapshot) {
        self.argument = Some(argument);
    }
}

#[derive(Debug)]
pub(super) struct TempInvocationParameterSnapshot {
    parameter_id: Option<String>,
    name: Option<String>,
    type_ref: Option<String>,
}

impl From<TempInvocationParameterSnapshot> for DmnInvocationParameterSnapshot {
    fn from(value: TempInvocationParameterSnapshot) -> Self {
        Self {
            parameter_id: value.parameter_id,
            name: value.name,
            type_ref: value.type_ref,
        }
    }
}

impl TempInvocationParameterSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            parameter_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
        })
    }
}

#[derive(Debug)]
pub(super) struct TempInvocationLiteralSnapshot {
    expression_id: Option<String>,
    type_ref: Option<String>,
    text: String,
}

impl From<TempInvocationLiteralSnapshot> for DmnInvocationLiteralSnapshot {
    fn from(value: TempInvocationLiteralSnapshot) -> Self {
        let text = non_empty_text(&value.text);
        Self {
            expression_id: value.expression_id,
            type_ref: value.type_ref,
            text,
        }
    }
}

impl TempInvocationLiteralSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            expression_id: attribute_value(source, reader, event, "id")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
            text: String::new(),
        })
    }

    pub(super) fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }
}

fn non_empty_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
