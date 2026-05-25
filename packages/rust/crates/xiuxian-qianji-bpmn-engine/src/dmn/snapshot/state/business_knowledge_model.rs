use super::decision::TempFunctionDefinitionSnapshot;
use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{
    DmnBusinessKnowledgeModelLiteralSnapshot, DmnBusinessKnowledgeModelSnapshot, DmnSourceFile,
    DmnVariableSnapshot,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempBusinessKnowledgeModelSnapshot {
    business_knowledge_model_id: Option<String>,
    name: Option<String>,
    variable: Option<DmnVariableSnapshot>,
    encapsulated_logic: Option<TempFunctionDefinitionSnapshot>,
    body: Option<TempBusinessKnowledgeModelLiteralSnapshot>,
}

impl From<TempBusinessKnowledgeModelSnapshot> for DmnBusinessKnowledgeModelSnapshot {
    fn from(value: TempBusinessKnowledgeModelSnapshot) -> Self {
        Self {
            business_knowledge_model_id: value.business_knowledge_model_id,
            name: value.name,
            variable: value.variable,
            encapsulated_logic: value.encapsulated_logic.map(Into::into),
            body: value.body.map(Into::into),
        }
    }
}

impl TempBusinessKnowledgeModelSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            business_knowledge_model_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            variable: None,
            encapsulated_logic: None,
            body: None,
        })
    }

    pub(super) fn set_direct_variable(&mut self, variable: DmnVariableSnapshot) {
        self.variable = Some(variable);
    }

    pub(super) fn set_encapsulated_logic(
        &mut self,
        encapsulated_logic: TempFunctionDefinitionSnapshot,
    ) {
        self.encapsulated_logic = Some(encapsulated_logic);
    }

    pub(super) fn set_body(&mut self, body: TempBusinessKnowledgeModelLiteralSnapshot) {
        self.body = Some(body);
    }
}

#[derive(Debug)]
pub(super) struct TempBusinessKnowledgeModelLiteralSnapshot {
    expression_id: Option<String>,
    type_ref: Option<String>,
    text: String,
}

impl From<TempBusinessKnowledgeModelLiteralSnapshot> for DmnBusinessKnowledgeModelLiteralSnapshot {
    fn from(value: TempBusinessKnowledgeModelLiteralSnapshot) -> Self {
        let text = non_empty_text(&value.text);
        Self {
            expression_id: value.expression_id,
            type_ref: value.type_ref,
            text,
        }
    }
}

impl TempBusinessKnowledgeModelLiteralSnapshot {
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
