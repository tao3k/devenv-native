use super::business_context::{TempOrganizationUnitSnapshot, TempPerformanceIndicatorSnapshot};
use super::business_knowledge_model::TempBusinessKnowledgeModelSnapshot;
use super::decision::TempDecisionSnapshot;
use super::decision_service::TempDecisionServiceSnapshot;
use super::dmndi::{TempDiagramSnapshot, TempDmndiSnapshot};
use super::document_structure::{
    TempAssociationSnapshot, TempElementCollectionSnapshot, TempGroupSnapshot,
};
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
    current_item_definition: Option<TempItemDefinitionSnapshot>,
    current_text_annotation: Option<TempTextAnnotationSnapshot>,
    current_association: Option<TempAssociationSnapshot>,
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
            current_item_definition: None,
            current_text_annotation: None,
            current_association: None,
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
        if tag == "inputData" && parent_tag == Some("definitions") {
            return self.start_input_data(source, reader, event, is_empty);
        }
        if tag == "knowledgeSource" && parent_tag == Some("definitions") {
            return self.capture_knowledge_source(source, reader, event);
        }
        if tag == "businessKnowledgeModel" && parent_tag == Some("definitions") {
            return self.capture_business_knowledge_model(source, reader, event);
        }
        if tag == "decisionService" && parent_tag == Some("definitions") {
            return self.capture_decision_service(source, reader, event);
        }
        if tag == "organizationUnit" && parent_tag == Some("definitions") {
            return self.capture_organization_unit(source, reader, event);
        }
        if tag == "performanceIndicator" && parent_tag == Some("definitions") {
            return self.capture_performance_indicator(source, reader, event);
        }
        if tag == "textAnnotation" && parent_tag == Some("definitions") {
            return self.start_text_annotation(source, reader, event, is_empty);
        }
        if tag == "association" && parent_tag == Some("definitions") {
            return self.start_association(source, reader, event, is_empty);
        }
        if tag == "DMNDI" && parent_tag == Some("definitions") {
            return self.start_dmndi(source, reader, event, is_empty);
        }
        if tag == "DMNDiagram" && parent_tag == Some("DMNDI") {
            return self.start_dmn_diagram(source, reader, event, is_empty);
        }
        if tag == "elementCollection" && parent_tag == Some("definitions") {
            return self.capture_element_collection(source, reader, event);
        }
        if tag == "group" && parent_tag == Some("definitions") {
            return self.capture_group(source, reader, event);
        }
        if tag == "DMNShape" && parent_tag == Some("DMNDiagram") {
            return self.capture_dmn_shape(source, reader, event);
        }
        if tag == "DMNEdge" && parent_tag == Some("DMNDiagram") {
            return self.capture_dmn_edge(source, reader, event);
        }
        if tag == "waypoint" && parent_tag == Some("DMNEdge") {
            return self.capture_dmn_edge_waypoint(source, reader, event);
        }
        if tag == "waypoint" && parent_tag == Some("DMNDecisionServiceDividerLine") {
            return self
                .capture_dmn_shape_decision_service_divider_line_waypoint(source, reader, event);
        }
        if tag == "Bounds" && parent_tag == Some("DMNShape") {
            return self.capture_dmn_shape_bounds(source, reader, event);
        }
        if tag == "Bounds" && parent_tag == Some("DMNLabel") {
            return self.capture_dmn_label_bounds(source, reader, event);
        }
        if tag == "DMNLabel" && parent_tag == Some("DMNShape") {
            return self.capture_dmn_shape_label(source, reader, event);
        }
        if tag == "DMNLabel" && parent_tag == Some("DMNEdge") {
            return self.capture_dmn_edge_label(source, reader, event);
        }
        if tag == "DMNDecisionServiceDividerLine" && parent_tag == Some("DMNShape") {
            self.capture_dmn_shape_decision_service_divider_line();
            return Ok(());
        }
        if tag == "variable" && parent_tag == Some("inputData") {
            return self.capture_input_data_variable(source, reader, event);
        }
        if tag == "itemDefinition" && parent_tag == Some("definitions") {
            return self.start_item_definition(source, reader, event, is_empty);
        }
        if tag == "itemComponent" && parent_tag == Some("itemDefinition") {
            return self.capture_item_component(source, reader, event);
        }
        self.track_root_construct(tag, parent_tag);
        self.track_decision_construct(tag, parent_tag);

        Ok(())
    }

    pub(crate) fn finish_decision_end(&mut self) {
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

    pub(crate) fn finish_association_end(&mut self) {
        self.finish_open_association();
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

    fn capture_business_knowledge_model(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.business_knowledge_model_count += 1;
        root.business_knowledge_models
            .push(TempBusinessKnowledgeModelSnapshot::from_event(source, reader, event)?.into());
        Ok(())
    }

    fn capture_decision_service(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.decision_service_count += 1;
        root.decision_services
            .push(TempDecisionServiceSnapshot::from_event(source, reader, event)?.into());
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

    fn track_root_construct(&mut self, tag: &str, parent_tag: Option<&str>) {
        if parent_tag != Some("definitions") {
            return;
        }
        let Some(root) = self.root.as_mut() else {
            return;
        };
        if tag == "import" {
            root.import_count += 1;
        }
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
