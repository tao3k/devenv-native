use super::business_context::{TempOrganizationUnitSnapshot, TempPerformanceIndicatorSnapshot};
use super::business_knowledge_model::{
    TempBusinessKnowledgeModelLiteralSnapshot, TempBusinessKnowledgeModelSnapshot,
};
use super::decision::{
    TempDecisionSnapshot, TempFunctionDefinitionLiteralSnapshot,
    TempFunctionDefinitionParameterSnapshot, TempFunctionDefinitionSnapshot,
    TempInvocationBindingSnapshot, TempInvocationLiteralSnapshot, TempInvocationParameterSnapshot,
    TempInvocationSnapshot, TempRequirementReferenceSnapshot,
};
use super::decision_service::{TempDecisionServiceReferenceSnapshot, TempDecisionServiceSnapshot};
use super::dmndi::{TempDiagramSnapshot, TempDmndiSnapshot};
use super::document_structure::{
    TempAssociationSnapshot, TempElementCollectionSnapshot, TempGroupSnapshot,
};
use super::import::TempImportSnapshot;
use super::input_data::{TempInputDataSnapshot, variable_from_event};
use super::item_definition::{TempItemDefinitionSnapshot, item_component_from_event};
use super::knowledge_source::TempKnowledgeSourceSnapshot;
use super::text_annotation::TempTextAnnotationSnapshot;
use crate::dmn::snapshot::root::build_root_snapshot;
use crate::dmn::snapshot::xml::local_name;
use crate::dmn_model_api::{DmnDecisionSnapshot, DmnRootSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(crate) struct SnapshotScanState {
    root: Option<DmnRootSnapshot>,
    current_decision: Option<TempDecisionSnapshot>,
    current_input_data: Option<TempInputDataSnapshot>,
    current_business_knowledge_model: Option<TempBusinessKnowledgeModelSnapshot>,
    current_business_knowledge_model_literal: Option<TempBusinessKnowledgeModelLiteralSnapshot>,
    current_decision_service: Option<TempDecisionServiceSnapshot>,
    current_item_definition: Option<TempItemDefinitionSnapshot>,
    current_text_annotation: Option<TempTextAnnotationSnapshot>,
    current_association: Option<TempAssociationSnapshot>,
    current_invocation: Option<TempInvocationSnapshot>,
    current_invocation_binding: Option<TempInvocationBindingSnapshot>,
    current_invocation_literal: Option<TempInvocationLiteralSnapshot>,
    current_invocation_literal_target: Option<InvocationLiteralTarget>,
    current_function_definition: Option<TempFunctionDefinitionSnapshot>,
    current_function_definition_owner: Option<FunctionDefinitionOwner>,
    current_function_definition_literal: Option<TempFunctionDefinitionLiteralSnapshot>,
    current_dmndi: Option<TempDmndiSnapshot>,
    current_dmn_diagram: Option<TempDiagramSnapshot>,
    current_dmn_label_owner: Option<DmnLabelOwner>,
    decisions: Vec<DmnDecisionSnapshot>,
}

impl SnapshotScanState {
    pub(crate) fn new() -> Self {
        Self {
            root: None,
            current_decision: None,
            current_input_data: None,
            current_business_knowledge_model: None,
            current_business_knowledge_model_literal: None,
            current_decision_service: None,
            current_item_definition: None,
            current_text_annotation: None,
            current_association: None,
            current_invocation: None,
            current_invocation_binding: None,
            current_invocation_literal: None,
            current_invocation_literal_target: None,
            current_function_definition: None,
            current_function_definition_owner: None,
            current_function_definition_literal: None,
            current_dmndi: None,
            current_dmn_diagram: None,
            current_dmn_label_owner: None,
            decisions: Vec::new(),
        }
    }

    pub(crate) fn handle_start_event(
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
        if self.handle_invocation_start_event(source, reader, event, parent_tag, is_empty)? {
            return Ok(());
        }
        if self
            .handle_function_definition_start_event(source, reader, event, parent_tag, is_empty)?
        {
            return Ok(());
        }
        if self.handle_requirement_reference_start_event(source, reader, event, parent_tag)? {
            return Ok(());
        }
        if self.handle_definitions_start_event(source, reader, event, parent_tag, is_empty)? {
            return Ok(());
        }
        if self.handle_dmndi_start_event(source, reader, event, parent_tag, is_empty)? {
            return Ok(());
        }
        self.track_decision_construct(tag, parent_tag);

        Ok(())
    }

    fn handle_invocation_start_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<bool> {
        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        match (tag, parent_tag) {
            ("invocation", Some("decision")) => {
                self.start_invocation(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("binding", Some("invocation")) => {
                self.start_invocation_binding(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("parameter", Some("binding")) => {
                self.capture_invocation_parameter(source, reader, event)?;
                Ok(true)
            }
            ("literalExpression", Some("invocation")) => {
                self.start_invocation_literal_expression(
                    source,
                    reader,
                    event,
                    InvocationLiteralTarget::InvokedExpression,
                    is_empty,
                )?;
                Ok(true)
            }
            ("literalExpression", Some("binding")) => {
                self.start_invocation_literal_expression(
                    source,
                    reader,
                    event,
                    InvocationLiteralTarget::BindingArgument,
                    is_empty,
                )?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_function_definition_start_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<bool> {
        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        match (tag, parent_tag) {
            ("functionDefinition", Some("decision")) => {
                self.start_function_definition(
                    source,
                    reader,
                    event,
                    FunctionDefinitionOwner::Decision,
                    is_empty,
                )?;
                Ok(true)
            }
            ("encapsulatedLogic", Some("businessKnowledgeModel")) => {
                self.start_function_definition(
                    source,
                    reader,
                    event,
                    FunctionDefinitionOwner::BusinessKnowledgeModel,
                    is_empty,
                )?;
                Ok(true)
            }
            ("formalParameter", Some("functionDefinition" | "encapsulatedLogic")) => {
                self.capture_function_definition_parameter(source, reader, event)?;
                Ok(true)
            }
            ("literalExpression", Some("functionDefinition" | "encapsulatedLogic")) => {
                self.start_function_definition_literal_expression(source, reader, event, is_empty)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_definitions_start_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<bool> {
        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        match (tag, parent_tag) {
            ("inputData", Some("definitions")) => {
                self.start_input_data(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("import", Some("definitions")) => {
                self.capture_import(source, reader, event)?;
                Ok(true)
            }
            ("knowledgeSource", Some("definitions")) => {
                self.capture_knowledge_source(source, reader, event)?;
                Ok(true)
            }
            ("businessKnowledgeModel", Some("definitions")) => {
                self.start_business_knowledge_model(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("variable", Some("businessKnowledgeModel")) => {
                self.capture_business_knowledge_model_variable(source, reader, event)?;
                Ok(true)
            }
            ("literalExpression", Some("businessKnowledgeModel")) => {
                self.start_business_knowledge_model_literal_expression(
                    source, reader, event, is_empty,
                )?;
                Ok(true)
            }
            ("decisionService", Some("definitions")) => {
                self.start_decision_service(source, reader, event, is_empty)?;
                Ok(true)
            }
            (
                "outputDecision" | "encapsulatedDecision" | "inputDecision" | "inputData",
                Some("decisionService"),
            ) => {
                self.capture_decision_service_reference(source, reader, event, tag)?;
                Ok(true)
            }
            ("organizationUnit", Some("definitions")) => {
                self.capture_organization_unit(source, reader, event)?;
                Ok(true)
            }
            ("performanceIndicator", Some("definitions")) => {
                self.capture_performance_indicator(source, reader, event)?;
                Ok(true)
            }
            ("textAnnotation", Some("definitions")) => {
                self.start_text_annotation(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("association", Some("definitions")) => {
                self.start_association(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("elementCollection", Some("definitions")) => {
                self.capture_element_collection(source, reader, event)?;
                Ok(true)
            }
            ("group", Some("definitions")) => {
                self.capture_group(source, reader, event)?;
                Ok(true)
            }
            ("variable", Some("inputData")) => {
                self.capture_input_data_variable(source, reader, event)?;
                Ok(true)
            }
            ("itemDefinition", Some("definitions")) => {
                self.start_item_definition(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("itemComponent", Some("itemDefinition")) => {
                self.capture_item_component(source, reader, event)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_requirement_reference_start_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
    ) -> Result<bool> {
        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        match (tag, parent_tag) {
            ("requiredInput" | "requiredDecision", Some("informationRequirement"))
            | ("requiredKnowledge", Some("knowledgeRequirement"))
            | (
                "requiredAuthority" | "requiredDecision" | "requiredInput",
                Some("authorityRequirement"),
            ) => {
                self.capture_decision_requirement_reference(
                    source, reader, event, parent_tag, tag,
                )?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_dmndi_start_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<bool> {
        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        match (tag, parent_tag) {
            ("DMNDI", Some("definitions")) => {
                self.start_dmndi(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("DMNDiagram", Some("DMNDI")) => {
                self.start_dmn_diagram(source, reader, event, is_empty)?;
                Ok(true)
            }
            ("DMNShape", Some("DMNDiagram")) => {
                self.capture_dmn_shape(source, reader, event)?;
                Ok(true)
            }
            ("DMNEdge", Some("DMNDiagram")) => {
                self.capture_dmn_edge(source, reader, event)?;
                Ok(true)
            }
            ("waypoint", Some("DMNEdge")) => {
                self.capture_dmn_edge_waypoint(source, reader, event)?;
                Ok(true)
            }
            ("waypoint", Some("DMNDecisionServiceDividerLine")) => {
                self.capture_dmn_shape_decision_service_divider_line_waypoint(
                    source, reader, event,
                )?;
                Ok(true)
            }
            ("Bounds", Some("DMNShape")) => {
                self.capture_dmn_shape_bounds(source, reader, event)?;
                Ok(true)
            }
            ("Bounds", Some("DMNLabel")) => {
                self.capture_dmn_label_bounds(source, reader, event)?;
                Ok(true)
            }
            ("DMNLabel", Some("DMNShape")) => {
                self.capture_dmn_shape_label(source, reader, event)?;
                Ok(true)
            }
            ("DMNLabel", Some("DMNEdge")) => {
                self.capture_dmn_edge_label(source, reader, event)?;
                Ok(true)
            }
            ("DMNDecisionServiceDividerLine", Some("DMNShape")) => {
                self.capture_dmn_shape_decision_service_divider_line();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn finish_decision_end(&mut self) {
        self.finish_open_invocation();
        self.finish_open_function_definition();
        if let Some(decision) = self.current_decision.take() {
            self.decisions.push(decision.into());
        }
    }

    pub(crate) fn finish_pending_decision(&mut self) {
        self.finish_decision_end();
    }

    pub(crate) fn finish_item_definition_end(&mut self) {
        self.finish_open_item_definition();
    }

    pub(crate) fn finish_text_annotation_end(&mut self) {
        self.finish_open_text_annotation();
    }

    pub(crate) fn finish_input_data_end(&mut self) {
        self.finish_open_input_data();
    }

    pub(crate) fn finish_business_knowledge_model_end(&mut self) {
        self.finish_open_business_knowledge_model();
    }

    pub(crate) fn finish_decision_service_end(&mut self) {
        self.finish_open_decision_service();
    }

    pub(crate) fn finish_association_end(&mut self) {
        self.finish_open_association();
    }

    pub(crate) fn finish_invocation_end(&mut self) {
        self.finish_open_invocation();
    }

    pub(crate) fn finish_invocation_binding_end(&mut self) {
        self.finish_open_invocation_binding();
    }

    pub(crate) fn finish_literal_expression_end(&mut self) {
        self.finish_open_invocation_literal_expression();
        self.finish_open_function_definition_literal_expression();
        self.finish_open_business_knowledge_model_literal_expression();
    }

    pub(crate) fn finish_function_definition_end(&mut self) {
        self.finish_open_function_definition();
    }

    pub(crate) fn finish_dmndi_end(&mut self) {
        self.finish_open_dmndi();
    }

    pub(crate) fn finish_dmn_diagram_end(&mut self) {
        self.finish_dmn_label_end();
        self.finish_open_dmn_diagram();
    }

    pub(crate) fn finish_dmn_label_end(&mut self) {
        self.current_dmn_label_owner = None;
    }

    pub(crate) fn finish_pending_item_definition(&mut self) {
        self.finish_open_item_definition();
    }

    pub(crate) fn finish_pending_text_annotation(&mut self) {
        self.finish_open_text_annotation();
    }

    pub(crate) fn finish_pending_input_data(&mut self) {
        self.finish_open_input_data();
    }

    pub(crate) fn finish_pending_business_knowledge_model(&mut self) {
        self.finish_open_business_knowledge_model();
    }

    pub(crate) fn finish_pending_decision_service(&mut self) {
        self.finish_open_decision_service();
    }

    pub(crate) fn finish_pending_association(&mut self) {
        self.finish_open_association();
    }

    pub(crate) fn finish_pending_dmndi(&mut self) {
        self.finish_open_dmndi();
    }

    pub(crate) fn handle_text_chunk(
        &mut self,
        text: &str,
        current_tag: Option<&str>,
        parent_tag: Option<&str>,
    ) {
        match (current_tag, parent_tag) {
            (Some("text"), Some("textAnnotation")) => {
                let Some(text_annotation) = self.current_text_annotation.as_mut() else {
                    return;
                };
                text_annotation.append_text(text);
            }
            (Some("sourceRef"), Some("association")) => {
                let Some(association) = self.current_association.as_mut() else {
                    return;
                };
                association.append_source_ref(text);
            }
            (Some("targetRef"), Some("association")) => {
                let Some(association) = self.current_association.as_mut() else {
                    return;
                };
                association.append_target_ref(text);
            }
            (Some("text"), Some("literalExpression")) => {
                self.append_literal_expression_text(text);
            }
            (Some("Text"), Some("DMNLabel")) => match self.current_dmn_label_owner {
                Some(DmnLabelOwner::Shape) => {
                    let Some(diagram) = self.current_dmn_diagram.as_mut() else {
                        return;
                    };
                    diagram.append_shape_label_text(text);
                }
                Some(DmnLabelOwner::Edge) => {
                    let Some(diagram) = self.current_dmn_diagram.as_mut() else {
                        return;
                    };
                    diagram.append_edge_label_text(text);
                }
                None => {}
            },
            _ => {}
        }
    }

    pub(crate) fn into_parts(self) -> (Option<DmnRootSnapshot>, Vec<DmnDecisionSnapshot>) {
        (self.root, self.decisions)
    }

    fn append_literal_expression_text(&mut self, text: &str) {
        if let Some(literal) = self.current_invocation_literal.as_mut() {
            literal.append_text(text);
        }
        if let Some(literal) = self.current_function_definition_literal.as_mut() {
            literal.append_text(text);
        }
        if let Some(literal) = self.current_business_knowledge_model_literal.as_mut() {
            literal.append_text(text);
        }
    }

    fn finish_open_input_data(&mut self) {
        let Some(input_data) = self.current_input_data.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.input_data.push(input_data.into());
    }

    fn finish_open_item_definition(&mut self) {
        let Some(item_definition) = self.current_item_definition.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.item_definitions.push(item_definition.into());
    }

    fn finish_open_business_knowledge_model(&mut self) {
        self.finish_open_business_knowledge_model_literal_expression();
        let Some(business_knowledge_model) = self.current_business_knowledge_model.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.business_knowledge_models
            .push(business_knowledge_model.into());
    }

    fn finish_open_business_knowledge_model_literal_expression(&mut self) {
        let Some(literal) = self.current_business_knowledge_model_literal.take() else {
            return;
        };
        if let Some(business_knowledge_model) = self.current_business_knowledge_model.as_mut() {
            business_knowledge_model.set_body(literal);
        }
    }

    fn finish_open_decision_service(&mut self) {
        let Some(decision_service) = self.current_decision_service.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.decision_services.push(decision_service.into());
    }

    fn finish_open_text_annotation(&mut self) {
        let Some(text_annotation) = self.current_text_annotation.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.text_annotations.push(text_annotation.into());
    }

    fn finish_open_association(&mut self) {
        let Some(association) = self.current_association.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.associations.push(association.into());
    }

    fn finish_open_invocation(&mut self) {
        self.finish_open_invocation_binding();
        self.finish_open_invocation_literal_expression();
        let Some(invocation) = self.current_invocation.take() else {
            return;
        };
        let Some(decision) = self.current_decision.as_mut() else {
            return;
        };
        decision.push_invocation(invocation);
    }

    fn finish_open_invocation_binding(&mut self) {
        self.finish_open_invocation_literal_expression();
        let Some(binding) = self.current_invocation_binding.take() else {
            return;
        };
        let Some(invocation) = self.current_invocation.as_mut() else {
            return;
        };
        invocation.push_binding(binding);
    }

    fn finish_open_invocation_literal_expression(&mut self) {
        let Some(literal) = self.current_invocation_literal.take() else {
            return;
        };
        match self.current_invocation_literal_target.take() {
            Some(InvocationLiteralTarget::InvokedExpression) => {
                if let Some(invocation) = self.current_invocation.as_mut() {
                    invocation.set_invoked_expression(literal);
                }
            }
            Some(InvocationLiteralTarget::BindingArgument) => {
                if let Some(binding) = self.current_invocation_binding.as_mut() {
                    binding.set_argument(literal);
                }
            }
            None => {}
        }
    }

    fn finish_open_function_definition(&mut self) {
        self.finish_open_function_definition_literal_expression();
        let Some(function_definition) = self.current_function_definition.take() else {
            return;
        };
        match self.current_function_definition_owner.take() {
            Some(FunctionDefinitionOwner::Decision) => {
                let Some(decision) = self.current_decision.as_mut() else {
                    return;
                };
                decision.push_function_definition(function_definition);
            }
            Some(FunctionDefinitionOwner::BusinessKnowledgeModel) => {
                let Some(business_knowledge_model) = self.current_business_knowledge_model.as_mut()
                else {
                    return;
                };
                business_knowledge_model.set_encapsulated_logic(function_definition);
            }
            None => {}
        }
    }

    fn finish_open_function_definition_literal_expression(&mut self) {
        let Some(literal) = self.current_function_definition_literal.take() else {
            return;
        };
        if let Some(function_definition) = self.current_function_definition.as_mut() {
            function_definition.set_body(literal);
        }
    }

    fn finish_open_dmndi(&mut self) {
        self.finish_dmn_label_end();
        self.finish_open_dmn_diagram();
        let Some(dmndi) = self.current_dmndi.take() else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        root.dmndi_blocks.push(dmndi.into());
    }

    fn finish_open_dmn_diagram(&mut self) {
        let Some(diagram) = self.current_dmn_diagram.take() else {
            return;
        };
        let Some(dmndi) = self.current_dmndi.as_mut() else {
            return;
        };
        dmndi.push_diagram(diagram.into());
    }

    fn start_decision(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_pending_decision();
        let decision = TempDecisionSnapshot::from_event(source, reader, event)?;
        if is_empty {
            self.decisions.push(decision.into());
        } else {
            self.current_decision = Some(decision);
        }
        Ok(())
    }

    fn start_invocation(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        if self.current_decision.is_none() {
            return Ok(());
        }
        self.finish_open_invocation();
        let Some(decision) = self.current_decision.as_mut() else {
            return Ok(());
        };
        decision.track_construct("invocation", Some("decision"));
        let invocation = TempInvocationSnapshot::from_event(source, reader, event)?;
        if is_empty {
            decision.push_invocation(invocation);
        } else {
            self.current_invocation = Some(invocation);
        }
        Ok(())
    }

    fn start_function_definition(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        owner: FunctionDefinitionOwner,
        is_empty: bool,
    ) -> Result<()> {
        let owner_is_available = match owner {
            FunctionDefinitionOwner::Decision => self.current_decision.is_some(),
            FunctionDefinitionOwner::BusinessKnowledgeModel => {
                self.current_business_knowledge_model.is_some()
            }
        };
        if !owner_is_available {
            return Ok(());
        }
        self.finish_open_function_definition();
        let function_definition =
            TempFunctionDefinitionSnapshot::from_event(source, reader, event)?;
        if is_empty {
            self.current_function_definition_owner = Some(owner);
            self.current_function_definition = Some(function_definition);
            self.finish_open_function_definition();
        } else {
            if let Some(decision) = self.current_decision.as_mut()
                && matches!(owner, FunctionDefinitionOwner::Decision)
            {
                decision.track_construct("functionDefinition", Some("decision"));
            }
            self.current_function_definition_owner = Some(owner);
            self.current_function_definition = Some(function_definition);
        }
        Ok(())
    }

    fn capture_function_definition_parameter(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(function_definition) = self.current_function_definition.as_mut() else {
            return Ok(());
        };
        function_definition.push_parameter(TempFunctionDefinitionParameterSnapshot::from_event(
            source, reader, event,
        )?);
        Ok(())
    }

    fn start_function_definition_literal_expression(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        if self.current_function_definition.is_none() {
            return Ok(());
        }
        self.finish_open_function_definition_literal_expression();
        self.current_function_definition_literal = Some(
            TempFunctionDefinitionLiteralSnapshot::from_event(source, reader, event)?,
        );
        if is_empty {
            self.finish_open_function_definition_literal_expression();
        }
        Ok(())
    }

    fn start_invocation_binding(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        if self.current_invocation.is_none() {
            return Ok(());
        }
        self.finish_open_invocation_binding();
        let binding = TempInvocationBindingSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(invocation) = self.current_invocation.as_mut() {
                invocation.push_binding(binding);
            }
        } else {
            self.current_invocation_binding = Some(binding);
        }
        Ok(())
    }

    fn capture_invocation_parameter(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(binding) = self.current_invocation_binding.as_mut() else {
            return Ok(());
        };
        binding.set_parameter(TempInvocationParameterSnapshot::from_event(
            source, reader, event,
        )?);
        Ok(())
    }

    fn capture_decision_requirement_reference(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        requirement_kind: Option<&str>,
        reference_kind: &str,
    ) -> Result<()> {
        let Some(requirement_kind) = requirement_kind else {
            return Ok(());
        };
        let Some(decision) = self.current_decision.as_mut() else {
            return Ok(());
        };
        decision.track_construct(reference_kind, Some(requirement_kind));
        decision.push_requirement_reference(TempRequirementReferenceSnapshot::from_event(
            source,
            reader,
            event,
            requirement_kind,
            reference_kind,
        )?);
        Ok(())
    }

    fn start_invocation_literal_expression(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        target: InvocationLiteralTarget,
        is_empty: bool,
    ) -> Result<()> {
        let target_is_available = match target {
            InvocationLiteralTarget::InvokedExpression => self.current_invocation.is_some(),
            InvocationLiteralTarget::BindingArgument => self.current_invocation_binding.is_some(),
        };
        if !target_is_available {
            return Ok(());
        }
        self.finish_open_invocation_literal_expression();
        self.current_invocation_literal = Some(TempInvocationLiteralSnapshot::from_event(
            source, reader, event,
        )?);
        self.current_invocation_literal_target = Some(target);
        if is_empty {
            self.finish_open_invocation_literal_expression();
        }
        Ok(())
    }

    fn start_input_data(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_input_data();
        if let Some(root) = self.root.as_mut() {
            root.input_data_count += 1;
        }
        let input_data = TempInputDataSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.input_data.push(input_data.into());
            }
        } else {
            self.current_input_data = Some(input_data);
        }
        Ok(())
    }

    fn start_item_definition(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_item_definition();
        if let Some(root) = self.root.as_mut() {
            root.item_definition_count += 1;
        }
        let item_definition = TempItemDefinitionSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.item_definitions.push(item_definition.into());
            }
        } else {
            self.current_item_definition = Some(item_definition);
        }
        Ok(())
    }

    fn start_business_knowledge_model(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_business_knowledge_model();
        if let Some(root) = self.root.as_mut() {
            root.business_knowledge_model_count += 1;
        }
        let business_knowledge_model =
            TempBusinessKnowledgeModelSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.business_knowledge_models
                    .push(business_knowledge_model.into());
            }
        } else {
            self.current_business_knowledge_model = Some(business_knowledge_model);
        }
        Ok(())
    }

    fn start_business_knowledge_model_literal_expression(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        if self.current_business_knowledge_model.is_none() {
            return Ok(());
        }
        self.finish_open_business_knowledge_model_literal_expression();
        self.current_business_knowledge_model_literal = Some(
            TempBusinessKnowledgeModelLiteralSnapshot::from_event(source, reader, event)?,
        );
        if is_empty {
            self.finish_open_business_knowledge_model_literal_expression();
        }
        Ok(())
    }

    fn start_decision_service(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_decision_service();
        if let Some(root) = self.root.as_mut() {
            root.decision_service_count += 1;
        }
        let decision_service = TempDecisionServiceSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.decision_services.push(decision_service.into());
            }
        } else {
            self.current_decision_service = Some(decision_service);
        }
        Ok(())
    }

    fn start_text_annotation(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_text_annotation();
        if let Some(root) = self.root.as_mut() {
            root.text_annotation_count += 1;
        }
        let text_annotation = TempTextAnnotationSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.text_annotations.push(text_annotation.into());
            }
        } else {
            self.current_text_annotation = Some(text_annotation);
        }
        Ok(())
    }

    fn start_association(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_association();
        if let Some(root) = self.root.as_mut() {
            root.association_count += 1;
        }
        let association = TempAssociationSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.associations.push(association.into());
            }
        } else {
            self.current_association = Some(association);
        }
        Ok(())
    }

    fn start_dmndi(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_dmndi();
        if let Some(root) = self.root.as_mut() {
            root.dmndi_count += 1;
        }
        let dmndi = TempDmndiSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(root) = self.root.as_mut() {
                root.dmndi_blocks.push(dmndi.into());
            }
        } else {
            self.current_dmndi = Some(dmndi);
        }
        Ok(())
    }

    fn start_dmn_diagram(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        self.finish_open_dmn_diagram();
        if self.current_dmndi.is_none() {
            return Ok(());
        }
        let diagram = TempDiagramSnapshot::from_event(source, reader, event)?;
        if is_empty {
            if let Some(dmndi) = self.current_dmndi.as_mut() {
                dmndi.push_diagram(diagram.into());
            }
        } else {
            self.current_dmn_diagram = Some(diagram);
        }
        Ok(())
    }

    fn capture_input_data_variable(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(input_data) = self.current_input_data.as_mut() else {
            return Ok(());
        };
        input_data.set_direct_variable(variable_from_event(source, reader, event)?);
        Ok(())
    }

    fn capture_business_knowledge_model_variable(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(business_knowledge_model) = self.current_business_knowledge_model.as_mut() else {
            return Ok(());
        };
        business_knowledge_model.set_direct_variable(variable_from_event(source, reader, event)?);
        Ok(())
    }

    fn capture_item_component(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(item_definition) = self.current_item_definition.as_mut() else {
            return Ok(());
        };
        item_definition
            .push_direct_item_component(item_component_from_event(source, reader, event)?);
        Ok(())
    }

    fn capture_knowledge_source(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.knowledge_source_count += 1;
        root.knowledge_sources
            .push(TempKnowledgeSourceSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_import(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.import_count += 1;
        root.imports
            .push(TempImportSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_decision_service_reference(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        reference_kind: &str,
    ) -> Result<()> {
        let Some(decision_service) = self.current_decision_service.as_mut() else {
            return Ok(());
        };
        let reference = TempDecisionServiceReferenceSnapshot::from_event(
            source,
            reader,
            event,
            reference_kind,
        )?;
        match reference_kind {
            "outputDecision" => decision_service.push_output_decision(reference),
            "encapsulatedDecision" => decision_service.push_encapsulated_decision(reference),
            "inputDecision" => decision_service.push_input_decision(reference),
            "inputData" => decision_service.push_input_data(reference),
            _ => {}
        }
        Ok(())
    }

    fn capture_organization_unit(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.organization_unit_count += 1;
        root.organization_units
            .push(TempOrganizationUnitSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_performance_indicator(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.performance_indicator_count += 1;
        root.performance_indicators
            .push(TempPerformanceIndicatorSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_element_collection(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.element_collection_count += 1;
        root.element_collections
            .push(TempElementCollectionSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_group(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.group_count += 1;
        root.groups
            .push(TempGroupSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_dmn_shape(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.push_shape_from_event(source, reader, event)
    }

    fn capture_dmn_edge(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.push_edge_from_event(source, reader, event)
    }

    fn capture_dmn_edge_waypoint(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.push_edge_waypoint_from_event(source, reader, event)
    }

    fn capture_dmn_shape_decision_service_divider_line(&mut self) {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return;
        };
        diagram.attach_shape_decision_service_divider_line();
    }

    fn capture_dmn_shape_decision_service_divider_line_waypoint(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.push_shape_decision_service_divider_line_waypoint(source, reader, event)
    }

    fn capture_dmn_shape_bounds(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.attach_shape_bounds_from_event(source, reader, event)
    }

    fn capture_dmn_label_bounds(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        match self.current_dmn_label_owner {
            Some(DmnLabelOwner::Shape) => {
                diagram.attach_shape_label_bounds_from_event(source, reader, event)
            }
            Some(DmnLabelOwner::Edge) => {
                diagram.attach_edge_label_bounds_from_event(source, reader, event)
            }
            None => Ok(()),
        }
    }

    fn capture_dmn_shape_label(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.attach_shape_label_from_event(source, reader, event)?;
        self.current_dmn_label_owner = Some(DmnLabelOwner::Shape);
        Ok(())
    }

    fn capture_dmn_edge_label(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(diagram) = self.current_dmn_diagram.as_mut() else {
            return Ok(());
        };
        diagram.attach_edge_label_from_event(source, reader, event)?;
        self.current_dmn_label_owner = Some(DmnLabelOwner::Edge);
        Ok(())
    }

    fn track_decision_construct(&mut self, tag: &str, parent_tag: Option<&str>) {
        let Some(decision) = self.current_decision.as_mut() else {
            return;
        };
        decision.track_construct(tag, parent_tag);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DmnLabelOwner {
    Shape,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvocationLiteralTarget {
    InvokedExpression,
    BindingArgument,
}

#[derive(Debug, Clone, Copy)]
enum FunctionDefinitionOwner {
    Decision,
    BusinessKnowledgeModel,
}
