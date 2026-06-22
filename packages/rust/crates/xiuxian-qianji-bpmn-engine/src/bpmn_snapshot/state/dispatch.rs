use super::{
    BpmnSnapshotScanState, BpmnSourceFile, BytesStart, DataAssociationAssignmentExpressionKind,
    DataAssociationKind, Reader, Result, is_artifact_container, is_choreography_activity_tag,
    is_collaboration_container, is_conversation_node_tag, is_data_association_tag,
    is_flow_element_metadata_owner_tag, is_global_task_tag, is_resource_role_tag, local_name,
    root_from_event,
};

impl BpmnSnapshotScanState {
    pub(in crate::bpmn_snapshot) fn new() -> Self {
        Self::default()
    }

    pub(in crate::bpmn_snapshot) fn handle_start_event(
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
        self.handle_flow_element_metadata_start_event(
            source, reader, event, parent_tag, tag, is_empty,
        )?;
        if self.handle_process_start_event(source, reader, event, parent_tag, tag, is_empty)? {
            return Ok(());
        }
        if self
            .handle_data_metadata_start_event(source, reader, event, parent_tag, tag, is_empty)?
        {
            return Ok(());
        }
        if self.handle_callable_io_start_event(source, reader, event, parent_tag, tag, is_empty)? {
            return Ok(());
        }
        if self
            .handle_resource_role_start_event(source, reader, event, parent_tag, tag, is_empty)?
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
            "participantMultiplicity"
                if parent_tag == Some("participant") && self.current_participant.is_some() =>
            {
                self.attach_participant_multiplicity(source, reader, event)
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
            _ => Ok(()),
        }
    }

    pub(super) fn handle_flow_element_metadata_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        if parent_tag == Some("process")
            && self.current_process.is_some()
            && is_flow_element_metadata_owner_tag(tag)
        {
            self.start_flow_element_metadata(source, reader, event, tag, is_empty)?;
            return Ok(());
        }
        if self
            .current_flow_element_metadata
            .as_ref()
            .is_some_and(|(_, metadata)| parent_tag == Some(metadata.element_kind.as_str()))
        {
            match tag {
                "auditing" => self.attach_flow_element_auditing(source, reader, event)?,
                "monitoring" => self.attach_flow_element_monitoring(source, reader, event)?,
                "categoryValueRef" if !is_empty => {
                    self.collecting_flow_element_category_value_ref = true;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn handle_process_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            "laneSet" if self.current_process.is_some() => {
                self.start_lane_set(source, reader, event, is_empty)?;
            }
            "lane" if self.current_lane_set().is_some() => {
                self.start_lane(source, reader, event, is_empty)?;
            }
            "property" if parent_tag == Some("process") && self.current_process.is_some() => {
                self.capture_process_property(source, reader, event)?;
            }
            "correlationSubscription"
                if parent_tag == Some("process") && self.current_process.is_some() =>
            {
                self.start_correlation_subscription(source, reader, event, is_empty)?;
            }
            "correlationPropertyBinding"
                if parent_tag == Some("correlationSubscription")
                    && self.current_correlation_subscription.is_some() =>
            {
                self.start_correlation_property_binding(source, reader, event, is_empty)?;
            }
            "dataPath"
                if parent_tag == Some("correlationPropertyBinding")
                    && self.current_correlation_property_binding.is_some() =>
            {
                self.attach_correlation_binding_data_path_metadata(source, reader, event)?;
            }
            "dataObject" if self.current_process.is_some() => {
                self.start_data_object(source, reader, event, is_empty)?;
            }
            "dataObjectReference" if self.current_process.is_some() => {
                self.start_data_object_reference(source, reader, event, is_empty)?;
            }
            "dataStoreReference" if self.current_process.is_some() => {
                self.start_data_store_reference(source, reader, event, is_empty)?;
            }
            "ioSpecification" if self.current_process.is_some() => {
                self.start_io_specification(source, reader, event, is_empty)?;
            }
            "dataInput" if self.current_io_specification().is_some() => {
                self.capture_io_data_input(source, reader, event, is_empty)?;
            }
            "dataOutput" if self.current_io_specification().is_some() => {
                self.capture_io_data_output(source, reader, event, is_empty)?;
            }
            "dataInputAssociation" if self.current_process.is_some() => self
                .start_data_association(
                    source,
                    reader,
                    event,
                    DataAssociationKind::Input,
                    is_empty,
                )?,
            "dataOutputAssociation" if self.current_process.is_some() => self
                .start_data_association(
                    source,
                    reader,
                    event,
                    DataAssociationKind::Output,
                    is_empty,
                )?,
            "association"
                if self.current_process.is_some() && is_artifact_container(parent_tag) =>
            {
                self.capture_artifact_association(source, reader, event)?;
            }
            "group" if self.current_process.is_some() && is_artifact_container(parent_tag) => {
                self.capture_artifact_group(source, reader, event)?;
            }
            "textAnnotation"
                if self.current_process.is_some() && is_artifact_container(parent_tag) =>
            {
                self.start_text_annotation(source, reader, event, is_empty)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(super) fn handle_callable_io_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            "ioSpecification"
                if parent_tag.is_some_and(is_global_task_tag)
                    && self.current_global_task.is_some() =>
            {
                self.start_global_task_io_specification(source, reader, event, is_empty)?;
            }
            "ioBinding" if parent_tag == Some("process") && self.current_process.is_some() => {
                self.capture_process_io_binding(source, reader, event)?;
            }
            "ioBinding"
                if parent_tag.is_some_and(is_global_task_tag)
                    && self.current_global_task.is_some() =>
            {
                self.capture_global_task_io_binding(source, reader, event)?;
            }
            "dataInput" if self.current_io_specification().is_some() => {
                self.capture_io_data_input(source, reader, event, is_empty)?;
            }
            "dataOutput" if self.current_io_specification().is_some() => {
                self.capture_io_data_output(source, reader, event, is_empty)?;
            }
            "inputSet" if self.current_io_specification().is_some() => {
                self.start_io_input_set(source, reader, event, is_empty)?;
            }
            "outputSet" if self.current_io_specification().is_some() => {
                self.start_io_output_set(source, reader, event, is_empty)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(super) fn handle_data_metadata_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            "dataState" if self.current_data_state_owner.is_some() => {
                self.attach_data_state(source, reader, event)?;
            }
            "transformation"
                if parent_tag.is_some_and(is_data_association_tag)
                    && self.current_data_association.is_some() =>
            {
                self.start_data_association_transformation(source, reader, event)?;
            }
            "assignment"
                if parent_tag.is_some_and(is_data_association_tag)
                    && self.current_data_association.is_some() =>
            {
                self.start_data_association_assignment(source, reader, event, is_empty)?;
            }
            "from"
                if parent_tag == Some("assignment")
                    && self.current_data_association_assignment.is_some() =>
            {
                self.start_data_association_assignment_expression(
                    source,
                    reader,
                    event,
                    DataAssociationAssignmentExpressionKind::From,
                )?;
            }
            "to" if parent_tag == Some("assignment")
                && self.current_data_association_assignment.is_some() =>
            {
                self.start_data_association_assignment_expression(
                    source,
                    reader,
                    event,
                    DataAssociationAssignmentExpressionKind::To,
                )?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(super) fn handle_resource_role_start_event(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        parent_tag: Option<&str>,
        tag: &str,
        is_empty: bool,
    ) -> Result<bool> {
        match tag {
            tag if is_resource_role_tag(tag) && parent_tag == Some("process") => {
                self.start_process_resource_role(source, reader, event, tag, is_empty)?;
            }
            tag if is_resource_role_tag(tag) && parent_tag.is_some_and(is_global_task_tag) => {
                self.start_global_task_resource_role(source, reader, event, tag, is_empty)?;
            }
            "resourceParameterBinding"
                if parent_tag.is_some_and(is_resource_role_tag)
                    && self.current_resource_role.is_some() =>
            {
                self.start_resource_parameter_binding(source, reader, event, is_empty)?;
            }
            "resourceAssignmentExpression"
                if parent_tag.is_some_and(is_resource_role_tag)
                    && self.current_resource_role.is_some() =>
            {
                self.start_resource_assignment_expression(source, reader, event, is_empty)?;
            }
            "formalExpression"
                if parent_tag == Some("resourceAssignmentExpression")
                    && self.current_resource_assignment_expression.is_some() =>
            {
                self.attach_resource_assignment_expression_metadata(source, reader, event)?;
            }
            "formalExpression"
                if parent_tag == Some("resourceParameterBinding")
                    && self.current_resource_parameter_binding.is_some() =>
            {
                self.attach_resource_parameter_binding_expression_metadata(source, reader, event)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(super) fn handle_bpmn_di_start_event(
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

    pub(super) fn handle_collaboration_start_event(
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
                self.start_participant(source, reader, event, is_empty)?;
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

    pub(super) fn handle_definitions_start_event(
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
            tag if is_global_task_tag(tag) => {
                self.start_global_task(source, reader, event, tag, is_empty)?;
            }
            "endPoint" => self.capture_end_point(source, reader, event)?,
            "partnerEntity" => self.start_partner_entity(source, reader, event, is_empty)?,
            "partnerRole" => self.start_partner_role(source, reader, event, is_empty)?,
            "resource" => self.start_resource(source, reader, event, is_empty)?,
            "category" => self.start_category(source, reader, event, is_empty)?,
            "correlationProperty" => {
                self.capture_correlation_property(source, reader, event, is_empty)?;
            }
            "error" => self.capture_error(source, reader, event)?,
            "escalation" => self.capture_escalation(source, reader, event)?,
            "signal" => self.capture_signal(source, reader, event)?,
            "dataStore" => self.start_data_store(source, reader, event, is_empty)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
