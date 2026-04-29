use super::xml::{attribute_value, boolean_attribute_value, bpmn_model_namespace, local_name};
use crate::bpmn_model_api::{
    BpmnAssociationSnapshot, BpmnBoundsSnapshot, BpmnCategorySnapshot, BpmnCategoryValueSnapshot,
    BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationAssociationSnapshot, BpmnConversationLinkSnapshot,
    BpmnConversationNodeSnapshot, BpmnCorrelationKeySnapshot, BpmnCorrelationPropertySnapshot,
    BpmnCorrelationRetrievalExpressionSnapshot, BpmnDataAssociationSnapshot,
    BpmnDataInputOutputSnapshot, BpmnDataObjectReferenceSnapshot, BpmnDataObjectSnapshot,
    BpmnDataStoreReferenceSnapshot, BpmnDataStoreSnapshot, BpmnDiagramSnapshot,
    BpmnDocumentSnapshot, BpmnEdgeSnapshot, BpmnErrorSnapshot, BpmnEscalationSnapshot,
    BpmnExtensionSnapshot, BpmnFontSnapshot, BpmnGroupSnapshot, BpmnImportSnapshot,
    BpmnInterfaceSnapshot, BpmnIoSpecificationSnapshot, BpmnItemDefinitionSnapshot,
    BpmnLabelSnapshot, BpmnLabelStyleSnapshot, BpmnLaneSetSnapshot, BpmnLaneSnapshot,
    BpmnMessageFlowAssociationSnapshot, BpmnMessageFlowSnapshot, BpmnMessageSnapshot,
    BpmnOperationSnapshot, BpmnParticipantAssociationSnapshot, BpmnParticipantSnapshot,
    BpmnPlaneSnapshot, BpmnProcessSnapshot, BpmnRelationshipSnapshot,
    BpmnResourceParameterSnapshot, BpmnResourceSnapshot, BpmnRootSnapshot, BpmnShapeSnapshot,
    BpmnSignalSnapshot, BpmnTextAnnotationSnapshot, BpmnWaypointSnapshot,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug, Clone, Copy)]
pub(super) enum TextTarget {
    LaneFlowNode,
    DataAssociationSource,
    DataAssociationTarget,
    CorrelationMessagePath,
    OperationInMessageRef,
    OperationOutMessageRef,
    OperationErrorRef,
    ExtensionDocumentation,
    RelationshipSource,
    RelationshipTarget,
    ConversationParticipantRef,
    ConversationMessageFlowRef,
    ChoreographyParticipantRef,
    ChoreographyMessageFlowRef,
    TextAnnotationText,
    CorrelationKeyPropertyRef,
    ParticipantAssociationInnerRef,
    ParticipantAssociationOuterRef,
    CollaborationChoreographyRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataAssociationKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BpmnDiLabelTarget {
    Shape(usize, usize),
    Edge(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CollaborationMetadataOwner {
    Collaboration(usize),
    ConversationNode(usize, Vec<usize>),
    ChoreographyActivity(usize, Vec<usize>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtifactMetadataOwner {
    Collaboration(usize),
    Process(usize),
}

#[derive(Debug, Default)]
pub(super) struct BpmnSnapshotScanState {
    root: Option<BpmnRootSnapshot>,
    collaborations: Vec<BpmnCollaborationSnapshot>,
    processes: Vec<BpmnProcessSnapshot>,
    current_collaboration: Option<usize>,
    conversation_node_stack: Vec<(usize, Vec<usize>)>,
    choreography_activity_stack: Vec<(usize, Vec<usize>)>,
    current_conversation_correlation_key:
        Option<(CollaborationMetadataOwner, BpmnCorrelationKeySnapshot)>,
    current_participant_association: Option<(
        CollaborationMetadataOwner,
        BpmnParticipantAssociationSnapshot,
    )>,
    current_text_annotation: Option<(ArtifactMetadataOwner, BpmnTextAnnotationSnapshot)>,
    current_process: Option<usize>,
    lane_set_stack: Vec<(usize, usize)>,
    lane_stack: Vec<(usize, usize, usize)>,
    current_correlation_property: Option<usize>,
    current_correlation_retrieval_expression:
        Option<(usize, BpmnCorrelationRetrievalExpressionSnapshot)>,
    current_interface: Option<usize>,
    current_operation: Option<(usize, usize)>,
    current_resource: Option<usize>,
    current_category: Option<usize>,
    current_extension: Option<usize>,
    current_extension_documentation: Option<(usize, String)>,
    current_relationship: Option<usize>,
    current_diagram: Option<usize>,
    current_plane: Option<usize>,
    current_shape: Option<(usize, usize)>,
    current_edge: Option<(usize, usize)>,
    current_label: Option<BpmnDiLabelTarget>,
    current_label_style: Option<(usize, usize)>,
    io_specification_stack: Vec<(usize, usize)>,
    current_data_association: Option<(usize, DataAssociationKind, BpmnDataAssociationSnapshot)>,
}

impl BpmnSnapshotScanState {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn handle_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        is_empty: bool,
    ) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(root_from_event(source, reader, event)?);
        }

        let event_name = event.name();
        let tag = local_name(event_name.as_ref());
        if parent_tag == Some("definitions")
            && self.handle_definitions_start_event(source, reader, event, tag, is_empty)?
        {
            return Ok(());
        }
        if self.handle_bpmn_di_start_event(source, reader, event, parent_tag, tag, is_empty)? {
            return Ok(());
        }
        if self
            .handle_collaboration_start_event(source, reader, event, parent_tag, tag, is_empty)?
        {
            return Ok(());
        }

        match tag {
            "operation" if self.current_interface.is_some() => {
                self.start_operation(source, reader, event, is_empty)
            }
            "resourceParameter" if self.current_resource.is_some() => {
                self.capture_resource_parameter(source, reader, event)
            }
            "categoryValue" if self.current_category.is_some() => {
                self.capture_category_value(source, reader, event)
            }
            "correlationPropertyRetrievalExpression"
                if self.current_correlation_property.is_some() =>
            {
                self.start_correlation_retrieval_expression(source, reader, event, is_empty)
            }
            "documentation"
                if parent_tag == Some("extension") && self.current_extension.is_some() =>
            {
                self.start_extension_documentation(is_empty);
                Ok(())
            }
            "laneSet" if self.current_process.is_some() => {
                self.start_lane_set(source, reader, event, is_empty)
            }
            "lane" if self.current_lane_set().is_some() => {
                self.start_lane(source, reader, event, is_empty)
            }
            "dataObject" if self.current_process.is_some() => {
                self.capture_data_object(source, reader, event)
            }
            "dataObjectReference" if self.current_process.is_some() => {
                self.capture_data_object_reference(source, reader, event)
            }
            "dataStoreReference" if self.current_process.is_some() => {
                self.capture_data_store_reference(source, reader, event)
            }
            "ioSpecification" if self.current_process.is_some() => {
                self.start_io_specification(source, reader, event, is_empty)
            }
            "dataInput" if self.current_io_specification().is_some() => {
                self.capture_io_data_input(source, reader, event)
            }
            "dataOutput" if self.current_io_specification().is_some() => {
                self.capture_io_data_output(source, reader, event)
            }
            "dataInputAssociation" if self.current_process.is_some() => self
                .start_data_association(
                    source,
                    reader,
                    event,
                    DataAssociationKind::Input,
                    is_empty,
                ),
            "dataOutputAssociation" if self.current_process.is_some() => self
                .start_data_association(
                    source,
                    reader,
                    event,
                    DataAssociationKind::Output,
                    is_empty,
                ),
            "association"
                if self.current_process.is_some() && is_artifact_container(parent_tag) =>
            {
                self.capture_artifact_association(source, reader, event)
            }
            "group" if self.current_process.is_some() && is_artifact_container(parent_tag) => {
                self.capture_artifact_group(source, reader, event)
            }
            "textAnnotation"
                if self.current_process.is_some() && is_artifact_container(parent_tag) =>
            {
                self.start_text_annotation(source, reader, event, is_empty)
            }
            _ => Ok(()),
        }
    }

    fn handle_bpmn_di_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            "BPMNPlane" if self.current_diagram.is_some() => {
                self.start_bpmn_plane(source, reader, event, is_empty)?;
            }
            "BPMNShape" if self.current_plane.is_some() => {
                self.start_bpmn_shape(source, reader, event, is_empty)?;
            }
            "BPMNEdge" if self.current_plane.is_some() => {
                self.start_bpmn_edge(source, reader, event, is_empty)?;
            }
            "BPMNLabel" if parent_tag == Some("BPMNShape") && self.current_shape.is_some() => {
                self.start_bpmn_shape_label(source, reader, event, is_empty)?;
            }
            "BPMNLabel" if parent_tag == Some("BPMNEdge") && self.current_edge.is_some() => {
                self.start_bpmn_edge_label(source, reader, event, is_empty)?;
            }
            "BPMNLabelStyle" if self.current_diagram.is_some() => {
                self.start_bpmn_label_style(source, reader, event, is_empty)?;
            }
            "Bounds" if parent_tag == Some("BPMNShape") && self.current_shape.is_some() => {
                self.attach_bpmn_shape_bounds(source, reader, event)?;
            }
            "Bounds" if parent_tag == Some("BPMNLabel") && self.current_label.is_some() => {
                self.attach_bpmn_label_bounds(source, reader, event)?;
            }
            "waypoint" if self.current_edge.is_some() => {
                self.push_bpmn_edge_waypoint(source, reader, event)?;
            }
            "Font"
                if parent_tag == Some("BPMNLabelStyle") && self.current_label_style.is_some() =>
            {
                self.attach_bpmn_label_style_font(source, reader, event)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn handle_collaboration_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            "participant" if is_collaboration_container(parent_tag) => {
                self.capture_participant(source, reader, event)?;
            }
            "messageFlow" if is_collaboration_container(parent_tag) => {
                self.capture_message_flow(source, reader, event)?;
            }
            tag if is_conversation_node_tag(tag) && self.current_collaboration.is_some() => {
                self.start_conversation_node(source, reader, event, tag, is_empty)?;
            }
            tag if is_choreography_activity_tag(tag) && self.current_collaboration.is_some() => {
                self.start_choreography_activity(source, reader, event, tag, is_empty)?;
            }
            "conversationAssociation" if is_collaboration_container(parent_tag) => {
                self.capture_conversation_association(source, reader, event)?;
            }
            "participantAssociation"
                if self.current_collaboration_metadata_owner().is_some()
                    && (is_collaboration_container(parent_tag)
                        || parent_tag == Some("callConversation")
                        || parent_tag == Some("callChoreography")) =>
            {
                self.start_participant_association(source, reader, event, is_empty)?;
            }
            "messageFlowAssociation" if is_collaboration_container(parent_tag) => {
                self.capture_message_flow_association(source, reader, event)?;
            }
            "correlationKey" if self.current_collaboration_metadata_owner().is_some() => {
                self.start_conversation_correlation_key(source, reader, event, is_empty)?;
            }
            "conversationLink" if is_collaboration_container(parent_tag) => {
                self.capture_conversation_link(source, reader, event)?;
            }
            "association" if is_artifact_container(parent_tag) => {
                self.capture_artifact_association(source, reader, event)?;
            }
            "group" if is_artifact_container(parent_tag) => {
                self.capture_artifact_group(source, reader, event)?;
            }
            "textAnnotation" if is_artifact_container(parent_tag) => {
                self.start_text_annotation(source, reader, event, is_empty)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn handle_definitions_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            "import" => self.capture_import(source, reader, event)?,
            "extension" => self.start_extension(source, reader, event, is_empty)?,
            "BPMNDiagram" => self.start_bpmn_diagram(source, reader, event, is_empty)?,
            "relationship" => self.start_relationship(source, reader, event, is_empty)?,
            "collaboration" | "globalConversation" | "choreography" | "globalChoreographyTask" => {
                self.start_collaboration(source, reader, event, tag, is_empty)?;
            }
            "process" => self.start_process(source, reader, event, is_empty)?,
            "itemDefinition" => self.capture_item_definition(source, reader, event)?,
            "message" => self.capture_message(source, reader, event)?,
            "interface" => self.start_interface(source, reader, event, is_empty)?,
            "resource" => self.start_resource(source, reader, event, is_empty)?,
            "category" => self.start_category(source, reader, event, is_empty)?,
            "correlationProperty" => {
                self.capture_correlation_property(source, reader, event, is_empty)?;
            }
            "error" => self.capture_error(source, reader, event)?,
            "escalation" => self.capture_escalation(source, reader, event)?,
            "signal" => self.capture_signal(source, reader, event)?,
            "dataStore" => self.capture_data_store(source, reader, event)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(super) fn finish_end_event(&mut self, tag: &str) {
        match tag {
            "collaboration" | "globalConversation" | "choreography" | "globalChoreographyTask" => {
                self.finish_conversation_correlation_key();
                self.finish_participant_association();
                self.conversation_node_stack.clear();
                self.choreography_activity_stack.clear();
                self.current_collaboration = None;
            }
            "conversation" | "subConversation" | "callConversation" => {
                self.finish_conversation_correlation_key();
                self.finish_participant_association();
                let _ = self.conversation_node_stack.pop();
            }
            "choreographyTask" | "subChoreography" | "callChoreography" => {
                self.finish_conversation_correlation_key();
                self.finish_participant_association();
                let _ = self.choreography_activity_stack.pop();
            }
            "textAnnotation" => self.finish_text_annotation(),
            "correlationKey" => self.finish_conversation_correlation_key(),
            "participantAssociation" => self.finish_participant_association(),
            "correlationProperty" => {
                self.finish_correlation_retrieval_expression();
                self.current_correlation_property = None;
            }
            "correlationPropertyRetrievalExpression" => {
                self.finish_correlation_retrieval_expression();
            }
            "operation" => self.current_operation = None,
            "interface" => {
                self.current_operation = None;
                self.current_interface = None;
            }
            "documentation" => self.finish_extension_documentation(),
            "extension" => {
                self.finish_extension_documentation();
                self.current_extension = None;
            }
            "BPMNLabel" => self.current_label = None,
            "BPMNLabelStyle" => self.current_label_style = None,
            "BPMNShape" => self.current_shape = None,
            "BPMNEdge" => self.current_edge = None,
            "BPMNPlane" => {
                self.current_label = None;
                self.current_shape = None;
                self.current_edge = None;
                self.current_plane = None;
            }
            "BPMNDiagram" => {
                self.current_label = None;
                self.current_label_style = None;
                self.current_shape = None;
                self.current_edge = None;
                self.current_plane = None;
                self.current_diagram = None;
            }
            "resource" => self.current_resource = None,
            "category" => self.current_category = None,
            "relationship" => self.current_relationship = None,
            "process" => {
                self.current_process = None;
                self.lane_set_stack.clear();
                self.lane_stack.clear();
                self.io_specification_stack.clear();
            }
            "laneSet" => {
                let _ = self.lane_set_stack.pop();
            }
            "lane" => {
                let _ = self.lane_stack.pop();
            }
            "ioSpecification" => {
                let _ = self.io_specification_stack.pop();
            }
            "dataInputAssociation" => self.finish_data_association(DataAssociationKind::Input),
            "dataOutputAssociation" => self.finish_data_association(DataAssociationKind::Output),
            _ => {}
        }
    }

    pub(super) fn handle_text_chunk(&mut self, text: &str, target: Option<TextTarget>) {
        let Some(target) = target else {
            return;
        };
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        match target {
            TextTarget::LaneFlowNode => self.push_lane_flow_node_ref(text),
            TextTarget::DataAssociationSource => self.push_data_association_source_ref(text),
            TextTarget::DataAssociationTarget => self.set_data_association_target_ref(text),
            TextTarget::CorrelationMessagePath => self.append_correlation_message_path(text),
            TextTarget::OperationInMessageRef => self.set_operation_in_message_ref(text),
            TextTarget::OperationOutMessageRef => self.set_operation_out_message_ref(text),
            TextTarget::OperationErrorRef => self.push_operation_error_ref(text),
            TextTarget::ExtensionDocumentation => self.append_extension_documentation(text),
            TextTarget::RelationshipSource => self.push_relationship_source_ref(text),
            TextTarget::RelationshipTarget => self.push_relationship_target_ref(text),
            TextTarget::ConversationParticipantRef => self.push_conversation_participant_ref(text),
            TextTarget::ConversationMessageFlowRef => self.push_conversation_message_flow_ref(text),
            TextTarget::ChoreographyParticipantRef => {
                self.push_choreography_participant_ref(text);
            }
            TextTarget::ChoreographyMessageFlowRef => {
                self.push_choreography_message_flow_ref(text);
            }
            TextTarget::TextAnnotationText => self.append_text_annotation_text(text),
            TextTarget::CorrelationKeyPropertyRef => {
                self.push_conversation_correlation_property_ref(text);
            }
            TextTarget::ParticipantAssociationInnerRef => {
                self.set_participant_association_inner_ref(text);
            }
            TextTarget::ParticipantAssociationOuterRef => {
                self.set_participant_association_outer_ref(text);
            }
            TextTarget::CollaborationChoreographyRef => self.push_choreography_ref(text),
        }
    }

    pub(super) fn finish_pending(&mut self) {
        if self
            .current_data_association
            .as_ref()
            .is_some_and(|(_, kind, _)| *kind == DataAssociationKind::Input)
        {
            self.finish_data_association(DataAssociationKind::Input);
        }
        if self.current_data_association.is_some() {
            self.finish_data_association(DataAssociationKind::Output);
        }
        self.finish_correlation_retrieval_expression();
        self.current_operation = None;
        self.current_interface = None;
        self.current_resource = None;
        self.current_category = None;
        self.finish_extension_documentation();
        self.current_extension = None;
        self.finish_text_annotation();
        self.finish_conversation_correlation_key();
        self.finish_participant_association();
        self.conversation_node_stack.clear();
        self.choreography_activity_stack.clear();
        self.current_relationship = None;
        self.current_label = None;
        self.current_label_style = None;
        self.current_shape = None;
        self.current_edge = None;
        self.current_plane = None;
        self.current_diagram = None;
    }

    pub(super) fn into_snapshot(self, source: &BpmnSourceFile) -> BpmnDocumentSnapshot {
        BpmnDocumentSnapshot {
            source_id: source.source_id.clone(),
            root: self
                .root
                .unwrap_or_else(|| crate::bpmn_model_api::empty_bpmn_root_snapshot(source)),
            collaborations: self.collaborations,
            processes: self.processes,
        }
    }

    fn start_collaboration(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let collaboration = BpmnCollaborationSnapshot {
            collaboration_kind: tag.to_string(),
            collaboration_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            is_closed: boolean_attribute_value(source, reader, event, "isClosed")?,
            initiating_participant_ref: attribute_value(
                source,
                reader,
                event,
                "initiatingParticipantRef",
            )?,
            participants: Vec::new(),
            message_flows: Vec::new(),
            conversation_nodes: Vec::new(),
            conversation_associations: Vec::new(),
            participant_associations: Vec::new(),
            message_flow_associations: Vec::new(),
            correlation_keys: Vec::new(),
            choreography_refs: Vec::new(),
            choreography_activities: Vec::new(),
            conversation_links: Vec::new(),
            associations: Vec::new(),
            groups: Vec::new(),
            text_annotations: Vec::new(),
        };
        self.collaborations.push(collaboration);
        if let Some(root) = self.root.as_mut() {
            root.collaboration_count += 1;
        }
        if !is_empty {
            self.current_collaboration = self.collaborations.len().checked_sub(1);
        }
        Ok(())
    }

    fn capture_participant(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration.participants.push(BpmnParticipantSnapshot {
            participant_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            process_ref: attribute_value(source, reader, event, "processRef")?,
        });
        Ok(())
    }

    fn capture_message_flow(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration.message_flows.push(BpmnMessageFlowSnapshot {
            message_flow_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            source_ref: attribute_value(source, reader, event, "sourceRef")?,
            target_ref: attribute_value(source, reader, event, "targetRef")?,
            message_ref: attribute_value(source, reader, event, "messageRef")?,
        });
        Ok(())
    }

    fn start_conversation_node(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(collaboration_index) = self.current_collaboration else {
            return Ok(());
        };
        let node = BpmnConversationNodeSnapshot {
            node_kind: tag.to_string(),
            node_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            called_collaboration_ref: attribute_value(
                source,
                reader,
                event,
                "calledCollaborationRef",
            )?,
            participant_refs: Vec::new(),
            message_flow_refs: Vec::new(),
            correlation_keys: Vec::new(),
            participant_associations: Vec::new(),
            child_nodes: Vec::new(),
        };
        let path = self.push_conversation_node(collaboration_index, node);
        if !is_empty {
            self.conversation_node_stack
                .push((collaboration_index, path));
        }
        Ok(())
    }

    fn start_choreography_activity(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(collaboration_index) = self.current_collaboration else {
            return Ok(());
        };
        let activity = BpmnChoreographyActivitySnapshot {
            activity_kind: tag.to_string(),
            activity_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            initiating_participant_ref: attribute_value(
                source,
                reader,
                event,
                "initiatingParticipantRef",
            )?,
            loop_type: attribute_value(source, reader, event, "loopType")?,
            called_choreography_ref: attribute_value(
                source,
                reader,
                event,
                "calledChoreographyRef",
            )?,
            participant_refs: Vec::new(),
            message_flow_refs: Vec::new(),
            correlation_keys: Vec::new(),
            participant_associations: Vec::new(),
            child_activities: Vec::new(),
        };
        let path = self.push_choreography_activity(collaboration_index, activity);
        if !is_empty {
            self.choreography_activity_stack
                .push((collaboration_index, path));
        }
        Ok(())
    }

    fn capture_conversation_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration
            .conversation_associations
            .push(BpmnConversationAssociationSnapshot {
                association_id: attribute_value(source, reader, event, "id")?,
                inner_conversation_node_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "innerConversationNodeRef",
                )?,
                outer_conversation_node_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "outerConversationNodeRef",
                )?,
            });
        Ok(())
    }

    fn start_participant_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_collaboration_metadata_owner() else {
            return Ok(());
        };
        let association = BpmnParticipantAssociationSnapshot {
            association_id: attribute_value(source, reader, event, "id")?,
            inner_participant_ref: None,
            outer_participant_ref: None,
        };
        if is_empty {
            self.push_participant_association(owner, association);
            return Ok(());
        }
        self.current_participant_association = Some((owner, association));
        Ok(())
    }

    fn capture_message_flow_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration
            .message_flow_associations
            .push(BpmnMessageFlowAssociationSnapshot {
                association_id: attribute_value(source, reader, event, "id")?,
                inner_message_flow_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "innerMessageFlowRef",
                )?,
                outer_message_flow_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "outerMessageFlowRef",
                )?,
            });
        Ok(())
    }

    fn start_conversation_correlation_key(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_collaboration_metadata_owner() else {
            return Ok(());
        };
        let key = BpmnCorrelationKeySnapshot {
            correlation_key_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            correlation_property_refs: Vec::new(),
        };
        if is_empty {
            self.push_conversation_correlation_key(owner, key);
            return Ok(());
        }
        self.current_conversation_correlation_key = Some((owner, key));
        Ok(())
    }

    fn capture_conversation_link(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration
            .conversation_links
            .push(BpmnConversationLinkSnapshot {
                link_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                source_ref: attribute_value(source, reader, event, "sourceRef")?,
                target_ref: attribute_value(source, reader, event, "targetRef")?,
            });
        Ok(())
    }

    fn capture_artifact_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(owner) = self.current_artifact_owner() else {
            return Ok(());
        };
        let association = BpmnAssociationSnapshot {
            association_id: attribute_value(source, reader, event, "id")?,
            source_ref: attribute_value(source, reader, event, "sourceRef")?,
            target_ref: attribute_value(source, reader, event, "targetRef")?,
            association_direction: attribute_value(source, reader, event, "associationDirection")?,
        };
        self.push_artifact_association(owner, association);
        Ok(())
    }

    fn capture_artifact_group(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(owner) = self.current_artifact_owner() else {
            return Ok(());
        };
        let group = BpmnGroupSnapshot {
            group_id: attribute_value(source, reader, event, "id")?,
            category_value_ref: attribute_value(source, reader, event, "categoryValueRef")?,
        };
        self.push_artifact_group(owner, group);
        Ok(())
    }

    fn start_text_annotation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_artifact_owner() else {
            return Ok(());
        };
        let annotation = BpmnTextAnnotationSnapshot {
            annotation_id: attribute_value(source, reader, event, "id")?,
            text_format: attribute_value(source, reader, event, "textFormat")?,
            text: None,
        };
        if is_empty {
            self.push_text_annotation(owner, annotation);
            return Ok(());
        }
        self.current_text_annotation = Some((owner, annotation));
        Ok(())
    }

    fn capture_message(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.message_count += 1;
        root.messages.push(BpmnMessageSnapshot {
            message_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_ref: attribute_value(source, reader, event, "itemRef")?,
        });
        Ok(())
    }

    fn start_interface(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.interface_count += 1;
        root.interfaces.push(BpmnInterfaceSnapshot {
            interface_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            implementation_ref: attribute_value(source, reader, event, "implementationRef")?,
            operations: Vec::new(),
        });
        if !is_empty {
            self.current_interface = root.interfaces.len().checked_sub(1);
        }
        Ok(())
    }

    fn start_operation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(interface_index) = self.current_interface else {
            return Ok(());
        };
        let operation = BpmnOperationSnapshot {
            operation_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            implementation_ref: attribute_value(source, reader, event, "implementationRef")?,
            in_message_ref: None,
            out_message_ref: None,
            error_refs: Vec::new(),
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(interface) = root.interfaces.get_mut(interface_index) else {
            return Ok(());
        };
        interface.operations.push(operation);
        if !is_empty {
            let operation_index = interface.operations.len().saturating_sub(1);
            self.current_operation = Some((interface_index, operation_index));
        }
        Ok(())
    }

    fn start_resource(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.resource_count += 1;
        root.resources.push(BpmnResourceSnapshot {
            resource_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            resource_parameters: Vec::new(),
        });
        if !is_empty {
            self.current_resource = root.resources.len().checked_sub(1);
        }
        Ok(())
    }

    fn capture_resource_parameter(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(resource_index) = self.current_resource else {
            return Ok(());
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(resource) = root.resources.get_mut(resource_index) else {
            return Ok(());
        };
        resource
            .resource_parameters
            .push(BpmnResourceParameterSnapshot {
                resource_parameter_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                type_ref: attribute_value(source, reader, event, "type")?,
                is_required: boolean_attribute_value(source, reader, event, "isRequired")?,
            });
        Ok(())
    }

    fn start_category(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.category_count += 1;
        root.categories.push(BpmnCategorySnapshot {
            category_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            category_values: Vec::new(),
        });
        if !is_empty {
            self.current_category = root.categories.len().checked_sub(1);
        }
        Ok(())
    }

    fn capture_category_value(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(category_index) = self.current_category else {
            return Ok(());
        };
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        let Some(category) = root.categories.get_mut(category_index) else {
            return Ok(());
        };
        category.category_values.push(BpmnCategoryValueSnapshot {
            category_value_id: attribute_value(source, reader, event, "id")?,
            value: attribute_value(source, reader, event, "value")?,
        });
        Ok(())
    }

    fn capture_item_definition(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.item_definition_count += 1;
        root.item_definitions.push(BpmnItemDefinitionSnapshot {
            item_definition_id: attribute_value(source, reader, event, "id")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
            item_kind: attribute_value(source, reader, event, "itemKind")?,
            is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
        });
        Ok(())
    }

    fn capture_correlation_property(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.correlation_property_count += 1;
        root.correlation_properties
            .push(BpmnCorrelationPropertySnapshot {
                correlation_property_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                type_ref: attribute_value(source, reader, event, "type")?,
                retrieval_expressions: Vec::new(),
            });
        if !is_empty {
            self.current_correlation_property = root.correlation_properties.len().checked_sub(1);
        }
        Ok(())
    }

    fn start_correlation_retrieval_expression(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(property_index) = self.current_correlation_property else {
            return Ok(());
        };
        let retrieval_expression = BpmnCorrelationRetrievalExpressionSnapshot {
            retrieval_expression_id: attribute_value(source, reader, event, "id")?,
            message_ref: attribute_value(source, reader, event, "messageRef")?,
            message_path: None,
        };
        if is_empty {
            self.push_correlation_retrieval_expression(property_index, retrieval_expression);
            return Ok(());
        }
        self.current_correlation_retrieval_expression =
            Some((property_index, retrieval_expression));
        Ok(())
    }

    fn capture_error(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.error_count += 1;
        root.errors.push(BpmnErrorSnapshot {
            error_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            error_code: attribute_value(source, reader, event, "errorCode")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
        });
        Ok(())
    }

    fn capture_escalation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.escalation_count += 1;
        root.escalations.push(BpmnEscalationSnapshot {
            escalation_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            escalation_code: attribute_value(source, reader, event, "escalationCode")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
        });
        Ok(())
    }

    fn capture_signal(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.signal_count += 1;
        root.signals.push(BpmnSignalSnapshot {
            signal_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            structure_ref: attribute_value(source, reader, event, "structureRef")?,
        });
        Ok(())
    }

    fn capture_import(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.import_count += 1;
        root.imports.push(BpmnImportSnapshot {
            namespace: attribute_value(source, reader, event, "namespace")?,
            location: attribute_value(source, reader, event, "location")?,
            import_type: attribute_value(source, reader, event, "importType")?,
        });
        Ok(())
    }

    fn start_extension(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.extension_count += 1;
        root.extensions.push(BpmnExtensionSnapshot {
            definition: attribute_value(source, reader, event, "definition")?,
            must_understand: boolean_attribute_value(source, reader, event, "mustUnderstand")?
                .unwrap_or(false),
            documentation: Vec::new(),
        });
        if !is_empty {
            self.current_extension = root.extensions.len().checked_sub(1);
        }
        Ok(())
    }

    fn start_extension_documentation(&mut self, is_empty: bool) {
        if is_empty {
            return;
        }
        let Some(extension_index) = self.current_extension else {
            return;
        };
        self.current_extension_documentation = Some((extension_index, String::new()));
    }

    fn start_bpmn_diagram(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.diagram_count += 1;
        root.diagrams.push(BpmnDiagramSnapshot {
            diagram_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            documentation: attribute_value(source, reader, event, "documentation")?,
            resolution: attribute_value(source, reader, event, "resolution")?,
            plane: None,
            label_styles: Vec::new(),
        });
        if !is_empty {
            self.current_diagram = root.diagrams.len().checked_sub(1);
        }
        Ok(())
    }

    fn start_bpmn_plane(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_diagram else {
            return Ok(());
        };
        let Some(diagram) = self.diagram_mut(diagram_index) else {
            return Ok(());
        };
        diagram.plane = Some(BpmnPlaneSnapshot {
            plane_id: attribute_value(source, reader, event, "id")?,
            bpmn_element: attribute_value(source, reader, event, "bpmnElement")?,
            shapes: Vec::new(),
            edges: Vec::new(),
        });
        if !is_empty {
            self.current_plane = Some(diagram_index);
        }
        Ok(())
    }

    fn start_bpmn_shape(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_plane else {
            return Ok(());
        };
        let shape = BpmnShapeSnapshot {
            shape_id: attribute_value(source, reader, event, "id")?,
            bpmn_element: attribute_value(source, reader, event, "bpmnElement")?,
            is_horizontal: boolean_attribute_value(source, reader, event, "isHorizontal")?,
            is_expanded: boolean_attribute_value(source, reader, event, "isExpanded")?,
            is_marker_visible: boolean_attribute_value(source, reader, event, "isMarkerVisible")?,
            is_message_visible: boolean_attribute_value(source, reader, event, "isMessageVisible")?,
            participant_band_kind: attribute_value(source, reader, event, "participantBandKind")?,
            choreography_activity_shape: attribute_value(
                source,
                reader,
                event,
                "choreographyActivityShape",
            )?,
            bounds: None,
            label: None,
        };
        let Some(plane) = self.diagram_plane_mut(diagram_index) else {
            return Ok(());
        };
        plane.shapes.push(shape);
        let shape_index = plane.shapes.len().saturating_sub(1);
        if !is_empty {
            self.current_shape = Some((diagram_index, shape_index));
        }
        Ok(())
    }

    fn start_bpmn_edge(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_plane else {
            return Ok(());
        };
        let edge = BpmnEdgeSnapshot {
            edge_id: attribute_value(source, reader, event, "id")?,
            bpmn_element: attribute_value(source, reader, event, "bpmnElement")?,
            source_element: attribute_value(source, reader, event, "sourceElement")?,
            target_element: attribute_value(source, reader, event, "targetElement")?,
            message_visible_kind: attribute_value(source, reader, event, "messageVisibleKind")?,
            waypoints: Vec::new(),
            label: None,
        };
        let Some(plane) = self.diagram_plane_mut(diagram_index) else {
            return Ok(());
        };
        plane.edges.push(edge);
        let edge_index = plane.edges.len().saturating_sub(1);
        if !is_empty {
            self.current_edge = Some((diagram_index, edge_index));
        }
        Ok(())
    }

    fn start_bpmn_shape_label(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((diagram_index, shape_index)) = self.current_shape else {
            return Ok(());
        };
        let Some(shape) = self.diagram_shape_mut(diagram_index, shape_index) else {
            return Ok(());
        };
        shape.label = Some(label_from_event(source, reader, event)?);
        if !is_empty {
            self.current_label = Some(BpmnDiLabelTarget::Shape(diagram_index, shape_index));
        }
        Ok(())
    }

    fn start_bpmn_edge_label(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((diagram_index, edge_index)) = self.current_edge else {
            return Ok(());
        };
        let Some(edge) = self.diagram_edge_mut(diagram_index, edge_index) else {
            return Ok(());
        };
        edge.label = Some(label_from_event(source, reader, event)?);
        if !is_empty {
            self.current_label = Some(BpmnDiLabelTarget::Edge(diagram_index, edge_index));
        }
        Ok(())
    }

    fn start_bpmn_label_style(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_diagram else {
            return Ok(());
        };
        let Some(diagram) = self.diagram_mut(diagram_index) else {
            return Ok(());
        };
        diagram.label_styles.push(BpmnLabelStyleSnapshot {
            style_id: attribute_value(source, reader, event, "id")?,
            font: None,
        });
        let style_index = diagram.label_styles.len().saturating_sub(1);
        if !is_empty {
            self.current_label_style = Some((diagram_index, style_index));
        }
        Ok(())
    }

    fn attach_bpmn_shape_bounds(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((diagram_index, shape_index)) = self.current_shape else {
            return Ok(());
        };
        let Some(shape) = self.diagram_shape_mut(diagram_index, shape_index) else {
            return Ok(());
        };
        shape.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    fn attach_bpmn_label_bounds(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(target) = self.current_label else {
            return Ok(());
        };
        let Some(label) = self.diagram_label_mut(target) else {
            return Ok(());
        };
        label.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    fn push_bpmn_edge_waypoint(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((diagram_index, edge_index)) = self.current_edge else {
            return Ok(());
        };
        let Some(edge) = self.diagram_edge_mut(diagram_index, edge_index) else {
            return Ok(());
        };
        edge.waypoints
            .push(waypoint_from_event(source, reader, event)?);
        Ok(())
    }

    fn attach_bpmn_label_style_font(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((diagram_index, style_index)) = self.current_label_style else {
            return Ok(());
        };
        let Some(style) = self.diagram_label_style_mut(diagram_index, style_index) else {
            return Ok(());
        };
        style.font = Some(font_from_event(source, reader, event)?);
        Ok(())
    }

    fn start_relationship(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.relationship_count += 1;
        root.relationships.push(BpmnRelationshipSnapshot {
            relationship_id: attribute_value(source, reader, event, "id")?,
            relationship_type: attribute_value(source, reader, event, "type")?,
            direction: attribute_value(source, reader, event, "direction")?,
            source_refs: Vec::new(),
            target_refs: Vec::new(),
        });
        if !is_empty {
            self.current_relationship = root.relationships.len().checked_sub(1);
        }
        Ok(())
    }

    fn start_process(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let process = BpmnProcessSnapshot {
            process_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            is_executable: boolean_attribute_value(source, reader, event, "isExecutable")?,
            lane_set_count: 0,
            lane_sets: Vec::new(),
            data_object_count: 0,
            data_objects: Vec::new(),
            data_object_reference_count: 0,
            data_object_references: Vec::new(),
            data_store_reference_count: 0,
            data_store_references: Vec::new(),
            io_specification_count: 0,
            io_specifications: Vec::new(),
            data_input_association_count: 0,
            data_input_associations: Vec::new(),
            data_output_association_count: 0,
            data_output_associations: Vec::new(),
            association_count: 0,
            associations: Vec::new(),
            group_count: 0,
            groups: Vec::new(),
            text_annotation_count: 0,
            text_annotations: Vec::new(),
        };
        self.processes.push(process);
        if let Some(root) = self.root.as_mut() {
            root.process_count += 1;
        }
        if !is_empty {
            self.current_process = self.processes.len().checked_sub(1);
        }
        Ok(())
    }

    fn capture_data_store(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.data_store_count += 1;
        root.data_stores.push(BpmnDataStoreSnapshot {
            data_store_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
            capacity: attribute_value(source, reader, event, "capacity")?,
            is_unlimited: boolean_attribute_value(source, reader, event, "isUnlimited")?,
        });
        Ok(())
    }

    fn start_lane_set(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let lane_set = BpmnLaneSetSnapshot {
            lane_set_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            lanes: Vec::new(),
        };
        let Some(process) = self.processes.get_mut(process_index) else {
            return Ok(());
        };
        process.lane_set_count += 1;
        process.lane_sets.push(lane_set);
        if !is_empty {
            let lane_set_index = process.lane_sets.len().saturating_sub(1);
            self.lane_set_stack.push((process_index, lane_set_index));
        }
        Ok(())
    }

    fn start_lane(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((process_index, lane_set_index)) = self.current_lane_set() else {
            return Ok(());
        };
        let lane = BpmnLaneSnapshot {
            lane_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            flow_node_refs: Vec::new(),
        };
        let Some(lane_set) = self.lane_set_mut(process_index, lane_set_index) else {
            return Ok(());
        };
        lane_set.lanes.push(lane);
        if !is_empty {
            let lane_index = lane_set.lanes.len().saturating_sub(1);
            self.lane_stack
                .push((process_index, lane_set_index, lane_index));
        }
        Ok(())
    }

    fn capture_data_object(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_object_count += 1;
        process.data_objects.push(BpmnDataObjectSnapshot {
            data_object_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
            is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
        });
        Ok(())
    }

    fn capture_data_object_reference(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_object_reference_count += 1;
        process
            .data_object_references
            .push(BpmnDataObjectReferenceSnapshot {
                data_object_reference_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                data_object_ref: attribute_value(source, reader, event, "dataObjectRef")?,
            });
        Ok(())
    }

    fn capture_data_store_reference(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(process) = self.current_process_mut() else {
            return Ok(());
        };
        process.data_store_reference_count += 1;
        process
            .data_store_references
            .push(BpmnDataStoreReferenceSnapshot {
                data_store_reference_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                data_store_ref: attribute_value(source, reader, event, "dataStoreRef")?,
            });
        Ok(())
    }

    fn start_io_specification(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let Some(process) = self.processes.get_mut(process_index) else {
            return Ok(());
        };
        process.io_specification_count += 1;
        process.io_specifications.push(BpmnIoSpecificationSnapshot {
            io_specification_id: attribute_value(source, reader, event, "id")?,
            data_inputs: Vec::new(),
            data_outputs: Vec::new(),
        });
        if !is_empty {
            let io_index = process.io_specifications.len().saturating_sub(1);
            self.io_specification_stack.push((process_index, io_index));
        }
        Ok(())
    }

    fn capture_io_data_input(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification
            .data_inputs
            .push(data_input_output_from_event(source, reader, event)?);
        Ok(())
    }

    fn capture_io_data_output(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(io_specification) = self.current_io_specification_mut() else {
            return Ok(());
        };
        io_specification
            .data_outputs
            .push(data_input_output_from_event(source, reader, event)?);
        Ok(())
    }

    fn start_data_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        kind: DataAssociationKind,
        is_empty: bool,
    ) -> Result<()> {
        let Some(process_index) = self.current_process else {
            return Ok(());
        };
        let association = BpmnDataAssociationSnapshot {
            association_id: attribute_value(source, reader, event, "id")?,
            source_refs: Vec::new(),
            target_ref: None,
        };
        if is_empty {
            self.push_data_association(process_index, kind, association);
            return Ok(());
        }
        self.current_data_association = Some((process_index, kind, association));
        Ok(())
    }

    fn push_lane_flow_node_ref(&mut self, text: &str) {
        let Some((process_index, lane_set_index, lane_index)) = self.lane_stack.last().copied()
        else {
            return;
        };
        let Some(lane_set) = self.lane_set_mut(process_index, lane_set_index) else {
            return;
        };
        let Some(lane) = lane_set.lanes.get_mut(lane_index) else {
            return;
        };
        lane.flow_node_refs.push(text.to_string());
    }

    fn push_data_association_source_ref(&mut self, text: &str) {
        let Some((_, _, association)) = self.current_data_association.as_mut() else {
            return;
        };
        association.source_refs.push(text.to_string());
    }

    fn set_data_association_target_ref(&mut self, text: &str) {
        let Some((_, _, association)) = self.current_data_association.as_mut() else {
            return;
        };
        association.target_ref = Some(text.to_string());
    }

    fn append_correlation_message_path(&mut self, text: &str) {
        let Some((_, retrieval_expression)) =
            self.current_correlation_retrieval_expression.as_mut()
        else {
            return;
        };
        retrieval_expression
            .message_path
            .get_or_insert_with(String::new)
            .push_str(text);
    }

    fn set_operation_in_message_ref(&mut self, text: &str) {
        let Some(operation) = self.current_operation_mut() else {
            return;
        };
        operation.in_message_ref = Some(text.to_string());
    }

    fn set_operation_out_message_ref(&mut self, text: &str) {
        let Some(operation) = self.current_operation_mut() else {
            return;
        };
        operation.out_message_ref = Some(text.to_string());
    }

    fn push_operation_error_ref(&mut self, text: &str) {
        let Some(operation) = self.current_operation_mut() else {
            return;
        };
        operation.error_refs.push(text.to_string());
    }

    fn push_relationship_source_ref(&mut self, text: &str) {
        let Some(relationship) = self.current_relationship_mut() else {
            return;
        };
        relationship.source_refs.push(text.to_string());
    }

    fn push_relationship_target_ref(&mut self, text: &str) {
        let Some(relationship) = self.current_relationship_mut() else {
            return;
        };
        relationship.target_refs.push(text.to_string());
    }

    fn push_conversation_participant_ref(&mut self, text: &str) {
        let Some(conversation) = self.current_conversation_node_mut() else {
            return;
        };
        conversation.participant_refs.push(text.to_string());
    }

    fn push_conversation_message_flow_ref(&mut self, text: &str) {
        let Some(conversation) = self.current_conversation_node_mut() else {
            return;
        };
        conversation.message_flow_refs.push(text.to_string());
    }

    fn push_choreography_participant_ref(&mut self, text: &str) {
        let Some(activity) = self.current_choreography_activity_mut() else {
            return;
        };
        activity.participant_refs.push(text.to_string());
    }

    fn push_choreography_message_flow_ref(&mut self, text: &str) {
        let Some(activity) = self.current_choreography_activity_mut() else {
            return;
        };
        activity.message_flow_refs.push(text.to_string());
    }

    fn push_conversation_correlation_property_ref(&mut self, text: &str) {
        let Some((_, key)) = self.current_conversation_correlation_key.as_mut() else {
            return;
        };
        key.correlation_property_refs.push(text.to_string());
    }

    fn set_participant_association_inner_ref(&mut self, text: &str) {
        let Some((_, association)) = self.current_participant_association.as_mut() else {
            return;
        };
        association.inner_participant_ref = Some(text.to_string());
    }

    fn set_participant_association_outer_ref(&mut self, text: &str) {
        let Some((_, association)) = self.current_participant_association.as_mut() else {
            return;
        };
        association.outer_participant_ref = Some(text.to_string());
    }

    fn push_choreography_ref(&mut self, text: &str) {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return;
        };
        collaboration.choreography_refs.push(text.to_string());
    }

    fn append_extension_documentation(&mut self, text: &str) {
        let Some((_, documentation)) = self.current_extension_documentation.as_mut() else {
            return;
        };
        if !documentation.is_empty() {
            documentation.push(' ');
        }
        documentation.push_str(text);
    }

    fn append_text_annotation_text(&mut self, text: &str) {
        let Some((_, annotation)) = self.current_text_annotation.as_mut() else {
            return;
        };
        let payload = annotation.text.get_or_insert_with(String::new);
        if !payload.is_empty() {
            payload.push(' ');
        }
        payload.push_str(text);
    }

    fn finish_extension_documentation(&mut self) {
        let Some((extension_index, documentation)) = self.current_extension_documentation.take()
        else {
            return;
        };
        let documentation = documentation.trim();
        if documentation.is_empty() {
            return;
        }
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let Some(extension) = root.extensions.get_mut(extension_index) else {
            return;
        };
        extension.documentation.push(documentation.to_string());
    }

    fn finish_text_annotation(&mut self) {
        let Some((owner, mut annotation)) = self.current_text_annotation.take() else {
            return;
        };
        annotation.text = annotation
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(ToString::to_string);
        self.push_text_annotation(owner, annotation);
    }

    fn finish_correlation_retrieval_expression(&mut self) {
        let Some((property_index, retrieval_expression)) =
            self.current_correlation_retrieval_expression.take()
        else {
            return;
        };
        self.push_correlation_retrieval_expression(property_index, retrieval_expression);
    }

    fn push_correlation_retrieval_expression(
        &mut self,
        property_index: usize,
        retrieval_expression: BpmnCorrelationRetrievalExpressionSnapshot,
    ) {
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let Some(property) = root.correlation_properties.get_mut(property_index) else {
            return;
        };
        property.retrieval_expressions.push(retrieval_expression);
    }

    fn finish_conversation_correlation_key(&mut self) {
        let Some((owner, key)) = self.current_conversation_correlation_key.take() else {
            return;
        };
        self.push_conversation_correlation_key(owner, key);
    }

    fn push_conversation_correlation_key(
        &mut self,
        owner: CollaborationMetadataOwner,
        key: BpmnCorrelationKeySnapshot,
    ) {
        match owner {
            CollaborationMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.correlation_keys.push(key);
            }
            CollaborationMetadataOwner::ConversationNode(collaboration_index, path) => {
                let Some(node) = self.conversation_node_mut(collaboration_index, &path) else {
                    return;
                };
                node.correlation_keys.push(key);
            }
            CollaborationMetadataOwner::ChoreographyActivity(collaboration_index, path) => {
                let Some(activity) = self.choreography_activity_mut(collaboration_index, &path)
                else {
                    return;
                };
                activity.correlation_keys.push(key);
            }
        }
    }

    fn finish_participant_association(&mut self) {
        let Some((owner, association)) = self.current_participant_association.take() else {
            return;
        };
        self.push_participant_association(owner, association);
    }

    fn push_participant_association(
        &mut self,
        owner: CollaborationMetadataOwner,
        association: BpmnParticipantAssociationSnapshot,
    ) {
        match owner {
            CollaborationMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.participant_associations.push(association);
            }
            CollaborationMetadataOwner::ConversationNode(collaboration_index, path) => {
                let Some(node) = self.conversation_node_mut(collaboration_index, &path) else {
                    return;
                };
                node.participant_associations.push(association);
            }
            CollaborationMetadataOwner::ChoreographyActivity(collaboration_index, path) => {
                let Some(activity) = self.choreography_activity_mut(collaboration_index, &path)
                else {
                    return;
                };
                activity.participant_associations.push(association);
            }
        }
    }

    fn finish_data_association(&mut self, expected_kind: DataAssociationKind) {
        let Some((process_index, kind, association)) = self.current_data_association.take() else {
            return;
        };
        if kind != expected_kind {
            self.current_data_association = Some((process_index, kind, association));
            return;
        }
        self.push_data_association(process_index, kind, association);
    }

    fn push_data_association(
        &mut self,
        process_index: usize,
        kind: DataAssociationKind,
        association: BpmnDataAssociationSnapshot,
    ) {
        let Some(process) = self.processes.get_mut(process_index) else {
            return;
        };
        match kind {
            DataAssociationKind::Input => {
                process.data_input_association_count += 1;
                process.data_input_associations.push(association);
            }
            DataAssociationKind::Output => {
                process.data_output_association_count += 1;
                process.data_output_associations.push(association);
            }
        }
    }

    fn current_artifact_owner(&self) -> Option<ArtifactMetadataOwner> {
        if let Some(collaboration_index) = self.current_collaboration {
            return Some(ArtifactMetadataOwner::Collaboration(collaboration_index));
        }
        self.current_process.map(ArtifactMetadataOwner::Process)
    }

    fn push_artifact_association(
        &mut self,
        owner: ArtifactMetadataOwner,
        association: BpmnAssociationSnapshot,
    ) {
        match owner {
            ArtifactMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.associations.push(association);
            }
            ArtifactMetadataOwner::Process(process_index) => {
                let Some(process) = self.processes.get_mut(process_index) else {
                    return;
                };
                process.association_count += 1;
                process.associations.push(association);
            }
        }
    }

    fn push_artifact_group(&mut self, owner: ArtifactMetadataOwner, group: BpmnGroupSnapshot) {
        match owner {
            ArtifactMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.groups.push(group);
            }
            ArtifactMetadataOwner::Process(process_index) => {
                let Some(process) = self.processes.get_mut(process_index) else {
                    return;
                };
                process.group_count += 1;
                process.groups.push(group);
            }
        }
    }

    fn push_text_annotation(
        &mut self,
        owner: ArtifactMetadataOwner,
        annotation: BpmnTextAnnotationSnapshot,
    ) {
        match owner {
            ArtifactMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.text_annotations.push(annotation);
            }
            ArtifactMetadataOwner::Process(process_index) => {
                let Some(process) = self.processes.get_mut(process_index) else {
                    return;
                };
                process.text_annotation_count += 1;
                process.text_annotations.push(annotation);
            }
        }
    }

    fn current_collaboration_mut(&mut self) -> Option<&mut BpmnCollaborationSnapshot> {
        self.current_collaboration
            .and_then(|index| self.collaborations.get_mut(index))
    }

    fn current_collaboration_metadata_owner(&self) -> Option<CollaborationMetadataOwner> {
        if let Some((collaboration_index, path)) = self.choreography_activity_stack.last() {
            return Some(CollaborationMetadataOwner::ChoreographyActivity(
                *collaboration_index,
                path.clone(),
            ));
        }
        if let Some((collaboration_index, path)) = self.conversation_node_stack.last() {
            return Some(CollaborationMetadataOwner::ConversationNode(
                *collaboration_index,
                path.clone(),
            ));
        }
        self.current_collaboration
            .map(CollaborationMetadataOwner::Collaboration)
    }

    fn push_conversation_node(
        &mut self,
        collaboration_index: usize,
        node: BpmnConversationNodeSnapshot,
    ) -> Vec<usize> {
        let Some((_, parent_path)) = self.conversation_node_stack.last().cloned() else {
            let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                return Vec::new();
            };
            collaboration.conversation_nodes.push(node);
            return vec![collaboration.conversation_nodes.len().saturating_sub(1)];
        };
        let Some(parent) = self.conversation_node_mut(collaboration_index, &parent_path) else {
            return parent_path;
        };
        parent.child_nodes.push(node);
        let mut path = parent_path;
        path.push(parent.child_nodes.len().saturating_sub(1));
        path
    }

    fn push_choreography_activity(
        &mut self,
        collaboration_index: usize,
        activity: BpmnChoreographyActivitySnapshot,
    ) -> Vec<usize> {
        let Some((_, parent_path)) = self.choreography_activity_stack.last().cloned() else {
            let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                return Vec::new();
            };
            collaboration.choreography_activities.push(activity);
            return vec![
                collaboration
                    .choreography_activities
                    .len()
                    .saturating_sub(1),
            ];
        };
        let Some(parent) = self.choreography_activity_mut(collaboration_index, &parent_path) else {
            return parent_path;
        };
        parent.child_activities.push(activity);
        let mut path = parent_path;
        path.push(parent.child_activities.len().saturating_sub(1));
        path
    }

    fn current_conversation_node_mut(&mut self) -> Option<&mut BpmnConversationNodeSnapshot> {
        let (collaboration_index, path) = self.conversation_node_stack.last().cloned()?;
        self.conversation_node_mut(collaboration_index, &path)
    }

    fn current_choreography_activity_mut(
        &mut self,
    ) -> Option<&mut BpmnChoreographyActivitySnapshot> {
        let (collaboration_index, path) = self.choreography_activity_stack.last().cloned()?;
        self.choreography_activity_mut(collaboration_index, &path)
    }

    fn conversation_node_mut(
        &mut self,
        collaboration_index: usize,
        path: &[usize],
    ) -> Option<&mut BpmnConversationNodeSnapshot> {
        let (first, rest) = path.split_first()?;
        let mut node = self
            .collaborations
            .get_mut(collaboration_index)?
            .conversation_nodes
            .get_mut(*first)?;
        for index in rest {
            node = node.child_nodes.get_mut(*index)?;
        }
        Some(node)
    }

    fn choreography_activity_mut(
        &mut self,
        collaboration_index: usize,
        path: &[usize],
    ) -> Option<&mut BpmnChoreographyActivitySnapshot> {
        let (first, rest) = path.split_first()?;
        let mut activity = self
            .collaborations
            .get_mut(collaboration_index)?
            .choreography_activities
            .get_mut(*first)?;
        for index in rest {
            activity = activity.child_activities.get_mut(*index)?;
        }
        Some(activity)
    }

    fn current_process_mut(&mut self) -> Option<&mut BpmnProcessSnapshot> {
        self.current_process
            .and_then(|index| self.processes.get_mut(index))
    }

    fn current_lane_set(&self) -> Option<(usize, usize)> {
        self.lane_set_stack.last().copied()
    }

    fn lane_set_mut(
        &mut self,
        process_index: usize,
        lane_set_index: usize,
    ) -> Option<&mut BpmnLaneSetSnapshot> {
        self.processes
            .get_mut(process_index)?
            .lane_sets
            .get_mut(lane_set_index)
    }

    fn current_io_specification(&self) -> Option<(usize, usize)> {
        self.io_specification_stack.last().copied()
    }

    fn current_io_specification_mut(&mut self) -> Option<&mut BpmnIoSpecificationSnapshot> {
        let (process_index, io_index) = self.current_io_specification()?;
        self.processes
            .get_mut(process_index)?
            .io_specifications
            .get_mut(io_index)
    }

    fn current_operation_mut(&mut self) -> Option<&mut BpmnOperationSnapshot> {
        let (interface_index, operation_index) = self.current_operation?;
        self.root
            .as_mut()?
            .interfaces
            .get_mut(interface_index)?
            .operations
            .get_mut(operation_index)
    }

    fn current_relationship_mut(&mut self) -> Option<&mut BpmnRelationshipSnapshot> {
        let relationship_index = self.current_relationship?;
        self.root
            .as_mut()?
            .relationships
            .get_mut(relationship_index)
    }

    fn diagram_mut(&mut self, diagram_index: usize) -> Option<&mut BpmnDiagramSnapshot> {
        self.root.as_mut()?.diagrams.get_mut(diagram_index)
    }

    fn diagram_plane_mut(&mut self, diagram_index: usize) -> Option<&mut BpmnPlaneSnapshot> {
        self.diagram_mut(diagram_index)?.plane.as_mut()
    }

    fn diagram_shape_mut(
        &mut self,
        diagram_index: usize,
        shape_index: usize,
    ) -> Option<&mut BpmnShapeSnapshot> {
        self.diagram_plane_mut(diagram_index)?
            .shapes
            .get_mut(shape_index)
    }

    fn diagram_edge_mut(
        &mut self,
        diagram_index: usize,
        edge_index: usize,
    ) -> Option<&mut BpmnEdgeSnapshot> {
        self.diagram_plane_mut(diagram_index)?
            .edges
            .get_mut(edge_index)
    }

    fn diagram_label_mut(&mut self, target: BpmnDiLabelTarget) -> Option<&mut BpmnLabelSnapshot> {
        match target {
            BpmnDiLabelTarget::Shape(diagram_index, shape_index) => self
                .diagram_shape_mut(diagram_index, shape_index)?
                .label
                .as_mut(),
            BpmnDiLabelTarget::Edge(diagram_index, edge_index) => self
                .diagram_edge_mut(diagram_index, edge_index)?
                .label
                .as_mut(),
        }
    }

    fn diagram_label_style_mut(
        &mut self,
        diagram_index: usize,
        style_index: usize,
    ) -> Option<&mut BpmnLabelStyleSnapshot> {
        self.diagram_mut(diagram_index)?
            .label_styles
            .get_mut(style_index)
    }
}

fn label_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnLabelSnapshot> {
    Ok(BpmnLabelSnapshot {
        label_id: attribute_value(source, reader, event, "id")?,
        label_style: attribute_value(source, reader, event, "labelStyle")?,
        bounds: None,
    })
}

fn bounds_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnBoundsSnapshot> {
    Ok(BpmnBoundsSnapshot {
        x: attribute_value(source, reader, event, "x")?,
        y: attribute_value(source, reader, event, "y")?,
        width: attribute_value(source, reader, event, "width")?,
        height: attribute_value(source, reader, event, "height")?,
    })
}

fn waypoint_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnWaypointSnapshot> {
    Ok(BpmnWaypointSnapshot {
        x: attribute_value(source, reader, event, "x")?,
        y: attribute_value(source, reader, event, "y")?,
    })
}

fn font_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnFontSnapshot> {
    Ok(BpmnFontSnapshot {
        name: attribute_value(source, reader, event, "name")?,
        size: attribute_value(source, reader, event, "size")?,
        is_bold: boolean_attribute_value(source, reader, event, "isBold")?,
        is_italic: boolean_attribute_value(source, reader, event, "isItalic")?,
        is_underline: boolean_attribute_value(source, reader, event, "isUnderline")?,
        is_strike_through: boolean_attribute_value(source, reader, event, "isStrikeThrough")?,
    })
}

fn root_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnRootSnapshot> {
    let event_name = event.name();
    Ok(BpmnRootSnapshot {
        element_name: local_name(event_name.as_ref()).to_string(),
        definitions_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        target_namespace: attribute_value(source, reader, event, "targetNamespace")?,
        model_namespace_uri: bpmn_model_namespace(source, reader, event)?,
        import_count: 0,
        imports: Vec::new(),
        extension_count: 0,
        extensions: Vec::new(),
        relationship_count: 0,
        relationships: Vec::new(),
        diagram_count: 0,
        diagrams: Vec::new(),
        collaboration_count: 0,
        process_count: 0,
        item_definition_count: 0,
        item_definitions: Vec::new(),
        message_count: 0,
        messages: Vec::new(),
        interface_count: 0,
        interfaces: Vec::new(),
        resource_count: 0,
        resources: Vec::new(),
        category_count: 0,
        categories: Vec::new(),
        correlation_property_count: 0,
        correlation_properties: Vec::new(),
        error_count: 0,
        errors: Vec::new(),
        escalation_count: 0,
        escalations: Vec::new(),
        signal_count: 0,
        signals: Vec::new(),
        data_store_count: 0,
        data_stores: Vec::new(),
    })
}

fn data_input_output_from_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<BpmnDataInputOutputSnapshot> {
    Ok(BpmnDataInputOutputSnapshot {
        data_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        item_subject_ref: attribute_value(source, reader, event, "itemSubjectRef")?,
        is_collection: boolean_attribute_value(source, reader, event, "isCollection")?,
    })
}

fn is_collaboration_container(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some("collaboration" | "globalConversation" | "choreography" | "globalChoreographyTask")
    )
}

fn is_conversation_node_tag(tag: &str) -> bool {
    matches!(tag, "conversation" | "subConversation" | "callConversation")
}

fn is_choreography_activity_tag(tag: &str) -> bool {
    matches!(
        tag,
        "choreographyTask" | "subChoreography" | "callChoreography"
    )
}

fn is_artifact_container(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some(
            "collaboration"
                | "globalConversation"
                | "choreography"
                | "globalChoreographyTask"
                | "process"
                | "subProcess"
                | "transaction"
                | "subChoreography"
        )
    )
}
