use super::root::build_root_snapshot;
use super::xml::{attribute_value, local_name, required_attribute};
use crate::dmn_model_api::{DmnDecisionSnapshot, DmnRootSnapshot, DmnSourceFile};
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
    fn from_event(
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
}

pub(super) struct SnapshotScanState {
    root: Option<DmnRootSnapshot>,
    current_decision: Option<TempDecisionSnapshot>,
    decisions: Vec<DmnDecisionSnapshot>,
}

impl SnapshotScanState {
    pub(super) fn new() -> Self {
        Self {
            root: None,
            current_decision: None,
            decisions: Vec::new(),
        }
    }

    pub(super) fn handle_start_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(build_root_snapshot(source, reader, event)?);
        }

        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        if tag == "decision" {
            return self.start_decision(source, reader, event, is_empty);
        }
        self.track_root_construct(tag, parent_tag);
        self.track_decision_construct(tag, parent_tag);

        Ok(())
    }

    pub(super) fn finish_decision_end(&mut self) {
        self.finish_open_decision();
    }

    pub(super) fn finish_pending_decision(&mut self) {
        self.finish_open_decision();
    }

    pub(super) fn into_parts(self) -> (Option<DmnRootSnapshot>, Vec<DmnDecisionSnapshot>) {
        (self.root, self.decisions)
    }

    fn finish_open_decision(&mut self) {
        if let Some(decision) = self.current_decision.take() {
            self.decisions.push(decision.into());
        }
    }

    fn start_decision(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_decision();
        let decision = TempDecisionSnapshot::from_event(source, reader, event)?;
        if is_empty {
            self.decisions.push(decision.into());
        } else {
            self.current_decision = Some(decision);
        }
        Ok(())
    }

    fn track_root_construct(&mut self, tag: &str, parent_tag: Option<&str>) {
        if parent_tag != Some("definitions") {
            return;
        }
        let Some(root) = self.root.as_mut() else {
            return;
        };
        match tag {
            "import" => root.import_count += 1,
            "itemDefinition" => root.item_definition_count += 1,
            "inputData" => root.input_data_count += 1,
            "knowledgeSource" => root.knowledge_source_count += 1,
            "businessKnowledgeModel" => root.business_knowledge_model_count += 1,
            "decisionService" => root.decision_service_count += 1,
            "organizationUnit" => root.organization_unit_count += 1,
            "performanceIndicator" => root.performance_indicator_count += 1,
            "textAnnotation" => root.text_annotation_count += 1,
            "association" => root.association_count += 1,
            "elementCollection" => root.element_collection_count += 1,
            "group" => root.group_count += 1,
            "DMNDI" => root.dmndi_count += 1,
            _ => {}
        }
    }

    fn track_decision_construct(&mut self, tag: &str, parent_tag: Option<&str>) {
        let Some(decision) = self.current_decision.as_mut() else {
            return;
        };
        match (parent_tag, tag) {
            (Some("decision"), "allowedAnswers") => decision.allowed_answers_count += 1,
            (Some("decision"), "decisionMaker") => decision.decision_maker_count += 1,
            (Some("decision"), "decisionOwner") => decision.decision_owner_count += 1,
            (Some("decision"), "decisionTable") => decision.decision_table_count += 1,
            (Some("decision"), "informationRequirement") => {
                decision.information_requirement_count += 1;
            }
            (Some("informationRequirement"), "requiredInput") => {
                decision.required_input_count += 1;
            }
            (Some("informationRequirement"), "requiredDecision") => {
                decision.required_decision_count += 1;
            }
            (Some("decision"), "knowledgeRequirement") => {
                decision.knowledge_requirement_count += 1;
            }
            (Some("knowledgeRequirement"), "requiredKnowledge") => {
                decision.required_knowledge_count += 1;
            }
            (Some("decision"), "authorityRequirement") => {
                decision.authority_requirement_count += 1;
            }
            (Some("authorityRequirement"), "requiredAuthority") => {
                decision.required_authority_count += 1;
            }
            (Some("decision"), "literalExpression") => decision.literal_expression_count += 1,
            (Some("decision"), "context") => decision.context_count += 1,
            (Some("decision"), "invocation") => decision.invocation_count += 1,
            (Some("decision"), "relation") => decision.relation_count += 1,
            (Some("decision"), "functionDefinition") => {
                decision.function_definition_count += 1;
            }
            (Some("decision"), "list") => decision.list_count += 1,
            _ => {}
        }
    }
}
